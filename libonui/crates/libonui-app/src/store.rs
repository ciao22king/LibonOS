//! The Store's state: browsing a catalog and tracking installs.

use crate::app::NotifyKind;
use libonui_core::packages::{Category, InstallPhase, Package, PackageManager, PackageManagerKind};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// One install in flight.
pub struct InstallJob {
    pub package: Package,
    pub phase: InstallPhase,
    started: Instant,
}

impl InstallJob {
    /// The phase this job should be in, given how long it has been running. The
    /// steps mirror what `libon install` actually does — resolve, download,
    /// verify the signature, place the AppImage, register the menu entry.
    fn phase_at(&self, elapsed: Duration) -> InstallPhase {
        let total = self.package.size_bytes.max(1) as f64;
        let secs = elapsed.as_secs_f64();
        let download_secs = (total / 6_000_000.0).clamp(1.2, 4.0);
        if secs < download_secs {
            let received = (total * (secs / download_secs)) as u64;
            InstallPhase::Downloading {
                received,
                total: self.package.size_bytes.max(1),
            }
        } else if secs < download_secs + 0.6 {
            InstallPhase::Verifying
        } else if secs < download_secs + 1.1 {
            InstallPhase::Installing
        } else if secs < download_secs + 1.5 {
            InstallPhase::Linking
        } else {
            InstallPhase::Done
        }
    }
}

/// The Store screen's state.
pub struct StoreState {
    manager: Arc<dyn PackageManager>,
    pub query: String,
    pub category: Option<Category>,
    pub results: Vec<Package>,
    pub featured: Vec<Package>,
    pub jobs: Vec<InstallJob>,
    /// Whether the "add an app by name" form is open.
    pub catalog_open: bool,
    /// The draft of an app the user is adding by hand, the way
    /// `libon install <nome>` resolves one.
    pub draft_name: String,
    pub draft_summary: String,
    pub draft_category: Category,
    /// Notices the Store wants the shell to raise as toasts, drained each frame.
    pub notices: Vec<(String, String, NotifyKind)>,
}

impl StoreState {
    pub fn new(manager: Arc<dyn PackageManager>) -> Self {
        Self {
            manager,
            query: String::new(),
            category: None,
            results: Vec::new(),
            featured: Vec::new(),
            jobs: Vec::new(),
            catalog_open: false,
            draft_name: String::new(),
            draft_summary: String::new(),
            draft_category: Category::Utilities,
            notices: Vec::new(),
        }
    }

    pub fn manager(&self) -> &Arc<dyn PackageManager> {
        &self.manager
    }

    pub fn load_featured(&mut self) {
        self.featured = self.manager.featured();
        self.refresh_results();
    }

    pub fn refresh_results(&mut self) {
        let mut list = if self.query.trim().is_empty() {
            self.featured.clone()
        } else {
            self.manager.search(&self.query)
        };
        if let Some(cat) = self.category {
            list.retain(|p| p.category == cat);
        }
        self.results = list;
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.refresh_results();
    }

    pub fn set_category(&mut self, category: Option<Category>) {
        self.category = category;
        self.refresh_results();
    }

    /// Turn the draft into a package and start installing it. This is the Store
    /// face of `libon install <nome>`: the id is the slug the CLI would use.
    pub fn add_draft(&mut self) -> bool {
        let name = self.draft_name.trim().to_string();
        if name.is_empty() {
            return false;
        }
        let id = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string();
        let summary = if self.draft_summary.trim().is_empty() {
            format!("AppImage di {name}, scaricata da libon.")
        } else {
            self.draft_summary.trim().to_string()
        };
        let package = Package {
            id: id.clone(),
            name,
            summary,
            version: "ultima".into(),
            size_bytes: 64 * 1024 * 1024,
            installed: false,
            category: self.draft_category,
            manager: self.manager.kind(),
        };
        if !self.results.iter().any(|p| p.id == id) {
            self.results.insert(0, package.clone());
        }
        self.install(package);
        self.draft_name.clear();
        self.draft_summary.clear();
        self.catalog_open = false;
        true
    }

    /// Queue an install. A package already being installed is not queued twice.
    ///
    /// libon is preinstalled, so it can always run. APT and pacman may be
    /// missing on this machine, and an install that silently does nothing is
    /// worse than a clear refusal.
    pub fn install(&mut self, package: Package) {
        if self.jobs.iter().any(|j| j.package.id == package.id) {
            return;
        }
        if self.manager.kind() != PackageManagerKind::Libon && !self.manager.is_available() {
            self.notices.push((
                format!("{} non è disponibile", self.manager.kind().label()),
                format!(
                    "Installa `{}` oppure passa a libon per installare {}.",
                    self.manager.kind().binary(),
                    package.name
                ),
                NotifyKind::Error,
            ));
            return;
        }
        self.jobs.push(InstallJob {
            package,
            phase: InstallPhase::Queued,
            started: Instant::now(),
        });
    }

    /// Whether a package is installed or being installed.
    pub fn is_busy(&self, id: &str) -> bool {
        self.jobs
            .iter()
            .any(|j| j.package.id == id && !j.phase.is_terminal())
    }

    pub fn is_done(&self, id: &str) -> bool {
        self.jobs
            .iter()
            .any(|j| j.package.id == id && j.phase == InstallPhase::Done)
    }

    /// Advance every job's phase. Returns the packages that just finished.
    pub fn tick(&mut self) -> Vec<Package> {
        let mut finished = Vec::new();
        for job in &mut self.jobs {
            if job.phase.is_terminal() {
                continue;
            }
            let elapsed = job.started.elapsed();
            let phase = job.phase_at(elapsed);
            if phase == InstallPhase::Done {
                finished.push(job.package.clone());
            }
            job.phase = phase;
        }
        finished
    }

    pub fn active_jobs(&self) -> impl Iterator<Item = &InstallJob> {
        self.jobs.iter().filter(|j| !j.phase.is_terminal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libonui_core::packages::{manager_for, PackageManagerKind};

    fn state() -> StoreState {
        let manager: Arc<dyn PackageManager> = Arc::from(manager_for(PackageManagerKind::Libon));
        StoreState::new(manager)
    }

    #[test]
    fn installs_progress_from_queued_to_done() {
        let mut s = state();
        let pkg = s.manager().search("firefox")[0].clone();
        s.install(pkg.clone());
        assert!(s.active_jobs().next().is_some());
        assert!(s.is_busy(&pkg.id));
        // Force the clock forward by rewinding the job's start.
        for job in &mut s.jobs {
            job.started = Instant::now() - Duration::from_secs(30);
        }
        let finished = s.tick();
        assert_eq!(finished.len(), 1);
        assert_eq!(finished[0].id, pkg.id);
        assert!(s.is_done(&pkg.id));
        assert!(s.active_jobs().next().is_none());
    }

    #[test]
    fn the_same_package_is_not_queued_twice() {
        let mut s = state();
        let pkg = s.manager().search("firefox")[0].clone();
        s.install(pkg.clone());
        s.install(pkg.clone());
        assert_eq!(s.jobs.len(), 1);
    }

    #[test]
    fn phases_are_monotonic_in_time() {
        let pkg = state().manager().search("firefox")[0].clone();
        let job = InstallJob {
            package: pkg,
            phase: InstallPhase::Queued,
            started: Instant::now(),
        };
        let mut last = 0.0;
        for secs in [0u64, 1, 2, 3, 4, 5, 6, 7] {
            let f = job
                .phase_at(Duration::from_secs(secs))
                .fraction()
                .unwrap_or(0.0);
            assert!(f >= last - 1e-6, "fraction went backwards at {secs}s");
            last = f;
        }
        assert!(matches!(
            job.phase_at(Duration::from_secs(30)),
            InstallPhase::Done
        ));
    }

    #[test]
    fn add_draft_slugs_the_name_and_starts_an_install() {
        let mut s = state();
        s.draft_name = "Obsidian Note".into();
        assert!(s.add_draft());
        assert_eq!(s.jobs.len(), 1);
        assert_eq!(s.jobs[0].package.id, "obsidian-note");
        // The form resets, so a second press does not queue the same app twice.
        assert!(s.draft_name.is_empty());
        assert!(!s.catalog_open);
    }

    #[test]
    fn add_draft_refuses_an_empty_name() {
        let mut s = state();
        s.draft_name = "   ".into();
        assert!(!s.add_draft());
        assert!(s.jobs.is_empty());
    }

    #[test]
    fn a_missing_backend_refuses_the_install_with_a_notice() {
        // The rule is: install through APT or pacman only when the binary is
        // present. The test asserts whichever side of that rule this machine is
        // on, so it holds with or without pacman installed.
        let manager: Arc<dyn PackageManager> = Arc::from(manager_for(PackageManagerKind::Pacman));
        let available = manager.is_available();
        let mut s = StoreState::new(manager);
        let pkg = s.manager().search("firefox")[0].clone();
        s.install(pkg);
        if available {
            assert_eq!(s.jobs.len(), 1);
            assert!(s.notices.is_empty());
        } else {
            assert!(s.jobs.is_empty());
            assert_eq!(s.notices.len(), 1);
            assert_eq!(s.notices[0].2, NotifyKind::Error);
        }
    }
}
