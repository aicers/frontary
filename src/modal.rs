use std::{marker::PhantomData, rc::Rc, str::FromStr};

use json_gettext::get_text;
use yew::{AttrValue, Component, Context, Html, Properties, classes, html};

use crate::{Texts, define_u32_consts, language::Language, text};

const MAX_HEIGHT: u32 = 700;
const DEFAULT_MIN_HEIGHT: u32 = 306;
const DEFAULT_MIN_OPTION_WIDTH: u32 = 220;
const DEFAULT_MAX_OPTION_WIDTH: u32 = 440;

#[cfg(feature = "pumpkin")]
define_u32_consts! {
    DEFAULT_WIDTH => 600
}
#[cfg(not(feature = "pumpkin"))]
define_u32_consts! {
    DEFAULT_WIDTH => 714
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    Alert,
    Info,
    TextOnly,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AlignButton {
    Row,
    Column,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextStyle {
    Key,
    RawNormal,
    RawBold,
    RawHtml,
}

#[derive(PartialEq, Eq)]
pub enum Message {
    Close,
    ClickButton(usize),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub txt: Texts,
    pub language: Language,
    #[prop_or(DEFAULT_WIDTH)]
    pub width: u32,
    #[prop_or(None)]
    pub height: Option<u32>,
    #[prop_or(DEFAULT_MIN_HEIGHT)]
    pub min_height: u32,
    #[prop_or(DEFAULT_MIN_OPTION_WIDTH)]
    pub min_option_width: u32,
    #[prop_or(DEFAULT_MAX_OPTION_WIDTH)]
    pub max_option_width: u32,
    pub kind: MsgType,
    pub align_button: AlignButton,
    #[prop_or(None)]
    pub title_header: Option<&'static str>,
    pub title_messages: Rc<Vec<Vec<(String, TextStyle)>>>,
    pub option_messages: Rc<Vec<String>>,
    pub parent_messages: Vec<T::Message>,
    pub parent_cancel_message: T::Message,
}

pub struct Model<T> {
    phantom: PhantomData<T>,
}

impl<T> Component for Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    type Message = Message;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Close => {
                if let Some(parent) = ctx.link().get_parent() {
                    parent
                        .clone()
                        .downcast::<T>()
                        .send_message(ctx.props().parent_cancel_message.clone());
                }
                false
            }
            Message::ClickButton(index) => {
                if let (Some(parent), Some(message)) = (
                    ctx.link().get_parent(),
                    ctx.props().parent_messages.get(index),
                ) {
                    parent.clone().downcast::<T>().send_message(message.clone());
                }
                false
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let (icon, icon_class) = match ctx.props().kind {
            MsgType::Info => ("/frontary/modal-info.png", "modal-info"),
            MsgType::Alert => ("/frontary/modal-alert.png", "modal-alert"),
            MsgType::TextOnly => ("", ""),
        };
        let (align_class, button_class) = match ctx.props().align_button {
            AlignButton::Row => ("modal-buttons-row", "modal-button-item-row"),
            AlignButton::Column => ("modal-buttons-column", "modal-button-item-column"),
        };
        let style = if cfg!(feature = "pumpkin") {
            format!(
                "width: {}px;{}",
                ctx.props().width,
                ctx.props()
                    .height
                    .map(|h| format!(" height: {h}px;"))
                    .unwrap_or_default()
            )
        } else if matches!(ctx.props().kind, MsgType::TextOnly) {
            format!(
                "width: {}px;{}",
                ctx.props().width,
                ctx.props()
                    .height
                    .map(|h| format!(" height: {h}px;"))
                    .unwrap_or_default()
            )
        } else {
            format!(
                "width: {}px; min-height: {}px; max-height: {}px;",
                ctx.props().width,
                ctx.props().min_height,
                MAX_HEIGHT,
            )
        };
        let button_style = if cfg!(feature = "pumpkin") {
            String::new()
        } else {
            format!(
                "min-width: {}px; max-width: {}px;",
                ctx.props().min_option_width,
                ctx.props().max_option_width
            )
        };
        let onclick_close = ctx.link().callback(|_| Message::Close);
        let txt = ctx.props().txt.txt.clone();
        let title_header = if let Some(title_header) = ctx.props().title_header {
            html! {
                <div class="modal-title-header">
                    { text!(txt, ctx.props().language, title_header) }
                </div>
            }
        } else {
            html! {}
        };
        let modal_close = html! {
            <div class="modal-close">
                <img
                    src={ if cfg!(feature = "pumpkin") {
                        "/frontary/pumpkin/modal-close.svg"
                    } else {
                        "/frontary/modal-close.png"
                    } }
                    class="modal-close"
                    onclick={onclick_close}
                />
            </div>
        };
        let divider = if cfg!(feature = "pumpkin") && ctx.props().height.is_none() {
            html! {
                <img src="/frontary/pumpkin/modal-divider.svg" class="modal-divider" />
            }
        } else {
            html! {}
        };
        let modal_contents =
            if matches!(ctx.props().kind, MsgType::TextOnly) && !cfg!(feature = "pumpkin") {
                "modal-contents-text"
            } else {
                "modal-contents"
            };
        html! {
            <div class="modal-outer">
                <div class={modal_contents} style={style}>
                    if cfg!(feature="pumpkin") {
                        <div class="modal-icon-close">
                            { title_header }
                            { modal_close }
                        </div>
                        { divider }
                    } else {
                        if matches!(ctx.props().kind, MsgType::TextOnly) {
                            <div class="modal-icon-close">
                                { title_header }
                                { modal_close }
                            </div>
                        } else {
                            { modal_close }
                            <div class="modal-icon">
                                <img src={icon} class={icon_class} />
                            </div>
                        }
                    }
                    <div class="modal-messages">
                    {
                        for ctx.props().title_messages.iter().map(|ms| {
                            html! {
                                <div class="modal-message-item">
                                {
                                    for ms.iter().map(|(m, t)| {
                                        let v_node = Html::from_html_unchecked(
                                            AttrValue::from_str(m).expect("AttrValue never returns Err.")
                                        );
                                        match t {
                                            TextStyle::Key => html! {
                                                { text!(txt, ctx.props().language, m) }
                                            },
                                            TextStyle::RawNormal => html! {
                                                { m }
                                            },
                                            TextStyle::RawBold => html! {
                                                <b> { m } </b>
                                            },
                                            TextStyle::RawHtml => html! {
                                                { v_node }
                                            }
                                        }
                                    })
                                }
                                </div>
                            }
                        })
                    }
                    </div>
                    if cfg!(feature="pumpkin") {
                        if ctx.props().height.is_none() {
                            <img src="/frontary/pumpkin/modal-divider.svg" class="modal-divider" />
                        }
                    }
                    <div class={align_class}>
                    {
                        for ctx.props().option_messages.iter().enumerate().map(|(index, m)| {
                            let button_color_class = if index == 0 {
                                "modal-button-item-recommendation"
                            } else {
                                "modal-button-item-option"
                            };
                            let onclick_button = ctx.link().callback(move |_| Message::ClickButton(index));

                            html! {
                                <div
                                    class={classes!(button_class, button_color_class)}
                                    style={button_style.clone()}
                                    onclick={onclick_button}
                                >
                                    { text!(txt, ctx.props().language, m) }
                                </div>
                            }
                        })
                    }
                    </div>
                </div>
            </div>
        }
    }
}
