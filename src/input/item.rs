use std::{
    cell::RefCell,
    collections::HashSet,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use super::{
    parse_host_network, CheckStatus, Comparison, HostNetwork, InputHostNetworkGroup, InputNic,
    InputTagGroup,
};
use crate::list::Column;

#[derive(Clone, PartialEq)]
pub struct TextItem {
    text: String,
}

// TextItem == String && &TextItem == String
impl PartialEq<String> for TextItem {
    fn eq(&self, other: &String) -> bool {
        &self.text == other
    }
}

// TextItem == &String && &TextItem == &String
impl PartialEq<&String> for TextItem {
    fn eq(&self, other: &&String) -> bool {
        &self.text == *other
    }
}

// TextItem == &str && &TextItem == &str
impl PartialEq<&str> for TextItem {
    fn eq(&self, other: &&str) -> bool {
        self.text == *other
    }
}

// String == TextItem && &String == TextItem
impl PartialEq<TextItem> for String {
    fn eq(&self, other: &TextItem) -> bool {
        self == &other.text
    }
}

// &str == TextItem && &&str == TextItem
impl PartialEq<TextItem> for &str {
    fn eq(&self, other: &TextItem) -> bool {
        *self == other.text
    }
}

impl Deref for TextItem {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl DerefMut for TextItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.text
    }
}

impl TextItem {
    #[must_use]
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

#[derive(Clone, PartialEq)]
pub struct PasswordItem {
    password: String,
}

impl PartialEq<String> for PasswordItem {
    fn eq(&self, other: &String) -> bool {
        &self.password == other
    }
}

impl PartialEq<&String> for PasswordItem {
    fn eq(&self, other: &&String) -> bool {
        &self.password == *other
    }
}

impl PartialEq<&str> for PasswordItem {
    fn eq(&self, other: &&str) -> bool {
        self.password == *other
    }
}

impl PartialEq<PasswordItem> for String {
    fn eq(&self, other: &PasswordItem) -> bool {
        self == &other.password
    }
}

impl PartialEq<PasswordItem> for &str {
    fn eq(&self, other: &PasswordItem) -> bool {
        *self == other.password
    }
}

impl Deref for PasswordItem {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.password
    }
}

impl DerefMut for PasswordItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.password
    }
}

impl PasswordItem {
    #[must_use]
    pub fn new(password: String) -> Self {
        Self { password }
    }
}

#[derive(Clone, PartialEq)]
pub struct HostNetworkGroupItem {
    host_network_group: InputHostNetworkGroup,
}

impl Deref for HostNetworkGroupItem {
    type Target = InputHostNetworkGroup;

    fn deref(&self) -> &Self::Target {
        &self.host_network_group
    }
}

impl DerefMut for HostNetworkGroupItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.host_network_group
    }
}

impl HostNetworkGroupItem {
    #[must_use]
    pub fn new(host_network_group: InputHostNetworkGroup) -> Self {
        Self { host_network_group }
    }

    #[must_use]
    pub fn into_inner(&self) -> InputHostNetworkGroup {
        self.host_network_group.clone()
    }
}

#[derive(Clone, PartialEq)]
pub struct SelectSingleItem {
    selected: Option<String>, // key
}

impl Deref for SelectSingleItem {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.selected
    }
}

impl DerefMut for SelectSingleItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.selected
    }
}

impl SelectSingleItem {
    #[must_use]
    pub fn new(selected: Option<String>) -> Self {
        Self { selected }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.selected.is_none()
    }

    #[must_use]
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    #[must_use]
    pub fn into_inner(&self) -> Option<String> {
        self.selected.clone()
    }

    pub fn clear(&mut self) {
        *self = Self::new(None);
    }
}

#[derive(Clone, PartialEq)]
pub struct SelectMultipleItem {
    selected_list: HashSet<String>, // keys
}

impl Deref for SelectMultipleItem {
    type Target = HashSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.selected_list
    }
}

impl DerefMut for SelectMultipleItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.selected_list
    }
}

impl SelectMultipleItem {
    #[must_use]
    pub fn new(selected_list: HashSet<String>) -> Self {
        Self { selected_list }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.selected_list.is_empty()
    }

    pub fn selected_list(&self) -> HashSet<&str> {
        self.selected_list.iter().map(String::as_str).collect()
    }

    #[must_use]
    pub fn into_inner(&self) -> HashSet<String> {
        self.selected_list.clone()
    }

    pub fn clear(&mut self) {
        self.selected_list.clear();
    }
}

#[derive(Clone, PartialEq)]
pub struct TagItem {
    tag_group: InputTagGroup,
}

impl Deref for TagItem {
    type Target = InputTagGroup;

    fn deref(&self) -> &Self::Target {
        &self.tag_group
    }
}

impl DerefMut for TagItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tag_group
    }
}

impl TagItem {
    #[must_use]
    pub fn new(tag_group: InputTagGroup) -> Self {
        Self { tag_group }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tag_group.is_empty()
    }

    #[must_use]
    pub fn tag_group(&self) -> &InputTagGroup {
        &self.tag_group
    }

    #[must_use]
    pub fn into_inner(&self) -> InputTagGroup {
        self.tag_group.clone()
    }

    pub fn clear(&mut self) {
        self.tag_group.clear();
    }
}

#[derive(Clone, PartialEq)]
pub struct Unsigned32Item {
    value: Option<u32>,
}

impl Deref for Unsigned32Item {
    type Target = Option<u32>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Unsigned32Item {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl Unsigned32Item {
    #[must_use]
    pub fn new(value: Option<u32>) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    #[must_use]
    pub fn into_inner(&self) -> Option<u32> {
        self.value
    }

    pub fn clear(&mut self) {
        *self = Self::new(None);
    }
}

#[derive(Clone, PartialEq)]
pub struct Float64Item {
    value: Option<f64>,
}

impl Deref for Float64Item {
    type Target = Option<f64>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Float64Item {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl Float64Item {
    #[must_use]
    pub fn new(value: Option<f64>) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    #[must_use]
    pub fn into_inner(&self) -> Option<f64> {
        self.value
    }

    pub fn clear(&mut self) {
        *self = Self::new(None);
    }
}

#[derive(Clone, PartialEq)]
pub struct PercentageItem {
    value: Option<f32>,
}

impl Deref for PercentageItem {
    type Target = Option<f32>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for PercentageItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl PercentageItem {
    #[must_use]
    pub fn new(value: Option<f32>) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    #[must_use]
    pub fn into_inner(&self) -> Option<f32> {
        self.value
    }

    pub fn clear(&mut self) {
        *self = Self::new(None);
    }
}

#[derive(Clone, PartialEq)]
pub struct NicItem {
    nics: Vec<InputNic>,
}

impl Deref for NicItem {
    type Target = Vec<InputNic>;

    fn deref(&self) -> &Self::Target {
        &self.nics
    }
}

impl DerefMut for NicItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nics
    }
}

impl NicItem {
    #[must_use]
    pub fn new(nics: Vec<InputNic>) -> Self {
        Self { nics }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nics.is_empty()
    }

    #[must_use]
    pub fn into_inner(&self) -> Vec<InputNic> {
        self.nics.clone()
    }

    pub fn clear(&mut self) {
        self.nics.clear();
    }
}

#[derive(Clone, PartialEq)]
pub struct FileItem {
    // TODO: document this properly
    name: String,    // file name
    content: String, // base64 encoded content
}

impl FileItem {
    #[must_use]
    pub fn new(name: String, content: String) -> Self {
        Self { name, content }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.content.is_empty()
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn clear(&mut self) {
        self.name.clear();
        self.content.clear();
    }
}

#[derive(Clone, PartialEq)]
pub struct ComparisonItem {
    comparison: Option<Comparison>,
}

impl Deref for ComparisonItem {
    type Target = Option<Comparison>;

    fn deref(&self) -> &Self::Target {
        &self.comparison
    }
}

impl DerefMut for ComparisonItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.comparison
    }
}

impl ComparisonItem {
    #[must_use]
    pub fn new(comparison: Option<Comparison>) -> Self {
        Self { comparison }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.comparison.is_none()
    }

    #[must_use]
    pub fn into_inner(&self) -> Option<Comparison> {
        self.comparison.clone()
    }

    pub fn clear(&mut self) {
        *self = Self::new(None);
    }
}

#[derive(Clone, PartialEq)]
pub struct VecSelectItem {
    /// The list of groups of selected keys. This `list` must be initialized having the same number
    /// of `HashSet::new()` as the number of the `<Select*>` components.
    list: Vec<HashSet<String>>,
}

impl Deref for VecSelectItem {
    type Target = Vec<HashSet<String>>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for VecSelectItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl VecSelectItem {
    #[must_use]
    pub fn new(list: Vec<HashSet<String>>) -> Self {
        Self { list }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[must_use]
    pub fn into_inner(&self) -> Vec<HashSet<String>> {
        self.list.clone()
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }
}

#[derive(Clone, PartialEq)]
pub struct GroupItem {
    // TODO: Define a Group like Vec<Group> ?
    groups: Vec<Vec<Rc<RefCell<InputItem>>>>,
}

impl Deref for GroupItem {
    type Target = Vec<Vec<Rc<RefCell<InputItem>>>>;

    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}

impl DerefMut for GroupItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.groups
    }
}

impl GroupItem {
    #[must_use]
    pub fn new(groups: Vec<Vec<Rc<RefCell<InputItem>>>>) -> Self {
        Self { groups }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    #[must_use]
    pub fn into_inner(&self) -> Vec<Vec<Rc<RefCell<InputItem>>>> {
        self.groups.clone()
    }

    pub fn clear(&mut self) {
        // for group in &mut self.groups {
        //     for item in group {
        //         if let Ok(mut item) = item.try_borrow_mut() {
        //             item.clear();
        //         }
        //     }
        // }
        self.groups.clear();
    }
}

#[derive(Clone, PartialEq)]
pub enum InputItem {
    Text(TextItem),
    Password(PasswordItem),
    HostNetworkGroup(HostNetworkGroupItem),
    SelectSingle(SelectSingleItem),
    SelectMultiple(SelectMultipleItem),
    Tag(TagItem),
    Unsigned32(Unsigned32Item),
    Float64(Float64Item),
    Percentage(PercentageItem),
    Nic(NicItem),
    File(FileItem),
    Comparison(ComparisonItem),
    VecSelect(VecSelectItem),
    Group(GroupItem),
    CheckBox(CheckStatus, Vec<Rc<RefCell<InputItem>>>), // Vec = children
    Radio(String, Vec<Vec<Rc<RefCell<InputItem>>>>),
}

impl InputItem {
    pub fn clear(&mut self) {
        match self {
            InputItem::Text(txt) => txt.clear(),
            InputItem::Password(pw) => pw.clear(),
            InputItem::HostNetworkGroup(group) => group.clear(),
            InputItem::SelectSingle(selected) => selected.clear(),
            InputItem::SelectMultiple(selected_list) => selected_list.clear(),
            InputItem::Tag(tag) => tag.clear(),
            InputItem::Unsigned32(value) => value.clear(),
            InputItem::Float64(value) => value.clear(),
            InputItem::Percentage(value) => value.clear(),
            InputItem::Nic(nics) => nics.clear(),
            InputItem::File(file) => file.clear(),
            InputItem::Comparison(cmp) => cmp.clear(),
            InputItem::VecSelect(list) => list.clear(),
            InputItem::Group(group) => group.clear(),
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
        }
    }
}

impl From<&Column> for InputItem {
    fn from(col: &Column) -> Self {
        match col {
            Column::Text(txt) => Self::Text(TextItem::new(txt.to_string())),
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
                Self::HostNetworkGroup(HostNetworkGroupItem::new(input))
            }
            Column::SelectSingle(value) => {
                Self::SelectSingle(SelectSingleItem::new(value.as_ref().map(|d| d.0.clone())))
            }
            Column::SelectMultiple(list) => Self::SelectMultiple(SelectMultipleItem::new(
                list.keys().map(Clone::clone).collect::<HashSet<String>>(),
            )),
            Column::VecSelect(list) => {
                let list = list
                    .iter()
                    .map(|l| l.keys().map(Clone::clone).collect::<HashSet<String>>())
                    .collect::<Vec<_>>();
                Self::VecSelect(VecSelectItem::new(list))
            }
            Column::Tag(tags) => Self::Tag(TagItem::new(InputTagGroup {
                old: tags.clone(),
                new: None,
                edit: None,
                delete: None,
            })),
            Column::Unsigned32(value) => Self::Unsigned32(Unsigned32Item::new(*value)),
            Column::Float64(value) => Self::Float64(Float64Item::new(*value)),
            Column::Percentage(f, _) => Self::Percentage(PercentageItem::new(*f)),
            Column::Nic(nics) => Self::Nic(NicItem::new(nics.clone())),
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
                Self::Group(GroupItem::new(input))
            }
            Column::Comparison(value) => Self::Comparison(ComparisonItem::new(value.clone())),
        }
    }
}
