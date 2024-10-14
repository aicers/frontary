use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;
use std::rc::Rc;
use std::time::Duration;

use gloo_timers::callback::Timeout;
use json_gettext::get_text;
use num_traits::ToPrimitive;
use yew::{html, Component, Context, Html, Properties};

use crate::{define_u32_consts, language::Language, text, window_inner_height, Texts};

#[cfg(feature = "pumpkin-dark")]
define_u32_consts! {
    DEFAULT_NOTIFICATION_WIDTH => 400
}
#[cfg(not(feature = "pumpkin-dark"))]
define_u32_consts! {
    DEFAULT_NOTIFICATION_WIDTH => 252
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Fail,
    Success,
}

const SUCCESS_COLOR: &str = "#83CA29";
const FAIL_COLOR: &str = "#B5131A";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Timeout(usize),
    Close(usize),
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, PartialEq, Eq)]
pub struct NotificationItem {
    pub message: String,
    pub sub_message: String,
    pub status_code: Option<u16>,
    pub time: Option<Duration>,
    pub category: Category, // color differs according to Category
}

pub const TIMEOUT_SECS: Duration = Duration::from_secs(10);

pub struct Model {
    timeouts: HashMap<usize, Timeout>,
}

#[derive(Clone, Eq, PartialEq, Properties)]
pub struct Props {
    pub txt: Texts,
    pub language: Language,
    pub list: Rc<RefCell<Vec<(usize, NotificationItem)>>>,
    pub serial: usize,
    #[prop_or(DEFAULT_NOTIFICATION_WIDTH)]
    pub width: u32,
}

impl Component for Model {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            timeouts: HashMap::new(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let Ok(list) = ctx.props().list.try_borrow() else {
            return false;
        };
        let (serial, time) = if let Some((serial, item)) = list.last() {
            (*serial, item.time)
        } else {
            return false;
        };

        if let Some(time) = time {
            let handle = {
                let link = ctx.link().clone();
                Timeout::new(
                    time.as_millis()
                        .to_u32()
                        .expect("timeout should be u32 size"),
                    move || link.send_message(Message::Timeout(serial)),
                )
            };
            self.timeouts.insert(serial, handle);
        }
        true
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
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Ok(list) = ctx.props().list.try_borrow() else {
            return html! {};
        };
        let style = format!(
            "width: {}px; max-height: {}px;",
            ctx.props().width,
            window_inner_height() - 60
        );
        html! {
            <>
                <div id="notification" class="notification" style={style}>
                { for list.iter().rev().map(|l| Self::view_item(ctx, l.0, &l.1)) }
                </div>
            </>
        }
    }
}

impl Model {
    fn view_item(ctx: &Context<Self>, serial: usize, noti: &NotificationItem) -> Html {
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

        html! {
            <table class="notification">
                <tr>
                    <td class="notification-contents" style={style_contents}>
                        if cfg!(feature = "pumpkin-dark") {
                            {
                                if noti.time.is_none() {
                                    html! {
                                        <div class="clumit-notification-error">
                                            <img src="/frontary/clumit-notification-error.svg" class="clumit-notification-error"/>
                                            { text!(txt, ctx.props().language, "Error") }
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
                                    <td class="notification-contents-text-close">
                                        <img src={
                                            if cfg!(feature = "pumpkin-dark") {
                                                "/frontary/clumit-notification-close.svg"
                                            } else {
                                                "/frontary/notification-close.png"
                                            }
                                        }
                                        class="notification-close"
                                        onclick={onclick_close}
                                        />
                                    </td>
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
                    if !cfg!(feature = "pumpkin-dark")  {
                        <td class="notification-label" style={style_label}>
                        </td>
                    }
                </tr>
            </table>
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum CommonError {
    SendGraphQLQueryError,
    HttpStatusNoSuccess(u16),
    GraphQLResponseError,
    GraphQLParseError,
    UnknownError,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, PartialEq, Eq)]
pub enum NotificationType {
    CommonError(CommonError),
    ErrorList(String, Vec<String>),
}

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
