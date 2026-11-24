//! The header row of the library mode (UI-SPEC E3, E4): the Library tab and the search box.

/// The longest search text, in characters (legacy parity).
pub const SEARCH_MAX_CHARS: usize = 20;

/// The search text as the library filters by it: trimmed, lowercased and cut to [`SEARCH_MAX_CHARS`] characters.
pub fn normalize_query(_text: &str) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::{SEARCH_MAX_CHARS, normalize_query};

    #[test]
    fn search_text_is_trimmed_and_lowercased() {
        assert_eq!(normalize_query("  Sea NIGHT "), "sea night");
        assert_eq!(normalize_query("   "), "");
        assert_eq!(normalize_query(""), "");
    }

    #[test]
    fn search_text_is_cut_to_twenty_characters_not_bytes() {
        assert_eq!(normalize_query(&"a".repeat(30)), "a".repeat(SEARCH_MAX_CHARS));
        // Each of these is two bytes: the cut counts characters.
        assert_eq!(normalize_query(&"é".repeat(25)), "é".repeat(SEARCH_MAX_CHARS));
        assert_eq!(normalize_query(&"é".repeat(25)).chars().count(), SEARCH_MAX_CHARS);
    }
}
