//! The update feed LibonOS ships with, and the network check that refreshes it.
//!
//! The parsing is split from the HTTP call so both halves are testable: a real
//! build fetches `updates.libonos.dev/<channel>.json`, while tests feed the same
//! parser a fixed document.

use crate::version::{ReleaseEntry, UpdateChannel, Version};

/// Parse an update feed document.
pub fn parse_channel(json: &str) -> Result<UpdateChannel, serde_json::Error> {
    serde_json::from_str(json)
}

/// The bundled stable feed. A live system replaces this after its first
/// successful network check, but a fresh install can always show *something*
/// honest rather than an error.
pub fn bundled_stable() -> UpdateChannel {
    UpdateChannel {
        name: "stable".into(),
        releases: vec![
            ReleaseEntry {
                version: Version::new(1, 0, 0),
                channel: "stable".into(),
                title: "LibonOS 1.0.0 \"Violet\"".into(),
                summary: "La prima versione stabile: shell LibonUI, Store e libon.".into(),
                published: "2026-10-01".into(),
                size_bytes: 0,
                mandatory: false,
            },
            ReleaseEntry {
                version: Version::new(1, 0, 1),
                channel: "stable".into(),
                title: "LibonOS 1.0.1".into(),
                summary: "Correzioni di sicurezza e miglioramenti al risparmio energetico.".into(),
                published: "2026-10-18".into(),
                size_bytes: 210_000_000,
                mandatory: true,
            },
            ReleaseEntry {
                version: Version::new(1, 1, 0),
                channel: "stable".into(),
                title: "LibonOS 1.1.0".into(),
                summary: "Night light, nuove scorciatoie e Store più veloce.".into(),
                published: "2026-11-05".into(),
                size_bytes: 480_000_000,
                mandatory: false,
            },
            ReleaseEntry {
                version: Version::new(2, 0, 0),
                channel: "stable".into(),
                title: "LibonOS 2.0.0 \"Aurora\"".into(),
                summary: "Nuovo compositore, cifratura del disco di serie e un nuovo \
                          pannello di controllo. Aggiornamento completo di sistema."
                    .into(),
                published: "2027-02-20".into(),
                size_bytes: 3_600_000_000,
                mandatory: false,
            },
        ],
    }
}

/// The beta feed, offered only when the user opts in from Settings.
pub fn bundled_beta() -> UpdateChannel {
    UpdateChannel {
        name: "beta".into(),
        releases: vec![ReleaseEntry {
            version: Version::new(1, 2, 0).with_pre("beta1"),
            channel: "beta".into(),
            title: "LibonOS 1.2.0 beta".into(),
            summary: "Anteprima del nuovo gestore delle finestre.".into(),
            published: "2026-11-20".into(),
            size_bytes: 520_000_000,
            mandatory: false,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::{UpdateKind, UpdateStatus};

    #[test]
    fn bundled_stable_contains_the_major_upgrade() {
        let channel = bundled_stable();
        let status = UpdateStatus::check(&Version::new(1, 0, 1), &channel);
        assert_eq!(status.kind, UpdateKind::InPlace);
        assert_eq!(status.target, Version::new(1, 1, 0));

        // The major upgrade is offered separately, and only from Settings.
        let major = UpdateStatus::check_major_upgrade(&Version::new(1, 1, 0), &channel).unwrap();
        assert_eq!(major.kind, UpdateKind::FullSystemUpgrade);
        assert_eq!(major.target, Version::new(2, 0, 0));
        assert!(major.kind.allowed_from_settings());
        assert!(!major.kind.allowed_from_terminal());
    }

    #[test]
    fn running_the_latest_reports_up_to_date() {
        let channel = bundled_stable();
        let status = UpdateStatus::check(&Version::new(1, 1, 0), &channel);
        assert_eq!(status.kind, UpdateKind::UpToDate);
    }

    #[test]
    fn parsing_a_feed_document_works() {
        let json = r#"{
            "name": "stable",
            "releases": [{
                "version": {"major": 1, "minor": 3, "patch": 0, "pre": null},
                "channel": "stable",
                "title": "1.3.0",
                "summary": "ok",
                "published": "2026-12-01",
                "size_bytes": 10,
                "mandatory": false
            }]
        }"#;
        let channel = parse_channel(json).unwrap();
        assert_eq!(channel.name, "stable");
        assert_eq!(channel.releases[0].version, Version::new(1, 3, 0));
    }

    #[test]
    fn malformed_feed_is_an_error_not_a_panic() {
        assert!(parse_channel("{ not json").is_err());
    }
}
