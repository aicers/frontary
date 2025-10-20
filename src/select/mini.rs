use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};

use gloo_events::EventListener;
use json_gettext::get_text;
use web_sys::{Event, HtmlElement};
use yew::virtual_dom::AttrValue;
use yew::{Component, Context, Html, NodeRef, Properties, classes, html};

use crate::{Texts, Theme, ViewString, language::Language, text, toggle_visibility};

pub struct Model<T, U> {
    click_listener: Option<EventListener>,
    click_count: usize,
    phantom: PhantomData<(T, U)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    DirectionAll,
    DirectionItem,
    MoreAction,
    MoreActionNoImage,
    MoreActionBasic,
    OnOffAction,
    SortList,
    Round,
    Soft,
}

pub enum Message {
    ClickTop,
    ClickItem(usize),
    ListenClick,
}

#[cfg(feature = "pumpkin")]
const DEFAULT_BG_COLOR: &str = "rgba(97, 105, 116, 0.24);";
#[cfg(not(feature = "pumpkin"))]
const DEFAULT_BG_COLOR: &str = "#EAEAEA";
#[cfg(feature = "pumpkin")]
const DEFAULT_VALUE_TEXT_COLOR: &str = "#FFFFFF";
#[cfg(not(feature = "pumpkin"))]
const DEFAULT_VALUE_TEXT_COLOR: &str = "#363636";
const DEFAULT_LIST_TEXT_COLOR: &str = "#363636";

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
    pub deactive_class_suffix: Option<AttrValue>,
    pub id: AttrValue,
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
    #[prop_or(DEFAULT_BG_COLOR.into())]
    pub top_bg_color: AttrValue,
    #[prop_or(DEFAULT_VALUE_TEXT_COLOR.into())]
    pub value_text_color: AttrValue,
    #[prop_or(DEFAULT_LIST_TEXT_COLOR.into())]
    pub list_text_color: AttrValue,
    #[prop_or(None)]
    pub theme: Option<Theme>,
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
            click_listener: None,
            click_count: 0,
            phantom: PhantomData,
        };
        if let Some(value) = ctx.props().default_value
            && let Ok(mut selected) = ctx.props().selected_value.try_borrow_mut()
            && selected.is_none()
        {
            *selected = Some(value);
        }
        s
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }
        if let Some(home_div) = ctx.props().home_div.as_ref()
            && let Some(element) = home_div.cast::<HtmlElement>()
        {
            let callback = ctx.link().callback(|_: Event| Message::ListenClick);
            let listener = EventListener::new(&element, "click", move |e| callback.emit(e.clone()));
            self.click_listener = Some(listener);
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
        let txt = ctx.props().txt.txt.clone();
        let theme = ctx.props().theme;
        let msg = text!(txt, ctx.props().language, "Select one").to_string();
        let value = if let Ok(selected) = ctx.props().selected_value.try_borrow() {
            selected.map_or(msg.clone(), |value| {
                ctx.props()
                    .candidate_values
                    .iter()
                    .enumerate()
                    .find(|(_, v)| *v == &value)
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
        html! {
            <div class="mini-select">
            {
                match ctx.props().kind {
                    Kind::DirectionAll => Self::view_direction_all(ctx),
                    Kind::DirectionItem => Self::view_direction_item(ctx),
                    Kind::SortList => Self::view_sort_list(ctx),
                    Kind::MoreAction => Self::view_more_action(ctx),
                    Kind::MoreActionNoImage => html! {},
                    Kind::MoreActionBasic => Self::view_more_action_basic(ctx),
                    Kind::OnOffAction => Self::view_on_off_action(ctx),
                    Kind::Round | Kind::Soft => Self::view_basic(ctx,&value),
                }
            }
            { Self::view_list(ctx,&value, theme) }
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
    fn value_to_text(ctx: &Context<Self>, value: &T) -> Option<String> {
        let txt = &ctx.props().txt.txt;
        ctx.props()
            .candidate_values
            .iter()
            .enumerate()
            .find(|(_, candidate)| *candidate == value)
            .and_then(|(index, _)| ctx.props().list.get(index))
            .map(|view_string| match view_string {
                ViewString::Key(key) => text!(txt, ctx.props().language, key).to_string(),
                ViewString::Raw(raw) => raw.clone(),
            })
    }

    #[allow(clippy::too_many_lines)]
    fn view_list(ctx: &Context<Self>, value: &str, theme: Option<Theme>) -> Html {
        let list = ctx.props().list.clone();
        let onclick_item = |index: usize| ctx.link().callback(move |_| Message::ClickItem(index));
        let style_width = ctx
            .props()
            .list_min_width
            .map_or_else(String::new, |w| format!("min-width: {w}px;"));
        let align = if ctx.props().align_left {
            "left"
        } else {
            "right"
        };
        let list_top = if cfg!(feature = "pumpkin") {
            43
        } else {
            ctx.props().list_top
        };
        let style = format!(
            "{}: 0px; top: {}px; {}; color: {};",
            align,
            list_top,
            &style_width,
            ctx.props().list_text_color,
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
                        for list.iter().enumerate().map(|(index, item)|{
                        let class_select = match item {
                            ViewString::Key(key) => if key == value {
                                "mini-select-list-down-item-selected"
                            } else {
                                "mini-select-list-down-item"
                            },
                            ViewString::Raw(txt) => if txt == value {
                                "mini-select-list-down-item-selected"
                            } else {
                                "mini-select-list-down-item"
                            },
                        };
                        html! {
                            <tr>
                                {
                                    if ctx.props().kind == Kind::MoreAction {
                                        match item {
                                            ViewString::Key(key) => {
                                                let ext = if cfg!(feature = "pumpkin") { "svg" } else { "png" };
                                                let icon = match key.as_ref() {
                                                    "Edit" => Theme::path(&theme, &format!("edit.{ext}")),
                                                    "Delete" => Theme::path(&theme, &format!("delete-trash.{ext}")),
                                                    _ => String::new(),
                                                };
                                                html! {
                                                    <td class={classes!("mini-select-list-down-item", class_list_align)} onclick={onclick_item(index)} style={style_width.clone()}>
                                                        <div class="mini-select-list-down-item-more-action">
                                                            <img src={icon} class="mini-select-list-down-item-more-action" />
                                                            <div class={classes!("mini-select-list-down-item-more-action-text", class_list_align_more_action)}>
                                                            { text!(txt, ctx.props().language, key) }
                                                            </div>
                                                        </div>
                                                    </td>
                                                }
                                            }
                                            ViewString::Raw(_) => html! {}
                                        }
                                    } else if ctx.props().kind == Kind::OnOffAction {
                                        match item {
                                            ViewString::Key(key) => {
                                                let icon = if key == "On" {
                                                    Theme::path(&theme, "on.png")
                                                } else if key == "Off" {
                                                    Theme::path(&theme, "off.png")
                                                } else {
                                                    String::new()
                                                };
                                                html! {
                                                    <td class={classes!("mini-select-list-down-item", class_list_align)} onclick={onclick_item(index)} style={style_width.clone()}>
                                                        <div class="mini-select-list-down-item-more-action">
                                                            <img src={icon} class="mini-select-list-down-item-more-action" />
                                                            <div class={classes!("mini-select-list-down-item-more-action-text", class_list_align_more_action)}>
                                                            { text!(txt, ctx.props().language, key) }
                                                            </div>
                                                        </div>
                                                    </td>
                                                }
                                            }
                                            ViewString::Raw(_) => html! {}
                                        }
                                    } else if ctx.props().kind == Kind::MoreActionBasic {
                                        match item {
                                            ViewString::Key(key) => {
                                                html! {
                                                    <td class={classes!("mini-select-list-down-item", class_list_align)} onclick={onclick_item(index)} style={style_width.clone()}>
                                                        <div class="mini-select-list-down-item-more-action">
                                                            <div class={classes!("mini-select-list-down-item-more-action-text", class_list_align_more_action)}>
                                                            { text!(txt, ctx.props().language, key) }
                                                            </div>
                                                        </div>
                                                    </td>
                                                }
                                            }
                                            ViewString::Raw(_) => html! {}
                                        }
                                    } else {
                                        match item {
                                            ViewString::Key(key) =>
                                                html! {
                                                    <td class={classes!(class_select, class_list_align)} onclick={onclick_item(index)} style={style_width.clone()}>
                                                        {text!(txt, ctx.props().language, key)}
                                                    </td>
                                                },
                                            ViewString::Raw(txt) =>
                                                html! {
                                                    <td class={classes!(class_select, class_list_align)} onclick={onclick_item(index)} style={style_width.clone()}>
                                                        { txt }
                                                    </td>
                                                },
                                        }
                                    }
                                }
                            </tr>
                        }})
                    }
                </table>
            </div>
        }
    }

    fn view_basic(ctx: &Context<Self>, value: &str) -> Html {
        let style_width = ctx.props().top_width.map_or_else(String::new, |w| {
            if cfg!(feature = "pumpkin") {
                String::new()
            } else {
                format!("width: {w}px;")
            }
        });
        let style_height = ctx
            .props()
            .top_height
            .map_or_else(String::new, |h| format!("height: {h}px;"));
        let style = if ctx.props().kind == Kind::Round {
            if cfg!(feature = "pumpkin") {
                format!(
                    "{} {} color: {};",
                    style_width,
                    style_height,
                    &ctx.props().value_text_color,
                )
            } else {
                format!(
                    "{} {} background-color: {}; color: {};",
                    style_width,
                    style_height,
                    &ctx.props().top_bg_color,
                    &ctx.props().value_text_color,
                )
            }
        } else {
            // `Kind::Soft` only
            if cfg!(feature = "pumpkin") {
                format!("{style_width} {style_height}")
            } else {
                format!("{style_width} {style_height} background-color: #FFFFFF;",)
            }
        };
        let (outer_sub_class, icon_sub_class) = if ctx.props().kind == Kind::Round {
            ("mini-select-basic-basic", "mini-select-basic-icon-basic")
        } else {
            ("mini-select-basic-square", "mini-select-basic-icon-square")
        };
        let onclick = ctx.link().callback(|_| Message::ClickTop);

        html! {
            <div onclick={onclick} class={classes!("mini-select-basic", outer_sub_class)} style={style}>
            <div class="mini-select-basic-value" style={
                if cfg!(feature = "pumpkin") {
                    String::new()
                } else {
                    style_height.clone()
                }
            }>
                    { value }
                </div>
                <div class={classes!("mini-select-basic-icon", icon_sub_class)} style={
                    if cfg!(feature = "pumpkin") {
                        String::new()
                    } else {
                        style_height
                    }
                }>
                </div>
            </div>
        }
    }

    fn view_direction_all(ctx: &Context<Self>) -> Html {
        let suffix = if ctx.props().active {
            String::new()
        } else if let Some(suffix) = ctx.props().deactive_class_suffix.as_ref() {
            suffix.as_ref().into()
        } else {
            String::new()
        };
        let disabled = !ctx.props().active;
        let class_base = format!("mini-select-top-direction{suffix}");
        let class_text_base = format!("mini-select-top-direction-text{suffix}");
        let class_icon_base = format!("mini-select-top-direction-icon{suffix}");

        let class = classes!(&class_base, disabled.then_some("is-disabled"));
        let class_text = classes!(&class_text_base, disabled.then_some("is-disabled"));
        let class_icon = classes!(&class_icon_base, disabled.then_some("is-disabled"));
        let txt = ctx.props().txt.txt.clone();
        let onclick = ctx.link().callback(|_| Message::ClickTop);

        html! {
            <div onclick={onclick} class={class}>
                <div class={class_icon}>
                </div>
                <div class={class_text}>
                    {  text!(txt, ctx.props().language, "Set directions") }
                </div>
            </div>
        }
    }

    fn view_direction_item(ctx: &Context<Self>) -> Html {
        let selected_value = ctx
            .props()
            .selected_value
            .try_borrow()
            .ok()
            .and_then(|selected| selected.as_ref().copied());
        let default_value = if ctx.props().active {
            None
        } else {
            ctx.props().default_value
        };
        let first_candidate = if ctx.props().active {
            None
        } else {
            ctx.props().candidate_values.first().copied()
        };
        let value = [selected_value, default_value, first_candidate]
            .into_iter()
            .flatten()
            .find_map(|candidate| Self::value_to_text(ctx, &candidate))
            .unwrap_or_default();
        let style_width = ctx
            .props()
            .top_width
            .map_or_else(String::new, |w| format!("width: {w}px;"));
        let style = if cfg!(feature = "pumpkin") {
            style_width.to_string()
        } else {
            format!(
                "{} background-color: {};",
                style_width,
                &ctx.props().top_bg_color
            )
        };
        let onclick = ctx.link().callback(|_| Message::ClickTop);

        html! {
            <div onclick={onclick} class="mini-select-item-direction" style={style}>
                <table>
                    <tr>
                        <td class="mini-select-item-direction-text">
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
                    .find(|(_, v)| *v == &value)
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

    fn view_on_off_action(ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Message::ClickTop);
        html! {
            <div onclick={onclick} class="mini-select-on-off-action">
            </div>
        }
    }

    fn view_more_action_basic(ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Message::ClickTop);
        html! {
            <div onclick={onclick} class="mini-select-on-off-action">
            </div>
        }
    }
}
