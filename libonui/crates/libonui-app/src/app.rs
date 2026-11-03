//! The shell's top-level state: which stage we are in, and the state each
//! screen needs.
//!
//! The stages are deliberately few and explicit — `Setup`, `Login`, `Desktop` —
//! because the whole point of the first-run flow is that it happens exactly
//! once and cannot be bypassed.

use crate::store::StoreState;
use crate::theme::{Fonts, Theme};
use libonui_core::config::{DesktopConfig, StatePaths, WallpaperStyle};
use libonui_core::onboarding::{FirstRunRecord, SetupState};
use libonui_core::packages::{manager_for, PackageManager, PackageManagerKind};
use libonui_core::tokens::ThemeChoice;
use libonui_core::version::{UpdateStatus, Version};
use libonui_core::{channels, system, Session};
use std::sync::Arc;

/// Which stage of the shell's life we are in.
pub enum Stage {
    /// First boot: the setup questions.
    Setup(SetupState),
    /// An account exists and the user must unlock it.
    Login(LoginState),
    /// The desktop.
    Desktop(Box<Desktop>),
}

/// The login screen's transient state.
pub struct LoginState {
    pub password: String,
    pub error: Option<String>,
    pub first_name: String,
    pub attempts: u32,
}

/// A desktop notification, shown as a card in the top-right corner.
#[derive(Clone)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub kind: NotifyKind,
    pub created: std::time::Instant,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NotifyKind {
    Info,
    Success,
    Warning,
    Error,
}

impl Notification {
    pub fn new(title: impl Into<String>, body: impl Into<String>, kind: NotifyKind) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            kind,
            created: std::time::Instant::now(),
        }
    }
    /// Notifications fade after a while so they never pile up.
    pub fn age_secs(&self) -> f32 {
        self.created.elapsed().as_secs_f32()
    }
}

/// The running desktop's state.
pub struct Desktop {
    pub session: Session,
    /// Where settings are written; kept here so a screen can save without the
    /// shell threading a path through every call.
    pub config_path: std::path::PathBuf,
    pub config: DesktopConfig,
    pub manager: Arc<dyn PackageManager>,
    pub store: StoreState,
    pub update: UpdateStatus,
    pub major_update: Option<UpdateStatus>,
    pub notifications: Vec<Notification>,
    pub settings_tab: SettingsTab,
    pub clock: chrono::DateTime<chrono::Local>,
    /// Text typed into the top-panel search field.
    pub panel_search: String,
    /// Whether the session menu in the top panel is open.
    pub session_menu_open: bool,
    /// Whether the notification centre (the panel bell) is open.
    pub bell_open: bool,
    /// Whether the missing-backend warning has already been shown this session.
    pub manager_warned: bool,
    /// A running OS update: when it started, and whether it crosses a major line.
    pub update_run: Option<UpdateRun>,
}

/// An in-flight OS update.
pub struct UpdateRun {
    pub started: std::time::Instant,
    pub major: bool,
    pub target: Version,
}

impl UpdateRun {
    /// Progress in `0.0..=1.0`. A major upgrade rewrites the whole image, so it
    /// takes longer than an in-place one.
    pub fn progress(&self) -> f32 {
        let total = if self.major { 9.0 } else { 4.0 };
        (self.started.elapsed().as_secs_f32() / total).clamp(0.0, 1.0)
    }

    /// The step the update is on, for the progress label.
    pub fn stage_label(&self) -> &'static str {
        let p = self.progress();
        if self.major {
            match p {
                x if x < 0.2 => "Preparazione dell'aggiornamento di sistema…",
                x if x < 0.45 => "Download dell'immagine completa…",
                x if x < 0.7 => "Verifica della firma dell'immagine…",
                x if x < 0.9 => "Scrittura della nuova versione…",
                _ => "Riavvio imminente…",
            }
        } else {
            match p {
                x if x < 0.3 => "Download dei pacchetti…",
                x if x < 0.6 => "Applicazione degli aggiornamenti…",
                x if x < 0.9 => "Riavvio dei servizi…",
                _ => "Pulizia dei file temporanei…",
            }
        }
    }
}

/// Settings navigation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Appearance,
    Updates,
    Packages,
    Account,
    Network,
    Privacy,
    About,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 7] = [
        SettingsTab::Appearance,
        SettingsTab::Updates,
        SettingsTab::Packages,
        SettingsTab::Account,
        SettingsTab::Network,
        SettingsTab::Privacy,
        SettingsTab::About,
    ];
    pub fn label(self) -> &'static str {
        match self {
            SettingsTab::Appearance => "Aspetto",
            SettingsTab::Updates => "Aggiornamenti",
            SettingsTab::Packages => "Pacchetti",
            SettingsTab::Account => "Account",
            SettingsTab::Network => "Rete",
            SettingsTab::Privacy => "Privacy",
            SettingsTab::About => "Informazioni",
        }
    }
}

/// How the shell was launched, so a demo run never touches real user files.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    /// Normal: read and write the user's state.
    Normal,
    /// Demo: start with a populated desktop and an isolated state directory.
    Demo,
}

pub struct App {
    pub stage: Stage,
    pub theme: Theme,
    pub fonts: Fonts,
    pub paths: StatePaths,
    /// Set when the user closes the session; the runner turns this into an exit.
    pub should_exit: bool,
    /// Present only in `--shots` mode, where it drives the screenshot script.
    pub script: Option<crate::shots::ShotScript>,
}

impl App {
    /// Build the app for the state on disk: setup on a fresh machine, login
    /// otherwise.
    pub fn new(paths: StatePaths, mode: LaunchMode) -> Self {
        let config = DesktopConfig::load_or_default(&paths.config_file());
        let theme = Theme::from_tokens(config.theme.tokens());
        let fonts = Fonts::from(&theme.tokens);

        let stage = if mode == LaunchMode::Demo {
            Stage::Desktop(Box::new(Desktop::demo(config.clone())))
        } else if paths.is_first_run() {
            Stage::Setup(SetupState::default())
        } else {
            let first_name = paths
                .load_account()
                .map(|a| a.first_name)
                .unwrap_or_else(|| "utente".into());
            Stage::Login(LoginState {
                password: String::new(),
                error: None,
                first_name,
                attempts: 0,
            })
        };

        Self {
            stage,
            theme,
            fonts,
            paths,
            should_exit: false,
            script: None,
        }
    }

    /// Rebuild the theme from the current configuration. Call after any change
    /// to the theme, wallpaper or accent.
    pub fn refresh_theme(&mut self) {
        let choice = match &self.stage {
            Stage::Desktop(d) => d.config.theme,
            _ => ThemeChoice::Dark,
        };
        self.theme = Theme::from_tokens(choice.tokens());
        self.fonts = Fonts::from(&self.theme.tokens);
    }

    /// Load a text face that covers Italian diacritics well. We look for a
    /// handful of system fonts and fall back to egui's bundled faces, so the
    /// shell still renders correctly on a bare image.
    pub fn install_fonts(&self, ctx: &egui::Context) {
        let candidates = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
        ];
        let Some(bytes) = candidates.iter().find_map(|p| std::fs::read(p).ok()) else {
            return;
        };
        let mut fonts = egui::FontDefinitions::default();
        fonts
            .font_data
            .insert("libon-sans".into(), egui::FontData::from_owned(bytes));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "libon-sans".into());
        ctx.set_fonts(fonts);
    }

    /// Persist the current desktop configuration.
    pub fn save_config(&self) {
        if let Stage::Desktop(d) = &self.stage {
            let _ = d.config.save(&self.paths.config_file());
        }
    }

    /// Complete setup: write the account and the first-run record, then open the
    /// desktop.
    pub fn complete_setup(&mut self) {
        let Stage::Setup(setup) = &self.stage else {
            return;
        };
        let Ok(account) = setup.build_account() else {
            return;
        };
        let _ = self.paths.ensure();
        let _ = self.paths.save_account(&account);
        let _ = self
            .paths
            .save_first_run(&FirstRunRecord::from_answers(&setup.answers));

        let config = DesktopConfig {
            package_manager: setup.answers.package_manager,
            telemetry_enabled: setup.answers.telemetry_enabled,
            automatic_updates: setup.answers.automatic_updates,
            ..DesktopConfig::default()
        };
        let _ = config.save(&self.paths.config_file());

        self.stage = Stage::Desktop(Box::new(Desktop::new(
            account.first_name.clone(),
            account.username.clone(),
            config,
        )));
        self.refresh_theme();
    }

    /// Try to unlock with the typed password.
    pub fn attempt_login(&mut self) {
        let Stage::Login(login) = &mut self.stage else {
            return;
        };
        let Some(account) = self.paths.load_account() else {
            login.error = Some("Nessun account trovato. Ripeti la configurazione.".into());
            return;
        };
        if account.verify(&login.password) {
            let config = DesktopConfig::load_or_default(&self.paths.config_file());
            let name = account.first_name.clone();
            let username = account.username.clone();
            self.stage = Stage::Desktop(Box::new(Desktop::new(name, username, config)));
            self.refresh_theme();
        } else {
            login.attempts += 1;
            login.error = Some(if login.attempts >= 3 {
                "Password errata. Dopo altri tentativi la sessione verrà chiusa.".into()
            } else {
                "Password errata. Riprova.".into()
            });
            login.password.clear();
        }
    }
}

impl Desktop {
    pub fn new(
        first_name: impl Into<String>,
        username: impl Into<String>,
        config: DesktopConfig,
    ) -> Self {
        let session = Session::new(first_name, username);
        let manager: Arc<dyn PackageManager> = Arc::from(manager_for(config.package_manager));
        let current = system::os_version();
        let channel = if config.beta_channel {
            channels::bundled_beta()
        } else {
            channels::bundled_stable()
        };
        let update = UpdateStatus::check(&current, &channel);
        let major_update = UpdateStatus::check_major_upgrade(&current, &channel);

        let mut store = StoreState::new(manager.clone());
        store.load_featured();

        Self {
            session,
            config_path: StatePaths::user().config_file(),
            config,
            manager,
            store,
            update,
            major_update,
            notifications: Vec::new(),
            settings_tab: SettingsTab::Appearance,
            clock: chrono::Local::now(),
            panel_search: String::new(),
            session_menu_open: false,
            bell_open: false,
            manager_warned: false,
            update_run: None,
        }
    }

    /// A desktop for demos and screenshots: windows open, a notification shown.
    pub fn demo(config: DesktopConfig) -> Self {
        let mut d = Self::new("Giulia", "giulia", config);
        d.session.launch("files", "File");
        d.session.wm.open("store", "Store");
        d.session.launch("terminal", "Terminale");
        d.session.wm.open("settings", "Impostazioni");
        d.notifications.push(Notification::new(
            "LibonOS è aggiornato",
            "Versione 1.0.0 · canale stable",
            NotifyKind::Success,
        ));
        d.notifications.push(Notification::new(
            "Aggiornamento di sistema disponibile",
            "LibonOS 2.0.0 è pronto in Impostazioni › Aggiornamenti",
            NotifyKind::Info,
        ));
        d
    }

    pub fn notify(&mut self, title: impl Into<String>, body: impl Into<String>, kind: NotifyKind) {
        self.notifications
            .push(Notification::new(title, body, kind));
        if self.notifications.len() > 4 {
            self.notifications.remove(0);
        }
    }

    pub fn tick(&mut self) {
        self.clock = chrono::Local::now();
        self.notifications.retain(|n| n.age_secs() < 8.0);
        self.check_manager_availability();
    }

    /// Warn once when the configured package backend is missing, so a fresh
    /// LibonOS install on a machine without APT or pacman explains itself
    /// instead of failing silently at install time.
    fn check_manager_availability(&mut self) {
        if self.manager_warned {
            return;
        }
        let kind = self.config.package_manager;
        if !manager_for(kind).is_available() {
            self.manager_warned = true;
            self.notify(
                format!("{} non è disponibile", kind.label()),
                "Lo Store userà libon, che è preinstallato.",
                NotifyKind::Warning,
            );
        }
    }

    pub fn current_version(&self) -> Version {
        system::os_version()
    }

    /// Recompute the update status after a channel change.
    pub fn refresh_updates(&mut self) {
        let current = self.current_version();
        let channel = if self.config.beta_channel {
            channels::bundled_beta()
        } else {
            channels::bundled_stable()
        };
        self.update = UpdateStatus::check(&current, &channel);
        self.major_update = UpdateStatus::check_major_upgrade(&current, &channel);
    }

    /// Switch the configured package manager and rebuild the Store around it.
    pub fn set_manager(&mut self, kind: PackageManagerKind) {
        self.config.package_manager = kind;
        self.manager = Arc::<dyn PackageManager>::from(manager_for(kind));
        self.store = StoreState::new(self.manager.clone());
        self.store.load_featured();
    }

    pub fn wallpaper(&self) -> WallpaperStyle {
        self.config.wallpaper
    }

    /// Persist the desktop configuration.
    pub fn save_config(&self) {
        let _ = self.config.save(&self.config_path);
    }
}
