//! The first-run experience: the sequence of questions LibonOS asks before the
//! desktop appears for the very first time.
//!
//! The flow is modelled as an explicit step machine so the UI cannot skip a
//! required answer, and so the same order is testable without a display.

use crate::accounts::{validate_new_account, Account, AuthError};
use crate::i18n::{Key, Locale, Strings};
use crate::packages::PackageManagerKind;
use serde::{Deserialize, Serialize};

/// A single question in the setup flow, in the order it is asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupStep {
    Welcome,
    Language,
    Name,
    Password,
    PackageManager,
    Privacy,
    Finish,
}

impl SetupStep {
    pub const ALL: [SetupStep; 7] = [
        SetupStep::Welcome,
        SetupStep::Language,
        SetupStep::Name,
        SetupStep::Password,
        SetupStep::PackageManager,
        SetupStep::Privacy,
        SetupStep::Finish,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    pub fn next(self) -> SetupStep {
        let i = self.index();
        Self::ALL.get(i + 1).copied().unwrap_or(SetupStep::Finish)
    }

    pub fn previous(self) -> SetupStep {
        let i = self.index();
        if i == 0 {
            SetupStep::Welcome
        } else {
            Self::ALL[i - 1]
        }
    }

    pub fn is_last(self) -> bool {
        self == SetupStep::Finish
    }

    /// Progress in `0.0..=1.0`, used to draw the step rail.
    pub fn progress(self) -> f32 {
        self.index() as f32 / (Self::ALL.len() - 1) as f32
    }

    /// The heading for this step, in the active language.
    pub fn title(self, t: &Strings) -> &'static str {
        t.get(self.title_key())
    }

    /// The key behind [`SetupStep::title`], for callers that need the key rather
    /// than the text — a test, or a log line.
    pub fn title_key(self) -> Key {
        match self {
            SetupStep::Welcome => Key::WelcomeTitle,
            SetupStep::Language => Key::LanguageTitle,
            SetupStep::Name => Key::NameTitle,
            SetupStep::Password => Key::PasswordTitle,
            SetupStep::PackageManager => Key::ManagerTitle,
            SetupStep::Privacy => Key::PrivacyTitle,
            SetupStep::Finish => Key::FinishTitle,
        }
    }
}

/// Answers gathered during setup, before the account is written to disk.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetupAnswers {
    /// The interface language chosen on the first step. It becomes the system
    /// locale once setup finishes.
    pub locale: Locale,
    pub first_name: String,
    pub password: String,
    pub confirm: String,
    pub encrypt_home: bool,
    pub package_manager: PackageManagerKind,
    pub telemetry_enabled: bool,
    pub automatic_updates: bool,
}

impl Default for SetupAnswers {
    fn default() -> Self {
        Self {
            // Opening setup in the language the machine already speaks is the
            // one default that never has to be asked twice.
            locale: Locale::from_env(),
            first_name: String::new(),
            password: String::new(),
            confirm: String::new(),
            encrypt_home: true,
            // LibonOS ships with its own manager, so it is the sensible default.
            package_manager: PackageManagerKind::Libon,
            telemetry_enabled: false,
            automatic_updates: true,
        }
    }
}

/// The live setup state.
#[derive(Clone, Debug)]
pub struct SetupState {
    pub step: SetupStep,
    pub answers: SetupAnswers,
    /// Set after a failed attempt to leave a step, so the UI can show why. It
    /// holds a [`Key`] rather than a sentence, so switching language in the
    /// middle of setup re-renders the reason in the new language.
    pub error: Option<Key>,
}

impl Default for SetupState {
    fn default() -> Self {
        Self {
            step: SetupStep::Welcome,
            answers: SetupAnswers::default(),
            error: None,
        }
    }
}

impl SetupState {
    /// Advance if the current step is satisfied. Returns the step that is now
    /// active; the caller renders whatever it returns.
    pub fn advance(&mut self) -> SetupStep {
        self.error = None;
        if let Err(err) = self.validate_current() {
            self.error = Some(err.key());
            return self.step;
        }
        self.step = self.step.next();
        self.step
    }

    /// The string table for the language this setup is running in.
    pub fn strings(&self) -> Strings {
        Strings::new(self.answers.locale)
    }

    /// Switch the interface language, keeping every answer already given. A
    /// validation message on screen re-renders in the new language, because the
    /// error is stored as a key.
    pub fn set_locale(&mut self, locale: Locale) {
        self.answers.locale = locale;
    }

    pub fn back(&mut self) -> SetupStep {
        self.error = None;
        self.step = self.step.previous();
        self.step
    }

    /// Validate only the question currently on screen.
    pub fn validate_current(&self) -> Result<(), AuthError> {
        match self.step {
            SetupStep::Name => {
                if self.answers.first_name.trim().is_empty() {
                    Err(AuthError::EmptyName)
                } else if self.answers.first_name.chars().count() > 32 {
                    Err(AuthError::NameTooLong)
                } else {
                    Ok(())
                }
            }
            SetupStep::Password => validate_new_account(
                &self.answers.first_name,
                &self.answers.password,
                &self.answers.confirm,
            )
            .map(|_| ()),
            _ => Ok(()),
        }
    }

    /// Validate every answer and, if all good, produce the account to persist.
    pub fn build_account(&self) -> Result<Account, AuthError> {
        validate_new_account(
            &self.answers.first_name,
            &self.answers.password,
            &self.answers.confirm,
        )?;
        Ok(crate::accounts::create_account(
            &self.answers.first_name,
            &self.answers.password,
            self.answers.encrypt_home,
        ))
    }

    /// A rough password strength in `0..=4` for the live meter on the password
    /// step. Strength is advisory; the hard rule is in `validate_new_account`.
    pub fn password_strength(&self) -> u8 {
        let pw = &self.answers.password;
        if pw.is_empty() {
            return 0;
        }
        let mut score = 0;
        if pw.chars().count() >= 8 {
            score += 1;
        }
        if pw.chars().count() >= 12 {
            score += 1;
        }
        if pw.chars().any(|c| c.is_ascii_digit()) && pw.chars().any(|c| c.is_alphabetic()) {
            score += 1;
        }
        if pw.chars().any(|c| !c.is_alphanumeric()) {
            score += 1;
        }
        score
    }
}

/// Persisted record of a completed setup, written next to the account.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FirstRunRecord {
    pub completed: bool,
    /// The language setup was finished in. The desktop reads it on boot, so the
    /// choice survives a restart without a second setting to keep in sync.
    #[serde(default)]
    pub locale: Locale,
    pub package_manager: PackageManagerKind,
    pub telemetry_enabled: bool,
    pub automatic_updates: bool,
    pub completed_utc: String,
}

impl FirstRunRecord {
    pub fn from_answers(answers: &SetupAnswers) -> Self {
        Self {
            completed: true,
            locale: answers.locale,
            package_manager: answers.package_manager,
            telemetry_enabled: answers.telemetry_enabled,
            automatic_updates: answers.automatic_updates,
            completed_utc: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_advance_past_name_without_a_name() {
        let mut s = SetupState {
            step: SetupStep::Name,
            ..SetupState::default()
        };
        assert_eq!(s.advance(), SetupStep::Name);
        assert!(s.error.is_some());
        s.answers.first_name = "Giulia".into();
        assert_eq!(s.advance(), SetupStep::Password);
        assert!(s.error.is_none());
    }

    #[test]
    fn cannot_advance_past_password_when_weak() {
        let mut s = SetupState::default();
        s.answers.first_name = "Giulia".into();
        s.step = SetupStep::Password;
        s.answers.password = "abc".into();
        s.answers.confirm = "abc".into();
        assert_eq!(s.advance(), SetupStep::Password);
        s.answers.password = "libon2026os".into();
        s.answers.confirm = "libon2026os".into();
        assert_eq!(s.advance(), SetupStep::PackageManager);
    }

    #[test]
    fn navigation_is_bounded() {
        assert_eq!(SetupStep::Welcome.previous(), SetupStep::Welcome);
        assert_eq!(SetupStep::Finish.next(), SetupStep::Finish);
        let mut s = SetupState::default();
        s.back();
        assert_eq!(s.step, SetupStep::Welcome);
        assert!(SetupStep::Welcome.progress() == 0.0);
        assert!(SetupStep::Finish.progress() == 1.0);
    }

    #[test]
    fn language_comes_first_and_defaults_to_the_environment() {
        assert_eq!(SetupStep::Welcome.next(), SetupStep::Language);
        assert_eq!(SetupStep::Language.next(), SetupStep::Name);
        assert_eq!(SetupStep::ALL.len(), 7);
        let s = SetupState::default();
        assert!(Locale::ALL.contains(&s.answers.locale));
    }

    #[test]
    fn switching_language_keeps_the_answers_given_so_far() {
        let mut s = SetupState::default();
        s.answers.first_name = "Giulia".into();
        s.set_locale(Locale::Ru);
        assert_eq!(s.answers.first_name, "Giulia");
        assert_eq!(s.strings().get(Key::NameTitle), "Как вас называть?");
    }

    #[test]
    fn a_validation_error_survives_a_language_change() {
        // The reason is stored as a key, so it re-renders rather than being a
        // sentence frozen in the language it was raised in.
        let mut s = SetupState {
            step: SetupStep::Name,
            ..SetupState::default()
        };
        s.set_locale(Locale::En);
        s.advance();
        assert_eq!(s.error, Some(Key::ErrorEmptyName));
        s.set_locale(Locale::De);
        let shown = s.strings().get(s.error.unwrap());
        assert_eq!(shown, "Gib deinen Namen ein.");
    }

    #[test]
    fn every_step_has_a_title_in_every_language() {
        let mut s = SetupState::default();
        for locale in Locale::ALL {
            s.set_locale(locale);
            let t = s.strings();
            for step in SetupStep::ALL {
                assert!(!step.title(&t).trim().is_empty(), "{locale:?} // {step:?}");
            }
        }
    }

    #[test]
    fn build_account_requires_valid_answers() {
        let mut s = SetupState::default();
        assert!(s.build_account().is_err());
        s.answers.first_name = "Luca".into();
        s.answers.password = "libon2026os".into();
        s.answers.confirm = "libon2026os".into();
        let acct = s.build_account().unwrap();
        assert_eq!(acct.username, "luca");
        assert!(acct.verify("libon2026os"));
    }

    #[test]
    fn strength_grows_with_variety() {
        let mut s = SetupState::default();
        s.answers.password = "aaaaaaaa".into();
        let weak = s.password_strength();
        s.answers.password = "Libon-2026-os".into();
        assert!(s.password_strength() > weak);
        assert_eq!(s.password_strength(), 4);
    }
}
