//! Resolving an app name to a concrete AppImage install.
//!
//! `libon install <app>` needs three things before it can touch the network: a
//! download URL, a destination in the user's home, and a `.desktop` entry so the
//! app appears in the menu. All three are computed here as a pure `AppImagePlan`,
//! which keeps the network call in the CLI and the decision-making testable.

use std::path::PathBuf;

/// A resolved AppImage install.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppImagePlan {
    pub app_id: String,
    pub display_name: String,
    pub url: String,
    /// Absolute destination, e.g. `/home/ada/Applications/Firefox.AppImage`.
    pub dest: PathBuf,
    /// The `.desktop` entry written to `~/.local/share/applications`.
    pub desktop_path: PathBuf,
    pub desktop_entry: String,
}

/// A curated map of apps whose publishers ship an AppImage.
///
/// The third field is the GitHub repository to ask for the current asset, or an
/// empty string when the publisher hosts a stable direct URL. A repository is
/// preferred over a direct URL: these projects version their asset file names
/// (`Obsidian-1.15.0.AppImage`), so the `/releases/latest/download/<fixed-name>`
/// shortcut goes stale the moment the publisher bumps a version.
pub const APPIMAGE_SOURCES: &[AppImageSource] = &[
    AppImageSource {
        id: "obsidian",
        name: "Obsidian",
        repo: "obsidianmd/obsidian-releases",
        direct: "",
    },
    AppImageSource {
        id: "keepassxc",
        name: "KeePassXC",
        repo: "keepassxreboot/keepassxc",
        direct: "",
    },
    AppImageSource {
        id: "vscodium",
        name: "VSCodium",
        repo: "VSCodium/vscodium",
        direct: "",
    },
    AppImageSource {
        id: "localsend",
        name: "LocalSend",
        repo: "localsend/localsend",
        direct: "",
    },
    AppImageSource {
        id: "kdenlive",
        name: "Kdenlive",
        repo: "KDE/kdenlive",
        direct: "",
    },
    AppImageSource {
        id: "audacity",
        name: "Audacity",
        repo: "audacity/audacity",
        direct: "",
    },
    AppImageSource {
        id: "balena-etcher",
        name: "balenaEtcher",
        repo: "balena-io/etcher",
        direct: "",
    },
];

/// One curated AppImage publisher.
pub struct AppImageSource {
    pub id: &'static str,
    pub name: &'static str,
    /// `owner/name` for the GitHub releases API, or empty for a direct URL.
    pub repo: &'static str,
    /// A direct download URL used when `repo` is empty.
    pub direct: &'static str,
}

impl AppImageSource {
    pub fn by_id(app_id: &str) -> Option<&'static AppImageSource> {
        APPIMAGE_SOURCES
            .iter()
            .find(|s| s.id == app_id.to_lowercase())
    }

    /// The URL to download from when no API lookup is possible.
    pub fn fallback_url(&self) -> String {
        if !self.repo.is_empty() {
            format!("https://github.com/{}/releases/latest", self.repo)
        } else {
            self.direct.to_string()
        }
    }
}

/// The directory AppImages are installed into, under the user's home.
pub fn applications_dir(home: &std::path::Path) -> PathBuf {
    home.join("Applications")
}

/// Resolve an app to a plan rooted at `home`.
///
/// Returns `None` for an app that has no known AppImage source, so the CLI can
/// say so plainly rather than downloading the wrong thing.
pub fn resolve(app_id: &str, home: &std::path::Path) -> Option<AppImagePlan> {
    let source = AppImageSource::by_id(app_id)?;
    resolve_with_url(source, &source.fallback_url(), home)
}

/// Resolve a known source with an explicit asset URL, as discovered from the
/// publisher's release API.
pub fn resolve_with_url(
    source: &AppImageSource,
    url: &str,
    home: &std::path::Path,
) -> Option<AppImagePlan> {
    plan(source, url.to_string(), home)
}

fn plan(source: &AppImageSource, url: String, home: &std::path::Path) -> Option<AppImagePlan> {
    let id = source.id;
    let name = source.name;

    let file_name = format!("{}.AppImage", pascal_case(name));
    let dest = applications_dir(home).join(&file_name);
    let desktop_path = home
        .join(".local/share/applications")
        .join(format!("{id}.desktop"));
    let desktop_entry = desktop_entry(id, name, &dest);

    Some(AppImagePlan {
        app_id: id.to_string(),
        display_name: name.to_string(),
        url,
        dest,
        desktop_path,
        desktop_entry,
    })
}

/// Build the freedesktop `.desktop` entry for an installed AppImage.
pub fn desktop_entry(app_id: &str, name: &str, exec_path: &std::path::Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={name}\n\
         Comment=Installato con libon\n\
         Exec=\"{}\" %U\n\
         Terminal=false\n\
         Categories=Utility;\n\
         StartupWMClass={app_id}\n\
         X-Libon-AppImage=true\n",
        exec_path.display()
    )
}

fn pascal_case(s: &str) -> String {
    s.split([' ', '-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn resolves_a_known_app() {
        let home = Path::new("/home/ada");
        let plan = resolve("obsidian", home).unwrap();
        assert_eq!(plan.app_id, "obsidian");
        assert_eq!(plan.display_name, "Obsidian");
        assert!(plan.url.starts_with("https://"));
        assert_eq!(
            plan.dest,
            PathBuf::from("/home/ada/Applications/Obsidian.AppImage")
        );
        assert!(plan.desktop_entry.contains("Name=Obsidian"));
        assert!(plan.desktop_entry.contains("X-Libon-AppImage=true"));
        assert!(plan
            .desktop_path
            .to_string_lossy()
            .ends_with("applications/obsidian.desktop"));
    }

    #[test]
    fn unknown_app_resolves_to_nothing() {
        assert!(resolve("definitely-not-an-app", Path::new("/home/ada")).is_none());
    }

    #[test]
    fn lookup_is_case_insensitive() {
        assert!(resolve("KeePassXC", Path::new("/home/ada")).is_some());
    }

    #[test]
    fn pascal_case_handles_separators() {
        assert_eq!(pascal_case("balena-etcher"), "BalenaEtcher");
        assert_eq!(pascal_case("visual studio code"), "VisualStudioCode");
    }
}
