mod component;

pub use crate::checkbox::{CheckStatus, Model as CheckBox};
pub use crate::input::{
    ChildrenPosition, Essential as InputEssential, HostNetworkHtml, HostNetworkKind,
    InputHostNetworkGroup, InputItem, InputNic, InputTag, InputTagGroup, InputType, Model as Input,
    Tag,
};
pub use crate::list::{Column, ListItem, MessageType, WholeList}; //Column, ListItem 추가
pub use crate::modal::{
    AlignButton as ModalAlign, Model as Modal, MsgType as ModalType, TextStyle as ModalTextStyle,
};
pub use crate::notification::{
    gen_notifications, Category as NotificationCategory, Model as Notification, NotificationItem,
    NotificationType, TIMEOUT_SECS,
};
pub use crate::pages::{Info as PagesInfo, Model as Pages};
pub use crate::radio::Model as Radio;
pub use crate::radio_separate::Model as RadioSeparate;
pub use crate::select::complex::{Kind as SelectComplexKind, Model as SelectComplex};
pub use crate::select::mini::{Kind as SelectMiniKind, Model as SelectMini};
pub use crate::select::searchable::{Kind as SelectSearchableKind, Model as SelectSearchable};
pub use crate::sort::{Model as Sort, Status as SortStatus};
pub use crate::tab_menu::Model as TabMenu;

use crate::language::Language;
use ipnet::Ipv4Net;
use json_gettext::{get_text, JSONGetText};
use num_traits::ToPrimitive;
use std::net::Ipv4Addr;
use std::rc::Rc;
use std::str::FromStr;
use wasm_bindgen::JsCast;

use anyhow::Result;
pub use component::{Message, Model};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use yew::{html::Scope, Callback, Component, Context, NodeRef, Properties};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoreAction {
    Edit,
    Delete,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViewString {
    Key(String),
    Raw(String),
}

impl ToString for ViewString {
    fn to_string(&self) -> String {
        match self {
            Self::Key(key) => key.clone(),
            Self::Raw(txt) => txt.clone(),
        }
    }
}

impl ViewString {}

pub enum HostNetwork {
    Host(String),
    Network(String),
    Range(IpRange),
}

pub fn parse_host_network(input: &str) -> Option<HostNetwork> {
    if Ipv4Addr::from_str(input).is_ok() {
        return Some(HostNetwork::Host(input.to_string()));
    }
    if Ipv4Net::from_str(input).is_ok() {
        return Some(HostNetwork::Network(input.to_string()));
    }
    if let Some((start, end)) = input.split_once('~') {
        let (start, end) = (start.trim(), end.trim());
        if let (Ok(start), Ok(end)) = (Ipv4Addr::from_str(start), Ipv4Addr::from_str(end)) {
            if start < end {
                return Some(HostNetwork::Range(IpRange {
                    start: start.to_string(),
                    end: end.to_string(),
                }));
            }
        }
    }

    None
}

pub fn validate_host_network(input: &str) -> (bool, Option<String>) {
    if Ipv4Addr::from_str(input).is_ok() {
        return (true, None);
    }

    if Ipv4Net::from_str(input).is_ok() {
        return (true, None);
    }

    if let Some(range) = validate_ip_range(input, '~') {
        return (true, Some(range));
    }
    if let Some(range) = validate_ip_range(input, '-') {
        return (true, Some(range));
    }
    if let Some(range) = validate_ip_range(input, ' ') {
        return (true, Some(range));
    }

    (false, None)
}

#[inline]
fn validate_ip_range(txt: &str, del: char) -> Option<String> {
    if let Some((ip_start, ip_end)) = txt.split_once(del) {
        let (ip_start, ip_end) = (ip_start.trim(), ip_end.trim());
        if let (Ok(start), Ok(end)) = (Ipv4Addr::from_str(ip_start), Ipv4Addr::from_str(ip_end)) {
            if start < end {
                return Some(format!("{} ~ {}", ip_start, ip_end));
            }
        }
    }
    None
}

pub(crate) fn text_width(text: &str, font: &str) -> Result<u32, ()> {
    let window = web_sys::window().expect("window should exist");
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .create_element("canvas")
        .map_err(|_| ())?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())?;

    if let Some(ctx) = canvas.get_context("2d").map_err(|_| ())? {
        let ctx = ctx
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| ())?;
        ctx.set_font(font);
        ctx.measure_text(text)
            .map_err(|_| ())?
            .width()
            .to_u32()
            .ok_or(())
    } else {
        Err(())
    }
}

pub(crate) fn shorten_text(item_org: &str, width: u32, font: &str, margin: u32) -> String {
    if item_org.len() > 4 {
        let mut sized_item = item_org.to_string();
        let item = item_org.as_bytes();
        for i in 4..item.len() {
            if let Ok(split) = std::str::from_utf8(&item[0..=i]) {
                if let Ok(w) = text_width(split, font) {
                    if width > (60 + margin) && w > width - (60 + margin) {
                        sized_item = format!("{}...", split);
                        break;
                    }
                }
            }
        }
        sized_item
    } else {
        item_org.to_string()
    }
}
//////////////////////

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndpointKind {
    Source,
    Destination,
    Both,
}

impl Default for EndpointKind {
    fn default() -> Self {
        Self::Both
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectionExtraInfo {
    Network(EndpointKind),
    Basic, // dead code
}

type RefSelectionExtraInfo = Rc<RefCell<Option<SelectionExtraInfo>>>;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct ComplexSelection {
    pub predefined: Rc<RefCell<Option<HashMap<String, RefSelectionExtraInfo>>>>,
    pub custom: Rc<RefCell<HashMap<String, RefSelectionExtraInfo>>>,
}

impl ComplexSelection {
    pub fn len(&self) -> (Option<usize>, usize) {
        if let (Ok(predefined), Ok(custom)) =
            (self.predefined.try_borrow(), self.custom.try_borrow())
        {
            predefined.as_ref().map_or_else(
                || (None, custom.len()),
                |predefined| (Some(predefined.len()), custom.len()),
            )
        } else {
            (Some(usize::MAX), usize::MAX)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == (Some(0), 0)
    }
}

#[derive(Deserialize, Serialize, Clone, Default, Eq, PartialEq)]
pub struct IpRange {
    pub start: String,
    pub end: String,
}

impl PartialOrd for IpRange {
    fn partial_cmp(&self, other: &IpRange) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IpRange {
    fn cmp(&self, other: &IpRange) -> Ordering {
        if let (Ok(l_start), Ok(l_end), Ok(r_start), Ok(r_end)) = (
            Ipv4Addr::from_str(&self.start),
            Ipv4Addr::from_str(&self.end),
            Ipv4Addr::from_str(&other.start),
            Ipv4Addr::from_str(&other.end),
        ) {
            match l_start.cmp(&r_start) {
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
                Ordering::Equal => l_end.cmp(&r_end),
            }
        } else {
            Ordering::Equal
        }
    }

    fn max(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Greater => self,
            _ => other,
        }
    }

    fn min(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Less => self,
            _ => other,
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        if self.cmp(&max) == Ordering::Greater {
            max
        } else if self.cmp(&min) == Ordering::Less {
            min
        } else {
            self
        }
    }
}

impl ToString for IpRange {
    fn to_string(&self) -> String {
        format!("{} ~ {}", &self.start, &self.end)
    }
}

impl IpRange {
    #[allow(dead_code)]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if let (Ok(start), Ok(end)) = (
            Ipv4Addr::from_str(&self.start),
            Ipv4Addr::from_str(&self.end),
        ) {
            if start < end {
                return true;
            }
        }
        false
    }
}

pub trait HostNetworkGroupTrait {
    fn is_valid(&self) -> bool {
        for h in self.hosts() {
            if Ipv4Addr::from_str(h).is_err() {
                return false;
            }
        }
        for n in self.networks() {
            if Ipv4Net::from_str(n).is_err() {
                return false;
            }
        }
        for r in &self.ranges() {
            if !r.is_valid() {
                return false;
            }
        }

        true
    }

    fn to_string_vec(&self) -> Vec<String> {
        let mut elems = Vec::<String>::new();
        for host in self.hosts() {
            elems.push(host.clone());
        }
        for network in self.networks() {
            elems.push(network.clone());
        }
        let ranges: Vec<String> = self.ranges().iter().map(ToString::to_string).collect();
        for range in &ranges {
            elems.push(range.clone());
        }
        elems
    }

    fn hosts(&self) -> &[String];
    fn networks(&self) -> &[String];
    fn ranges(&self) -> Vec<IpRange>;
}

pub fn sort_hosts(hosts: &mut Vec<String>) {
    hosts.sort_unstable_by_key(|h| {
        if let Ok(addr) = Ipv4Addr::from_str(h) {
            addr
        } else {
            Ipv4Addr::new(0, 0, 0, 0)
        }
    });
    hosts.dedup();
}

pub fn sort_networks(networks: &mut Vec<String>) {
    networks.sort_unstable_by_key(|n| {
        if let Ok(network) = Ipv4Net::from_str(n) {
            network
        } else {
            Ipv4Net::from_str("0.0.0.0/32").expect("0.0.0.0/32 can be Ipv4Net")
        }
    });
    networks.dedup();
}

#[derive(Clone, PartialEq, Eq)]
pub enum Item {
    // Network(Network),
    //Customer(Customer),
    KeyString(String, ViewString),
}

impl Item {
    pub fn key(&self) -> &String {
        match self {
            // Item::Customer(item) => &item.id,
            // Item::Network(item) => &item.id,
            Item::KeyString(key, _) => key,
        }
    }

    pub fn value(&self, txt: Option<(Rc<JSONGetText<'static>>, Language)>) -> String {
        match self {
            // Item::Customer(item) => item.name.clone(),
            // Item::Network(item) => item.name.clone(),
            Item::KeyString(_, ViewString::Raw(value)) => value.clone(),
            Item::KeyString(_, ViewString::Key(key)) => {
                txt.map_or_else(String::new, |(t, language)| {
                    get_text!(t, language.tag(), key).map_or_else(String::new, |t| t.to_string())
                })
            }
        }
    }
}

#[derive(Clone, Properties)]
pub struct Props {}

impl PartialEq for Props {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct HomeContext {
    pub token: Rc<String>,
    pub txt: Rc<JSONGetText<'static>>,
    pub link: Rc<Scope<Model>>,
    pub div: Rc<NodeRef>,
    //pub base_info: Rc<BaseInfo>,
}

impl PartialEq for HomeContext {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn home_context<T>(ctx: &Context<T>) -> HomeContext
where
    T: Component,
{
    let (home_ctx, _) = ctx
        .link()
        .context::<HomeContext>(Callback::noop())
        .expect("home context should exist");

    home_ctx
}
