#![allow(clippy::module_name_repetitions)]
mod component;
mod host_network;
mod recursive;
mod tag;
mod user_input;

pub use component::{InputSecondId, Model};
pub use host_network::Kind as HostNetworkKind;
pub use host_network::Model as HostNetworkHtml;
pub use tag::Model as Tag;

use crate::home::{
    parse_host_network, CheckStatus, HostNetwork, HostNetworkGroupTrait, IpRange, ViewString,
};
use crate::list::Column;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Default)]
pub struct InputHostNetworkGroup {
    pub hosts: Vec<String>,
    pub networks: Vec<String>,
    pub ranges: Vec<IpRange>,
}

impl InputHostNetworkGroup {
    pub fn is_empty(&self) -> bool {
        self.hosts.is_empty() && self.networks.is_empty() && self.ranges.is_empty()
    }
}

impl HostNetworkGroupTrait for InputHostNetworkGroup {
    fn hosts(&self) -> &[String] {
        &self.hosts
    }
    fn networks(&self) -> &[String] {
        &self.networks
    }
    fn ranges(&self) -> Vec<IpRange> {
        self.ranges.clone()
    }
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct InputTagGroup {
    pub old: HashSet<String>,           // keys from review
    pub new: Option<String>,            // the name of a tag input by users
    pub edit: Option<(String, String)>, // (the key, a new name)
    pub delete: Option<String>,         // the key that users want to be deleted
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct InputNic {
    pub name: String,
    pub interface_ip: String,
    pub gateway_ip: String,
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct InputTag {
    pub new: Option<String>,
    pub edit: Option<(String, String)>,
    pub delete: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct Essential {
    pub title: &'static str,
    pub notice: &'static str,
    pub required: bool, // in CheckBox/Radio, this is meaningless so can be an arbitrary value
    pub unique: bool,   // in CheckBox/Radio, this is meaningless so can be an arbitrary value
    pub default: Option<InputItem>, // in CheckBox, CheckStatus only should be set properly in hierarchical meaning
                                    // e.g. `default: Some(InputItem::CheckBox(CheckStatus::Checked, None))` where `children` is always set to `None` and `CheckStatus` only is set to a value
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChildrenPosition {
    NextLine,
    Right,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum InputType {
    Text(Essential, Option<usize>, Option<u32>), // (length, width)
    Password(Essential, Option<u32>),
    Radio(Essential, Vec<ViewString>),
    HostNetworkGroup(Essential, HostNetworkKind, Option<usize>, Option<u32>), // (usize, u32) = (# of input, width)
    SelectSingle(Essential, Vec<(String, ViewString)>), // (String, ViewString) = (key, display)
    SelectMultiple(Essential, Vec<(String, ViewString)>, bool), // (String, ViewString) = (key, display), bool = all selected by default
    Tag(Essential, HashMap<String, String>), // (String, String) = (key, tag value(name))
    Unsigned32(Essential, u32, u32, Option<u32>), // (u32, u32, Option<u32>) = (min, max, width)
    CheckBox(
        Essential,
        Option<CheckStatus>, // if whether always checked/unchecked/indeterminate, Some(CheckStatus::*)
        Option<(ChildrenPosition, Vec<Rc<InputType>>)>, // children
    ),
    // HIGHLIGHT: If an item is set to always something, all of its children should be set to the same.
    Nic(Essential),
    File(Essential),
}

impl InputType {
    pub fn required(&self) -> bool {
        match self {
            Self::Text(ess, _, _)
            | Self::Password(ess, _)
            | Self::Radio(ess, _)
            | Self::HostNetworkGroup(ess, _, _, _)
            | Self::SelectSingle(ess, _)
            | Self::SelectMultiple(ess, _, _)
            | Self::Tag(ess, _)
            | Self::Unsigned32(ess, _, _, _)
            | Self::CheckBox(ess, _, _)
            | Self::Nic(ess)
            | Self::File(ess) => ess.required,
        }
    }

    pub fn unique(&self) -> bool {
        match self {
            Self::Text(ess, _, _)
            | Self::Password(ess, _)
            | Self::Radio(ess, _)
            | Self::HostNetworkGroup(ess, _, _, _)
            | Self::SelectSingle(ess, _)
            | Self::SelectMultiple(ess, _, _)
            | Self::Tag(ess, _)
            | Self::Unsigned32(ess, _, _, _)
            | Self::CheckBox(ess, _, _)
            | Self::Nic(ess)
            | Self::File(ess) => ess.unique,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum InputItem {
    Text(String), // includes InputType::Radio
    Password(String),
    HostNetworkGroup(InputHostNetworkGroup),
    SelectSingle(Option<String>),    // key
    SelectMultiple(HashSet<String>), // key
    Tag(InputTagGroup),
    Unsigned32(Option<u32>),
    CheckBox(CheckStatus, Option<Vec<Rc<RefCell<InputItem>>>>),
    Nic(Vec<InputNic>),
    File(String, String), // (file name, base64 encoded content)
}

impl InputItem {
    pub fn clear(&mut self) {
        match self {
            InputItem::Text(_) => *self = InputItem::Text(String::new()),
            InputItem::Password(_) => *self = InputItem::Password(String::new()),
            InputItem::HostNetworkGroup(group) => *group = InputHostNetworkGroup::default(),
            InputItem::SelectSingle(item) => *item = None,
            InputItem::SelectMultiple(list) => list.clear(),
            InputItem::Tag(group) => *group = InputTagGroup::default(),
            InputItem::Unsigned32(value) => *value = None,
            InputItem::CheckBox(value, _) => *value = CheckStatus::Unchecked,
            InputItem::Nic(value) => value.clear(),
            InputItem::File(name, content) => {
                *name = String::new();
                *content = String::new();
            }
        }
    }
}

impl From<&Column> for InputItem {
    fn from(col: &Column) -> Self {
        match col {
            Column::Text(txt) => Self::Text(txt.to_string()),
            Column::HostNetworkGroup(items) => {
                let mut input = InputHostNetworkGroup::default();
                for item in items {
                    match parse_host_network(item) {
                        Some(HostNetwork::Host(host)) => input.hosts.push(host),
                        Some(HostNetwork::Network(network)) => input.networks.push(network),
                        Some(HostNetwork::Range(range)) => input.ranges.push(range),
                        _ => (),
                    }
                }
                Self::HostNetworkGroup(input)
            }
            Column::KeyValueList(list) => {
                Self::SelectMultiple(list.keys().map(Clone::clone).collect::<HashSet<String>>())
            }
            Column::Tag(tags) => Self::Tag(InputTagGroup {
                old: tags.clone(),
                new: None,
                edit: None,
                delete: None,
            }),
            Column::Unsigned32(value) => Self::Unsigned32(*value),
        }
    }
}
