use std::rc::Rc;
use std::str::FromStr;
use std::{cell::RefCell, marker::PhantomData};

use json_gettext::get_text;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent, MouseEvent};
use yew::{Callback, Component, Context, Html, Properties, events::InputEvent, html};

use crate::{Texts, language::Language, text};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Info {
    // starts at 1
    pub current: usize, // current page
    pub total: usize,   // total number of pages
    pub start: usize,
    pub end: usize,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            current: 1,
            total: 0,
            start: 1,
            end: 1,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Message {
    Previous,
    Next,
    First,
    Last,
    Page(usize),
    InputPage(String),
    GoToPage,
    InputError,
}

const NUM_PAGES: usize = 10;

#[derive(Clone, Properties, PartialEq)]
pub struct Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub txt: Texts,
    pub language: Language,
    pub parent_message: T::Message,
    pub pages_info: Rc<RefCell<Info>>,
    pub pages_info_cache: Option<Info>,
    #[prop_or(NUM_PAGES)]
    pub num_pages: usize, // # of pages that shows at the same time
    #[prop_or(true)]
    pub to_ends: bool,
    #[prop_or(false)]
    pub input: bool,
    #[prop_or(false)]
    pub disable: bool,
}

pub struct Model<T> {
    go_to_page: usize,
    phantom: PhantomData<T>,
}

fn page_button(class: &str, label: &str, onclick: Callback<MouseEvent>, disable: bool) -> Html {
    let adjust_class = if disable {
        format!("{class} disabled")
    } else {
        class.to_string()
    };
    html! {
        <div class={adjust_class} {onclick}>
            { label }
        </div>
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
            go_to_page: 0,
            phantom: PhantomData,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let num_pages = ctx.props().num_pages;
        if let Ok(mut info) = ctx.props().pages_info.try_borrow_mut() {
            if info.end + 1 - info.start < num_pages {
                if info.start + num_pages - 1 < info.total {
                    info.end = info.start + num_pages - 1;
                } else if info.end > num_pages && info.end + 1 - num_pages > 0 {
                    info.start = info.end + 1 - num_pages;
                }
            }
        }
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let num_pages = ctx.props().num_pages;
        let Ok(mut info) = ctx.props().pages_info.try_borrow_mut() else {
            return false;
        };
        match msg {
            Message::Next => {
                info.start += num_pages;
                info.end = std::cmp::min(info.end + num_pages, info.total);
                info.current = info.start;
            }
            Message::Previous => {
                info.end = info.end.saturating_sub(num_pages);
                info.start = if info.end >= num_pages {
                    info.end - num_pages + 1
                } else {
                    1
                };
                info.current = info.end;
            }
            Message::First => {
                info.start = 1;
                info.current = 1;
                info.end = std::cmp::min(num_pages, info.total);
            }
            Message::Last => {
                info.start = if info.total > num_pages {
                    let last_page_display_count = info.total % num_pages;
                    let last_page_display_count = if last_page_display_count == 0 {
                        num_pages
                    } else {
                        last_page_display_count
                    };
                    info.total + 1 - last_page_display_count
                } else {
                    1
                };
                info.current = info.total;
                info.end = info.total;
            }
            Message::Page(page) => {
                info.current = page;
            }
            Message::InputPage(text) => self.go_to_page = usize::from_str(&text).unwrap_or(0),
            Message::GoToPage => {
                if self.go_to_page >= 1 && self.go_to_page <= info.total {
                    info.current = self.go_to_page;
                    if info.total - info.current < num_pages - 1 {
                        if info.total <= num_pages {
                            info.start = 1;
                        } else {
                            info.start = info.total + 1 - num_pages;
                        }
                        info.end = info.total;
                    } else {
                        info.start = info.current;
                        info.end = std::cmp::min(info.start + num_pages - 1, info.total);
                    }
                }
            }
            Message::InputError => {
                // TODO: issue #5
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

    #[allow(clippy::too_many_lines)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Ok(info) = ctx.props().pages_info.try_borrow() {
            let onclick_prev = ctx.link().callback(|_| Message::Previous);
            let onclick_next = ctx.link().callback(|_| Message::Next);
            let onclick_first = ctx.link().callback(|_| Message::First);
            let onclick_last = ctx.link().callback(|_| Message::Last);
            let onclick_page = |page: usize| ctx.link().callback(move |_| Message::Page(page));

            html! {
                <div class="page-outer">
                    <div class="page-number">
                    {
                        if info.start > 1 {
                            html! {
                                <>
                                {
                                    if ctx.props().to_ends {
                                        page_button("page-unselected", "1", onclick_first, ctx.props().disable)
                                    } else {
                                        html! {}
                                    }
                                }
                                {
                                    if info.start > 2 {
                                        page_button("page-unselected", "•••", onclick_prev, ctx.props().disable)
                                    } else {
                                        html! {}
                                    }
                                }
                                </>
                            }
                        } else {
                            html! {}
                        }
                    }
                    {
                        for (info.start..=info.end).map(|page| {
                            let class = if page == info.current {
                                "page-selected"
                            } else {
                                "page-unselected"
                            };

                            page_button(class, &page.to_string(), onclick_page(page), ctx.props().disable)
                        })
                    }
                    {
                        if info.end < info.total {
                            html! {
                                <>
                                {
                                    if info.end + 1 < info.total {
                                        page_button("page-unselected", "•••", onclick_next, ctx.props().disable)
                                    } else {
                                        html! {}
                                    }
                                }
                                {
                                    if ctx.props().to_ends {
                                        page_button("page-unselected", &info.total.to_string(), onclick_last,
                                        ctx.props().disable)
                                    } else {
                                        html! {}
                                    }
                                }
                                </>
                            }
                        } else {
                            html! {}
                        }
                    }
                    </div>
                    {
                        if ctx.props().input {
                            let txt = ctx.props().txt.txt.clone();
                            let placeholder = text!(txt, ctx.props().language, "Page").to_string();
                            let oninput_page = ctx.link().callback(|e: InputEvent| {
                                e.target()
                                    .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                                    .map_or(Message::InputError, |input| {
                                        Message::InputPage(input.value())
                                    })
                            });
                            let onclick_page = ctx.link().callback(|_| Message::GoToPage);
                            let onkeyup_page = ctx.link().batch_callback(move |e: KeyboardEvent| {
                                (e.key() == "Enter").then_some(Message::GoToPage)
                            });

                            html! {
                                <>
                                    <table class="page-bar">
                                        <tr>
                                            <td class="page-bar">
                                                { text!(txt, ctx.props().language, "Go to") }
                                            </td>
                                        </tr>
                                    </table>
                                    <table class="page-go">
                                        <tr>
                                            <td class="page-go-input">
                                                <input type="text" class="page-input" placeholder={placeholder}
                                                    oninput={oninput_page}
                                                    onkeyup={onkeyup_page}
                                                />
                                            </td>
                                            <td class="page-go-icon" onclick={onclick_page}>
                                            </td>
                                        </tr>
                                    </table>
                                </>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            }
        } else {
            html! {}
        }
    }
}
