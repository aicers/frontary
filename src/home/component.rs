use super::Props;
use crate::home::{CheckBox, CheckStatus, NotificationItem};
use yew::{html, Component, Context, Html};

pub struct Model;

pub enum Message {
    Notify(NotificationItem),
}

impl Component for Model {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="input-checkbox">
            <CheckBox
                status={CheckStatus::Checked}
            />
            </div>
        }
    }
}
