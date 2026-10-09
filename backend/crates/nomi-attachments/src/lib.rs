//! Files people attach in chat, before any model sees them: what a file really is (from its
//! bytes, not its name), how big it may be, the reference a message carries for it, and the text
//! that can be read out of it on the server (documents, spreadsheets, slides, PDFs) plus a small
//! preview of an image. Reading images, audio and video takes a model; that happens in the
//! attachment worker (nomi-server) and the turn (nomi-agent-core::attachments).

pub mod extract;
pub mod kind;
pub mod limits;
pub mod reference;

pub use extract::{extract, Extracted};
pub use kind::{sniff, Kind, Sniffed};
pub use limits::Limits;
pub use reference::{parse_references, write_reference, Reference};
