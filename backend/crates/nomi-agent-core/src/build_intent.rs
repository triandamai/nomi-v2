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

/// Asks to put a project live (English and Indonesian): what the Publish suggestion and button send,
/// and how people say it. Not questions about a publish ("why did the publish fail?").
const PUBLISH: &[&str] = &[
    "publish it", "publish this", "publish now", "publish the app", "publish my app", "publish again", "republish",
    "deploy it", "deploy this", "deploy now", "deploy the app", "deploy my app", "deploy again", "redeploy", "go live",
    "make it live", "update the live app", "terbitkan", "publikasikan", "rilis", "tolong publish", "tolong deploy",
];

/// Whether the message asks for the project to be published (or the live app updated): only Rena
/// and Koda can, so the router hands it to Rena whoever was talking.
pub fn wants_to_publish(text: &str) -> bool {
    let text = text.trim().trim_end_matches(['.', '!']).to_lowercase();
    matches!(text.as_str(), "publish" | "deploy") || PUBLISH.iter().any(|p| has_phrase(&text, p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_requests() {
        for text in ["Publish it", "publish", "Deploy!", "please go live", "Update the live app", "tolong publish ya", "terbitkan aplikasinya"] {
            assert!(wants_to_publish(text), "{text}");
        }
        for text in ["why did the publish fail?", "is it published?", "the website is down", "check the deploy logs"] {
            assert!(!wants_to_publish(text), "{text}");
        }
    }

    #[test]
    fn whole_words_only() {
        assert!(!has_phrase("happy", "app"));
        assert!(!has_phrase("websites are fun", "website"));
        assert!(has_phrase("a web app.", "web app"));
    }
}
