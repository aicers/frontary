use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};

use json_gettext::get_text;
use yew::{Component, Context, Html, Properties, html};

use crate::{Texts, Theme, ViewString, language::Language, text};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Message {
    ClickItem,
}

pub struct Model<T> {
    phantom: PhantomData<T>,
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
    #[prop_or(None)]
    pub theme: Option<Theme>,
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
        Self {
            phantom: PhantomData,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        if msg == Message::ClickItem
            && let Ok(mut selected_index) = ctx.props().selected_index.try_borrow_mut()
        {
            if *selected_index == Some(ctx.props().index) {
                *selected_index = None;
            } else {
                *selected_index = Some(ctx.props().index);
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
            (*selected_index).is_some_and(|s| s == ctx.props().index)
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
        let onclick = ctx.link().callback(move |_| Message::ClickItem);

        html! {
            <div class="radio-outer" role="radiogroup">
                <div role="radio" class="radio-item" onclick={onclick}>
                    <img src={radio_img} class="radio-separate-img" />
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
