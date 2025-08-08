use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};

use json_gettext::get_text;
use yew::virtual_dom::AttrValue;
use yew::{Component, Context, Html, Properties, html};

use crate::{Texts, Theme, ViewString, language::Language, text};

#[derive(PartialEq, Eq)]
pub enum Message {
    ClickItem(usize),
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
    #[prop_or(None)]
    pub parent_message: Option<T::Message>,
    pub list: Rc<Vec<ViewString>>,
    pub candidate_values: Rc<Vec<String>>,
    #[prop_or(None)]
    pub default_value: Option<AttrValue>,
    pub selected_value: Rc<RefCell<String>>,
    #[prop_or(None)]
    pub width_item: Option<u32>,
    #[prop_or(false)]
    pub allow_empty: bool,
    #[prop_or(None)]
    pub theme: Option<Theme>,
}

impl<T> Component for Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    type Message = Message;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let s = Self {
            phantom: PhantomData,
        };
        if let Some(value) = ctx.props().default_value.as_ref()
            && let Ok(mut selected) = ctx.props().selected_value.try_borrow_mut()
        {
            *selected = value.as_ref().into();
        }
        s
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let Message::ClickItem(index) = msg;
        if let Ok(mut selected) = ctx.props().selected_value.try_borrow_mut()
            && let Some(value) = ctx.props().candidate_values.get(index)
        {
            selected.clone_from(value);
        }
        if let (Some(msg), Some(parent)) =
            (ctx.props().parent_message.as_ref(), ctx.link().get_parent())
        {
            parent.clone().downcast::<T>().send_message(msg.clone());
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="radio-outer" role="radiogroup">
            {
                for ctx.props().list.iter().enumerate().map(|(index, item)| {
                    let checked = if let Ok(selected) = ctx.props().selected_value.try_borrow() {
                        ctx.props().candidate_values.get(index).is_some_and(|candidate| *selected == *candidate)
                    } else {
                        false
                    };
                    let theme = ctx.props().theme;
                    let ext = if cfg!(feature = "pumpkin") {
                        "svg"
                    } else {
                        "png"
                    };
                    let radio_img_file = if checked {
                        "radio-checked"
                    } else {
                        "radio-unchecked"
                    };
                    let radio_img = Theme::path(&theme, &format!("{radio_img_file}.{ext}"));
                    let txt = ctx.props().txt.txt.clone();
                    let onclick = |index: usize| ctx.link().callback(move |_| Message::ClickItem(index));
                    let style = ctx.props().width_item.map_or_else(String::new, |w| format!("width: {w}px;"));

                    html! {
                        <>
                            <div class="radio-item" role="radio" onclick={onclick(index)} style={style}>
                                <img src={radio_img} class="radio-img" />
                                {
                                    match item {
                                        ViewString::Key(key) => html! { text!(txt, ctx.props().language, key) },
                                        ViewString::Raw(txt) => html! { txt },
                                    }
                                }
                            </div>
                            {
                                if index < ctx.props().list.len() - 1 {
                                    html! {
                                        <div class="radio-space">
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </>
                    }
                })
            }
            </div>
        }
    }
}
