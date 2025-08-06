#![allow(clippy::module_name_repetitions)]
mod component;
mod config;
mod host_network;
mod item;
mod recursive;
mod tag;
mod user_input;
mod user_input_comparison;
mod user_input_composite;
mod user_input_nic;
mod user_input_select;

use core::panic;
use std::{cell::RefCell, collections::HashSet, fmt, net::IpAddr, rc::Rc, sync::LazyLock};

use bincode::Options;
pub use component::{InputSecondId, Model};
pub use config::{
    CheckboxChildrenConfig, CheckboxConfig, ChildrenPosition, ComparisonConfig, DomainNameConfig,
    Essential, FileConfig, Float64Config, GroupConfig, HostNetworkGroupConfig, InputConfig,
    NicConfig, PasswordConfig, PercentageConfig, RadioConfig, SelectMultipleConfig,
    SelectSingleConfig, TagConfig, TextConfig, Unsigned8Config, Unsigned32Config, ValidationError,
    ValidationRule, VecSelectConfig,
};
pub use host_network::Kind as HostNetworkKind;
pub use host_network::Model as HostNetworkHtml;
pub use item::{
    CheckboxItem, ComparisonItem, DomainNameItem, FileItem, Float64Item, GroupItem,
    HostNetworkGroupItem, InputItem, NicItem, PasswordItem, PercentageItem, RadioItem,
    SelectMultipleItem, SelectSingleItem, TagItem, TextItem, Unsigned8Item, Unsigned32Item,
    VecSelectItem,
};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
pub use recursive::invalid_password;
use strum_macros::{Display, EnumIter, EnumString};
pub use tag::Model as Tag;

pub use self::user_input::view_asterisk;
use crate::{CheckStatus, HostNetwork, HostNetworkGroupTrait, IpRange, parse_host_network};

const POWER_OF_MAX_NUM_OF_LAYER: u32 = 6; // 2^6 = 64 is the maximum number of items in a layer.
static MAX_NUM_OF_LAYER: LazyLock<BigUint> =
    LazyLock::new(|| BigUint::from(2_u32.pow(POWER_OF_MAX_NUM_OF_LAYER)));

#[cfg(test)]
mod tests {
    use num_traits::ToPrimitive;
    fn cal_index_first_ver(base_index: Option<usize>, layer_index: usize) -> usize {
        // `base_index` means parent's index
        if let Some(base_index) = base_index {
            let max = 2_u32
                .pow(super::POWER_OF_MAX_NUM_OF_LAYER)
                .to_f64()
                .expect("usize to f64 is safe.");
            let base = base_index.to_f64().expect("usize to f64 is safe.");
            let base = base.log(max).floor();
            let Some(base) = base.to_u32() else {
                panic!("Too many levels in hierarchy of input items");
            };
            let base = 2_usize.pow(super::POWER_OF_MAX_NUM_OF_LAYER).pow(base + 1);
            base_index + base * (1 + layer_index)
        } else {
            layer_index
        }
    }

    fn cal_index_with_bit_op(base_index: Option<usize>, layer_index: usize) -> usize {
        if let Some(base_index) = base_index {
            let base = (63 - base_index.leading_zeros()) / super::POWER_OF_MAX_NUM_OF_LAYER;
            let base = 1 << (super::POWER_OF_MAX_NUM_OF_LAYER * (base + 1));
            base_index + base * (1 + layer_index)
        } else {
            layer_index
        }
    }

    #[test]
    fn cal_index_test() {
        assert_eq!(cal_index_first_ver(Some(4), 0), 68);
        assert_eq!(
            cal_index_first_ver(Some(790_596_usize), 1),
            34_345_028_usize
        );
    }

    #[test]
    fn cal_index_bit_test() {
        assert_eq!(cal_index_with_bit_op(Some(4), 0), 68);
        assert_eq!(
            cal_index_with_bit_op(Some(790_596_usize), 1),
            34_345_028_usize
        );
    }

    #[test]
    fn cal_index_big_test() {
        assert_eq!(
            super::cal_index(Some(&num_bigint::BigUint::from(4_u32)), 0),
            num_bigint::BigUint::from(68_u32)
        );
        assert_eq!(
            super::cal_index(Some(&num_bigint::BigUint::from(790_596_u32)), 1),
            num_bigint::BigUint::from(34_345_028_u32)
        );
    }
}

fn cal_index(base_index: Option<&BigUint>, layer_index: usize) -> BigUint {
    if let Some(base_index) = base_index {
        let bits = base_index.bits();
        let bits = if bits == 0 { 1 } else { bits - 1 };
        let base = bits
            / POWER_OF_MAX_NUM_OF_LAYER
                .to_u64()
                .expect("u32 to u64 is safe.");
        let Some(base) = base.to_u32() else {
            panic!("Too many levels in hierarchy of input items.");
        };
        let base = MAX_NUM_OF_LAYER.pow(base + 1);
        base_index + base * (BigUint::from(1u32) + BigUint::from(layer_index))
    } else {
        BigUint::from(layer_index)
    }
}

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

    pub fn clear(&mut self) {
        self.hosts.clear();
        self.networks.clear();
        self.ranges.clear();
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

impl InputTagGroup {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.old.is_empty() && self.new.is_none() && self.edit.is_none() && self.delete.is_none()
    }

    pub fn clear(&mut self) {
        self.old.clear();
        self.new = None;
        self.edit = None;
        self.delete = None;
    }
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

#[derive(Clone, Copy, Display, EnumIter, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "PascalCase")]
pub enum ValueKind {
    String,
    Integer,
    UInteger,
    Vector,
    Float,
    IpAddr,
    Bool,
}

#[derive(Clone, PartialEq)]
pub enum Value {
    String(Option<String>),
    Integer(Option<i64>),
    UInteger(Option<u64>),
    Vector(Option<Vec<u8>>),
    Float(Option<f64>),
    IpAddr(Option<IpAddr>),
    Bool(Option<bool>),
}

impl Value {
    #[must_use]
    pub fn serialize(&self) -> Option<Vec<u8>> {
        match self {
            Self::String(Some(v)) => bincode::DefaultOptions::new().serialize(v).ok(),
            Self::Integer(Some(v)) => bincode::DefaultOptions::new().serialize(v).ok(),
            Self::UInteger(Some(v)) => bincode::DefaultOptions::new().serialize(v).ok(),
            Self::Vector(Some(v)) => bincode::DefaultOptions::new().serialize(v).ok(),
            Self::Float(Some(v)) => bincode::DefaultOptions::new().serialize(v).ok(),
            Self::IpAddr(Some(v)) => bincode::DefaultOptions::new().serialize(v).ok(),
            Self::Bool(Some(v)) => bincode::DefaultOptions::new().serialize(v).ok(),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(Some(v)) => write!(f, "{v}"),
            Self::Integer(Some(v)) => write!(f, "{v}"),
            Self::UInteger(Some(v)) => write!(f, "{v}"),
            Self::Vector(Some(v)) => write!(f, "{v:?}",),
            Self::Float(Some(v)) => write!(f, "{v}"),
            Self::IpAddr(Some(v)) => write!(f, "{v}"),
            Self::Bool(Some(v)) => write!(f, "{v}"),
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
    // TODO: issue #183
    #[allow(clippy::missing_errors_doc)]
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
                Value::UInteger(_) => ValueKind::UInteger,
                Value::Vector(_) => ValueKind::Vector,
                Value::Float(_) => ValueKind::Float,
                Value::IpAddr(_) => ValueKind::IpAddr,
                Value::Bool(_) => ValueKind::Bool,
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

#[must_use]
pub fn gen_default_items_from_confs(confs: &[Rc<InputConfig>]) -> Vec<Rc<RefCell<InputItem>>> {
    default_items(confs, 0)
}

fn default_items(confs: &[Rc<InputConfig>], level: usize) -> Vec<Rc<RefCell<InputItem>>> {
    confs
        .iter()
        .map(|conf| {
            Rc::new(RefCell::new(match &**conf {
                InputConfig::Text(_) => InputItem::Text(TextItem::default()),
                InputConfig::DomainName(_) => InputItem::DomainName(DomainNameItem::default()),
                InputConfig::Password(_) => InputItem::Password(PasswordItem::default()),
                InputConfig::HostNetworkGroup(_) => {
                    InputItem::HostNetworkGroup(HostNetworkGroupItem::default())
                }
                InputConfig::SelectSingle(_) => {
                    InputItem::SelectSingle(SelectSingleItem::default())
                }
                InputConfig::SelectMultiple(_) => {
                    InputItem::SelectMultiple(SelectMultipleItem::default())
                }
                InputConfig::Tag(_) => InputItem::Tag(TagItem::default()),
                InputConfig::Unsigned32(_) => InputItem::Unsigned32(Unsigned32Item::default()),
                InputConfig::Unsigned8(_) => InputItem::Unsigned8(Unsigned8Item::default()),
                InputConfig::Float64(_) => InputItem::Float64(Float64Item::default()),
                InputConfig::Percentage(_) => InputItem::Percentage(PercentageItem::default()),
                InputConfig::Nic(_) => InputItem::Nic(NicItem::default()),
                InputConfig::File(_) => InputItem::File(FileItem::default()),
                InputConfig::Comparison(_) => InputItem::Comparison(ComparisonItem::default()),
                InputConfig::VecSelect(conf) => InputItem::VecSelect(VecSelectItem::new(vec![
                        HashSet::new();
                        conf.items_ess_list.len()
                    ])),
                InputConfig::Group(conf) => {
                    if level == 0 {
                        let items = vec![default_items(&conf.items, level + 1)];
                        InputItem::Group(GroupItem::new(items))
                    } else {
                        InputItem::Group(GroupItem::default())
                    }
                }
                InputConfig::Checkbox(conf) => {
                    if let Some(children) = conf.children.as_ref() {
                        if children.children.is_empty() {
                            InputItem::Checkbox(CheckboxItem::default())
                        } else {
                            InputItem::Checkbox(CheckboxItem::default_with_children(default_items(
                                &children.children,
                                level + 1,
                            )))
                        }
                    } else {
                        InputItem::Checkbox(CheckboxItem::default())
                    }
                }
                InputConfig::Radio(conf) => {
                    if conf.children_group.is_empty() {
                        InputItem::Radio(RadioItem::default())
                    } else {
                        let children = conf
                            .children_group
                            .iter()
                            .map(|c| {
                                c.as_ref()
                                    .map_or_else(Vec::new, |c| default_items(c, level + 1))
                            })
                            .collect::<Vec<_>>();
                        InputItem::Radio(RadioItem::default_with_children(children))
                    }
                }
            }))
        })
        .collect::<Vec<_>>()
}

fn group_item_list_preset(confs: &[Rc<InputConfig>]) -> Vec<Rc<RefCell<InputItem>>> {
    confs
        .iter()
        .map(|conf| Rc::new(RefCell::new(item_preset(conf))))
        .collect::<Vec<_>>()
}

fn item_preset(conf: &Rc<InputConfig>) -> InputItem {
    match &**conf {
        InputConfig::Text(conf) => InputItem::Text(TextItem::new(
            conf.preset.as_deref().unwrap_or_default().to_string(),
        )),
        InputConfig::DomainName(conf) => InputItem::DomainName(DomainNameItem::new(
            conf.preset.as_deref().unwrap_or_default().to_string(),
        )),
        InputConfig::HostNetworkGroup(_) => {
            InputItem::HostNetworkGroup(HostNetworkGroupItem::new(InputHostNetworkGroup::default()))
        }
        InputConfig::SelectSingle(conf) => {
            InputItem::SelectSingle(SelectSingleItem::new(conf.preset.clone()))
        }
        InputConfig::SelectMultiple(conf) => InputItem::SelectMultiple(SelectMultipleItem::new(
            conf.preset.as_ref().map_or_else(HashSet::new, |p| {
                p.iter().cloned().collect::<HashSet<String>>()
            }),
        )),
        InputConfig::Unsigned32(conf) => InputItem::Unsigned32(Unsigned32Item::new(conf.preset)),
        InputConfig::Unsigned8(conf) => InputItem::Unsigned8(Unsigned8Item::new(conf.preset)),
        InputConfig::Float64(conf) => InputItem::Float64(Float64Item::new(conf.preset)),
        InputConfig::Percentage(conf) => InputItem::Percentage(PercentageItem::new(conf.preset)),
        InputConfig::Comparison(_) => InputItem::Comparison(ComparisonItem::new(None)),
        InputConfig::VecSelect(config) => {
            InputItem::VecSelect(VecSelectItem::new(config.preset.as_ref().map_or_else(
                || vec![HashSet::new(); config.items_ess_list.len()],
                Clone::clone,
            )))
        }
        InputConfig::Password(_)
        | InputConfig::Tag(_)
        | InputConfig::Nic(_)
        | InputConfig::File(_)
        | InputConfig::Group(_)
        | InputConfig::Checkbox(_)
        | InputConfig::Radio(_) => {
            panic!(
                "Input Group does not support some items such as Password, Tag, Nic, File, Group, Checkbox, and Radio."
            )
        }
    }
}
