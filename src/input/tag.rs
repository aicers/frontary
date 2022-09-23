use crate::{
    texts, toggle_visibility, visibile_tag_select, {InputTagGroup, Texts},
};
use json_gettext::get_text;
use language::{text, Language};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::{events::InputEvent, html, Component, Context, Html, Properties, TargetCast};

pub struct Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    _dummy: Option<T>,
    prev_list: Rc<HashMap<String, String>>,
    input: String,
    message: Option<&'static str>,
    view_order: Vec<String>, // String = key of tag
    search_list: Vec<(String, String)>,
    search_cursor: Option<usize>,
    edit: Option<String>, // String = key of tag
    edit_message: Option<&'static str>,
    input_edit: String,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Message {
    Focus,
    Input(String),
    InputEdit(String),
    Enter,
    SelectTag(String),   // key
    UnselectTag(String), // key
    EditTag(String),     // key
    DeleteTag(String),   // key
    Keyboard(String),
    CancelEdit,
    EditDone,
    InputError,
}

const DEFAULT_MAX_HEIGHT: u32 = 280;
const EXIST_MSG: &str = "The input already exists.";
const ID: &str = "tag-group-input-select";

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
    pub prev_list: Rc<HashMap<String, String>>,
    pub input_data: Rc<RefCell<InputTagGroup>>,
    #[prop_or(None)]
    pub input_notice: Option<&'static str>,
    #[prop_or(None)]
    pub width: Option<u32>,
    #[prop_or(DEFAULT_MAX_HEIGHT)]
    pub max_height: u32,
}

impl<T> Component for Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    type Message = Message;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let mut s = Self {
            _dummy: None,
            prev_list: ctx.props().prev_list.clone(),
            input: String::new(),
            message: None,
            view_order: Vec::new(),
            search_list: Vec::new(),
            search_cursor: None,
            edit: None,
            edit_message: None,
            input_edit: String::new(),
        };
        s.init_view_order(ctx);
        s.reset_search_list(ctx);
        s
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if self.prev_list != ctx.props().prev_list {
            for old_key in self.prev_list.keys() {
                if !ctx.props().prev_list.contains_key(old_key) {
                    self.view_order.retain(|v| v != old_key);
                }
            }
            for new_key in ctx.props().prev_list.keys() {
                if !self.prev_list.contains_key(new_key) {
                    self.view_order.push(new_key.clone());
                }
            }
            self.prev_list = ctx.props().prev_list.clone();
        }

        self.reset_search_list(ctx);
        true
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Focus => visibile_tag_select(ID),
            Message::Input(input) => {
                self.input = input;
                self.reset_search_list(ctx);
                self.message = None;
            }
            Message::Enter => {
                if !self.input.is_empty() {
                    let send_msg = if let Ok(mut data) = ctx.props().input_data.try_borrow_mut() {
                        if let Some((key, _)) = ctx
                            .props()
                            .prev_list
                            .iter()
                            .find(|(_, v)| *v == &self.input)
                        {
                            if data.old.contains(key) {
                                self.message = Some(EXIST_MSG);
                                false
                            } else {
                                data.old.insert(key.clone());
                                self.view_order.push(key.clone());
                                self.input = String::new();
                                true
                            }
                        } else {
                            data.new = Some(self.input.clone());
                            self.input = String::new();
                            true
                        }
                    } else {
                        false
                    };
                    self.reset_search_list(ctx);
                    if send_msg {
                        if let (Some(parent), Some(msg)) =
                            (ctx.link().get_parent(), ctx.props().parent_message.as_ref())
                        {
                            parent.clone().downcast::<T>().send_message(msg.clone());
                        }
                    }
                }
            }
            Message::Keyboard(keyboard) => match keyboard.as_str() {
                "ArrowUp" => {
                    if let Some(index) = self.search_cursor {
                        match index.cmp(&0) {
                            Ordering::Greater => self.search_cursor = Some(index - 1),
                            Ordering::Equal => {
                                self.search_cursor = Some(self.search_list.len() - 1);
                            }
                            Ordering::Less => (),
                        }
                    }
                }
                "ArrowDown" => {
                    if let Some(index) = self.search_cursor {
                        if index == self.search_list.len() - 1 {
                            self.search_cursor = Some(0);
                        } else {
                            self.search_cursor = Some(index + 1);
                        }
                    }
                }
                "Tab" => {
                    if let Some(index) = self.search_cursor {
                        if let Some((k, _)) = self.search_list.get(index) {
                            ctx.link().send_message(Message::SelectTag(k.clone()));
                            return false;
                        }
                    }
                }
                "Backspace" => {
                    if self.input.is_empty() {
                        if let Some(last) = self.view_order.last() {
                            ctx.link().send_message(Message::UnselectTag(last.clone()));
                        }
                    }
                    return false;
                }
                _ => {}
            },
            Message::SelectTag(key) => {
                let send_msg = if let Ok(mut data) = ctx.props().input_data.try_borrow_mut() {
                    if data.old.contains(&key) {
                        self.message = Some(EXIST_MSG);
                        false
                    } else {
                        data.old.insert(key.clone());
                        self.view_order.push(key);
                        self.input = String::new();
                        toggle_visibility(ID);
                        true
                    }
                } else {
                    false
                };
                self.reset_search_list(ctx);
                if send_msg {
                    if let (Some(parent), Some(msg)) =
                        (ctx.link().get_parent(), ctx.props().parent_message.as_ref())
                    {
                        parent.clone().downcast::<T>().send_message(msg.clone());
                    }
                }
            }
            Message::UnselectTag(key) => {
                if let Ok(mut data) = ctx.props().input_data.try_borrow_mut() {
                    data.old.remove(&key);
                    self.view_order.retain(|k| k != &key);
                }
                self.reset_search_list(ctx);
            }
            Message::EditTag(key) => {
                self.edit = Some(key);
                self.edit_message = None;
            }
            Message::DeleteTag(key) => {
                if let Ok(mut data) = ctx.props().input_data.try_borrow_mut() {
                    data.delete = Some(key);
                }
                // AICE TODO: revive this if necessary
                // self.reset_search_list(ctx);
                if let (Some(parent), Some(msg)) =
                    (ctx.link().get_parent(), ctx.props().parent_message.as_ref())
                {
                    parent.clone().downcast::<T>().send_message(msg.clone());
                }
            }
            Message::InputEdit(input) => {
                self.input_edit = input;
            }
            Message::EditDone => {
                let send_msg = if self.input_edit.is_empty() {
                    false
                } else if let (Ok(mut input), Some(key)) =
                    (ctx.props().input_data.try_borrow_mut(), self.edit.as_ref())
                {
                    if ctx
                        .props()
                        .prev_list
                        .iter()
                        .any(|(_, v)| v == &self.input_edit)
                    {
                        self.edit_message = Some(EXIST_MSG);
                        false
                    } else {
                        self.edit_message = None;
                        input.edit = Some((key.clone(), self.input_edit.clone()));
                        true
                    }
                } else {
                    false
                };
                if send_msg {
                    if let (Some(parent), Some(msg)) =
                        (ctx.link().get_parent(), ctx.props().parent_message.as_ref())
                    {
                        parent.clone().downcast::<T>().send_message(msg.clone());
                    }
                    self.input_edit = String::new();
                    self.edit = None;
                }
            }
            Message::CancelEdit => {
                self.input_edit = String::new();
                self.edit = None;
            }
            Message::InputError => {
                //TODO: issue #5
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = format!("max-height: {}px;", ctx.props().max_height);
        html! {
            <div class="tag-group-input-outer">
                <div class="tag-group-input" style={style}>
                    { self.view_tag_group(ctx) }
                    { self.view_input(ctx) }
                </div>
                { self.view_message(ctx) }
            </div>
        }
    }
}

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    fn view_tag_group(&self, ctx: &Context<Self>) -> Html {
        if let Ok(data) = ctx.props().input_data.try_borrow() {
            html! {
                for self.view_order.iter().map(|key| {
                    if data.old.contains(key) {
                        Self::view_item(ctx, key, ctx.props().prev_list.get(key).unwrap_or(key))
                    } else {
                        html! {}
                    }
                })
            }
        } else {
            html! {}
        }
    }

    fn view_item(ctx: &Context<Self>, key: &str, tag: &str) -> Html {
        let onclick_unselect = |key: String| {
            ctx.link()
                .callback(move |_| Message::UnselectTag(key.clone()))
        };
        html! {
            <div class="tag-group-input-item">
                { tag }
                <img src="/img/tag-input-close.png" class="tag-input-close" onclick={onclick_unselect(key.to_string())} />
            </div>
        }
    }

    fn view_input(&self, ctx: &Context<Self>) -> Html {
        let txt = texts(ctx).txt;

        let placeholder = if let (Ok(data), Some(notice)) = (
            ctx.props().input_data.try_borrow(),
            ctx.props().input_notice,
        ) {
            if data.old.is_empty() {
                text!(txt, ctx.props().language, notice).to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let oninput = ctx.link().callback(|e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| Message::Input(input.value()))
        });
        let onkeyup = ctx
            .link()
            .batch_callback(move |e: KeyboardEvent| (e.key() == "Enter").then_some(Message::Enter));
        let onkeydown = ctx.link().batch_callback(move |e: KeyboardEvent| {
            (e.key() == "Backspace"
                || e.key() == "Tab"
                || e.key() == "ArrowUp"
                || e.key() == "ArrowDown")
                .then(|| {
                    let input: HtmlInputElement = e.target_unchecked_into();
                    let value = input.value();
                    input.set_value("");
                    Message::Keyboard(value)
                })
        });
        let onfocus = ctx.link().callback(|_| Message::Focus);

        html! {
            <div class="tag-group-input-outer">
                <input type="text"
                    class="tag-group-input-input-input"
                    value={self.input.clone()}
                    placeholder={placeholder}
                    oninput={oninput}
                    onkeyup={onkeyup}
                    onkeydown={onkeydown}
                    onfocus={onfocus}
                />
                { self.view_select(ctx) }
            </div>
        }
    }

    fn view_select(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id={ID.to_string()} class="tag-group-input-select">
            {
                for self.search_list.iter().enumerate().map(|(index, (k, v))| {
                    if self.edit.as_ref().map_or(false, |t| t == k) {
                        let oninput = ctx.link().callback(|e: InputEvent| {
                            e.target()
                                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                                .map_or(Message::InputError, |input| Message::InputEdit(input.value()))
                        });
                        let onkeyup = ctx
                            .link()
                            .batch_callback(move |e: KeyboardEvent| (e.key() == "Enter").then_some(Message::EditDone));
                        let onclick_cancel_edit = ctx.link().callback(|_| Message::CancelEdit);
                        let onclick_edit_done = ctx.link().callback(move |_| Message::EditDone);
                        let done_img = if self.input_edit.is_empty() {
                            "/img/tag-select-edit-done-dim.png"
                        } else {
                            "/img/tag-select-edit-done.png"
                        };
                        html! {
                            <div class="tag-group-input-select-item-outer-edit">
                                <div class="tag-group-input-select-item-edit">
                                    <div class="tag-group-input-select-item-edit-text">
                                        <input type="text"
                                            class="tag-select-edit-tag"
                                            placeholder={v.clone()}
                                            oninput={oninput}
                                            onkeyup={onkeyup}
                                        />
                                    </div>
                                    <div class="tag-group-input-select-item-edit-img">
                                        <img src="/img/tag-select-bar.png" class="tag-select-bar" />
                                        <img src={done_img} class="tag-select-edit-done" onclick={onclick_edit_done} />
                                        <img src="/img/tag-input-close.png" class="tag-input-close" onclick={onclick_cancel_edit} />
                                    </div>
                                </div>
                                {
                                    if let Some(msg) = self.edit_message {
                                        let txt = texts(ctx).txt;
                                        html! {
                                            <div class="tag-edit-message">
                                                { text!(txt, ctx.props().language, msg) }
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        }
                    } else {
                        let class = if self.search_cursor.map_or(false, |c| c == index) {
                            "tag-group-input-select-item-outer-cursor"
                        } else {
                            "tag-group-input-select-item-outer"
                        };
                        let onclick_item = |key: String| {
                            ctx.link()
                                .callback(move |_| Message::SelectTag(key.clone()))
                        };
                        let onclick_edit = |key: String| {
                            ctx.link()
                                .callback(move |_| Message::EditTag(key.clone()))
                        };
                        let onclick_delete = |key: String| {
                            ctx.link()
                                .callback(move |_| Message::DeleteTag(key.clone()))
                        };
                        html! {
                            <div class={class}>
                                <div class="tag-group-input-select-item">
                                    <div class="tag-group-input-select-item-text" onclick={ onclick_item(k.clone()) }>
                                    { v.clone() }
                                    </div>
                                    <img src="/img/tag-select-bar.png" class="tag-select-bar" />
                                    <img src="/img/tag-select-edit.png" class="tag-select-edit" onclick={ onclick_edit(k.clone()) } />
                                    <img src="/img/tag-select-trash.png" class="tag-select-trash" onclick={ onclick_delete(k.clone()) } />
                                </div>
                            </div>
                        }
                    }
                })
            }
            </div>
        }
    }

    fn view_message(&self, ctx: &Context<Self>) -> Html {
        let txt = texts(ctx).txt;
        if let Some(message) = self.message {
            html! {
                <div class="tag-group-message-text">
                    { text!(txt, ctx.props().language, message) }
                </div>
            }
        } else {
            html! {}
        }
    }

    fn reset_search_list(&mut self, ctx: &Context<Self>) {
        self.search_list = ctx
            .props()
            .prev_list
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(String, String)>>();
        if let Ok(data) = ctx.props().input_data.try_borrow() {
            self.search_list.retain(|(k, _)| data.old.get(k).is_none());
        }
        if !self.input.is_empty() {
            let input = self.input.to_lowercase();
            self.search_list
                .retain(|(_, v)| v.to_lowercase().contains(&input));
        }
        self.search_list.sort_unstable_by(|a, b| a.1.cmp(&b.1));
        self.search_cursor = Some(0);
    }

    fn init_view_order(&mut self, ctx: &Context<Self>) {
        if let Ok(data) = ctx.props().input_data.try_borrow() {
            let mut old = data.old.iter().map(Clone::clone).collect::<Vec<String>>();
            old.sort_unstable();
            self.view_order = old;
        }
    }
}
