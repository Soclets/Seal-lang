//! Messages: what Seal tells the host about a script that was refused or stopped.
//!
//! Every message goes to the host two ways (D26): as its parts, the fields of [`Message`], and as ready-made text in
//! Unio's layout (Unio D54, D86), from [`Message::text`].

use std::fmt::{self, Write};

/// Where issues with Seal itself are reported (Building rule 3).
pub const REPORT_ADDRESS: &str = "https://github.com/Soclets/Seal-lang/issues";

/// The first word of a message: what kind of thing happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    /// The script was rejected at load time (D17, D26).
    Refused,
    /// A check failed while the script was running (D17, D18, D26).
    CheckFailed,
    /// `?? crash` stopped the script on a failure (Unio D152). A draft: the kind's wording isn't decided for Seal.
    Failed,
    /// The step budget or the memory cap was reached (D14). A draft: this kind isn't decided yet.
    LimitReached,
    /// Something decided but not built yet (Building rule 11).
    NotBuiltYet,
    /// Something no decision covers (Building rule 11).
    NotDesignedYet,
    /// Seal itself went wrong (Building rule 3).
    Internal,
}

impl MessageKind {
    /// The kind as it's written at the start of a message.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            MessageKind::Refused => "refused",
            MessageKind::CheckFailed => "check failed",
            MessageKind::Failed => "failed",
            MessageKind::LimitReached => "limit reached",
            MessageKind::NotBuiltYet => "NotBuiltYet",
            MessageKind::NotDesignedYet => "NotDesignedYet",
            MessageKind::Internal => "internal error",
        }
    }
}

/// A place in a script: a line, a column, and how many characters wide the part in question is.
/// Lines and columns count from 1, and columns count characters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Place {
    pub line: u32,
    pub column: u32,
    pub width: u32,
}

impl Place {
    #[must_use]
    pub fn new(line: u32, column: u32, width: u32) -> Self {
        Place { line, column, width }
    }

    /// The place from the start of `self` to the end of `other`, when both are on one line; otherwise `self`.
    #[must_use]
    pub fn to(self, other: Place) -> Place {
        if self.line == other.line && other.column >= self.column {
            Place {
                line: self.line,
                column: self.column,
                width: other.column.saturating_add(other.width).saturating_sub(self.column),
            }
        } else {
            self
        }
    }
}

/// The earlier place a message comes from, shown after `because:` (Unio D54).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Because {
    pub sentence: String,
    pub place: Place,
    pub source_line: String,
}

/// One message (D26). Its parts are the fields of [`MessageParts`], read straight from the message, as in
/// `message.kind`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    // Boxed so that passing a message around as an error stays cheap on the path where nothing goes wrong.
    parts: Box<MessageParts>,
}

/// A message's parts (D26).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageParts {
    pub kind: MessageKind,
    pub file: String,
    pub place: Place,
    /// The script's line the message is about, as written. Empty when the message has no line.
    pub source_line: String,
    /// The whole corrected line, when there's exactly one fix (Unio D84, D101).
    pub fix: Option<String>,
    pub what: Option<String>,
    pub when: Option<String>,
    pub because: Option<Because>,
    pub why: Option<String>,
}

impl std::ops::Deref for Message {
    type Target = MessageParts;

    fn deref(&self) -> &MessageParts {
        &self.parts
    }
}

impl std::ops::DerefMut for Message {
    fn deref_mut(&mut self) -> &mut MessageParts {
        &mut self.parts
    }
}

impl Message {
    #[must_use]
    pub fn new(kind: MessageKind, place: Place, what: impl Into<String>) -> Self {
        Message {
            parts: Box::new(MessageParts {
                kind,
                file: String::new(),
                place,
                source_line: String::new(),
                fix: None,
                what: Some(what.into()),
                when: None,
                because: None,
                why: None,
            }),
        }
    }

    #[must_use]
    pub fn refused(place: Place, what: impl Into<String>) -> Self {
        Message::new(MessageKind::Refused, place, what)
    }

    /// A stop for something no decision covers: `what` names it, and the message says it isn't decided yet
    /// (Unio D103's pattern).
    #[must_use]
    pub fn not_designed_yet(place: Place, what: impl AsRef<str>) -> Self {
        Message::new(MessageKind::NotDesignedYet, place, format!("{} isn't decided yet", what.as_ref()))
    }

    /// A stop for something decided but not built: `what` names it with its decisions (Unio D103's pattern).
    #[must_use]
    pub fn not_built_yet(place: Place, what: impl AsRef<str>) -> Self {
        Message::new(MessageKind::NotBuiltYet, place, format!("{} built yet", what.as_ref()))
    }

    /// Seal itself went wrong: says so, and how to report it (Building rule 3).
    #[must_use]
    pub fn internal(detail: impl AsRef<str>) -> Self {
        let mut message = Message::new(
            MessageKind::Internal,
            Place::default(),
            format!("Seal itself went wrong: {}", detail.as_ref()),
        );
        message.why = Some(format!(
            "this is a bug in Seal. Please report it at {REPORT_ADDRESS}, with the script that caused it."
        ));
        message
    }

    #[must_use]
    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }

    #[must_use]
    pub fn with_when(mut self, when: impl Into<String>) -> Self {
        self.when = Some(when.into());
        self
    }

    #[must_use]
    pub fn with_why(mut self, why: impl Into<String>) -> Self {
        self.why = Some(why.into());
        self
    }

    #[must_use]
    pub fn with_because(mut self, sentence: impl Into<String>, place: Place) -> Self {
        self.because = Some(Because { sentence: sentence.into(), place, source_line: String::new() });
        self
    }

    /// Fills in the file's name and the source lines the message shows.
    pub fn locate(&mut self, file: &str, source: &str) {
        file.clone_into(&mut self.file);
        self.source_line = source_line(source, self.place.line);
        if let Some(because) = &mut self.because {
            because.source_line = source_line(source, because.place.line);
        }
    }

    /// The message as ready-made text, in Unio's layout (D26, Unio D54, D86).
    #[must_use]
    pub fn text(&self) -> String {
        let mut text = String::new();
        // Writing to a `String` can't fail.
        let _ = self.write_text(&mut text);
        text
    }

    fn write_text(&self, out: &mut String) -> fmt::Result {
        if self.place.line == 0 {
            writeln!(out, "{}: {}", self.kind.word(), self.file)?;
        } else {
            writeln!(
                out,
                "{}: {}:{}:{}, line {} column {}",
                self.kind.word(),
                self.file,
                self.place.line,
                self.place.column,
                self.place.line,
                self.place.column
            )?;
            write_source_line(out, self.place, &self.source_line)?;
        }
        // Unio D101: `fix:` stands in place of `what:` when there's exactly one fix that can be shown as a line.
        if let Some(fix) = &self.fix {
            writeln!(out, " {:<8} `{}`", "fix:", fix)?;
        } else if let Some(what) = &self.what {
            writeln!(out, " {:<8} {}", "what:", what)?;
        }
        if let Some(when) = &self.when {
            writeln!(out, " {:<8} {}", "when:", when)?;
        }
        if let Some(because) = &self.because {
            writeln!(out, " {:<8} {}", "because:", because.sentence)?;
            write_source_line(out, because.place, &because.source_line)?;
        }
        if let Some(why) = &self.why {
            writeln!(out, " {:<8} {}", "why:", why)?;
        }
        Ok(())
    }
}

impl std::error::Error for Message {}

impl std::error::Error for Messages {}

impl fmt::Display for Message {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text())
    }
}

fn write_source_line(out: &mut String, place: Place, line: &str) -> fmt::Result {
    let number = place.line.to_string();
    writeln!(out, " {number} | {line}")?;
    let gutter = " ".repeat(number.len());
    let indent = " ".repeat(place.column.saturating_sub(1) as usize);
    let marks = "^".repeat(place.width.max(1) as usize);
    writeln!(out, " {gutter} | {indent}{marks}")
}

/// Line `number` of `source`, counting from 1, or nothing if there's no such line.
fn source_line(source: &str, number: u32) -> String {
    if number == 0 {
        return String::new();
    }
    source.split('\n').nth((number - 1) as usize).unwrap_or("").to_owned()
}

/// Every message from one load, most often refusals (D17, D26).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Messages {
    pub list: Vec<Message>,
}

impl Messages {
    #[must_use]
    pub fn one(message: Message) -> Self {
        Messages { list: vec![message] }
    }

    /// All the messages as text, one after another.
    #[must_use]
    pub fn text(&self) -> String {
        let mut text = String::new();
        for message in &self.list {
            text.push_str(&message.text());
        }
        text
    }
}

impl fmt::Display for Messages {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace: D26
    #[test]
    fn d26_message_layout() {
        let source = "mut tries = 0\nmut won = false\n\n    tires = tries + 1\n";
        let mut message = Message::refused(Place::new(4, 5, 5), "`tires` is never used");
        message.locate("goblin.seal", source);
        assert_eq!(
            message.text(),
            "refused: goblin.seal:4:5, line 4 column 5\n 4 |     tires = tries + 1\n   |     ^^^^^\n what:    `tires` is never used\n"
        );
    }
}
