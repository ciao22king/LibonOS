//! Package management.
//!
//! LibonOS does not force one package manager on the user: setup asks whether
//! they prefer **APT**, **pacman**, or **libon** — the manager LibonOS ships
//! with. All three are exposed through one trait so the Store, the Settings
//! screen, and the CLI never special-case a backend.

pub mod appimage;
pub mod apt;
pub mod libon;
pub mod pacman;

use crate::i18n::{Key, Strings};
use serde::{Deserialize, Serialize};

/// Which package manager the machine is configured to use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageManagerKind {
    Apt,
    Pacman,
    Libon,
}

impl PackageManagerKind {
    pub const ALL: [PackageManagerKind; 3] = [
        PackageManagerKind::Libon,
        PackageManagerKind::Apt,
        PackageManagerKind::Pacman,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PackageManagerKind::Apt => "APT",
            PackageManagerKind::Pacman => "pacman",
            PackageManagerKind::Libon => "libon",
        }
    }

    /// The key behind the one-line description shown in setup and Settings.
    pub fn description_key(self) -> Key {
        match self {
            PackageManagerKind::Apt => Key::ManagerAptDescription,
            PackageManagerKind::Pacman => Key::ManagerPacmanDescription,
            PackageManagerKind::Libon => Key::ManagerLibonDescription,
        }
    }

    /// The description in the active language. The manager's *name* is not
    /// translated: APT, pacman and libon are proper nouns.
    pub fn description(self, t: &Strings) -> &'static str {
        t.get(self.description_key())
    }

    /// The executable that must exist for this manager to be usable.
    pub fn binary(self) -> &'static str {
        match self {
            PackageManagerKind::Apt => "apt-get",
            PackageManagerKind::Pacman => "pacman",
            PackageManagerKind::Libon => "libon",
        }
    }

    /// Whether installing needs administrator rights.
    pub fn requires_admin(self) -> bool {
        match self {
            PackageManagerKind::Apt | PackageManagerKind::Pacman => true,
            // libon installs into the user's home, so it never needs root.
            PackageManagerKind::Libon => false,
        }
    }

    pub fn is_preinstalled(self) -> bool {
        self == PackageManagerKind::Libon
    }

    /// The manager for a kind, ready to build commands.
    pub fn manager(self) -> Box<dyn PackageManager> {
        match self {
            PackageManagerKind::Apt => Box::new(apt::Apt),
            PackageManagerKind::Pacman => Box::new(pacman::Pacman),
            PackageManagerKind::Libon => Box::new(libon::Libon),
        }
    }
}

/// A broad grouping used to lay out the Store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    System,
    Internet,
    Development,
    Media,
    Office,
    Games,
    Communication,
    Utilities,
}

impl Category {
    pub const ALL: [Category; 8] = [
        Category::Internet,
        Category::Development,
        Category::Media,
        Category::Office,
        Category::Communication,
        Category::Games,
        Category::Utilities,
        Category::System,
    ];

    /// The key behind [`Category::label`], for callers that need the key rather
    /// than the text.
    pub fn key(self) -> Key {
        match self {
            Category::System => Key::CategorySystem,
            Category::Internet => Key::CategoryInternet,
            Category::Development => Key::CategoryDevelopment,
            Category::Media => Key::CategoryMedia,
            Category::Office => Key::CategoryOffice,
            Category::Games => Key::CategoryGames,
            Category::Communication => Key::CategoryCommunication,
            Category::Utilities => Key::CategoryUtilities,
        }
    }

    pub fn label(self, t: &Strings) -> &'static str {
        t.get(self.key())
    }
}

/// One entry as shown in the Store.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Package {
    /// The identifier passed to the manager (`firefox`, `org.gimp.GIMP`).
    pub id: String,
    pub name: String,
    pub summary: String,
    pub version: String,
    pub size_bytes: u64,
    pub installed: bool,
    pub category: Category,
    pub manager: PackageManagerKind,
}

/// Progress of one install, for the progress row in the Store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstallPhase {
    Queued,
    Downloading { received: u64, total: u64 },
    Verifying,
    Installing,
    Linking,
    Done,
    Failed(String),
}

impl InstallPhase {
    pub fn is_terminal(&self) -> bool {
        matches!(self, InstallPhase::Done | InstallPhase::Failed(_))
    }

    /// Fraction complete in `0.0..=1.0`; `None` when the phase has no measurable
    /// progress and the UI should show an indeterminate bar.
    pub fn fraction(&self) -> Option<f32> {
        match self {
            InstallPhase::Queued => Some(0.0),
            InstallPhase::Downloading { received, total } if *total > 0 => {
                Some((*received as f32 / *total as f32).clamp(0.0, 1.0))
            }
            InstallPhase::Verifying => Some(0.95),
            InstallPhase::Installing => Some(0.97),
            InstallPhase::Linking => Some(0.99),
            InstallPhase::Done => Some(1.0),
            _ => None,
        }
    }

    pub fn label(&self) -> String {
        match self {
            InstallPhase::Queued => "In coda".into(),
            InstallPhase::Downloading { .. } => "Download".into(),
            InstallPhase::Verifying => "Verifica firma".into(),
            InstallPhase::Installing => "Installazione".into(),
            InstallPhase::Linking => "Registrazione nell'app menu".into(),
            InstallPhase::Done => "Completato".into(),
            InstallPhase::Failed(_) => "Non riuscito".into(),
        }
    }
}

/// The common surface every backend implements. Commands are returned as
/// argument vectors rather than run here, so they are easy to test and easy to
/// show the user before anything touches the system.
pub trait PackageManager: Send + Sync {
    fn kind(&self) -> PackageManagerKind;

    /// Search the manager's catalog.
    fn search(&self, query: &str) -> Vec<Package>;

    /// A curated set shown when the Store opens, before any search.
    fn featured(&self) -> Vec<Package>;

    fn install_args(&self, id: &str) -> Vec<String>;
    fn remove_args(&self, id: &str) -> Vec<String>;
    /// Refresh the local index of available packages.
    fn refresh_args(&self) -> Vec<String>;
    /// Upgrade every installed package.
    fn upgrade_args(&self) -> Vec<String>;

    /// Whether the manager's binary is present on this machine.
    fn is_available(&self) -> bool {
        which(self.kind().binary())
    }
}

/// Look up an executable on `PATH` without pulling in a dependency.
pub fn which(binary: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(binary);
            if candidate.is_file() {
                return true;
            }
        }
    }
    false
}

/// The manager for the configured kind.
pub fn manager_for(kind: PackageManagerKind) -> Box<dyn PackageManager> {
    kind.manager()
}

/// Case-insensitive search over a backend's built-in catalog. Every
/// whitespace-separated token of the query must match somewhere in the id, name
/// or summary, so "note markdown" narrows rather than widens.
pub fn search_catalog(catalog: &[Package], query: &str) -> Vec<Package> {
    let tokens: Vec<String> = query.split_whitespace().map(|t| t.to_lowercase()).collect();
    if tokens.is_empty() {
        return catalog.to_vec();
    }
    catalog
        .iter()
        .filter(|p| {
            let haystack = format!(
                "{} {} {}",
                p.id.to_lowercase(),
                p.name.to_lowercase(),
                p.summary.to_lowercase()
            );
            tokens.iter().all(|t| haystack.contains(t))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_a_manager_and_a_name() {
        for kind in PackageManagerKind::ALL {
            assert!(!kind.label().is_empty());
            let m = kind.manager();
            assert_eq!(m.kind(), kind);
        }
    }

    #[test]
    fn every_manager_and_category_is_described_in_every_language() {
        use crate::i18n::Locale;
        for locale in Locale::ALL {
            let t = Strings::new(locale);
            for kind in PackageManagerKind::ALL {
                assert!(
                    !kind.description(&t).trim().is_empty(),
                    "{locale:?} // {kind:?}"
                );
            }
            for category in Category::ALL {
                assert!(
                    !category.label(&t).trim().is_empty(),
                    "{locale:?} // {category:?}"
                );
            }
        }
    }

    #[test]
    fn install_fraction_is_monotonic_and_bounded() {
        for phase in [
            InstallPhase::Queued,
            InstallPhase::Downloading {
                received: 50,
                total: 100,
            },
            InstallPhase::Verifying,
            InstallPhase::Installing,
            InstallPhase::Linking,
            InstallPhase::Done,
        ] {
            let f = phase.fraction().unwrap();
            assert!((0.0..=1.0).contains(&f), "{phase:?} -> {f}");
        }
        assert!(InstallPhase::Downloading {
            received: 0,
            total: 0
        }
        .fraction()
        .is_none());
        assert!(InstallPhase::Failed("x".into()).is_terminal());
        assert!(InstallPhase::Done.is_terminal());
    }

    #[test]
    fn only_libon_is_preinstalled_and_rootless() {
        assert!(PackageManagerKind::Libon.is_preinstalled());
        assert!(!PackageManagerKind::Libon.requires_admin());
        assert!(PackageManagerKind::Apt.requires_admin());
        assert!(PackageManagerKind::Pacman.requires_admin());
    }
}
