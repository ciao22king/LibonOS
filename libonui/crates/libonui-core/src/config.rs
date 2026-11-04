//! Persisted settings and the on-disk layout of the shell's state.

use crate::i18n::Locale;
use crate::packages::PackageManagerKind;
use crate::tokens::ThemeChoice;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

/// The desktop background treatment. Each is generated from the palette rather
/// than shipped as a bitmap, so it stays crisp at any resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WallpaperStyle {
    /// A soft two-tone vertical wash — the default.
    Aurora,
    /// A slow drifting field of violet and cyan blobs.
    Mesh,
    /// A single flat canvas colour, for the least distraction.
    Solid,
}

impl WallpaperStyle {
    pub const ALL: [WallpaperStyle; 3] = [
        WallpaperStyle::Aurora,
        WallpaperStyle::Mesh,
        WallpaperStyle::Solid,
    ];

    pub fn label(self) -> &'static str {
        match self {
            WallpaperStyle::Aurora => "Aurora",
            WallpaperStyle::Mesh => "Mesh",
            WallpaperStyle::Solid => "Tinta unita",
        }
    }
}

/// Everything the Settings app can change, stored as one document.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DesktopConfig {
    /// The interface language, chosen during setup and changeable from Settings.
    pub locale: Locale,
    pub theme: ThemeChoice,
    pub wallpaper: WallpaperStyle,
    pub accent_cyan: bool,
    pub package_manager: PackageManagerKind,
    pub telemetry_enabled: bool,
    pub automatic_updates: bool,
    pub beta_channel: bool,
    pub animations: bool,
    pub night_light: bool,
    pub font_scale: f32,
    pub keyboard_layout: String,
    pub timezone: String,
    pub hostname: String,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            // A machine that has never been through setup still opens in the
            // language it was booted in.
            locale: Locale::from_env(),
            theme: ThemeChoice::Dark,
            wallpaper: WallpaperStyle::Aurora,
            accent_cyan: false,
            package_manager: PackageManagerKind::Libon,
            telemetry_enabled: false,
            automatic_updates: true,
            beta_channel: false,
            animations: true,
            night_light: false,
            font_scale: 1.0,
            keyboard_layout: "it".into(),
            timezone: "Europe/Rome".into(),
            hostname: "libon-pc".into(),
        }
    }
}

impl DesktopConfig {
    /// Read settings from `path`, falling back to defaults when the file is
    /// missing or unreadable. A corrupt file must never block the desktop.
    pub fn load_or_default(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Write settings, creating the parent directory if needed.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(io::Error::other)?;
        std::fs::write(path, text)
    }

    /// The font scale clamped to the range the UI supports.
    pub fn clamped_font_scale(&self) -> f32 {
        self.font_scale.clamp(0.8, 1.6)
    }
}

/// Where the shell keeps its files.
#[derive(Clone, Debug)]
pub struct StatePaths {
    pub root: PathBuf,
}

impl StatePaths {
    /// System-wide state, used by the installer and the login service.
    pub fn system() -> Self {
        Self {
            root: PathBuf::from("/var/lib/libonui"),
        }
    }

    /// Per-user state under `$XDG_DATA_HOME` (or `~/.local/share`).
    pub fn user() -> Self {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
        Self {
            root: base.join("libonui"),
        }
    }

    /// A state directory at an explicit path, for tests and for a live session
    /// running out of a checkout.
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn account_file(&self) -> PathBuf {
        self.root.join("account.toml")
    }

    pub fn first_run_file(&self) -> PathBuf {
        self.root.join("first-run.toml")
    }

    pub fn config_file(&self) -> PathBuf {
        self.root.join("settings.toml")
    }

    pub fn session_file(&self) -> PathBuf {
        self.root.join("session.toml")
    }

    pub fn ensure(&self) -> io::Result<()> {
        std::fs::create_dir_all(&self.root)
    }

    /// Whether setup has already been completed on this machine.
    pub fn is_first_run(&self) -> bool {
        !self.first_run_file().exists()
    }

    pub fn load_account(&self) -> Option<crate::accounts::Account> {
        let text = std::fs::read_to_string(self.account_file()).ok()?;
        toml::from_str(&text).ok()
    }

    pub fn save_account(&self, account: &crate::accounts::Account) -> io::Result<()> {
        self.ensure()?;
        let text = toml::to_string_pretty(account).map_err(io::Error::other)?;
        std::fs::write(self.account_file(), text)
    }

    pub fn load_first_run(&self) -> Option<crate::onboarding::FirstRunRecord> {
        let text = std::fs::read_to_string(self.first_run_file()).ok()?;
        toml::from_str(&text).ok()
    }

    pub fn save_first_run(&self, record: &crate::onboarding::FirstRunRecord) -> io::Result<()> {
        self.ensure()?;
        let text = toml::to_string_pretty(record).map_err(io::Error::other)?;
        std::fs::write(self.first_run_file(), text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "libonui-test-{}-{}",
            std::process::id(),
            rand::random::<u32>()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn config_round_trips_through_toml() {
        let dir = tmp();
        let path = dir.join("settings.toml");
        let cfg = DesktopConfig {
            theme: ThemeChoice::Light,
            package_manager: PackageManagerKind::Pacman,
            font_scale: 1.2,
            ..DesktopConfig::default()
        };
        cfg.save(&path).unwrap();
        let loaded = DesktopConfig::load_or_default(&path);
        assert_eq!(loaded, cfg);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_or_corrupt_config_falls_back_to_defaults() {
        let dir = tmp();
        assert_eq!(
            DesktopConfig::load_or_default(&dir.join("nope.toml")),
            DesktopConfig::default()
        );
        let bad = dir.join("bad.toml");
        std::fs::write(&bad, "this is not = = toml").unwrap();
        assert_eq!(
            DesktopConfig::load_or_default(&bad),
            DesktopConfig::default()
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn first_run_transitions_to_completed() {
        let dir = tmp();
        let paths = StatePaths::at(&dir);
        assert!(paths.is_first_run());
        let record = crate::onboarding::FirstRunRecord {
            completed: true,
            locale: crate::i18n::Locale::En,
            package_manager: PackageManagerKind::Libon,
            telemetry_enabled: false,
            automatic_updates: true,
            completed_utc: "2026-01-01T00:00:00Z".into(),
        };
        paths.save_first_run(&record).unwrap();
        assert!(!paths.is_first_run());
        assert_eq!(paths.load_first_run().unwrap(), record);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn account_round_trips() {
        let dir = tmp();
        let paths = StatePaths::at(&dir);
        let acct = crate::accounts::create_account("Sara", "libon2026os", true);
        paths.save_account(&acct).unwrap();
        let loaded = paths.load_account().unwrap();
        assert_eq!(loaded.first_name, "Sara");
        assert!(loaded.verify("libon2026os"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn font_scale_is_clamped() {
        let mut cfg = DesktopConfig {
            font_scale: 5.0,
            ..DesktopConfig::default()
        };
        assert_eq!(cfg.clamped_font_scale(), 1.6);
        cfg.font_scale = 0.1;
        assert_eq!(cfg.clamped_font_scale(), 0.8);
    }
}
