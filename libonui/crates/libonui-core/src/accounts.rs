//! Local account storage: the first name and password created on first boot,
//! and the check the login screen runs against them.
//!
//! Passwords are never stored in clear. We derive a salted PBKDF2-HMAC-SHA256
//! digest; a login succeeds only when the recomputed digest matches in constant
//! time.

use crate::i18n::{Key, Locale, Strings};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Number of PBKDF2 rounds. High enough to be a real cost for guessing, low
/// enough that unlocking the desktop stays instant on modest hardware.
pub const PBKDF2_ROUNDS: u32 = 120_000;
const SALT_LEN: usize = 16;
const KEY_LEN: usize = 32;

/// The stored credential for the single local user of this machine.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub first_name: String,
    pub username: String,
    /// Hex-encoded random salt.
    pub salt_hex: String,
    /// Hex-encoded derived key.
    pub hash_hex: String,
    pub rounds: u32,
    /// Whether the user asked for their home directory to be encrypted at rest.
    pub encrypt_home: bool,
    pub created_utc: String,
}

/// Errors a caller can present to the user, in the product's own language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthError {
    EmptyName,
    NameTooLong,
    PasswordTooShort { min: usize },
    PasswordNeedsVariety,
    PasswordsDoNotMatch,
    UnknownUser,
    WrongPassword,
}

impl AuthError {
    /// The message as a string key, so the UI renders it in the active language
    /// instead of in whichever language this enum was written in.
    ///
    /// `PasswordTooShort` carries a length the sentence has to show, so the key
    /// alone is not enough: [`AuthError::placeholder`] carries the value and
    /// [`AuthError::message`] fills it back in.
    pub fn key(&self) -> Key {
        match self {
            AuthError::EmptyName => Key::ErrorEmptyName,
            AuthError::NameTooLong => Key::ErrorNameTooLong,
            AuthError::PasswordTooShort { .. } => Key::ErrorPasswordTooShort,
            AuthError::PasswordNeedsVariety => Key::ErrorPasswordNeedsVariety,
            AuthError::PasswordsDoNotMatch => Key::ErrorPasswordsDoNotMatch,
            AuthError::UnknownUser => Key::ErrorUnknownUser,
            AuthError::WrongPassword => Key::ErrorWrongPassword,
        }
    }

    /// The value the message's `{n}` placeholder takes, when it has one.
    pub fn placeholder(&self) -> Option<String> {
        match self {
            AuthError::PasswordTooShort { min } => Some(min.to_string()),
            _ => None,
        }
    }

    /// The message in one locale. Kept for callers with no string table handy —
    /// a test, a log line, or the CLI's error path.
    pub fn message(&self, locale: Locale) -> String {
        let t = Strings::new(locale);
        match self.placeholder() {
            Some(value) => t.get_with(self.key(), &value),
            None => t.get(self.key()).to_string(),
        }
    }
}

/// The minimum password length LibonOS accepts.
pub const MIN_PASSWORD_LEN: usize = 8;

/// Validate the credentials the user typed during first-run setup.
pub fn validate_new_account(
    first_name: &str,
    password: &str,
    confirm: &str,
) -> Result<String, AuthError> {
    let name = first_name.trim();
    if name.is_empty() {
        return Err(AuthError::EmptyName);
    }
    if name.chars().count() > 32 {
        return Err(AuthError::NameTooLong);
    }
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(AuthError::PasswordTooShort {
            min: MIN_PASSWORD_LEN,
        });
    }
    if !has_letter_and_number(password) {
        return Err(AuthError::PasswordNeedsVariety);
    }
    if password != confirm {
        return Err(AuthError::PasswordsDoNotMatch);
    }
    Ok(name.to_string())
}

fn has_letter_and_number(s: &str) -> bool {
    s.chars().any(|c| c.is_alphabetic()) && s.chars().any(|c| c.is_ascii_digit())
}

/// Turn the chosen name into a POSIX-friendly username.
pub fn username_from(name: &str) -> String {
    let mut out: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c == ' ' || c == '_' || c == '-' || c == '.' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect();
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "libon".to_string()
    } else {
        out
    }
}

/// Create the account, hashing the password with a fresh random salt.
pub fn create_account(first_name: &str, password: &str, encrypt_home: bool) -> Account {
    let salt = random_salt();
    let hash = derive(password, &salt, PBKDF2_ROUNDS);
    Account {
        first_name: first_name.trim().to_string(),
        username: username_from(first_name),
        salt_hex: to_hex(&salt),
        hash_hex: to_hex(&hash),
        rounds: PBKDF2_ROUNDS,
        encrypt_home,
        created_utc: chrono::Utc::now().to_rfc3339(),
    }
}

impl Account {
    /// Check a typed password against the stored digest in constant time.
    pub fn verify(&self, password: &str) -> bool {
        let Ok(salt) = from_hex(&self.salt_hex) else {
            return false;
        };
        let Ok(expected) = from_hex(&self.hash_hex) else {
            return false;
        };
        let candidate = derive(password, &salt, self.rounds);
        constant_time_eq(&candidate, &expected)
    }
}

fn random_salt() -> Vec<u8> {
    use rand::RngCore;
    let mut salt = vec![0u8; SALT_LEN];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}

/// PBKDF2-HMAC-SHA256 (RFC 8018) implemented directly so the shell carries no
/// cryptographic dependency beyond SHA-256.
pub fn derive(password: &str, salt: &[u8], rounds: u32) -> Vec<u8> {
    let blocks = KEY_LEN.div_ceil(32);
    let mut key = Vec::with_capacity(blocks * 32);
    for block_index in 1..=blocks as u32 {
        let mut u = hmac_sha256(password.as_bytes(), salt, block_index);
        let mut acc = u;
        for _ in 1..rounds {
            u = hmac_sha256(password.as_bytes(), &u, 0);
            for (a, b) in acc.iter_mut().zip(u.iter()) {
                *a ^= *b;
            }
        }
        key.extend_from_slice(&acc);
    }
    key.truncate(KEY_LEN);
    key
}

/// HMAC-SHA256 with `msg = salt || block_index_be`.
fn hmac_sha256(key: &[u8], salt: &[u8], block_index: u32) -> [u8; 32] {
    let mut msg = salt.to_vec();
    if block_index != 0 {
        msg.extend_from_slice(&block_index.to_be_bytes());
    }

    let mut key_block = [0u8; 64];
    if key.len() > 64 {
        let mut hasher = Sha256::new();
        hasher.update(key);
        key_block[..32].copy_from_slice(&hasher.finalize());
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(&msg);
    let inner = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner);
    outer.finalize().into()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Why a hex string could not be decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HexError;

impl std::fmt::Display for HexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("stringa esadecimale non valida")
    }
}

pub fn from_hex(s: &str) -> Result<Vec<u8>, HexError> {
    if s.len() % 2 != 0 {
        return Err(HexError);
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| HexError))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_verifies_and_rejects_wrong_password() {
        let acct = create_account("Marco", "cavallo7stella", false);
        assert!(acct.verify("cavallo7stella"));
        assert!(!acct.verify("cavallo7stelli"));
        assert!(!acct.verify(""));
    }

    #[test]
    fn two_accounts_with_same_password_differ() {
        let a = create_account("Marco", "cavallo7stella", false);
        let b = create_account("Marco", "cavallo7stella", false);
        assert_ne!(a.salt_hex, b.salt_hex);
        assert_ne!(a.hash_hex, b.hash_hex);
    }

    #[test]
    fn validation_rules() {
        assert_eq!(
            validate_new_account("", "abcdefgh1", "abcdefgh1"),
            Err(AuthError::EmptyName)
        );
        assert_eq!(
            validate_new_account("Ada", "short1", "short1"),
            Err(AuthError::PasswordTooShort { min: 8 })
        );
        assert_eq!(
            validate_new_account("Ada", "onlyletters", "onlyletters"),
            Err(AuthError::PasswordNeedsVariety)
        );
        assert_eq!(
            validate_new_account("Ada", "abcdefgh1", "abcdefgh2"),
            Err(AuthError::PasswordsDoNotMatch)
        );
        assert_eq!(
            validate_new_account("  Ada  ", "abcdefgh1", "abcdefgh1"),
            Ok("Ada".to_string())
        );
    }

    #[test]
    fn username_is_posix_friendly() {
        assert_eq!(username_from("Marco Rossi"), "marco-rossi");
        assert_eq!(username_from("  Ada!!  "), "ada");
        assert_eq!(username_from("---"), "libon");
        assert_eq!(username_from(""), "libon");
    }

    #[test]
    fn derive_is_deterministic() {
        let salt = b"0123456789abcdef";
        assert_eq!(derive("pw1", salt, 10), derive("pw1", salt, 10));
        assert_ne!(derive("pw1", salt, 10), derive("pw2", salt, 10));
    }

    #[test]
    fn hex_round_trips() {
        let bytes = vec![0x00, 0x0f, 0xa5, 0xff];
        assert_eq!(from_hex(&to_hex(&bytes)).unwrap(), bytes);
        assert!(from_hex("abc").is_err());
    }
}

#[cfg(test)]
mod i18n_tests {
    use super::*;

    #[test]
    fn every_error_maps_to_a_key_that_carries_its_own_message() {
        for err in [
            AuthError::EmptyName,
            AuthError::NameTooLong,
            AuthError::PasswordTooShort { min: 8 },
            AuthError::PasswordNeedsVariety,
            AuthError::PasswordsDoNotMatch,
            AuthError::UnknownUser,
            AuthError::WrongPassword,
        ] {
            let key = err.key();
            for locale in Locale::ALL {
                let text = err.message(locale);
                assert!(!text.trim().is_empty(), "{locale:?} // {key:?}");
                assert!(!text.contains("{n}"), "{locale:?} left a placeholder: {text}");
            }
        }
    }

    #[test]
    fn the_short_password_message_shows_the_real_minimum() {
        let err = AuthError::PasswordTooShort { min: 12 };
        assert_eq!(err.placeholder().as_deref(), Some("12"));
        assert!(err.message(Locale::En).contains("12"));
        assert!(err.message(Locale::Ru).contains("12"));
        assert!(err.message(Locale::ZhHans).contains("12"));
    }
}
