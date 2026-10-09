//! How much people may attach. Defaults: 10 MB a file, 50 MB a video, 10 files and 100 MB a
//! message. Each can be changed with an env var (in MB, or a count for files).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_file_bytes: u64,
    pub max_video_bytes: u64,
    pub max_message_bytes: u64,
    pub max_files: usize,
}

const MB: u64 = 1024 * 1024;

impl Default for Limits {
    fn default() -> Self {
        Limits { max_file_bytes: 10 * MB, max_video_bytes: 50 * MB, max_message_bytes: 100 * MB, max_files: 10 }
    }
}

impl Limits {
    /// The defaults, with `ATTACHMENT_MAX_FILE_MB`, `ATTACHMENT_MAX_VIDEO_MB`,
    /// `ATTACHMENT_MAX_MESSAGE_MB` and `ATTACHMENT_MAX_FILES` applied.
    pub fn from_env() -> Limits {
        let defaults = Limits::default();
        let mb = |name: &str, default: u64| {
            std::env::var(name).ok().and_then(|v| v.trim().parse::<u64>().ok()).filter(|v| *v > 0).map(|v| v * MB).unwrap_or(default)
        };
        Limits {
            max_file_bytes: mb("ATTACHMENT_MAX_FILE_MB", defaults.max_file_bytes),
            max_video_bytes: mb("ATTACHMENT_MAX_VIDEO_MB", defaults.max_video_bytes),
            max_message_bytes: mb("ATTACHMENT_MAX_MESSAGE_MB", defaults.max_message_bytes),
            max_files: std::env::var("ATTACHMENT_MAX_FILES")
                .ok()
                .and_then(|v| v.trim().parse::<usize>().ok())
                .filter(|v| *v > 0)
                .unwrap_or(defaults.max_files),
        }
    }

    /// The most any one upload may be: the video limit, or the file limit if that's bigger.
    pub fn max_upload_bytes(&self) -> u64 {
        self.max_file_bytes.max(self.max_video_bytes)
    }

    /// The most a file of `kind` may be.
    pub fn max_bytes_for(&self, kind: crate::Kind) -> u64 {
        if kind == crate::Kind::Video {
            self.max_video_bytes
        } else {
            self.max_file_bytes
        }
    }
}
