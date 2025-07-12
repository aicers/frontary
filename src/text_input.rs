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

pub struct Model {
    input: String,
    is_invalid: bool,
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
    pub warning_message: Option<String>,
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
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::InputString(text) => {
                let trimmed_text = text.trim().to_string();

                // Validate input if validator is provided
                self.is_invalid = if let Some(validator) = ctx.props().validator {
                    !validator(&trimmed_text)
                } else {
                    false
                };

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
                {Self::view_warning_msg(ctx, self.is_invalid)}
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

    fn view_warning_msg(ctx: &Context<Self>, is_invalid: bool) -> Html {
        let txt = ctx.props().txt.txt.clone();
        if is_invalid {
            if let Some(warning_message) = &ctx.props().warning_message {
                html! {
                    <div class="input-contents-item-alert-message">
                        { text!(txt, ctx.props().language, warning_message) }
                    </div>
                }
            } else {
                html! {
                    <div class="input-contents-item-alert-message">
                        { text!(txt, ctx.props().language, "Invalid input") }
                    </div>
                }
            }
        } else {
            html! {}
        }
    }
}
