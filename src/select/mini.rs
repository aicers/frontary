use crate::{toggle_visibility, Texts, ViewString};
use gloo_events::EventListener;
use json_gettext::get_text;
use language::{text, Language};
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{Event, HtmlElement};
use yew::{classes, html, Component, Context, Html, NodeRef, Properties};

pub struct Model<T, U>
where
    T: Copy + Clone + PartialEq + 'static,
    U: Clone + Component + PartialEq,
    <U as Component>::Message: Clone + PartialEq,
{
    _dummy: Option<(T, U)>,
    click_listener: Option<EventListener>,
    click_count: usize,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    DirectionAll,
    DirectionItem,
    MoreAction,
    MoreActionNoImage,
    SortList,
    Round,
    Soft, // dead code if ai-model is not included
}

pub enum Message {
    ClickTop,
    ClickItem(usize),
    ListenClick,
}

const DEFAULT_BG_COLOR: &str = "#EAEAEA";

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T, U>
where
    T: Copy + Clone + PartialEq + 'static,
    U: Clone + Component + PartialEq,
    <U as Component>::Message: Clone + PartialEq,
{
    pub txt: Texts,
    pub language: Language,
    #[prop_or(None)]
    pub home_div: Option<NodeRef>,
    pub parent_message: U::Message,
    #[prop_or(None)]
    pub parent_cancel_message: Option<U::Message>,
    #[prop_or(true)]
    pub active: bool,
    #[prop_or(None)]
    pub deactive_class_suffix: Option<String>,
    pub id: String,
    pub list: Rc<Vec<ViewString>>,
    pub candidate_values: Rc<Vec<T>>,
    #[prop_or(None)]
    pub default_value: Option<T>,
    // HIGHTLIGHT:
    // `selected_value` can be altered by parents.
    // When this is the case, `changed` is not called because `Properties` don't change.
    // `selected_value_cache` comes in to call `changed` by forcing `Properties` to change.
    pub selected_value: Rc<RefCell<Option<T>>>,
    pub selected_value_cache: Option<T>,
    #[prop_or(None)]
    pub top_width: Option<u32>,
    #[prop_or(None)]
    pub top_height: Option<u32>,
    #[prop_or(None)]
    pub list_min_width: Option<u32>,
    #[prop_or(true)]
    pub align_left: bool,
    pub list_top: u32,
    #[prop_or(false)]
    pub list_align_center: bool,
    pub kind: Kind,
    #[prop_or(DEFAULT_BG_COLOR.to_string())]
    pub top_bg_color: String,
}

impl<T, U> Component for Model<T, U>
where
    T: Copy + Clone + PartialEq + 'static,
    U: Clone + Component + PartialEq,
    <U as Component>::Message: Clone + PartialEq,
{
    type Message = Message;
    type Properties = Props<T, U>;

    fn create(ctx: &Context<Self>) -> Self {
        let s = Self {
            _dummy: None,
            click_listener: None,
            click_count: 0,
        };
        if let Some(value) = ctx.props().default_value {
            if let Ok(mut selected) = ctx.props().selected_value.try_borrow_mut() {
                if selected.is_none() {
                    *selected = Some(value);
                }
            }
        }
        s
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }
        if let Some(home_div) = ctx.props().home_div.as_ref() {
            if let Some(element) = home_div.cast::<HtmlElement>() {
                let callback = ctx.link().callback(|_: Event| Message::ListenClick);
                let listener =
                    EventListener::new(&element, "click", move |e| callback.emit(e.clone()));
                self.click_listener = Some(listener);
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let parent_msg = match msg {
            Message::ClickTop => {
                if ctx.props().active {
                    toggle_visibility(&ctx.props().id);
                    ctx.props().parent_message.clone()
                } else {
                    return false;
                }
            }
            Message::ClickItem(index) => {
                if let Ok(mut selected) = ctx.props().selected_value.try_borrow_mut() {
                    *selected = ctx.props().candidate_values.get(index).copied();
                }
                toggle_visibility(&ctx.props().id);
                ctx.props().parent_message.clone()
            }
            Message::ListenClick => {
                self.click_count += 1;
                if self.click_count > 1 {
                    ctx.props()
                        .parent_cancel_message
                        .as_ref()
                        .map_or(ctx.props().parent_message.clone(), Clone::clone)
                } else {
                    return false;
                }
            }
        };
        if let Some(parent) = ctx.link().get_parent() {
            parent.clone().downcast::<U>().send_message(parent_msg);
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="mini-select">
            {
                match ctx.props().kind {
                    Kind::DirectionAll => Self::view_direction_all(ctx),
                    Kind::DirectionItem => Self::view_direction_item(ctx),
                    Kind::SortList => Self::view_sort_list(ctx),
                    Kind::MoreAction => Self::view_more_action(ctx),
                    Kind::MoreActionNoImage => html! {},
                    Kind::Round | Kind::Soft => Self::view_basic(ctx),
                }
            }
            { Self::view_list(ctx) }
            </div>
        }
    }
}

impl<T, U> Model<T, U>
where
    T: Copy + Clone + PartialEq + 'static,
    U: Clone + Component + PartialEq,
    <U as Component>::Message: Clone + PartialEq,
{
    fn view_list(ctx: &Context<Self>) -> Html {
        let list = ctx.props().list.clone();
        let onclick_item = |index: usize| ctx.link().callback(move |_| Message::ClickItem(index));
        let style_width = ctx
            .props()
            .list_min_width
            .map_or_else(String::new, |w| format!("min-width: {}px;", w));
        let align = if ctx.props().align_left {
            "left"
        } else {
            "right"
        };
        let style = format!(
            "{}: 0px; top: {}px; {}",
            align,
            ctx.props().list_top,
            &style_width
        );
        let class = if ctx.props().kind == Kind::MoreActionNoImage {
            "mini-select-list-down-visible"
        } else {
            "mini-select-list-down-hidden"
        };
        let class_list_align = if ctx.props().list_align_center {
            "mini-select-list-down-item-center"
        } else {
            "mini-select-list-down-item-left"
        };
        let class_list_align_more_action = if ctx.props().list_align_center {
            "mini-select-list-down-item-more-action-text-center"
        } else {
            "mini-select-list-down-item-more-action-text-left"
        };
        let txt = ctx.props().txt.txt.clone();
        html! {
            <div id={ctx.props().id.clone()} class={classes!("mini-select-list-down", class)} style={style}>
                <table class="mini-select-list-down-table">
                    {
                        for list.iter().enumerate().map(|(index, item)| html! {
                            <tr>
                                <td class={classes!("mini-select-list-down-item", class_list_align)} onclick={onclick_item(index)} style={style_width.clone()}>
                                {
                                    if ctx.props().kind == Kind::MoreAction {
                                        match item {
                                            ViewString::Key(key) => {
                                                let icon = if key == "Edit" {
                                                    "/img/edit.png"
                                                } else if key == "Delete" {
                                                    "/img/delete-trash.png"
                                                } else {
                                                    ""
                                                };
                                                html! {
                                                    <div class="mini-select-list-down-item-more-action">
                                                        <img src={icon} class="mini-select-list-down-item-more-action" />
                                                        <div class={classes!("mini-select-list-down-item-more-action-text", class_list_align_more_action)}>
                                                        { text!(txt, ctx.props().language, key) }
                                                        </div>
                                                    </div>
                                                }
                                            }
                                            ViewString::Raw(_) => html! {}
                                        }
                                    } else {
                                        match item {
                                            ViewString::Key(key) => html! { text!(txt, ctx.props().language, key) },
                                            ViewString::Raw(txt) => html! { txt },
                                        }
                                    }
                                }
                                </td>
                            </tr>
                        })
                    }
                </table>
            </div>
        }
    }

    fn view_basic(ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let msg = text!(txt, ctx.props().language, "Select one").to_string();
        let value = if let Ok(selected) = ctx.props().selected_value.try_borrow() {
            selected.map_or(msg.clone(), |value| {
                ctx.props()
                    .candidate_values
                    .iter()
                    .enumerate()
                    .find(|(_, &v)| v == value)
                    .map_or(msg.clone(), |(index, _)| {
                        ctx.props()
                            .list
                            .get(index)
                            .map_or(msg.clone(), |v| match v {
                                ViewString::Key(key) => {
                                    text!(txt, ctx.props().language, key).to_string()
                                }
                                ViewString::Raw(txt) => txt.clone(),
                            })
                    })
            })
        } else {
            msg
        };
        let style_width = ctx
            .props()
            .top_width
            .map_or_else(String::new, |w| format!("width: {}px;", w));
        let style_height = ctx
            .props()
            .top_height
            .map_or_else(String::new, |h| format!("height: {}px;", h));
        let style = if ctx.props().kind == Kind::Round {
            format!(
                "{} {} background-color: {};",
                style_width,
                style_height,
                &ctx.props().top_bg_color
            )
        } else {
            // `Kind::Soft` only
            format!(
                "{} {} background-color: #FFFFFF;",
                style_width, style_height
            )
        };
        let (outer_sub_class, icon_sub_class) = if ctx.props().kind == Kind::Round {
            ("mini-select-basic-basic", "mini-select-basic-icon-basic")
        } else {
            ("mini-select-basic-square", "mini-select-basic-icon-square")
        };
        let onclick = ctx.link().callback(|_| Message::ClickTop);

        html! {
            <div onclick={onclick} class={classes!("mini-select-basic", outer_sub_class)} style={style}>
                <div class="mini-select-basic-value" style={style_height.clone()}>
                    { value }
                </div>
                <div class={classes!("mini-select-basic-icon", icon_sub_class)} style={style_height}>
                </div>
            </div>
        }
    }

    fn view_direction_all(ctx: &Context<Self>) -> Html {
        let suffix = if ctx.props().active {
            String::new()
        } else if let Some(suffix) = ctx.props().deactive_class_suffix.as_ref() {
            suffix.clone()
        } else {
            String::new()
        };
        let (class, class_text, class_icon) = (
            format!("mini-select-top-direction{}", suffix),
            format!("mini-select-top-direction-text{}", suffix),
            format!("mini-select-top-direction-icon{}", suffix),
        );
        let txt = ctx.props().txt.txt.clone();
        let onclick = ctx.link().callback(|_| Message::ClickTop);

        html! {
            <div onclick={onclick} class={class}>
                <div class={class_text}>
                    {  text!(txt, ctx.props().language, "Set directions") }
                </div>
                <div class={class_icon}>
                </div>
            </div>
        }
    }

    fn view_direction_item(ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let value = if let Ok(selected) = ctx.props().selected_value.try_borrow() {
            selected.map_or_else(String::new, |value| {
                ctx.props()
                    .candidate_values
                    .iter()
                    .enumerate()
                    .find(|(_, &v)| v == value)
                    .map_or_else(String::new, |(index, _)| {
                        ctx.props()
                            .list
                            .get(index)
                            .map_or_else(String::new, |v| match v {
                                ViewString::Key(key) => {
                                    text!(txt, ctx.props().language, key).to_string()
                                }
                                ViewString::Raw(txt) => txt.clone(),
                            })
                    })
            })
        } else {
            String::new()
        };
        let style_width = ctx
            .props()
            .top_width
            .map_or_else(String::new, |w| format!("width: {}px;", w));
        let style = format!(
            "{} background-color: {};",
            style_width,
            &ctx.props().top_bg_color
        );
        let onclick = ctx.link().callback(|_| Message::ClickTop);

        html! {
            <div onclick={onclick} class="mini-select-item-direction" style={style}>
                <table>
                    <tr>
                        <td class="mini-select-item-direction">
                            { value }
                        </td>
                        <td class="mini-select-item-direction-icon">
                        </td>
                    </tr>
                </table>
            </div>
        }
    }

    fn view_sort_list(ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let value = if let Ok(selected) = ctx.props().selected_value.try_borrow() {
            selected.map(|value| {
                ctx.props()
                    .candidate_values
                    .iter()
                    .enumerate()
                    .find(|(_, &v)| v == value)
                    .map_or_else(String::new, |(index, _)| {
                        ctx.props()
                            .list
                            .get(index)
                            .map_or_else(String::new, |v| match v {
                                ViewString::Key(key) => {
                                    text!(txt, ctx.props().language, key).to_string()
                                }
                                ViewString::Raw(txt) => txt.clone(),
                            })
                    })
            })
        } else {
            None
        };
        let onclick = ctx.link().callback(|_| Message::ClickTop);

        if let Some(value) = value {
            html! {
                <div onclick={onclick} class="mini-select-list-sort-recently-text">
                    <div class="mini-select-list-sort-recently-text-icon">
                    </div>
                    <div class="mini-select-list-sort-recently-text-text">
                        { value }
                    </div>
                </div>
            }
        } else {
            html! {
                <div onclick={onclick} class="mini-select-list-sort-recently-icon">
                </div>
            }
        }
    }

    fn view_more_action(ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Message::ClickTop);
        html! {
            <div onclick={onclick} class="mini-select-more-action">
            </div>
        }
    }
}
