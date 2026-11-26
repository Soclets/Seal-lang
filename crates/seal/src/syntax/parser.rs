//! Reading lines of tokens into a syntax tree.
//!
//! Blocks have no `:` on their opening line (D9), a body indented one level deeper, and a bare `end` at the opening
//! line's depth, which may carry a checked note naming its block (D20, D102).

use super::lexer::{Gap, Keyword, Line, Symbol, Token, TokenKind, lex};
use super::spare::SpareLists;
use super::tree::{
    BinaryOperator, Block, CaseDeclaration, DefaultDeclaration, Entry, Expression, ExpressionKind, Fallback, Family,
    FieldDeclaration, Handler, HandlerGroup, HandlerUse, Item, LabelDeclaration, Marker, MatchCase, OneOfDeclaration,
    OtherwiseCase, Parameter, PositionKind, Procedure, RecordDeclaration, Script, SlideStep, Statement, StatementKind,
    TypeExpression, UnaryOperator, UnitGroup, UnitMember, UnitName, UnitWritten, Word,
};
use crate::message::{Message, Place};

/// One level of indentation. Seal's own size isn't decided (the standard, chapter 2); every test and example uses
/// Unio's four spaces (Unio D10), so that's what's read, and anything else stops as not designed yet.
const LEVEL: u32 = 4;

/// How deep blocks and expressions may nest. Deeper ones would need more of the host's stack than Seal may safely
/// take (Building rule 1), so they stop as not built yet.
const MAX_BLOCK_DEPTH: u32 = 48;
const MAX_EXPRESSION_DEPTH: u32 = 160;

struct Parser<'a> {
    lines: Vec<Line>,
    /// Every token of the script, which the lines are runs of (D140).
    tokens: Vec<Token<'a>>,
    /// The script as written, for fixes that show a whole line put right.
    source: &'a str,
    line_index: usize,
    /// Where the current line's tokens start and end in `tokens`, kept as the line changes.
    line_start: usize,
    line_end: usize,
    /// The index of the next token in the current line.
    position: usize,
    next_id: u32,
    block_depth: u32,
    expression_depth: u32,
    /// A value already read, with an operator after it, for the next operand to give (D140).
    pending: Option<Expression<'a>>,
    /// Lists kept from scripts read before on this thread, for this one's tree (D140).
    spare: SpareLists,
}

/// What a block's body ran into: the line that isn't part of it.
struct Opener<'a> {
    word: &'static str,
    place: Place,
    indentation: u32,
    /// What the note after its `end` says, when one is written: what the block is, with its name (D102).
    note: Note<'a>,
}

/// An `end`'s note, as what it starts with and the block's name after it, put together only when it's shown (D102).
struct Note<'a> {
    start: &'static str,
    name: std::borrow::Cow<'a, str>,
}

impl<'a> Note<'a> {
    fn new(start: &'static str, name: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        Note { start, name: name.into() }
    }

    /// Whether a written note is this one.
    fn is(&self, written: &str) -> bool {
        written.strip_prefix(self.start) == Some(&*self.name)
    }
}

impl std::fmt::Display for Note<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}{}", self.start, self.name)
    }
}

/// A line indented by anything but 4 spaces a level is refused, with the line indented right as the fix (D63, Unio
/// D10).
fn wrong_indentation(line: &Line, wanted: u32) -> Message {
    Message::refused(
        Place::new(line.place.line, 1, line.indentation.max(1)),
        format!("this line is indented {} spaces, where its block's body is indented {wanted}", line.indentation),
    )
    .with_indentation_fix(wanted)
    .with_why("one level of indentation is 4 spaces.")
}

/// Reads a script's text into its syntax tree.
///
/// # Errors
///
/// Returns the first thing that can't be read, as a message.
pub fn parse(source: &str) -> Result<Script<'_>, Message> {
    let lexed = lex(source)?;
    let mut parser = Parser {
        lines: lexed.lines,
        tokens: lexed.tokens,
        source,
        line_index: 0,
        line_start: 0,
        line_end: 0,
        position: 0,
        next_id: 0,
        block_depth: 0,
        expression_depth: 0,
        pending: None,
        spare: SpareLists::take(),
    };
    parser.enter_line();
    let mut items = parser.spare.items();
    while parser.line_index < parser.lines.len() {
        let indentation = parser.indentation();
        if indentation != 0 {
            return Err(Message::refused(parser.line_place(), "this line is indented, but it isn't inside a block")
                .with_why("only a block's body is indented."));
        }
        items.push(parser.item()?);
    }
    let expression_count = parser.next_id;
    super::lexer::keep(parser.tokens, parser.lines);
    parser.spare.give_back();
    Ok(Script { items, expression_count, source })
}

impl<'a> Parser<'a> {
    // ---- tokens ----

    fn line(&self) -> Option<&Line> {
        self.lines.get(self.line_index)
    }

    fn indentation(&self) -> u32 {
        self.line().map_or(0, |line| line.indentation)
    }

    fn line_place(&self) -> Place {
        self.line().map_or(Place::default(), |line| line.place)
    }

    /// Notes where the current line's tokens are, once it's the current line.
    fn enter_line(&mut self) {
        (self.line_start, self.line_end) = self.line().map_or((0, 0), |line| (line.start as usize, line.end as usize));
        self.position = 0;
    }

    fn peek(&self) -> Option<&Token<'a>> {
        self.peek_at(0)
    }

    fn peek_at(&self, offset: usize) -> Option<&Token<'a>> {
        let index = self.line_start + self.position + offset;
        if index < self.line_end { self.tokens.get(index) } else { None }
    }

    fn peek_kind(&self) -> Option<&TokenKind<'a>> {
        self.peek().map(|token| &token.kind)
    }

    fn at_symbol(&self, symbol: Symbol) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Symbol(found)) if *found == symbol)
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Keyword(found)) if *found == keyword)
    }

    fn first_keyword(&self) -> Option<Keyword> {
        match self.line().and_then(|line| line.tokens(&self.tokens).first()).map(|token| &token.kind) {
            Some(TokenKind::Keyword(keyword)) => Some(*keyword),
            _ => None,
        }
    }

    fn advance(&mut self) {
        if self.peek().is_some() {
            self.position += 1;
        }
    }

    /// Steps past the next token, giving its place.
    fn advance_place(&mut self) -> Place {
        let place = self.peek().map_or(Place::default(), |token| token.place);
        self.advance();
        place
    }

    /// Steps past the next token, taking what it holds instead of copying it: a name's or a value's text is read
    /// once. The token keeps its place, which is all that's looked back at.
    fn take(&mut self) -> Option<Token<'a>> {
        let index = self.line_start + self.position;
        if index >= self.line_end {
            return None;
        }
        let token = self.tokens.get_mut(index)?;
        let kind = std::mem::replace(&mut token.kind, TokenKind::Symbol(Symbol::Comma));
        let whole = Token { kind, place: token.place, gap: token.gap };
        self.position += 1;
        Some(whole)
    }

    /// Puts a token just taken back, for reading it another way.
    fn put_back(&mut self, token: Token<'a>) {
        self.position = self.position.saturating_sub(1);
        if let Some(slot) = self.tokens.get_mut(self.line_start + self.position) {
            *slot = token;
        }
    }

    /// The text of a name or type name about to be read, taken out of its token.
    fn take_text(&mut self) -> &'a str {
        match self.take().map(|token| token.kind) {
            Some(TokenKind::Name(text) | TokenKind::TypeName(text)) => text,
            _ => "",
        }
    }

    /// The place right after the line's last token, for "something is missing here".
    fn end_place(&self) -> Place {
        self.line()
            .and_then(|line| line.tokens(&self.tokens).last())
            .map_or(self.line_place(), |token| Place::new(token.place.line, token.place.column + token.place.width, 1))
    }

    fn here(&self) -> Place {
        self.peek().map_or_else(|| self.end_place(), |token| token.place)
    }

    fn unexpected(&self, wanted: &str) -> Message {
        match self.peek() {
            Some(token) => {
                Message::refused(token.place, format!("{wanted} goes here, but {} was found", token.kind.describe()))
            }
            None => Message::refused(self.end_place(), format!("{wanted} goes here, but the line ends")),
        }
    }

    fn expect_symbol(&mut self, symbol: Symbol) -> Result<Place, Message> {
        if self.at_symbol(symbol) {
            Ok(self.advance_place())
        } else {
            Err(self.unexpected(&format!("`{}`", symbol.text())))
        }
    }

    fn expect_keyword(&mut self, keyword: Keyword) -> Result<Place, Message> {
        if self.at_keyword(keyword) {
            Ok(self.advance_place())
        } else {
            Err(self.unexpected(&format!("`{}`", keyword.word())))
        }
    }

    fn expect_name(&mut self, what: &str) -> Result<Word<'a>, Message> {
        match self.peek() {
            Some(Token { kind: TokenKind::Name(_), place, .. }) => {
                let place = *place;
                Ok(Word { text: self.take_text(), place })
            }
            Some(Token { kind: TokenKind::Keyword(keyword), place, .. }) => {
                // trace: D7
                Err(Message::refused(*place, format!("`{}` is a keyword, so it can't be a name", keyword.word()))
                    .with_why("a name can't be one of Seal's own words."))
            }
            _ => Err(self.unexpected(what)),
        }
    }

    fn expect_type_name(&mut self) -> Result<Word<'a>, Message> {
        match self.peek() {
            Some(Token { kind: TokenKind::TypeName(_), place, .. }) => {
                let place = *place;
                Ok(Word { text: self.take_text(), place })
            }
            Some(Token { kind: TokenKind::Name(name), place, .. }) => {
                // trace: D24
                Err(Message::refused(*place, format!("a type's name starts with a capital, but `{name}` doesn't"))
                    .with_why("a capital always means a type."))
            }
            _ => Err(self.unexpected("a type's name")),
        }
    }

    /// Ends the current line: nothing may follow a statement on its line (D8, Unio D146).
    fn end_line(&mut self) -> Result<(), Message> {
        if let Some(token) = self.peek() {
            return Err(Message::refused(
                token.place,
                format!("{} comes after the end of this line's statement", token.kind.describe()),
            )
            .with_why("each statement goes on a line of its own."));
        }
        self.line_index += 1;
        self.enter_line();
        Ok(())
    }

    fn expression(&mut self, kind: ExpressionKind<'a>, place: Place) -> Expression<'a> {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        Expression { id, kind, place }
    }

    // ---- blocks ----

    /// Reads a body: the lines one level deeper than `opener`, up to the line at the opener's own depth.
    fn body(&mut self, opener: &Opener<'a>) -> Result<Block<'a>, Message> {
        self.block_depth += 1;
        if self.block_depth > MAX_BLOCK_DEPTH {
            return Err(Message::not_built_yet(
                opener.place,
                format!("blocks nested more than {MAX_BLOCK_DEPTH} deep (D20) aren't"),
            ));
        }
        let mut block = self.spare.statements();
        while self.in_body(opener)? {
            block.push(self.statement()?);
        }
        self.block_depth -= 1;
        Ok(block)
    }

    /// Whether the current line belongs to `opener`'s body. A line at a depth that's neither the body's nor the
    /// opener's is an error.
    fn in_body(&self, opener: &Opener<'a>) -> Result<bool, Message> {
        let Some(line) = self.line() else {
            return Err(self.no_end(opener));
        };
        let body = opener.indentation + LEVEL;
        let wrong_end = || {
            // trace: D20
            Message::refused(line.place, "this `end` is at the wrong depth")
                .with_because(format!("line {} opens the block it would close", opener.place.line), opener.place)
                .with_why("an `end` goes at the depth of the line that opened its block.")
        };
        if line.indentation == body {
            if self.first_keyword() == Some(Keyword::End) {
                return Err(wrong_end());
            }
            return Ok(true);
        }
        if line.indentation == opener.indentation {
            return Ok(false);
        }
        if line.indentation < opener.indentation {
            return Err(self.no_end(opener));
        }
        if self.first_keyword() == Some(Keyword::End) {
            return Err(wrong_end());
        }
        Err(wrong_indentation(line, body))
    }

    /// trace: D20
    fn no_end(&self, opener: &Opener<'a>) -> Message {
        let place = self.line().map_or(opener.place, |line| line.place);
        Message::refused(place, format!("the `{}` on line {} has no `end`", opener.word, opener.place.line))
            .with_because(format!("line {} opens it", opener.place.line), opener.place)
            .with_why("every block closes with `end`, at the depth of the line that opened it.")
    }

    /// Reads the `end` that closes `opener`'s block, with its note when one is written (D102).
    fn end_of(&mut self, opener: &Opener<'a>) -> Result<Place, Message> {
        let at_depth = self.line().is_some_and(|line| line.indentation == opener.indentation);
        if !at_depth || self.first_keyword() != Some(Keyword::End) {
            return Err(self.no_end(opener));
        }
        let place = self.expect_keyword(Keyword::End)?;
        if let Some(Token { kind: TokenKind::Note(note), place: note_place, .. }) = self.peek() {
            if !opener.note.is(note) {
                // trace: D102. The note is checked, so it can't go stale.
                return Err(Message::refused(
                    *note_place,
                    format!(
                        "this `end` closes `{}` on line {}, so its note is `\\\\ {}`",
                        opener.note, opener.place.line, opener.note
                    ),
                )
                .with_fix(format!("{}end \\\\ {}", " ".repeat(opener.indentation as usize), opener.note))
                .with_because(format!("line {} opens the block", opener.place.line), opener.place)
                .with_why("an `end`'s note names the block it closes, so it can't go stale."));
            }
            self.advance();
        }
        self.end_line()?;
        Ok(place)
    }

    fn opener(&self, word: &'static str) -> Opener<'a> {
        Opener { word, place: self.line_place(), indentation: self.indentation(), note: Note::new(word, "") }
    }

    /// The script's line as written, by its number.
    fn physical_line(&self, number: u32) -> String {
        let line = self.source.split('\n').nth(number.saturating_sub(1) as usize).unwrap_or("");
        line.strip_suffix('\r').unwrap_or(line).to_owned()
    }

    /// trace: D63, D91. A `///` comment goes only before a declaration.
    fn refuse_doc(&self, place: Place) -> Message {
        let _ = self;
        Message::refused(place, "this `///` has no declaration after it")
            .with_why("a `///` comment describes the declaration on the line after it.")
    }

    // ---- the top level ----

    fn item(&mut self) -> Result<Item<'a>, Message> {
        match self.first_keyword() {
            Some(Keyword::Proc) => Ok(Item::Procedure(self.procedure(false)?)),
            Some(Keyword::Fam) => Ok(Item::Family(self.family()?)),
            Some(Keyword::Record) => Ok(Item::Record(self.record()?)),
            Some(Keyword::OneOf) => Ok(Item::OneOf(self.one_of()?)),
            // trace: D110. `group.handler name` declares a handler group; `group` is read as such only here.
            _ if matches!(self.peek_kind(), Some(TokenKind::Name(word)) if *word == "group")
                && matches!(self.peek_at(1).map(|token| &token.kind), Some(TokenKind::Symbol(Symbol::Dot)))
                && matches!(self.peek_at(2).map(|token| &token.kind), Some(TokenKind::Name(word)) if *word == "handler") =>
            {
                Ok(Item::HandlerGroup(self.handler_group()?))
            }
            // trace: D123. `group.unit name` declares a script's own units.
            _ if matches!(self.peek_kind(), Some(TokenKind::Name(word)) if *word == "group")
                && matches!(self.peek_at(1).map(|token| &token.kind), Some(TokenKind::Symbol(Symbol::Dot)))
                && matches!(self.peek_at(2).map(|token| &token.kind), Some(TokenKind::Name(word)) if *word == "unit") =>
            {
                Ok(Item::UnitGroup(self.unit_group()?))
            }
            _ => {
                let documented = self.line().and_then(|line| line.documented);
                let statement = self.statement()?;
                // trace: D91. A top-level name is a declaration, and can carry a `///` comment.
                if let Some(doc) = documented
                    && !matches!(statement.kind, StatementKind::Name { .. })
                {
                    return Err(self.refuse_doc(doc));
                }
                Ok(Item::Statement(statement))
            }
        }
    }

    fn path(&mut self) -> Result<Vec<Word<'a>>, Message> {
        let mut path = self.spare.words();
        path.push(self.expect_name("a procedure's name")?);
        while self.at_symbol(Symbol::Dot) {
            self.advance();
            path.push(self.expect_name("a narrowing's name")?);
        }
        Ok(path)
    }

    /// A procedure: `proc name[parameters] -> Type ! [failures]` and its body, or a family's member, written the same
    /// way without `proc` (D21, D41, D42, D45, D81, D84, D88, D97).
    #[allow(clippy::too_many_lines)] // the first line, then the `labels` and `defaults` lines, read in order
    fn procedure(&mut self, member: bool) -> Result<Procedure<'a>, Message> {
        let mut opener = self.opener("proc");
        if !member {
            self.expect_keyword(Keyword::Proc)?;
        }
        let path = self.path()?;
        let name: std::borrow::Cow<'a, str> = match path.as_slice() {
            [word] => word.text.into(),
            path => path.iter().map(|word| word.text).collect::<Vec<_>>().join(".").into(),
        };
        // trace: D102. A procedure's note is `proc` and its name; a member's is its own name.
        opener.note = Note::new(if member { "" } else { "proc " }, name.clone());
        if self.at_symbol(Symbol::OpenRound) {
            // trace: Unio D166
            return Err(Message::refused(self.here(), "a procedure's parameters go in `[ ]`, not `( )`")
                .with_why("`( )` calls a procedure; `[ ]` describes one."));
        }
        let parameters = self.parameters(if member { &name } else { "" }, &name)?;
        let result = if self.at_symbol(Symbol::Arrow) {
            self.advance();
            Some(self.type_expression()?)
        } else {
            None
        };
        let failures = if self.at_symbol(Symbol::Bang) {
            self.advance();
            self.expect_symbol(Symbol::OpenSquare)?;
            let mut names = Vec::new();
            loop {
                names.push(self.expect_name("a failure's name")?);
                if self.at_symbol(Symbol::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect_symbol(Symbol::CloseSquare)?;
            Some(names)
        } else {
            None
        };
        self.end_line()?;
        let (labels, labels_any_order, defaults) = self.labels_and_defaults(&opener)?;
        let body = self.body(&opener)?;
        let end_place = self.end_of(&opener)?;
        Ok(Procedure {
            path,
            parameters,
            result,
            failures,
            labels,
            labels_any_order,
            defaults,
            body,
            place: opener.place,
            end_place,
        })
    }

    /// A procedure's or a handler's parameters in `[ ]`, each with its type or in a group sharing one (D21, D60, D81,
    /// D88). `written` is how it's written with no parameters, for the message when `[ ]` is empty.
    fn parameters(&mut self, written: &str, name: &str) -> Result<Vec<Parameter<'a>>, Message> {
        let mut parameters = self.spare.parameters();
        // trace: D60. A procedure that takes nothing has no brackets.
        if self.at_symbol(Symbol::OpenSquare) {
            self.advance();
            if self.at_symbol(Symbol::CloseSquare) {
                let written = if written.is_empty() { format!("proc {name}") } else { written.to_owned() };
                return Err(Message::refused(
                    self.here(),
                    format!("a procedure that takes nothing has no `[ ]`: it's written `{written}`"),
                )
                .with_why("`[ ]` describes the values a procedure takes, and this one takes none."));
            }
            // The names still waiting for the type of their group (D88).
            let mut waiting: Vec<Parameter<'a>> = Vec::new();
            loop {
                // trace: D61
                let unused = self.at_keyword(Keyword::Unused);
                if unused {
                    self.advance();
                }
                let parameter_name = self.expect_name("a parameter's name")?;
                if self.at_symbol(Symbol::Colon) {
                    let colon = self.peek().map_or(Place::default(), |token| token.place);
                    let spaced = matches!(self.peek().map(|token| token.gap), Some(Gap::Spaces(1)));
                    self.advance();
                    let written_type = self.type_expression()?;
                    let group = !waiting.is_empty();
                    if group != spaced {
                        // trace: D88. A group's `:` has a space on each side; a single name's has none before it.
                        let mut fixed: Vec<char> = self.physical_line(colon.line).chars().collect();
                        let at = colon.column.saturating_sub(1) as usize;
                        let what = if group {
                            fixed.insert(at.min(fixed.len()), ' ');
                            "this `:` gives a group of parameters their type, so it has a space on each side"
                        } else {
                            if at > 0 && at <= fixed.len() {
                                fixed.remove(at - 1);
                            }
                            "this `:` gives one parameter its type, so it has no space before it"
                        };
                        return Err(Message::refused(colon, what)
                            .with_fix(fixed.into_iter().collect::<String>())
                            .with_why("parameters sharing a type are grouped as `[hp, amount : Whole]`, with a space on each side of the group's `:`; a single name's type is written `amount: Whole`."));
                    }
                    for mut waiting_parameter in waiting.drain(..) {
                        waiting_parameter.written_type = Some(written_type.clone());
                        parameters.push(waiting_parameter);
                    }
                    parameters.push(Parameter { name: parameter_name, written_type: Some(written_type), unused });
                } else {
                    waiting.push(Parameter { name: parameter_name, written_type: None, unused });
                }
                if self.at_symbol(Symbol::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            if let Some(untyped) = waiting.first() {
                // trace: D81. Every parameter writes its type.
                return Err(Message::refused(untyped.name.place, format!("the parameter `{}` has no type", untyped.name.text))
                    .with_why("a procedure's header writes each parameter's type, as `amount: Whole`, or a group's, as `[hp, amount : Whole]`."));
            }
            self.expect_symbol(Symbol::CloseSquare)?;
        }
        Ok(parameters)
    }

    /// The `labels` and `defaults` lines at the top of a procedure's or a handler's body (D41, D42, D87, D104).
    #[allow(clippy::type_complexity)] // the three things the two lines say, in the order they're written
    fn labels_and_defaults(
        &mut self,
        opener: &Opener<'a>,
    ) -> Result<(Vec<LabelDeclaration<'a>>, bool, Vec<DefaultDeclaration<'a>>), Message> {
        let mut labels = Vec::new();
        let mut defaults = Vec::new();
        let mut labels_any_order = false;
        if self.in_body(opener)? && self.first_keyword() == Some(Keyword::Labels) {
            self.advance();
            // trace: D87. `labels.any_order` lets the labelled arguments go in any order.
            if self.at_symbol(Symbol::Dot) {
                self.advance();
                match self.peek_kind() {
                    Some(TokenKind::Name(narrowing)) if *narrowing == "any_order" => {
                        self.advance();
                        labels_any_order = true;
                    }
                    _ => {
                        let found = self.here();
                        return Err(Message::refused(found, "`labels` has one narrowing, `labels.any_order`")
                            .with_why(
                                "`labels.any_order [ … ]` lets a call write the labelled arguments in any order.",
                            ));
                    }
                }
            }
            self.expect_symbol(Symbol::OpenSquare)?;
            loop {
                let parameter = self.expect_name("a parameter's name")?;
                self.expect_keyword(Keyword::As)?;
                let label = self.expect_name("a label")?;
                labels.push(LabelDeclaration { parameter, label });
                if self.at_symbol(Symbol::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect_symbol(Symbol::CloseSquare)?;
            self.end_line()?;
            if self.in_body(opener)? && self.first_keyword() == Some(Keyword::Defaults) {
                self.advance();
                self.expect_symbol(Symbol::OpenSquare)?;
                loop {
                    let label = self.expect_name("a label")?;
                    self.expect_symbol(Symbol::Colon)?;
                    let value = self.full_expression()?;
                    defaults.push(DefaultDeclaration { label, value });
                    if self.at_symbol(Symbol::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect_symbol(Symbol::CloseSquare)?;
                self.end_line()?;
            }
        }
        Ok((labels, labels_any_order, defaults))
    }

    /// `group.handler name`: an optional `handles [ … ]` line, then its members, each `::name` with its parameters
    /// and its cases (D97, D110, Unio D42, D44).
    fn handler_group(&mut self) -> Result<HandlerGroup<'a>, Message> {
        let mut opener = self.opener("group.handler");
        self.advance();
        self.expect_symbol(Symbol::Dot)?;
        match self.peek_kind() {
            Some(TokenKind::Name(kind)) if *kind == "handler" => {
                self.advance();
            }
            _ => {
                return Err(Message::refused(self.here(), "a group is `group.handler`, a named set of handlers")
                    .with_why("`group.handler name` declares a handler group; Seal has no other kind of group yet."));
            }
        }
        let name = self.expect_name("the group's name")?;
        opener.note = Note::new("group.handler ", name.text);
        self.end_line()?;
        let mut handles = Vec::new();
        if self.in_body(&opener)? && matches!(self.peek_kind(), Some(TokenKind::Name(word)) if *word == "handles") {
            self.advance();
            self.expect_symbol(Symbol::OpenSquare)?;
            loop {
                // trace: D129. A built-in, as `be.Whole`, can be named too.
                if self.at_keyword(Keyword::Be) {
                    let be = self.advance_place();
                    let mut path = vec![Word { text: "be", place: be }];
                    while self.at_symbol(Symbol::Dot) {
                        self.advance();
                        path.push(self.member_name()?);
                    }
                    handles.push(path);
                } else {
                    handles.push(self.path()?);
                }
                if self.at_symbol(Symbol::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect_symbol(Symbol::CloseSquare)?;
            self.end_line()?;
        }
        let mut members = Vec::new();
        while self.in_body(&opener)? {
            if !self.at_symbol(Symbol::Path) {
                return Err(Message::refused(self.line_place(), "a handler group holds only handlers, each `::name`")
                    .with_why(
                        "a handler group's members are written with the `::` they're reached with, as `::start_over`.",
                    ));
            }
            self.block_depth += 1;
            members.push(self.handler()?);
            self.block_depth -= 1;
        }
        self.end_of(&opener)?;
        Ok(HandlerGroup { name, handles, members, place: opener.place })
    }

    /// One handler: `::name[parameters]`, its `labels` and `defaults`, and its cases (D110, Unio D42, D44).
    fn handler(&mut self) -> Result<Handler<'a>, Message> {
        let mut opener = self.opener("::");
        self.expect_symbol(Symbol::Path)?;
        let name = self.expect_name("the handler's name")?;
        opener.note = Note::new("::", name.text);
        let parameters = self.parameters(&format!("::{}", name.text), name.text)?;
        self.end_line()?;
        let (labels, labels_any_order, defaults) = self.labels_and_defaults(&opener)?;
        let mut cases = Vec::new();
        while self.in_body(&opener)? {
            let case_opener = self.opener("case");
            self.expect_keyword(Keyword::Case)?;
            let failure = self.expect_name("a failure's name")?;
            self.end_line()?;
            let body = self.case_body(&case_opener, &opener)?;
            cases.push(OtherwiseCase { failure, body });
        }
        self.end_of(&opener)?;
        Ok(Handler { name, parameters, labels, labels_any_order, defaults, cases, place: opener.place })
    }

    /// A handler where it's used, after `??` or `include`: `::name`, `group::name`, `*` before a name that can jump,
    /// and the arguments in `( )` (D98, D110, Unio D42, D44).
    fn handler_use(&mut self) -> Result<HandlerUse<'a>, Message> {
        let place = self.here();
        let group = if matches!(self.peek_kind(), Some(TokenKind::Name(_))) {
            Some(self.expect_name("a handler group's name")?)
        } else {
            None
        };
        self.expect_symbol(Symbol::Path)?;
        let jumps = self.at_symbol(Symbol::Star);
        if jumps {
            self.advance();
        }
        let name = self.expect_name("a handler's name")?;
        let arguments = if self.at_symbol(Symbol::OpenRound)
            && matches!(self.peek().map(|token| token.gap), Some(Gap::Spaces(0)))
        {
            self.advance();
            self.entries(Symbol::CloseRound)?
        } else {
            Vec::new()
        };
        Ok(HandlerUse { group, name, jumps, arguments, place: place.to(self.previous_place()) })
    }

    /// trace: D123. `group.unit name`, then each member: `::name base`, or `::name = amount unit`.
    fn unit_group(&mut self) -> Result<UnitGroup<'a>, Message> {
        let mut opener = self.opener("group.unit");
        self.advance();
        self.expect_symbol(Symbol::Dot)?;
        self.advance();
        let name = self.expect_name("the unit group's name")?;
        opener.note = Note::new("group.unit ", name.text);
        self.end_line()?;
        let mut members = Vec::new();
        while self.in_body(&opener)? {
            if !self.at_symbol(Symbol::Path) {
                return Err(Message::refused(self.line_place(), "a unit group holds only units, each `::name`")
                    .with_why(
                        "a unit group's members are written with the `::` they're reached with, as `::cents base`.",
                    ));
            }
            self.advance();
            let member = self.expect_name("a unit's name")?;
            let size = if matches!(self.peek_kind(), Some(TokenKind::Name(word)) if *word == "base") {
                self.advance();
                None
            } else {
                self.expect_symbol(Symbol::Equals)?;
                // `100 cents` reads as a number with its unit, which is what it is.
                let amount = self.unary()?;
                match amount.kind {
                    ExpressionKind::Measured { number, unit } if unit.per.is_none() && !unit.point => {
                        Some((*number, unit.top))
                    }
                    _ => {
                        let unit = self.unit_name()?;
                        Some((amount, unit))
                    }
                }
            };
            self.end_line()?;
            members.push(UnitMember { name: member, size });
        }
        self.end_of(&opener)?;
        Ok(UnitGroup { name, members, place: opener.place })
    }

    /// A unit's name, with its group before `::` when it's written (D112).
    fn unit_name(&mut self) -> Result<UnitName<'a>, Message> {
        let first = self.expect_name("a unit")?;
        if self.at_symbol(Symbol::Path) {
            self.advance();
            let name = self.expect_name("a unit")?;
            return Ok(UnitName { group: Some(first), name });
        }
        Ok(UnitName { group: None, name: first })
    }

    /// A unit as written: its name, `.point` when it's a point, and `per` and another unit for a rate (D112, D120,
    /// D121).
    fn unit_written(&mut self) -> Result<UnitWritten<'a>, Message> {
        let start = self.here();
        let top = self.unit_name()?;
        let mut point = false;
        if self.at_symbol(Symbol::Dot)
            && matches!(self.peek_at(1).map(|token| &token.kind), Some(TokenKind::Name(word)) if *word == "point")
        {
            self.advance();
            self.advance();
            point = true;
        }
        let per = if matches!(self.peek_kind(), Some(TokenKind::Name(word)) if *word == "per") {
            self.advance();
            Some(self.unit_name()?)
        } else {
            None
        };
        Ok(UnitWritten { top, point, per, place: start.to(self.previous_place()) })
    }

    /// Whether a handler is written here: `::name`, or a group's name and `::`.
    fn at_handler(&self) -> bool {
        self.at_symbol(Symbol::Path)
            || (matches!(self.peek_kind(), Some(TokenKind::Name(_)))
                && matches!(self.peek_at(1).map(|token| &token.kind), Some(TokenKind::Symbol(Symbol::Path))))
    }

    /// trace: D43, D97
    fn family(&mut self) -> Result<Family<'a>, Message> {
        let mut opener = self.opener("fam");
        self.expect_keyword(Keyword::Fam)?;
        self.expect_symbol(Symbol::Dot)?;
        if !self.at_keyword(Keyword::Proc) {
            if matches!(self.peek_kind(), Some(TokenKind::Name(name)) if *name == "handler") {
                // trace: D110. Handlers live in a `group.handler`, since they aren't kinds of a word.
                return Err(Message::refused(self.here(), "handlers live in a `group.handler`, not a family")
                    .with_why("a family is a word and its kinds; a handler group is a named set, reached with `::`."));
            }
            return Err(self.unexpected("`proc`"));
        }
        self.advance();
        let word = self.expect_name("the family's word")?;
        opener.note = Note::new("fam.proc ", word.text);
        self.end_line()?;
        let mut members = Vec::new();
        while self.in_body(&opener)? {
            if self.first_keyword() == Some(Keyword::Proc) {
                // trace: D97. The family's first line already says its members are procedures.
                let place = self.line_place();
                let line = self.physical_line(place.line);
                let fixed = line.replacen("proc ", "", 1);
                return Err(Message::refused(place, "a family's member is written without `proc`")
                    .with_fix(fixed)
                    .with_why("the family's first line, `fam.proc`, already says its members are procedures."));
            }
            if !matches!(self.peek_kind(), Some(TokenKind::Name(_))) {
                return Err(Message::refused(self.line_place(), "a `fam.proc` block holds only procedures")
                    .with_why("a family holds only the kind of thing it's declared for."));
            }
            self.block_depth += 1;
            members.push(self.procedure(true)?);
            self.block_depth -= 1;
        }
        self.end_of(&opener)?;
        Ok(Family { word, members, place: opener.place })
    }

    fn field(&mut self) -> Result<FieldDeclaration<'a>, Message> {
        let name = self.expect_name("a field's name")?;
        self.expect_symbol(Symbol::Colon)?;
        let written_type = self.type_expression()?;
        Ok(FieldDeclaration { name, written_type })
    }

    /// trace: D27
    fn record(&mut self) -> Result<RecordDeclaration<'a>, Message> {
        let mut opener = self.opener("record");
        self.expect_keyword(Keyword::Record)?;
        let name = self.expect_type_name()?;
        opener.note = Note::new("record ", name.text);
        self.end_line()?;
        let mut fields = Vec::new();
        while self.in_body(&opener)? {
            fields.push(self.field()?);
            self.end_line()?;
        }
        self.end_of(&opener)?;
        Ok(RecordDeclaration { name, fields })
    }

    /// trace: D35, D36
    fn one_of(&mut self) -> Result<OneOfDeclaration<'a>, Message> {
        let mut opener = self.opener("one_of");
        self.expect_keyword(Keyword::OneOf)?;
        let name = self.expect_type_name()?;
        opener.note = Note::new("one_of ", name.text);
        self.end_line()?;
        let mut cases = Vec::new();
        while self.in_body(&opener)? {
            let case_name = self.expect_name("a case's name")?;
            let mut fields = Vec::new();
            if self.at_symbol(Symbol::OpenSquare) {
                self.advance();
                loop {
                    fields.push(self.field()?);
                    if self.at_symbol(Symbol::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect_symbol(Symbol::CloseSquare)?;
            }
            self.end_line()?;
            cases.push(CaseDeclaration { name: case_name, fields });
        }
        self.end_of(&opener)?;
        Ok(OneOfDeclaration { name, cases })
    }

    /// trace: D30, D31, Unio D167
    fn type_expression(&mut self) -> Result<TypeExpression<'a>, Message> {
        self.expression_depth += 1;
        if self.expression_depth > MAX_EXPRESSION_DEPTH {
            return Err(self.too_deep());
        }
        let name = self.expect_type_name()?;
        let mut place = name.place;
        let mut blanks = Vec::new();
        if self.at_symbol(Symbol::TypeOpen) {
            self.advance();
            loop {
                let label = match (self.peek_kind(), self.peek_at(1).map(|token| &token.kind)) {
                    (Some(TokenKind::Name(_)), Some(TokenKind::Symbol(Symbol::Colon))) => {
                        let label = self.expect_name("a label")?;
                        self.advance();
                        Some(label)
                    }
                    _ => None,
                };
                blanks.push((label, self.type_expression()?));
                if self.at_symbol(Symbol::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            if !self.at_symbol(Symbol::TypeClose) {
                return Err(self.unexpected("`>`"));
            }
            place = place.to(self.here());
            self.advance();
        }
        let mut settings = Vec::new();
        if self.at_symbol(Symbol::OpenSquare) && matches!(self.peek().map(|token| token.gap), Some(Gap::Spaces(0))) {
            self.advance();
            loop {
                let mut path = vec![self.expect_name("a setting's name")?];
                while self.at_symbol(Symbol::Dot) {
                    self.advance();
                    path.push(self.expect_name("a setting's name")?);
                }
                self.expect_symbol(Symbol::Colon)?;
                if let [word] = path.as_slice()
                    && word.text == "unit"
                {
                    // trace: D112. A number's unit, as `Float[unit: px per secs]`.
                    let unit = self.unit_written()?;
                    let place = unit.place;
                    let value = self.expression(ExpressionKind::Unit(Box::new(unit)), place);
                    settings.push((path, value));
                } else {
                    // A ranged number's edge can be below zero (D107).
                    settings.push((path, self.unary()?));
                }
                if self.at_symbol(Symbol::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            place = place.to(self.here());
            self.expect_symbol(Symbol::CloseSquare)?;
        }
        self.expression_depth -= 1;
        Ok(TypeExpression { name, blanks, settings, place })
    }

    // ---- statements ----

    #[allow(clippy::too_many_lines)] // one case per kind of statement
    fn statement(&mut self) -> Result<Statement<'a>, Message> {
        let place = self.line_place();
        if self.block_depth > 0
            && let Some(doc) = self.line().and_then(|line| line.documented)
        {
            // trace: D91. Inside a block, only a name could be a declaration, and none of D91's are made there.
            let makes_a_name = matches!(self.first_keyword(), Some(Keyword::Mut | Keyword::Unused))
                || matches!(
                    (self.peek_kind(), self.peek_at(1).map(|token| &token.kind)),
                    (Some(TokenKind::Name(_)), Some(TokenKind::Symbol(Symbol::Equals | Symbol::Colon)))
                );
            if makes_a_name {
                return Err(Message::not_designed_yet(doc, "a `///` before a name made inside a block"));
            }
            return Err(self.refuse_doc(doc));
        }
        let kind = match self.first_keyword() {
            Some(keyword @ (Keyword::Mut | Keyword::Unused)) => {
                self.advance();
                let marker = if keyword == Keyword::Mut { Marker::Mut } else { Marker::Unused };
                let name = self.expect_name("a name")?;
                let written_type = if self.at_symbol(Symbol::Colon) {
                    self.advance();
                    Some(Box::new(self.type_expression()?))
                } else {
                    None
                };
                self.expect_symbol(Symbol::Equals)?;
                let value = self.value_with_otherwise()?;
                StatementKind::Name { marker, name, written_type, value }
            }
            Some(Keyword::If) => return self.if_statement(),
            Some(Keyword::While) => {
                let opener = self.opener("while");
                self.advance();
                let condition = self.full_expression()?;
                self.end_line()?;
                let body = self.body(&opener)?;
                self.end_of(&opener)?;
                return Ok(Statement { kind: StatementKind::While { condition, body }, place });
            }
            Some(Keyword::ForEach) => return self.for_each(),
            Some(Keyword::Match) => return self.match_statement(),
            Some(Keyword::Break) => {
                self.advance();
                self.end_line()?;
                return Ok(Statement { kind: StatementKind::Break, place });
            }
            Some(Keyword::Return) => {
                self.advance();
                if self.peek().is_none() {
                    self.end_line()?;
                    return Ok(Statement { kind: StatementKind::Return(None), place });
                }
                let value = self.value_with_otherwise()?;
                return Ok(Statement { kind: StatementKind::Return(Some(value)), place });
            }
            Some(Keyword::Fail) => {
                self.advance();
                let name = self.expect_name("a failure's name")?;
                self.end_line()?;
                return Ok(Statement { kind: StatementKind::Fail(name), place });
            }
            Some(Keyword::Instead) => {
                self.advance();
                let value = self.full_expression()?;
                self.end_line()?;
                return Ok(Statement { kind: StatementKind::Use(value), place });
            }
            Some(keyword @ (Keyword::Proc | Keyword::Fam | Keyword::Record | Keyword::OneOf)) => {
                // trace: D93
                return Err(Message::refused(
                    place,
                    format!("a `{}` is declared at the top level, not inside a block", keyword.word()),
                )
                .with_why("procedures, families, records and `one_of`s are declared at a script's top level only."));
            }
            Some(keyword @ (Keyword::Labels | Keyword::Defaults)) => {
                // trace: D41, D42
                return Err(Message::refused(
                    place,
                    format!("a `{}` line goes at the top of a procedure's body", keyword.word()),
                )
                .with_why("`labels` is the first line of a procedure's body, and `defaults` comes right after it."));
            }
            Some(keyword @ (Keyword::Elif | Keyword::Else | Keyword::Case | Keyword::End)) => {
                return Err(Message::refused(place, format!("this `{}` has no block to belong to", keyword.word())));
            }
            _ => {
                let target = self.full_expression()?;
                if self.at_symbol(Symbol::Colon) || self.at_symbol(Symbol::Equals) {
                    let written_type = if self.at_symbol(Symbol::Colon) {
                        self.advance();
                        Some(Box::new(self.type_expression()?))
                    } else {
                        None
                    };
                    self.expect_symbol(Symbol::Equals)?;
                    let value = self.value_with_otherwise()?;
                    match target.kind {
                        ExpressionKind::Name(text) => StatementKind::Name {
                            marker: Marker::None,
                            name: Word { text, place: target.place },
                            written_type,
                            value,
                        },
                        ExpressionKind::Member { .. } | ExpressionKind::Bracketed { .. } if written_type.is_none() => {
                            StatementKind::Change { place: target, value }
                        }
                        _ => {
                            return Err(Message::refused(target.place, "this can't be given a value")
                                .with_why("`=` gives a value to a name, a field or an item."));
                        }
                    }
                } else if self.at_keyword(Keyword::Otherwise) {
                    let call = self.otherwise(target)?;
                    return Ok(Statement { kind: StatementKind::Expression(call), place });
                } else {
                    self.end_line()?;
                    return Ok(Statement { kind: StatementKind::Expression(target), place });
                }
            }
        };
        Ok(Statement { kind, place })
    }

    /// A statement's value, which may carry an `otherwise` block (Unio D40). Ends the line either way.
    fn value_with_otherwise(&mut self) -> Result<Expression<'a>, Message> {
        let value = self.full_expression()?;
        if self.at_keyword(Keyword::Otherwise) {
            self.otherwise(value)
        } else {
            self.end_line()?;
            Ok(value)
        }
    }

    /// trace: D37, Unio D40
    fn otherwise(&mut self, call: Expression<'a>) -> Result<Expression<'a>, Message> {
        let opener = self.opener("otherwise");
        let place = call.place;
        let at = self.expect_keyword(Keyword::Otherwise)?;
        if self.peek().is_some() {
            // trace: D98. `otherwise` is only ever a block; on one line a handler follows `??`.
            let line = self.physical_line(at.line);
            let start = at.column.saturating_sub(1) as usize;
            let fixed: String =
                line.chars().take(start).chain("??".chars()).chain(line.chars().skip(start + 9)).collect();
            return Err(Message::refused(at, "`otherwise` opens a block of cases; on one line, a handler follows `??`")
                .with_fix(fixed)
                .with_why("`??` deals with a failure on one line, with a value, `ignore`, `crash` or a handler; `otherwise` is only ever a block."));
        }
        self.end_line()?;
        let mut cases = Vec::new();
        let mut includes = Vec::new();
        while self.in_body(&opener)? {
            // trace: Unio D43. `include` brings a handler's cases in beside the block's own.
            if matches!(self.peek_kind(), Some(TokenKind::Name(word)) if *word == "include")
                && matches!(
                    self.peek_at(1).map(|token| &token.kind),
                    Some(TokenKind::Symbol(Symbol::Path) | TokenKind::Name(_))
                )
            {
                self.advance();
                includes.push(self.handler_use()?);
                self.end_line()?;
                continue;
            }
            let case_opener = self.opener("case");
            self.expect_keyword(Keyword::Case)?;
            let failure = self.expect_name("a failure's name")?;
            self.end_line()?;
            let body = self.case_body(&case_opener, &opener)?;
            cases.push(OtherwiseCase { failure, body });
        }
        self.end_of(&opener)?;
        Ok(self.expression(ExpressionKind::Otherwise { call: Box::new(call), cases, includes }, place))
    }

    /// A case's body: it runs until the next `case` or the block's `end` (D37).
    fn case_body(&mut self, case: &Opener<'a>, block: &Opener<'a>) -> Result<Block<'a>, Message> {
        self.block_depth += 1;
        if self.block_depth > MAX_BLOCK_DEPTH {
            return Err(Message::not_built_yet(
                case.place,
                format!("blocks nested more than {MAX_BLOCK_DEPTH} deep (D20) aren't"),
            ));
        }
        let mut body = Vec::new();
        loop {
            let Some(line) = self.line() else {
                return Err(self.no_end(block));
            };
            if line.indentation <= case.indentation {
                break;
            }
            if line.indentation != case.indentation + LEVEL {
                return Err(wrong_indentation(line, case.indentation + LEVEL));
            }
            body.push(self.statement()?);
        }
        self.block_depth -= 1;
        Ok(body)
    }

    /// trace: D9, D10, D20
    fn if_statement(&mut self) -> Result<Statement<'a>, Message> {
        let opener = self.opener("if");
        self.expect_keyword(Keyword::If)?;
        let mut branches = Vec::new();
        let condition = self.full_expression()?;
        self.end_line()?;
        branches.push((condition, self.body(&opener)?));
        let mut otherwise = None;
        loop {
            if self.line().is_none() {
                return Err(self.no_end(&opener));
            }
            match self.first_keyword() {
                Some(Keyword::Elif) => {
                    self.advance();
                    let condition = self.full_expression()?;
                    self.end_line()?;
                    branches.push((condition, self.body(&opener)?));
                }
                Some(Keyword::Else) => {
                    self.advance();
                    if self.at_keyword(Keyword::If) {
                        // trace: D10, D40
                        return Err(Message::refused(self.here(), "Seal writes `elif`, not `else if`")
                            .with_why("`elif` is short for \"else if\"."));
                    }
                    self.end_line()?;
                    otherwise = Some(self.body(&opener)?);
                    break;
                }
                _ => break,
            }
        }
        self.end_of(&opener)?;
        Ok(Statement { kind: StatementKind::If { branches, otherwise }, place: opener.place })
    }

    /// trace: D22, D33, Unio D171, D174
    fn for_each(&mut self) -> Result<Statement<'a>, Message> {
        let opener = self.opener("for_each");
        self.expect_keyword(Keyword::ForEach)?;
        // trace: D61
        let item_unused = self.at_keyword(Keyword::Unused);
        if item_unused {
            self.advance();
        }
        let item = self.expect_name("the item's name")?;
        let mut position_unused = false;
        let position = if self.at_keyword(Keyword::At) {
            self.advance();
            let kind = match self.peek_kind() {
                Some(TokenKind::Name(name)) if *name == "pos" => PositionKind::Pos,
                Some(TokenKind::Name(name)) if *name == "offset" => PositionKind::Offset,
                Some(TokenKind::Name(name)) if *name == "key" => PositionKind::Key,
                _ => return Err(self.unexpected("`pos`, `offset` or `key`")),
            };
            self.advance();
            // trace: D86. An `at` name that's never read is marked `unused`.
            if self.at_keyword(Keyword::Unused) {
                self.advance();
                position_unused = true;
            }
            Some((kind, self.expect_name("a name")?))
        } else {
            None
        };
        self.expect_keyword(Keyword::In)?;
        let source = self.full_expression()?;
        self.end_line()?;
        let body = self.body(&opener)?;
        self.end_of(&opener)?;
        Ok(Statement {
            kind: StatementKind::ForEach { item, item_unused, position, position_unused, source, body },
            place: opener.place,
        })
    }

    /// trace: D37
    fn match_statement(&mut self) -> Result<Statement<'a>, Message> {
        let opener = self.opener("match");
        self.expect_keyword(Keyword::Match)?;
        let subject = self.full_expression()?;
        self.end_line()?;
        let mut cases = Vec::new();
        while self.in_body(&opener)? {
            let case_opener = self.opener("case");
            self.expect_keyword(Keyword::Case)?;
            let name = self.expect_name("a case's name")?;
            let mut values = Vec::new();
            if self.at_symbol(Symbol::OpenSquare) {
                self.advance();
                loop {
                    values.push(self.expect_name("a name for one of the case's values")?);
                    if self.at_symbol(Symbol::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect_symbol(Symbol::CloseSquare)?;
            }
            self.end_line()?;
            let body = self.case_body(&case_opener, &opener)?;
            cases.push(MatchCase { name, values, body });
        }
        self.end_of(&opener)?;
        Ok(Statement { kind: StatementKind::Match { subject, cases }, place: opener.place })
    }

    // ---- expressions ----

    fn too_deep(&self) -> Message {
        Message::not_built_yet(
            self.here(),
            format!("values nested more than {MAX_EXPRESSION_DEPTH} deep in one statement aren't"),
        )
    }

    fn deeper(&mut self) -> Result<(), Message> {
        self.expression_depth += 1;
        if self.expression_depth > MAX_EXPRESSION_DEPTH { Err(self.too_deep()) } else { Ok(()) }
    }

    /// A whole expression: slides come last, taking everything on their left (Unio D73).
    fn full_expression(&mut self) -> Result<Expression<'a>, Message> {
        let saved = self.expression_depth;
        // Most values are one operand with no operator after it, so the operand is read first, and the operators'
        // levels, from `or` down, are gone through only when an operator follows: each level gives back what it reads
        // unchanged when its operator isn't next, so this reads the same (D140).
        let start = if self.at_keyword(Keyword::Not) || self.at_symbol(Symbol::Negate) {
            self.or_expression()?
        } else {
            let operand = self.operand()?;
            if self.at_operator() {
                self.pending = Some(operand);
                self.or_expression()?
            } else {
                operand
            }
        };
        let result = if self.at_symbol(Symbol::Slide) { self.slide(start) } else { Ok(start) };
        self.expression_depth = saved;
        result
    }

    /// trace: Unio D5, D73, D81
    fn slide(&mut self, start: Expression<'a>) -> Result<Expression<'a>, Message> {
        let place = start.place;
        let mut steps = Vec::new();
        while self.at_symbol(Symbol::Slide) {
            self.advance();
            let step_place = self.here();
            let mut target = match self.peek_kind() {
                Some(TokenKind::Name(name)) => {
                    let name = *name;
                    self.advance();
                    self.expression(ExpressionKind::Name(name), step_place)
                }
                Some(TokenKind::Keyword(Keyword::Be)) => {
                    self.advance();
                    self.expression(ExpressionKind::Name("be"), step_place)
                }
                _ => return Err(self.unexpected("a procedure's name")),
            };
            while self.at_symbol(Symbol::Dot) {
                self.advance();
                let name = self.member_name()?;
                let member_place = target.place.to(name.place);
                target = self.expression(ExpressionKind::Member { receiver: Box::new(target), name }, member_place);
            }
            let arguments = if self.at_symbol(Symbol::OpenRound) {
                self.advance();
                Some(self.entries(Symbol::CloseRound)?)
            } else {
                None
            };
            steps.push(SlideStep { target, arguments, place: step_place });
        }
        if let Some(token) = self.peek()
            && !matches!(token.kind, TokenKind::Keyword(Keyword::Otherwise))
            && !matches!(token.kind, TokenKind::Symbol(Symbol::CloseRound | Symbol::CloseSquare | Symbol::Comma))
        {
            // trace: Unio D81
            return Err(Message::refused(
                token.place,
                format!("{} comes after a slide's last step", token.kind.describe()),
            )
            .with_why("a slide's step is a procedure, so anything after the slide needs the slide in `( )`."));
        }
        Ok(self.expression(ExpressionKind::Slide { start: Box::new(start), steps }, place))
    }

    /// Whether an operator that one of the operators' levels reads is next, after an operand.
    fn at_operator(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(
                TokenKind::Keyword(
                    Keyword::Or
                        | Keyword::And
                        | Keyword::Eq
                        | Keyword::NotEq
                        | Keyword::AtLeast
                        | Keyword::AtMost
                        | Keyword::Div
                        | Keyword::Rem
                ) | TokenKind::Symbol(
                    Symbol::Less
                        | Symbol::Greater
                        | Symbol::Ampersand
                        | Symbol::Plus
                        | Symbol::Minus
                        | Symbol::Star
                        | Symbol::Slash
                        | Symbol::Power
                ) | TokenKind::Name("to")
            )
        )
    }

    fn binary(
        &mut self,
        operator: BinaryOperator,
        left: Expression<'a>,
        right: Expression<'a>,
        at: Place,
    ) -> Expression<'a> {
        let place = left.place.to(right.place);
        self.expression(
            ExpressionKind::Binary { operator, left: Box::new(left), right: Box::new(right), operator_place: at },
            place,
        )
    }

    fn or_expression(&mut self) -> Result<Expression<'a>, Message> {
        let mut left = self.and_expression()?;
        while self.at_keyword(Keyword::Or) {
            self.deeper()?;
            let at = self.here();
            self.advance();
            let right = self.and_expression()?;
            left = self.binary(BinaryOperator::Or, left, right, at);
        }
        Ok(left)
    }

    fn and_expression(&mut self) -> Result<Expression<'a>, Message> {
        let mut left = self.not_expression()?;
        while self.at_keyword(Keyword::And) {
            self.deeper()?;
            let at = self.here();
            self.advance();
            let right = self.not_expression()?;
            left = self.binary(BinaryOperator::And, left, right, at);
        }
        Ok(left)
    }

    /// trace: Unio D34: `not` comes after comparisons and before `and`.
    fn not_expression(&mut self) -> Result<Expression<'a>, Message> {
        if self.pending.is_none() && self.at_keyword(Keyword::Not) {
            self.deeper()?;
            let place = self.here();
            self.advance();
            let operand = self.not_expression()?;
            let whole = place.to(operand.place);
            return Ok(self.expression(
                ExpressionKind::Unary { operator: UnaryOperator::Not, operand: Box::new(operand) },
                whole,
            ));
        }
        self.comparison()
    }

    fn comparison_operator(&self) -> Option<BinaryOperator> {
        match self.peek_kind()? {
            // trace: D158
            TokenKind::Keyword(Keyword::Eq) => Some(BinaryOperator::Equal),
            TokenKind::Keyword(Keyword::NotEq) => Some(BinaryOperator::NotEqual),
            TokenKind::Symbol(Symbol::Less) => Some(BinaryOperator::Less),
            TokenKind::Symbol(Symbol::Greater) => Some(BinaryOperator::Greater),
            TokenKind::Keyword(Keyword::AtLeast) => Some(BinaryOperator::AtLeast),
            TokenKind::Keyword(Keyword::AtMost) => Some(BinaryOperator::AtMost),
            _ => None,
        }
    }

    fn comparison(&mut self) -> Result<Expression<'a>, Message> {
        let left = self.measure_to()?;
        let Some(operator) = self.comparison_operator() else {
            return Ok(left);
        };
        self.deeper()?;
        let at = self.here();
        self.advance();
        let right = self.measure_to()?;
        if self.comparison_operator().is_some() {
            // trace: Unio D81
            return Err(Message::refused(self.here(), "two comparisons in a row")
                .with_why("each comparison stands alone; join two with `and`, as in `1 < x and x < 10`."));
        }
        Ok(self.binary(operator, left, right, at))
    }

    /// trace: D122, D148. `value to unit` measures a value in a unit of its group: a bare `to` between a value and a
    /// unit's name, where the label `to:` never stands. Next to maths written without brackets, which part it
    /// measures isn't decided, so it stops.
    fn measure_to(&mut self) -> Result<Expression<'a>, Message> {
        let value = self.join()?;
        if !(matches!(self.peek_kind(), Some(TokenKind::Name("to")))
            && matches!(self.peek_at(1).map(|token| &token.kind), Some(TokenKind::Name(_))))
        {
            return Ok(value);
        }
        // trace: D149, Unio D72. `to` has no place in maths' order, so maths it measures is bracketed.
        if matches!(&value.kind, ExpressionKind::Binary { operator, .. } if operator.is_maths()) {
            let mut message = Message::refused(value.place, "this is maths measured with `to`, without brackets")
                .with_why(
                    "`to` has no place in maths' order, so the maths it measures is in brackets, as `(a + b) to ms`.",
                );
            // The fix brackets the maths, when it's all on one line.
            let line: Vec<char> = self.physical_line(value.place.line).chars().collect();
            let from = value.place.column.saturating_sub(1) as usize;
            let to = (value.place.column + value.place.width).saturating_sub(1) as usize;
            if from < to && to <= line.len() {
                let fixed: String = line.get(..from).unwrap_or(&[]).iter().collect::<String>()
                    + "("
                    + &line.get(from..to).unwrap_or(&[]).iter().collect::<String>()
                    + ")"
                    + &line.get(to..).unwrap_or(&[]).iter().collect::<String>();
                message = message.with_fix(fixed);
            }
            return Err(message);
        }
        self.advance();
        let unit = self.unit_written()?;
        let place = value.place.to(unit.place);
        Ok(self.expression(ExpressionKind::MeasureTo { value: Box::new(value), unit: Box::new(unit) }, place))
    }

    /// trace: D77. Text is joined only by filling it in with braces, so `&` is refused, with the braces form as the
    /// fix when the pieces are texts and names.
    fn join(&mut self) -> Result<Expression<'a>, Message> {
        let first = self.additive()?;
        if !self.at_symbol(Symbol::Ampersand) {
            return Ok(first);
        }
        let at = self.here();
        let mut pieces = vec![first];
        while self.at_symbol(Symbol::Ampersand) {
            self.deeper()?;
            self.advance();
            pieces.push(self.additive()?);
        }
        let mut message = Message::refused(at, "`&` doesn't join text; text is joined by filling it in with braces")
            .with_why("text is joined only with braces, as in `\"Score: {score}\"`, which looks like what it shows.");
        let mut filled = String::new();
        let mut fits = true;
        for piece in &pieces {
            match &piece.kind {
                ExpressionKind::Text(text) => filled.push_str(text),
                _ => match braced(piece) {
                    Some(name) => {
                        filled.push('{');
                        filled.push_str(&name);
                        filled.push('}');
                    }
                    None => fits = false,
                },
            }
        }
        let (start, end) = (pieces.first().map(|piece| piece.place), pieces.last().map(|piece| piece.place));
        if fits
            && let (Some(start), Some(end)) = (start, end)
            && start.line == end.line
        {
            let line: Vec<char> = self.physical_line(start.line).chars().collect();
            let from = start.column.saturating_sub(1) as usize;
            let to = (end.column + end.width).saturating_sub(1) as usize;
            if from <= to && to <= line.len() {
                let fixed: String = line.get(..from).unwrap_or(&[]).iter().collect::<String>()
                    + "\""
                    + &filled
                    + "\""
                    + &line.get(to..).unwrap_or(&[]).iter().collect::<String>();
                message = message.with_fix(fixed);
            }
        }
        Err(message)
    }

    fn additive(&mut self) -> Result<Expression<'a>, Message> {
        let mut left = self.multiplicative()?;
        loop {
            let operator = match self.peek_kind() {
                Some(TokenKind::Symbol(Symbol::Plus)) => BinaryOperator::Add,
                Some(TokenKind::Symbol(Symbol::Minus)) => BinaryOperator::Subtract,
                _ => break,
            };
            self.deeper()?;
            let at = self.here();
            self.advance();
            let right = self.multiplicative()?;
            left = self.binary(operator, left, right, at);
        }
        Ok(left)
    }

    fn multiplicative(&mut self) -> Result<Expression<'a>, Message> {
        let mut left = self.unary()?;
        loop {
            let at = self.here();
            let operator = match self.peek_kind() {
                Some(TokenKind::Symbol(Symbol::Star)) => {
                    self.advance();
                    BinaryOperator::Multiply
                }
                Some(TokenKind::Symbol(Symbol::Slash)) => {
                    self.advance();
                    BinaryOperator::Divide
                }
                Some(TokenKind::Keyword(keyword @ (Keyword::Div | Keyword::Rem))) => {
                    let keyword = *keyword;
                    self.advance();
                    self.rounding(keyword, at)?
                }
                _ => break,
            };
            self.deeper()?;
            let right = self.unary()?;
            left = self.binary(operator, left, right, at);
        }
        Ok(left)
    }

    /// trace: D108, D132. `a ** b` binds tighter than a negating `-`, and that than `*` and `/`.
    fn power(&mut self) -> Result<Expression<'a>, Message> {
        let base = self.operand()?;
        if !self.at_symbol(Symbol::Power) {
            return Ok(base);
        }
        let at = self.here();
        self.advance();
        self.deeper()?;
        // trace: D132. A chain groups from the right, so the exponent takes in the rest of it, and a `-` after `**`
        // negates what follows it.
        let exponent = self.unary()?;
        Ok(self.binary(BinaryOperator::Power, base, exponent, at))
    }

    /// `div` and `rem` say how they round, with `.floor` or `.trunc` (Unio D70, D71).
    fn rounding(&mut self, keyword: Keyword, at: Place) -> Result<BinaryOperator, Message> {
        let bare = || {
            Message::refused(at, format!("`{0}` needs `.floor` or `.trunc`, as in `{0}.floor`", keyword.word()))
                .with_why("how the fraction goes is always said.")
        };
        if !self.at_symbol(Symbol::Dot) {
            return Err(bare());
        }
        self.advance();
        let floor = match self.peek_kind() {
            Some(TokenKind::Name(name)) if *name == "floor" => true,
            Some(TokenKind::Name(name)) if *name == "trunc" => false,
            _ => return Err(bare()),
        };
        self.advance();
        Ok(match (keyword, floor) {
            (Keyword::Div, true) => BinaryOperator::DivFloor,
            (Keyword::Div, false) => BinaryOperator::DivTrunc,
            (_, true) => BinaryOperator::RemFloor,
            (_, false) => BinaryOperator::RemTrunc,
        })
    }

    fn unary(&mut self) -> Result<Expression<'a>, Message> {
        if self.pending.is_none() && self.at_symbol(Symbol::Negate) {
            self.deeper()?;
            let place = self.here();
            self.advance();
            // trace: D132. `**` comes before a negating `-`, so `-2 ** 2` is `-(2 ** 2)`, as maths reads -x².
            let operand = self.unary()?;
            let whole = place.to(operand.place);
            return Ok(self.expression(
                ExpressionKind::Unary { operator: UnaryOperator::Negate, operand: Box::new(operand) },
                whole,
            ));
        }
        self.power()
    }

    fn member_name(&mut self) -> Result<Word<'a>, Message> {
        match self.peek() {
            Some(Token { kind: TokenKind::Name(_) | TokenKind::TypeName(_), place, .. }) => {
                let place = *place;
                Ok(Word { text: self.take_text(), place })
            }
            _ => Err(self.unexpected("a name")),
        }
    }

    /// A single value with everything that sticks to it: `.name`, `( )`, `[ ]`, `?` and `??` (Unio D81).
    fn operand(&mut self) -> Result<Expression<'a>, Message> {
        if let Some(value) = self.pending.take() {
            return Ok(value);
        }
        self.deeper()?;
        let mut value = self.primary()?;
        while let Some(token) = self.peek() {
            let touching = token.gap == Gap::Spaces(0);
            match token.kind {
                TokenKind::Symbol(Symbol::Dot) => {
                    self.advance();
                    let name = self.member_name()?;
                    let place = value.place.to(name.place);
                    value = self.expression(ExpressionKind::Member { receiver: Box::new(value), name }, place);
                }
                TokenKind::Symbol(Symbol::OpenRound) if touching => {
                    self.advance();
                    let start = value.place;
                    let parts = self.entries_after(Some(Entry { label: None, value }), Symbol::CloseRound)?;
                    let place = start.to(self.previous_place());
                    value = self.expression(ExpressionKind::Call { parts }, place);
                }
                TokenKind::Symbol(Symbol::OpenSquare) if touching => {
                    self.advance();
                    let entries = self.entries(Symbol::CloseSquare)?;
                    let place = value.place.to(self.previous_place());
                    value = self.expression(ExpressionKind::Bracketed { receiver: Box::new(value), entries }, place);
                }
                TokenKind::Symbol(Symbol::Question) => {
                    let place = value.place.to(token.place);
                    self.advance();
                    value = self.expression(ExpressionKind::Try(Box::new(value)), place);
                }
                TokenKind::Symbol(Symbol::QuestionQuestion) => {
                    self.advance();
                    let fallback = if self.at_keyword(Keyword::Ignore) {
                        self.advance();
                        Fallback::Ignore
                    } else if self.at_keyword(Keyword::Crash) {
                        self.advance();
                        Fallback::Crash
                    } else if self.at_handler() {
                        // trace: D98. On one line, a handler follows `??`.
                        Fallback::Handler(Box::new(self.handler_use()?))
                    } else {
                        Fallback::Value(Box::new(self.unary()?))
                    };
                    let place = value.place.to(self.previous_place());
                    value = self.expression(ExpressionKind::Handled { call: Box::new(value), fallback }, place);
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn previous_place(&self) -> Place {
        self.line()
            .and_then(|line| line.tokens(&self.tokens).get(self.position.wrapping_sub(1)))
            .map_or(Place::default(), |token| token.place)
    }

    /// The entries up to `closer`, which is read too: values, each with a label when it has one.
    fn entries(&mut self, closer: Symbol) -> Result<Vec<Entry<'a>>, Message> {
        self.entries_after(None, closer)
    }

    /// The entries up to `closer`, after `first` when there is one, all in one list.
    fn entries_after(&mut self, first: Option<Entry<'a>>, closer: Symbol) -> Result<Vec<Entry<'a>>, Message> {
        let mut entries = self.spare.entries();
        entries.extend(first);
        if self.at_symbol(closer) {
            self.advance();
            return Ok(entries);
        }
        loop {
            let label = match (self.peek_kind(), self.peek_at(1).map(|token| &token.kind)) {
                (Some(TokenKind::Name(_)), Some(TokenKind::Symbol(Symbol::Colon))) => {
                    let label = self.expect_name("a label")?;
                    self.advance();
                    Some(label)
                }
                _ => None,
            };
            let value = self.full_expression()?;
            entries.push(Entry { label, value });
            if self.at_symbol(Symbol::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect_symbol(closer)?;
        Ok(entries)
    }

    fn primary(&mut self) -> Result<Expression<'a>, Message> {
        let Some(token) = self.take() else {
            // trace: D8
            return Err(Message::refused(self.end_place(), "this line ends where a value is still wanted")
                .with_why("a line carries on only inside an open `(` or `[`, so wrap the value in `( )`."));
        };
        let place = token.place;
        let kind = match token.kind {
            TokenKind::Whole(_) | TokenKind::Float { .. }
                if matches!(self.peek(), Some(Token { kind: TokenKind::Name(_), gap: Gap::Spaces(_), .. })) =>
            {
                // trace: D112. A unit stands right after its number, as `2 secs`.
                let number = match token.kind {
                    TokenKind::Whole(digits) => ExpressionKind::Whole(digits),
                    TokenKind::Float { text, approximate } => ExpressionKind::Float { text, approximate },
                    _ => ExpressionKind::Whole(""),
                };
                let number = self.expression(number, place);
                let unit = self.unit_written()?;
                let whole = place.to(unit.place);
                return Ok(
                    self.expression(ExpressionKind::Measured { number: Box::new(number), unit: Box::new(unit) }, whole)
                );
            }
            TokenKind::Whole(digits) => ExpressionKind::Whole(digits),
            TokenKind::Float { text, approximate } => ExpressionKind::Float { text, approximate },
            TokenKind::Text(text) => ExpressionKind::Text(text),
            TokenKind::Keyword(Keyword::True) => ExpressionKind::Bool(true),
            TokenKind::Keyword(Keyword::False) => ExpressionKind::Bool(false),
            TokenKind::Keyword(Keyword::Be) => ExpressionKind::Name("be"),
            TokenKind::Name(name) => ExpressionKind::Name(name),
            TokenKind::Underscore => ExpressionKind::Current,
            TokenKind::TypeName(text) => ExpressionKind::TypeName(Word { text, place }),
            TokenKind::Symbol(Symbol::OpenRound) => {
                let inner = self.full_expression()?;
                let close = self.expect_symbol(Symbol::CloseRound)?;
                return Ok(self.expression(ExpressionKind::Group(Box::new(inner)), place.to(close)));
            }
            TokenKind::Symbol(Symbol::OpenSquare) => return self.square_literal(place),
            other => {
                self.put_back(Token { kind: other, place, gap: token.gap });
                return Err(self.unexpected("a value"));
            }
        };
        Ok(self.expression(kind, place))
    }

    /// `[a, b]`, `[key: value]`, `[all: value]`, `[]` and `[:]` (D32, Unio D75, D76, D169).
    fn square_literal(&mut self, open: Place) -> Result<Expression<'a>, Message> {
        if self.at_symbol(Symbol::CloseSquare) {
            let close = self.here();
            self.advance();
            return Ok(self.expression(ExpressionKind::List(Vec::new()), open.to(close)));
        }
        if self.at_symbol(Symbol::Colon) {
            self.advance();
            let close = self.expect_symbol(Symbol::CloseSquare)?;
            return Ok(self.expression(ExpressionKind::EmptyDict, open.to(close)));
        }
        let first = self.full_expression()?;
        if self.at_symbol(Symbol::Colon) {
            self.advance();
            let mut pairs = vec![(first, self.full_expression()?)];
            while self.at_symbol(Symbol::Comma) {
                self.advance();
                let key = self.full_expression()?;
                self.expect_symbol(Symbol::Colon)?;
                pairs.push((key, self.full_expression()?));
            }
            let close = self.expect_symbol(Symbol::CloseSquare)?;
            return Ok(self.expression(ExpressionKind::Pairs(pairs), open.to(close)));
        }
        let mut items = vec![first];
        while self.at_symbol(Symbol::Comma) {
            self.advance();
            items.push(self.full_expression()?);
        }
        let close = self.expect_symbol(Symbol::CloseSquare)?;
        Ok(self.expression(ExpressionKind::List(items), open.to(close)))
    }
}

/// A name with its fields and facts, as braces can hold it (D28, D75).
fn braced(expression: &Expression<'_>) -> Option<String> {
    match &expression.kind {
        ExpressionKind::Name(name) => Some((*name).to_owned()),
        ExpressionKind::Member { receiver, name } => Some(format!("{}.{}", braced(receiver)?, name.text)),
        ExpressionKind::Group(inner) => braced(inner),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageKind;

    fn refusal(source: &str) -> String {
        parse(source).err().and_then(|message| message.what.clone()).unwrap_or_default()
    }

    // trace: D20
    #[test]
    fn d20_a_missing_end_names_its_block() {
        assert_eq!(
            refusal("mut tries = 0\nwhile tries < 3\n    tries = tries + 1\n"),
            "the `while` on line 2 has no `end`"
        );
        assert!(refusal("if true\n    x = 1\n    end\n").contains("wrong depth"));
    }

    // trace: D8
    #[test]
    fn d08_a_line_ending_where_a_value_is_wanted_is_refused() {
        assert_eq!(refusal("total = 1 +\n"), "this line ends where a value is still wanted");
    }

    // trace: D10
    #[test]
    fn d10_else_if_is_written_elif() {
        assert!(refusal("if true\n    x = 1\nelse if false\n    y = 2\nend\n").contains("elif"));
    }

    #[test]
    fn d37_match_and_d40_otherwise_parse() {
        let source = "match last\n    case hurt[amount]\n        hp = hp - amount\n    case idle\nend\nc = f(1) otherwise\n    case too_high\n        instead 99\nend\n";
        let script = parse(source);
        assert!(script.is_ok(), "{:?}", script.err());
    }

    #[test]
    fn d60_a_procedure_that_takes_nothing_has_no_brackets() {
        let script = parse("proc tick\nend\nproc count -> Whole\n    return 1\nend\ntick()\n");
        assert!(script.is_ok(), "{:?}", script.err());
        let kind = parse("proc tick[]\nend\n").err().map(|message| message.kind);
        assert_eq!(kind, Some(MessageKind::Refused));
    }
}
