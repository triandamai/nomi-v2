//! How much people may attach. Defaults: 10 MB a file, 50 MB a video, 10 files and 100 MB a
//! message. Each can be changed in Admin → Configuration, or with an env var (in MB, or a count for
//! files).

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
        Limits::from_lookup(|field| {
            let name = match field {
                "max_file_mb" => "ATTACHMENT_MAX_FILE_MB",
                "max_video_mb" => "ATTACHMENT_MAX_VIDEO_MB",
                "max_message_mb" => "ATTACHMENT_MAX_MESSAGE_MB",
                _ => "ATTACHMENT_MAX_FILES",
            };
            std::env::var(name).ok().and_then(|v| v.trim().parse::<u64>().ok())
        })
    }

    /// The defaults, with what `get` has for `max_file_mb`, `max_video_mb`, `max_message_mb`
    /// (in MB) and `max_files` applied; zero and missing values keep the default.
    pub fn from_lookup(get: impl Fn(&str) -> Option<u64>) -> Limits {
        let defaults = Limits::default();
        let mb = |field: &str, default: u64| get(field).filter(|v| *v > 0).map(|v| v * MB).unwrap_or(default);
        Limits {
            max_file_bytes: mb("max_file_mb", defaults.max_file_bytes),
            max_video_bytes: mb("max_video_mb", defaults.max_video_bytes),
            max_message_bytes: mb("max_message_mb", defaults.max_message_bytes),
            max_files: get("max_files").filter(|v| *v > 0).map(|v| v as usize).unwrap_or(defaults.max_files),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lookup_overrides_the_defaults_and_zero_keeps_them() {
        let limits = Limits::from_lookup(|field| match field {
            "max_file_mb" => Some(25),
            "max_files" => Some(3),
            "max_video_mb" => Some(0),
            _ => None,
        });
        assert_eq!(limits.max_file_bytes, 25 * MB);
        assert_eq!(limits.max_files, 3);
        assert_eq!(limits.max_video_bytes, Limits::default().max_video_bytes);
        assert_eq!(limits.max_message_bytes, Limits::default().max_message_bytes);
    }
}
