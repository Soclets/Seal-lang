//! Reading a script's text into lines of tokens.
//!
//! A statement ends at the end of its line, and carries on only inside an open `(` or `[` (D8), so the lexer hands
//! the parser logical lines: each one statement's tokens, with its indentation.

use crate::message::{Message, Place};

/// A word that's part of Seal itself. A name can't be one of these (D7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keyword {
    Mut,
    Unused,
    If,
    Elif,
    Else,
    End,
    While,
    Break,
    For,
    Each,
    In,
    At,
    Proc,
    Return,
    Fail,
    Record,
    OneOf,
    Match,
    Case,
    Otherwise,
    Instead,
    Ignore,
    Crash,
    Labels,
    Defaults,
    As,
    Fam,
    True,
    False,
    And,
    Or,
    Not,
    AtLeast,
    AtMost,
    Be,
    Div,
    Rem,
    /// `comment name` opens a block comment, so a name can't be `comment` (D91).
    Comment,
}

impl Keyword {
    fn from_word(word: &str) -> Option<Keyword> {
        Some(match word {
            "mut" => Keyword::Mut,
            "unused" => Keyword::Unused,
            "if" => Keyword::If,
            "elif" => Keyword::Elif,
            "else" => Keyword::Else,
            "end" => Keyword::End,
            "while" => Keyword::While,
            "break" => Keyword::Break,
            "for" => Keyword::For,
            "each" => Keyword::Each,
            "in" => Keyword::In,
            "at" => Keyword::At,
            "proc" => Keyword::Proc,
            "return" => Keyword::Return,
            "fail" => Keyword::Fail,
            "record" => Keyword::Record,
            "one_of" => Keyword::OneOf,
            "match" => Keyword::Match,
            "case" => Keyword::Case,
            "otherwise" => Keyword::Otherwise,
            "instead" => Keyword::Instead,
            "ignore" => Keyword::Ignore,
            "crash" => Keyword::Crash,
            "labels" => Keyword::Labels,
            "defaults" => Keyword::Defaults,
            "as" => Keyword::As,
            "fam" => Keyword::Fam,
            "true" => Keyword::True,
            "false" => Keyword::False,
            "and" => Keyword::And,
            "or" => Keyword::Or,
            "not" => Keyword::Not,
            "at_least" => Keyword::AtLeast,
            "at_most" => Keyword::AtMost,
            "be" => Keyword::Be,
            "div" => Keyword::Div,
            "rem" => Keyword::Rem,
            "comment" => Keyword::Comment,
            _ => return None,
        })
    }

    /// Whether a word is one of Seal's keywords.
    #[must_use]
    pub fn is_keyword(word: &str) -> bool {
        Keyword::from_word(word).is_some()
    }

    /// The keyword as it's written.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Keyword::Mut => "mut",
            Keyword::Unused => "unused",
            Keyword::If => "if",
            Keyword::Elif => "elif",
            Keyword::Else => "else",
            Keyword::End => "end",
            Keyword::While => "while",
            Keyword::Break => "break",
            Keyword::For => "for",
            Keyword::Each => "each",
            Keyword::In => "in",
            Keyword::At => "at",
            Keyword::Proc => "proc",
            Keyword::Return => "return",
            Keyword::Fail => "fail",
            Keyword::Record => "record",
            Keyword::OneOf => "one_of",
            Keyword::Match => "match",
            Keyword::Case => "case",
            Keyword::Otherwise => "otherwise",
            Keyword::Instead => "instead",
            Keyword::Ignore => "ignore",
            Keyword::Crash => "crash",
            Keyword::Labels => "labels",
            Keyword::Defaults => "defaults",
            Keyword::As => "as",
            Keyword::Fam => "fam",
            Keyword::True => "true",
            Keyword::False => "false",
            Keyword::And => "and",
            Keyword::Or => "or",
            Keyword::Not => "not",
            Keyword::AtLeast => "at_least",
            Keyword::AtMost => "at_most",
            Keyword::Be => "be",
            Keyword::Div => "div",
            Keyword::Rem => "rem",
            Keyword::Comment => "comment",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symbol {
    Equals,
    EqualEqual,
    NotEqual,
    Less,
    Greater,
    /// The `<` that opens a type's blanks, as in `List<Text>`.
    TypeOpen,
    /// The `>` that closes them.
    TypeClose,
    Plus,
    /// A `-` between two values.
    Minus,
    /// A `-` right before a value (Unio D27).
    Negate,
    Star,
    /// `**`, raising to a power (D108).
    Power,
    Slash,
    Ampersand,
    Slide,
    Question,
    QuestionQuestion,
    Arrow,
    Bang,
    OpenRound,
    CloseRound,
    OpenSquare,
    CloseSquare,
    Comma,
    Colon,
    Dot,
    /// `::`, reaching into a named set, as a handler group (D110).
    Path,
}

impl Symbol {
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Symbol::Equals => "=",
            Symbol::EqualEqual => "==",
            Symbol::NotEqual => "!=",
            Symbol::Less | Symbol::TypeOpen => "<",
            Symbol::Greater | Symbol::TypeClose => ">",
            Symbol::Plus => "+",
            Symbol::Minus | Symbol::Negate => "-",
            Symbol::Star => "*",
            Symbol::Power => "**",
            Symbol::Slash => "/",
            Symbol::Ampersand => "&",
            Symbol::Slide => "|>",
            Symbol::Question => "?",
            Symbol::QuestionQuestion => "??",
            Symbol::Arrow => "->",
            Symbol::Bang => "!",
            Symbol::OpenRound => "(",
            Symbol::CloseRound => ")",
            Symbol::OpenSquare => "[",
            Symbol::CloseSquare => "]",
            Symbol::Comma => ",",
            Symbol::Colon => ":",
            Symbol::Dot => ".",
            Symbol::Path => "::",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    /// A lowercase word that isn't a keyword (D24).
    Name(String),
    Keyword(Keyword),
    /// A capitalised word: a capital always means a type (D24).
    TypeName(String),
    /// A number with no point: its digits (Unio D180).
    Whole(String),
    /// A number with a point: its text without the `~`, and whether `~` was written (Unio D155, D180).
    Float {
        text: String,
        approximate: bool,
    },
    /// Text in double quotes, without the quotes (D16, Unio D21).
    Text(String),
    Symbol(Symbol),
    /// The note after `end \\`, naming the block the `end` closes, with single spaces between its words (D102).
    Note(String),
}

impl TokenKind {
    /// How the token reads in a message.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Name(name) | TokenKind::TypeName(name) => format!("`{name}`"),
            TokenKind::Keyword(keyword) => format!("`{}`", keyword.word()),
            TokenKind::Whole(digits) => format!("`{digits}`"),
            TokenKind::Float { text, approximate } => {
                if *approximate {
                    format!("`~{text}`")
                } else {
                    format!("`{text}`")
                }
            }
            TokenKind::Text(_) => "text".to_owned(),
            TokenKind::Symbol(symbol) => format!("`{}`", symbol.text()),
            TokenKind::Note(note) => format!("`\\\\ {note}`"),
        }
    }
}

/// What's between a token and the one before it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gap {
    /// The first token of its statement.
    LineStart,
    /// This many spaces, on the same line.
    Spaces(u32),
    /// A line break inside an open bracket (D8).
    LineBreak,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub place: Place,
    pub gap: Gap,
}

/// One statement's worth of tokens (D8).
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// The number of spaces before the first token.
    pub indentation: u32,
    pub tokens: Vec<Token>,
    /// The place of the whole first physical line, for messages about the line itself.
    pub place: Place,
    /// Where the `///` comment right before the line starts, when it has one (D63, D91).
    pub documented: Option<Place>,
}

struct Lexer {
    lines: Vec<Line>,
    current: Vec<Token>,
    current_indentation: u32,
    current_place: Place,
    /// The open `(` and `[`, innermost last.
    open_brackets: Vec<(Symbol, Place)>,
    /// How many type `<` are open.
    type_depth: u32,
    /// Spacing mistakes found while reading the statement's lines, beyond the gaps between its tokens.
    mistakes: Vec<SpacingMistake>,
    /// The `///` comment waiting for the line it describes.
    documented: Option<Place>,
    /// The word being read, in room every word reuses.
    word: String,
}

/// Splits a script's text into logical lines of tokens.
///
/// # Errors
///
/// Returns the first thing that can't be read, as a message.
#[allow(clippy::too_many_lines)] // one pass over a script's lines: each kind of line is dealt with where it's met
pub fn lex(source: &str) -> Result<Vec<Line>, Message> {
    let mut lexer = Lexer {
        lines: Vec::new(),
        current: Vec::new(),
        current_indentation: 0,
        current_place: Place::default(),
        open_brackets: Vec::new(),
        type_depth: 0,
        mistakes: Vec::new(),
        documented: None,
        word: String::with_capacity(32),
    };
    let mut physical: Vec<&str> = source.split('\n').collect();
    lexer.lines.reserve(physical.len());
    // A file's last line break ends its last line; it doesn't start an empty one.
    if physical.last() == Some(&"") {
        physical.pop();
    }
    // trace: D63, Unio D144. A `\r` right before a line break is part of the line break.
    for line in &mut physical {
        *line = line.strip_suffix('\r').unwrap_or(line);
    }
    let mut blank_run = 0u32;
    // The open block comment: its depth, its name and where it starts (D63, D102).
    let mut comment_block: Option<(usize, String, Place)> = None;
    // The first `///` line still waiting for its declaration (D63, Unio D74).
    let mut pending_doc: Option<Place> = None;
    // Comment lines waiting for the line after them, which says how they're indented (D91, Unio D145).
    let mut pending_comments: Vec<(u32, usize)> = Vec::new();
    let doc_without_declaration = |place: Place| {
        Message::refused(place, "this `///` has no declaration after it")
            .with_why("a `///` comment describes the declaration on the line after it.")
    };
    // One line's characters, in room that every line reuses.
    let mut characters: Vec<char> = Vec::new();
    for (index, text) in physical.iter().enumerate() {
        let number = u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1);
        characters.clear();
        characters.extend(text.chars());
        let leading = characters.iter().take_while(|&&character| character == ' ').count();
        if let Some((depth, name, _)) = &comment_block {
            // Everything inside a block comment is ignored; an `end` at its own depth closes it, and the note after
            // it, when written, names the comment (D102).
            if leading == *depth
                && let Some(rest) = characters.get(leading..)
                && rest.starts_with(&['e', 'n', 'd'])
                && (rest.len() == 3 || rest.get(3) == Some(&' '))
            {
                let after: String = rest.get(3..).unwrap_or(&[]).iter().collect();
                if !after.is_empty() {
                    let wanted = format!("comment {name}");
                    let written = after.trim_start().strip_prefix("\\\\").map(str::trim);
                    if written != Some(wanted.as_str()) {
                        let column = column_of(leading + 4);
                        return Err(Message::refused(
                            Place::new(
                                number,
                                column,
                                u32::try_from(after.len()).unwrap_or(1).saturating_sub(1).max(1),
                            ),
                            format!("this `end` closes `comment {name}`, so its note is `\\\\ {wanted}`"),
                        )
                        .with_fix(format!("{}end \\\\ {wanted}", " ".repeat(leading)))
                        .with_why("an `end`'s note names the block it closes, so it can't go stale."));
                    }
                }
                comment_block = None;
            }
            continue;
        }
        let width = u32::try_from(characters.len()).unwrap_or(u32::MAX);
        if let Some(position) = characters.iter().position(|&character| character == '\r') {
            return Err(Message::not_designed_yet(
                Place::new(number, column_of(position), 1),
                "a `\\r` that isn't right before a line break",
            ));
        }
        if characters.is_empty() {
            if let Some(place) = pending_doc {
                return Err(doc_without_declaration(place));
            }
            blank_run += 1;
            if !lexer.open_brackets.is_empty() {
                // trace: D93. A statement spread over lines stays in one piece.
                return Err(Message::refused(Place::new(number, 1, 1), "this blank line is inside an open bracket")
                    .with_why(
                        "a statement carries on over lines only inside its brackets, with no blank line between them.",
                    ));
            }
            if blank_run > 1 {
                // trace: D63, Unio D97, D101
                let extra = physical.get(index..).unwrap_or(&[]).iter().take_while(|line| line.is_empty()).count();
                let lines = if extra == 1 { "line" } else { "lines" };
                return Err(Message::refused(
                    Place::new(number, 1, 1),
                    format!("{extra} extra blank {lines}, after the one on line {}", number - 1),
                )
                .with_why("one blank line is enough to separate the parts of a script."));
            }
            continue;
        }
        blank_run = 0;
        // trace: D63, Unio D10. A tab in the indentation is refused, with the line indented by spaces as the fix.
        let indentation = characters.iter().take_while(|&&character| character == ' ' || character == '\t').count();
        if indentation > leading {
            let tabs = characters.iter().take(indentation).filter(|&&character| character == '\t').count();
            let rest: String = characters.get(indentation..).unwrap_or(&[]).iter().collect();
            return Err(Message::refused(
                Place::new(number, 1, u32::try_from(indentation).unwrap_or(1)),
                "this line is indented with a tab",
            )
            .with_fix(format!("{}{}", " ".repeat(indentation - tabs + tabs * 4), rest.trim_end()))
            .with_why("one level of indentation is 4 spaces."));
        }
        if leading == characters.len() {
            // trace: D63, Unio D98
            return Err(Message::refused(Place::new(number, 1, width), "this blank line has spaces on it")
                .with_why("a blank line is empty."));
        }
        if characters.last() == Some(&' ') {
            let trailing = characters.iter().rev().take_while(|&&character| character == ' ').count();
            characters.truncate(characters.len() - trailing);
            lexer.mistakes.push(SpacingMistake {
                line: number,
                column: column_of(characters.len()),
                actual: u32::try_from(trailing).unwrap_or(1),
                wanted: 0,
            });
        }
        let continued = !lexer.open_brackets.is_empty();
        let rest = characters.get(leading..).unwrap_or(&[]);
        // trace: D63, D91, Unio D74. A line that's only a comment, inside an open bracket too.
        if rest.starts_with(&['/', '/']) {
            let place = Place::new(number, column_of(leading), width.saturating_sub(column_of(leading) - 1));
            if rest.starts_with(&['/', '/', '/']) && rest.get(3) != Some(&'/') {
                if continued {
                    return Err(Message::refused(place, "this `///` is inside an open bracket")
                        .with_why("a `///` comment describes the declaration on the line after it."));
                }
                pending_doc.get_or_insert(place);
            } else if let Some(doc) = pending_doc {
                return Err(doc_without_declaration(doc));
            }
            if continued {
                // A comment line inside a statement keeps the statement's own spacing mistakes waiting.
            } else {
                check_spacing(&[], &std::mem::take(&mut lexer.mistakes), &physical)?;
            }
            pending_comments.push((number, leading));
            continue;
        }
        // trace: D20, D63, D102. `comment name` on its own line opens a block comment.
        if !continued && let Some(name) = comment_opener(rest) {
            if let Some(doc) = pending_doc {
                return Err(doc_without_declaration(doc));
            }
            check_comment_indentation(&mut pending_comments, leading, rest)?;
            check_spacing(&[], &std::mem::take(&mut lexer.mistakes), &physical)?;
            comment_block = Some((leading, name, Place::new(number, column_of(leading), 7)));
            continue;
        }
        check_comment_indentation(&mut pending_comments, leading, rest)?;
        lexer.documented = pending_doc.take().or(lexer.documented);
        let width = u32::try_from(characters.len()).unwrap_or(u32::MAX);
        if continued {
            // trace: D106. A statement's later lines are indented one level deeper than its first.
            let wanted = lexer.current_indentation + 4;
            if u32::try_from(leading).ok() != Some(wanted) {
                return Err(Message::refused(
                    Place::new(number, 1, u32::try_from(leading).unwrap_or(1).max(1)),
                    format!(
                        "this line carries on a statement, so it's indented {wanted} spaces, one level deeper than the statement's first line"
                    ),
                )
                .with_indentation_fix(wanted)
                .with_why("a statement's later lines are indented one level, 4 spaces, deeper than its first."));
            }
        } else {
            lexer.current_indentation = u32::try_from(leading).unwrap_or(u32::MAX);
            lexer.current_place = Place::new(number, column_of(leading), width.saturating_sub(column_of(leading) - 1));
        }
        lexer.line(number, &characters, leading, continued)?;
        if lexer.open_brackets.is_empty() {
            lexer.finish_line(&physical)?;
        }
    }
    if let Some((_, _, place)) = comment_block {
        return Err(Message::refused(place, "this `comment` is never closed with `end`")
            .with_why("a block comment closes with `end`, at the depth of its `comment` line."));
    }
    if let Some(place) = pending_doc {
        return Err(doc_without_declaration(place));
    }
    if let Some((symbol, place)) = lexer.open_brackets.last() {
        return Err(Message::refused(*place, format!("this `{}` is never closed", symbol.text())));
    }
    Ok(lexer.lines)
}

/// Checks the comment lines waiting for this one: each is indented like it, or one level in when it's an `end`, an
/// `elif` or an `else`, which close what the comment is about (D91, Unio D145).
fn check_comment_indentation(pending: &mut Vec<(u32, usize)>, leading: usize, rest: &[char]) -> Result<(), Message> {
    if pending.is_empty() {
        return Ok(());
    }
    let length = rest.iter().take_while(|character| character.is_ascii_alphanumeric() || **character == '_').count();
    let word = rest.get(..length).unwrap_or(&[]);
    let closes = [&['e', 'n', 'd'][..], &['e', 'l', 'i', 'f'], &['e', 'l', 's', 'e']].contains(&word);
    let wanted = if closes { leading + 4 } else { leading };
    for (line, indentation) in pending.drain(..) {
        if indentation != wanted {
            return Err(Message::refused(
                Place::new(line, 1, u32::try_from(indentation).unwrap_or(1).max(1)),
                format!("this comment is indented {indentation} spaces, where the line after it puts it at {wanted}"),
            )
            .with_indentation_fix(u32::try_from(wanted).unwrap_or(0))
            .with_why(
                "a comment on its own line is indented like the line after it, or one level in before `end`, `elif` or `else`.",
            ));
        }
    }
    Ok(())
}

/// One gap that isn't the fixed spacing: where it starts, how many spaces it has, and how many it should have.
#[derive(Clone, Copy, Debug)]
struct SpacingMistake {
    line: u32,
    column: u32,
    actual: u32,
    wanted: u32,
}

/// The comment's name, when a line, past its indentation, is `comment name` and nothing else (D63).
fn comment_opener(rest: &[char]) -> Option<String> {
    let name = rest.strip_prefix(&['c', 'o', 'm', 'm', 'e', 'n', 't', ' '])?;
    let well_formed = name.first().is_some_and(|first| first.is_ascii_lowercase() || *first == '_')
        && name
            .iter()
            .all(|&character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_');
    well_formed.then(|| name.iter().collect())
}

fn column_of(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX - 1).saturating_add(1)
}

impl Lexer {
    fn finish_line(&mut self, physical: &[&str]) -> Result<(), Message> {
        // The next line's tokens get room for a usual line at once, instead of growing into it.
        let tokens = std::mem::replace(&mut self.current, Vec::with_capacity(12));
        check_spacing(&tokens, &std::mem::take(&mut self.mistakes), physical)?;
        self.type_depth = 0;
        let documented = self.documented.take();
        self.lines.push(Line { indentation: self.current_indentation, tokens, place: self.current_place, documented });
        Ok(())
    }

    fn push(&mut self, kind: TokenKind, place: Place, gap: Gap) {
        self.current.push(Token { kind, place, gap });
    }

    /// Whether a value just ended, so that a `-` here is between two values.
    fn after_value(&self) -> bool {
        // The `floor` of `div.floor` is part of an operator, not a value.
        if ends_with_rounding(&self.current) {
            return false;
        }
        matches!(
            self.current.last().map(|token| &token.kind),
            Some(
                TokenKind::Name(_)
                    | TokenKind::TypeName(_)
                    | TokenKind::Whole(_)
                    | TokenKind::Float { .. }
                    | TokenKind::Text(_)
                    | TokenKind::Keyword(Keyword::True | Keyword::False)
                    | TokenKind::Symbol(
                        Symbol::CloseRound | Symbol::CloseSquare | Symbol::Question | Symbol::TypeClose
                    )
            )
        )
    }

    #[allow(clippy::too_many_lines)] // one pass over a line's characters, a case per kind of token
    fn line(&mut self, number: u32, characters: &[char], start: usize, continued: bool) -> Result<(), Message> {
        let mut index = start;
        let mut first = true;
        loop {
            let mut spaces = 0u32;
            while characters.get(index) == Some(&' ') {
                spaces += 1;
                index += 1;
            }
            let Some(&character_here) = characters.get(index) else {
                break;
            };
            let gap = if self.current.is_empty() {
                Gap::LineStart
            } else if first && continued {
                Gap::LineBreak
            } else {
                Gap::Spaces(spaces)
            };
            first = false;
            let column = column_of(index);
            let place_of = |length: usize| Place::new(number, column, u32::try_from(length).unwrap_or(u32::MAX));
            match character_here {
                '\t' => {
                    // trace: D63. A tab can't be told from spaces.
                    return Err(Message::refused(place_of(1), "this line has a tab in it")
                        .with_why("spacing is written with spaces, since a tab can't be told from them."));
                }
                '\\' if characters.get(index + 1) == Some(&'\\') => {
                    // trace: D102. After `end`, `\\` starts a note naming the block it closes.
                    let after_end =
                        matches!(self.current.as_slice(), [Token { kind: TokenKind::Keyword(Keyword::End), .. }]);
                    if !after_end {
                        return Err(Message::refused(place_of(2), "`\\\\` outside text starts a note after `end`")
                            .with_why("an `end` can carry a note naming its block, as `end \\\\ proc on_hit`; nothing else uses `\\\\`."));
                    }
                    let mut rest: String = characters.get(index + 2..).unwrap_or(&[]).iter().collect();
                    // A comment after the note runs to the end of the line, as after any code.
                    if let Some(comment) = rest.find(" //") {
                        rest.truncate(comment);
                    }
                    let spaces_before = rest.chars().take_while(|&character| character == ' ').count();
                    let words: Vec<&str> = rest.split(' ').filter(|word| !word.is_empty()).collect();
                    let note = words.join(" ");
                    if spaces_before != 1 || rest.trim_start().len() != note.len() {
                        let note_column = column + 2;
                        self.mistakes.push(SpacingMistake {
                            line: number,
                            column: note_column,
                            actual: u32::try_from(spaces_before).unwrap_or(0),
                            wanted: 1,
                        });
                    }
                    let length = 2 + rest.len();
                    self.push(TokenKind::Note(note), place_of(length), gap);
                    index += length;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut end = index;
                    while characters.get(end).is_some_and(|&next| next.is_ascii_alphanumeric() || next == '_') {
                        end += 1;
                    }
                    // One room for every word, copied out only for a name, at its exact length.
                    self.word.clear();
                    self.word.extend(characters.get(index..end).unwrap_or(&[]));
                    let place = place_of(end - index);
                    let kind = word_token(&self.word, place)?;
                    self.push(kind, place, gap);
                    index = end;
                }
                '0'..='9' | '~' => {
                    let (kind, length) = number_token(number, characters, index)?;
                    self.push(kind, place_of(length), gap);
                    index += length;
                }
                '"' => {
                    let mut end = index + 1;
                    let mut text = String::new();
                    loop {
                        match characters.get(end) {
                            None => {
                                return Err(Message::refused(
                                    place_of(end - index),
                                    "this text is never closed with `\"`",
                                ));
                            }
                            Some('"') => break,
                            // trace: D63, Unio D30. `\` makes the next character literal; the text is kept as
                            // written, and read with `cook_text`, so that `fmt` can tell `\{` from `{`.
                            Some('\\') => match characters.get(end + 1) {
                                Some(&next @ ('"' | '\\' | '{')) => {
                                    text.push('\\');
                                    text.push(next);
                                    end += 1;
                                }
                                next => {
                                    // trace: D92. A habit from another language mustn't turn into a stray letter.
                                    let shown = next.map_or_else(String::new, ToString::to_string);
                                    return Err(Message::refused(
                                        Place::new(number, column_of(end), 2),
                                        format!("`\\{shown}` isn't something text can hold"),
                                    )
                                    .with_why(
                                        "in text, `\\` goes only before `\"`, `\\` or `{`; a line break is `{newln}` and a tab `{tab}`, so `\\n` isn't a new line in Seal.",
                                    ));
                                }
                            },
                            Some('\t') => {
                                // trace: D92
                                return Err(Message::refused(
                                    Place::new(number, column_of(end), 1),
                                    "this text has a tab character in it",
                                )
                                .with_why("a tab can't be told from spaces, so a tab in text is written `{tab}`."));
                            }
                            Some(&other) => text.push(other),
                        }
                        end += 1;
                    }
                    self.push(TokenKind::Text(text), place_of(end + 1 - index), gap);
                    index = end + 1;
                }
                _ => {
                    let next = characters.get(index + 1).copied();
                    let (symbol, length) = match (character_here, next) {
                        // trace: D63, Unio D74. A comment after a statement runs to the end of the line.
                        ('/', Some('/')) => {
                            if !self.open_brackets.is_empty() {
                                return Err(Message::not_designed_yet(place_of(2), "a comment inside an open bracket"));
                            }
                            match gap {
                                // trace: D91. One space before a `//` after code.
                                Gap::Spaces(actual) if actual != 1 => self.mistakes.push(SpacingMistake {
                                    line: number,
                                    column: column - actual,
                                    actual,
                                    wanted: 1,
                                }),
                                _ => {}
                            }
                            break;
                        }
                        ('=', Some('=')) => (Symbol::EqualEqual, 2),
                        ('!', Some('=')) => (Symbol::NotEqual, 2),
                        ('|', Some('>')) => (Symbol::Slide, 2),
                        ('?', Some('?')) => (Symbol::QuestionQuestion, 2),
                        ('-', Some('>')) => (Symbol::Arrow, 2),
                        (':', Some(':')) => (Symbol::Path, 2),
                        ('=', _) => (Symbol::Equals, 1),
                        ('<', _) => {
                            // A `<` touching a type's name opens its blanks; any other `<` compares (Unio D17).
                            let touching_type = gap == Gap::Spaces(0)
                                && matches!(self.current.last().map(|token| &token.kind), Some(TokenKind::TypeName(_)));
                            if touching_type {
                                self.type_depth += 1;
                                (Symbol::TypeOpen, 1)
                            } else {
                                (Symbol::Less, 1)
                            }
                        }
                        ('>', _) => {
                            if self.type_depth > 0 {
                                self.type_depth -= 1;
                                (Symbol::TypeClose, 1)
                            } else {
                                (Symbol::Greater, 1)
                            }
                        }
                        ('+', _) => (Symbol::Plus, 1),
                        ('-', _) => {
                            if self.after_value() {
                                (Symbol::Minus, 1)
                            } else {
                                (Symbol::Negate, 1)
                            }
                        }
                        ('*', Some('*')) => (Symbol::Power, 2),
                        ('*', _) => (Symbol::Star, 1),
                        ('/', _) => (Symbol::Slash, 1),
                        ('&', _) => (Symbol::Ampersand, 1),
                        ('?', _) => (Symbol::Question, 1),
                        ('!', _) => (Symbol::Bang, 1),
                        ('(', _) => (Symbol::OpenRound, 1),
                        (')', _) => (Symbol::CloseRound, 1),
                        ('[', _) => (Symbol::OpenSquare, 1),
                        (']', _) => (Symbol::CloseSquare, 1),
                        (',', _) => (Symbol::Comma, 1),
                        (':', _) => (Symbol::Colon, 1),
                        ('.', _) => (Symbol::Dot, 1),
                        (';', _) => {
                            // trace: D8
                            return Err(Message::refused(place_of(1), "a statement has no `;`")
                                .with_why("the end of a line ends its statement."));
                        }
                        ('$', _) => {
                            // trace: D7
                            return Err(Message::refused(place_of(1), "a name has no `$`")
                                .with_why("Seal's names are plain words."));
                        }
                        (other, _) if !other.is_ascii() => {
                            // trace: D93. Letters beyond plain English go only in text, for now.
                            return Err(Message::refused(
                                place_of(1),
                                format!("`{other}` is beyond the plain English letters a script is written in"),
                            )
                            .with_why("outside text, a script is written in plain English letters, digits and symbols, for now."));
                        }
                        (other, _) => {
                            return Err(Message::refused(place_of(1), format!("`{other}` can't be read here")));
                        }
                    };
                    let place = place_of(length);
                    match symbol {
                        Symbol::OpenRound | Symbol::OpenSquare => {
                            self.open_brackets.push((symbol, place));
                        }
                        Symbol::CloseRound | Symbol::CloseSquare => {
                            let wanted =
                                if symbol == Symbol::CloseRound { Symbol::OpenRound } else { Symbol::OpenSquare };
                            match self.open_brackets.pop() {
                                Some((open, _)) if open == wanted => {}
                                Some((open, open_place)) => {
                                    return Err(Message::refused(
                                        place,
                                        format!("`{}` closes a `{}`", symbol.text(), open.text()),
                                    )
                                    .with_because(format!("line {} opens it", open_place.line), open_place));
                                }
                                None => {
                                    return Err(Message::refused(
                                        place,
                                        format!("this `{}` closes nothing", symbol.text()),
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                    self.push(TokenKind::Symbol(symbol), place, gap);
                    index += length;
                }
            }
        }
        Ok(())
    }
}

/// A word: a keyword, a name, or a type's name (D7, D24).
fn word_token(word: &str, place: Place) -> Result<TokenKind, Message> {
    let first = word.chars().next().unwrap_or('_');
    if first.is_ascii_uppercase() {
        if word.contains('_') {
            // trace: D93
            let fixed: String = word
                .split('_')
                .map(|part| {
                    let mut letters = part.chars();
                    letters
                        .next()
                        .map_or_else(String::new, |first| first.to_ascii_uppercase().to_string() + letters.as_str())
                })
                .collect();
            return Err(Message::refused(place, format!("a type's name has no `_`, so `{word}` is written `{fixed}`"))
                .with_why("a type's name joins its words with capitals, as `HighScore`."));
        }
        return Ok(TokenKind::TypeName(word.to_owned()));
    }
    if let Some(keyword) = Keyword::from_word(word) {
        return Ok(TokenKind::Keyword(keyword));
    }
    check_name_spelling(word, place)?;
    Ok(TokenKind::Name(word.to_owned()))
}

/// D24's spelling: lowercase English letters, digits, and `_` between words, starting with a letter.
///
/// # Errors
///
/// Returns a refusal saying which part of the spelling is wrong.
pub fn check_name_spelling(word: &str, place: Place) -> Result<(), Message> {
    // trace: D24
    let why = "a name is lowercase letters, digits, and `_` between words, starting with a letter.";
    let problem = if word.is_empty() {
        Some("is empty")
    } else if word.starts_with('_') {
        Some("starts with `_`")
    } else if word.starts_with(|character: char| character.is_ascii_digit()) {
        Some("starts with a digit")
    } else if word.ends_with('_') {
        Some("ends with `_`")
    } else if word.contains("__") {
        Some("has two `_` in a row")
    } else if word.chars().any(|character| character.is_ascii_uppercase()) {
        Some("has a capital letter")
    } else if word
        .chars()
        .any(|character| !(character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'))
    {
        Some("has a character that isn't a lowercase letter, a digit or `_`")
    } else {
        None
    };
    match problem {
        Some(problem) => Err(Message::refused(place, format!("the name `{word}` {problem}")).with_why(why)),
        None => Ok(()),
    }
}

/// A number: digits, with a point for a `Float` (Unio D180) and `~` in front of an inexact one (Unio D155).
fn number_token(number: u32, characters: &[char], start: usize) -> Result<(TokenKind, usize), Message> {
    let approximate = characters.get(start) == Some(&'~');
    let digits_start = if approximate { start + 1 } else { start };
    let mut end = digits_start;
    while characters.get(end).is_some_and(char::is_ascii_digit) {
        end += 1;
    }
    let mut is_float = false;
    if characters.get(end) == Some(&'.') && characters.get(end + 1).is_some_and(char::is_ascii_digit) {
        is_float = true;
        end += 1;
        while characters.get(end).is_some_and(char::is_ascii_digit) {
            end += 1;
        }
    }
    let length = end - start;
    let place = Place::new(number, column_of(start), u32::try_from(length).unwrap_or(u32::MAX));
    let text: String = characters.get(digits_start..end).unwrap_or(&[]).iter().collect();
    if characters.get(end).is_some_and(|&next| next.is_ascii_alphabetic() || next == '_') {
        return Err(Message::refused(place, "a letter comes right after this number"));
    }
    if end == digits_start {
        return Err(Message::refused(place, "`~` goes in front of a number with a point, as in `~0.1`"));
    }
    let whole_part: &str = text.split('.').next().unwrap_or("");
    if whole_part.len() > 1 && whole_part.starts_with('0') {
        // trace: D93
        let trimmed = whole_part.trim_start_matches('0');
        let shown = if trimmed.is_empty() { "0" } else { trimmed };
        return Err(Message::refused(place, format!("a number has no leading zero, so this is written `{shown}`"))
            .with_why("a number is written without zeros in front, as `7`, not `007`."));
    }
    if is_float {
        Ok((TokenKind::Float { text, approximate }, length))
    } else if approximate {
        Err(Message::refused(place, "`~` goes in front of a number with a point, as in `~0.1`")
            .with_why("only a `Float` can be inexact."))
    } else {
        Ok((TokenKind::Whole(text), length))
    }
}

/// What a text holds, from how it's written: `\` makes the next character literal (D63, Unio D30).
#[must_use]
pub fn cook_text(written: &str) -> String {
    let mut text = String::with_capacity(written.len());
    let mut characters = written.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            text.extend(characters.next());
        } else {
            text.push(character);
        }
    }
    text
}

/// Checks the spaces between a statement's tokens, with the mistakes already found on its lines.
///
/// Spacing is fixed, as Unio's table has it (D63, Unio D98). Every mistake on a line makes one message, with a `^`
/// under each gap and the whole corrected line as the fix (Unio D100).
fn check_spacing(tokens: &[Token], found: &[SpacingMistake], physical: &[&str]) -> Result<(), Message> {
    let mut mistakes = found.to_vec();
    let header = header_brackets(tokens);
    for (index, pair) in tokens.windows(2).enumerate() {
        let [before, after] = pair else { continue };
        let Gap::Spaces(actual) = after.gap else {
            continue;
        };
        // trace: D88. A parameter group's `:` has a space on each side; the parser tells a group's `:` from a single
        // name's, so here either spacing passes.
        if after.kind == TokenKind::Symbol(Symbol::Colon)
            && header.is_some_and(|(open, close)| open < index + 1 && index + 1 < close)
            && actual == 1
        {
            continue;
        }
        // `div.floor` and its like are operators, with a space on each side like any other (Unio D70, D98).
        let after_operator = ends_with_rounding(tokens.get(..=index).unwrap_or(&[]));
        // trace: D87. `labels.any_order [ … ]` keeps the space `labels [ … ]` has.
        let after_labels = index == 2
            && after.kind == TokenKind::Symbol(Symbol::OpenSquare)
            && tokens.first().map(|token| &token.kind) == Some(&TokenKind::Keyword(Keyword::Labels));
        // trace: D110. `handles [ … ]`, a handler group's first line, is spaced as `labels [ … ]` is.
        let after_handles = index == 0
            && after.kind == TokenKind::Symbol(Symbol::OpenSquare)
            && matches!(&before.kind, TokenKind::Name(word) if word == "handles");
        // trace: D110. `::*give_up` marks a handler that can jump, with the `*` against its name.
        let jump_mark = index >= 1
            && before.kind == TokenKind::Symbol(Symbol::Star)
            && tokens.get(index - 1).map(|token| &token.kind) == Some(&TokenKind::Symbol(Symbol::Path));
        let wanted = if jump_mark {
            0
        } else if after_operator || after_labels || after_handles {
            1
        } else {
            wanted_spaces(&before.kind, &after.kind)
        };
        if actual != wanted {
            mistakes.push(SpacingMistake {
                line: before.place.line,
                column: before.place.column + before.place.width,
                actual,
                wanted,
            });
        }
    }
    let Some(line) = mistakes.iter().map(|mistake| mistake.line).min() else { return Ok(()) };
    mistakes.retain(|mistake| mistake.line == line);
    mistakes.sort_by_key(|mistake| mistake.column);
    let mut corrected: Vec<char> = physical.get(line.saturating_sub(1) as usize).unwrap_or(&"").chars().collect();
    // The gaps are put right from the line's end backwards, so the places of the earlier ones stay true.
    for mistake in mistakes.iter().rev() {
        let start = (mistake.column.saturating_sub(1) as usize).min(corrected.len());
        let end = (start + mistake.actual as usize).min(corrected.len());
        corrected.splice(start..end, std::iter::repeat_n(' ', mistake.wanted as usize));
    }
    let mut places = mistakes.iter().map(|mistake| Place::new(mistake.line, mistake.column, mistake.actual.max(1)));
    let first = places.next().unwrap_or_default();
    Err(Message::refused(first, "this line's spacing isn't the fixed spacing")
        .with_more_places(places.collect())
        .with_fix(corrected.into_iter().collect::<String>())
        .with_why(
            "spacing is fixed, so every script reads alike: one space around operators and after `,` and `:`, and \
             none inside brackets or at a line's end.",
        ))
}

/// Where a procedure's header line has its parameters' `[ ]`, by the tokens' places in the line: a line that's `proc`
/// and a name, or a family member's name, followed by `[ … ]` and nothing but `->` or `!` after it (D21, D88, D97).
fn header_brackets(tokens: &[Token]) -> Option<(usize, usize)> {
    // A handler's header starts with `::` (D110).
    let mut index = usize::from(matches!(
        tokens.first().map(|token| &token.kind),
        Some(TokenKind::Keyword(Keyword::Proc) | TokenKind::Symbol(Symbol::Path))
    ));
    if !matches!(tokens.get(index).map(|token| &token.kind), Some(TokenKind::Name(_))) {
        return None;
    }
    index += 1;
    while matches!(tokens.get(index).map(|token| &token.kind), Some(TokenKind::Symbol(Symbol::Dot)))
        && matches!(tokens.get(index + 1).map(|token| &token.kind), Some(TokenKind::Name(_)))
    {
        index += 2;
    }
    if tokens.get(index).map(|token| &token.kind) != Some(&TokenKind::Symbol(Symbol::OpenSquare)) {
        return None;
    }
    let open = index;
    let mut depth = 0u32;
    for (at, token) in tokens.iter().enumerate().skip(open) {
        match token.kind {
            TokenKind::Symbol(Symbol::OpenSquare | Symbol::OpenRound) => depth += 1,
            TokenKind::Symbol(Symbol::CloseSquare | Symbol::CloseRound) => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let after = tokens.get(at + 1).map(|token| &token.kind);
                    let header = matches!(after, None | Some(TokenKind::Symbol(Symbol::Arrow | Symbol::Bang)));
                    return header.then_some((open, at));
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether the tokens end with `div.floor`, `div.trunc`, `rem.floor` or `rem.trunc` (Unio D70, D71).
fn ends_with_rounding(tokens: &[Token]) -> bool {
    let [.., word, dot, rounding] = tokens else { return false };
    matches!(word.kind, TokenKind::Keyword(Keyword::Div | Keyword::Rem))
        && dot.kind == TokenKind::Symbol(Symbol::Dot)
        && matches!(&rounding.kind, TokenKind::Name(name) if name == "floor" || name == "trunc")
}

fn wanted_spaces(before: &TokenKind, after: &TokenKind) -> u32 {
    use Symbol::{
        CloseRound, CloseSquare, Colon, Comma, Dot, Negate, OpenRound, OpenSquare, Question, TypeClose, TypeOpen,
    };
    if let TokenKind::Symbol(Comma | Colon | CloseRound | CloseSquare | Dot | Question | TypeOpen | TypeClose) = after {
        return 0;
    }
    if let TokenKind::Symbol(OpenRound | OpenSquare | Dot | TypeOpen | Negate | Symbol::Path) = before {
        return 0;
    }
    // trace: D110, Unio D43. A group's name touches its `::`; after `??` or `include`, the `::` stands apart.
    if *after == TokenKind::Symbol(Symbol::Path) {
        return u32::from(!matches!(before, TokenKind::Name(name) if name != "include"));
    }
    let touches_brackets = matches!(
        before,
        TokenKind::Name(_) | TokenKind::TypeName(_) | TokenKind::Symbol(TypeClose | CloseSquare | CloseRound)
    );
    match after {
        // A call's `(` touches the procedure's name; a grouping `(` stands apart.
        TokenKind::Symbol(OpenRound)
            if matches!(before, TokenKind::Name(_) | TokenKind::TypeName(_) | TokenKind::Symbol(TypeClose)) =>
        {
            0
        }
        // The `[` of parameters, items reached, settings and building touches what it follows.
        TokenKind::Symbol(OpenSquare) if touches_brackets => 0,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageKind;

    fn kinds(source: &str) -> Vec<TokenKind> {
        lex(source)
            .map(|lines| lines.into_iter().flat_map(|line| line.tokens).map(|token| token.kind).collect())
            .unwrap_or_default()
    }

    // trace: D8
    #[test]
    fn d08_a_line_carries_on_inside_an_open_bracket() {
        let lines = lex("total = (first +\n    second)\nmut x = 1\n").unwrap_or_default();
        assert_eq!(lines.len(), 2);
    }

    // trace: D24
    #[test]
    fn d24_names_and_types() {
        assert_eq!(
            kinds("hp_max: Whole = 3"),
            vec![
                TokenKind::Name("hp_max".into()),
                TokenKind::Symbol(Symbol::Colon),
                TokenKind::TypeName("Whole".into()),
                TokenKind::Symbol(Symbol::Equals),
                TokenKind::Whole("3".into()),
            ]
        );
        assert!(lex("getService = 3").is_err());
        assert!(lex("hp__max = 3").is_err());
    }

    #[test]
    fn d16_negating_and_subtracting() {
        assert_eq!(
            kinds("a - -1"),
            vec![
                TokenKind::Name("a".into()),
                TokenKind::Symbol(Symbol::Minus),
                TokenKind::Symbol(Symbol::Negate),
                TokenKind::Whole("1".into()),
            ]
        );
    }

    // trace: D63
    #[test]
    fn d63_spacing_mistakes_are_refused_with_the_line_put_right() {
        let message = lex("a=1+2 ").err();
        assert_eq!(message.as_ref().map(|message| message.kind), Some(MessageKind::Refused));
        assert_eq!(message.and_then(|message| message.fix.clone()).as_deref(), Some("a = 1 + 2"));
        let message = lex("show_msg( \"three\" )").err();
        assert_eq!(message.and_then(|message| message.fix.clone()).as_deref(), Some("show_msg(\"three\")"));
    }

    // trace: D63
    #[test]
    fn d63_comments_blank_lines_and_line_endings() {
        assert_eq!(kinds("// hi\na = 1 // one\r\n"), kinds("a = 1"));
        assert_eq!(kinds("comment old\n    broken ((( stuff\n\n\n\tend\nend\na = 1"), kinds("a = 1"));
        assert_eq!(kinds("/// Doubles.\nproc double[n]"), kinds("proc double[n]"));
        let kind = |source: &str| lex(source).err().map(|message| message.kind);
        assert_eq!(kind("a = 1\n\n\nb = 2"), Some(MessageKind::Refused));
        assert_eq!(kind("a = 1\n\nb = 2"), None);
        assert_eq!(kind("/// Nothing after."), Some(MessageKind::Refused));
        assert_eq!(kind("comment never\na = 1"), Some(MessageKind::Refused));
        assert_eq!(kind("\ta = 1"), Some(MessageKind::Refused));
        assert_eq!(kind("a = 1 \r b"), Some(MessageKind::NotDesignedYet));
        // trace: D91, D106, Unio D145
        assert_eq!(kind("x = 1// hi"), Some(MessageKind::Refused));
        assert_eq!(kind("total = (1 +\n    // the second\n    2)"), None);
        assert_eq!(kind("total = (1 +\n        2)"), Some(MessageKind::Refused));
        assert_eq!(kind("    // misplaced\nx = 1"), Some(MessageKind::Refused));
        assert_eq!(kind("if x\n    y = 1\n    // the end of the way\nend"), None);
    }

    // trace: D102
    #[test]
    fn d102_an_end_may_carry_a_note() {
        assert_eq!(kinds("end \\\\ if"), vec![TokenKind::Keyword(Keyword::End), TokenKind::Note("if".into())]);
        assert_eq!(
            kinds("end \\\\ proc on_hit // done"),
            vec![TokenKind::Keyword(Keyword::End), TokenKind::Note("proc on_hit".into())]
        );
        assert!(lex("x = 1 \\\\ no").is_err());
        assert_eq!(kinds("comment old\n    stuff\nend \\\\ comment old\na = 1"), kinds("a = 1"));
        assert!(lex("comment old\n    stuff\nend \\\\ comment other\n").is_err());
    }

    // trace: D63
    #[test]
    fn d63_a_backslash_makes_the_next_character_literal() {
        assert_eq!(cook_text(r#"She said \"hi\" and \\ left \{"#), "She said \"hi\" and \\ left {");
        // trace: D92
        let kind = lex(r#"a = "new\nline""#).err().map(|message| message.kind);
        assert_eq!(kind, Some(MessageKind::Refused));
    }
}
