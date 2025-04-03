use strum_macros::Display;
use yew::{Html, Properties, function_component, html};

use crate::define_str_consts;

#[cfg(feature = "pumpkin")]
define_str_consts! {
    CHECKED_URL => "/frontary/pumpkin/enabled-checked.svg",
    INDETERMINATE_URL => "/frontary/pumpkin/enabled-indeterminate.svg",
    UNCHECKED_URL => "/frontary/pumpkin/enabled-unchecked.svg",
    ALWAYS_CHECKED_URL => "/frontary/pumpkin/disabled-checked.svg",
    ALWAYS_INDETERMINATE_URL => "/frontary/pumpkin/disabled-indeterminate.svg",
    ALWAYS_UNCHECKED_URL => "/frontary/pumpkin/disabled-unchecked.svg"
}
#[cfg(not(feature = "pumpkin"))]
define_str_consts! {
    CHECKED_URL => "/frontary/checkbox-checked.png",
    INDETERMINATE_URL => "/frontary/checkbox-indeterminate.png",
    UNCHECKED_URL => "/frontary/checkbox-unchecked.png",
    ALWAYS_CHECKED_URL => "/frontary/checkbox-checked-always.png",
    ALWAYS_INDETERMINATE_URL => "/frontary/checkbox-indeterminate-always.png",
    ALWAYS_UNCHECKED_URL => "/frontary/checkbox-unchecked-always.png"
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
}

#[function_component(Model)]
pub fn model(props: &Props) -> Html {
    let url = props.always.map_or_else(
        || match props.status {
            CheckStatus::Checked => CHECKED_URL,
            CheckStatus::Indeterminate => INDETERMINATE_URL,
            CheckStatus::Unchecked => UNCHECKED_URL,
        },
        |x| match x {
            CheckStatus::Checked => ALWAYS_CHECKED_URL,
            CheckStatus::Indeterminate => ALWAYS_INDETERMINATE_URL,
            CheckStatus::Unchecked => ALWAYS_UNCHECKED_URL,
        },
    );
    let style = format!("background-image: url({url});");

    html! {
        <div role="checkbox" class="basic-checkbox" {style}>
        </div>
    }
}
