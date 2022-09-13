#![allow(clippy::module_name_repetitions)]
mod whole;

pub use whole::MessageType;
pub use whole::Model as WholeList;
pub use whole::SortColumn;

//use crate::html_element::ViewString;
use crate::home::ViewString;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Clone, PartialEq, Eq)]
pub struct ListItem {
    pub columns: Vec<Column>,
    pub sub_items: Vec<Vec<Column>>,
    pub creation_time: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq)]
pub enum Column {
    Text(ViewString),
    HostNetworkGroup(Vec<String>),
    KeyValueList(HashMap<String, String>),
    Tag(HashSet<String>),
    Unsigned32(Option<u32>),
}

impl ToString for Column {
    fn to_string(&self) -> String {
        match self {
            Self::Text(d) => d.to_string(),
            Self::HostNetworkGroup(d) => concat(d),
            Self::KeyValueList(d) => {
                concat(&(d.values().map(Clone::clone).collect::<Vec<String>>()))
            }
            Self::Tag(d) => concat(&(d.iter().map(Clone::clone).collect::<Vec<String>>())),
            Self::Unsigned32(d) => d.map_or_else(String::new, |d| d.to_string()),
        }
    }
}

#[inline]
fn concat(list: &[String]) -> String {
    let mut conc = String::new();
    for (index, el) in list.iter().enumerate() {
        conc += el;
        if index < list.len() - 1 {
            conc += ",";
        }
    }
    conc
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
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct DisplayInfo {
    pub width_cols: Vec<Option<u32>>,
    pub height_cols: Vec<Option<u32>>,
    pub width_full: u32,
    pub titles: Vec<&'static str>,
}
