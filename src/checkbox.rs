use strum_macros::Display;
use yew::{Html, Properties, function_component, html};

use crate::{Theme, define_str_consts};

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
    #[prop_or(None)]
    pub theme: Option<Theme>,
}

#[function_component(Model)]
pub fn model(props: &Props) -> Html {
    let filename = props.always.map_or_else(
        || match props.status {
            CheckStatus::Checked => CHECKED_URL,
            CheckStatus::Indeterminate => INDETERMINATE_URL,
            CheckStatus::Unchecked => UNCHECKED_URL,
        },
        |status| match status {
            CheckStatus::Checked => ALWAYS_CHECKED_URL,
            CheckStatus::Indeterminate => ALWAYS_INDETERMINATE_URL,
            CheckStatus::Unchecked => ALWAYS_UNCHECKED_URL,
        },
    );
    let theme = props.theme;
    let url = Theme::path(&theme, filename);
    let style = format!("background-image: url({url});");

    html! {
        <div role="checkbox" class="basic-checkbox" {style}>
        </div>
    }
}
