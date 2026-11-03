//! Headless screenshot rendering, used to produce the images in the README and
//! to check the real rendered result rather than trusting the code.
//!
//! It drives the ordinary `App` through a script of states and, for each one,
//! asks the renderer for a screenshot of the framebuffer. Run it under a virtual
//! display:
//!
//! ```text
//! xvfb-run -s "-screen 0 1440x900x24" libonui --shots artifacts
//! ```

use crate::app::{App, Desktop, LaunchMode, LoginState, SettingsTab, Stage};
use libonui_core::config::StatePaths;
use libonui_core::onboarding::{SetupAnswers, SetupState, SetupStep};
use libonui_core::tokens::ThemeChoice;
use std::path::{Path, PathBuf};

/// One frame in the screenshot script.
#[derive(Clone)]
struct Shot {
    name: &'static str,
    kind: ShotKind,
}

#[derive(Clone)]
enum ShotKind {
    Setup(SetupStep, bool),
    Login,
    Desktop {
        app: &'static str,
        title: &'static str,
        tab: Option<SettingsTab>,
        theme: ThemeChoice,
        /// Show the application menu.
        menu: bool,
        /// Show an OS update running: (major, seconds already elapsed).
        updating: Option<(bool, f32)>,
        /// Show an install in progress in the Store.
        installing: bool,
        /// Show the notification centre open.
        bell: bool,
        /// Show the Store's "add an app by name" form.
        catalog: bool,
    },
}

#[allow(clippy::too_many_arguments)]
const fn desk(
    name: &'static str,
    app: &'static str,
    title: &'static str,
    tab: Option<SettingsTab>,
    theme: ThemeChoice,
) -> Shot {
    Shot {
        name,
        kind: ShotKind::Desktop {
            app,
            title,
            tab,
            theme,
            menu: false,
            updating: None,
            installing: false,
            bell: false,
            catalog: false,
        },
    }
}

fn script() -> Vec<Shot> {
    vec![
        Shot {
            name: "01-setup-welcome",
            kind: ShotKind::Setup(SetupStep::Welcome, false),
        },
        Shot {
            name: "02-setup-name",
            kind: ShotKind::Setup(SetupStep::Name, true),
        },
        Shot {
            name: "03-setup-password",
            kind: ShotKind::Setup(SetupStep::Password, true),
        },
        Shot {
            name: "04-setup-packages",
            kind: ShotKind::Setup(SetupStep::PackageManager, true),
        },
        Shot {
            name: "05-setup-privacy",
            kind: ShotKind::Setup(SetupStep::Privacy, true),
        },
        Shot {
            name: "06-setup-finish",
            kind: ShotKind::Setup(SetupStep::Finish, true),
        },
        Shot {
            name: "07-login",
            kind: ShotKind::Login,
        },
        Shot {
            name: "08-desktop",
            kind: ShotKind::Desktop {
                app: "files",
                title: "File",
                tab: None,
                theme: ThemeChoice::Dark,
                menu: false,
                updating: None,
                installing: false,
                bell: false,
                catalog: false,
            },
        },
        desk(
            "09-settings-updates",
            "settings",
            "Impostazioni",
            Some(SettingsTab::Updates),
            ThemeChoice::Dark,
        ),
        Shot {
            name: "10-store",
            kind: ShotKind::Desktop {
                app: "store",
                title: "Store",
                tab: None,
                theme: ThemeChoice::Dark,
                menu: false,
                updating: None,
                installing: false,
                bell: false,
                catalog: false,
            },
        },
        Shot {
            name: "11-terminal",
            kind: ShotKind::Desktop {
                app: "terminal",
                title: "Terminale",
                tab: None,
                theme: ThemeChoice::Dark,
                menu: false,
                updating: None,
                installing: false,
                bell: false,
                catalog: false,
            },
        },
        desk(
            "12-settings-packages",
            "settings",
            "Impostazioni",
            Some(SettingsTab::Packages),
            ThemeChoice::Dark,
        ),
        Shot {
            name: "13-desktop-light",
            kind: ShotKind::Desktop {
                app: "files",
                title: "File",
                tab: None,
                theme: ThemeChoice::Light,
                menu: false,
                updating: None,
                installing: false,
                bell: false,
                catalog: false,
            },
        },
        Shot {
            name: "14-major-upgrade-running",
            kind: ShotKind::Desktop {
                app: "settings",
                title: "Impostazioni",
                tab: Some(SettingsTab::Updates),
                theme: ThemeChoice::Dark,
                menu: false,
                updating: Some((true, 4.6)),
                installing: false,
                bell: false,
                catalog: false,
            },
        },
        Shot {
            name: "15-store-installing",
            kind: ShotKind::Desktop {
                app: "store",
                title: "Store",
                tab: None,
                theme: ThemeChoice::Dark,
                menu: false,
                updating: None,
                installing: true,
                bell: false,
                catalog: false,
            },
        },
        Shot {
            name: "16-start-menu",
            kind: ShotKind::Desktop {
                app: "files",
                title: "File",
                tab: None,
                theme: ThemeChoice::Dark,
                menu: true,
                updating: None,
                installing: false,
                bell: false,
                catalog: false,
            },
        },
        Shot {
            name: "17-settings-appearance",
            kind: ShotKind::Desktop {
                app: "settings",
                title: "Impostazioni",
                tab: Some(SettingsTab::Appearance),
                theme: ThemeChoice::Dark,
                menu: false,
                updating: None,
                installing: false,
                bell: false,
                catalog: false,
            },
        },
        Shot {
            name: "18-notification-centre",
            kind: ShotKind::Desktop {
                app: "files",
                title: "File",
                tab: None,
                theme: ThemeChoice::Dark,
                menu: false,
                updating: None,
                installing: false,
                bell: true,
                catalog: false,
            },
        },
        Shot {
            name: "19-store-add-app",
            kind: ShotKind::Desktop {
                app: "store",
                title: "Store",
                tab: None,
                theme: ThemeChoice::Dark,
                menu: false,
                updating: None,
                installing: false,
                bell: false,
                catalog: true,
            },
        },
    ]
}

/// The live script's progress.
#[derive(Clone)]
pub struct ShotScript {
    shots: Vec<Shot>,
    index: usize,
    out: PathBuf,
    requested: bool,
    settle: u32,
}

impl ShotScript {
    pub fn new(out: PathBuf) -> Self {
        Self {
            shots: script(),
            index: 0,
            out,
            requested: false,
            settle: 0,
        }
    }
}

/// Configure `app` for shot `index`.
fn configure(app: &mut App, index: usize) {
    let shots = script();
    let Some(shot) = shots.get(index) else {
        return;
    };
    match &shot.kind {
        ShotKind::Setup(step, filled) => {
            let mut state = SetupState {
                step: *step,
                ..SetupState::default()
            };
            if *filled {
                state.answers = SetupAnswers {
                    first_name: "Giulia".into(),
                    password: "libon-2026-os".into(),
                    confirm: "libon-2026-os".into(),
                    encrypt_home: true,
                    package_manager: libonui_core::packages::PackageManagerKind::Libon,
                    telemetry_enabled: false,
                    automatic_updates: true,
                };
            }
            app.stage = Stage::Setup(state);
        }
        ShotKind::Login => {
            app.stage = Stage::Login(LoginState {
                password: String::new(),
                error: None,
                first_name: "Giulia".into(),
                attempts: 0,
            });
        }
        ShotKind::Desktop {
            app: app_id,
            title,
            tab,
            theme,
            menu,
            updating,
            installing,
            bell,
            catalog,
        } => {
            let config = libonui_core::DesktopConfig {
                theme: *theme,
                package_manager: libonui_core::packages::PackageManagerKind::Libon,
                ..libonui_core::DesktopConfig::default()
            };
            let mut desktop = Desktop::new("Giulia", "giulia", config);
            if let Some(tab) = tab {
                desktop.settings_tab = *tab;
            }
            desktop.session.launch(app_id, title);
            // A second, minimised window so the dock shows an activity dot.
            if *app_id != "files" {
                let other = desktop.session.wm.open("files", "File");
                desktop.session.wm.minimize(other);
                let target = desktop.session.wm.existing_window_for(app_id).unwrap();
                desktop.session.wm.focus(target);
            }
            if *menu {
                desktop.session.start_menu_open = true;
            }
            if let Some((major, elapsed)) = updating {
                desktop.update_run = Some(crate::app::UpdateRun {
                    started: std::time::Instant::now()
                        - std::time::Duration::from_secs_f32(*elapsed),
                    major: *major,
                    target: libonui_core::version::Version::parse(if *major {
                        "2.0.0"
                    } else {
                        "1.1.0"
                    })
                    .unwrap(),
                });
            }
            if *installing {
                if let Some(pkg) = desktop
                    .store
                    .results
                    .iter()
                    .find(|p| p.id == "firefox")
                    .cloned()
                {
                    desktop.store.install(pkg);
                }
            }
            if *catalog {
                desktop.store.catalog_open = true;
                desktop.store.draft_name = "Obsidian".into();
                desktop.store.draft_summary = "Note e grafo di conoscenza, in locale.".into();
                desktop.store.draft_category = libonui_core::packages::Category::Utilities;
            }
            if *bell {
                desktop.bell_open = true;
                desktop.notifications.push(crate::app::Notification::new(
                    "Aggiornamento disponibile",
                    "LibonOS 1.1.0 è pronto: `libon update`.",
                    crate::app::NotifyKind::Info,
                ));
                desktop.notifications.push(crate::app::Notification::new(
                    "APT non è disponibile",
                    "Lo Store userà libon, che è preinstallato.",
                    crate::app::NotifyKind::Warning,
                ));
            }
            desktop.notifications.push(crate::app::Notification::new(
                "LibonOS è aggiornato",
                "Versione 1.0.0 · canale stable",
                crate::app::NotifyKind::Success,
            ));
            app.stage = Stage::Desktop(Box::new(desktop));
        }
    }
    app.refresh_theme();
}

/// Advance the script. Called once per frame from the app's update.
pub fn tick(app: &mut App, ctx: &egui::Context) {
    let Some(mut script) = app.script.take() else {
        return;
    };

    if script.index >= script.shots.len() {
        // The contact sheet is written from the same list that produced the
        // PNGs, so a new shot can never be missing from the index.
        let _ = write_index(&script);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        app.script = Some(script);
        return;
    }

    let name = script.shots[script.index].name;

    // Save whatever screenshot arrived this frame.
    let events = ctx.input(|i| i.events.clone());
    for event in events {
        if let egui::Event::Screenshot { image, .. } = event {
            let path = script.out.join(format!("{name}.png"));
            match save_png(&image, &path) {
                Ok(()) => println!("salvato {}", path.display()),
                Err(e) => eprintln!("screenshot {} non salvato: {e}", path.display()),
            }
            script.index += 1;
            script.requested = false;
            script.settle = 0;
            let next = script.index;
            app.script = Some(script);
            configure(app, next);
            crate::theme::install_style(ctx, &app.theme);
            return;
        }
    }

    // Let the layout settle before capturing.
    script.settle += 1;
    if !script.requested && script.settle > 5 {
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
        script.requested = true;
    }
    ctx.request_repaint();
    app.script = Some(script);
}

/// Write the contact sheet that lists every rendered screen, with the shot's
/// own name as the caption and the theme it was rendered in.
fn write_index(script: &ShotScript) -> std::io::Result<()> {
    let mut html = String::from(
        "<!doctype html><html lang=\"it\"><head><meta charset=\"utf-8\">\
         <title>LibonUI — schermate</title><style>\
         :root{color-scheme:dark}body{background:#0D0B16;color:#F3F1FA;\
         font-family:system-ui,sans-serif;margin:0;padding:32px}\
         h1{font-size:20px;font-weight:600;margin:0 0 4px}\
         p{color:#8B84A8;font-size:13px;margin:0 0 28px}\
         .grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(420px,1fr));gap:24px}\
         figure{margin:0}img{width:100%;display:block;border:1px solid #332E4A;border-radius:12px;\
         box-shadow:0 10px 24px rgba(0,0,0,.45)}\
         figcaption{font-size:12px;color:#BDB7D6;padding-top:8px}\
         </style></head><body><h1>LibonUI — schermate</h1>\
         <p>Le schermate generate con <code>libonui --shots</code>. Tema scuro e chiaro.</p>\
         <div class=\"grid\">\n",
    );
    for shot in &script.shots {
        html.push_str(&format!(
            "<figure><img src=\"{name}.png\" alt=\"{name}\"><figcaption>{name}</figcaption></figure>\n",
            name = shot.name
        ));
    }
    html.push_str("</div></body></html>\n");
    std::fs::write(script.out.join("index.html"), html)
}

fn save_png(image: &egui::ColorImage, path: &Path) -> std::io::Result<()> {
    let w = image.size[0] as u32;
    let h = image.size[1] as u32;
    let mut buf = Vec::with_capacity((w * h * 4) as usize);
    for px in &image.pixels {
        buf.extend_from_slice(&[px.r(), px.g(), px.b(), px.a()]);
    }
    image::save_buffer(path, &buf, w, h, image::ColorType::Rgba8).map_err(std::io::Error::other)
}

/// Run the shell in screenshot mode.
pub fn run(
    out_dir: &str,
    paths: StatePaths,
    native_options: eframe::NativeOptions,
) -> eframe::Result<()> {
    let out = PathBuf::from(out_dir);
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("impossibile creare {}: {e}", out.display());
    }
    eframe::run_native(
        "LibonOS shots",
        native_options,
        Box::new(move |cc| {
            let mut app = App::new(paths.clone(), LaunchMode::Demo);
            configure(&mut app, 0);
            crate::theme::install_style(&cc.egui_ctx, &app.theme);
            app.install_fonts(&cc.egui_ctx);
            app.script = Some(ShotScript::new(out.clone()));
            Ok(Box::new(app))
        }),
    )
}
