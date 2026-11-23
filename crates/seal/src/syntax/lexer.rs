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
}

/// Splits a script's text into logical lines of tokens.
///
/// # Errors
///
/// Returns the first thing that can't be read, as a message.
pub fn lex(source: &str) -> Result<Vec<Line>, Message> {
    let mut lexer = Lexer {
        lines: Vec::new(),
        current: Vec::new(),
        current_indentation: 0,
        current_place: Place::default(),
        open_brackets: Vec::new(),
        type_depth: 0,
    };
    let mut physical: Vec<&str> = source.split('\n').collect();
    // A file's last line break ends its last line; it doesn't start an empty one.
    if physical.last() == Some(&"") {
        physical.pop();
    }
    let mut blank_run = 0u32;
    for (index, text) in physical.iter().enumerate() {
        let number = u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1);
        let characters: Vec<char> = text.chars().collect();
        let width = u32::try_from(characters.len()).unwrap_or(u32::MAX);
        if let Some(position) = characters.iter().position(|&character| character == '\r') {
            return Err(Message::not_designed_yet(
                Place::new(number, column_of(position), 1),
                "a line ending other than a plain line break",
            ));
        }
        if characters.is_empty() {
            blank_run += 1;
            if !lexer.open_brackets.is_empty() {
                return Err(Message::not_designed_yet(Place::new(number, 1, 1), "a blank line inside an open bracket"));
            }
            if blank_run > 1 {
                return Err(Message::not_designed_yet(Place::new(number, 1, 1), "two or more blank lines in a row"));
            }
            continue;
        }
        blank_run = 0;
        let leading = characters.iter().take_while(|&&character| character == ' ').count();
        if leading == characters.len() {
            return Err(Message::not_designed_yet(Place::new(number, 1, width), "a line of only spaces"));
        }
        if characters.get(leading) == Some(&'\t') {
            return Err(Message::not_designed_yet(Place::new(number, column_of(leading), 1), "a tab in indentation"));
        }
        if characters.last() == Some(&' ') {
            let trailing = characters.iter().rev().take_while(|&&character| character == ' ').count();
            return Err(Message::not_designed_yet(
                Place::new(number, column_of(characters.len() - trailing), u32::try_from(trailing).unwrap_or(1)),
                "spaces at the end of a line",
            ));
        }
        let continued = !lexer.open_brackets.is_empty();
        if !continued {
            lexer.current_indentation = u32::try_from(leading).unwrap_or(u32::MAX);
            lexer.current_place = Place::new(number, column_of(leading), width.saturating_sub(column_of(leading) - 1));
        }
        lexer.line(number, &characters, leading, continued)?;
        if lexer.open_brackets.is_empty() {
            lexer.finish_line()?;
        }
    }
    if let Some((symbol, place)) = lexer.open_brackets.last() {
        return Err(Message::refused(*place, format!("this `{}` is never closed", symbol.text())));
    }
    Ok(lexer.lines)
}

fn column_of(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX - 1).saturating_add(1)
}

impl Lexer {
    fn finish_line(&mut self) -> Result<(), Message> {
        let tokens = std::mem::take(&mut self.current);
        check_spacing(&tokens)?;
        self.type_depth = 0;
        self.lines.push(Line { indentation: self.current_indentation, tokens, place: self.current_place });
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
                    return Err(Message::not_designed_yet(place_of(1), "a tab"));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut end = index;
                    while characters.get(end).is_some_and(|&next| next.is_ascii_alphanumeric() || next == '_') {
                        end += 1;
                    }
                    let word: String = characters.get(index..end).unwrap_or(&[]).iter().collect();
                    let place = place_of(end - index);
                    let kind = word_token(&word, place)?;
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
                            Some('\\') => {
                                return Err(Message::not_designed_yet(
                                    Place::new(number, column_of(end), 1),
                                    "a `\\` inside text",
                                ));
                            }
                            Some('\t') => {
                                return Err(Message::not_designed_yet(
                                    Place::new(number, column_of(end), 1),
                                    "a tab inside text",
                                ));
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
                        ('/', Some('/')) => {
                            return Err(Message::not_designed_yet(place_of(2), "a comment"));
                        }
                        ('=', Some('=')) => (Symbol::EqualEqual, 2),
                        ('!', Some('=')) => (Symbol::NotEqual, 2),
                        ('|', Some('>')) => (Symbol::Slide, 2),
                        ('?', Some('?')) => (Symbol::QuestionQuestion, 2),
                        ('-', Some('>')) => (Symbol::Arrow, 2),
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
                            return Err(Message::not_designed_yet(
                                place_of(1),
                                format!("`{other}` outside text, a character beyond plain English ones,"),
                            ));
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
            return Err(Message::not_designed_yet(place, "a `_` in a type's name"));
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
        return Err(Message::not_designed_yet(place, "a number written with a leading zero"));
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

/// Checks the spaces between a statement's tokens.
///
/// Seal's spacing isn't decided (the standard, chapter 2). Unio's fixed spacing (Unio D98) is the one form every
/// example and test uses, so that form is read, and anything else stops as not designed yet.
fn check_spacing(tokens: &[Token]) -> Result<(), Message> {
    for (index, pair) in tokens.windows(2).enumerate() {
        let [before, after] = pair else { continue };
        let Gap::Spaces(actual) = after.gap else {
            continue;
        };
        // `div.floor` and its like are operators, with a space on each side like any other (Unio D70, D98).
        let after_operator = ends_with_rounding(tokens.get(..=index).unwrap_or(&[]));
        let wanted = if after_operator { 1 } else { wanted_spaces(&before.kind, &after.kind) };
        if actual != wanted {
            let column = before.place.column + before.place.width;
            let place = Place::new(before.place.line, column, actual.max(1));
            return Err(Message::not_designed_yet(
                place,
                format!(
                    "spacing other than Unio's, which has {} here between {} and {},",
                    match wanted {
                        0 => "no space",
                        _ => "one space",
                    },
                    before.kind.describe(),
                    after.kind.describe()
                ),
            ));
        }
    }
    Ok(())
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
    if let TokenKind::Symbol(OpenRound | OpenSquare | Dot | TypeOpen | Negate) = before {
        return 0;
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
        let lines = lex("total = (first +\n         second)\nmut x = 1\n").unwrap_or_default();
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

    #[test]
    fn undecided_spacing_is_not_designed_yet() {
        let error = lex("a = 1+2").err().map(|message| message.kind);
        assert_eq!(error, Some(MessageKind::NotDesignedYet));
        let error = lex("// hi").err().map(|message| message.kind);
        assert_eq!(error, Some(MessageKind::NotDesignedYet));
    }
}
