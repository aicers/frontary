use std::collections::HashSet;
use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};

use json_gettext::get_text;
use num_traits::ToPrimitive;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::virtual_dom::AttrValue;
use yew::{Component, Context, Html, Properties, classes, events::InputEvent, html};

use crate::click_outside::toggle_visibility;
use crate::{
    CheckStatus, Checkbox, Item, Texts, Theme, language::Language, shorten_text, text, text_width,
};

#[cfg(feature = "pumpkin")]
const DEFAULT_FONT: &str = "";
#[cfg(not(feature = "pumpkin"))]
const DEFAULT_FONT: &str = "13px 'Spoqa Han Sans Neo'";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Single,
    Multi,
}

#[derive(Clone)]
pub struct Model<T> {
    search_result: Option<Vec<usize>>,
    search_text: String,
    phantom: PhantomData<T>,
}

#[derive(Clone)]
pub enum Message {
    Click,
    InputSearch(String),
    ClickAll,
    ClickItem(String),
    InputError,
}
#[cfg(feature = "pumpkin")]
const ELEM_HEIGHT: u32 = 48;
#[cfg(not(feature = "pumpkin"))]
const ELEM_HEIGHT: u32 = 32;
const DEFAULT_MAX_WIDTH: u32 = 500;
pub(super) const DEFAULT_SIZED_VALUE: bool = true;

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, PartialEq, Properties)]
pub struct Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub txt: Texts,
    pub language: Language,
    pub id: AttrValue,
    pub kind: Kind,
    pub title: AttrValue,
    pub empty_msg: AttrValue,
    pub top_width: u32,
    #[prop_or(DEFAULT_MAX_WIDTH)]
    pub max_width: u32,
    pub max_height: u32,
    #[prop_or(true)]
    pub align_left: bool,
    #[prop_or(DEFAULT_FONT.into())]
    pub font: AttrValue,
    pub list: Rc<RefCell<Vec<Item>>>,
    pub selected: Rc<RefCell<Option<HashSet<String>>>>,
    #[prop_or(false)]
    pub allow_empty: bool,
    #[prop_or(true)]
    pub default_all: bool,
    #[prop_or(DEFAULT_SIZED_VALUE)]
    pub sized_value: bool,
    #[prop_or(false)]
    pub is_required: bool,

    #[prop_or(None)]
    pub parent_message: Option<T::Message>,
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

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            search_result: None,
            search_text: String::new(),
            phantom: PhantomData,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if ctx.props().kind == Kind::Multi
            && let (Ok(mut sel), Ok(list)) = (
                ctx.props().selected.try_borrow_mut(),
                ctx.props().list.try_borrow(),
            )
        {
            // if threre is any deleted item that belongs to the list of the selected
            if let Some(selected) = sel.as_mut() {
                let list_tmp = list.iter().map(Item::id).collect::<HashSet<&String>>();
                selected.retain(|k| list_tmp.contains(k));
                if !list.is_empty() && selected.len() == list.len() {
                    *sel = None;
                }
            }
        }

        true
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let txt = ctx.props().txt.txt.clone();
        let send_msg = match msg {
            Message::Click => {
                let _ = toggle_visibility(&ctx.props().id);
                false
            }
            Message::InputSearch(input) => {
                self.search_text.clone_from(&input);
                if input.is_empty() {
                    self.search_result = None;
                } else if let Ok(list) = ctx.props().list.try_borrow() {
                    let input = input.to_lowercase();
                    self.search_result = Some(
                        list.iter()
                            .enumerate()
                            .filter_map(|(i, item)| {
                                if item
                                    .value_txt(&txt, ctx.props().language)
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
                let (Ok(mut sel), Ok(list)) = (
                    ctx.props().selected.try_borrow_mut(),
                    ctx.props().list.try_borrow(),
                ) else {
                    return false;
                };
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
                                .map(|i| i.id().clone())
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
            }
            Message::ClickAll => {
                if ctx.props().kind == Kind::Multi {
                    let (Ok(mut sel), Ok(list)) = (
                        ctx.props().selected.try_borrow_mut(),
                        ctx.props().list.try_borrow(),
                    ) else {
                        return false;
                    };
                    if let Some(search_result) = self.search_result.as_ref() {
                        if !search_result.is_empty() {
                            if sel.is_none() {
                                *sel = Some(
                                    list.iter()
                                        .map(|x| x.id().clone())
                                        .collect::<HashSet<String>>(),
                                );
                            }
                            let selected = sel.as_mut().expect("ensured Some");
                            if search_result
                                .iter()
                                .filter_map(|&r| {
                                    if selected.contains(list.get(r).expect("should exist").id()) {
                                        Some(true)
                                    } else {
                                        None
                                    }
                                })
                                .count()
                                == search_result.len()
                            {
                                for &r in search_result {
                                    selected.remove(list.get(r).expect("shoud exist").id());
                                }
                            } else {
                                for &r in search_result {
                                    let key = list.get(r).expect("should exist").id();
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
            }
            Message::InputError => {
                // TODO: issue #5
                false
            }
        };

        if send_msg
            && let (Some(parent), Some(msg)) =
                (ctx.link().get_parent(), ctx.props().parent_message.as_ref())
        {
            parent.clone().downcast::<T>().send_message(msg.clone());
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = if cfg!(feature = "pumpkin") {
            if ctx.props().id == "select-searchable-report" {
                format!("width: {}px;", ctx.props().top_width)
            } else {
                "width: 100%;".to_string()
            }
        } else {
            format!("width: {}px;", ctx.props().top_width)
        };
        let onclick = ctx.link().callback(|_| Message::Click);
        let txt = ctx.props().txt.txt.clone();
        let theme = ctx.props().theme;
        let mut class_input = "searchable-select-input";
        let value = if let (Ok(selected), Ok(list)) = (
            ctx.props().selected.try_borrow(),
            ctx.props().list.try_borrow(),
        ) {
            if list.is_empty() {
                class_input = if cfg!(feature = "pumpkin") {
                    "searchable-select-input-empty-disabled"
                } else {
                    "searchable-select-input-empty"
                };
                text!(txt, ctx.props().language, "None").to_string()
            } else if let Some(selected) = selected.as_ref() {
                if selected.is_empty() {
                    if ctx.props().is_required {
                        class_input = "searchable-select-input-empty-alert";
                    } else {
                        class_input = "searchable-select-input-empty";
                    }
                    text!(txt, ctx.props().language, &ctx.props().empty_msg).to_string()
                } else {
                    match ctx.props().kind {
                        Kind::Multi => {
                            if selected.len() == list.len() {
                                text!(txt, ctx.props().language, "All").to_string()
                            } else {
                                format!(
                                    "({}) {}",
                                    selected.len(),
                                    text!(txt, ctx.props().language, &ctx.props().title)
                                )
                            }
                        }
                        Kind::Single => {
                            let key = selected.iter().next().cloned();

                            if let (Some(key), Ok(list)) = (key, ctx.props().list.try_borrow()) {
                                let mut value = String::new();
                                for l in list.iter() {
                                    if l.id() == &key {
                                        value = l.value_txt(&txt, ctx.props().language);
                                        break;
                                    }
                                }
                                value
                            } else {
                                String::new()
                            }
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
                    <input type="text" class={classes!("searchable-select-top-input", class_input)} readonly={true} value={value.clone()} style={style} />
                </div>
                { self.view_searchable_list(ctx, &value, theme) }
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
        let txt = ctx.props().txt.txt.clone();
        let max_size = ctx.props().list.try_borrow().ok().and_then(|list| {
            list.iter()
                .map(|item| {
                    text_width(
                        item.value_txt(&txt, ctx.props().language).as_str(),
                        &ctx.props().font,
                    )
                    .unwrap_or(0)
                })
                .max()
        });

        max_size.map_or(ctx.props().top_width, |text_size| {
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
    fn view_searchable_list(&self, ctx: &Context<Self>, value: &str, theme: Option<Theme>) -> Html {
        let width = Self::caculate_width(ctx);
        let list_len = ctx
            .props()
            .list
            .try_borrow()
            .map_or(0, |list| list.len())
            .to_u32()
            .expect("> u32::MAX never happens");
        let extra_height = if ctx.props().kind == Kind::Single {
            if cfg!(feature = "pumpkin") { 67 } else { 42 }
        } else if cfg!(feature = "pumpkin") {
            60
        } else {
            80
        };
        let height = if cfg!(feature = "pumpkin") {
            if ctx.props().kind == Kind::Single {
                std::cmp::min(
                    list_len * ELEM_HEIGHT + extra_height,
                    ctx.props().max_height,
                )
            } else {
                std::cmp::min(
                    (list_len + 1) * ELEM_HEIGHT + extra_height,
                    ctx.props().max_height,
                )
            }
        } else {
            std::cmp::min(
                list_len * ELEM_HEIGHT + extra_height,
                ctx.props().max_height,
            )
        };

        let max_height = 6 * ELEM_HEIGHT + extra_height;
        let left = if width > ctx.props().top_width - 8 && !ctx.props().align_left {
            format!("-{}", width - (ctx.props().top_width - 8) + 4)
        } else {
            "4".to_string() // 4 is for left shadow
        };

        let (
            style,
            style_inner,
            style_inner_width,
            style_inner_width_search,
            style_scrollable_table,
        ) = if cfg!(feature = "pumpkin") {
            if list_len > 5 {
                (
                    format!("height: {height}px; max-height: {max_height}px;"),
                    format!(
                        "width: 100%; max-height: {}px; height: {}px;",
                        max_height - 6,
                        height - 6
                    ),
                    "width: 100%;".to_string(),
                    "width: 100%;".to_string(),
                    format!(
                        "height: {}px; overflow-y: scroll;",
                        std::cmp::min(6 * ELEM_HEIGHT + 6, height - (ELEM_HEIGHT + 10))
                    ),
                )
            } else {
                (
                    format!("height: {height}px; min-height: fit-content;"),
                    format!(
                        "width: 100%; min-height: fit-content; height: {}px;",
                        height - 6
                    ),
                    "width: 100%;".to_string(),
                    "width: 100%;".to_string(),
                    "min-height: fit-content;".to_string(),
                )
            }
        } else {
            (
                format!("width: {width}px; height: {height}px; left: {left}px;"),
                format!("width: {}px; height: {height}px;", width - 10),
                format!("width: {}px;", width - 10),
                format!("width: {}px", width - 10 - 28),
                String::new(),
            )
        };
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

        let txt = ctx.props().txt.txt.clone();
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
                        } else if selected.len() == list.len() {
                            CheckStatus::Checked
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
                                    if selected.contains(list.get(r).expect("should exist").id()) {
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

        let Ok(list) = ctx.props().list.try_borrow() else {
            return html! {};
        };
        if list.is_empty() {
            return html! {};
        }
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
                    <div class="scrollable-table-wrapper" style={style_scrollable_table}>
                    {
                        if ctx.props().kind == Kind::Multi {
                            html! {
                                <div class="searchable-select-list-search-all" style={style_inner_width.clone()}>
                                    <table>
                                        <tr>
                                            <td class="searchable-select-list-checkbox">
                                                <div onclick={onclick_all}>
                                                    <Checkbox status={check_status} {theme}/>
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
                                            if selected.contains(item.id()) {
                                                CheckStatus::Checked
                                            } else {
                                                CheckStatus::Unchecked
                                            }
                                        )
                                    } else {
                                        CheckStatus::Unchecked
                                    };
                                    let mut item_value = item.value_txt(&txt, ctx.props().language);
                                    if  ctx.props().sized_value { item_value = shorten_text(item.value_txt(&txt, ctx.props().language).as_str(), width, &ctx.props().font, 5); }
                                    html! {
                                        <tr>
                                            <td class="searchable-select-list-checkbox">
                                                <div onclick={onclick_item(item.id().clone())}>
                                                    <Checkbox status={check_status} {theme}/>
                                                </div>
                                            </td>
                                            <td class="searchable-select-list-item">
                                                { item_value }
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
                                            if selected.contains(item.id()) {
                                                CheckStatus::Checked
                                            } else {
                                            CheckStatus::Unchecked
                                        })
                                    } else {
                                        CheckStatus::Unchecked
                                    };
                                    let mut item_value = item.value_txt(&txt, ctx.props().language);
                                    if ctx.props().sized_value { item_value = shorten_text(item.value_txt(&txt, ctx.props().language).as_str(), width, &ctx.props().font, 5); }
                                    if ctx.props().kind == Kind::Multi {
                                        html! {
                                            <tr>
                                                <td class="searchable-select-list-checkbox">
                                                    <div onclick={onclick_item(item.id().clone())}>
                                                        <Checkbox status={check_status} {theme} />
                                                    </div>
                                                </td>
                                                <td class="searchable-select-list-item">
                                                    { item_value }
                                                </td>
                                            </tr>
                                        }
                                    } else if item.value_txt(&txt, ctx.props().language) == value {
                                        html!{
                                            <tr class="searchable-select-list-item-single" onclick={onclick_item(item.id().clone())}>
                                                <td class="searchable-select-list-item-single-selected">
                                                    { item_value }
                                                </td>
                                            </tr>
                                        }
                                    }
                                    else {
                                        html! {
                                            <tr class="searchable-select-list-item-single" onclick={onclick_item(item.id().clone())}>
                                                <td class="searchable-select-list-item-single">
                                                    { item_value }
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
            </div>
        }
    }
}
