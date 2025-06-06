use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};

use json_gettext::get_text;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::{Component, Context, Html, Properties, TargetCast, events::InputEvent, html};

use crate::{
    HostNetwork, InputHostNetworkGroup, Texts, language::Language, parse_host_network, text,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    HostOnly,
    NetworkOnly,
    RangeOnly,
    All,
}

#[derive(Clone, Copy, PartialEq)]
enum ItemType {
    Host(usize),
    Network(usize),
    Range(usize),
}

#[derive(PartialEq)]
pub struct Model<T> {
    input: String,
    message: Option<&'static str>,
    view_order: Vec<ItemType>,
    phantom: PhantomData<T>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Message {
    Input(String),
    InputHostNetwork(String),
    Delete(DeleteIndex),
    TabBackspace(String),
    VerifyToSave,
    InputError,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DeleteIndex {
    Host(usize),
    Network(usize),
    Range(usize),
}

const DEFAULT_MAX_HEIGHT: u32 = 280;
const EXIST_MSG: &str = "The input already exists.";
const INVALID_INPUT_MSG: &str =
    "Invalid input (valid examples: 10.84.1.7, 10.1.1.1 - 10.1.1.20, 192.168.10.0/24)";
const INVALID_INPUT_MSG_HOST: &str = "Invalid IP address";
const INVALID_INPUT_MSG_RANGE: &str = "Invalid input (valid examples: 10.1.1.1 - 10.1.1.20)";
const MAX_NUM_MSG: &str = "The maximum number of input was reached.";
const INPUT_ALL_NOTICE: &str =
    "Multiple inputs possible (valid examples: 10.84.1.7, 10.1.1.1 - 10.1.1.20, 192.168.10.0/24)";
const INPUT_HOST_NOTICE: &str = "Multiple IP addresses possible";
const INPUT_RANGE_NOTICE: &str = "(Input Example: 192.168.1.100 - 192.168.1.200)";
const INPUT_NETWORK_NOTICE: &str = "(Input Example: 192.168.10.0/24)";

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub txt: Texts,
    pub language: Language,
    pub rerender_serial: u64,
    #[prop_or(Kind::All)]
    pub kind: Kind,
    #[prop_or(None)]
    pub num: Option<usize>,
    #[prop_or(None)]
    pub parent_message: Option<T::Message>,
    #[prop_or(None)]
    pub parent_message_save: Option<T::Message>,
    #[prop_or(None)]
    pub parent_message_no_save: Option<T::Message>,
    #[prop_or(None)]
    pub parent_message_user_input: Option<T::Message>,
    pub input_data: Rc<RefCell<InputHostNetworkGroup>>,
    #[prop_or(None)]
    pub input_notice: Option<&'static str>,
    #[prop_or(None)]
    pub width: Option<u32>,
    #[prop_or(DEFAULT_MAX_HEIGHT)]
    pub max_height: u32,
    #[prop_or(false)]
    pub verify_to_save: bool,
    #[prop_or(false)]
    pub is_required: bool,
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
            input: String::new(),
            message: None,
            view_order: Vec::new(),
            phantom: PhantomData,
        };
        s.init_view_order(ctx);
        s
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if ctx.props().verify_to_save {
            ctx.link().send_message(Message::VerifyToSave);
            false
        } else {
            true
        }
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Input(input) => {
                self.input = input;
                self.message = None;
                if let (Some(parent), Some(msg)) = (
                    ctx.link().get_parent(),
                    ctx.props().parent_message_user_input.as_ref(),
                ) {
                    parent.clone().downcast::<T>().send_message(msg.clone());
                }
            }
            Message::InputHostNetwork(last) => {
                match last.as_str() {
                    ";" | "," => {
                        self.input.pop();
                    }
                    _ => (),
                }
                if Self::max_num(ctx) {
                    self.message = Some(MAX_NUM_MSG);
                } else if self.verify(ctx).unwrap_or(false) {
                    if let (Some(parent), Some(msg)) =
                        (ctx.link().get_parent(), ctx.props().parent_message.as_ref())
                    {
                        parent.clone().downcast::<T>().send_message(msg.clone());
                    }
                }
            }
            Message::TabBackspace(key) => match key.as_str() {
                "Backspace" => {
                    if self.input.is_empty() {
                        if let Ok(mut data) = ctx.props().input_data.try_borrow_mut() {
                            let last = self.view_order.pop();
                            let send_msg = match last {
                                Some(ItemType::Host(index)) => {
                                    if index < data.hosts.len() {
                                        data.hosts.remove(index);
                                    }
                                    true
                                }
                                Some(ItemType::Network(index)) => {
                                    if index < data.networks.len() {
                                        data.networks.remove(index);
                                    }
                                    true
                                }
                                Some(ItemType::Range(index)) => {
                                    if index < data.ranges.len() {
                                        data.ranges.remove(index);
                                    }
                                    true
                                }
                                None => false,
                            };
                            if send_msg {
                                if let (Some(parent), Some(msg)) =
                                    (ctx.link().get_parent(), ctx.props().parent_message.as_ref())
                                {
                                    parent.clone().downcast::<T>().send_message(msg.clone());
                                }
                            }
                        }
                    }
                }
                "Tab" => {
                    if !self.input.is_empty() {
                        ctx.link()
                            .send_message(Message::InputHostNetwork("Tab".to_string()));
                    }
                }
                _ => (),
            },
            Message::Delete(index) => {
                if let Ok(mut data) = ctx.props().input_data.try_borrow_mut() {
                    match index {
                        DeleteIndex::Host(index) => {
                            if index < data.hosts.len() {
                                data.hosts.remove(index);
                                self.view_order
                                    .retain(|item| *item != ItemType::Host(index));
                                for v in &mut self.view_order {
                                    if let ItemType::Host(i) = *v {
                                        if i > index {
                                            *v = ItemType::Host(i - 1);
                                        }
                                    }
                                }
                            }
                        }
                        DeleteIndex::Network(index) => {
                            if index < data.networks.len() {
                                data.networks.remove(index);
                                self.view_order
                                    .retain(|item| *item != ItemType::Network(index));
                                for v in &mut self.view_order {
                                    if let ItemType::Network(i) = *v {
                                        if i > index {
                                            *v = ItemType::Network(i - 1);
                                        }
                                    }
                                }
                            }
                        }
                        DeleteIndex::Range(index) => {
                            if index < data.ranges.len() {
                                data.ranges.remove(index);
                                self.view_order
                                    .retain(|item| *item != ItemType::Range(index));
                                for v in &mut self.view_order {
                                    if let ItemType::Range(i) = *v {
                                        if i > index {
                                            *v = ItemType::Range(i - 1);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    self.message = None;
                    if let (Some(parent), Some(msg)) =
                        (ctx.link().get_parent(), ctx.props().parent_message.as_ref())
                    {
                        parent.clone().downcast::<T>().send_message(msg.clone());
                    }
                }
            }
            Message::VerifyToSave => {
                if Self::max_num(ctx) {
                    self.message = Some(MAX_NUM_MSG);
                } else {
                    let verify = self.verify(ctx);
                    if let (Some(parent), Some(msg_save), Some(msg_no_save)) = (
                        ctx.link().get_parent(),
                        ctx.props().parent_message_save.as_ref(),
                        ctx.props().parent_message_no_save.as_ref(),
                    ) {
                        parent
                            .clone()
                            .downcast::<T>()
                            .send_message(verify.map_or_else(
                                || msg_save.clone(),
                                |verify| {
                                    if verify {
                                        msg_save.clone()
                                    } else {
                                        msg_no_save.clone()
                                    }
                                },
                            ));
                    }
                }
            }
            Message::InputError => {
                // TODO: issue #5
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = format!(
            "max-height: {}px; width: {};",
            ctx.props().max_height,
            ctx.props()
                .width
                .map_or("100%".to_string(), |w| format!("{w}px"))
        );
        let class = if ctx.props().is_required {
            "input-host-network-group-required"
        } else {
            "host-network-group-input"
        };
        html! {
            <>
                <div {class} {style}>
                    { self.view_host_network_group(ctx) }
                    { self.view_input(ctx) }
                </div>
                { self.view_message(ctx) }
            </>
        }
    }
}

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    fn view_host_network_group(&self, ctx: &Context<Self>) -> Html {
        if let Ok(data) = ctx.props().input_data.try_borrow() {
            html! {
                for self.view_order.iter().map(|item| {
                    match item {
                        ItemType::Host(index) => {
                            if let Some(host) = data.hosts.get(*index) {
                                Self::view_item(ctx, DeleteIndex::Host(*index), host)
                            } else {
                                html! {}
                            }
                        }
                        ItemType::Network(index) => {
                            if let Some(network) = data.networks.get(*index) {
                                Self::view_item(ctx, DeleteIndex::Network(*index), network)
                            } else {
                                html! {}
                            }
                        }
                        ItemType::Range(index) => {
                            if let Some(range) = data.ranges.get(*index) {
                                Self::view_item(ctx, DeleteIndex::Range(*index), &range.to_string())
                            } else {
                                html! {}
                            }
                        }
                    }
                })
            }
        } else {
            html! {}
        }
    }

    fn view_item(ctx: &Context<Self>, index: DeleteIndex, item: &str) -> Html {
        let onclick_delete =
            |index: DeleteIndex| ctx.link().callback(move |_| Message::Delete(index));
        html! {
            <div class="host-network-group-input-item">
                { item }
                if cfg!(feature = "pumpkin") {
                    <img src="/frontary/pumpkin/delete-x.svg" class="host-network-group-close" onclick={onclick_delete(index)} />
                } else {
                    <img src="/frontary/host-network-close.png" class="host-network-group-close" onclick={onclick_delete(index)} />
                }
            </div>
        }
    }

    fn view_input(&self, ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();

        let placeholder = if let (Ok(_), Some(notice)) = (
            ctx.props().input_data.try_borrow(),
            ctx.props().input_notice,
        ) {
            text!(txt, ctx.props().language, notice).to_string()
        } else {
            String::new()
        };
        let oninput = ctx.link().callback(|e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| Message::Input(input.value()))
        });
        let onkeyup = ctx.link().batch_callback(move |e: KeyboardEvent| {
            (e.key() == "Enter" || e.key() == ";" || e.key() == ",").then(|| {
                let input: HtmlInputElement = e.target_unchecked_into();
                let value = input.value();
                input.set_value("");
                Message::InputHostNetwork(value)
            })
        });
        let onkeydown = ctx.link().batch_callback(move |e: KeyboardEvent| {
            (e.key() == "Backspace" || e.key() == "Tab").then(|| {
                let input: HtmlInputElement = e.target_unchecked_into();
                let value = input.value();
                input.set_value("");
                Message::TabBackspace(value)
            })
        });

        html! {
            <>
                <input type="text"
                    class="host-network-group-input-input-input"
                    value={self.input.clone()}
                    placeholder={placeholder}
                    oninput={oninput}
                    onkeyup={onkeyup}
                    onkeydown={onkeydown}
                />
            </>
        }
    }

    fn view_message(&self, ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();

        let notice = match ctx.props().kind {
            Kind::All => Some(INPUT_ALL_NOTICE),
            Kind::HostOnly => {
                if ctx.props().num == Some(1) {
                    None
                } else {
                    Some(INPUT_HOST_NOTICE)
                }
            }
            Kind::NetworkOnly => {
                if ctx.props().num == Some(1) {
                    None
                } else {
                    Some(INPUT_NETWORK_NOTICE)
                }
            }
            Kind::RangeOnly => {
                if ctx.props().num == Some(1) {
                    None
                } else {
                    Some(INPUT_RANGE_NOTICE)
                }
            }
        };
        if let Some(msg) = self.message {
            html! {
                <div class="host-network-group-input-input-message">
                    { text!(txt, ctx.props().language, msg) }
                </div>
            }
        } else if let Some(notice) = notice {
            html! {
                <div class="host-network-group-input-input-notice">
                    { text!(txt, ctx.props().language, notice) }
                </div>
            }
        } else {
            html! {}
        }
    }

    fn init_view_order(&mut self, ctx: &Context<Self>) {
        if let Ok(data) = ctx.props().input_data.try_borrow() {
            for (index, _) in data.hosts.iter().enumerate() {
                self.view_order.push(ItemType::Host(index));
            }
            for (index, _) in data.networks.iter().enumerate() {
                self.view_order.push(ItemType::Network(index));
            }
            for (index, _) in data.ranges.iter().enumerate() {
                self.view_order.push(ItemType::Range(index));
            }
        }
    }

    // None means empty
    fn verify(&mut self, ctx: &Context<Self>) -> Option<bool> {
        if let Some((l, _)) = self.input.split_once(',') {
            self.input = l.to_string();
        } else if let Some((l, _)) = self.input.split_once(';') {
            self.input = l.to_string();
        }
        if self.input.is_empty() {
            None
        } else if let Ok(mut data) = ctx.props().input_data.try_borrow_mut() {
            match ctx.props().kind {
                Kind::All => match parse_host_network(&self.input) {
                    Some(HostNetwork::Host(host)) => {
                        if data.hosts.binary_search(&host).is_ok() {
                            self.message = Some(EXIST_MSG);
                            Some(false)
                        } else {
                            self.view_order.push(ItemType::Host(data.hosts.len()));
                            data.hosts.push(host);
                            self.input = String::new();
                            Some(true)
                        }
                    }
                    Some(HostNetwork::Network(network)) => {
                        if data.networks.binary_search(&network).is_ok() {
                            self.message = Some(EXIST_MSG);
                            Some(false)
                        } else {
                            self.view_order.push(ItemType::Network(data.networks.len()));
                            data.networks.push(network);
                            self.input = String::new();
                            Some(true)
                        }
                    }
                    Some(HostNetwork::Range(range)) => {
                        if data.ranges.binary_search(&range).is_ok() {
                            self.message = Some(EXIST_MSG);
                            Some(false)
                        } else {
                            self.view_order.push(ItemType::Range(data.ranges.len()));
                            data.ranges.push(range);
                            self.input = String::new();
                            Some(true)
                        }
                    }
                    None => {
                        self.message = Some(INVALID_INPUT_MSG);
                        Some(false)
                    }
                },
                Kind::HostOnly => {
                    if let Some(HostNetwork::Host(host)) = parse_host_network(&self.input) {
                        if data.hosts.binary_search(&host).is_ok() {
                            self.message = Some(EXIST_MSG);
                            Some(false)
                        } else {
                            self.view_order.push(ItemType::Host(data.hosts.len()));
                            data.hosts.push(host);
                            self.input = String::new();
                            Some(true)
                        }
                    } else {
                        self.message = Some(INVALID_INPUT_MSG_HOST);
                        Some(false)
                    }
                }
                Kind::NetworkOnly => {
                    if let Some(HostNetwork::Network(network)) = parse_host_network(&self.input) {
                        if data.networks.binary_search(&network).is_ok() {
                            self.message = Some(EXIST_MSG);
                            Some(false)
                        } else {
                            self.view_order.push(ItemType::Network(data.networks.len()));
                            data.networks.push(network);
                            self.input = String::new();
                            Some(true)
                        }
                    } else {
                        self.message = Some(INVALID_INPUT_MSG);
                        Some(false)
                    }
                }
                Kind::RangeOnly => {
                    if let Some(HostNetwork::Range(range)) = parse_host_network(&self.input) {
                        if data.ranges.binary_search(&range).is_ok() {
                            self.message = Some(EXIST_MSG);
                            Some(false)
                        } else {
                            self.view_order.push(ItemType::Range(data.ranges.len()));
                            data.ranges.push(range);
                            self.input = String::new();
                            Some(true)
                        }
                    } else {
                        self.message = Some(INVALID_INPUT_MSG_RANGE);
                        Some(false)
                    }
                }
            }
        } else {
            Some(false)
        }
    }

    fn max_num(ctx: &Context<Self>) -> bool {
        if let (Ok(data), Some(num)) = (ctx.props().input_data.try_borrow(), ctx.props().num) {
            data.hosts.len() + data.networks.len() + data.ranges.len() > num
        } else {
            false
        }
    }
}
