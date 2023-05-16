use crate::{input::view_asterisk, language::Language, text, IpRange, Texts};
use json_gettext::get_text;
use std::{cell::RefCell, net::Ipv4Addr, rc::Rc, str::FromStr};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, InputEvent, Properties};

const DEFAULT_MAX_HEIGHT: u32 = 280;

#[derive(Clone, PartialEq, Eq)]
pub enum Message {
    InputString(String),
    InputError,
}

pub struct Model {
    input: String,
}

#[derive(Clone, Eq, PartialEq, Properties)]
pub struct Props {
    pub txt: Texts,
    pub language: Language,
    #[prop_or(None)]
    pub width: Option<u32>,
    #[prop_or(DEFAULT_MAX_HEIGHT)]
    pub max_height: u32,
    pub input_data: Rc<RefCell<Option<IpRange>>>,
    #[prop_or(None)]
    pub title: Option<String>,
    #[prop_or(None)]
    pub example_message: Option<String>,
    #[prop_or(None)]
    pub placeholder_message: Option<String>,
    #[prop_or(false)]
    pub required: bool,
}

impl Component for Model {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input: String::new(),
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::InputString(text) => {
                if let Ok(mut data) = ctx.props().input_data.try_borrow_mut() {
                    if let Some(range) = check_input(&text) {
                        data.replace(range);
                    } else {
                        data.take();
                    }
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
        let placeholder = ctx
            .props()
            .placeholder_message
            .clone()
            .unwrap_or(String::new());
        let class = if check_input(&self.input).is_some() || self.input.is_empty() {
            "frontary-input-text"
        } else {
            "frontary-input-text-alert"
        };

        html! {
            <div class="input-contents" style={style.clone()}>
                {Self::view_title(ctx)}
                <div class="input-item-group" >
                    <input type="text"
                        class={class}
                        value={self.input.clone()}
                        style={style.clone()}
                        placeholder={placeholder}
                        oninput={oninput}
                    />
                </div>
                {Self::view_explanation_msg(ctx)}
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
}

fn check_input(input: &str) -> Option<IpRange> {
    if input.contains('~') {
        input.split_once('~').map_or_else(
            || None,
            |(start, end)| {
                if Ipv4Addr::from_str(start.trim()).is_ok()
                    && Ipv4Addr::from_str(end.trim()).is_ok()
                {
                    Some(IpRange {
                        start: start.trim().to_string(),
                        end: end.trim().to_string(),
                    })
                } else {
                    None
                }
            },
        )
    } else {
        Ipv4Addr::from_str(input.trim()).ok().map(|ip| IpRange {
            start: ip.to_string(),
            end: String::new(),
        })
    }
}
