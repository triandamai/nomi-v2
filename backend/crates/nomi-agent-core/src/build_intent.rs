//! Whether a message asks for an app, website or the like to be built: the router sends those to
//! Rena in a project chat, and Rena only brings her app-building playbook to them.

/// Words that ask for something to be made (English and Indonesian).
const MAKE: &[&str] = &[
    "build", "create", "make", "develop", "code", "program", "design", "prototype", "start", "set up", "setup", "generate",
    "bikin", "bikinin", "buat", "buatkan", "buatin", "membuat", "membangun", "bangun", "kembangkan", "rancang",
];

/// Things that are built as a project.
const THINGS: &[&str] = &[
    "app", "apps", "application", "web app", "webapp", "website", "web site", "site", "landing page", "web page", "webpage",
    "homepage", "portfolio", "dashboard", "game", "chrome extension", "extension", "browser extension", "chatbot", "bot",
    "api", "backend", "frontend", "saas", "online store", "online shop", "e-commerce", "ecommerce", "blog", "calculator",
    "todo list", "to-do list", "tracker", "aplikasi", "situs", "halaman web", "laman", "toko online", "permainan",
];

/// Mentions that make it a Google Workspace request instead ("create a Google Doc").
const WORKSPACE: &[&str] = &["google doc", "google sheet", "google slide", "gmail", "google calendar", "google drive", "spreadsheet", "google form"];

/// Whether `phrase` appears in `text` as whole words.
fn has_phrase(text: &str, phrase: &str) -> bool {
    let mut from = 0;
    while let Some(at) = text[from..].find(phrase) {
        let start = from + at;
        let end = start + phrase.len();
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        if !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric) {
            return true;
        }
        from = end;
    }
    false
}

/// Whether the message asks for an app, website or the like to be built.
pub fn wants_to_build(text: &str) -> bool {
    let text = text.to_lowercase();
    if WORKSPACE.iter().any(|w| text.contains(w)) {
        return false;
    }
    MAKE.iter().any(|w| has_phrase(&text, w)) && THINGS.iter().any(|w| has_phrase(&text, w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_words_only() {
        assert!(!has_phrase("happy", "app"));
        assert!(!has_phrase("websites are fun", "website"));
        assert!(has_phrase("a web app.", "web app"));
    }
}
