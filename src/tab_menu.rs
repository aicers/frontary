use crate::{language::Language, text, Texts};
use json_gettext::get_text;
use std::{marker::PhantomData, rc::Rc};
use yew::{classes, html, Component, Context, Html, Properties};

pub enum Message {
    ClickMenu(usize),
}

const DEFAULT_FULL_WIDTH: u32 = 1080;
const DEFAULT_ITEM_WIDTH: u32 = 120;
const MAX_ITEM_WIDTH: u32 = 240;

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
        let style = format!("width: {}px;", ctx.props().full_width);
        let style_menu = format!(
            "width: {}px; max-width: {}px;",
            ctx.props().item_width,
            MAX_ITEM_WIDTH
        );

        html! {
            <div class="tab-menu" style={style.clone()}>
                <table class="tab-menu" style={style}>
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
