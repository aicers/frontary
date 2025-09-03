//! Notification system for displaying success and error messages.
//!
//! This module provides a notification component that can display timed messages
//! with different categories (success/failure) and automatic dismissal.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;
use std::rc::Rc;
use std::time::Duration;

use gloo_timers::callback::Timeout;
use json_gettext::get_text;
use num_traits::ToPrimitive;
use yew::{Component, Context, Html, Properties, html};

use crate::{Texts, Theme, define_u32_consts, language::Language, text, window_inner_height};

#[cfg(feature = "pumpkin")]
define_u32_consts! {
    DEFAULT_NOTIFICATION_WIDTH => 400
}
#[cfg(not(feature = "pumpkin"))]
define_u32_consts! {
    DEFAULT_NOTIFICATION_WIDTH => 252
}

/// Category of notification determining visual styling.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Error or failure notification (red styling)
    Fail,
    /// Success notification (green styling)
    Success,
}

const SUCCESS_COLOR: &str = "#83CA29";
const FAIL_COLOR: &str = "#B5131A";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Timeout(usize),
    Close(usize),
    CloseAll,
}

/// A single notification item with message content and context.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, PartialEq, Eq)]
pub struct NotificationItem {
    /// Primary notification message
    pub message: String,
    /// Secondary message with additional details
    pub sub_message: String,
    /// HTTP status code if this notification is related to a network request
    pub status_code: Option<u16>,
    /// Optional timestamp for when the notification occurred
    pub time: Option<Duration>,
    /// Notification category affecting visual styling
    pub category: Category,
}

/// Default timeout duration for notifications before auto-dismissal
pub const TIMEOUT_SECS: Duration = Duration::from_secs(10);

/// Notification component model managing active notifications and their timers.
pub struct Model {
    /// Active timeouts for automatic notification dismissal
    timeouts: HashMap<usize, Timeout>,
}

/// Properties for the notification component.
#[derive(Clone, Eq, PartialEq, Properties)]
pub struct Props {
    /// Translation context for localized text
    pub txt: Texts,
    /// Current language for display
    pub language: Language,
    /// List of active notifications with their IDs
    pub list: Rc<RefCell<Vec<(usize, NotificationItem)>>>,
    /// Serial number for tracking updates
    pub serial: usize,
    /// Width of the notification panel in pixels
    #[prop_or(DEFAULT_NOTIFICATION_WIDTH)]
    pub width: u32,
    #[prop_or(None)]
    pub theme: Option<Theme>,
}

impl Component for Model {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut s = Self {
            timeouts: HashMap::new(),
        };
        s.add_timer(ctx);
        s
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.add_timer(ctx)
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Timeout(serial) => {
                ctx.link().send_message(Message::Close(serial));
                return false;
            }
            Message::Close(serial) => {
                self.timeouts.remove(&serial);
                if let Ok(mut list) = ctx.props().list.try_borrow_mut() {
                    list.retain(|l| l.0 != serial);
                }
            }
            Message::CloseAll => {
                self.timeouts.clear();
                if let Ok(mut list) = ctx.props().list.try_borrow_mut() {
                    list.clear();
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Ok(list) = ctx.props().list.try_borrow() else {
            return html! {};
        };
        let style = if cfg!(feature = "pumpkin") {
            format!("max-height: {}px;", window_inner_height() - 60)
        } else {
            format!(
                "width: {}px; max-height: {}px;",
                ctx.props().width,
                window_inner_height() - 60
            )
        };
        let theme = ctx.props().theme;
        html! {
            <>
                <div id="notification" class="notification" style={style}>
                    { if list.len() > 1 { Self::view_close_all_button(ctx) } else { html! {} } }
                    { for list.iter().rev().map(|l| Self::view_item(ctx, l.0, &l.1, theme)) }
                </div>
            </>
        }
    }
}

impl Model {
    fn add_timer(&mut self, ctx: &Context<Self>) -> bool {
        let Ok(list) = ctx.props().list.try_borrow() else {
            return false;
        };
        let (serial, time) = if let Some((serial, item)) = list.last() {
            (*serial, item.time)
        } else {
            return false;
        };
        if self.timeouts.contains_key(&serial) {
            return false;
        }

        if let Some(time) = time {
            let handle = {
                let link = ctx.link().clone();
                Timeout::new(
                    time.as_millis().to_u32().unwrap_or_else(|| {
                        TIMEOUT_SECS
                            .as_millis()
                            .to_u32()
                            .expect("Default timeout is within u32 range")
                    }),
                    move || link.send_message(Message::Timeout(serial)),
                )
            };
            self.timeouts.insert(serial, handle);
        }
        true
    }

    fn view_close_all_button(ctx: &Context<Self>) -> Html {
        let onclick_close_all = ctx.link().callback(|_| Message::CloseAll);
        let txt = ctx.props().txt.txt.clone();

        html! {
            <div class="notification-close-all">
                <div class="notification-close-all-button" onclick={onclick_close_all}>
                    { text!(txt, ctx.props().language, "Close All") }
                </div>
            </div>
        }
    }

    #[allow(clippy::too_many_lines)]
    fn view_item(
        ctx: &Context<Self>,
        serial: usize,
        noti: &NotificationItem,
        theme: Option<Theme>,
    ) -> Html {
        let color = match noti.category {
            Category::Fail => FAIL_COLOR,
            Category::Success => SUCCESS_COLOR,
        };
        let style_contents = format!("width: {}px;", ctx.props().width - 4);
        let style_label = format!("background-color: {color};");
        let txt = ctx.props().txt.txt.clone();
        let msg = get_text!(txt, ctx.props().language.tag(), &noti.message)
            .map_or(noti.message.clone(), |text| text.to_string());
        let msg = if noti.sub_message.is_empty() {
            msg
        } else {
            format!(
                "{}: {}",
                msg,
                get_text!(txt, ctx.props().language.tag(), &noti.sub_message)
                    .map_or(noti.sub_message.clone(), |text| text.to_string())
            )
        };

        let onclick_close = ctx.link().callback(move |_| Message::Close(serial));
        let onclick_done = ctx.link().callback(move |_| Message::Close(serial));
        let ext = if cfg!(feature = "pumpkin") {
            "svg"
        } else {
            "png"
        };
        let close_img = Theme::path(&theme, &format!("notification-close.{ext}"));
        let error_img = Theme::path(&theme, "notification-error.svg");

        html! {
            <table class="notification">
                <tr>
                    <td class="notification-contents" style={style_contents}>
                        if cfg!(feature = "pumpkin") {
                            {
                                if noti.time.is_none() {
                                    html! {
                                        <div class="clumit-notification-error">
                                            <img src={error_img} class="clumit-notification-error"/>
                                            { text!(txt, ctx.props().language, "Error") }
                                            if cfg!(feature = "pumpkin") {
                                                <td class="notification-contents-text-close">
                                                    <img src={close_img.clone()}
                                                    class="notification-close"
                                                    onclick={onclick_close.clone()}
                                                    />
                                                </td>
                                            }
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        }
                        <div class="notification-contents-text">
                            <table class="notification-contents-text-table">
                                <tr>
                                    <td class="notification-contents-text-text">
                                        { msg }
                                    </td>
                                    if cfg!(feature = "pumpkin") {
                                        if noti.time.is_some() {
                                            <td class="notification-contents-text-close">
                                                <img src={ close_img.clone() }
                                                class="notification-close"
                                                onclick={onclick_close.clone()}
                                                />
                                            </td>
                                        }
                                    } else {
                                        <td class="notification-contents-text-close">
                                            <img src={close_img}
                                            class="notification-close"
                                            onclick={onclick_close}
                                            />
                                        </td>
                                    }
                                </tr>
                            </table>
                        </div>
                        {
                            if noti.time.is_none() {
                                html! {
                                    <div class="notification-contents-done">
                                        <div class="notification-contents-done-button" onclick={onclick_done}>
                                            { text!(txt, ctx.props().language, "Done") }
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </td>
                    if !cfg!(feature = "pumpkin")  {
                        <td class="notification-label" style={style_label}>
                        </td>
                    }
                </tr>
            </table>
        }
    }
}

/// Common error types that can be converted to notifications.
#[derive(Clone, PartialEq, Eq)]
pub enum CommonError {
    /// Error sending GraphQL query
    SendGraphQLQueryError,
    /// HTTP request returned non-success status code
    HttpStatusNoSuccess(u16),
    /// Invalid GraphQL response received
    GraphQLResponseError,
    /// Error parsing GraphQL response
    GraphQLParseError,
    /// Unspecified error occurred
    UnknownError,
}

/// Types of notifications that can be generated.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, PartialEq, Eq)]
pub enum NotificationType {
    /// A common error type
    CommonError(CommonError),
    /// A list of errors with a primary message
    ErrorList(String, Vec<String>),
}

/// Generates a notification item from a notification type.
///
/// Converts various error types into structured notification items
/// with appropriate messages and styling.
///
/// # Arguments
///
/// * `noti` - The notification type to convert
///
/// # Returns
///
/// A `NotificationItem` ready for display
///
/// # Examples
///
/// ```rust
/// use frontary::{gen_notifications, NotificationType, CommonError};
///
/// let notification = gen_notifications(
///     NotificationType::CommonError(CommonError::UnknownError)
/// );
/// assert_eq!(notification.message, "Unknown error");
/// ```
#[must_use]
pub fn gen_notifications(noti: NotificationType) -> NotificationItem {
    match noti {
        NotificationType::CommonError(msg) => match msg {
            CommonError::GraphQLResponseError => NotificationItem {
                message: "Invalid GraphQL response".to_string(),
                sub_message: String::new(),
                status_code: None,
                time: None,
                category: Category::Fail,
            },
            CommonError::SendGraphQLQueryError => NotificationItem {
                message: "Invalid GraphQL query".to_string(),
                sub_message: String::new(),
                status_code: None,
                time: None,
                category: Category::Fail,
            },
            CommonError::HttpStatusNoSuccess(status) => NotificationItem {
                message: "No success HTTPS status code".to_string(),
                sub_message: status.to_string(),
                status_code: Some(status),
                time: None,
                category: Category::Fail,
            },
            CommonError::UnknownError => NotificationItem {
                message: "Unknown error".to_string(),
                sub_message: String::new(),
                status_code: None,
                time: None,
                category: Category::Fail,
            },
            CommonError::GraphQLParseError => NotificationItem {
                message: "GraphQL parse error".to_string(),
                sub_message: String::new(),
                status_code: None,
                time: None,
                category: Category::Fail,
            },
        },
        NotificationType::ErrorList(message, errors) => {
            let mut sub_message = String::new();
            if !errors.is_empty() {
                let last = errors.len() - 1;
                for (index, error) in errors.into_iter().enumerate() {
                    if index == last {
                        if error.ends_with("Forbidden") {
                            sub_message += "Unauthorized";
                        } else {
                            sub_message += &error;
                        }
                    } else {
                        write!(sub_message, "{error} & ").expect("in-memory operation");
                    }
                }
            }
            NotificationItem {
                message,
                sub_message,
                status_code: None,
                time: None,
                category: Category::Fail,
            }
        }
    }
}
