use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

const STORAGE_KEY: &str = "aice.theme";

#[derive(
    Clone, Copy, PartialEq, Eq, Deserialize, Serialize, EnumString, AsRefStr, Display, Debug,
)]
#[strum(serialize_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    #[must_use]
    pub fn load_from_browser() -> Option<Theme> {
        LocalStorage::get(STORAGE_KEY).ok()
    }

    pub fn save_to_browser(theme: Theme) {
        let _ = LocalStorage::set(STORAGE_KEY, theme);
    }

    pub fn themed_path(&self, file: &str) -> String {
        if cfg!(feature = "pumpkin") {
            match self {
                Theme::Light => format!("/frontary/pumpkin/light/{file}"),
                Theme::Dark => format!("/frontary/pumpkin/{file}"),
            }
        } else {
            format!("/frontary/{file}")
        }
    }
}
