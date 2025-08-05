//! Language support and internationalization utilities.
//!
//! This module provides language detection, browser storage, and utilities
//! for working with localized text in the UI.

use gloo_storage::{LocalStorage, Result as GlooResult, Storage};
use serde::{Deserialize, Serialize};

/// Type alias for text parsing results
type Text = Result<String, anyhow::Error>;

/// Local storage key for persisting language preferences
const STORAGE_KEY: &str = "aice.language";

/// Supported languages for the UI.
///
/// Currently supports English and Korean with automatic detection
/// from browser preferences.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Language {
    /// English language (en-US)
    English,
    /// Korean language (ko-KR)
    Korean,
}

impl From<Text> for Language {
    fn from(text: Text) -> Self {
        const DEFAULT_TAG: &str = "en-US";

        let tag = text
            .ok()
            .or_else(|| web_sys::window().and_then(|w| w.navigator().language()))
            .unwrap_or_else(|| DEFAULT_TAG.to_string());
        if tag.starts_with("ko") {
            Language::Korean
        } else {
            Language::English
        }
    }
}

impl From<Language> for Text {
    fn from(language: Language) -> Self {
        Ok(language.tag().to_string())
    }
}

impl Language {
    /// Returns the ISO 639-1 language subtag.
    ///
    /// # Returns
    ///
    /// The two-letter language code ("en" or "ko")
    #[must_use]
    pub fn language_subtag(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Korean => "ko",
        }
    }

    /// Returns the full BCP 47 language tag.
    ///
    /// # Returns
    ///
    /// The full language tag including region ("en-US" or "ko-KR")
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::Korean => "ko-KR",
        }
    }
}

/// Gets the current language preference from browser storage.
///
/// Falls back to English if no preference is stored or if the stored
/// value is invalid.
///
/// # Returns
///
/// The user's preferred language or the default (English)
#[must_use]
pub fn get() -> Language {
    const DEFAULT_LANGUAGE: Language = Language::English;

    let lang: GlooResult<Language> = LocalStorage::get(STORAGE_KEY);
    lang.unwrap_or(DEFAULT_LANGUAGE)
}

/// Saves the language preference to browser storage.
///
/// # Arguments
///
/// * `lang` - The language to save as the user's preference
pub fn set(lang: Language) {
    let _rtn = LocalStorage::set(STORAGE_KEY, lang);
}

#[macro_export]
macro_rules! text {
    ($c:ident, $l:expr, $k:expr) => {{
        let invalid_text_key_msg = if cfg!(feature = "test") {
            format!("invalid text key: {}", $k)
        } else {
            "invalid text key".to_string()
        };
        get_text!($c, $l.tag(), $k)
            .expect(&invalid_text_key_msg)
            .to_string()
    }};
}
