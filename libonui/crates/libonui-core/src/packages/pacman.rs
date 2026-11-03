//! The pacman backend — Arch-style rolling repositories.

use super::{search_catalog, Category, Package, PackageManager, PackageManagerKind};

const CATALOG: &[(&str, &str, &str, Category, u64)] = &[
    (
        "firefox",
        "Firefox",
        "Browser web di Mozilla.",
        Category::Internet,
        82_000_000,
    ),
    (
        "chromium",
        "Chromium",
        "Browser open source.",
        Category::Internet,
        95_000_000,
    ),
    (
        "thunderbird",
        "Thunderbird",
        "Posta e calendario.",
        Category::Communication,
        64_000_000,
    ),
    (
        "telegram-desktop",
        "Telegram",
        "Messaggistica istantanea.",
        Category::Communication,
        88_000_000,
    ),
    (
        "gimp",
        "GIMP",
        "Fotoritocco e grafica raster.",
        Category::Media,
        47_000_000,
    ),
    (
        "kdenlive",
        "Kdenlive",
        "Montaggio video non lineare.",
        Category::Media,
        210_000_000,
    ),
    (
        "mpv",
        "mpv",
        "Lettore multimediale minimale.",
        Category::Media,
        6_200_000,
    ),
    (
        "libreoffice-fresh",
        "LibreOffice",
        "Suite per l'ufficio, canale fresh.",
        Category::Office,
        255_000_000,
    ),
    (
        "code",
        "Visual Studio Code",
        "Editor di codice estendibile.",
        Category::Development,
        110_000_000,
    ),
    (
        "rust",
        "Rust",
        "Toolchain del linguaggio Rust.",
        Category::Development,
        420_000_000,
    ),
    (
        "git",
        "Git",
        "Controllo di versione distribuito.",
        Category::Development,
        13_000_000,
    ),
    (
        "htop",
        "htop",
        "Monitor dei processi.",
        Category::Utilities,
        1_100_000,
    ),
    (
        "fastfetch",
        "fastfetch",
        "Riepilogo di sistema rapido.",
        Category::Utilities,
        240_000,
    ),
    (
        "reflector",
        "reflector",
        "Sceglie i mirror più veloci.",
        Category::System,
        60_000,
    ),
    (
        "timeshift",
        "Timeshift",
        "Snapshot di sistema.",
        Category::System,
        10_100_000,
    ),
    (
        "steam",
        "Steam",
        "Libreria di videogiochi.",
        Category::Games,
        4_900_000,
    ),
];

pub struct Pacman;

impl Pacman {
    fn catalog(&self) -> Vec<Package> {
        CATALOG
            .iter()
            .map(|(id, name, summary, category, size)| Package {
                id: (*id).into(),
                name: (*name).into(),
                summary: (*summary).into(),
                version: "rolling".into(),
                size_bytes: *size,
                installed: false,
                category: *category,
                manager: PackageManagerKind::Pacman,
            })
            .collect()
    }
}

impl PackageManager for Pacman {
    fn kind(&self) -> PackageManagerKind {
        PackageManagerKind::Pacman
    }

    fn search(&self, query: &str) -> Vec<Package> {
        search_catalog(&self.catalog(), query)
    }

    fn featured(&self) -> Vec<Package> {
        let catalog = self.catalog();
        [
            "firefox",
            "libreoffice-fresh",
            "code",
            "mpv",
            "steam",
            "gimp",
        ]
        .iter()
        .filter_map(|id| catalog.iter().find(|p| p.id == *id).cloned())
        .collect()
    }

    fn install_args(&self, id: &str) -> Vec<String> {
        vec![
            "pacman".into(),
            "-S".into(),
            "--noconfirm".into(),
            id.into(),
        ]
    }

    fn remove_args(&self, id: &str) -> Vec<String> {
        vec![
            "pacman".into(),
            "-Rns".into(),
            "--noconfirm".into(),
            id.into(),
        ]
    }

    fn refresh_args(&self) -> Vec<String> {
        vec!["pacman".into(), "-Sy".into()]
    }

    fn upgrade_args(&self) -> Vec<String> {
        vec!["pacman".into(), "-Su".into(), "--noconfirm".into()]
    }
}
