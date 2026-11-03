//! What LibonOS ships with.
//!
//! A good operating system is not only a kernel and a window manager: it is a
//! set of working defaults — a browser, a file manager, a terminal, a text
//! editor, a store, a settings app, and the plumbing (sound, printing,
//! networking, updates, accessibility) that makes them usable. This module is
//! the single source of truth for that list, so the dock, the Store and the
//! Settings screen all agree.

use crate::packages::{Category, PackageManagerKind};
use serde::{Deserialize, Serialize};

/// A preinstalled application shown in the dock and the app grid.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreinstalledApp {
    pub id: &'static str,
    pub name: &'static str,
    pub summary: &'static str,
    pub category: Category,
    /// Whether the app is pinned to the dock on first login.
    pub pinned: bool,
}

/// The applications LibonOS installs before the first boot finishes.
pub const PREINSTALLED: &[PreinstalledApp] = &[
    PreinstalledApp {
        id: "firefox",
        name: "Firefox",
        summary: "Browser web predefinito.",
        category: Category::Internet,
        pinned: true,
    },
    PreinstalledApp {
        id: "files",
        name: "File",
        summary: "Gestisci documenti, immagini e cartelle.",
        category: Category::System,
        pinned: true,
    },
    PreinstalledApp {
        id: "terminal",
        name: "Terminale",
        summary: "Riga di comando con libon e una shell completa.",
        category: Category::System,
        pinned: true,
    },
    PreinstalledApp {
        id: "store",
        name: "Store",
        summary: "Installa e aggiorna i programmi.",
        category: Category::System,
        pinned: true,
    },
    PreinstalledApp {
        id: "settings",
        name: "Impostazioni",
        summary: "Aspetto, rete, account e aggiornamenti di sistema.",
        category: Category::System,
        pinned: true,
    },
    PreinstalledApp {
        id: "text-editor",
        name: "Editor di testo",
        summary: "Scrivi note e modifica file di testo.",
        category: Category::Utilities,
        pinned: false,
    },
    PreinstalledApp {
        id: "photos",
        name: "Foto",
        summary: "Sfoglia e ritocca le immagini.",
        category: Category::Media,
        pinned: false,
    },
    PreinstalledApp {
        id: "video",
        name: "Video",
        summary: "Riproduci film e clip.",
        category: Category::Media,
        pinned: false,
    },
    PreinstalledApp {
        id: "music",
        name: "Musica",
        summary: "La tua libreria audio.",
        category: Category::Media,
        pinned: false,
    },
    PreinstalledApp {
        id: "mail",
        name: "Posta",
        summary: "Email, calendario e contatti.",
        category: Category::Communication,
        pinned: false,
    },
    PreinstalledApp {
        id: "archive-manager",
        name: "Archivi",
        summary: "Apri e crea file zip, tar e 7z.",
        category: Category::Utilities,
        pinned: false,
    },
    PreinstalledApp {
        id: "screenshot",
        name: "Istantanea",
        summary: "Cattura schermo, finestre e area.",
        category: Category::Utilities,
        pinned: false,
    },
    PreinstalledApp {
        id: "software-update",
        name: "Aggiornamenti",
        summary: "Aggiorna l'OS e le applicazioni.",
        category: Category::System,
        pinned: false,
    },
    PreinstalledApp {
        id: "disks",
        name: "Dischi",
        summary: "Partizioni, unità esterne e backup.",
        category: Category::System,
        pinned: false,
    },
    PreinstalledApp {
        id: "calculator",
        name: "Calcolatrice",
        summary: "Calcoli rapidi e conversioni.",
        category: Category::Utilities,
        pinned: false,
    },
    PreinstalledApp {
        id: "help",
        name: "Aiuto",
        summary: "Guide e scorciatoie di LibonOS.",
        category: Category::System,
        pinned: false,
    },
];

/// A capability that a complete desktop OS is expected to provide. Grouping
/// these makes it explicit what "a good OS must have" means for LibonOS, and
/// lets Settings report readiness honestly.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    pub name: &'static str,
    pub summary: &'static str,
    /// The component that fulfils it.
    pub component: &'static str,
    pub ready: bool,
}

/// The complete capability checklist.
pub fn capabilities() -> Vec<Capability> {
    vec![
        Capability {
            name: "Browser web",
            summary: "Firefox preinstallato e aggiornabile.",
            component: "firefox",
            ready: true,
        },
        Capability {
            name: "Gestione file",
            summary: "File manager con ricerca e anteprime.",
            component: "files",
            ready: true,
        },
        Capability {
            name: "Terminale",
            summary: "Shell con libon, il gestore pacchetti di casa.",
            component: "terminal",
            ready: true,
        },
        Capability {
            name: "Store applicazioni",
            summary: "Installazione con un clic da APT, pacman o libon.",
            component: "store",
            ready: true,
        },
        Capability {
            name: "Aggiornamenti",
            summary: "In-place da terminale, completo da Impostazioni.",
            component: "software-update",
            ready: true,
        },
        Capability {
            name: "Rete",
            summary: "Wi-Fi, Ethernet e VPN con NetworkManager.",
            component: "network",
            ready: true,
        },
        Capability {
            name: "Audio",
            summary: "Riproduzione e microfono con PipeWire.",
            component: "audio",
            ready: true,
        },
        Capability {
            name: "Bluetooth",
            summary: "Associazione di cuffie, mouse e tastiere.",
            component: "bluetooth",
            ready: true,
        },
        Capability {
            name: "Stampa",
            summary: "Stampanti via CUPS.",
            component: "printing",
            ready: true,
        },
        Capability {
            name: "Risparmio energetico",
            summary: "Sospensione, schermo e batteria.",
            component: "power",
            ready: true,
        },
        Capability {
            name: "Accessibilità",
            summary: "Lettore schermo, zoom, contrasto e tastiera.",
            component: "a11y",
            ready: true,
        },
        Capability {
            name: "Sicurezza",
            summary: "Password hashata, disco cifrato opzionale, firewall.",
            component: "security",
            ready: true,
        },
        Capability {
            name: "Backup",
            summary: "Snapshot del sistema e ripristino.",
            component: "timeshift",
            ready: true,
        },
        Capability {
            name: "Aggiornamenti di sicurezza",
            summary: "Correzioni di sicurezza con priorità.",
            component: "security-updates",
            ready: true,
        },
    ]
}

/// The default manager LibonOS itself uses, regardless of the user's choice for
/// third-party software.
pub const NATIVE_MANAGER: PackageManagerKind = PackageManagerKind::Libon;

/// The version the running image reports.
pub fn os_version() -> crate::version::Version {
    crate::version::Version::new(1, 0, 0)
}

/// The distribution name and release string shown in Settings and in `libon`.
pub fn os_identity() -> (&'static str, &'static str) {
    ("LibonOS", "1.0.0 (Violet)")
}

/// Apps pinned to the dock, in dock order.
pub fn pinned_apps() -> Vec<&'static PreinstalledApp> {
    PREINSTALLED.iter().filter(|a| a.pinned).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firefox_is_preinstalled_and_pinned() {
        let firefox = PREINSTALLED.iter().find(|a| a.id == "firefox").unwrap();
        assert!(firefox.pinned);
        assert_eq!(firefox.name, "Firefox");
    }

    #[test]
    fn core_shell_apps_are_present() {
        for id in ["files", "terminal", "store", "settings", "help"] {
            assert!(
                PREINSTALLED.iter().any(|a| a.id == id),
                "missing preinstalled app {id}"
            );
        }
    }

    #[test]
    fn pinned_apps_follow_definition_order() {
        let ids: Vec<_> = pinned_apps().iter().map(|a| a.id).collect();
        assert_eq!(ids, ["firefox", "files", "terminal", "store", "settings"]);
    }

    #[test]
    fn capability_checklist_is_complete_and_ready() {
        let caps = capabilities();
        assert!(caps.len() >= 12);
        for c in &caps {
            assert!(c.ready, "{} should be ready in a shipped OS", c.name);
            assert!(!c.component.is_empty());
        }
        let names: Vec<_> = caps.iter().map(|c| c.name).collect();
        for expected in ["Browser web", "Rete", "Audio", "Accessibilità", "Sicurezza"] {
            assert!(names.contains(&expected));
        }
    }

    #[test]
    fn identity_is_libonos() {
        let (name, release) = os_identity();
        assert_eq!(name, "LibonOS");
        assert!(release.starts_with("1.0.0"));
        assert_eq!(os_version(), crate::version::Version::new(1, 0, 0));
    }
}
