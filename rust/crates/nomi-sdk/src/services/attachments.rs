//! Files people attached to their messages (`routes/attachments.rs`): an image's small preview,
//! and any file's content.

use super::Services;
use crate::error::NomiError;

const ATTACHMENTS: &str = "attachments";
const PREVIEW: &str = "preview";
const CONTENT: &str = "content";

impl Services {
    /// A small preview of an attached image, as the encoded image (JPEG, PNG or WebP).
    pub async fn attachment_preview(&self, id: &str) -> Result<Vec<u8>, NomiError> {
        self.get_bytes(&[ATTACHMENTS, id, PREVIEW]).await
    }

    /// An attached file's content as uploaded.
    pub async fn attachment_content(&self, id: &str) -> Result<Vec<u8>, NomiError> {
        self.get_bytes(&[ATTACHMENTS, id, CONTENT]).await
    }
}
