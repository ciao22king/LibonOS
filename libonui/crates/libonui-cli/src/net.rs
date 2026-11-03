//! Minimal HTTP support for the CLI.
//!
//! The download itself is delegated to `curl`, which every LibonOS image ships
//! with, so the binary stays small and free of a TLS stack. Keeping the network
//! surface this thin also means the interesting logic — resolving the app,
//! building the plan, classifying the update — is testable without a socket.

use std::path::Path;
use std::process::Command;

/// Download `url` to `dest`, creating the parent directory. Returns an error
/// message in the user's language on failure.
pub fn download(url: &str, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Impossibile creare {}: {e}", parent.display()))?;
    }
    let status = Command::new("curl")
        .args(["-fL", "--retry", "3", "--progress-bar", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .map_err(|e| format!("curl non è disponibile: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Download non riuscito da {url}"))
    }
}

/// Make a file executable.
pub fn make_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Ask the GitHub releases API for the newest AppImage asset of `repo`
/// (`owner/name`) and return its download URL.
///
/// GitHub's `releases/latest/download/<asset>` shortcut only works when the
/// asset name never changes; the publishers here version their file names, so
/// the API is the only reliable way to find the current one. Returns `None`
/// when offline, rate-limited, or when the release ships no AppImage.
pub fn github_appimage_asset(repo: &str) -> Option<String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "--max-time",
            "25",
            "-H",
            "Accept: application/vnd.github+json",
        ])
        .arg(&url)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let body = String::from_utf8(output.stdout).ok()?;
    pick_appimage_asset(&body)
}

/// Pick the best AppImage asset from a GitHub release JSON body. Pure, so the
/// selection rule is testable without a network call.
pub fn pick_appimage_asset(json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    let assets = value.get("assets")?.as_array()?;
    let mut fallback: Option<String> = None;
    for asset in assets {
        let name = asset.get("name")?.as_str()?;
        if !name.ends_with(".AppImage") {
            continue;
        }
        let url = asset.get("browser_download_url")?.as_str()?.to_string();
        // Prefer an x86_64 build, and never an arm64 one on an x86_64 host.
        let lower = name.to_lowercase();
        if lower.contains("arm64") || lower.contains("aarch64") {
            continue;
        }
        if lower.contains("x86_64") || lower.contains("amd64") {
            return Some(url);
        }
        fallback.get_or_insert(url);
    }
    fallback
}

/// Fetch the update feed. `None` when offline, so the caller can fall back to
/// the bundled feed rather than failing.
pub fn fetch_feed(url: &str) -> Option<String> {
    let output = Command::new("curl")
        .args(["-fsSL", "--max-time", "20"])
        .arg(url)
        .output()
        .ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_x86_64_appimage_over_arm64() {
        let json = r#"{
          "assets": [
            {"name":"Obsidian-1.15.0-arm64.AppImage","browser_download_url":"https://example/arm.AppImage"},
            {"name":"Obsidian-1.15.0.AppImage","browser_download_url":"https://example/x86.AppImage"},
            {"name":"obsidian-1.15.0.tar.gz","browser_download_url":"https://example/src.tar.gz"}
          ]
        }"#;
        assert_eq!(
            pick_appimage_asset(json).as_deref(),
            Some("https://example/x86.AppImage")
        );
    }

    #[test]
    fn falls_back_to_an_unlabelled_appimage() {
        let json = r#"{"assets":[
            {"name":"App-2.0.AppImage","browser_download_url":"https://example/app.AppImage"}
        ]}"#;
        assert_eq!(
            pick_appimage_asset(json).as_deref(),
            Some("https://example/app.AppImage")
        );
    }

    #[test]
    fn returns_none_when_the_release_has_no_appimage() {
        let json =
            r#"{"assets":[{"name":"app.deb","browser_download_url":"https://example/app.deb"}]}"#;
        assert_eq!(pick_appimage_asset(json), None);
        assert_eq!(pick_appimage_asset("not json"), None);
        assert_eq!(pick_appimage_asset("{}"), None);
    }
}
