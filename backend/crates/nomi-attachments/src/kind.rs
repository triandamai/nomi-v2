//! What a file is, from its first bytes. The name and the type the browser claims only break
//! ties (a CSV and a log file look alike) or pick between containers that hold either (a WebM
//! voice note is audio, a WebM recording of the screen is video).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Text,
    Pdf,
    Document,
    Spreadsheet,
    Presentation,
    Image,
    Audio,
    /// A voice note recorded in the composer.
    Voice,
    Video,
    Other,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Text => "text",
            Kind::Pdf => "pdf",
            Kind::Document => "document",
            Kind::Spreadsheet => "spreadsheet",
            Kind::Presentation => "presentation",
            Kind::Image => "image",
            Kind::Audio => "audio",
            Kind::Voice => "voice",
            Kind::Video => "video",
            Kind::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Kind> {
        Some(match s {
            "text" | "file" => Kind::Text,
            "pdf" => Kind::Pdf,
            "document" => Kind::Document,
            "spreadsheet" => Kind::Spreadsheet,
            "presentation" => Kind::Presentation,
            "image" => Kind::Image,
            "audio" => Kind::Audio,
            "voice" => Kind::Voice,
            "video" => Kind::Video,
            "other" => Kind::Other,
            _ => return None,
        })
    }

    /// Kinds only a model can read (in the background, or by looking at the file in a turn).
    pub fn needs_model(self) -> bool {
        matches!(self, Kind::Image | Kind::Audio | Kind::Voice | Kind::Video)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sniffed {
    pub kind: Kind,
    pub mime: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// Programs and libraries are never accepted.
    Executable,
    Empty,
}

const EXECUTABLE_MIMES: [&str; 6] = [
    "application/vnd.microsoft.portable-executable",
    "application/x-executable",
    "application/x-mach-binary",
    "application/x-sharedlib",
    "application/x-msdownload",
    "application/vnd.android.dex",
];

/// What `head` (the first few KB of the file) is. `name` and `claimed` (the browser's
/// Content-Type) only settle what the bytes leave open; `voice` marks a composer voice note.
pub fn sniff(head: &[u8], name: &str, claimed: &str, voice: bool) -> Result<Sniffed, Refused> {
    if head.is_empty() {
        return Err(Refused::Empty);
    }
    let extension = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
    let claimed = claimed.split(';').next().unwrap_or("").trim().to_ascii_lowercase();

    if is_program(head) {
        return Err(Refused::Executable);
    }
    if let Some(found) = infer::get(head) {
        let mime = found.mime_type();
        if EXECUTABLE_MIMES.contains(&mime) || found.matcher_type() == infer::MatcherType::App {
            return Err(Refused::Executable);
        }
        let sniffed = |kind: Kind, mime: &str| Ok(Sniffed { kind, mime: mime.to_string() });
        // WebM, Ogg, MP4 and friends hold sound alone or sound and pictures.
        let audio_container = claimed.starts_with("audio/") || voice || matches!(extension.as_str(), "m4a" | "ogg" | "oga" | "opus" | "weba");
        return match mime {
            m if m.starts_with("image/") => sniffed(Kind::Image, m),
            "application/pdf" => sniffed(Kind::Pdf, mime),
            m if m.starts_with("audio/") => sniffed(if voice { Kind::Voice } else { Kind::Audio }, m),
            m if m.starts_with("video/") && audio_container => {
                let audio = match m {
                    "video/webm" => "audio/webm",
                    "video/mp4" => "audio/mp4",
                    "video/ogg" => "audio/ogg",
                    _ => m,
                };
                sniffed(if voice { Kind::Voice } else { Kind::Audio }, audio)
            }
            m if m.starts_with("video/") => sniffed(Kind::Video, m),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.oasis.opendocument.text"
            | "application/rtf" => sniffed(Kind::Document, mime),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" | "application/vnd.oasis.opendocument.spreadsheet" => {
                sniffed(Kind::Spreadsheet, mime)
            }
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            | "application/vnd.oasis.opendocument.presentation" => sniffed(Kind::Presentation, mime),
            // Old binary Office files share one container; the name says which.
            "application/msword" | "application/vnd.ms-excel" | "application/vnd.ms-powerpoint" | "application/x-ole-storage" => {
                match extension.as_str() {
                    "xls" => sniffed(Kind::Spreadsheet, "application/vnd.ms-excel"),
                    "ppt" => sniffed(Kind::Presentation, "application/vnd.ms-powerpoint"),
                    _ => sniffed(Kind::Document, "application/msword"),
                }
            }
            // A plain zip that's really an Office file infer didn't place.
            "application/zip" => match extension.as_str() {
                "docx" => sniffed(Kind::Document, "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
                "xlsx" => sniffed(Kind::Spreadsheet, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                "pptx" => sniffed(Kind::Presentation, "application/vnd.openxmlformats-officedocument.presentationml.presentation"),
                _ => sniffed(Kind::Other, mime),
            },
            m if m.starts_with("text/") || m == "application/xml" || m == "application/json" => sniffed(Kind::Text, &text_mime(&extension, m)),
            m => sniffed(Kind::Other, m),
        };
    }

    if looks_like_text(head) {
        let mime = text_mime(&extension, "text/plain");
        return Ok(Sniffed { kind: Kind::Text, mime });
    }
    Ok(Sniffed { kind: Kind::Other, mime: "application/octet-stream".to_string() })
}

/// Windows, Linux and Mac programs, whatever the rest of the header says.
fn is_program(head: &[u8]) -> bool {
    const MAGICS: [&[u8]; 6] = [b"MZ", b"\x7fELF", b"\xfe\xed\xfa\xce", b"\xfe\xed\xfa\xcf", b"\xce\xfa\xed\xfe", b"\xcf\xfa\xed\xfe"];
    MAGICS.iter().any(|m| head.starts_with(m))
}

/// UTF-8 (a cut-off last character allowed) with almost no control characters.
fn looks_like_text(head: &[u8]) -> bool {
    let valid = match std::str::from_utf8(head) {
        Ok(s) => s,
        Err(e) if e.error_len().is_none() => match std::str::from_utf8(&head[..e.valid_up_to()]) {
            Ok(s) => s,
            Err(_) => return false,
        },
        Err(_) => return false,
    };
    let controls = valid.chars().filter(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t' | '\u{c}')).count();
    controls * 100 <= valid.chars().count().max(1)
}

fn text_mime(extension: &str, fallback: &str) -> String {
    match extension {
        "csv" => "text/csv",
        "tsv" => "text/tab-separated-values",
        "md" | "markdown" => "text/markdown",
        "json" => "application/json",
        "html" | "htm" => "text/html",
        "xml" => "application/xml",
        "yaml" | "yml" => "application/yaml",
        _ => fallback,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_bytes_not_the_name() {
        let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, b'I', b'H', b'D', b'R'];
        assert_eq!(sniff(&png, "notes.txt", "text/plain", false).unwrap(), Sniffed { kind: Kind::Image, mime: "image/png".into() });
        assert_eq!(sniff(b"%PDF-1.7\n", "x", "", false).unwrap().kind, Kind::Pdf);
        assert_eq!(sniff(b"date,amount\n2026-01-01,5\n", "spend.csv", "", false).unwrap(), Sniffed { kind: Kind::Text, mime: "text/csv".into() });
    }

    #[test]
    fn refuses_programs() {
        assert_eq!(sniff(b"MZ\x90\x00\x03\x00\x00\x00\x04\x00\x00\x00\xff\xff\x00\x00PE\0\0", "setup.pdf", "application/pdf", false), Err(Refused::Executable));
        assert_eq!(sniff(b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00", "a", "", false), Err(Refused::Executable));
        assert_eq!(sniff(b"", "a", "", false), Err(Refused::Empty));
    }

    #[test]
    fn a_webm_voice_note_is_audio() {
        let webm = [0x1A, 0x45, 0xDF, 0xA3, 0x9F, 0x42, 0x86, 0x81, 0x01, 0x42, 0xF7, 0x81, 0x01, 0x42, 0xF2, 0x81, 0x04, 0x42, 0xF3, 0x81, 0x08, 0x42, 0x82, 0x84, b'w', b'e', b'b', b'm'];
        assert_eq!(sniff(&webm, "Voice note.webm", "audio/webm;codecs=opus", true).unwrap(), Sniffed { kind: Kind::Voice, mime: "audio/webm".into() });
        assert_eq!(sniff(&webm, "screen.webm", "video/webm", false).unwrap().kind, Kind::Video);
    }

    #[test]
    fn unknown_binary_is_other() {
        assert_eq!(sniff(&[0u8, 1, 2, 3, 0, 0, 0, 9, 200, 201], "blob", "", false).unwrap().kind, Kind::Other);
    }
}
