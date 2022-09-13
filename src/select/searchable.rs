use crate::{
    language::Language,
    text, toggle_visibility, {home_context, shorten_text, text_width, CheckBox, CheckStatus, Item},
};
use json_gettext::get_text;
use num_traits::ToPrimitive;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::{events::InputEvent, html, Component, Context, Html, Properties};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Single,
    Multi,
}

#[derive(Clone)]
pub struct Model<T>
where
    T: Clone + Component,
    <T as Component>::Message: Clone,
{
    _dummy: Option<T>,
    search_result: Option<Vec<usize>>,
    search_text: String,
}

#[derive(Clone)]
pub enum Message {
    Click,
    InputSearch(String),
    ClickAll,
    ClickItem(String),
    InputError,
}

const ELEM_HEIGHT: u32 = 32;
const DEFAULT_MAX_WIDTH: u32 = 500;

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub language: Language,
    pub id: String,
    pub kind: Kind,
    pub title: String,
    pub empty_msg: String,
    pub top_width: u32,
    #[prop_or(DEFAULT_MAX_WIDTH)]
    pub max_width: u32,
    pub max_height: u32,
    #[prop_or(true)]
    pub align_left: bool,
    pub font: String,
    pub list: Rc<RefCell<Vec<Item>>>,
    pub selected: Rc<RefCell<Option<HashSet<String>>>>,
    #[prop_or(false)]
    pub allow_empty: bool,
    #[prop_or(true)]
    pub default_all: bool,

    #[prop_or(None)]
    pub parent_message: Option<T::Message>,
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
            _dummy: None,
            search_result: None,
            search_text: String::new(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if ctx.props().kind == Kind::Multi {
            if let (Ok(mut sel), Ok(list)) = (
                ctx.props().selected.try_borrow_mut(),
                ctx.props().list.try_borrow(),
            ) {
                // if threre is any deleted item that belongs to the list of the selected
                if let Some(selected) = sel.as_mut() {
                    let list_tmp = list.iter().map(Item::key).collect::<HashSet<&String>>();
                    selected.retain(|k| list_tmp.get(k).is_some());
                    if !list.is_empty() && selected.len() == list.len() {
                        *sel = None;
                    }
                }
            }
        }

        true
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let txt = home_context(ctx).txt;
        let send_msg = match msg {
            Message::Click => {
                toggle_visibility(&ctx.props().id);
                false
            }
            Message::InputSearch(input) => {
                self.search_text = input.clone();
                if input.is_empty() {
                    self.search_result = None;
                } else if let Ok(list) = ctx.props().list.try_borrow() {
                    let input = input.to_lowercase();
                    self.search_result = Some(
                        list.iter()
                            .enumerate()
                            .filter_map(|(i, item)| {
                                if item
                                    .value(Some((txt.clone(), ctx.props().language)))
                                    .to_lowercase()
                                    .contains(&input)
                                {
                                    Some(i)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    );
                }
                false
            }
            Message::ClickItem(key) => {
                if let (Ok(mut sel), Ok(list)) = (
                    ctx.props().selected.try_borrow_mut(),
                    ctx.props().list.try_borrow(),
                ) {
                    #[allow(clippy::option_if_let_else)]
                    // can't use `map_or_else` since `*sel` is used in the closure
                    if let Some(selected) = sel.as_mut() {
                        match ctx.props().kind {
                            Kind::Multi => {
                                if selected.contains(&key) {
                                    selected.remove(&key);
                                } else {
                                    selected.insert(key);
                                }
                            }
                            Kind::Single => {
                                if !selected.is_empty() {
                                    selected.clear();
                                }
                                selected.insert(key);
                                ctx.link().send_message(Message::Click);
                            }
                        }
                    } else {
                        match ctx.props().kind {
                            Kind::Multi => {
                                let mut s = list
                                    .iter()
                                    .map(|i| i.key().clone())
                                    .collect::<HashSet<String>>();
                                s.remove(&key);
                                *sel = Some(s);
                            }
                            Kind::Single => {
                                *sel = None;
                            }
                        }
                    }
                    true
                } else {
                    false
                }
            }
            Message::ClickAll => {
                if ctx.props().kind == Kind::Multi {
                    if let (Ok(mut sel), Ok(list)) = (
                        ctx.props().selected.try_borrow_mut(),
                        ctx.props().list.try_borrow(),
                    ) {
                        if let Some(search_result) = self.search_result.as_ref() {
                            if !search_result.is_empty() {
                                if sel.is_none() {
                                    *sel = Some(
                                        list.iter()
                                            .map(|x| x.key().clone())
                                            .collect::<HashSet<String>>(),
                                    );
                                }
                                let selected = sel.as_mut().expect("ensured Some");
                                if search_result
                                    .iter()
                                    .filter_map(|&r| {
                                        if selected
                                            .contains(list.get(r).expect("should exist").key())
                                        {
                                            Some(true)
                                        } else {
                                            None
                                        }
                                    })
                                    .count()
                                    == search_result.len()
                                {
                                    for &r in search_result {
                                        selected.remove(list.get(r).expect("shoud exist").key());
                                    }
                                } else {
                                    for &r in search_result {
                                        let key = list.get(r).expect("should exist").key();
                                        if !selected.contains(key) {
                                            selected.insert(key.clone());
                                        }
                                    }
                                    if selected.len() == list.len() {
                                        *sel = None;
                                    }
                                }
                            }
                        } else if !list.is_empty() {
                            if let Some(selected) = sel.as_mut() {
                                if list.len() == selected.len() {
                                    selected.clear();
                                } else {
                                    *sel = None;
                                }
                            } else {
                                *sel = Some(HashSet::<String>::new());
                            }
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Message::InputError => {
                //TODO: issue #5
                false
            }
        };

        if send_msg {
            if let (Some(parent), Some(msg)) =
                (ctx.link().get_parent(), ctx.props().parent_message.as_ref())
            {
                parent.clone().downcast::<T>().send_message(msg.clone());
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = format!("width: {}px;", ctx.props().top_width);
        let onclick = ctx.link().callback(|_| Message::Click);
        let txt = home_context(ctx).txt;
        let mut class_input = "searchable-select-input";
        let value = if let (Ok(selected), Ok(list)) = (
            ctx.props().selected.try_borrow(),
            ctx.props().list.try_borrow(),
        ) {
            if list.is_empty() {
                class_input = "searchable-select-input-empty";
                text!(txt, ctx.props().language, "None").to_string()
            } else if let Some(selected) = selected.as_ref() {
                if selected.is_empty() {
                    if ctx.props().allow_empty {
                        class_input = "searchable-select-input-empty";
                    } else {
                        class_input = "searchable-select-input-empty-alert";
                    }
                    text!(txt, ctx.props().language, &ctx.props().empty_msg).to_string()
                } else {
                    match ctx.props().kind {
                        Kind::Multi => {
                            format!(
                                "({}) {}",
                                selected.len(),
                                text!(txt, ctx.props().language, &ctx.props().title)
                            )
                        }
                        Kind::Single => {
                            let key = selected.iter().map(Clone::clone).collect::<Vec<String>>();
                            let key = key.first();

                            let value = if let (Some(key), Ok(list)) =
                                (key, ctx.props().list.try_borrow())
                            {
                                let mut value = String::new();
                                for l in list.iter() {
                                    if l.key() == key {
                                        value = l.value(Some((txt, ctx.props().language)));
                                        break;
                                    }
                                }
                                value
                            } else {
                                String::new()
                            };

                            value
                        }
                    }
                }
            } else {
                text!(txt, ctx.props().language, "All").to_string()
            }
        } else {
            "searchable-select-input-empty".to_string()
        };
        html! {
            <div class="searchable-select">
                <div onclick={onclick} class="searchable-select-top">
                    <input type="text" class={class_input} disabled={true} value={value} style={style} />
                </div>
                { self.view_searchable_list(ctx) }
            </div>
        }
    }
}

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    fn caculate_width(ctx: &Context<Self>) -> u32 {
        let txt = home_context(ctx).txt;
        let sizes: Vec<u32> = ctx.props().list.try_borrow().map_or_else(
            |_| Vec::new(),
            |list| {
                list.iter()
                    .map(|item| {
                        text_width(
                            item.value(Some((txt.clone(), ctx.props().language)))
                                .as_str(),
                            &ctx.props().font,
                        )
                        .unwrap_or(0)
                    })
                    .collect()
            },
        );

        sizes
            .iter()
            .max()
            .map_or(ctx.props().top_width, |text_size| {
                let size = text_size + 34;
                if size <= ctx.props().top_width - 8 {
                    // 8 = 4(left shadow) + 4(right shadow)
                    ctx.props().top_width - 8
                } else {
                    std::cmp::min(size, ctx.props().max_width - 8)
                }
            })
    }

    #[allow(clippy::too_many_lines)]
    fn view_searchable_list(&self, ctx: &Context<Self>) -> Html {
        let width = Self::caculate_width(ctx);
        let list_len = ctx
            .props()
            .list
            .try_borrow()
            .map_or(0, |list| list.len())
            .to_u32()
            .expect("> u32::MAX never happens");
        let height = std::cmp::min(list_len * ELEM_HEIGHT + 80, ctx.props().max_height);

        let left = if width > ctx.props().top_width - 8 && !ctx.props().align_left {
            format!("-{}", width - (ctx.props().top_width - 8) + 4)
        } else {
            "4".to_string() // 4 is for left shadow
        };
        let style = format!(
            "width: {}px; height: {}px; left: {}px;",
            width, height, left,
        );
        let style_inner = format!("width: {}px; height: {}px;", width - 10, height);
        let style_inner_width = format!("width: {}px;", width - 10);
        let style_inner_width_search = format!("width: {}px", width - 10 - 28);
        let oninput_search = ctx.link().callback(|e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputSearch(input.value())
                })
        });
        let onclick_all = ctx.link().callback(|_| Message::ClickAll);
        let onclick_item = |key: String| {
            ctx.link()
                .callback(move |_| Message::ClickItem(key.clone()))
        };

        let txt = home_context(ctx).txt;
        let search_notice = text!(txt, ctx.props().language, "Search").to_string();
        let check_status = if let (Ok(selected), Ok(list)) = (
            ctx.props().selected.try_borrow(),
            ctx.props().list.try_borrow(),
        ) {
            self.search_result.as_ref().map_or_else(
                || {
                    selected.as_ref().map_or(CheckStatus::Checked, |selected| {
                        if selected.is_empty() {
                            CheckStatus::Unchecked
                        } else {
                            CheckStatus::Indeterminate
                        }
                    })
                },
                |search_result| {
                    selected.as_ref().map_or_else(
                        || {
                            if search_result.is_empty() {
                                CheckStatus::Unchecked
                            } else {
                                CheckStatus::Checked
                            }
                        },
                        |selected| {
                            let s_len = search_result
                                .iter()
                                .filter_map(|&r| {
                                    if selected.contains(list.get(r).expect("should exist").key()) {
                                        Some(true)
                                    } else {
                                        None
                                    }
                                })
                                .count();
                            if s_len == 0 {
                                CheckStatus::Unchecked
                            } else if s_len == search_result.len() {
                                CheckStatus::Checked
                            } else {
                                CheckStatus::Indeterminate
                            }
                        },
                    )
                },
            )
        } else {
            CheckStatus::Unchecked
        };

        if let Ok(list) = ctx.props().list.try_borrow() {
            if list.is_empty() {
                html! {}
            } else {
                html! {
                    <div id={ctx.props().id.clone()} class="searchable-select-list-down" style={style}>
                        <div style ={style_inner}>
                            <div class="searchable-select-list-search" style={style_inner_width.clone()}>
                                <input type="text" class="searchable-select-search"
                                    value={self.search_text.clone()}
                                    placeholder={search_notice}
                                    style={style_inner_width_search}
                                    oninput={oninput_search}
                                />
                            </div>
                            <div class="searchable-select-list-search-space" style={style_inner_width.clone()}>
                            </div>
                        {
                            if ctx.props().kind == Kind::Multi {
                                html! {
                                    <div class="searchable-select-list-search-all" style={style_inner_width.clone()}>
                                        <table>
                                            <tr>
                                                <td class="searchable-select-list-checkbox">
                                                    <div onclick={onclick_all}>
                                                        <CheckBox status={check_status} />
                                                    </div>
                                                </td>
                                                <td class="searchable-select-list-item">
                                                {
                                                    if self.search_result.is_none() {
                                                        html! { text!(txt, ctx.props().language, "All") }
                                                    } else {
                                                        html! { text!(txt, ctx.props().language, "All Search Results") }
                                                    }
                                                }
                                                </td>
                                            </tr>
                                        </table>
                                    </div>
                                }

                            } else {
                                html! {}
                            }
                        }
                            <table style={style_inner_width}>
                            {
                                if let Some(search_result) = self.search_result.as_ref() {
                                    html! {
                                        for search_result.iter().map(|&index| {
                                            let item = list.get(index).expect("should exist");
                                            let check_status = if let Ok(selected) = ctx.props().selected.try_borrow() {
                                                selected.as_ref().map_or(CheckStatus::Checked, |selected|
                                                    if selected.contains(item.key()) {
                                                        CheckStatus::Checked
                                                    } else {
                                                        CheckStatus::Unchecked
                                                    }
                                                )
                                            } else {
                                                CheckStatus::Unchecked
                                            };
                                            let sized_item_value = shorten_text(item.value(Some((txt.clone(), ctx.props().language))).as_str(), width, &ctx.props().font, 5);
                                            html! {
                                                <tr>
                                                    <td class="searchable-select-list-checkbox">
                                                        <div onclick={onclick_item(item.key().clone())}>
                                                            <CheckBox status={check_status} />
                                                        </div>
                                                    </td>
                                                    <td class="searchable-select-list-item">
                                                        { sized_item_value }
                                                    </td>
                                                </tr>
                                            }
                                        })
                                    }
                                } else {
                                    html! {
                                        for list.iter().map(|item| {
                                            let check_status = if let Ok(selected) = ctx.props().selected.try_borrow() {
                                                selected.as_ref().map_or(CheckStatus::Checked, |selected|
                                                    if selected.contains(item.key()) {
                                                        CheckStatus::Checked
                                                    } else {
                                                    CheckStatus::Unchecked
                                                })
                                            } else {
                                                CheckStatus::Unchecked
                                            };
                                            let sized_item_value = shorten_text(item.value(Some((txt.clone(), ctx.props().language))).as_str(), width, &ctx.props().font, 5);
                                            if ctx.props().kind == Kind::Multi {
                                                html! {
                                                    <tr>
                                                        <td class="searchable-select-list-checkbox">
                                                            <div onclick={onclick_item(item.key().clone())}>
                                                                <CheckBox status={check_status} />
                                                            </div>
                                                        </td>
                                                        <td class="searchable-select-list-item">
                                                            { sized_item_value }
                                                        </td>
                                                    </tr>
                                                }
                                            } else {
                                                html! {
                                                    <tr class="searchable-select-list-item-single" onclick={onclick_item(item.key().clone())}>
                                                        <td class="searchable-select-list-item-single">
                                                            { sized_item_value }
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        })
                                    }
                                }
                            }
                            </table>
                        </div>
                    </div>
                }
            }
        } else {
            html! {}
        }
    }
}
