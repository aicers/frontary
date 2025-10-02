//! # Frontary
//!
//! Frontary is a comprehensive UI component library for building web applications
//! using the Yew framework. It provides a rich set of reusable components for
//! forms, lists, modals, and other common UI patterns.
//!
//! ## Features
//!
//! - **Form Components**: Input fields, checkboxes, radio buttons, selects
//! - **List Management**: Sortable and filterable data lists
//! - **Modal Dialogs**: Customizable modal windows with various styles
//! - **Theming Support**: Built-in theme system with customization options
//! - **Internationalization**: Multi-language support for UI text
//! - **Network Input**: Specialized components for IP addresses and ranges
//!
//! ## Quick Start
//!
//! ```rust
//! use frontary::{Checkbox, CheckStatus};
//! use yew::prelude::*;
//!
//! #[function_component(App)]
//! fn app() -> Html {
//!     html! {
//!         <Checkbox status={CheckStatus::Checked} />
//!     }
//! }
//! ```
//!
//! ## Module Organization
//!
//! The crate provides the following public components and utilities:
//!
//! - [`Checkbox`] - Checkbox components with various states
//! - [`Input`] - Generic input components and configurations
//! - [`WholeList`] - Data list components with sorting and filtering
//! - [`Modal`] - Modal dialog components
//! - [`Notification`] - Notification and alert components
//! - [`language`] - Internationalization support
//! - [`theme`] - Theme management and styling
//! - [`static_files`] - Static asset management

mod checkbox;
mod input;
mod ip_range_input;
pub mod language;
mod list;
mod modal;
mod notification;
mod pages;
mod password;
mod port_range_input;
mod radio;
mod radio_separate;
mod select;
mod sort;
pub mod static_files;
mod tab_menu;
mod text_input;
pub mod theme;

use std::cell::RefCell;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv4Addr;
use std::rc::Rc;
use std::str::FromStr;

use anyhow::Result;
use ipnet::Ipv4Net;
use json_gettext::{JSONGetText, get_text};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use yew::Properties;

pub use crate::checkbox::{CheckStatus, Model as Checkbox};
pub use crate::input::{
    CheckboxChildrenConfig, CheckboxConfig, CheckboxItem, ChildrenPosition, Comparison,
    ComparisonConfig, ComparisonItem, ComparisonKind, DomainNameConfig, DomainNameItem,
    Essential as InputEssential, FileConfig, FileItem, Float64Config, Float64Item, GroupConfig,
    GroupItem, HostNetworkGroupConfig, HostNetworkGroupItem, HostNetworkHtml, HostNetworkKind,
    InputConfig, InputHostNetworkGroup, InputItem, InputNic, InputTag, InputTagGroup,
    Model as Input, NicConfig, NicItem, PasswordConfig, PasswordItem, PercentageConfig,
    PercentageItem, RadioConfig, RadioItem, SelectMultipleConfig, SelectMultipleItem,
    SelectSingleConfig, SelectSingleItem, Tag, TagConfig, TagItem, TextConfig, TextItem,
    Unsigned8Config, Unsigned8Item, Unsigned16Config, Unsigned16Item, Unsigned32Config,
    Unsigned32Item, Value as ComparisonValue, ValueKind, VecSelectConfig, VecSelectItem,
    gen_default_items_from_confs, invalid_password, view_asterisk,
};
pub use crate::ip_range_input::Model as IpRangeInput;
pub use crate::language::Language;
pub use crate::list::{
    CheckboxColumn, ColWidths, Column, ComparisonColumn, DataType, DisplayInfo, DomainNameColumn,
    FileColumn, Float64Column, GroupColumn, HostNetworkGroupColumn, Kind, ListItem, MessageType,
    ModalDisplay, NicColumn, PercentageColumn, RadioColumn, SelectMultipleColumn,
    SelectSingleColumn, SortColumn, SortListKind, TagColumn, TextColumn, Unsigned8Column,
    Unsigned16Column, Unsigned32Column, VecSelectColumn, WholeList,
};
pub use crate::modal::{
    AlignButton as ModalAlign, Model as Modal, MsgType as ModalType, TextStyle as ModalTextStyle,
};
pub use crate::notification::{
    Category as NotificationCategory, CommonError, Model as Notification, NotificationItem,
    NotificationType, TIMEOUT_SECS, gen_notifications,
};
pub use crate::pages::{Info as PagesInfo, Model as Pages};
pub(crate) use crate::password::{PASSWORD_MIN_LEN, is_adjacent};
pub use crate::password::{Requirement as PasswordRequirement, check_password_requirements};
pub use crate::port_range_input::{Model as PortRangeInput, PortRange};
pub use crate::radio::Model as Radio;
pub use crate::radio_separate::Model as RadioSeparate;
pub use crate::select::complex::{Kind as SelectComplexKind, Model as SelectComplex};
pub use crate::select::mini::{Kind as SelectMiniKind, Model as SelectMini};
pub use crate::select::searchable::{Kind as SelectSearchableKind, Model as SelectSearchable};
pub use crate::select::vec_searchable::Model as VecSelect;
pub use crate::sort::{Model as Sort, Status as SortStatus};
pub use crate::tab_menu::Model as TabMenu;
pub use crate::text_input::Model as TextInput;
pub use crate::theme::Theme;

/// Enum representing different types of password validation errors.
///
/// Used by the password validation system to indicate specific reasons
/// why a password failed validation.
#[derive(Clone, Copy, PartialEq)]
pub enum InvalidPasswordKind {
    /// Password contains whitespace characters
    HasSpace,
    /// Password contains control characters
    HasControlCharacter,
    /// Password confirmation does not match the original
    NotMatch,
    /// Password is shorter than the minimum required length
    TooShort,
    /// Password lacks lowercase letters
    NoLowercaseLetter,
    /// Password lacks uppercase letters
    NoUppercaseLetter,
    /// Password lacks numeric characters
    NoNumber,
    /// Password lacks symbol characters
    NoSymbol,
    /// Password contains consecutive identical letters
    HasConsecutiveLetters,
    /// Password contains adjacent letters in alphabetical order
    HasAdjacentLetters,
}

#[allow(clippy::missing_panics_doc)]
#[cfg(feature = "test")]
pub fn alert(msg: &str) {
    web_sys::window()
        .expect("Window should exist")
        .alert_with_message(msg)
        .expect("Alert should show up");
}

/// Actions available in context menus and action buttons.
///
/// Used throughout the UI to represent common user actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoreAction {
    /// Edit the selected item
    Edit,
    /// Delete the selected item
    Delete,
}

/// Binary state actions for toggleable components.
///
/// Used for components that can be enabled or disabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnOffAction {
    /// Enable or turn on the component
    On,
    /// Disable or turn off the component
    Off,
}

/// A string value that can be either a translation key or raw text.
///
/// Used throughout the UI to support internationalization. The string
/// can either be a key that gets looked up in translation files or
/// raw text that is displayed as-is.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViewString {
    /// A translation key to be looked up in language files
    Key(String),
    /// Raw text to be displayed directly
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
    /// Converts the view string to a localized string.
    ///
    /// If this is a translation key, looks it up in the provided translation
    /// context. If it's raw text, returns the text as-is.
    ///
    /// # Arguments
    ///
    /// * `txt` - Translation context for looking up keys
    /// * `language` - Target language for translation
    ///
    /// # Returns
    ///
    /// The localized string, or empty string if translation key is not found.
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

/// Represents different types of network identifiers.
///
/// Used for parsing and validating network input that can be either
/// a single host, a network range, or an IP address range.
pub enum HostNetwork {
    /// A single host identifier (IP address or hostname)
    Host(String),
    /// A network in CIDR notation (e.g., "192.168.1.0/24")
    Network(String),
    /// An IP address range with start and end addresses
    Range(IpRange),
}

/// Parses a string into a `HostNetwork` variant.
///
/// Attempts to parse the input as an IPv4 address, network in CIDR notation,
/// or IP address range separated by a hyphen.
///
/// # Arguments
///
/// * `input` - The string to parse as a network identifier
///
/// # Returns
///
/// Returns `Some(HostNetwork)` if the input can be parsed as a valid network
/// identifier, or `None` if the format is not recognized.
///
/// # Examples
///
/// ```rust
/// use frontary::{parse_host_network, HostNetwork};
///
/// // Parse a single IP address
/// let host = parse_host_network("192.168.1.1");
/// assert!(matches!(host, Some(HostNetwork::Host(_))));
///
/// // Parse a network in CIDR notation
/// let network = parse_host_network("192.168.1.0/24");
/// assert!(matches!(network, Some(HostNetwork::Network(_))));
///
/// // Parse an IP range
/// let range = parse_host_network("192.168.1.1-192.168.1.10");
/// assert!(matches!(range, Some(HostNetwork::Range(_))));
/// ```
#[must_use]
pub fn parse_host_network(input: &str) -> Option<HostNetwork> {
    if Ipv4Addr::from_str(input).is_ok() {
        return Some(HostNetwork::Host(input.to_string()));
    }
    if Ipv4Net::from_str(input).is_ok() {
        return Some(HostNetwork::Network(input.to_string()));
    }
    if let Some((start, end)) = input.split_once('-') {
        let (start, end) = (start.trim(), end.trim());
        if let (Ok(start), Ok(end)) = (Ipv4Addr::from_str(start), Ipv4Addr::from_str(end))
            && start < end
        {
            return Some(HostNetwork::Range(IpRange {
                start: start.to_string(),
                end: end.to_string(),
            }));
        }
    }

    None
}

/// Validates and normalizes a host network string.
///
/// Checks if the input string represents a valid host, network, or IP range.
/// If valid, it may return a normalized version of the input.
///
/// # Arguments
///
/// * `input` - The string to validate as a network identifier
///
/// # Returns
///
/// Returns a tuple containing:
/// - `bool` - `true` if the input is valid, `false` otherwise
/// - `Option<String>` - A normalized version of the input if normalization was performed
///
/// # Examples
///
/// ```rust
/// use frontary::validate_host_network;
///
/// let (valid, normalized) = validate_host_network("192.168.1.1");
/// assert!(valid);
/// assert_eq!(normalized, None); // No normalization needed
///
/// let (valid, normalized) = validate_host_network("192.168.1.1~192.168.1.10");
/// assert!(valid);
/// assert!(normalized.is_some()); // Range was normalized
/// ```
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
        if let (Ok(start), Ok(end)) = (Ipv4Addr::from_str(ip_start), Ipv4Addr::from_str(ip_end))
            && start < end
        {
            return Some(format!("{ip_start} - {ip_end}"));
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

/// Shortens text to fit within a specified width using ellipsis.
///
/// Truncates the input text and adds "..." if it would exceed the given width
/// when rendered with the specified font. Uses canvas text measurement for accuracy.
///
/// # Arguments
///
/// * `item_org` - The original text to potentially shorten
/// * `width` - The maximum allowed width in pixels
/// * `font` - CSS font specification for measurement
/// * `margin` - Additional margin to account for in pixels
///
/// # Returns
///
/// Returns the original text if it fits, or a shortened version with "..." appended.
///
/// # Examples
///
/// ```rust,no_run
/// use frontary::shorten_text;
///
/// let short = shorten_text("Hello", 200, "12px Arial", 10);
/// // Will likely return "Hello" as it fits
///
/// let long = shorten_text("Very long text that exceeds width", 50, "12px Arial", 10);
/// // Will return something like "Very..."
/// ```
#[must_use]
pub fn shorten_text(item_org: &str, width: u32, font: &str, margin: u32) -> String {
    if item_org.len() > 4 {
        let mut sized_item = item_org.to_string();
        let item = item_org.as_bytes();
        for i in 4..item.len() {
            if let Ok(split) = std::str::from_utf8(&item[0..=i])
                && let Ok(w) = text_width(split, font)
                && width > (60 + margin)
                && w > width - (60 + margin)
            {
                sized_item = format!("{split}...");
                break;
            }
        }
        sized_item
    } else {
        item_org.to_string()
    }
}

/// Specifies the direction or type of network endpoint.
///
/// Used in network-related components to indicate whether an endpoint
/// represents the source, destination, or both directions of network traffic.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub enum EndpointKind {
    /// Traffic originating from this endpoint
    Source,
    /// Traffic destined for this endpoint
    Destination,
    /// Traffic in both directions (source and destination)
    Both,
}

impl Default for EndpointKind {
    fn default() -> Self {
        Self::Both
    }
}

/// Additional information associated with a selection.
///
/// Provides context for selected items, particularly in network-related
/// selections where endpoint direction matters.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectionExtraInfo {
    /// Network selection with endpoint direction information
    Network(EndpointKind),
    /// Basic selection without additional context
    Basic,
}

pub type RefSelectionExtraInfo = Rc<RefCell<Option<SelectionExtraInfo>>>;

/// A complex selection state that manages both predefined and custom selections.
///
/// Used in advanced selection components where users can choose from
/// predefined options and also create custom selections. Each selection
/// can have associated extra information.
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct ComplexSelection {
    /// Predefined selections available to the user
    pub predefined: Rc<RefCell<Option<HashMap<String, RefSelectionExtraInfo>>>>,
    /// Custom selections created by the user
    pub custom: Rc<RefCell<HashMap<String, RefSelectionExtraInfo>>>,
}

impl ComplexSelection {
    /// Returns the length of predefined and custom selections.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Option<usize>` - Number of predefined selections (None if not initialized)
    /// - `usize` - Number of custom selections
    #[must_use]
    pub fn len(&self) -> (Option<usize>, usize) {
        if let (Ok(predefined), Ok(custom)) =
            (self.predefined.try_borrow(), self.custom.try_borrow())
        {
            let is_selected = |value: &RefSelectionExtraInfo| {
                value
                    .try_borrow()
                    .map(|selection| selection.is_some())
                    .unwrap_or(false)
            };
            let custom_selected = custom.values().filter(|value| is_selected(value)).count();
            let predefined_selected = (*predefined)
                .as_ref()
                .map(|map| map.values().filter(|value| is_selected(value)).count());

            (predefined_selected, custom_selected)
        } else {
            (Some(usize::MAX), usize::MAX)
        }
    }

    /// Checks if both predefined and custom selections are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == (Some(0), 0)
    }
}

/// Represents an IP address range with start and end addresses.
///
/// Used for defining ranges of IP addresses, such as "192.168.1.1-192.168.1.100".
/// The range is inclusive of both start and end addresses.
#[derive(Deserialize, Serialize, Clone, Default, Eq, PartialEq)]
pub struct IpRange {
    /// The starting IP address of the range
    pub start: String,
    /// The ending IP address of the range
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
        write!(f, "{} - {}", self.start, self.end)
    }
}

impl IpRange {
    /// Validates that the IP range has valid start and end addresses.
    ///
    /// # Returns
    ///
    /// `true` if both start and end are valid IPv4 addresses and start < end,
    /// `false` otherwise.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if let (Ok(start), Ok(end)) = (
            Ipv4Addr::from_str(&self.start),
            Ipv4Addr::from_str(&self.end),
        ) && start < end
        {
            return true;
        }
        false
    }
}

/// Trait for types that can contain collections of network identifiers.
///
/// Provides common functionality for validating and accessing different
/// types of network identifiers (hosts, networks, and ranges).
pub trait HostNetworkGroupTrait {
    /// Validates that all network identifiers in the group are well-formed.
    ///
    /// Checks that all hosts are valid IP addresses, all networks are valid
    /// CIDR notation, and all ranges have valid start/end addresses.
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

    /// Converts all network identifiers to a single vector of strings.
    ///
    /// Combines hosts, networks, and ranges into one unified list for
    /// display or processing purposes.
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

    /// Returns a slice of host IP addresses.
    fn hosts(&self) -> &[String];
    /// Returns a slice of network addresses in CIDR notation.
    fn networks(&self) -> &[String];
    /// Returns a vector of IP address ranges.
    ///
    /// Note: Returns `Vec` rather than slice because most implementations
    /// need to convert from internal representation.
    fn ranges(&self) -> Vec<IpRange>;
}

/// Sorts a vector of host IP addresses in ascending order and removes duplicates.
///
/// Parses each string as an IPv4 address for proper numeric sorting.
/// Invalid IP addresses are treated as 0.0.0.0 for sorting purposes.
///
/// # Arguments
///
/// * `hosts` - A mutable reference to a vector of IP address strings
///
/// # Examples
///
/// ```rust
/// use frontary::sort_hosts;
///
/// let mut hosts = vec![
///     "192.168.1.10".to_string(),
///     "192.168.1.2".to_string(),
///     "192.168.1.10".to_string(), // duplicate
/// ];
/// sort_hosts(&mut hosts);
/// assert_eq!(hosts, vec!["192.168.1.2", "192.168.1.10"]);
/// ```
pub fn sort_hosts(hosts: &mut Vec<String>) {
    hosts.sort_unstable_by_key(|h| {
        if let Ok(addr) = Ipv4Addr::from_str(h) {
            addr
        } else {
            Ipv4Addr::UNSPECIFIED
        }
    });
    hosts.dedup();
}

/// Sorts a vector of network addresses in ascending order and removes duplicates.
///
/// Parses each string as an IPv4 network in CIDR notation for proper sorting.
/// Invalid network addresses are treated as 0.0.0.0/32 for sorting purposes.
///
/// # Arguments
///
/// * `networks` - A mutable reference to a vector of network address strings in CIDR notation
///
/// # Examples
///
/// ```rust
/// use frontary::sort_networks;
///
/// let mut networks = vec![
///     "192.168.2.0/24".to_string(),
///     "192.168.1.0/24".to_string(),
///     "192.168.2.0/24".to_string(), // duplicate
/// ];
/// sort_networks(&mut networks);
/// assert_eq!(networks, vec!["192.168.1.0/24", "192.168.2.0/24"]);
/// ```
#[allow(clippy::missing_panics_doc)] // because it never happens
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

/// A collection of network identifiers including hosts, networks, and ranges.
///
/// Used to group different types of network identifiers together for
/// validation and processing.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkGroup {
    /// Individual host IP addresses
    pub hosts: Vec<String>,
    /// Network addresses in CIDR notation
    pub networks: Vec<String>,
    /// IP address ranges
    pub ranges: Vec<IpRange>,
}

/// A generic item with an identifier and display value.
///
/// Used throughout the UI for selectable items in lists, dropdowns,
/// and other components. The value can be either a translation key
/// or raw display text.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    /// Unique identifier for the item
    id: String,
    /// Display value (translation key or raw text)
    value: ViewString,
}

impl Item {
    /// Creates a new item with the specified ID and value.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the item
    /// * `value` - Display value (translation key or raw text)
    #[must_use]
    pub fn new(id: String, value: ViewString) -> Self {
        Self { id, value }
    }

    /// Returns the item's unique identifier.
    #[must_use]
    pub fn id(&self) -> &String {
        &self.id
    }

    /// Returns the item's display value as a string.
    #[must_use]
    pub fn value(&self) -> String {
        self.value.to_string()
    }

    /// Returns the item's display value with translation support.
    ///
    /// If the value is a translation key, it will be looked up in the
    /// provided translation context. If it's raw text, it's returned as-is.
    ///
    /// # Arguments
    ///
    /// * `txt` - Translation context for looking up keys
    /// * `language` - Target language for translation
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
