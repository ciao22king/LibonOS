//! The `libon` backend — LibonOS's own package manager.
//!
//! It works differently from APT and pacman: `libon install <app>` looks the app
//! up online, downloads a single self-contained **AppImage**, drops it in the
//! user's `~/Applications` and registers a menu entry. Nothing needs root, and
//! the app can be run straight away.
//!
//! OS updates are a separate command — `libon update` for in-place updates and
//! the Settings app for a full major-version upgrade.

use super::{search_catalog, Category, Package, PackageManager, PackageManagerKind};

/// Where libon keeps the AppImages it downloads.
pub const APPIMAGE_DIR: &str = "~/Applications";

/// The catalog libon knows about out of the box. Everything else is resolved
/// online at install time.
const CATALOG: &[(&str, &str, &str, Category, u64)] = &[
    (
        "firefox",
        "Firefox",
        "Browser web di Mozilla. Preinstallato su LibonOS.",
        Category::Internet,
        75_000_000,
    ),
    (
        "chromium",
        "Chromium",
        "Browser open source.",
        Category::Internet,
        90_000_000,
    ),
    (
        "vivaldi",
        "Vivaldi",
        "Browser personalizzabile.",
        Category::Internet,
        96_000_000,
    ),
    (
        "brave",
        "Brave",
        "Browser con blocco dei tracker integrato.",
        Category::Internet,
        120_000_000,
    ),
    (
        "telegram",
        "Telegram Desktop",
        "Messaggistica veloce e sicura.",
        Category::Communication,
        62_000_000,
    ),
    (
        "discord",
        "Discord",
        "Voce, video e chat per community.",
        Category::Communication,
        85_000_000,
    ),
    (
        "signal",
        "Signal",
        "Messaggistica cifrata end-to-end.",
        Category::Communication,
        110_000_000,
    ),
    (
        "obsidian",
        "Obsidian",
        "Note e appunti in Markdown.",
        Category::Office,
        130_000_000,
    ),
    (
        "libreoffice",
        "LibreOffice",
        "Suite per l'ufficio.",
        Category::Office,
        240_000_000,
    ),
    (
        "gimp",
        "GIMP",
        "Fotoritocco e grafica raster.",
        Category::Media,
        45_000_000,
    ),
    (
        "kdenlive",
        "Kdenlive",
        "Montaggio video.",
        Category::Media,
        210_000_000,
    ),
    (
        "audacity",
        "Audacity",
        "Registrazione e editing audio.",
        Category::Media,
        40_000_000,
    ),
    (
        "blender",
        "Blender",
        "Grafica e animazione 3D.",
        Category::Media,
        320_000_000,
    ),
    (
        "vscodium",
        "VSCodium",
        "VS Code senza telemetria.",
        Category::Development,
        108_000_000,
    ),
    (
        "zed",
        "Zed",
        "Editor di codice collaborativo e veloce.",
        Category::Development,
        92_000_000,
    ),
    (
        "godot",
        "Godot",
        "Motore per videogiochi 2D e 3D.",
        Category::Development,
        120_000_000,
    ),
    (
        "keepassxc",
        "KeePassXC",
        "Gestore di password offline.",
        Category::Utilities,
        34_000_000,
    ),
    (
        "localsend",
        "LocalSend",
        "Condivisione file sulla rete locale.",
        Category::Utilities,
        26_000_000,
    ),
    (
        "balena-etcher",
        "balenaEtcher",
        "Scrive immagini su USB e SD.",
        Category::System,
        118_000_000,
    ),
    (
        "ventoy",
        "Ventoy",
        "USB multiboot per immagini ISO.",
        Category::System,
        42_000_000,
    ),
];

pub struct Libon;

impl Libon {
    fn catalog(&self) -> Vec<Package> {
        CATALOG
            .iter()
            .map(|(id, name, summary, category, size)| Package {
                id: (*id).into(),
                name: (*name).into(),
                summary: (*summary).into(),
                version: "latest".into(),
                size_bytes: *size,
                installed: false,
                category: *category,
                manager: PackageManagerKind::Libon,
            })
            .collect()
    }
}

impl PackageManager for Libon {
    fn kind(&self) -> PackageManagerKind {
        PackageManagerKind::Libon
    }

    fn search(&self, query: &str) -> Vec<Package> {
        search_catalog(&self.catalog(), query)
    }

    fn featured(&self) -> Vec<Package> {
        let catalog = self.catalog();
        [
            "firefox",
            "telegram",
            "obsidian",
            "gimp",
            "vscodium",
            "localsend",
        ]
        .iter()
        .filter_map(|id| catalog.iter().find(|p| p.id == *id).cloned())
        .collect()
    }

    /// `libon install <app>`: resolve online, download the AppImage, register it.
    fn install_args(&self, id: &str) -> Vec<String> {
        vec!["libon".into(), "install".into(), id.into()]
    }

    fn remove_args(&self, id: &str) -> Vec<String> {
        vec!["libon".into(), "remove".into(), id.into()]
    }

    fn refresh_args(&self) -> Vec<String> {
        vec!["libon".into(), "refresh".into()]
    }

    /// The AppImage world has no system-wide "upgrade all": `libon update`
    /// refreshes the app index and updates the installed AppImages, while the
    /// operating system itself is updated separately.
    fn upgrade_args(&self) -> Vec<String> {
        vec!["libon".into(), "update".into()]
    }
}
