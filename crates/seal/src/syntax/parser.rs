//! Reading lines of tokens into a syntax tree.
//!
//! Blocks have no `:` on their opening line (D9), a body indented one level deeper, and a plain `end` at the opening
//! line's depth (D20).

use super::lexer::{Gap, Keyword, Line, Symbol, Token, TokenKind, lex};
use super::tree::{
    BinaryOperator, Block, CaseDeclaration, DefaultDeclaration, Entry, Expression, ExpressionKind, Fallback, Family,
    FieldDeclaration, Item, LabelDeclaration, Marker, MatchCase, OneOfDeclaration, OtherwiseCase, Parameter,
    PositionKind, Procedure, RecordDeclaration, Script, SlideStep, Statement, StatementKind, TypeExpression,
    UnaryOperator, Word,
};
use crate::message::{Message, Place};

/// One level of indentation. Seal's own size isn't decided (the standard, chapter 2); every test and example uses
/// Unio's four spaces (Unio D10), so that's what's read, and anything else stops as not designed yet.
const LEVEL: u32 = 4;

/// How deep blocks and expressions may nest. Deeper ones would need more of the host's stack than Seal may safely
/// take (Building rule 1), so they stop as not built yet.
const MAX_BLOCK_DEPTH: u32 = 48;
const MAX_EXPRESSION_DEPTH: u32 = 160;

struct Parser {
    lines: Vec<Line>,
    line_index: usize,
    /// The index of the next token in the current line.
    position: usize,
    next_id: u32,
    block_depth: u32,
    expression_depth: u32,
}

/// What a block's body ran into: the line that isn't part of it.
struct Opener {
    word: &'static str,
    place: Place,
    indentation: u32,
}

/// Reads a script's text into its syntax tree.
///
/// # Errors
///
/// Returns the first thing that can't be read, as a message.
pub fn parse(source: &str) -> Result<Script, Message> {
    let lines = lex(source)?;
    let mut parser = Parser { lines, line_index: 0, position: 0, next_id: 0, block_depth: 0, expression_depth: 0 };
    let mut items = Vec::new();
    while parser.line_index < parser.lines.len() {
        let indentation = parser.indentation();
        if indentation != 0 {
            return Err(Message::refused(parser.line_place(), "this line is indented, but it isn't inside a block")
                .with_why("only a block's body is indented."));
        }
        items.push(parser.item()?);
    }
    Ok(Script { items, expression_count: parser.next_id })
}

impl Parser {
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

    fn peek(&self) -> Option<&Token> {
        self.line().and_then(|line| line.tokens.get(self.position))
    }

    fn peek_at(&self, offset: usize) -> Option<&Token> {
        self.line().and_then(|line| line.tokens.get(self.position + offset))
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|token| &token.kind)
    }

    fn at_symbol(&self, symbol: Symbol) -> bool {
        self.peek_kind() == Some(&TokenKind::Symbol(symbol))
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        self.peek_kind() == Some(&TokenKind::Keyword(keyword))
    }

    fn first_keyword(&self) -> Option<Keyword> {
        match self.line().and_then(|line| line.tokens.first()).map(|token| &token.kind) {
            Some(TokenKind::Keyword(keyword)) => Some(*keyword),
            _ => None,
        }
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.peek().cloned();
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    /// The place right after the line's last token, for "something is missing here".
    fn end_place(&self) -> Place {
        self.line()
            .and_then(|line| line.tokens.last())
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
            Ok(self.advance().map_or(Place::default(), |token| token.place))
        } else {
            Err(self.unexpected(&format!("`{}`", symbol.text())))
        }
    }

    fn expect_keyword(&mut self, keyword: Keyword) -> Result<Place, Message> {
        if self.at_keyword(keyword) {
            Ok(self.advance().map_or(Place::default(), |token| token.place))
        } else {
            Err(self.unexpected(&format!("`{}`", keyword.word())))
        }
    }

    fn expect_name(&mut self, what: &str) -> Result<Word, Message> {
        match self.peek() {
            Some(Token { kind: TokenKind::Name(name), place, .. }) => {
                let word = Word { text: name.clone(), place: *place };
                self.position += 1;
                Ok(word)
            }
            Some(Token { kind: TokenKind::Keyword(keyword), place, .. }) => {
                // trace: D7
                Err(Message::refused(*place, format!("`{}` is a keyword, so it can't be a name", keyword.word()))
                    .with_why("a name can't be one of Seal's own words."))
            }
            _ => Err(self.unexpected(what)),
        }
    }

    fn expect_type_name(&mut self) -> Result<Word, Message> {
        match self.peek() {
            Some(Token { kind: TokenKind::TypeName(name), place, .. }) => {
                let word = Word { text: name.clone(), place: *place };
                self.position += 1;
                Ok(word)
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
        self.position = 0;
        Ok(())
    }

    fn expression(&mut self, kind: ExpressionKind, place: Place) -> Expression {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        Expression { id, kind, place }
    }

    // ---- blocks ----

    /// Reads a body: the lines one level deeper than `opener`, up to the line at the opener's own depth.
    fn body(&mut self, opener: &Opener) -> Result<Block, Message> {
        self.block_depth += 1;
        if self.block_depth > MAX_BLOCK_DEPTH {
            return Err(Message::not_built_yet(
                opener.place,
                format!("blocks nested more than {MAX_BLOCK_DEPTH} deep (D20) aren't"),
            ));
        }
        let mut block = Vec::new();
        while self.in_body(opener)? {
            block.push(self.statement()?);
        }
        self.block_depth -= 1;
        Ok(block)
    }

    /// Whether the current line belongs to `opener`'s body. A line at a depth that's neither the body's nor the
    /// opener's is an error.
    fn in_body(&self, opener: &Opener) -> Result<bool, Message> {
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
        Err(Message::not_designed_yet(
            Place::new(line.place.line, 1, line.indentation.max(1)),
            format!("indentation other than {LEVEL} spaces a level"),
        ))
    }

    /// trace: D20
    fn no_end(&self, opener: &Opener) -> Message {
        let place = self.line().map_or(opener.place, |line| line.place);
        Message::refused(place, format!("the `{}` on line {} has no `end`", opener.word, opener.place.line))
            .with_because(format!("line {} opens it", opener.place.line), opener.place)
            .with_why("every block closes with `end`, at the depth of the line that opened it.")
    }

    /// Reads the `end` that closes `opener`'s block.
    fn end_of(&mut self, opener: &Opener) -> Result<Place, Message> {
        let at_depth = self.line().is_some_and(|line| line.indentation == opener.indentation);
        if !at_depth || self.first_keyword() != Some(Keyword::End) {
            return Err(self.no_end(opener));
        }
        let place = self.expect_keyword(Keyword::End)?;
        self.end_line()?;
        Ok(place)
    }

    fn opener(&self, word: &'static str) -> Opener {
        Opener { word, place: self.line_place(), indentation: self.indentation() }
    }

    // ---- the top level ----

    fn item(&mut self) -> Result<Item, Message> {
        match self.first_keyword() {
            Some(Keyword::Proc) => Ok(Item::Procedure(self.procedure()?)),
            Some(Keyword::Fam) => Ok(Item::Family(self.family()?)),
            Some(Keyword::Record) => Ok(Item::Record(self.record()?)),
            Some(Keyword::OneOf) => Ok(Item::OneOf(self.one_of()?)),
            _ => Ok(Item::Statement(self.statement()?)),
        }
    }

    fn path(&mut self) -> Result<Vec<Word>, Message> {
        let mut path = vec![self.expect_name("a procedure's name")?];
        while self.at_symbol(Symbol::Dot) {
            self.advance();
            path.push(self.expect_name("a narrowing's name")?);
        }
        Ok(path)
    }

    /// trace: D21, D41, D42, D45
    #[allow(clippy::too_many_lines)] // the first line, then the `labels` and `defaults` lines, read in order
    fn procedure(&mut self) -> Result<Procedure, Message> {
        let opener = self.opener("proc");
        self.expect_keyword(Keyword::Proc)?;
        let path = self.path()?;
        if self.at_symbol(Symbol::OpenRound) {
            // trace: Unio D166
            return Err(Message::refused(self.here(), "a procedure's parameters go in `[ ]`, not `( )`")
                .with_why("`( )` calls a procedure; `[ ]` describes one."));
        }
        let mut parameters = Vec::new();
        // trace: D60. A procedure that takes nothing has no brackets.
        if self.at_symbol(Symbol::OpenSquare) {
            self.advance();
            if self.at_symbol(Symbol::CloseSquare) {
                let name = path.iter().map(|word| word.text.as_str()).collect::<Vec<_>>().join(".");
                return Err(Message::refused(
                    self.here(),
                    format!("a procedure that takes nothing has no `[ ]`: it's written `proc {name}`"),
                )
                .with_why("`[ ]` describes the values a procedure takes, and this one takes none."));
            }
            loop {
                let name = self.expect_name("a parameter's name")?;
                let written_type = if self.at_symbol(Symbol::Colon) {
                    self.advance();
                    Some(self.type_expression()?)
                } else {
                    None
                };
                parameters.push(Parameter { name, written_type });
                if self.at_symbol(Symbol::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect_symbol(Symbol::CloseSquare)?;
        }
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
        let mut labels = Vec::new();
        let mut defaults = Vec::new();
        if self.in_body(&opener)? && self.first_keyword() == Some(Keyword::Labels) {
            self.advance();
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
            if self.in_body(&opener)? && self.first_keyword() == Some(Keyword::Defaults) {
                self.advance();
                self.expect_symbol(Symbol::OpenSquare)?;
                loop {
                    let label = self.expect_name("a label")?;
                    self.expect_symbol(Symbol::Colon)?;
                    let value = self.operand()?;
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
        let body = self.body(&opener)?;
        let end_place = self.end_of(&opener)?;
        Ok(Procedure { path, parameters, result, failures, labels, defaults, body, place: opener.place, end_place })
    }

    /// trace: D43
    fn family(&mut self) -> Result<Family, Message> {
        let opener = self.opener("fam");
        self.expect_keyword(Keyword::Fam)?;
        self.expect_symbol(Symbol::Dot)?;
        if !self.at_keyword(Keyword::Proc) {
            if matches!(self.peek_kind(), Some(TokenKind::Name(name)) if name == "handler") {
                return Err(Message::not_designed_yet(self.here(), "`fam.handler`, which comes with handlers (D43),"));
            }
            return Err(self.unexpected("`proc`"));
        }
        self.advance();
        let word = self.expect_name("the family's word")?;
        self.end_line()?;
        let mut members = Vec::new();
        while self.in_body(&opener)? {
            if self.first_keyword() != Some(Keyword::Proc) {
                return Err(Message::refused(self.line_place(), "a `fam.proc` block holds only procedures")
                    .with_why("a family holds only the kind of thing it's declared for."));
            }
            self.block_depth += 1;
            members.push(self.procedure()?);
            self.block_depth -= 1;
        }
        self.end_of(&opener)?;
        Ok(Family { word, members, place: opener.place })
    }

    fn field(&mut self) -> Result<FieldDeclaration, Message> {
        let name = self.expect_name("a field's name")?;
        self.expect_symbol(Symbol::Colon)?;
        let written_type = self.type_expression()?;
        Ok(FieldDeclaration { name, written_type })
    }

    /// trace: D27
    fn record(&mut self) -> Result<RecordDeclaration, Message> {
        let opener = self.opener("record");
        self.expect_keyword(Keyword::Record)?;
        let name = self.expect_type_name()?;
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
    fn one_of(&mut self) -> Result<OneOfDeclaration, Message> {
        let opener = self.opener("one_of");
        self.expect_keyword(Keyword::OneOf)?;
        let name = self.expect_type_name()?;
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
    fn type_expression(&mut self) -> Result<TypeExpression, Message> {
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
                settings.push((path, self.operand()?));
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
    fn statement(&mut self) -> Result<Statement, Message> {
        let place = self.line_place();
        let kind = match self.first_keyword() {
            Some(keyword @ (Keyword::Mut | Keyword::Unused)) => {
                self.advance();
                let marker = if keyword == Keyword::Mut { Marker::Mut } else { Marker::Unused };
                let name = self.expect_name("a name")?;
                let written_type = if self.at_symbol(Symbol::Colon) {
                    self.advance();
                    Some(self.type_expression()?)
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
            Some(Keyword::For) => return self.for_each(),
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
                return Ok(Statement { kind: StatementKind::Instead(value), place });
            }
            Some(keyword @ (Keyword::Proc | Keyword::Fam | Keyword::Record | Keyword::OneOf)) => {
                return Err(Message::not_designed_yet(place, format!("a `{}` inside a block", keyword.word())));
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
                        Some(self.type_expression()?)
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
    fn value_with_otherwise(&mut self) -> Result<Expression, Message> {
        let value = self.full_expression()?;
        if self.at_keyword(Keyword::Otherwise) {
            self.otherwise(value)
        } else {
            self.end_line()?;
            Ok(value)
        }
    }

    /// trace: D37, Unio D40
    fn otherwise(&mut self, call: Expression) -> Result<Expression, Message> {
        let opener = self.opener("otherwise");
        let place = call.place;
        self.expect_keyword(Keyword::Otherwise)?;
        if self.peek().is_some() {
            return Err(Message::not_designed_yet(
                self.here(),
                "a handler after `otherwise`, which comes with handlers (D43),",
            ));
        }
        self.end_line()?;
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
        Ok(self.expression(ExpressionKind::Otherwise { call: Box::new(call), cases }, place))
    }

    /// A case's body: it runs until the next `case` or the block's `end` (D37).
    fn case_body(&mut self, case: &Opener, block: &Opener) -> Result<Block, Message> {
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
                return Err(Message::not_designed_yet(
                    Place::new(line.place.line, 1, line.indentation.max(1)),
                    format!("indentation other than {LEVEL} spaces a level"),
                ));
            }
            body.push(self.statement()?);
        }
        self.block_depth -= 1;
        Ok(body)
    }

    /// trace: D9, D10, D20
    fn if_statement(&mut self) -> Result<Statement, Message> {
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
    fn for_each(&mut self) -> Result<Statement, Message> {
        let opener = self.opener("for");
        self.expect_keyword(Keyword::For)?;
        self.expect_keyword(Keyword::Each)?;
        let item = self.expect_name("the item's name")?;
        let position = if self.at_keyword(Keyword::At) {
            self.advance();
            let kind = match self.peek_kind() {
                Some(TokenKind::Name(name)) if name == "pos" => PositionKind::Pos,
                Some(TokenKind::Name(name)) if name == "offset" => PositionKind::Offset,
                Some(TokenKind::Name(name)) if name == "key" => PositionKind::Key,
                _ => return Err(self.unexpected("`pos`, `offset` or `key`")),
            };
            self.advance();
            Some((kind, self.expect_name("a name")?))
        } else {
            None
        };
        self.expect_keyword(Keyword::In)?;
        let source = self.full_expression()?;
        self.end_line()?;
        let body = self.body(&opener)?;
        self.end_of(&opener)?;
        Ok(Statement { kind: StatementKind::ForEach { item, position, source, body }, place: opener.place })
    }

    /// trace: D37
    fn match_statement(&mut self) -> Result<Statement, Message> {
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
    fn full_expression(&mut self) -> Result<Expression, Message> {
        let saved = self.expression_depth;
        let start = self.or_expression()?;
        let result = if self.at_symbol(Symbol::Slide) { self.slide(start) } else { Ok(start) };
        self.expression_depth = saved;
        result
    }

    /// trace: Unio D5, D73, D81
    fn slide(&mut self, start: Expression) -> Result<Expression, Message> {
        let place = start.place;
        let mut steps = Vec::new();
        while self.at_symbol(Symbol::Slide) {
            self.advance();
            let step_place = self.here();
            let mut target = match self.peek_kind() {
                Some(TokenKind::Name(name)) => {
                    let name = name.clone();
                    self.advance();
                    self.expression(ExpressionKind::Name(name), step_place)
                }
                Some(TokenKind::Keyword(Keyword::Be)) => {
                    self.advance();
                    self.expression(ExpressionKind::Name("be".to_owned()), step_place)
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

    fn binary(&mut self, operator: BinaryOperator, left: Expression, right: Expression, at: Place) -> Expression {
        let place = left.place.to(right.place);
        self.expression(
            ExpressionKind::Binary { operator, left: Box::new(left), right: Box::new(right), operator_place: at },
            place,
        )
    }

    fn or_expression(&mut self) -> Result<Expression, Message> {
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

    fn and_expression(&mut self) -> Result<Expression, Message> {
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
    fn not_expression(&mut self) -> Result<Expression, Message> {
        if self.at_keyword(Keyword::Not) {
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
            TokenKind::Symbol(Symbol::EqualEqual) => Some(BinaryOperator::Equal),
            TokenKind::Symbol(Symbol::NotEqual) => Some(BinaryOperator::NotEqual),
            TokenKind::Symbol(Symbol::Less) => Some(BinaryOperator::Less),
            TokenKind::Symbol(Symbol::Greater) => Some(BinaryOperator::Greater),
            TokenKind::Keyword(Keyword::AtLeast) => Some(BinaryOperator::AtLeast),
            TokenKind::Keyword(Keyword::AtMost) => Some(BinaryOperator::AtMost),
            _ => None,
        }
    }

    fn comparison(&mut self) -> Result<Expression, Message> {
        let left = self.join()?;
        let Some(operator) = self.comparison_operator() else {
            return Ok(left);
        };
        self.deeper()?;
        let at = self.here();
        self.advance();
        let right = self.join()?;
        if self.comparison_operator().is_some() {
            // trace: Unio D81
            return Err(Message::refused(self.here(), "two comparisons in a row")
                .with_why("each comparison stands alone; join two with `and`, as in `1 < x and x < 10`."));
        }
        Ok(self.binary(operator, left, right, at))
    }

    /// `&` joins text. Next to unbracketed maths it's refused, since maths doesn't say which goes first (Unio D72).
    fn join(&mut self) -> Result<Expression, Message> {
        let mut left = self.additive()?;
        while self.at_symbol(Symbol::Ampersand) {
            self.deeper()?;
            let at = self.here();
            self.advance();
            let right = self.additive()?;
            for side in [&left, &right] {
                if let ExpressionKind::Binary { operator, .. } = &side.kind
                    && operator.is_maths()
                {
                    return Err(Message::refused(side.place, "maths next to `&` needs `( )` around it")
                        .with_why("maths doesn't say whether `&` or the sum goes first, so brackets do."));
                }
            }
            left = self.binary(BinaryOperator::Join, left, right, at);
        }
        Ok(left)
    }

    fn additive(&mut self) -> Result<Expression, Message> {
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

    fn multiplicative(&mut self) -> Result<Expression, Message> {
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
            Some(TokenKind::Name(name)) if name == "floor" => true,
            Some(TokenKind::Name(name)) if name == "trunc" => false,
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

    fn unary(&mut self) -> Result<Expression, Message> {
        if self.at_symbol(Symbol::Negate) {
            self.deeper()?;
            let place = self.here();
            self.advance();
            let operand = self.unary()?;
            let whole = place.to(operand.place);
            return Ok(self.expression(
                ExpressionKind::Unary { operator: UnaryOperator::Negate, operand: Box::new(operand) },
                whole,
            ));
        }
        self.operand()
    }

    fn member_name(&mut self) -> Result<Word, Message> {
        match self.peek() {
            Some(Token { kind: TokenKind::Name(name) | TokenKind::TypeName(name), place, .. }) => {
                let word = Word { text: name.clone(), place: *place };
                self.position += 1;
                Ok(word)
            }
            _ => Err(self.unexpected("a name")),
        }
    }

    /// A single value with everything that sticks to it: `.name`, `( )`, `[ ]`, `?` and `??` (Unio D81).
    fn operand(&mut self) -> Result<Expression, Message> {
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
                    let arguments = self.entries(Symbol::CloseRound)?;
                    let place = value.place.to(self.previous_place());
                    value = self.expression(ExpressionKind::Call { callee: Box::new(value), arguments }, place);
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
            .and_then(|line| line.tokens.get(self.position.wrapping_sub(1)))
            .map_or(Place::default(), |token| token.place)
    }

    /// The entries up to `closer`, which is read too: values, each with a label when it has one.
    fn entries(&mut self, closer: Symbol) -> Result<Vec<Entry>, Message> {
        let mut entries = Vec::new();
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

    fn primary(&mut self) -> Result<Expression, Message> {
        let Some(token) = self.advance() else {
            // trace: D8
            return Err(Message::refused(self.end_place(), "this line ends where a value is still wanted")
                .with_why("a line carries on only inside an open `(` or `[`, so wrap the value in `( )`."));
        };
        let place = token.place;
        let kind = match token.kind {
            TokenKind::Whole(digits) => ExpressionKind::Whole(digits),
            TokenKind::Float { text, approximate } => ExpressionKind::Float { text, approximate },
            TokenKind::Text(text) => ExpressionKind::Text(text),
            TokenKind::Keyword(Keyword::True) => ExpressionKind::Bool(true),
            TokenKind::Keyword(Keyword::False) => ExpressionKind::Bool(false),
            TokenKind::Keyword(Keyword::Be) => ExpressionKind::Name("be".to_owned()),
            TokenKind::Name(name) => ExpressionKind::Name(name),
            TokenKind::TypeName(text) => ExpressionKind::TypeName(Word { text, place }),
            TokenKind::Symbol(Symbol::OpenRound) => {
                let inner = self.full_expression()?;
                let close = self.expect_symbol(Symbol::CloseRound)?;
                return Ok(self.expression(ExpressionKind::Group(Box::new(inner)), place.to(close)));
            }
            TokenKind::Symbol(Symbol::OpenSquare) => return self.square_literal(place),
            other => {
                self.position -= 1;
                let _ = other;
                return Err(self.unexpected("a value"));
            }
        };
        Ok(self.expression(kind, place))
    }

    /// `[a, b]`, `[key: value]`, `[all: value]`, `[]` and `[:]` (D32, Unio D75, D76, D169).
    fn square_literal(&mut self, open: Place) -> Result<Expression, Message> {
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
