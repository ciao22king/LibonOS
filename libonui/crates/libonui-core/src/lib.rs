//! **libonui-core** — the headless heart of the LibonOS desktop shell.
//!
//! Everything that can be decided without a screen lives here: the design
//! tokens, the account and first-run flow, the package-manager abstraction, the
//! OS update policy, and the window manager. The `libonui-app` crate draws it,
//! and `libon` (the CLI) drives the same package and update logic from the
//! terminal.
//!
//! Keeping this crate free of any UI dependency is what makes the shell's rules
//! testable: geometry, hashing and update classification are verified by ordinary
//! unit tests.

pub mod accounts;
pub mod channels;
pub mod config;
pub mod onboarding;
pub mod packages;
pub mod system;
pub mod tokens;
pub mod version;
pub mod wm;

pub use accounts::{Account, AuthError};
pub use config::{DesktopConfig, StatePaths, WallpaperStyle};
pub use onboarding::{SetupAnswers, SetupState, SetupStep};
pub use packages::{manager_for, Package, PackageManager, PackageManagerKind};
pub use system::{os_identity, os_version, PREINSTALLED};
pub use tokens::{ThemeChoice, Tokens};
pub use version::{classify_update, UpdateKind, UpdateStatus, Version};
pub use wm::{Session, WindowId, WindowManager, WindowState};

/// A one-line banner used by the app and the CLI, so both report the same thing.
pub fn banner() -> String {
    let (name, release) = os_identity();
    format!("{name} {release}")
}
