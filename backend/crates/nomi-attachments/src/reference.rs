//! How a message points at its files: one `<attachment id="…" name="…" kind="…" mime="…"
//! size="…"/>` tag per file after the typed text. The file itself stays in storage; the backend
//! writes the attributes from its own record, so they can be trusted once a message is saved.
//! (Messages from before uploads carried a file's text inline instead, as
//! `<attachment name="…" kind="…">…</attachment>`; those have no id and are left as they are.)

use uuid::Uuid;

use crate::Kind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub id: Uuid,
    pub name: String,
    pub kind: Kind,
    pub mime: String,
    pub size: u64,
}

/// A tag's attribute values never hold quotes, angle brackets or line breaks.
fn clean(value: &str) -> String {
    value.chars().map(|c| if matches!(c, '"' | '<' | '>' | '\n' | '\r') { '_' } else { c }).collect()
}

pub fn write_reference(r: &Reference) -> String {
    format!(
        "<attachment id=\"{}\" name=\"{}\" kind=\"{}\" mime=\"{}\" size=\"{}\"/>",
        r.id,
        clean(&r.name),
        r.kind.as_str(),
        clean(&r.mime),
        r.size
    )
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

/// Each reference tag in `text`, with its byte range, in order.
pub fn find_references(text: &str) -> Vec<(std::ops::Range<usize>, Reference)> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = text[from..].find("<attachment ") {
        let start = from + offset;
        let Some(close) = text[start..].find('>') else { break };
        let end = start + close + 1;
        let tag = &text[start..end];
        from = end;
        if !tag.ends_with("/>") {
            continue;
        }
        let parsed = (|| {
            Some(Reference {
                id: attribute(tag, "id")?.parse().ok()?,
                name: attribute(tag, "name").unwrap_or("file").to_string(),
                kind: attribute(tag, "kind").and_then(Kind::parse).unwrap_or(Kind::Other),
                mime: attribute(tag, "mime").unwrap_or("application/octet-stream").to_string(),
                size: attribute(tag, "size").and_then(|s| s.parse().ok()).unwrap_or(0),
            })
        })();
        if let Some(reference) = parsed {
            found.push((start..end, reference));
        }
    }
    found
}

pub fn parse_references(text: &str) -> Vec<Reference> {
    find_references(text).into_iter().map(|(_, r)| r).collect()
}

/// `text` with every reference tag replaced by what `replace` returns for it.
pub fn replace_references(text: &str, mut replace: impl FnMut(&Reference) -> String) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (range, reference) in find_references(text) {
        out.push_str(&text[last..range.start]);
        out.push_str(&replace(&reference));
        last = range.end;
    }
    out.push_str(&text[last..]);
    out
}

/// `text` without its reference tags: what the person typed.
pub fn typed_text(text: &str) -> String {
    replace_references(text, |_| String::new()).trim().to_string()
}

/// `text` with each reference tag as `[file name]`, for places that only need to know a file
/// was there (titles, summaries).
pub fn readable(text: &str) -> String {
    replace_references(text, |r| format!("[{}]", r.name)).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt() -> Reference {
        Reference { id: Uuid::nil(), name: "receipt \"May\".jpg".into(), kind: Kind::Image, mime: "image/jpeg".into(), size: 2048 }
    }

    #[test]
    fn round_trips_and_cleans_names() {
        let tag = write_reference(&receipt());
        assert_eq!(tag, "<attachment id=\"00000000-0000-0000-0000-000000000000\" name=\"receipt _May_.jpg\" kind=\"image\" mime=\"image/jpeg\" size=\"2048\"/>");
        let text = format!("log this\n\n{tag}");
        let parsed = parse_references(&text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "receipt _May_.jpg");
        assert_eq!(typed_text(&text), "log this");
    }

    #[test]
    fn leaves_old_inline_sections_alone() {
        let old = "<attachment name=\"a.csv\" kind=\"file\">\nx,y\n</attachment>";
        assert!(parse_references(old).is_empty());
        assert_eq!(replace_references(old, |_| "!".into()), old);
    }
}
