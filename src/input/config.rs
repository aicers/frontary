use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{user_input_select::VecSelectListMap, HostNetworkKind};
use crate::{CheckStatus, ViewString};

#[derive(Clone, PartialEq)]
pub struct Essential {
    pub title: String,
    pub notice: &'static str,
    pub required: bool,
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
    pub preset: Option<String>,
    pub unique: bool,
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
    pub preset: Option<String>,
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
    pub preset: Option<Vec<String>>,
}

/// `Tag` items must belong to the top level of the hierarchy. It cannot be a child of `Checkbox` or
/// `Radio`, and cannot be an item of `VecSelect` or `Group`.
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
    pub preset: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub struct Float64Config {
    pub ess: Essential,
    pub step: Option<f64>,
    pub width: Option<u32>,
    pub preset: Option<f64>,
}

#[derive(Clone, PartialEq)]
pub struct PercentageConfig {
    pub ess: Essential,
    pub min: Option<f32>,
    pub max: Option<f32>,
    /// The number of decimal places.
    pub num_decimals: Option<usize>,
    pub width: Option<u32>,
    pub preset: Option<f32>,
}

#[derive(Clone, PartialEq)]
pub struct NicConfig {
    pub ess: Essential,
}

#[derive(Clone, PartialEq)]
pub struct FileConfig {
    pub ess: Essential,
    pub allowed_extensions: Vec<String>,
}

#[derive(Clone, PartialEq)]
pub struct ComparisonConfig {
    pub ess: Essential,
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
    pub preset: Option<Vec<HashSet<String>>>,
}

#[derive(Clone, PartialEq)]
pub struct GroupConfig {
    // TODO: issue #183 (This comment should be clarified.)
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
pub struct CheckboxChildrenConfig {
    pub position: ChildrenPosition,
    pub children: Vec<Rc<InputConfig>>,
}

#[derive(Clone, PartialEq)]
pub struct CheckboxConfig {
    pub ess: Essential,
    /// `Some(CheckStatus::{Checked|Unchecked|Indeterminate})` means this is always that status.
    /// This `always` always shows but might contradict with the result of children.
    pub always: Option<CheckStatus>,
    pub children: Option<CheckboxChildrenConfig>,
    pub preset: Option<CheckStatus>,
}

#[derive(Clone, PartialEq)]
pub struct RadioConfig {
    pub ess: Essential,
    pub options: Vec<ViewString>,
    /// The list of children group. Each option corresponds to one children of the group.
    pub children_group: Vec<Option<Vec<Rc<InputConfig>>>>,
    pub preset: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum InputConfig {
    Text(TextConfig),
    Password(PasswordConfig),
    HostNetworkGroup(HostNetworkGroupConfig),
    SelectSingle(SelectSingleConfig),
    SelectMultiple(SelectMultipleConfig),
    Tag(TagConfig),
    Unsigned32(Unsigned32Config),
    Float64(Float64Config),
    Percentage(PercentageConfig),
    Nic(NicConfig),
    File(FileConfig),
    Comparison(ComparisonConfig),
    VecSelect(VecSelectConfig),
    Group(GroupConfig),
    Checkbox(CheckboxConfig),
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
            Self::Tag(config) => config.ess.required,
            Self::Unsigned32(config) => config.ess.required,
            Self::Float64(config) => config.ess.required,
            Self::Percentage(config) => config.ess.required,
            Self::Nic(config) => config.ess.required,
            Self::File(config) => config.ess.required,
            Self::Comparison(config) => config.ess.required,
            Self::VecSelect(config) => config.ess.required,
            Self::Group(config) => config.ess.required,
            Self::Checkbox(config) => config.ess.required,
            Self::Radio(config) => config.ess.required,
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
            Self::Tag(config) => config.ess.title(),
            Self::Unsigned32(config) => config.ess.title(),
            Self::Float64(config) => config.ess.title(),
            Self::Percentage(config) => config.ess.title(),
            Self::Nic(config) => config.ess.title(),
            Self::File(config) => config.ess.title(),
            Self::Comparison(config) => config.ess.title(),
            Self::VecSelect(config) => config.ess.title(),
            Self::Group(config) => config.ess.title(),
            Self::Checkbox(config) => config.ess.title(),
            Self::Radio(config, ..) => config.ess.title(),
        }
    }
}
