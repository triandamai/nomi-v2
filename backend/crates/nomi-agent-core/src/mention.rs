//! "@Dana" in a message: the person addressing a crew member by name. The turn goes to that
//! member instead of whoever the classifier would pick (see nomi_turn::routing::mentioned_agent).
//! Works the same from the web composer's mention picker and from a typed "@Dana" in Telegram.

/// The names written after "@" in `text`, in order. An "@" only counts at the start of the text
/// or after whitespace or an opening bracket, so an email address ("dana@example.com") isn't one.
pub fn mentioned_names(text: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut previous: Option<char> = None;
    for (index, c) in text.char_indices() {
        let starts = c == '@' && previous.is_none_or(|p| p.is_whitespace() || matches!(p, '(' | '[' | '"' | '\''));
        previous = Some(c);
        if !starts {
            continue;
        }
        let rest = &text[index + 1..];
        let end = rest.find(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch == '-')).unwrap_or(rest.len());
        if end > 0 {
            names.push(&rest[..end]);
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_names_in_order() {
        assert_eq!(mentioned_names("@Dana how much did I spend? cc @Kala"), vec!["Dana", "Kala"]);
        assert_eq!(mentioned_names("ask (@Koda) about it"), vec!["Koda"]);
        assert_eq!(mentioned_names("@Dana, log this"), vec!["Dana"]);
    }

    #[test]
    fn email_addresses_and_lone_signs_are_not_mentions() {
        assert!(mentioned_names("mail dana@example.com").is_empty());
        assert!(mentioned_names("meet @ 5pm").is_empty());
        assert!(mentioned_names("").is_empty());
    }
}
