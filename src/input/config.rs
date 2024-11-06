use std::{collections::HashMap, rc::Rc};

use super::{user_input_select::VecSelectListMap, HostNetworkKind, InputItem};
use crate::{CheckStatus, ViewString};

#[derive(Clone, PartialEq)]
pub struct Essential {
    pub title: String,
    pub notice: &'static str,
    pub required: bool,
    pub unique: bool, // for InputConfig::Text only. In other cases, this is meaningless.
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
    pub items: Vec<Rc<InputConfig>>,
}

#[derive(Clone, PartialEq)]
pub struct CheckBoxConfig {
    pub ess: Essential,
    // TODO: Check if the below second line is guaranteed.
    /// `Some(CheckStatus::{Checked|Unchecked|Indeterminate})` means this is always that status.
    /// This should not contradict with the result of all the configured status of children.
    pub always: Option<CheckStatus>,
    pub children: Option<(ChildrenPosition, Vec<Rc<InputConfig>>)>,
}

#[derive(Clone, PartialEq)]
pub struct RadioConfig {
    pub ess: Essential,
    pub options: Vec<ViewString>,
    /// The list of children group. Each option corresponds to one children of the group.
    pub children_group: Vec<Option<Vec<Rc<InputConfig>>>>,
}

#[derive(Clone, PartialEq)]
pub enum InputConfig {
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

impl InputConfig {
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