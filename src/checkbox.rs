use strum_macros::Display;
use yew::{function_component, html, Html, Properties};

use crate::{define_str_consts, theme::Theme};

#[cfg(feature = "pumpkin")]
define_str_consts! {
    CHECKED_URL => "enabled-checked.svg",
    INDETERMINATE_URL => "enabled-indeterminate.svg",
    UNCHECKED_URL => "enabled-unchecked.svg",
    ALWAYS_CHECKED_URL => "disabled-checked.svg",
    ALWAYS_INDETERMINATE_URL => "disabled-indeterminate.svg",
    ALWAYS_UNCHECKED_URL => "disabled-unchecked.svg"
}
#[cfg(not(feature = "pumpkin"))]
define_str_consts! {
    CHECKED_URL => "checkbox-checked.png",
    INDETERMINATE_URL => "checkbox-indeterminate.png",
    UNCHECKED_URL => "checkbox-unchecked.png",
    ALWAYS_CHECKED_URL => "checkbox-checked-always.png",
    ALWAYS_INDETERMINATE_URL => "checkbox-indeterminate-always.png",
    ALWAYS_UNCHECKED_URL => "checkbox-unchecked-always.png"
}

#[derive(Clone, Copy, Display, PartialEq, Eq, Default)]
pub enum CheckStatus {
    Checked,
    Indeterminate,
    #[default]
    Unchecked,
}

#[derive(Clone, PartialEq, Eq, Properties)]
pub struct Props {
    #[prop_or(CheckStatus::Unchecked)]
    pub status: CheckStatus,
    #[prop_or(None)]
    pub always: Option<CheckStatus>,
    #[prop_or_default]
    pub theme: Option<Theme>,
}

#[function_component(Model)]
pub fn model(props: &Props) -> Html {
    let status = props.always.unwrap_or(props.status);

    let filename = match (status, props.always.is_some()) {
        (CheckStatus::Checked, true) => ALWAYS_CHECKED_URL,
        (CheckStatus::Indeterminate, true) => ALWAYS_INDETERMINATE_URL,
        (CheckStatus::Unchecked, true) => ALWAYS_UNCHECKED_URL,
        (CheckStatus::Checked, false) => CHECKED_URL,
        (CheckStatus::Indeterminate, false) => INDETERMINATE_URL,
        (CheckStatus::Unchecked, false) => UNCHECKED_URL,
    };

    let url = props.theme.unwrap_or(Theme::Dark).themed_path(filename);
    let style = format!("background-image: url({url});");

    html! {
        <div role="checkbox" class="basic-checkbox" {style}>
        </div>
    }
}
