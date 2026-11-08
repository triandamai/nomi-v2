//! Pure planning helpers: no I/O, no clocks of their own. The caller passes in the time and the
//! manifest and gets back a decision.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Component, Path};
use std::time::{Duration, Instant};

use flate2::write::GzEncoder;
use flate2::Compression;

/// Length of the short start-rate window, in seconds.
const MINUTE_WINDOW_SECS: u64 = 60;
/// Length of the long start-rate window, in seconds.
const HOUR_WINDOW_SECS: u64 = 3600;
/// Unix permission bits given to every file in a built archive.
const ARCHIVE_FILE_MODE: u32 = 0o644;

/// One file in a project manifest: its path and when it last changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    pub path: String,
    pub updated_at: String,
}

/// Caps how often sandboxes may be started, per minute and per hour.
#[derive(Debug)]
pub struct StartLimiter {
    per_minute: u32,
    per_hour: u32,
    starts: VecDeque<Instant>,
}

impl StartLimiter {
    pub fn new(per_minute: u32, per_hour: u32) -> Self {
        Self {
            per_minute,
            per_hour,
            starts: VecDeque::new(),
        }
    }

    /// Records a start at `now`, or says how long to wait until one is allowed.
    pub fn try_acquire(&mut self, now: Instant) -> Result<(), Duration> {
        let hour = Duration::from_secs(HOUR_WINDOW_SECS);
        while let Some(&oldest) = self.starts.front() {
            if now.saturating_duration_since(oldest) >= hour {
                self.starts.pop_front();
            } else {
                break;
            }
        }
        let minute = Duration::from_secs(MINUTE_WINDOW_SECS);
        let mut wait = Duration::ZERO;
        for (limit, window) in [(self.per_minute, minute), (self.per_hour, hour)] {
            let inside: Vec<Instant> = self
                .starts
                .iter()
                .copied()
                .filter(|s| now.saturating_duration_since(*s) < window)
                .collect();
            if limit == 0 {
                // Nothing is ever allowed; report the whole window as the wait.
                wait = wait.max(window);
                continue;
            }
            if inside.len() >= limit as usize {
                // Allowed again once enough of the window's oldest starts have left it.
                let blocking = inside[inside.len() - limit as usize];
                wait = wait.max(window.saturating_sub(now.saturating_duration_since(blocking)));
            }
        }
        if wait > Duration::ZERO {
            return Err(wait);
        }
        self.starts.push_back(now);
        Ok(())
    }
}

/// One step of bringing a sandbox's files up to date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncAction {
    Write(String),
    Delete(String),
    Archive,
}

/// Decides how to sync: a full archive when nothing is known yet or more than
/// `archive_threshold` files changed, otherwise per-file writes and deletes.
pub fn plan_sync(
    known: &BTreeMap<String, String>,
    manifest: &[ManifestEntry],
    archive_threshold: usize,
) -> Vec<SyncAction> {
    if known.is_empty() {
        return vec![SyncAction::Archive];
    }
    let changed: Vec<&ManifestEntry> = manifest
        .iter()
        .filter(|e| known.get(&e.path) != Some(&e.updated_at))
        .collect();
    if changed.len() > archive_threshold {
        return vec![SyncAction::Archive];
    }
    let current: BTreeSet<&str> = manifest.iter().map(|e| e.path.as_str()).collect();
    let mut actions: Vec<SyncAction> = changed
        .into_iter()
        .map(|e| SyncAction::Write(e.path.clone()))
        .collect();
    actions.extend(
        known
            .keys()
            .filter(|p| !current.contains(p.as_str()))
            .map(|p| SyncAction::Delete(p.clone())),
    );
    actions
}

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("unsafe archive path: {0:?}")]
    UnsafePath(String),
    #[error("archive build failed: {0}")]
    Io(#[from] std::io::Error),
}

/// Packs files into a gzipped tar, refusing any path that is empty, absolute or climbs with `..`.
pub fn build_archive(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, ArchiveError> {
    for (path, _) in files {
        let p = Path::new(path);
        let safe = !path.is_empty()
            && p.components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
            && !p.has_root();
        if !safe {
            return Err(ArchiveError::UnsafePath(path.clone()));
        }
    }
    let mut builder = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (path, bytes) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(ARCHIVE_FILE_MODE);
        header.set_cksum();
        builder.append_data(&mut header, path, bytes.as_slice())?;
    }
    Ok(builder.into_inner()?.finish()?)
}

/// Exponential delay `base * 2^attempts`, never above `cap`.
pub fn backoff(attempts: u32, base: Duration, cap: Duration) -> Duration {
    let factor = 1u32.checked_shl(attempts).unwrap_or(u32::MAX);
    base.saturating_mul(factor).min(cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, at: &str) -> ManifestEntry {
        ManifestEntry {
            path: path.into(),
            updated_at: at.into(),
        }
    }

    fn known(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(p, a)| (p.to_string(), a.to_string()))
            .collect()
    }

    #[test]
    fn limiter_denies_the_eleventh_start_in_a_minute_with_wait() {
        let mut l = StartLimiter::new(10, 100);
        let t0 = Instant::now();
        for i in 0..10 {
            assert!(l.try_acquire(t0 + Duration::from_secs(i)).is_ok());
        }
        let wait = l.try_acquire(t0 + Duration::from_secs(10)).unwrap_err();
        assert_eq!(wait, Duration::from_secs(50));
    }

    #[test]
    fn limiter_recovers_after_the_window() {
        let mut l = StartLimiter::new(2, 100);
        let t0 = Instant::now();
        l.try_acquire(t0).unwrap();
        l.try_acquire(t0).unwrap();
        assert!(l.try_acquire(t0 + Duration::from_secs(30)).is_err());
        assert!(l.try_acquire(t0 + Duration::from_secs(61)).is_ok());
    }

    #[test]
    fn limiter_enforces_the_hourly_window() {
        let mut l = StartLimiter::new(100, 2);
        let t0 = Instant::now();
        l.try_acquire(t0).unwrap();
        l.try_acquire(t0 + Duration::from_secs(120)).unwrap();
        let wait = l.try_acquire(t0 + Duration::from_secs(240)).unwrap_err();
        assert_eq!(wait, Duration::from_secs(3600 - 240));
    }

    #[test]
    fn limiter_with_zero_limit_denies_without_panicking() {
        let t0 = Instant::now();
        assert!(StartLimiter::new(0, 5).try_acquire(t0).is_err());
        assert!(StartLimiter::new(5, 0).try_acquire(t0).is_err());
    }

    #[test]
    fn plan_sync_full_archive_when_nothing_known() {
        let plan = plan_sync(&BTreeMap::new(), &[entry("a", "1")], 10);
        assert_eq!(plan, vec![SyncAction::Archive]);
    }

    #[test]
    fn plan_sync_writes_only_changed_and_deletes_removed() {
        let k = known(&[("a", "1"), ("b", "1"), ("gone", "1")]);
        let m = [entry("a", "1"), entry("b", "2"), entry("c", "1")];
        let plan = plan_sync(&k, &m, 10);
        assert_eq!(
            plan,
            vec![
                SyncAction::Write("b".into()),
                SyncAction::Write("c".into()),
                SyncAction::Delete("gone".into()),
            ]
        );
    }

    #[test]
    fn plan_sync_archives_above_threshold() {
        let k = known(&[("a", "1")]);
        let m = [entry("a", "2"), entry("b", "1"), entry("c", "1")];
        assert_eq!(plan_sync(&k, &m, 2), vec![SyncAction::Archive]);
        assert_ne!(plan_sync(&k, &m, 3), vec![SyncAction::Archive]);
    }

    #[test]
    fn archive_rejects_parent_traversal_and_absolute_paths() {
        for bad in ["../x", "a/../../x", "/etc/passwd", ""] {
            let r = build_archive(&[(bad.to_string(), vec![1])]);
            assert!(matches!(r, Err(ArchiveError::UnsafePath(_))), "{bad}");
        }
    }

    #[test]
    fn archive_round_trips_files() {
        use std::io::Read;
        let bytes = build_archive(&[("src/a.txt".into(), b"hi".to_vec())]).unwrap();
        let mut ar = tar::Archive::new(flate2::read::GzDecoder::new(bytes.as_slice()));
        let mut entries = ar.entries().unwrap();
        let mut e = entries.next().unwrap().unwrap();
        assert_eq!(e.path().unwrap().to_str().unwrap(), "src/a.txt");
        let mut s = String::new();
        e.read_to_string(&mut s).unwrap();
        assert_eq!(s, "hi");
    }

    #[test]
    fn backoff_is_monotonic_and_capped() {
        let base = Duration::from_secs(1);
        let cap = Duration::from_secs(30);
        let mut prev = Duration::ZERO;
        for n in 0..70 {
            let d = backoff(n, base, cap);
            assert!(d >= prev && d <= cap);
            prev = d;
        }
        assert_eq!(backoff(0, base, cap), base);
        assert_eq!(backoff(3, base, cap), Duration::from_secs(8));
        assert_eq!(backoff(69, base, cap), cap);
    }
}
