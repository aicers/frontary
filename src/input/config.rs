use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{HostNetworkKind, user_input_select::VecSelectListMap};
use crate::{CheckStatus, Theme, ViewString};

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

pub(super) type ValidationFn = fn(&str) -> Result<(), String>;

#[derive(Clone)]
pub struct TextConfig {
    pub ess: Essential,
    pub length: Option<usize>,
    pub width: Option<u32>,
    pub preset: Option<String>,
    pub unique: bool,
    pub immutable: bool,
    pub validation: Option<ValidationFn>,
}

impl PartialEq for TextConfig {
    fn eq(&self, other: &Self) -> bool {
        self.ess == other.ess
            && self.length == other.length
            && self.width == other.width
            && self.preset == other.preset
            && self.unique == other.unique
            && self.immutable == other.immutable
    }
}

#[derive(Clone, PartialEq)]
pub struct DomainNameConfig {
    pub ess: Essential,
    pub width: Option<u32>,
    pub preset: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct PasswordConfig {
    pub ess: Essential,
    pub length: Option<usize>,
    pub width: Option<u32>,
}

#[derive(Clone)]
pub struct HostNetworkGroupConfig {
    pub ess: Essential,
    pub kind: HostNetworkKind,
    /// The number of user inputs for `HostNetworkGroup`.
    pub num: Option<usize>,
    pub width: Option<u32>,
    pub theme: Option<Theme>,
    pub length: Option<usize>,
    pub validation: Option<ValidationFn>,
}

impl PartialEq for HostNetworkGroupConfig {
    fn eq(&self, other: &Self) -> bool {
        self.ess == other.ess
            && self.kind == other.kind
            && self.num == other.num
            && self.width == other.width
            && self.theme == other.theme
            && self.length == other.length
    }
}

#[derive(Clone, PartialEq)]
pub struct SelectSingleConfig {
    pub ess: Essential,
    /// The list of options for user selection. Each element is a tuple of key and display string.
    pub options: Vec<(String, ViewString)>,
    pub width: Option<u32>,
    pub preset: Option<String>,
    pub theme: Option<Theme>,
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
    pub theme: Option<Theme>,
}

/// `TagConfig` defines how `InputItem::Tag` works. `InputItem::Tag` items must belong to the top
/// level of the hierarchy. It cannot be a child of `Checkbox` or `Radio`, and cannot be an item of
/// `VecSelect` or `Group`.
#[derive(Clone, PartialEq)]
pub struct TagConfig {
    pub ess: Essential,
    /// The map of tag's key and name.
    pub name_map: HashMap<String, String>,
    pub theme: Option<Theme>,
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
pub struct Unsigned8Config {
    pub ess: Essential,
    pub min: u8,
    pub max: u8,
    pub width: Option<u8>,
    pub preset: Option<u8>,
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
    /// A list of allowed file extensions.
    /// Each extension should start with a `.` (e.g. `.txt`, `.csv`).
    /// Extensions must not contain spaces or invalid characters (e.g. `!`, `@`, `#`).
    /// If an extension does not start with a `.`, one will be automatically added.
    pub allowed_extensions: Vec<String>,
}

#[derive(Clone, PartialEq)]
pub struct ComparisonConfig {
    // TODO: #183
    pub ess: Essential,
}

#[derive(Clone, PartialEq)]
pub struct VecSelectConfig {
    // TODO: #183
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

/// `GroupConfig` defines how `InputItem::Group` works. `InputItem::Group` handles multiple items in
/// one group. It can include `Text`, `HostNetworkGroup`, `SelectSingle`, `SelectMultiple`,
/// `Unsigned32`, `Float64`, `Percentage`, `Comparison`, and `VecSelect`. The other types such as
/// `Password`, `Tag`, `Nic`, `File`, `Group`, `Checkbox`, and `Radio` are not allowed. If
/// `Essential::required` of `GroupConfig` is set to `true`, at least one valid row must be
/// included. Rows where all columns are empty do not affect the validation of either the row or the
/// entire group; users should handle rows where all columns are empty. However, if one or more
/// columns in a given row are not empty, any column with `Essential::required == true` must be
/// filled.
#[derive(Clone, PartialEq)]
pub struct GroupConfig {
    pub ess: Essential,
    /// If true, all items are displayed in one row. If false, each item is displayed in one row.
    pub all_in_one_row: bool,
    /// The list of width for each column. Some if fixed, None if not fixed.
    pub widths: Vec<Option<u32>>,
    pub items: Vec<Rc<InputConfig>>,
    pub compact: bool,
    pub theme: Option<Theme>,
}

#[derive(Clone, PartialEq)]
pub struct CheckboxChildrenConfig {
    pub position: ChildrenPosition,
    pub children: Vec<Rc<InputConfig>>,
}

/// `CheckboxConfig` defines how `InputItem::Checkbox` works. `InputItem::Checkbox` is a special
/// item that can have child items recursively. As its children, it supports `Text`,
/// `HostNetworkGroup`, `SelectSingle`, `SelectMultiple`, `Unsigned32`, `Float64`, `Percentage`,
/// `Group`, `Checkbox`, and `Radio`. The other types, such as `Password`, `Tag`, `Nic`, `File`,
/// `VecSelect`, and `Comparison`, are not allowed.
#[derive(Clone, PartialEq)]
pub struct CheckboxConfig {
    pub ess: Essential,
    /// If true, `ess::title` is displayed accrording to the language set by the user.
    /// If false, it is displayed in English.
    pub language: bool,
    /// `Some(CheckStatus::{Checked|Unchecked|Indeterminate})` means this is always that status.
    /// This `always` always shows but might contradict with the result of children.
    pub always: Option<CheckStatus>,
    pub children: Option<CheckboxChildrenConfig>,
    pub preset: Option<CheckStatus>,
    pub theme: Option<Theme>,
}

/// `RadioConfig` defines how `InputItem::Radio` works. `InputItem::Radio` has child items like
/// `InputItem::Checkbox` in addition to its own option. As its children, `InputItem::Radio`
/// supports the same items as `InputItem::Checkbox` does.
#[derive(Clone, PartialEq)]
pub struct RadioConfig {
    pub ess: Essential,
    pub options: Vec<ViewString>,
    /// The list of children group. Each option corresponds to one children of the group.
    pub children_group: Vec<Option<Vec<Rc<InputConfig>>>>,
    pub preset: Option<String>,
    pub theme: Option<Theme>,
}

#[derive(Clone, PartialEq)]
pub enum InputConfig {
    Text(TextConfig),
    DomainName(DomainNameConfig),
    Password(PasswordConfig),
    HostNetworkGroup(HostNetworkGroupConfig),
    SelectSingle(SelectSingleConfig),
    SelectMultiple(SelectMultipleConfig),
    Tag(TagConfig),
    Unsigned32(Unsigned32Config),
    Unsigned8(Unsigned8Config),
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
            Self::DomainName(config) => config.ess.required,
            Self::Password(config) => config.ess.required,
            Self::HostNetworkGroup(config) => config.ess.required,
            Self::SelectSingle(config) => config.ess.required,
            Self::SelectMultiple(config) => config.ess.required,
            Self::Tag(config) => config.ess.required,
            Self::Unsigned32(config) => config.ess.required,
            Self::Unsigned8(config) => config.ess.required,
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
            Self::DomainName(config) => config.ess.title(),
            Self::Password(config) => config.ess.title(),
            Self::HostNetworkGroup(config) => config.ess.title(),
            Self::SelectSingle(config) => config.ess.title(),
            Self::SelectMultiple(config) => config.ess.title(),
            Self::Tag(config) => config.ess.title(),
            Self::Unsigned32(config) => config.ess.title(),
            Self::Unsigned8(config) => config.ess.title(),
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
