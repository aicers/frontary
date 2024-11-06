mod checkbox;
mod input;
mod ip_range_input;
pub mod language;
mod list;
mod modal;
mod notification;
mod pages;
mod port_range_input;
mod radio;
mod radio_separate;
mod select;
mod sort;
pub mod static_files;
mod svg;
mod tab_menu;
mod text_input;

use std::cell::RefCell;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv4Addr;
use std::rc::Rc;
use std::str::FromStr;

use anyhow::Result;
use ipnet::Ipv4Net;
use json_gettext::{get_text, JSONGetText};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use yew::Properties;

pub use crate::checkbox::{CheckStatus, Model as CheckBox};
pub use crate::input::{
    view_asterisk, CheckBoxConfig, ChildrenPosition, Comparison, ComparisonConfig, ComparisonItem,
    ComparisonKind, Essential as InputEssential, FileConfig, FileItem, Float64Config, Float64Item,
    GroupConfig, HostNetworkGroupConfig, HostNetworkGroupItem, HostNetworkHtml, HostNetworkKind,
    InputConfig, InputHostNetworkGroup, InputItem, InputNic, InputTag, InputTagGroup,
    Model as Input, NicConfig, NicItem, PasswordConfig, PasswordItem, PercentageConfig,
    PercentageItem, RadioConfig, SelectMultipleConfig, SelectMultipleItem, SelectSingleConfig,
    SelectSingleItem, Tag, TagConfig, TagItem, TextConfig, TextItem, Unsigned32Config,
    Unsigned32Item, Value as ComparisonValue, ValueKind, VecSelectConfig,
};
pub use crate::ip_range_input::Model as IpRangeInput;
pub use crate::language::Language;
pub use crate::list::{
    ColWidths, Column, DataType, DisplayInfo, Kind, ListItem, MessageType, SortColumn, WholeList,
};
pub use crate::modal::{
    AlignButton as ModalAlign, Model as Modal, MsgType as ModalType, TextStyle as ModalTextStyle,
};
pub use crate::notification::{
    gen_notifications, Category as NotificationCategory, CommonError, Model as Notification,
    NotificationItem, NotificationType, TIMEOUT_SECS,
};
pub use crate::pages::{Info as PagesInfo, Model as Pages};
pub use crate::port_range_input::{Model as PortRangeInput, PortRange};
pub use crate::radio::Model as Radio;
pub use crate::radio_separate::Model as RadioSeparate;
pub use crate::select::complex::{Kind as SelectComplexKind, Model as SelectComplex};
pub use crate::select::mini::{Kind as SelectMiniKind, Model as SelectMini};
pub use crate::select::searchable::{Kind as SelectSearchableKind, Model as SelectSearchable};
pub use crate::select::vec_searchable::Model as VecSelect;
pub use crate::sort::{Model as Sort, Status as SortStatus};
pub use crate::svg::Model as Svg;
pub use crate::tab_menu::Model as TabMenu;
pub use crate::text_input::Model as TextInput;

#[allow(clippy::missing_panics_doc)]
#[cfg(feature = "test")]
pub fn alert(msg: &str) {
    web_sys::window()
        .expect("Window should exist")
        .alert_with_message(msg)
        .expect("Alert should show up");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoreAction {
    Edit,
    Delete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnOffAction {
    On,
    Off,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViewString {
    Key(String),
    Raw(String),
}

impl fmt::Display for ViewString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Key(key) => write!(f, "{key}"),
            Self::Raw(txt) => write!(f, "{txt}"),
        }
    }
}

impl ViewString {
    #[must_use]
    pub fn to_string_txt(&self, txt: &JSONGetText<'static>, language: Language) -> String {
        match self {
            Self::Key(key) => {
                get_text!(txt, language.tag(), key).map_or_else(String::new, |t| t.to_string())
            }
            Self::Raw(raw) => raw.clone(),
        }
    }
}

pub enum HostNetwork {
    Host(String),
    Network(String),
    Range(IpRange),
}

#[must_use]
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

#[must_use]
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
                return Some(format!("{ip_start} ~ {ip_end}"));
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

    let Some(ctx) = canvas.get_context("2d").map_err(|_| ())? else {
        return Err(());
    };
    let ctx = ctx
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| ())?;
    ctx.set_font(font);
    ctx.measure_text(text)
        .map_err(|_| ())?
        .width()
        .to_u32()
        .ok_or(())
}

#[must_use]
pub fn shorten_text(item_org: &str, width: u32, font: &str, margin: u32) -> String {
    if item_org.len() > 4 {
        let mut sized_item = item_org.to_string();
        let item = item_org.as_bytes();
        for i in 4..item.len() {
            if let Ok(split) = std::str::from_utf8(&item[0..=i]) {
                if let Ok(w) = text_width(split, font) {
                    if width > (60 + margin) && w > width - (60 + margin) {
                        sized_item = format!("{split}...");
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectionExtraInfo {
    Network(EndpointKind),
    Basic,
}

pub type RefSelectionExtraInfo = Rc<RefCell<Option<SelectionExtraInfo>>>;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct ComplexSelection {
    pub predefined: Rc<RefCell<Option<HashMap<String, RefSelectionExtraInfo>>>>,
    pub custom: Rc<RefCell<HashMap<String, RefSelectionExtraInfo>>>,
}

impl ComplexSelection {
    #[must_use]
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

    #[must_use]
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

impl fmt::Display for IpRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ~ {}", self.start, self.end)
    }
}

impl IpRange {
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
        for r in self.ranges() {
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
        for range in self.ranges().iter().map(ToString::to_string) {
            elems.push(range.clone());
        }
        elems
    }

    fn hosts(&self) -> &[String];
    fn networks(&self) -> &[String];
    // should return Vec because most structs implementing this trait return a converted, i.e. newly created, Vec instead of a Vec field.
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

#[allow(clippy::missing_panics_doc)] // because it never happens
/// Sorts networks by the network address.
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkGroup {
    pub hosts: Vec<String>,
    pub networks: Vec<String>,
    pub ranges: Vec<IpRange>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    id: String,
    value: ViewString,
}

impl Item {
    #[must_use]
    pub fn new(id: String, value: ViewString) -> Self {
        Self { id, value }
    }

    #[must_use]
    pub fn id(&self) -> &String {
        &self.id
    }

    #[must_use]
    pub fn value(&self) -> String {
        self.value.to_string()
    }

    #[must_use]
    pub fn value_txt(&self, txt: &JSONGetText<'static>, language: Language) -> String {
        self.value.to_string_txt(txt, language)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkItem {
    id: String,
    value: ViewString,
    networks: Option<NetworkGroup>,
}

impl NetworkItem {
    #[must_use]
    pub fn new(id: String, value: ViewString, networks: Option<NetworkGroup>) -> Self {
        Self {
            id,
            value,
            networks,
        }
    }

    #[must_use]
    pub fn id(&self) -> &String {
        &self.id
    }

    #[must_use]
    pub fn value(&self) -> String {
        self.value.to_string()
    }

    #[must_use]
    pub fn value_txt(&self, txt: &JSONGetText<'static>, language: Language) -> String {
        self.value.to_string_txt(txt, language)
    }

    #[must_use]
    pub fn networks(&self) -> Option<&NetworkGroup> {
        self.networks.as_ref()
    }
}

#[derive(Clone, Properties)]
pub struct Props {}

impl PartialEq for Props {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
#[derive(Clone)]
pub struct Texts {
    pub txt: Rc<JSONGetText<'static>>,
}

impl PartialEq for Texts {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for Texts {}

const NBSP: &str = "&nbsp;";

#[wasm_bindgen(module = "/static/frontary/custom-select.js")]
extern "C" {
    fn toggle_visibility(id: &str);
    fn toggle_visibility_complex(id: &str);
    fn visible_tag_select(id: &str);
}

fn window_inner_height() -> u32 {
    web_sys::window()
        .expect("Window should exist")
        .inner_height()
        .expect("should have height")
        .as_f64()
        .expect("should be a number")
        .to_u32()
        .unwrap_or(u32::MAX)
}

trait Rerender {
    fn rerender_serial(&mut self) -> &mut u64;
    fn increase_rerender_serial(&mut self) {
        *self.rerender_serial() = self.rerender_serial().wrapping_add(1);
    }
}

#[macro_export]
macro_rules! define_str_consts {
    ($($name:ident => $value:expr),+) => {
        $(
            const $name: &str = $value;
        )+
    };
}

#[macro_export]
macro_rules! define_u32_consts {
    ($($name:ident => $value:expr),+) => {
        $(
            pub(crate) const $name: u32 = $value;
        )+
    };
}
