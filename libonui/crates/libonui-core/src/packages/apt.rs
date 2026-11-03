//! The APT backend — Debian and Ubuntu style repositories.

use super::{search_catalog, Category, Package, PackageManager, PackageManagerKind};

/// A compact slice of the Debian/Ubuntu catalog, enough to fill the Store on a
/// fresh install. A real deployment refreshes the full index with
/// `apt-get update` and reads `apt-cache search`.
const CATALOG: &[(&str, &str, &str, Category, u64)] = &[
    (
        "firefox",
        "Firefox",
        "Browser web di Mozilla, veloce e rispettoso della privacy.",
        Category::Internet,
        78_000_000,
    ),
    (
        "thunderbird",
        "Thunderbird",
        "Posta, calendario e contatti.",
        Category::Communication,
        62_000_000,
    ),
    (
        "chromium",
        "Chromium",
        "Il browser open source su cui si basa Chrome.",
        Category::Internet,
        92_000_000,
    ),
    (
        "gimp",
        "GIMP",
        "Editor di immagini e fotoritocco.",
        Category::Media,
        45_000_000,
    ),
    (
        "inkscape",
        "Inkscape",
        "Illustrazioni vettoriali in SVG.",
        Category::Media,
        68_000_000,
    ),
    (
        "blender",
        "Blender",
        "Modellazione, animazione e render 3D.",
        Category::Media,
        320_000_000,
    ),
    (
        "vlc",
        "VLC",
        "Legge praticamente qualsiasi file audio e video.",
        Category::Media,
        55_000_000,
    ),
    (
        "libreoffice",
        "LibreOffice",
        "Suite completa: documenti, fogli, presentazioni.",
        Category::Office,
        240_000_000,
    ),
    (
        "code",
        "Visual Studio Code",
        "Editor di codice estendibile.",
        Category::Development,
        105_000_000,
    ),
    (
        "git",
        "Git",
        "Controllo di versione distribuito.",
        Category::Development,
        12_000_000,
    ),
    (
        "python3",
        "Python 3",
        "Linguaggio di programmazione versatile.",
        Category::Development,
        22_000_000,
    ),
    (
        "docker.io",
        "Docker",
        "Contenitori per sviluppare e distribuire.",
        Category::Development,
        68_000_000,
    ),
    (
        "htop",
        "htop",
        "Monitor interattivo dei processi.",
        Category::Utilities,
        1_200_000,
    ),
    (
        "neofetch",
        "neofetch",
        "Riepilogo del sistema nel terminale.",
        Category::Utilities,
        180_000,
    ),
    (
        "curl",
        "curl",
        "Trasferimento dati da riga di comando.",
        Category::Utilities,
        1_500_000,
    ),
    (
        "timeshift",
        "Timeshift",
        "Snapshot e ripristino del sistema.",
        Category::System,
        9_800_000,
    ),
    (
        "gnome-disk-utility",
        "Dischi",
        "Partizioni, formattazione e backup.",
        Category::System,
        7_400_000,
    ),
    (
        "steam",
        "Steam",
        "Negozio e libreria di videogiochi.",
        Category::Games,
        4_600_000,
    ),
];

pub struct Apt;

impl Apt {
    fn catalog(&self) -> Vec<Package> {
        CATALOG
            .iter()
            .map(|(id, name, summary, category, size)| Package {
                id: (*id).into(),
                name: (*name).into(),
                summary: (*summary).into(),
                version: "candidate".into(),
                size_bytes: *size,
                installed: false,
                category: *category,
                manager: PackageManagerKind::Apt,
            })
            .collect()
    }
}

impl PackageManager for Apt {
    fn kind(&self) -> PackageManagerKind {
        PackageManagerKind::Apt
    }

    fn search(&self, query: &str) -> Vec<Package> {
        search_catalog(&self.catalog(), query)
    }

    fn featured(&self) -> Vec<Package> {
        let catalog = self.catalog();
        ["firefox", "libreoffice", "code", "vlc", "steam", "gimp"]
            .iter()
            .filter_map(|id| catalog.iter().find(|p| p.id == *id).cloned())
            .collect()
    }

    fn install_args(&self, id: &str) -> Vec<String> {
        vec!["apt-get".into(), "install".into(), "-y".into(), id.into()]
    }

    fn remove_args(&self, id: &str) -> Vec<String> {
        vec![
            "apt-get".into(),
            "remove".into(),
            "--purge".into(),
            "-y".into(),
            id.into(),
        ]
    }

    fn refresh_args(&self) -> Vec<String> {
        vec!["apt-get".into(), "update".into()]
    }

    fn upgrade_args(&self) -> Vec<String> {
        vec!["apt-get".into(), "upgrade".into(), "-y".into()]
    }
}
