//! What the composer may offer before it uploads: the kinds, extensions and types Nomi reads.
//! The server still decides by the file's bytes (see `kind::sniff`), and refuses only programs;
//! this is the list the composer checks a name against so a wrong pick fails before any upload.

/// Kinds of file the crew reads (`Kind` names, plus "archive" for zips the Files agent opens).
pub const ACCEPTED_KINDS: &[&str] = &["pdf", "image", "spreadsheet", "document", "presentation", "text", "audio", "video", "archive"];

pub const ACCEPTED_EXTENSIONS: &[&str] = &[
    ".pdf", ".jpg", ".jpeg", ".png", ".gif", ".webp", ".heic", ".csv", ".tsv", ".xls", ".xlsx", ".ods", ".doc", ".docx", ".odt", ".rtf", ".ppt", ".pptx",
    ".odp", ".txt", ".md", ".json", ".xml", ".yaml", ".yml", ".html", ".mp3", ".m4a", ".wav", ".ogg", ".webm", ".mp4", ".mov", ".zip",
];

pub const ACCEPTED_MIMES: &[&str] = &[
    "application/pdf",
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/heic",
    "text/csv",
    "text/tab-separated-values",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/vnd.oasis.opendocument.spreadsheet",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.oasis.opendocument.text",
    "application/rtf",
    "application/vnd.ms-powerpoint",
    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    "application/vnd.oasis.opendocument.presentation",
    "text/plain",
    "text/markdown",
    "application/json",
    "application/xml",
    "application/yaml",
    "text/html",
    "audio/mpeg",
    "audio/mp4",
    "audio/wav",
    "audio/ogg",
    "audio/webm",
    "video/mp4",
    "video/quicktime",
    "video/webm",
    "application/zip",
];

/// The extension of `name` as `.ext` in lower case, or "" when it has none.
pub fn extension_of(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() => format!(".{}", ext.to_ascii_lowercase()),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_extension() {
        assert_eq!(extension_of("Setup.DMG"), ".dmg");
        assert_eq!(extension_of("a.tar.gz"), ".gz");
        assert_eq!(extension_of("README"), "");
        assert_eq!(extension_of(".env"), "");
    }

    #[test]
    fn extensions_are_dotted_and_lower_case() {
        assert!(ACCEPTED_EXTENSIONS.iter().all(|e| e.starts_with('.') && *e == e.to_ascii_lowercase()));
    }
}
