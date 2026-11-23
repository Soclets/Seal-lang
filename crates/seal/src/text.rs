//! Text as a person sees it: graphemes, code points and bytes (D65, D66).
//!
//! A grapheme (in full, an extended grapheme cluster) is what a person sees as one character: 👍🏽 is one grapheme,
//! two Unicode code points and eight UTF-8 bytes. Where a grapheme ends comes from Unicode's tables, and each Seal
//! version pins one Unicode version, so a text's length never changes within a version (D50, D65).

use unicode_segmentation::UnicodeSegmentation;

/// The Unicode version this Seal version pins (D65). It changes only with a new Seal version, as a listed change.
pub const UNICODE_VERSION: (u64, u64, u64) = unicode_segmentation::UNICODE_VERSION;

/// Whether every byte of a text is a grapheme of its own: plain English characters, with no `\r`, which joins a
/// line break after it into one grapheme.
fn every_byte_is_a_grapheme(text: &str) -> bool {
    text.is_ascii() && !text.as_bytes().contains(&b'\r')
}

/// How many graphemes a text has.
#[must_use]
pub fn grapheme_count(text: &str) -> usize {
    if every_byte_is_a_grapheme(text) { text.len() } else { text.graphemes(true).count() }
}

/// The grapheme at `offset`, counting from 0, when the text has one there.
#[must_use]
pub fn grapheme_at(text: &str, offset: usize) -> Option<&str> {
    if every_byte_is_a_grapheme(text) {
        text.get(offset..offset.checked_add(1)?)
    } else {
        text.graphemes(true).nth(offset)
    }
}

/// A text's graphemes, in order.
pub fn graphemes(text: &str) -> impl Iterator<Item = &str> {
    text.graphemes(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace: D65, D66
    #[test]
    fn d66_a_grapheme_is_what_a_person_sees() {
        let thumb = "👍🏽";
        assert_eq!(grapheme_count(thumb), 1);
        assert_eq!(thumb.chars().count(), 2);
        assert_eq!(thumb.len(), 8);
        assert_eq!(grapheme_count("héllo"), 5);
        assert_eq!(grapheme_count("e\u{301}"), 1);
        assert_eq!(grapheme_count("a\r\nb"), 3);
        assert_eq!(grapheme_at("a👍🏽b", 1), Some(thumb));
        assert_eq!(grapheme_at("abc", 2), Some("c"));
        assert_eq!(grapheme_at("abc", 3), None);
        assert_eq!(graphemes("a👍🏽").collect::<Vec<_>>(), vec!["a", thumb]);
    }

    // trace: D65
    #[test]
    fn d65_one_unicode_version_is_pinned() {
        assert_eq!(UNICODE_VERSION, (17, 0, 0));
    }
}
