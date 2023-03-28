use strum_macros::Display;
use yew::{function_component, html, Html, Properties};

#[derive(Clone, Copy, Display, PartialEq, Eq)]
pub enum CheckStatus {
    Checked,
    Indeterminate,
    Unchecked,
}

#[derive(Clone, PartialEq, Eq, Properties)]
pub struct Props {
    #[prop_or(CheckStatus::Unchecked)]
    pub status: CheckStatus,
    #[prop_or(None)]
    pub always: Option<CheckStatus>,
}

const CHECKED_URL: &str = "/frontary/checkbox-checked.png";
const INDETERMINATE_URL: &str = "/frontary/checkbox-indeterminate.png";
const UNCHECKED_URL: &str = "/frontary/checkbox-unchecked.png";
const ALWAYS_CHECKED_URL: &str = "/frontary/checkbox-checked-always.png";
const ALWAYS_INDETERMINATE_URL: &str = "/frontary/checkbox-indeterminate-always.png";
const ALWAYS_UNCHECKED_URL: &str = "/frontary/checkbox-unchecked-always.png";

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
        <div class="basic-checkbox" style={style}>
        </div>
    }
}
