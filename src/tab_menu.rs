use std::{marker::PhantomData, rc::Rc};

use json_gettext::get_text;
use yew::{Component, Context, Html, Properties, classes, html};

use crate::{Texts, define_u32_consts, language::Language, text};

#[cfg(feature = "pumpkin")]
define_u32_consts! {
    DEFAULT_FULL_WIDTH => 1080,
    DEFAULT_ITEM_WIDTH => 80,
    MAX_ITEM_WIDTH => 100
}
#[cfg(not(feature = "pumpkin"))]
define_u32_consts! {
    DEFAULT_FULL_WIDTH => 1080,
    DEFAULT_ITEM_WIDTH => 120,
    MAX_ITEM_WIDTH => 240
}

pub enum Message {
    ClickMenu(usize),
}

pub struct Model<T> {
    phantom: PhantomData<T>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub txt: Texts,
    pub language: Language,
    pub menu_titles: Rc<Vec<&'static str>>,
    pub parent_messages: Vec<T::Message>,
    #[prop_or(None)]
    pub selected_menu: Option<T::Message>,
    #[prop_or(DEFAULT_FULL_WIDTH)]
    pub full_width: u32,
    #[prop_or(DEFAULT_ITEM_WIDTH)]
    pub item_width: u32,
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
            Message::ClickMenu(index) => {
                if let (Some(parent), Some(message)) = (
                    ctx.link().get_parent(),
                    ctx.props().parent_messages.get(index),
                ) {
                    parent.clone().downcast::<T>().send_message(message.clone());
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let div_style = if cfg!(feature = "pumpkin") {
            format!("width: {MAX_ITEM_WIDTH}%;")
        } else {
            format!("width: {}px;", ctx.props().full_width)
        };
        let table_style = if cfg!(feature = "pumpkin") {
            String::new()
        } else {
            format!("width: {}px;", ctx.props().full_width)
        };

        let style_menu = if cfg!(feature = "pumpkin") {
            String::new()
        } else {
            format!(
                "width: {}px; max-width: {}px;",
                ctx.props().item_width,
                MAX_ITEM_WIDTH
            )
        };

        html! {
            <div class="tab-menu" style={div_style}>
                <table class="tab-menu" style={table_style}>
                    <tr>
                    {
                        for ctx.props().menu_titles.iter().enumerate().map(|(index, title)| {
                            let (Some(selected), Some(menu)) = (ctx.props().selected_menu.as_ref(), ctx.props().parent_messages.get(index)) else {
                                return html! {};
                            };
                            let txt = ctx.props().txt.txt.clone();
                            let class_last = if index + 1 == ctx.props().menu_titles.len() {
                                "tab-menu-last"
                            } else {
                                "tab-menu-before-last"
                            };
                            if selected == menu {
                                html! {
                                    <td class={classes!("tab-menu-selected", class_last)} style={style_menu.clone()}>
                                        { text!(txt, ctx.props().language, title) }
                                        if cfg!(feature ="pumpkin") {
                                            <div class="selected-background">
                                            </div>
                                            <div class="selected-bar">
                                            </div>
                                        }
                                    </td>
                                }
                            } else {
                                let onclick = ctx.link().callback(move |_| Message::ClickMenu(index));
                                html! {
                                    <td class={classes!("tab-menu-unselected", class_last)} style={style_menu.clone()} onclick={onclick}>
                                        { text!(txt, ctx.props().language, title) }
                                    </td>
                                }
                            }
                        })
                    }
                        <td class="tab-menu-right">
                        </td>
                    </tr>
                </table>
            </div>
        }
    }
}
