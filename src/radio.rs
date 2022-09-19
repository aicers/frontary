use crate::{texts, Texts, ViewString};
use json_gettext::get_text;
use language::{text, Language};
use std::cell::RefCell;
use std::rc::Rc;
use yew::{html, Component, Context, Html, Properties};

#[derive(PartialEq, Eq)]
pub enum Message {
    ClickItem(usize),
}

pub struct Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    _dummy: Option<T>,
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
    pub default_value: Option<String>,
    pub selected_value: Rc<RefCell<String>>,
    #[prop_or(None)]
    pub width_item: Option<u32>,
    #[prop_or(false)]
    pub allow_empty: bool,
}

impl<T> Component for Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    type Message = Message;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let s = Self { _dummy: None };
        if let Some(value) = ctx.props().default_value.as_ref() {
            if let Ok(mut selected) = ctx.props().selected_value.try_borrow_mut() {
                *selected = value.clone();
            }
        }
        s
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let Message::ClickItem(index) = msg;
        if let Ok(mut selected) = ctx.props().selected_value.try_borrow_mut() {
            if let Some(value) = ctx.props().candidate_values.get(index) {
                *selected = value.clone();
            }
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
            <div class="radio-outer">
            {
                for ctx.props().list.iter().enumerate().map(|(index, item)| {
                    let checked = if let Ok(selected) = ctx.props().selected_value.try_borrow() {
                        ctx.props().candidate_values.get(index).map_or(false, |candidate| *selected == *candidate)
                    } else {
                        false
                    };
                    let img = if checked {
                        "/img/radio-checked.png"
                    } else {
                        "/img/radio-unchecked.png"
                    };
                    let txt = texts(ctx).txt;
                    let onclick = |index: usize| ctx.link().callback(move |_| Message::ClickItem(index));
                    let style = ctx.props().width_item.map_or_else(String::new, |w| format!("width: {}px;", w));

                    html! {
                        <>
                            <div class="radio-item" onclick={onclick(index)} style={style}>
                                <img src={img} class="radio-img" />
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
