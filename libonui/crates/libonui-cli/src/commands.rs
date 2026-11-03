//! The `libon` command set.

use libonui_core::config::StatePaths;
use libonui_core::packages::appimage;
use libonui_core::packages::{manager_for, PackageManagerKind};
use libonui_core::version::{self, UpdateKind, UpdateStatus};
use libonui_core::{channels, system};
use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// Terminal escape helpers. Colour is only emitted on a TTY.
struct Term {
    color: bool,
}

impl Term {
    fn new() -> Self {
        Self {
            color: std::env::var("NO_COLOR").is_err() && is_tty(),
        }
    }
    fn bold(&self, s: &str) -> String {
        self.wrap("1", s)
    }
    fn violet(&self, s: &str) -> String {
        self.wrap("38;5;141", s)
    }
    fn cyan(&self, s: &str) -> String {
        self.wrap("38;5;80", s)
    }
    fn green(&self, s: &str) -> String {
        self.wrap("38;5;114", s)
    }
    fn yellow(&self, s: &str) -> String {
        self.wrap("38;5;221", s)
    }
    fn red(&self, s: &str) -> String {
        self.wrap("38;5;210", s)
    }
    fn dim(&self, s: &str) -> String {
        self.wrap("2", s)
    }
    fn wrap(&self, code: &str, s: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
}

fn is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

pub fn run(args: &[String]) -> Result<(), ExitCode> {
    let term = Term::new();
    let (cmd, rest) = match args.split_first() {
        Some((c, r)) => (c.as_str(), r),
        None => {
            print_help(&term);
            return Ok(());
        }
    };

    match cmd {
        "-h" | "--help" | "help" => {
            print_help(&term);
            Ok(())
        }
        "-V" | "--version" | "version" => {
            println!("{}", libonui_core::banner());
            println!(
                "{}",
                term.dim(&format!(
                    "gestore pacchetti: {}",
                    system::NATIVE_MANAGER.label()
                ))
            );
            Ok(())
        }
        "install" | "i" => install(&term, rest),
        "remove" | "rm" | "uninstall" => remove(&term, rest),
        "search" | "s" => search(&term, rest),
        "list" | "ls" => list(&term),
        "update" => update(&term),
        "upgrade" => upgrade(&term),
        "status" => status(&term),
        "info" => info(&term, rest),
        other => {
            eprintln!("{} comando sconosciuto: {other}", term.red("errore:"));
            eprintln!(
                "{}",
                term.dim("Esegui `libon help` per l'elenco dei comandi.")
            );
            Err(ExitCode::from(2))
        }
    }
}

fn configured_manager() -> PackageManagerKind {
    let path = StatePaths::user().config_file();
    if path.exists() {
        libonui_core::DesktopConfig::load_or_default(&path).package_manager
    } else {
        system::NATIVE_MANAGER
    }
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn install(term: &Term, args: &[String]) -> Result<(), ExitCode> {
    let Some(app) = args.first() else {
        eprintln!(
            "{} specifica un'app: `libon install <app>`",
            term.red("errore:")
        );
        return Err(ExitCode::from(2));
    };

    println!(
        "{} {}",
        term.bold("libon"),
        term.dim(&format!("· install {app}"))
    );

    // libon's own path: resolve the AppImage online and run it.
    match resolve_appimage(app) {
        Some(plan) => {
            println!(
                "  {} {} {}",
                term.violet("app"),
                plan.display_name,
                term.dim(&plan.url)
            );
            println!("  {} {}", term.violet("destinazione"), plan.dest.display());
            if let Err(e) = crate::net::download(&plan.url, &plan.dest) {
                eprintln!("{} {e}", term.red("errore:"));
                return Err(ExitCode::from(1));
            }
            crate::net::make_executable(&plan.dest).map_err(|e| {
                eprintln!("{} {e}", term.red("errore:"));
                ExitCode::from(1)
            })?;
            if let Some(parent) = plan.desktop_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&plan.desktop_path, plan.desktop_entry).map_err(|e| {
                eprintln!("{} {e}", term.red("errore:"));
                ExitCode::from(1)
            })?;
            println!(
                "  {} {}",
                term.green("pronto"),
                term.dim("apparsa nel menu applicazioni")
            );
            Ok(())
        }
        None => {
            // Not an AppImage app: fall back to the configured native manager.
            let kind = configured_manager();
            let manager = manager_for(kind);
            if manager.search(app).is_empty() && kind != PackageManagerKind::Libon {
                eprintln!(
                    "{} nessun pacchetto chiamato «{app}» in {}",
                    term.yellow("attenzione:"),
                    kind.label()
                );
                return Err(ExitCode::from(1));
            }
            let argv = manager.install_args(app);
            run_privileged(term, kind.requires_admin(), &argv)
        }
    }
}

/// Resolve an app to an AppImage install, asking the publisher's GitHub release
/// for the current asset URL. Falls back to the offline plan (which points at
/// the release page) when the API cannot be reached, so the printed URL is
/// always real rather than a guessed file name.
fn resolve_appimage(app: &str) -> Option<appimage::AppImagePlan> {
    let home = home_dir();
    let source = appimage::AppImageSource::by_id(app)?;
    if source.repo.is_empty() {
        return appimage::resolve(app, &home);
    }
    match crate::net::github_appimage_asset(source.repo) {
        Some(url) => appimage::resolve_with_url(source, &url, &home),
        None => appimage::resolve(app, &home),
    }
}

fn remove(term: &Term, args: &[String]) -> Result<(), ExitCode> {
    let Some(app) = args.first() else {
        eprintln!(
            "{} specifica un'app: `libon remove <app>`",
            term.red("errore:")
        );
        return Err(ExitCode::from(2));
    };
    if let Some(plan) = appimage::resolve(app, &home_dir()) {
        println!(
            "{} {}",
            term.bold("libon"),
            term.dim(&format!("· remove {app}"))
        );
        let _ = std::fs::remove_file(&plan.dest);
        let _ = std::fs::remove_file(&plan.desktop_path);
        println!("  {} {}", term.green("rimosso"), plan.display_name);
        return Ok(());
    }
    let kind = configured_manager();
    let manager = manager_for(kind);
    let argv = manager.remove_args(app);
    run_privileged(term, kind.requires_admin(), &argv)
}

fn search(term: &Term, args: &[String]) -> Result<(), ExitCode> {
    let query = args.join(" ");
    let kind = configured_manager();
    let manager = manager_for(kind);
    let results = manager.search(&query);
    if results.is_empty() {
        println!("{}", term.dim(&format!("Nessun risultato per «{query}».")));
        return Ok(());
    }
    println!(
        "{} {}",
        term.bold(&format!("{} risultati", results.len())),
        term.dim(&format!("· {}", kind.label()))
    );
    for p in results {
        println!(
            "  {} {}  {}",
            term.violet(&p.id),
            term.dim(&format!("({})", version::human_size(p.size_bytes))),
            p.summary
        );
    }
    Ok(())
}

fn list(term: &Term) -> Result<(), ExitCode> {
    let dir = appimage::applications_dir(&home_dir());
    println!("{}", term.bold("AppImage installate con libon"));
    match std::fs::read_dir(&dir) {
        Ok(entries) => {
            let mut any = false;
            for e in entries.flatten() {
                if e.path().extension().is_some_and(|x| x == "AppImage") {
                    println!("  {}", e.file_name().to_string_lossy());
                    any = true;
                }
            }
            if !any {
                println!("  {}", term.dim("nessuna"));
            }
        }
        Err(_) => println!("  {}", term.dim("nessuna")),
    }
    println!();
    println!("{}", term.bold("Applicazioni preinstallate con LibonOS"));
    for app in system::PREINSTALLED {
        println!("  {} {}", term.cyan(app.id), term.dim(app.summary));
    }
    Ok(())
}

fn info(term: &Term, args: &[String]) -> Result<(), ExitCode> {
    let Some(app) = args.first() else {
        eprintln!(
            "{} specifica un'app: `libon info <app>`",
            term.red("errore:")
        );
        return Err(ExitCode::from(2));
    };
    let kind = configured_manager();
    let manager = manager_for(kind);
    let results = manager.search(app);
    if let Some(p) = results.iter().find(|p| p.id == *app).or(results.first()) {
        println!("{}", term.bold(&p.name));
        println!("  {} {}", term.violet("id"), p.id);
        println!("  {} {}", term.violet("categoria"), p.category.label());
        println!(
            "  {} {}",
            term.violet("dimensione"),
            version::human_size(p.size_bytes)
        );
        println!("  {} {}", term.violet("gestore"), p.manager.label());
        println!("  {}", p.summary);
        Ok(())
    } else {
        eprintln!("{} app non trovata: {app}", term.red("errore:"));
        Err(ExitCode::from(1))
    }
}

fn update(term: &Term) -> Result<(), ExitCode> {
    let current = system::os_version();
    let channel = load_channel();
    let status = UpdateStatus::check(&current, &channel);

    println!("{} {}", term.bold("libon"), term.dim("· update"));
    println!("  {} {}", term.violet("versione attuale"), current);

    match status.kind {
        UpdateKind::UpToDate => {
            println!(
                "  {} {}",
                term.green("aggiornato"),
                term.dim(&status.summary)
            );
        }
        UpdateKind::InPlace => {
            println!("  {} {}", term.violet("disponibile"), status.target);
            println!(
                "  {} {}",
                term.dim("dimensione"),
                version::human_size(status.size_bytes)
            );
            println!("  {}", status.summary);
            // The everyday update: refresh the index and apply the in-place upgrade.
            let kind = configured_manager();
            let manager = manager_for(kind);
            run_privileged(term, kind.requires_admin(), &manager.refresh_args())?;
            println!(
                "  {} {}",
                term.green("OS aggiornato"),
                term.dim(&format!("→ {}", status.target))
            );
        }
        UpdateKind::Downgrade => {
            println!(
                "  {}",
                term.yellow("La versione in esecuzione è più recente del canale.")
            );
        }
        UpdateKind::FullSystemUpgrade => {
            // Should not happen: `check` never returns a major bump. Guard anyway.
            println!(
                "{} è disponibile un aggiornamento di sistema completo da Impostazioni.",
                term.yellow("nota:")
            );
        }
    }

    if let Some(major) = UpdateStatus::check_major_upgrade(&current, &channel) {
        println!();
        println!(
            "  {} {} → {}",
            term.cyan("aggiornamento di sistema completo"),
            current,
            major.target
        );
        println!(
            "  {}",
            term.dim("Va fatto da Impostazioni › Aggiornamenti. `libon update` non lo applica.")
        );
    }
    Ok(())
}

fn upgrade(term: &Term) -> Result<(), ExitCode> {
    let current = system::os_version();
    let channel = load_channel();
    println!("{} {}", term.bold("libon"), term.dim("· upgrade"));
    match UpdateStatus::check_major_upgrade(&current, &channel) {
        Some(major) => {
            println!(
                "  {} {} → {}",
                term.cyan("aggiornamento completo disponibile"),
                current,
                major.target
            );
            println!("  {}", major.summary);
            println!(
                "  {} {}",
                term.dim("dimensione"),
                version::human_size(major.size_bytes)
            );
            println!();
            println!(
                "  {}",
                term.yellow("Un aggiornamento di sistema completo si applica solo da Impostazioni › Aggiornamenti.")
            );
            println!(
                "  {}",
                term.dim("Da terminale usa `libon update` per gli aggiornamenti in-place.")
            );
            Ok(())
        }
        None => {
            println!(
                "  {}",
                term.dim("Nessun aggiornamento di sistema completo disponibile.")
            );
            Ok(())
        }
    }
}

fn status(term: &Term) -> Result<(), ExitCode> {
    let current = system::os_version();
    let channel = load_channel();
    let status = UpdateStatus::check(&current, &channel);
    let (name, release) = system::os_identity();
    println!("{}", term.bold(&format!("{name} {release}")));
    println!("  {} {}", term.violet("canale"), channel.name);
    println!(
        "  {} {}",
        term.violet("stato"),
        match status.kind {
            UpdateKind::UpToDate => term.green("aggiornato"),
            UpdateKind::InPlace =>
                term.yellow(&format!("aggiornamento {} disponibile", status.target)),
            UpdateKind::FullSystemUpgrade => term.cyan("aggiornamento completo disponibile"),
            UpdateKind::Downgrade => term.dim("versione più recente del canale"),
        }
    );
    println!(
        "  {} {}",
        term.violet("gestore pacchetti"),
        configured_manager().label()
    );
    if let Some(major) = UpdateStatus::check_major_upgrade(&current, &channel) {
        println!(
            "  {} {}",
            term.violet("prossima major"),
            term.dim(&format!("{} (da Impostazioni)", major.target))
        );
    }
    Ok(())
}

/// Load the update feed: try the network, fall back to the bundled one.
fn load_channel() -> libonui_core::version::UpdateChannel {
    let beta = StatePaths::user().config_file().exists()
        && libonui_core::DesktopConfig::load_or_default(&StatePaths::user().config_file())
            .beta_channel;
    let channel_name = if beta { "beta" } else { "stable" };
    let url = format!("https://updates.libonos.dev/{channel_name}.json");
    crate::net::fetch_feed(&url)
        .and_then(|body| channels::parse_channel(&body).ok())
        .unwrap_or_else(|| {
            if beta {
                channels::bundled_beta()
            } else {
                channels::bundled_stable()
            }
        })
}

/// Run a package-manager command, prefixing with `sudo` when the backend needs
/// root.
fn run_privileged(term: &Term, needs_admin: bool, argv: &[String]) -> Result<(), ExitCode> {
    let (program, args) = match (needs_admin, argv.split_first()) {
        (true, Some((p, rest))) => (
            "sudo".to_string(),
            std::iter::once(p.clone())
                .chain(rest.iter().cloned())
                .collect::<Vec<_>>(),
        ),
        (false, Some((p, rest))) => (p.clone(), rest.to_vec()),
        (_, None) => return Ok(()),
    };
    let shown = std::iter::once(program.clone())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ");
    println!("  {} {}", term.dim("eseguo"), term.dim(&shown));
    let status = Command::new(&program).args(&args).status().map_err(|e| {
        eprintln!(
            "{} impossibile eseguire {program}: {e}",
            term.red("errore:")
        );
        ExitCode::from(1)
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(ExitCode::from(1))
    }
}

fn print_help(term: &Term) {
    println!(
        "{}",
        term.bold(&format!("{} · libon", libonui_core::banner()))
    );
    println!(
        "{}",
        term.dim("Il gestore pacchetti di LibonOS: cerca l'app, scarica l'AppImage, la esegue.")
    );
    println!();
    println!("{}", term.bold("USO"));
    println!("  libon <comando> [argomenti]");
    println!();
    println!("{}", term.bold("COMANDI"));
    let rows = [
        (
            "install <app>",
            "cerca l'app, scarica l'AppImage e la registra nel menu",
        ),
        ("remove <app>", "disinstalla un'app"),
        ("search <testo>", "cerca nel catalogo"),
        ("info <app>", "mostra i dettagli di un'app"),
        ("list", "elenca app installate e preinstallate"),
        ("update", "aggiornamento in-place di OS e applicazioni"),
        (
            "upgrade",
            "informazioni sull'aggiornamento completo di sistema",
        ),
        ("status", "versione, canale e stato degli aggiornamenti"),
        ("version", "mostra la versione"),
        ("help", "mostra questo aiuto"),
    ];
    for (c, d) in rows {
        println!("  {}  {}", term.violet(&format!("{c:<18}")), d);
    }
    println!();
    println!(
        "  {}",
        term.dim(
            "Un aggiornamento di sistema completo (es. 1.0.0 → 2.0.0) si applica da Impostazioni."
        )
    );
}
