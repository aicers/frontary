#![allow(clippy::module_name_repetitions)]
mod component;
mod host_network;
mod recursive;
mod tag;
mod user_input;
mod user_input_comparison;
mod user_input_composite;
mod user_input_nic;
mod user_input_select;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt,
    rc::Rc,
};

use bincode::Options;
pub use component::{InputSecondId, Model};
pub use host_network::Kind as HostNetworkKind;
pub use host_network::Model as HostNetworkHtml;
use strum_macros::{Display, EnumIter, EnumString};
pub use tag::Model as Tag;

pub use self::user_input::view_asterisk;
use self::user_input_select::VecSelectListMap;
use crate::list::Column;
use crate::{
    parse_host_network, CheckStatus, HostNetwork, HostNetworkGroupTrait, IpRange, ViewString,
};

#[derive(Clone, PartialEq, Eq, Default)]
pub struct InputHostNetworkGroup {
    pub hosts: Vec<String>,
    pub networks: Vec<String>,
    pub ranges: Vec<IpRange>,
}
impl InputHostNetworkGroup {
    #[must_use]
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
        // should return Vec because most structs implementing this trait return a converted, i.e. newly created, Vec instead of a Vec field.
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
    pub interface: String,
    pub gateway: String,
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct InputTag {
    pub new: Option<String>,
    pub edit: Option<(String, String)>,
    pub delete: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct Essential {
    pub title: String,
    pub notice: &'static str,
    pub required: bool,
    pub unique: bool, // for InputType::Text only. In other cases, this is meaningless.
    pub default: Option<InputItem>, // in CheckBox, CheckStatus only should be set properly in hierarchical meaning
                                    // e.g. `default: Some(InputItem::CheckBox(CheckStatus::Checked, None))` where `children` is always set to `None` and `CheckStatus` only is set to a value
                                    // as for VecSelect, default should be like the below
                                    // let v = vec![HashSet::new(), HashSet::new()];
                                    // ess.default = Some(InputItem::VecSelect(v));
}

impl Essential {
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChildrenPosition {
    NextLine,
    Right,
}

#[derive(Clone, Copy, Display, EnumIter, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "PascalCase")]
pub enum ValueKind {
    String,
    Integer,
    Float,
}

#[derive(Clone, PartialEq)]
pub struct TextConfig {
    pub ess: Essential,
    pub length: Option<usize>,
    pub width: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub struct PasswordConfig {
    pub ess: Essential,
    pub width: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub struct HostNetworkGroupConfig {
    pub ess: Essential,
    pub kind: HostNetworkKind,
    /// The number of user inputs for `HostNetworkGroup`.
    pub num: Option<usize>,
    pub width: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub struct SelectSingleConfig {
    pub ess: Essential,
    /// The list of options for user selection. Each element is a tuple of key and display string.
    pub options: Vec<(String, ViewString)>,
    pub width: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub struct SelectMultipleConfig {
    pub ess: Essential,
    /// The list of options for user selection. Each element is a tuple of key and display string.
    pub options: Option<Vec<(String, ViewString)>>,
    /// Just in case of using the NIC list, the index of data's NIC.
    pub nic_index: Option<usize>,
    pub width: Option<u32>,
    /// This represents whether all options are selected by default.
    pub all: bool,
}

#[derive(Clone, PartialEq)]
pub struct VecSelectConfig {
    pub ess: Essential,
    pub items_ess_list: Vec<Essential>,
    /// Whether the last item is for selecting multiple items.
    pub last: bool,
    pub map_list: Vec<VecSelectListMap>,
    pub full_width: Option<u32>,
    /// The list of width for each item.
    pub widths: Vec<u32>,
    /// The list of max width for each item.
    pub max_widths: Vec<u32>,
    /// The list of max height for each item.
    pub max_heights: Vec<u32>,
}

#[derive(Clone, PartialEq)]
pub struct TagConfig {
    pub ess: Essential,
    /// The map of tag's key and name.
    pub name_map: HashMap<String, String>,
}

#[derive(Clone, PartialEq)]
pub struct Unsigned32Config {
    pub ess: Essential,
    pub min: u32,
    pub max: u32,
    pub width: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub struct Float64Config {
    pub ess: Essential,
    pub step: Option<f64>,
    pub width: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub struct PercentageConfig {
    pub ess: Essential,
    pub min: Option<f32>,
    pub max: Option<f32>,
    /// The number of decimal places.
    pub num_decimals: Option<usize>,
    pub width: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub struct NicConfig {
    pub ess: Essential,
}

#[derive(Clone, PartialEq)]
pub struct FileConfig {
    pub ess: Essential,
}

#[derive(Clone, PartialEq)]
pub struct ComparisonConfig {
    pub ess: Essential,
}

#[derive(Clone, PartialEq)]
pub struct GroupConfig {
    // TODO: This comment should be clarified.
    /// If `Essesntial::required` is set true, one valid row should be included at least. If one or
    /// more of columns in a given row are not empty, all the columns with `Essential::required ==
    /// true` cannot be empty.
    pub ess: Essential,
    /// If true, all items are displayed in one row. If false, each item is displayed in one row.
    pub all_in_one_row: bool,
    /// The list of width for each column. Some if fixed, None if not fixed.
    pub widths: Vec<Option<u32>>,
    pub items: Vec<Rc<InputType>>,
}

#[derive(Clone, PartialEq)]
pub struct CheckBoxConfig {
    pub ess: Essential,
    // TODO: Check if the below second line is guaranteed.
    /// `Some(CheckStatus::{Checked|Unchecked|Indeterminate})` means this is always that status.
    /// This should not contradict with the result of all the configured status of children.
    pub always: Option<CheckStatus>,
    pub children: Option<(ChildrenPosition, Vec<Rc<InputType>>)>,
}

#[derive(Clone, PartialEq)]
pub struct RadioConfig {
    pub ess: Essential,
    pub options: Vec<ViewString>,
    /// The list of children group. Each option corresponds to one children of the group.
    pub children_group: Vec<Option<Vec<Rc<InputType>>>>,
}

#[derive(Clone, PartialEq)]
pub enum InputType {
    Text(TextConfig),
    Password(PasswordConfig),
    HostNetworkGroup(HostNetworkGroupConfig),
    SelectSingle(SelectSingleConfig),
    SelectMultiple(SelectMultipleConfig),
    VecSelect(VecSelectConfig),
    Tag(TagConfig),
    Unsigned32(Unsigned32Config),
    Float64(Float64Config),
    Percentage(PercentageConfig),
    Nic(NicConfig),
    File(FileConfig),
    Comparison(ComparisonConfig),
    Group(GroupConfig),
    CheckBox(CheckBoxConfig),
    Radio(RadioConfig),
}

impl InputType {
    #[must_use]
    pub fn required(&self) -> bool {
        match self {
            Self::Text(config) => config.ess.required,
            Self::Password(config) => config.ess.required,
            Self::HostNetworkGroup(config) => config.ess.required,
            Self::SelectSingle(config) => config.ess.required,
            Self::SelectMultiple(config) => config.ess.required,
            Self::VecSelect(config) => config.ess.required,
            Self::Tag(config) => config.ess.required,
            Self::Unsigned32(config) => config.ess.required,
            Self::Float64(config) => config.ess.required,
            Self::Percentage(config) => config.ess.required,
            Self::Nic(config) => config.ess.required,
            Self::File(config) => config.ess.required,
            Self::Comparison(config) => config.ess.required,
            Self::Group(config) => config.ess.required,
            Self::CheckBox(config) => config.ess.required,
            Self::Radio(config) => config.ess.required,
        }
    }

    #[must_use]
    pub fn unique(&self) -> bool {
        match self {
            Self::Text(config) => config.ess.unique,
            Self::Password(config) => config.ess.unique,
            Self::HostNetworkGroup(config) => config.ess.unique,
            Self::SelectSingle(config) => config.ess.unique,
            Self::SelectMultiple(config) => config.ess.unique,
            Self::VecSelect(config) => config.ess.unique,
            Self::Tag(config) => config.ess.unique,
            Self::Unsigned32(config) => config.ess.unique,
            Self::Float64(config) => config.ess.unique,
            Self::Percentage(config) => config.ess.unique,
            Self::Nic(config) => config.ess.unique,
            Self::File(config) => config.ess.unique,
            Self::Comparison(config) => config.ess.unique,
            Self::Group(config) => config.ess.unique,
            Self::CheckBox(config) => config.ess.unique,
            Self::Radio(config) => config.ess.unique,
        }
    }

    #[must_use]
    pub fn title(&self) -> &str {
        match self {
            Self::Text(config) => config.ess.title(),
            Self::Password(config) => config.ess.title(),
            Self::HostNetworkGroup(config) => config.ess.title(),
            Self::SelectSingle(config) => config.ess.title(),
            Self::SelectMultiple(config) => config.ess.title(),
            Self::VecSelect(config) => config.ess.title(),
            Self::Tag(config) => config.ess.title(),
            Self::Unsigned32(config) => config.ess.title(),
            Self::Float64(config) => config.ess.title(),
            Self::Percentage(config) => config.ess.title(),
            Self::Nic(config) => config.ess.title(),
            Self::File(config) => config.ess.title(),
            Self::Comparison(config) => config.ess.title(),
            Self::Group(config) => config.ess.title(),
            Self::CheckBox(config) => config.ess.title(),
            Self::Radio(config, ..) => config.ess.title(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Value {
    String(Option<String>),
    Integer(Option<i64>),
    Float(Option<f64>),
}

impl Value {
    #[must_use]
    pub fn serialize(&self) -> Option<Vec<u8>> {
        match self {
            Self::String(Some(v)) => bincode::DefaultOptions::new().serialize(v).ok(),
            Self::Integer(Some(v)) => bincode::DefaultOptions::new().serialize(v).ok(),
            Self::Float(Some(v)) => bincode::DefaultOptions::new().serialize(v).ok(),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(Some(v)) => write!(f, "{v}"),
            Self::Integer(Some(v)) => write!(f, "{v}"),
            Self::Float(Some(v)) => write!(f, "{v}"),
            _ => write!(f, ""),
        }
    }
}

#[derive(Clone, Copy, Display, EnumString, Eq, PartialEq)]
pub enum ComparisonKind {
    #[strum(serialize = "x < a")]
    Less,
    #[strum(serialize = "x = a")]
    Equal,
    #[strum(serialize = "x > a")]
    Greater,
    #[strum(serialize = "x ≤ a")]
    LessOrEqual,
    #[strum(serialize = "x ≥ a")]
    GreaterOrEqual,
    #[strum(serialize = "x Contains a")]
    Contain,
    #[strum(serialize = "a < x < b")]
    OpenRange,
    #[strum(serialize = "a ≤ x ≤ b")]
    CloseRange,
    #[strum(serialize = "a < x ≤ b")]
    LeftOpenRange,
    #[strum(serialize = "a ≤ x < b")]
    RightOpenRange,
    #[strum(serialize = "x != a")]
    NotEqual,
    #[strum(serialize = "x !Contains a")]
    NotContain,
    #[strum(serialize = "!(a < x < b)")]
    NotOpenRange,
    #[strum(serialize = "!(a ≤ x ≤ b)")]
    NotCloseRange,
    #[strum(serialize = "!(a < x ≤ b)")]
    NotLeftOpenRange,
    #[strum(serialize = "!(a ≤ x < b)")]
    NotRightOpenRange,
}

impl ComparisonKind {
    fn chain_cmp(self) -> bool {
        !matches!(
            self,
            Self::Less
                | Self::Equal
                | Self::Greater
                | Self::LessOrEqual
                | Self::GreaterOrEqual
                | Self::Contain
                | Self::NotEqual
                | Self::NotContain
        )
    }
}

#[derive(Clone, PartialEq)]
pub enum Comparison {
    Less(Value),
    Equal(Value),
    Greater(Value),
    LessOrEqual(Value),
    GreaterOrEqual(Value),
    Contain(Value),
    OpenRange(Value, Value),      // a < x < b
    CloseRange(Value, Value),     // a <= x <= b
    LeftOpenRange(Value, Value),  // a < x <= b
    RightOpenRange(Value, Value), // a <= x < b
    NotEqual(Value),
    NotContain(Value),
    NotOpenRange(Value, Value),      // !(a < x < b)
    NotCloseRange(Value, Value),     // !(a <= x <= b)
    NotLeftOpenRange(Value, Value),  // !(a < x <= b)
    NotRightOpenRange(Value, Value), // !(a <= x < b)
}

impl fmt::Display for Comparison {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Less(v) => write!(f, "x < {v}"),
            Self::Equal(v) => write!(f, "x = {v}"),
            Self::Greater(v) => write!(f, "x > {v}"),
            Self::LessOrEqual(v) => write!(f, "x ≤ {v}"),
            Self::GreaterOrEqual(v) => write!(f, "x ≥ {v}"),
            Self::Contain(v) => write!(f, "x Contains {v}"),
            Self::OpenRange(a, b) => write!(f, "{a} < x < {b}"),
            Self::CloseRange(a, b) => write!(f, "{a} ≤ x ≤ {b}"),
            Self::LeftOpenRange(a, b) => write!(f, "{a} < x ≤ {b}"),
            Self::RightOpenRange(a, b) => write!(f, "{a} ≤ x < {b}"),
            Self::NotEqual(v) => write!(f, "x != {v}"),
            Self::NotContain(v) => write!(f, "x !Contains {v}"),
            Self::NotOpenRange(a, b) => write!(f, "!({a} < x < {b})"),
            Self::NotCloseRange(a, b) => write!(f, "!({a} ≤ x ≤ {b})"),
            Self::NotLeftOpenRange(a, b) => write!(f, "!({a} < x ≤ {b})"),
            Self::NotRightOpenRange(a, b) => write!(f, "!({a} ≤ x < {b})"),
        }
    }
}

#[derive(Debug)]
pub struct IncompletePairOfValues;

impl fmt::Display for IncompletePairOfValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Incomplete Pair of Values")
    }
}

impl std::error::Error for IncompletePairOfValues {}

impl Comparison {
    #[allow(clippy::missing_errors_doc)] // TODO: document later
    pub fn try_new(
        cmp: ComparisonKind,
        first: Value,
        second: Option<Value>,
    ) -> Result<Self, IncompletePairOfValues> {
        match cmp {
            ComparisonKind::Less => Ok(Self::Less(first)),
            ComparisonKind::Equal => Ok(Self::Equal(first)),
            ComparisonKind::Greater => Ok(Self::Greater(first)),
            ComparisonKind::LessOrEqual => Ok(Self::LessOrEqual(first)),
            ComparisonKind::GreaterOrEqual => Ok(Self::GreaterOrEqual(first)),
            ComparisonKind::Contain => Ok(Self::Contain(first)),
            ComparisonKind::NotEqual => Ok(Self::NotEqual(first)),
            ComparisonKind::NotContain => Ok(Self::NotContain(first)),
            ComparisonKind::OpenRange => {
                if let Some(second) = second {
                    Ok(Self::OpenRange(first, second))
                } else {
                    Err(IncompletePairOfValues)
                }
            }
            ComparisonKind::CloseRange => {
                if let Some(second) = second {
                    Ok(Self::CloseRange(first, second))
                } else {
                    Err(IncompletePairOfValues)
                }
            }
            ComparisonKind::LeftOpenRange => {
                if let Some(second) = second {
                    Ok(Self::LeftOpenRange(first, second))
                } else {
                    Err(IncompletePairOfValues)
                }
            }
            ComparisonKind::RightOpenRange => {
                if let Some(second) = second {
                    Ok(Self::RightOpenRange(first, second))
                } else {
                    Err(IncompletePairOfValues)
                }
            }
            ComparisonKind::NotOpenRange => {
                if let Some(second) = second {
                    Ok(Self::NotOpenRange(first, second))
                } else {
                    Err(IncompletePairOfValues)
                }
            }
            ComparisonKind::NotCloseRange => {
                if let Some(second) = second {
                    Ok(Self::NotCloseRange(first, second))
                } else {
                    Err(IncompletePairOfValues)
                }
            }
            ComparisonKind::NotLeftOpenRange => {
                if let Some(second) = second {
                    Ok(Self::NotLeftOpenRange(first, second))
                } else {
                    Err(IncompletePairOfValues)
                }
            }
            ComparisonKind::NotRightOpenRange => {
                if let Some(second) = second {
                    Ok(Self::NotRightOpenRange(first, second))
                } else {
                    Err(IncompletePairOfValues)
                }
            }
        }
    }

    #[must_use]
    pub fn value_kind(&self) -> ValueKind {
        match self {
            Self::Less(v)
            | Self::Equal(v)
            | Self::Greater(v)
            | Self::LessOrEqual(v)
            | Self::GreaterOrEqual(v)
            | Self::Contain(v)
            | Self::OpenRange(v, _)
            | Self::CloseRange(v, _)
            | Self::LeftOpenRange(v, _)
            | Self::RightOpenRange(v, _)
            | Self::NotEqual(v)
            | Self::NotContain(v)
            | Self::NotOpenRange(v, _)
            | Self::NotCloseRange(v, _)
            | Self::NotLeftOpenRange(v, _)
            | Self::NotRightOpenRange(v, _) => match v {
                Value::String(_) => ValueKind::String,
                Value::Integer(_) => ValueKind::Integer,
                Value::Float(_) => ValueKind::Float,
            },
        }
    }

    #[must_use]
    pub fn comparison_kind(&self) -> ComparisonKind {
        match self {
            Self::Less(..) => ComparisonKind::Less,
            Self::Equal(..) => ComparisonKind::Equal,
            Self::Greater(..) => ComparisonKind::Greater,
            Self::LessOrEqual(..) => ComparisonKind::LessOrEqual,
            Self::GreaterOrEqual(..) => ComparisonKind::GreaterOrEqual,
            Self::Contain(..) => ComparisonKind::Contain,
            Self::OpenRange(..) => ComparisonKind::OpenRange,
            Self::CloseRange(..) => ComparisonKind::CloseRange,
            Self::LeftOpenRange(..) => ComparisonKind::LeftOpenRange,
            Self::RightOpenRange(..) => ComparisonKind::RightOpenRange,
            Self::NotEqual(..) => ComparisonKind::NotEqual,
            Self::NotContain(..) => ComparisonKind::NotContain,
            Self::NotOpenRange(..) => ComparisonKind::NotOpenRange,
            Self::NotCloseRange(..) => ComparisonKind::NotCloseRange,
            Self::NotLeftOpenRange(..) => ComparisonKind::NotLeftOpenRange,
            Self::NotRightOpenRange(..) => ComparisonKind::NotRightOpenRange,
        }
    }

    #[must_use]
    pub fn first(&self) -> Value {
        match self {
            Self::Less(v)
            | Self::Equal(v)
            | Self::Greater(v)
            | Self::LessOrEqual(v)
            | Self::GreaterOrEqual(v)
            | Self::Contain(v)
            | Self::NotEqual(v)
            | Self::NotContain(v)
            | Self::OpenRange(v, _)
            | Self::CloseRange(v, _)
            | Self::LeftOpenRange(v, _)
            | Self::RightOpenRange(v, _)
            | Self::NotOpenRange(v, _)
            | Self::NotCloseRange(v, _)
            | Self::NotLeftOpenRange(v, _)
            | Self::NotRightOpenRange(v, _) => v.clone(),
        }
    }

    #[must_use]
    pub fn second(&self) -> Option<Value> {
        match self {
            Self::Less(_)
            | Self::Equal(_)
            | Self::Greater(_)
            | Self::LessOrEqual(_)
            | Self::GreaterOrEqual(_)
            | Self::Contain(_)
            | Self::NotEqual(_)
            | Self::NotContain(_) => None,
            Self::OpenRange(_, v)
            | Self::CloseRange(_, v)
            | Self::LeftOpenRange(_, v)
            | Self::RightOpenRange(_, v)
            | Self::NotOpenRange(_, v)
            | Self::NotCloseRange(_, v)
            | Self::NotLeftOpenRange(_, v)
            | Self::NotRightOpenRange(_, v) => Some(v.clone()),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum InputItem {
    Text(String),
    Password(String),
    HostNetworkGroup(InputHostNetworkGroup),
    SelectSingle(Option<String>),    // key
    SelectMultiple(HashSet<String>), // key
    VecSelect(Vec<HashSet<String>>), // key, this must be initialized as the same number of `HashSet::new()` as the number of `Select`
    Tag(InputTagGroup),
    Unsigned32(Option<u32>),
    Float64(Option<f64>),
    Percentage(Option<f32>),
    CheckBox(CheckStatus, Vec<Rc<RefCell<InputItem>>>), // Vec = children
    Radio(String, Vec<Vec<Rc<RefCell<InputItem>>>>),
    Nic(Vec<InputNic>),
    File(String, String), // (file name, base64 encoded content)
    Group(Vec<Vec<Rc<RefCell<InputItem>>>>),
    Comparison(Option<Comparison>),
}

impl InputItem {
    pub fn clear(&mut self) {
        match self {
            InputItem::Text(_) => *self = InputItem::Text(String::new()),
            InputItem::Password(_) => *self = InputItem::Password(String::new()),
            InputItem::HostNetworkGroup(group) => *group = InputHostNetworkGroup::default(),
            InputItem::SelectSingle(item) => *item = None,
            InputItem::SelectMultiple(list) => list.clear(),
            InputItem::VecSelect(list) => list.clear(),
            InputItem::Tag(group) => *group = InputTagGroup::default(),
            InputItem::Unsigned32(value) => *value = None,
            InputItem::Float64(value) => *value = None,
            InputItem::Percentage(value) => *value = None,
            InputItem::CheckBox(value, children) => {
                *value = CheckStatus::Unchecked;
                // if let Some(children) = children {
                for child in children {
                    if let Ok(mut child) = child.try_borrow_mut() {
                        child.clear();
                    }
                }
                // }
            }
            InputItem::Radio(value, children_group) => {
                *value = String::new();
                for children in children_group {
                    for child in children {
                        if let Ok(mut child) = child.try_borrow_mut() {
                            child.clear();
                        }
                    }
                }
            }
            InputItem::Nic(value) => value.clear(),
            InputItem::File(name, content) => {
                *name = String::new();
                *content = String::new();
            }
            InputItem::Group(group) => group.clear(),
            InputItem::Comparison(cmp) => *cmp = None,
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
            Column::SelectSingle(value) => Self::SelectSingle(value.as_ref().map(|d| d.0.clone())),
            Column::SelectMultiple(list) => {
                Self::SelectMultiple(list.keys().map(Clone::clone).collect::<HashSet<String>>())
            }
            Column::VecSelect(list) => {
                let list = list
                    .iter()
                    .map(|l| l.keys().map(Clone::clone).collect::<HashSet<String>>())
                    .collect::<Vec<_>>();
                Self::VecSelect(list)
            }
            Column::Tag(tags) => Self::Tag(InputTagGroup {
                old: tags.clone(),
                new: None,
                edit: None,
                delete: None,
            }),
            Column::Unsigned32(value) => Self::Unsigned32(*value),
            Column::Float64(value) => Self::Float64(*value),
            Column::Percentage(f, _) => Self::Percentage(*f),
            Column::Nic(nics) => Self::Nic(nics.clone()),
            Column::CheckBox(status, children, _) => Self::CheckBox(
                *status,
                children
                    .iter()
                    .map(|child| Rc::new(RefCell::new(InputItem::from(child))))
                    .collect::<Vec<Rc<RefCell<InputItem>>>>(),
            ),
            Column::Radio(option, children_group, _) => Self::Radio(
                option.to_string(),
                children_group
                    .iter()
                    .map(|(_, children)| {
                        children
                            .iter()
                            .map(|child| Rc::new(RefCell::new(InputItem::from(child))))
                            .collect::<Vec<Rc<RefCell<InputItem>>>>()
                    })
                    .collect::<_>(),
            ),
            Column::Group(group) => {
                let mut input: Vec<Vec<Rc<RefCell<InputItem>>>> = Vec::new();
                for g in group {
                    let mut input_row: Vec<Rc<RefCell<InputItem>>> = Vec::new();
                    for c in g {
                        match c {
                            Column::Text(..)
                            | Column::Unsigned32(..)
                            | Column::Float64(..)
                            | Column::SelectSingle(..)
                            | Column::VecSelect(..)
                            | Column::Comparison(..) => {
                                input_row.push(Rc::new(RefCell::new(c.into())));
                            }
                            _ => {}
                        }
                    }
                    input.push(input_row);
                }
                Self::Group(input)
            }
            Column::Comparison(value) => Self::Comparison(value.clone()),
        }
    }
}
