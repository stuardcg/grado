//! Text that is never blank (ADR 0007).

/// A string with at least one non-whitespace character, kept as written.
///
/// ```
/// use shared::text::NonBlankString;
///
/// assert_eq!(NonBlankString::new("a").map(|text| text.into_string()), Some("a".to_owned()));
/// assert_eq!(NonBlankString::new(" \n"), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonBlankString(String);

impl NonBlankString {
    /// Returns `None` when `text` is empty or only whitespace.
    pub fn new(text: impl Into<String>) -> Option<Self> {
        let text = text.into();
        if text.trim().is_empty() { None } else { Some(Self(text)) }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::NonBlankString;

    #[test]
    fn empty_text_is_refused() {
        assert_eq!(NonBlankString::new(""), None);
    }

    #[test]
    fn whitespace_only_text_is_refused() {
        for text in [" ", "\t", "\n", " \t\r\n "] {
            assert_eq!(NonBlankString::new(text), None, "{text:?}");
        }
    }

    #[test]
    fn text_is_kept_as_written() {
        let text = NonBlankString::new("  a \n").expect("not blank");
        assert_eq!(text.as_str(), "  a \n");
        assert_eq!(text.into_string(), "  a \n");
    }
}
