#![allow(clippy::module_name_repetitions)]
mod whole;

use std::collections::{HashMap, HashSet};
use std::fmt;

use chrono::{DateTime, Utc};
pub use whole::MessageType;
pub use whole::Model as WholeList;
pub use whole::SortColumn;

use crate::{
    checkbox::CheckStatus,
    input::{Comparison, InputNic},
    ViewString,
};

const NUM_OF_DECIMALS_DEFAULT: usize = 2;

#[derive(Clone, PartialEq)]
pub struct ListItem {
    pub columns: Vec<Column>,
    pub sub_items: Vec<Vec<Column>>,
    pub creation_time: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq)]
pub enum Column {
    Text(ViewString),
    HostNetworkGroup(Vec<String>),
    SelectSingle(Option<(String, ViewString)>), // id, value
    SelectMultiple(HashMap<String, ViewString>), // id, value
    VecSelect(Vec<HashMap<String, ViewString>>), // id, value
    Tag(HashSet<String>),
    Unsigned32(Option<u32>),
    Float64(Option<f64>),
    Percentage(Option<f32>, Option<usize>), // usize = # of decimals
    Nic(Vec<InputNic>),
    Checkbox(CheckStatus, Vec<Column>, Option<String>), // String = display
    Radio(ViewString, Vec<(bool, Vec<Column>)>, Option<String>), // bool = checked, String = display
    Group(Vec<Vec<Column>>),
    Comparison(Option<Comparison>),
}

impl std::fmt::Display for Column {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Text(d) => write!(formatter, "{d}"),
            Self::HostNetworkGroup(d) => write!(formatter, "{}", d.join(",")),
            Self::SelectSingle(d) => {
                if let Some((_, value)) = d {
                    write!(formatter, "{value}")
                } else {
                    Ok(())
                }
            }
            Self::SelectMultiple(d) => {
                // Since the language is not known here, keys are used.
                let values = d
                    .values()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                write!(formatter, "{values}")
            }
            Self::VecSelect(d) => {
                // Since the language is not known here, keys are used.
                let values = d
                    .iter()
                    .map(|s| {
                        s.values()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                write!(formatter, "{values}")
            }
            Self::Tag(d) => {
                let values = d.iter().map(String::as_str).collect::<Vec<_>>().join(",");
                write!(formatter, "{values}")
            }
            Self::Unsigned32(d) => {
                let value = d.map_or_else(String::new, |d| d.to_string());
                write!(formatter, "{value}")
            }
            Self::Float64(d) => {
                let value = d.map_or_else(String::new, |d| d.to_string());
                write!(formatter, "{value}")
            }
            Self::Percentage(f, d) => {
                let value = f.map_or_else(String::new, |f| {
                    format!("{0:.1$}%", f * 100.0, d.unwrap_or(NUM_OF_DECIMALS_DEFAULT))
                });
                write!(formatter, "{value}")
            }
            Self::Nic(nics) => {
                let mut display = String::new();
                for nic in nics {
                    display.push_str(&format!(
                        "{{{}: {}(interface) {}(gateway)}} ",
                        nic.name, nic.interface, nic.gateway
                    ));
                }
                write!(formatter, "{display}")
            }
            Self::Checkbox(_, _, display) | Self::Radio(_, _, display) => {
                if let Some(display) = display {
                    write!(formatter, "{display}")
                } else {
                    Ok(())
                }
            }
            Self::Group(_) => Ok(()),
            Self::Comparison(d) => {
                if let Some(d) = d {
                    write!(formatter, "{d}")
                } else {
                    Ok(())
                }
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
}

#[derive(Clone, PartialEq)]
pub enum ColWidths {
    Pixel(Vec<Option<u32>>), // None means need to calculate: full width - sum of widths of the other Some(width)
    Ratio(Vec<Option<f32>>), // None means no need to specify
}

impl Default for ColWidths {
    fn default() -> Self {
        Self::Ratio(Vec::new())
    }
}

impl ColWidths {
    fn len(&self) -> usize {
        match self {
            Self::Pixel(v) => v.len(),
            Self::Ratio(v) => v.len(),
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct DisplayInfo {
    pub widths: Vec<ColWidths>, // The first row, widths[0] should be ColWidths::Pixel
    pub width_full: u32,        // sum of column widths
    pub width_view: u32, // width for display. if width_full > width_view, x scroll bar shows up.
    pub height: u32,
    pub titles: Vec<&'static str>,
}
