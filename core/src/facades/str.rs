use std::collections::HashSet;

/// String helper utilities inspired by Laravel's Str helpers.
///
/// Designed for internal DSL usage:
/// - UTF-8 safe
/// - Minimal allocations
/// - Idiomatic Rust
#[derive(Debug)]
pub struct Str;

impl Str {
    /// Split a string by the given delimiter.
    ///
    /// Behaves like PHP's `explode`:
    /// - Empty parts are preserved
    /// - If delimiter is empty, splits into Unicode scalar values
    ///
    /// # Example
    /// ```rust
    /// use rustavel_core::facades::str::Str;
    /// let v = Str::explode("a,,b", ",");
    /// assert_eq!(v, vec!["a", "", "b"]);
    /// ```
    pub fn explode(s: &str, delimiter: &str) -> Vec<String> {
        if delimiter.is_empty() {
            return s.chars().map(|c| c.to_string()).collect();
        }

        s.split(delimiter).map(str::to_owned).collect()
    }

    /// Join string parts using the given delimiter.
    ///
    /// Accepts any iterator of string-like values.
    ///
    /// # Example
    /// ```rust
    ///
    /// use rustavel_core::facades::str::Str;
    /// let s = Str::implode("-", ["a", "b", "c"]);
    /// assert_eq!(s, "a-b-c");
    /// ```
    pub fn implode<I, S>(delimiter: &str, parts: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = parts.into_iter();

        let mut out = match iter.next() {
            Some(first) => String::from(first.as_ref()),
            None => return String::new(),
        };

        for p in iter {
            out.push_str(delimiter);
            out.push_str(p.as_ref());
        }

        out
    }

    /// Limit a string by number of words.
    ///
    /// Words are detected using Unicode whitespace.
    /// If the string exceeds the limit, `end` is appended.
    ///
    /// # Example
    /// ```rust
    /// use rustavel_core::facades::str::Str;
    /// let s = Str::limit_words("hello world from rust", 2, "...");
    /// assert_eq!(s, "hello world...");
    /// ```
    pub fn limit_words(s: &str, count: usize, end: &str) -> String {
        if count == 0 {
            return end.to_owned();
        }

        let mut iter = s.split_whitespace();
        let mut out = String::new();

        for i in 0..count {
            match iter.next() {
                Some(word) => {
                    if i > 0 {
                        out.push(' ');
                    }
                    out.push_str(word);
                }
                None => return s.to_owned(),
            }
        }

        if iter.next().is_some() {
            out.push_str(end);
        }

        out
    }

    /// Trim characters from both ends of the string.
    ///
    /// Each character in `chars` is treated as an individual trim character.
    ///
    /// # Example
    /// ```rust
    /// use rustavel_core::facades::str::Str;
    /// let s = Str::trim("..hello..", ".");
    /// assert_eq!(s, "hello");
    /// ```
    pub fn trim(s: &str, chars: &str) -> String {
        if chars.is_empty() {
            return s.trim().to_owned();
        }

        let set: HashSet<char> = chars.chars().collect();

        let start = s
            .char_indices()
            .find(|(_, c)| !set.contains(c))
            .map(|(i, _)| i)
            .unwrap_or(s.len());

        let end = s
            .char_indices()
            .rev()
            .find(|(_, c)| !set.contains(c))
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(start);

        s[start..end].to_owned()
    }

    /// Trim characters from the start of the string.
    ///
    /// # Example
    /// ```rust
    /// use rustavel_core::facades::str::Str;
    /// let s = Str::ltrim("///path", "/");
    /// assert_eq!(s, "path");
    /// ```
    pub fn ltrim(s: &str, chars: &str) -> String {
        if chars.is_empty() {
            return s.trim_start().to_owned();
        }

        let set: HashSet<char> = chars.chars().collect();
        let start = s
            .char_indices()
            .find(|(_, c)| !set.contains(c))
            .map(|(i, _)| i)
            .unwrap_or(s.len());

        s[start..].to_owned()
    }

    /// Trim characters from the end of the string.
    ///
    /// # Example
    /// ```rust
    /// use rustavel_core::facades::str::Str;
    ///
    /// let s = Str::rtrim("file.txt\n\n", "\n");
    /// assert_eq!(s, "file.txt");
    /// ```
    pub fn rtrim(s: &str, chars: &str) -> String {
        if chars.is_empty() {
            return s.trim_end().to_owned();
        }

        let set: HashSet<char> = chars.chars().collect();
        let end = s
            .char_indices()
            .rev()
            .find(|(_, c)| !set.contains(c))
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);

        s[..end].to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::Str;

    #[test]
    fn test_explode_basic() {
        let parts = Str::explode("a,b,,c", ",");
        assert_eq!(parts, vec!["a", "b", "", "c"]);
    }

    #[test]
    fn test_explode_empty_delim() {
        let parts = Str::explode("abč", "");
        // splits by chars (UTF-8 codepoints), not grapheme clusters
        assert_eq!(parts, vec!["a", "b", "č"]);
    }

    #[test]
    fn test_implode_basic() {
        let vec = vec!["a", "b", "c"];
        let s = Str::implode("-", vec);
        assert_eq!(s, "a-b-c");
    }

    #[test]
    fn test_limit_words_trim() {
        let s = "This   is  a   test string";
        let out = Str::limit_words(s, 3, "...");
        assert_eq!(out, "This is a...");
    }

    #[test]
    fn test_limit_words_no_trim() {
        let s = "Short text";
        let out = Str::limit_words(s, 5, "...");
        assert_eq!(out, s);
    }

    #[test]
    fn test_limit_zero() {
        let s = "Anything";
        let out = Str::limit_words(s, 0, "...");
        assert_eq!(out, "...");
    }

    #[test]
    fn test_trim_default_spaces_when_chars_empty() {
        let s = "  Hello \n\t ";
        assert_eq!(Str::trim(s, ""), "Hello");
    }

    #[test]
    fn test_ltrim_custom_chars() {
        let s = "///,///abc/,,";
        assert_eq!(Str::ltrim(s, "/,"), "abc/,,");
    }

    #[test]
    fn test_rtrim_custom_chars() {
        let s = "///,///abc/,,";
        assert_eq!(Str::rtrim(s, "/,"), "///,///abc");
    }

    #[test]
    fn test_trim_custom_chars_both_sides() {
        let s = "/,,/hello/world,/,";
        assert_eq!(Str::trim(s, "/,"), "hello/world");
    }

    #[test]
    fn test_trim_all_trimmed_returns_empty() {
        let s = "////,,,";
        assert_eq!(Str::trim(s, "/,"), "");
    }

    #[test]
    fn test_ltrim_all_trimmed_returns_empty() {
        let s = "////";
        assert_eq!(Str::ltrim(s, "/"), "");
    }

    #[test]
    fn test_rtrim_all_trimmed_returns_empty() {
        let s = "aaaa";
        assert_eq!(Str::rtrim(s, "a"), "");
    }

    #[test]
    fn test_trim_unicode_chars() {
        let s = "¡¡hola!!";
        assert_eq!(Str::trim(s, "¡!"), "hola");
    }
}
