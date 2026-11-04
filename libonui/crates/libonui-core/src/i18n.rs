//! Interface languages.
//!
//! Every string the first-run flow and the settings surface put on screen lives
//! here, keyed by a stable [`Key`] rather than by the English sentence. A key
//! never changes because a translation did, and adding a language is a new
//! `TABLES` entry and nothing else.
//!
//! The module carries no UI dependency: `libonui-core` decides *which* string a
//! locale shows, and `libonui-app` and `libon` only draw it. That is what lets
//! every table be checked by an ordinary unit test — completeness, no untranslated
//! leftovers, and a placeholder that survives translation.

use std::fmt;

mod table;

pub use table::TABLES;

/// A language the interface can speak.
///
/// Codes follow BCP 47, so a future `zh-Hant` is a sibling of `zh-Hans` rather
/// than a rename.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Locale {
    En,
    It,
    Es,
    De,
    Fr,
    Ru,
    ZhHans,
}

impl Locale {
    /// Every locale, in the order the setup picker shows them: the two the
    /// project is written in first, then the rest ordered by endonym.
    pub const ALL: [Locale; 7] = [
        Locale::En,
        Locale::It,
        Locale::De,
        Locale::Es,
        Locale::Fr,
        Locale::Ru,
        Locale::ZhHans,
    ];

    /// The BCP 47 code, as stored in settings and as used to name a table.
    pub const fn code(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::It => "it",
            Locale::Es => "es",
            Locale::De => "de",
            Locale::Fr => "fr",
            Locale::Ru => "ru",
            Locale::ZhHans => "zh-Hans",
        }
    }

    /// The language's own name, shown in the picker so a speaker recognises it
    /// without reading the current interface language.
    pub const fn endonym(self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::It => "Italiano",
            Locale::Es => "Español",
            Locale::De => "Deutsch",
            Locale::Fr => "Français",
            Locale::Ru => "Русский",
            Locale::ZhHans => "简体中文",
        }
    }

    /// The English name, for logs and for `libon --help`.
    pub const fn english_name(self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::It => "Italian",
            Locale::Es => "Spanish",
            Locale::De => "German",
            Locale::Fr => "French",
            Locale::Ru => "Russian",
            Locale::ZhHans => "Simplified Chinese",
        }
    }

    /// Map a POSIX locale string to a [`Locale`], or `None` if LibonOS does not
    /// speak it yet. Accepts `it_IT.UTF-8`, `de-DE` and bare `fr` alike.
    pub fn from_code(code: &str) -> Option<Locale> {
        let lower = code.trim().to_ascii_lowercase();
        let base = lower
            .split(['.', '@'])
            .next()
            .unwrap_or("")
            .split(['_', '-'])
            .next()
            .unwrap_or("");
        match base {
            "en" => Some(Locale::En),
            "it" => Some(Locale::It),
            "es" => Some(Locale::Es),
            "de" => Some(Locale::De),
            "fr" => Some(Locale::Fr),
            "ru" => Some(Locale::Ru),
            // A bare `zh`, `zh_CN` and `zh-Hans` all land on Simplified; the
            // traditional form is not shipped yet.
            "zh" => Some(Locale::ZhHans),
            _ => None,
        }
    }

    /// The locale to open setup with, from the environment, falling back to
    /// English. Reads `LC_ALL`, then `LC_MESSAGES`, then `LANG`.
    pub fn from_env() -> Locale {
        ["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .filter_map(|k| std::env::var(k).ok())
            .find_map(|v| Locale::from_code(&v))
            .unwrap_or(Locale::En)
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::En
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.endonym())
    }
}

/// Every string the setup flow and settings can show.
///
/// A key names a role (`NameLabel`), never its text, so rewording a sentence is
/// a table edit and nothing more.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Key {
    // Setup chrome
    SetupStepOf,
    SetupBack,
    SetupNext,
    SetupStart,
    // Welcome
    WelcomeTitle,
    WelcomeBody,
    WelcomeBulletAccount,
    WelcomeBulletManager,
    WelcomeBulletPrivacy,
    // Language
    LanguageTitle,
    LanguageBody,
    LanguageHint,
    // Name
    NameTitle,
    NameBody,
    NameLabel,
    NamePlaceholder,
    NameUsernameIs,
    // Password
    PasswordTitle,
    PasswordBody,
    PasswordLabel,
    PasswordPlaceholder,
    PasswordConfirmLabel,
    PasswordConfirmPlaceholder,
    EncryptHome,
    EncryptHomeHint,
    // Package manager
    ManagerTitle,
    ManagerBody,
    ManagerPreinstalled,
    ManagerNoRoot,
    ManagerAptDescription,
    ManagerPacmanDescription,
    ManagerLibonDescription,
    // Privacy
    PrivacyTitle,
    PrivacyBody,
    PrivacyAutoUpdates,
    PrivacyAutoUpdatesHint,
    PrivacyTelemetry,
    PrivacyTelemetryHint,
    PrivacyTelemetryOn,
    PrivacyTelemetryOff,
    // Finish
    FinishTitle,
    FinishIntro,
    FinishRowName,
    FinishRowUsername,
    FinishRowManager,
    FinishRowEncrypted,
    FinishRowAutoUpdates,
    FinishRowTelemetry,
    FinishRowLanguage,
    FinishYes,
    FinishNo,
    FinishTelemetryOn,
    FinishTelemetryOff,
    FinishPreinstalled,
    FinishLocalHint,
    // Authentication errors
    ErrorEmptyName,
    ErrorNameTooLong,
    ErrorPasswordTooShort,
    ErrorPasswordNeedsVariety,
    ErrorPasswordsDoNotMatch,
    ErrorUnknownUser,
    ErrorWrongPassword,
    // Store categories
    CategorySystem,
    CategoryInternet,
    CategoryDevelopment,
    CategoryMedia,
    CategoryOffice,
    CategoryGames,
    CategoryCommunication,
    CategoryUtilities,
    // Settings
    SettingsLanguageTitle,
    SettingsLanguageBody,
    SettingsLanguageChanged,
}

impl Key {
    /// Every key, in declaration order, for the completeness tests.
    pub const ALL: &'static [Key] = &[
        Key::SetupStepOf,
        Key::SetupBack,
        Key::SetupNext,
        Key::SetupStart,
        Key::WelcomeTitle,
        Key::WelcomeBody,
        Key::WelcomeBulletAccount,
        Key::WelcomeBulletManager,
        Key::WelcomeBulletPrivacy,
        Key::LanguageTitle,
        Key::LanguageBody,
        Key::LanguageHint,
        Key::NameTitle,
        Key::NameBody,
        Key::NameLabel,
        Key::NamePlaceholder,
        Key::NameUsernameIs,
        Key::PasswordTitle,
        Key::PasswordBody,
        Key::PasswordLabel,
        Key::PasswordPlaceholder,
        Key::PasswordConfirmLabel,
        Key::PasswordConfirmPlaceholder,
        Key::EncryptHome,
        Key::EncryptHomeHint,
        Key::ManagerTitle,
        Key::ManagerBody,
        Key::ManagerPreinstalled,
        Key::ManagerNoRoot,
        Key::ManagerAptDescription,
        Key::ManagerPacmanDescription,
        Key::ManagerLibonDescription,
        Key::PrivacyTitle,
        Key::PrivacyBody,
        Key::PrivacyAutoUpdates,
        Key::PrivacyAutoUpdatesHint,
        Key::PrivacyTelemetry,
        Key::PrivacyTelemetryHint,
        Key::PrivacyTelemetryOn,
        Key::PrivacyTelemetryOff,
        Key::FinishTitle,
        Key::FinishIntro,
        Key::FinishRowName,
        Key::FinishRowUsername,
        Key::FinishRowManager,
        Key::FinishRowEncrypted,
        Key::FinishRowAutoUpdates,
        Key::FinishRowTelemetry,
        Key::FinishRowLanguage,
        Key::FinishYes,
        Key::FinishNo,
        Key::FinishTelemetryOn,
        Key::FinishTelemetryOff,
        Key::FinishPreinstalled,
        Key::FinishLocalHint,
        Key::ErrorEmptyName,
        Key::ErrorNameTooLong,
        Key::ErrorPasswordTooShort,
        Key::ErrorPasswordNeedsVariety,
        Key::ErrorPasswordsDoNotMatch,
        Key::ErrorUnknownUser,
        Key::ErrorWrongPassword,
        Key::CategorySystem,
        Key::CategoryInternet,
        Key::CategoryDevelopment,
        Key::CategoryMedia,
        Key::CategoryOffice,
        Key::CategoryGames,
        Key::CategoryCommunication,
        Key::CategoryUtilities,
        Key::SettingsLanguageTitle,
        Key::SettingsLanguageBody,
        Key::SettingsLanguageChanged,
    ];
}

/// A locale's strings.
///
/// The lookup is a `fn` rather than a map: every key is a jump table, a table
/// costs no allocation, and a missing arm is a compile error instead of a
/// runtime fallback to English.
#[derive(Clone, Copy)]
pub struct Table {
    pub locale: Locale,
    pub get: fn(Key) -> &'static str,
}

impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Table")
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

impl Table {
    /// Look a key up. A table is total by construction, so this cannot fail.
    pub fn text(&self, key: Key) -> &'static str {
        (self.get)(key)
    }
}

/// Fills the `{n}` placeholder in a string.
///
/// Building the sentence with `format!` at the call site would bake word order
/// into the code; a translation may need the number in a different place, so the
/// placeholder is data and survives the table edit.
pub fn format_one(template: &str, value: &str) -> String {
    template.replace("{n}", value)
}

/// The table for a locale, falling back to the first one (English) if a locale
/// somehow has no table.
pub fn table_for(locale: Locale) -> &'static Table {
    TABLES
        .iter()
        .find(|t| t.locale == locale)
        .unwrap_or(&TABLES[0])
}

/// A cheap view of one locale, handed to a screen so it never has to know which
/// language is active.
#[derive(Clone, Copy)]
pub struct Strings {
    table: &'static Table,
}

impl Strings {
    pub fn new(locale: Locale) -> Self {
        Self {
            table: table_for(locale),
        }
    }

    pub fn locale(&self) -> Locale {
        self.table.locale
    }

    /// The string for a key.
    pub fn get(&self, key: Key) -> &'static str {
        self.table.text(key)
    }

    /// The string for a key, with `{n}` filled in.
    pub fn get_with(&self, key: Key, value: &str) -> String {
        format_one(self.table.text(key), value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_locale_has_exactly_one_table() {
        for locale in Locale::ALL {
            let matches = TABLES.iter().filter(|t| t.locale == locale).count();
            assert_eq!(matches, 1, "{locale:?} has {matches} tables");
        }
        assert_eq!(TABLES.len(), Locale::ALL.len());
    }

    #[test]
    fn no_string_is_empty_or_untrimmed() {
        for table in TABLES {
            for key in Key::ALL {
                let text = table.text(*key);
                assert!(!text.trim().is_empty(), "{:?} // {key:?} is empty", table.locale);
                assert_eq!(text, text.trim(), "{:?} // {key:?} is padded", table.locale);
            }
        }
    }

    #[test]
    fn only_the_placeholder_strings_carry_a_placeholder() {
        // Exactly three strings substitute a value; anything else containing
        // `{n}` is a typo that would ship a literal brace to the screen.
        let with_placeholder = [
            Key::SetupStepOf,
            Key::NameUsernameIs,
            Key::ErrorPasswordTooShort,
        ];
        for table in TABLES {
            for key in Key::ALL {
                let has = table.text(*key).contains("{n}");
                let expected = with_placeholder.contains(key);
                assert_eq!(has, expected, "{:?} // {key:?}", table.locale);
            }
        }
    }

    #[test]
    fn translations_actually_differ() {
        // A table that copies English is worse than no table: it ships the
        // pretence of a translation.
        for table in TABLES.iter().filter(|t| t.locale != Locale::En) {
            for key in [Key::WelcomeTitle, Key::NameBody] {
                assert_ne!(
                    table.text(key),
                    table_for(Locale::En).text(key),
                    "{:?} // {key:?} is left in English",
                    table.locale
                );
            }
        }
    }

    #[test]
    fn codes_round_trip() {
        for locale in Locale::ALL {
            assert_eq!(Locale::from_code(locale.code()), Some(locale));
        }
    }

    #[test]
    fn codes_from_the_environment_are_understood() {
        assert_eq!(Locale::from_code("it_IT.UTF-8"), Some(Locale::It));
        assert_eq!(Locale::from_code("de-DE"), Some(Locale::De));
        assert_eq!(Locale::from_code("zh_CN"), Some(Locale::ZhHans));
        assert_eq!(Locale::from_code("zh-Hans"), Some(Locale::ZhHans));
        assert_eq!(Locale::from_code("pt_BR"), None);
        assert_eq!(Locale::from_code(""), None);
    }

    #[test]
    fn placeholder_substitution_keeps_the_value() {
        for locale in Locale::ALL {
            let s = Strings::new(locale);
            let out = s.get_with(Key::SetupStepOf, "2");
            assert!(out.contains('2'), "{locale:?} dropped the step number: {out}");
            assert!(!out.contains("{n}"), "{locale:?} left the placeholder: {out}");
        }
    }
}
