use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

const STORAGE_KEY: &str = "aice.theme";

#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize, EnumString, AsRefStr, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    #[must_use]
    pub fn load_from_browser() -> Option<Theme> {
        if cfg!(feature = "pumpkin") {
            LocalStorage::get(STORAGE_KEY).ok()
        } else {
            None
        }
    }

    pub fn save_to_browser(theme: Theme) {
        if cfg!(feature = "pumpkin") {
            let _ = LocalStorage::set(STORAGE_KEY, theme);
        }
    }

    #[must_use]
    pub fn path(theme: &Option<Theme>, file: &str) -> String {
        if cfg!(feature = "pumpkin") {
            match theme {
                Some(Theme::Light) => format!("/frontary/pumpkin/light/{file}"),
                Some(Theme::Dark) | None => format!("/frontary/pumpkin/{file}"),
            }
        } else {
            format!("/frontary/{file}")
        }
    }
}
