//! Version parsing and the OS-update policy.
//!
//! LibonOS separates two kinds of update:
//!
//! * **Minor / patch** (`1.0.0` → `1.0.1`, `1.1.0`): applied in place, from the
//!   terminal with `libon update` or from Settings.
//! * **Major** (`1.0.0` → `2.0.0`): a full release upgrade that rewrites the
//!   system image. It is only offered from Settings, never silently and never
//!   from the plain `libon update` path.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// A `major.minor.patch` version, optionally with a pre-release tag.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: None,
        }
    }

    /// Attach a pre-release tag, e.g. `beta1`.
    pub fn with_pre(mut self, pre: impl Into<String>) -> Self {
        self.pre = Some(pre.into());
        self
    }

    pub fn parse(s: &str) -> Result<Version, VersionError> {
        let s = s.trim();
        let s = s.strip_prefix('v').unwrap_or(s);
        let (core, pre) = match s.split_once('-') {
            Some((c, p)) => (c, Some(p.to_string())),
            None => (s, None),
        };
        let mut parts = core.split('.');
        let major = parts
            .next()
            .ok_or(VersionError::Malformed)?
            .parse()
            .map_err(|_| VersionError::Malformed)?;
        let minor = parts
            .next()
            .ok_or(VersionError::Malformed)?
            .parse()
            .map_err(|_| VersionError::Malformed)?;
        let patch = parts
            .next()
            .unwrap_or("0")
            .parse()
            .map_err(|_| VersionError::Malformed)?;
        if parts.next().is_some() {
            return Err(VersionError::Malformed);
        }
        Ok(Version {
            major,
            minor,
            patch,
            pre,
        })
    }

    /// The next patch release.
    pub fn next_patch(&self) -> Version {
        Version::new(self.major, self.minor, self.patch + 1)
    }

    /// The next minor release.
    pub fn next_minor(&self) -> Version {
        Version::new(self.major, self.minor + 1, 0)
    }

    /// The next major release.
    pub fn next_major(&self) -> Version {
        Version::new(self.major + 1, 0, 0)
    }

    pub fn is_major_bump_from(&self, older: &Version) -> bool {
        self.major != older.major
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.pre {
            Some(pre) => write!(f, "{}.{}.{}-{pre}", self.major, self.minor, self.patch),
            None => write!(f, "{}.{}.{}", self.major, self.minor, self.patch),
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
            // A release outranks its own pre-release.
            .then_with(|| match (&self.pre, &other.pre) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VersionError {
    Malformed,
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "versione non valida")
    }
}

/// The kind of update LibonOS is willing to apply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateKind {
    /// Nothing to do.
    UpToDate,
    /// Same major line: applied in place from terminal or Settings.
    InPlace,
    /// A new major line: full system upgrade, Settings only.
    FullSystemUpgrade,
    /// The running version is newer than the channel (downgrade territory).
    Downgrade,
}

impl UpdateKind {
    /// Whether `libon update` is allowed to apply this on the command line.
    pub fn allowed_from_terminal(self) -> bool {
        matches!(self, UpdateKind::InPlace)
    }

    /// Whether this update may be applied from Settings.
    pub fn allowed_from_settings(self) -> bool {
        matches!(self, UpdateKind::InPlace | UpdateKind::FullSystemUpgrade)
    }

    pub fn label(self) -> &'static str {
        match self {
            UpdateKind::UpToDate => "Aggiornato",
            UpdateKind::InPlace => "Aggiornamento",
            UpdateKind::FullSystemUpgrade => "Aggiornamento di sistema completo",
            UpdateKind::Downgrade => "Versione precedente",
        }
    }
}

/// Classify the update from `current` to `available`.
pub fn classify_update(current: &Version, available: &Version) -> UpdateKind {
    match available.cmp(current) {
        Ordering::Equal => UpdateKind::UpToDate,
        Ordering::Less => UpdateKind::Downgrade,
        Ordering::Greater => {
            if available.is_major_bump_from(current) {
                UpdateKind::FullSystemUpgrade
            } else {
                UpdateKind::InPlace
            }
        }
    }
}

/// A release published on the update channel.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseEntry {
    pub version: Version,
    pub channel: String,
    pub title: String,
    pub summary: String,
    pub published: String,
    /// Size of the payload in bytes, for the download estimate.
    pub size_bytes: u64,
    pub mandatory: bool,
}

/// The update feed the OS checks.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateChannel {
    pub name: String,
    pub releases: Vec<ReleaseEntry>,
}

impl UpdateChannel {
    /// The newest release strictly newer than `current`, if any.
    pub fn newest_available(&self, current: &Version) -> Option<&ReleaseEntry> {
        self.releases
            .iter()
            .filter(|r| &r.version > current)
            .max_by(|a, b| a.version.cmp(&b.version))
    }

    /// The newest release in the *same* major line as `current` — the update the
    /// terminal and the everyday Settings button apply in place.
    pub fn newest_in_place(&self, current: &Version) -> Option<&ReleaseEntry> {
        self.releases
            .iter()
            .filter(|r| &r.version > current && !r.version.is_major_bump_from(current))
            .max_by(|a, b| a.version.cmp(&b.version))
    }

    /// The release a full-system upgrade would land on: the newest release in a
    /// newer major line than `current`.
    pub fn newest_major_upgrade(&self, current: &Version) -> Option<&ReleaseEntry> {
        self.releases
            .iter()
            .filter(|r| r.version.is_major_bump_from(current) && &r.version > current)
            .max_by(|a, b| a.version.cmp(&b.version))
    }
}

/// The result the UI shows for a checked channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateStatus {
    pub kind: UpdateKind,
    pub current: Version,
    pub target: Version,
    pub title: String,
    pub summary: String,
    pub size_bytes: u64,
    pub mandatory: bool,
}

impl UpdateStatus {
    /// The status `libon update` and the everyday Settings button report: the
    /// best *in-place* update available, ignoring a pending major upgrade.
    pub fn check(current: &Version, channel: &UpdateChannel) -> UpdateStatus {
        match channel.newest_in_place(current) {
            Some(release) => UpdateStatus {
                kind: classify_update(current, &release.version),
                current: current.clone(),
                target: release.version.clone(),
                title: release.title.clone(),
                summary: release.summary.clone(),
                size_bytes: release.size_bytes,
                mandatory: release.mandatory,
            },
            None => UpdateStatus {
                kind: UpdateKind::UpToDate,
                current: current.clone(),
                target: current.clone(),
                title: "Nessun aggiornamento".into(),
                summary: "Stai usando la versione più recente di LibonOS.".into(),
                size_bytes: 0,
                mandatory: false,
            },
        }
    }

    /// The status of the full-system upgrade a newer major line offers, if any.
    /// This is the only path that may cross a major version, and Settings is the
    /// only place that offers it.
    pub fn check_major_upgrade(current: &Version, channel: &UpdateChannel) -> Option<UpdateStatus> {
        channel
            .newest_major_upgrade(current)
            .map(|release| UpdateStatus {
                kind: UpdateKind::FullSystemUpgrade,
                current: current.clone(),
                target: release.version.clone(),
                title: release.title.clone(),
                summary: release.summary.clone(),
                size_bytes: release.size_bytes,
                mandatory: release.mandatory,
            })
    }
}

/// A short, human size for download estimates.
pub fn human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.0} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn parses_and_orders() {
        assert_eq!(v("1.2.3"), Version::new(1, 2, 3));
        assert_eq!(v("v2.0.0"), Version::new(2, 0, 0));
        assert_eq!(v("1.2"), Version::new(1, 2, 0));
        assert!(v("1.0.1") > v("1.0.0"));
        assert!(v("1.10.0") > v("1.9.9"));
        assert!(v("2.0.0") > v("1.99.99"));
        assert!(v("1.0.0") > v("1.0.0-rc1"));
        assert!(Version::parse("1.x.0").is_err());
        assert!(Version::parse("1.0.0.0").is_err());
    }

    #[test]
    fn classifies_updates() {
        assert_eq!(
            classify_update(&v("1.0.0"), &v("1.0.0")),
            UpdateKind::UpToDate
        );
        assert_eq!(
            classify_update(&v("1.0.0"), &v("1.0.1")),
            UpdateKind::InPlace
        );
        assert_eq!(
            classify_update(&v("1.0.0"), &v("1.4.0")),
            UpdateKind::InPlace
        );
        assert_eq!(
            classify_update(&v("1.0.0"), &v("2.0.0")),
            UpdateKind::FullSystemUpgrade
        );
        assert_eq!(
            classify_update(&v("2.0.0"), &v("1.9.0")),
            UpdateKind::Downgrade
        );
    }

    #[test]
    fn terminal_may_not_apply_major_upgrade() {
        let major = classify_update(&v("1.0.0"), &v("2.0.0"));
        assert!(!major.allowed_from_terminal());
        assert!(major.allowed_from_settings());
        let minor = classify_update(&v("1.0.0"), &v("1.1.0"));
        assert!(minor.allowed_from_terminal());
        assert!(minor.allowed_from_settings());
    }

    #[test]
    fn channel_picks_newest_available() {
        let channel = UpdateChannel {
            name: "stable".into(),
            releases: vec![
                ReleaseEntry {
                    version: v("1.0.1"),
                    channel: "stable".into(),
                    title: "Correzioni".into(),
                    summary: String::new(),
                    published: String::new(),
                    size_bytes: 12,
                    mandatory: false,
                },
                ReleaseEntry {
                    version: v("1.2.0"),
                    channel: "stable".into(),
                    title: "Novità".into(),
                    summary: String::new(),
                    published: String::new(),
                    size_bytes: 34,
                    mandatory: false,
                },
                ReleaseEntry {
                    version: v("2.0.0"),
                    channel: "stable".into(),
                    title: "LibonOS 2".into(),
                    summary: String::new(),
                    published: String::new(),
                    size_bytes: 4_000_000_000,
                    mandatory: true,
                },
            ],
        };
        assert_eq!(
            channel.newest_available(&v("1.0.0")).unwrap().version,
            v("2.0.0")
        );
        assert_eq!(
            channel.newest_major_upgrade(&v("1.0.0")).unwrap().version,
            v("2.0.0")
        );
        // The everyday check stays inside the running major line.
        assert_eq!(
            channel.newest_in_place(&v("1.0.0")).unwrap().version,
            v("1.2.0")
        );
        let status = UpdateStatus::check(&v("1.0.0"), &channel);
        assert_eq!(status.kind, UpdateKind::InPlace);
        assert_eq!(status.target, v("1.2.0"));
        // The major upgrade is a separate, explicit offer.
        let major = UpdateStatus::check_major_upgrade(&v("1.0.0"), &channel).unwrap();
        assert_eq!(major.kind, UpdateKind::FullSystemUpgrade);
        assert!(major.mandatory);
        assert!(major.kind.allowed_from_settings());
        assert!(!major.kind.allowed_from_terminal());
    }

    #[test]
    fn human_size_reads_naturally() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(2048), "2 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5 MB");
        assert_eq!(human_size(4_000_000_000), "3.7 GB");
    }
}
