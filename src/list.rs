#![allow(clippy::module_name_repetitions)]
mod whole;

pub use whole::MessageType;
pub use whole::Model as WholeList;
pub use whole::SortColumn;

use crate::{checkbox::CheckStatus, input::InputNic, ViewString};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

const NUM_OF_DECIMALS_DEFAULT: usize = 2;

#[derive(Clone, PartialEq, Eq)]
pub struct ListItem {
    pub columns: Vec<Column>,
    pub sub_items: Vec<Vec<Column>>,
    pub creation_time: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Column {
    Text(ViewString),
    HostNetworkGroup(Vec<String>),
    SelectSingle(Option<(String, String)>),  // (id, value)
    SelectMultiple(HashMap<String, String>), // id, value
    Tag(HashSet<String>),
    Unsigned32(Option<u32>),
    Percentage(Option<f32>, Option<usize>), // usize = # of decimals
    Nic(Vec<InputNic>),
    CheckBox(CheckStatus, Option<Vec<Column>>, Option<String>), // String = display
}

impl ToString for Column {
    fn to_string(&self) -> String {
        match self {
            Self::Text(d) => d.to_string(),
            Self::HostNetworkGroup(d) => d.join(","),
            Self::SelectSingle(d) => d.as_ref().map_or_else(String::new, |d| d.1.clone()),
            Self::SelectMultiple(d) => d
                .values()
                .map(Clone::clone)
                .collect::<Vec<String>>()
                .join(","),
            Self::Tag(d) => d
                .iter()
                .map(Clone::clone)
                .collect::<Vec<String>>()
                .join(","),
            Self::Unsigned32(d) => d.map_or_else(String::new, |d| d.to_string()),
            Self::Percentage(f, d) => f.map_or_else(String::new, |f| {
                format!("{0:.1$}%", f * 100.0, d.unwrap_or(NUM_OF_DECIMALS_DEFAULT))
            }),
            Self::Nic(nics) => {
                let mut display = String::new();
                for n in nics {
                    display.push_str(&format!(
                        "{{{}: {}(ip) {}(gw)}} ",
                        n.name, n.interface_ip, n.gateway_ip
                    ));
                }
                display
            }
            Self::CheckBox(_, _, display) => {
                display.as_ref().map_or_else(String::new, Clone::clone)
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Flat,
    LayeredFirst,
    LayeredSecond,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Customer,
    Network,
    Account,
    Node,
    TiDb,
    TrustedDomain,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct DisplayInfo {
    pub width_cols: Vec<Option<u32>>,
    pub height_cols: Vec<Option<u32>>,
    pub width_full: u32, // sum of column widths
    pub width_view: u32, // width for display. if width_full > width_view, x scroll bar shows up.
    pub titles: Vec<&'static str>,
}
