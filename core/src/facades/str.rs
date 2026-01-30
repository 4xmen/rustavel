use std::borrow::Cow;

/// A small utility struct providing string helpers similar to PHP's fnc.
#[derive(Debug)]
pub struct Str;

impl Str {
    /// Explode: split string by the given delimiter into a Vec<String>.
    /// Behaves like PHP explode — returns all parts (including empty ones).
    pub fn explode(s: &str, delimiter: &str) -> Vec<String> {
        if delimiter.is_empty() {
            // If delimiter is empty, split into individual characters (UTF-8 graphemes not handled here).
            // Use chars to preserve valid UTF-8 codepoints (may split multi-codepoint graphemes).
            return s.chars().map(|c| c.to_string()).collect();
        }
        // Use split which preserves empty fields for consecutive delimiters
        s.split(delimiter).map(|p| p.to_string()).collect()
    }

    /// Implode (join): join an iterator of &str or String with the given delimiter.
    pub fn implode<'a, I, S>(delimiter: &str, parts: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: Into<Cow<'a, str>>,
    {
        let mut iter = parts.into_iter();
        match iter.next() {
            None => String::new(),
            Some(first) => {
                let mut out = String::from(first.into());
                for p in iter {
                    out.push_str(delimiter);
                    out.push_str(&p.into());
                }
                out
            }
        }
    }

    /// Limit: limit a string by number of words, similar to PHP word-based limiting.
    ///
    /// - `count`: maximum number of words to allow.
    /// - `end` : string to append when trimmed (e.g., "..." )
    /// - Words are split on Unicode whitespace (using `split_whitespace`).
    /// - Preserves original spacing only insofar as words are joined with single spaces.
    pub fn limit_words(s: &str, count: usize, end: &str) -> String {
        if count == 0 {
            return end.to_string();
        }

        // Collect words using split_whitespace (Unicode-aware)
        let words: Vec<&str> = s.split_whitespace().collect();
        if words.len() <= count {
            // Return original string (preserve original spacing) if not exceeding.
            // To preserve original spacing exactly, return s; PHP's str_word_count isn't identical,
            // but user asked for "exactly like PHP" — this approximates by returning original when not trimmed.
            return s.to_string();
        }

        let mut out = String::with_capacity(s.len());
        for (i, w) in words.iter().take(count).enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push_str(w);
        }
        out.push_str(end);
        out
    }

    /// Trim characters from both ends. `chars` is a string where each char is treated
    /// as a trimming character (like PHP trim($str, $chars)).
    pub fn trim(s: &str, chars: &str) -> String {
        Str::ltrim(&Str::rtrim(s, chars), chars)
    }

    /// Trim characters from the start (left) only.
    pub fn ltrim(s: &str, chars: &str) -> String {
        if chars.is_empty() {
            return s.trim_start().to_string();
        }
        let set: Vec<char> = chars.chars().collect();
        let mut start = 0;
        for (i, ch) in s.char_indices() {
            if set.contains(&ch) {
                // continue trimming
                start = i + ch.len_utf8();
                continue;
            } else {
                // found first non-trim char; slice from here
                return s[i..].to_string();
            }
        }
        // all characters were trimmed
        String::new()
    }

    /// Trim characters from the end (right) only.
    pub fn rtrim(s: &str, chars: &str) -> String {
        if chars.is_empty() {
            return s.trim_end().to_string();
        }
        let set: Vec<char> = chars.chars().collect();
        // iterate from the end using char_indices: collect indices then pick last non-trim
        let mut last_non_trim_byte = None;
        for (i, ch) in s.char_indices() {
            if !set.contains(&ch) {
                last_non_trim_byte = Some(i + ch.len_utf8());
            }
        }
        match last_non_trim_byte {
            Some(pos) => s[..pos].to_string(),
            None => String::new(),
        }
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
