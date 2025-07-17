use std::cell::RefCell;
use std::rc::Rc;

use json_gettext::get_text;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::{Component, Context, Html, InputEvent, Properties, html};

use crate::{Texts, input::view_asterisk, language::Language, text};
const DEFAULT_MAX_HEIGHT: u32 = 280;

#[derive(Clone, PartialEq, Eq)]
pub enum Message {
    InputString(String),
    InputError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationRule {
    UsernameFormat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationError {
    NoWhitespace,
    InvalidCharacters,
    ConsecutiveSpecialChars,
    SpecialCharAtEnd,
    MustStartWithLetter,
    InvalidLength,
}

pub struct Model {
    input: String,
    is_invalid: bool,
    validation_error: Option<ValidationError>,
}

#[derive(Clone, Eq, PartialEq, Properties)]
pub struct Props {
    pub txt: Texts,
    pub language: Language,
    #[prop_or(None)]
    pub width: Option<u32>,
    #[prop_or(DEFAULT_MAX_HEIGHT)]
    pub max_height: u32,
    pub input_data: Rc<RefCell<Option<String>>>,
    #[prop_or(None)]
    pub title: Option<String>,
    #[prop_or(None)]
    pub example_message: Option<String>,
    #[prop_or(None)]
    pub placeholder_message: Option<String>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or(None)]
    pub validation_rules: Option<ValidationRule>,
    #[prop_or(None)]
    pub validator: Option<fn(&str) -> bool>,
}

impl Component for Model {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input: String::new(),
            is_invalid: false,
            validation_error: None,
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::InputString(text) => {
                let trimmed_text = text.trim().to_string();

                // Validate input using validation rules or external validator
                if let Some(validation_rules) = &ctx.props().validation_rules {
                    self.validation_error = Self::validate_input(&trimmed_text, validation_rules);
                    self.is_invalid = self.validation_error.is_some();
                } else if let Some(validator) = ctx.props().validator {
                    self.is_invalid = !validator(&trimmed_text);
                    self.validation_error = None;
                } else {
                    self.is_invalid = false;
                    self.validation_error = None;
                }

                if let Ok(mut data) = ctx.props().input_data.try_borrow_mut() {
                    data.replace(trimmed_text);
                }
                self.input = text;
            }
            Message::InputError => {
                // TODO: issue #5
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputString(input.value())
                })
        });
        let style = format!(
            "max-height: {}px; width: {};",
            ctx.props().max_height,
            ctx.props()
                .width
                .map_or("100%".to_string(), |w| format!("{w}px"))
        );
        let placeholder = ctx.props().placeholder_message.clone().unwrap_or_default();

        html! {
            <div class="input-contents" style={style.clone()}>
                {Self::view_title(ctx)}
                <div class="input-item-group" >
                    <input type="text"
                        class={if self.is_invalid { "frontary-input-text-alert" } else { "frontary-input-text" }}
                        value={self.input.clone()}
                        style={style.clone()}
                        placeholder={placeholder}
                        oninput={oninput}
                    />
                </div>
                {Self::view_explanation_msg(ctx)}
                {Self::view_warning_msg(ctx, self.is_invalid, self.validation_error.as_ref())}
            </div>
        }
    }
}

impl Model {
    fn view_title(ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();
        if let Some(title) = &ctx.props().title {
            html! {
                <div class="input-contents-item-title">
                    { text!(txt, ctx.props().language, title) }{ view_asterisk(ctx.props().required) }
                </div>
            }
        } else {
            html! {}
        }
    }

    fn view_explanation_msg(ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();
        if let Some(example_message) = &ctx.props().example_message {
            html! {
                <div class="simple-input-input-notice">
                    { text!(txt, ctx.props().language, example_message)}
                </div>
            }
        } else {
            html! {}
        }
    }

    fn view_warning_msg(
        ctx: &Context<Self>,
        is_invalid: bool,
        validation_error: Option<&ValidationError>,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        if is_invalid {
            let warning_message = if let Some(error) = validation_error {
                Self::get_validation_error_message(error)
            } else {
                "Invalid input".to_string()
            };

            html! {
                <div class="input-contents-item-alert-message">
                    { text!(txt, ctx.props().language, &warning_message) }
                </div>
            }
        } else {
            html! {}
        }
    }

    fn validate_input(input: &str, rule: &ValidationRule) -> Option<ValidationError> {
        match rule {
            ValidationRule::UsernameFormat => Self::validate_username(input),
        }
    }

    fn validate_username(input: &str) -> Option<ValidationError> {
        // Check for whitespace
        if input.contains(char::is_whitespace) {
            return Some(ValidationError::NoWhitespace);
        }

        // Check length (3-30 characters)
        if input.len() < 3 || input.len() > 30 {
            return Some(ValidationError::InvalidLength);
        }

        // Check if starts with lowercase letter
        if !input.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
            return Some(ValidationError::MustStartWithLetter);
        }

        // Check for valid characters (lowercase letters, digits, ., -, _)
        if !input
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-' | '_'))
        {
            return Some(ValidationError::InvalidCharacters);
        }

        // Check for consecutive special characters
        let special_chars = ['.', '-', '_'];
        let chars: Vec<char> = input.chars().collect();
        for window in chars.windows(2) {
            if special_chars.contains(&window[0]) && special_chars.contains(&window[1]) {
                return Some(ValidationError::ConsecutiveSpecialChars);
            }
        }

        // Check if ends with special character
        if input
            .chars()
            .last()
            .is_some_and(|c| special_chars.contains(&c))
        {
            return Some(ValidationError::SpecialCharAtEnd);
        }

        None
    }

    fn get_validation_error_message(error: &ValidationError) -> String {
        match error {
            ValidationError::NoWhitespace => "Whitespace is not allowed".to_string(),
            ValidationError::InvalidCharacters => {
                "Only lowercase letters, digits, and special characters (., -, _) are allowed"
                    .to_string()
            }
            ValidationError::ConsecutiveSpecialChars => {
                "Consecutive special characters are not allowed".to_string()
            }
            ValidationError::SpecialCharAtEnd => {
                "Special characters at the end are not allowed".to_string()
            }
            ValidationError::MustStartWithLetter => {
                "Username must start with a lowercase letter".to_string()
            }
            ValidationError::InvalidLength => {
                "Length must be between 3 and 30 characters".to_string()
            }
        }
    }
}
