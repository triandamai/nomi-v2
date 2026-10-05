//! Files the user attaches travel inside the message text, one
//! `<attachment name="…" kind="…">…</attachment>` section per file (written by the web composer,
//! frontend/src/lib/attachments.ts). `kind` is "file" or "voice" (a voice note's transcript).

/// The agent every message with attachments is routed to (nomi-agent-files).
pub const FILES_AGENT_TYPE: &str = "files";

pub fn has_attachments(text: &str) -> bool {
    text.contains("<attachment name=\"") && text.contains("</attachment>")
}

#[cfg(test)]
mod tests {
    use super::has_attachments;

    #[test]
    fn spots_attachment_sections() {
        assert!(has_attachments("check this\n\n<attachment name=\"a.csv\">\nx\n</attachment>"));
        assert!(has_attachments("<attachment name=\"Voice note (0:12)\" kind=\"voice\">\nhi\n</attachment>"));
        assert!(!has_attachments("I mentioned <attachment name= in passing"));
    }
}
