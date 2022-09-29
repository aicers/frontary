use gloo_storage::{LocalStorage, Result as GlooResult, Storage};
use serde::{Deserialize, Serialize};

type Text = Result<String, anyhow::Error>;
#[allow(dead_code)]
const STORAGE_KEY: &str = "aice.language";

/// Supported languages.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Language {
    English,
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
    /// Returns the language subtag.
    #[must_use]
    pub fn language_subtag(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Korean => "ko",
        }
    }

    /// Returns the BCP 47 tag.
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::Korean => "ko-KR",
        }
    }
}

#[allow(dead_code)]
pub fn get() -> Language {
    const DEFAULT_LANGUAGE: Language = Language::English;

    let lang: GlooResult<Language> = LocalStorage::get(STORAGE_KEY);
    lang.unwrap_or(DEFAULT_LANGUAGE)
}

#[allow(dead_code)]
pub fn set(lang: Language) {
    let _rtn = LocalStorage::set(STORAGE_KEY, lang);
}

#[macro_export]
macro_rules! text {
    ($c:ident, $l:expr, $k:expr) => {
        get_text!($c, $l.tag(), $k).expect("valid key")
    };
}
