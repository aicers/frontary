//! Theme management and browser storage for UI theming.
//!
//! This module provides functionality for managing dark/light themes with
//! persistent storage in the browser's local storage.

use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

/// Local storage key for persisting theme preferences
const STORAGE_KEY: &str = "aice.theme";

/// Available UI themes for the application.
///
/// Supports dark and light theme variants with automatic serialization
/// for browser storage.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize, EnumString, AsRefStr, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Theme {
    /// Dark theme variant
    Dark,
    /// Light theme variant
    Light,
}

impl Theme {
    /// Loads the saved theme preference from browser local storage.
    ///
    /// # Returns
    ///
    /// Returns `Some(Theme)` if a valid theme is stored, `None` if no theme
    /// is saved or if the stored value is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use frontary::Theme;
    ///
    /// match Theme::load_from_browser() {
    ///     Some(theme) => println!("Loaded theme: {}", theme),
    ///     None => println!("No theme saved, using default"),
    /// }
    /// ```
    #[must_use]
    pub fn load_from_browser() -> Option<Theme> {
        LocalStorage::get(STORAGE_KEY).ok()
    }

    /// Saves the theme preference to browser local storage.
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme to save to browser storage
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use frontary::Theme;
    ///
    /// Theme::save_to_browser(Theme::Dark);
    /// ```
    pub fn save_to_browser(theme: Theme) {
        let _ = LocalStorage::set(STORAGE_KEY, theme);
    }
}
