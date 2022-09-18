//! Text processing utilities.

/// Extension trait for [`String`] and [`str`] with text processing utilities.
pub trait TextProcessExt {
    /// Remove Discord markdown from the String.
    ///
    /// This function removes **all** characters that can be used to format
    /// markdown. It does not check if it is valid markdown.
    fn remove_markdown(&self) -> String;

    /// Truncate text if it exceed a maximum size.
    ///
    /// Truncated characters will be replaced with `...` (without exceeding the
    /// maximum size). Maximum size must be larger at least 3 characters.
    ///
    /// ```
    /// # use raidprotect_util::text::TextProcessExt;
    /// assert_eq!("this will be truncated".to_owned().truncate(10), "this wi...".to_owned());
    /// assert_eq!("this not".to_owned().truncate(10), "this not".to_owned());
    /// ```
    fn max_len(&self, max: usize) -> String;
}

impl TextProcessExt for str {
    fn remove_markdown(&self) -> String {
        self.chars()
            .filter_map(|c| match c {
                '*' => None,
                '_' => None,
                '\\' => None,
                '~' => None,
                '|' => None,
                '`' => None,
                other => Some(other),
            })
            .collect()
    }

    fn max_len(&self, max: usize) -> String {
        debug_assert!(max >= 3, "cannot truncate to less than 3 characters");

        if self.len() < max {
            return self.to_owned();
        }

        let (start, _) = self.split_at(max - 3);
        start.to_owned() + "..."
    }
}

impl TextProcessExt for String {
    fn remove_markdown(&self) -> String {
        self.as_str().remove_markdown()
    }

    fn max_len(&self, max: usize) -> String {
        self.as_str().max_len(max)
    }
}

#[cfg(test)]
mod tests {
    use super::TextProcessExt;

    #[test]
    fn text_remove_markdown() {
        let text = "*italics* _italics_ **bold** ~~strikethrough~~ `code` ||spoiler||".to_owned();
        let expected = "italics italics bold strikethrough code spoiler".to_owned();

        assert_eq!(text.remove_markdown(), expected);
    }

    #[test]
    fn test_truncate() {
        assert_eq!("hello world".to_owned().max_len(9), "hello ...".to_owned());
        assert_eq!(
            "hello world".to_owned().max_len(15),
            "hello world".to_owned()
        );
        assert_eq!("hello world".to_owned().max_len(3), "...".to_owned());
    }
}
