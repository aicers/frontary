use crate::{language::Language, text, Texts, ViewString};
use json_gettext::get_text;
use std::cell::RefCell;
use std::rc::Rc;
use yew::{html, Component, Context, Html, Properties};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Message {
    ClickItem,
}

pub struct Model<T>
where
    T: Clone + Component,
    <T as Component>::Message: Clone,
{
    _dummy: Option<T>,
}

#[derive(Clone, Properties)]
pub struct Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub txt: Texts,
    pub language: Language,
    pub parent_message: T::Message,
    #[prop_or(None)]
    pub value: Option<ViewString>,
    pub index: usize,
    pub selected_index: Rc<RefCell<Option<usize>>>,
}

impl<T> PartialEq for Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<T> Component for Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    type Message = Message;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { _dummy: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        if msg == Message::ClickItem {
            if let Ok(mut selected_index) = ctx.props().selected_index.try_borrow_mut() {
                if *selected_index == Some(ctx.props().index) {
                    *selected_index = None;
                } else {
                    *selected_index = Some(ctx.props().index);
                }
            }
        }
        if let Some(parent) = ctx.link().get_parent() {
            parent
                .clone()
                .downcast::<T>()
                .send_message(ctx.props().parent_message.clone());
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let checked = if let Ok(selected_index) = ctx.props().selected_index.try_borrow() {
            (*selected_index).map_or(false, |s| s == ctx.props().index)
        } else {
            false
        };

        let img = if checked {
            "/frontary/radio-checked.png"
        } else {
            "/frontary/radio-unchecked.png"
        };

        let txt = ctx.props().txt.txt.clone();
        let onclick = ctx.link().callback(move |_| Message::ClickItem);

        html! {
            <div class="radio-outer">
                <div class="radio-item" onclick={onclick}>
                    <img src={img} class="radio-separate-img" />
                    {
                        match ctx.props().value.as_ref() {
                            Some(ViewString::Key(key)) => html! { text!(txt, ctx.props().language, key) },
                            Some(ViewString::Raw(raw)) => html! { raw },
                            None => html! {},
                        }
                    }
                </div>
            </div>
        }
    }
}
