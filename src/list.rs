#![allow(clippy::module_name_repetitions)]
mod whole;

use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Write},
};

use itertools::Itertools;
use jiff::Timestamp;
pub use whole::{MessageType, Model as WholeList, SortColumn, SortListKind};

use crate::{
    Theme, ViewString,
    checkbox::CheckStatus,
    input::{Comparison, InputNic},
};

const NUM_OF_DECIMALS_DEFAULT: usize = 2;

#[derive(Clone, PartialEq)]
pub struct ListItem {
    pub columns: Vec<Column>,
    pub sub_items: Vec<Vec<Column>>,
    pub creation_time: Option<Timestamp>,
}

#[derive(Clone, PartialEq)]
pub struct TextColumn {
    pub text: ViewString,
    pub display: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct DomainNameColumn {
    pub domain: ViewString,
    pub display: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct HostNetworkGroupColumn {
    pub host_network_group: Vec<String>,
}

#[derive(Clone, PartialEq)]
pub struct SelectSingleColumn {
    pub selected: Option<(String, ViewString)>, // id, value
    pub display: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct SelectMultipleColumn {
    pub selected: HashMap<String, ViewString>, // id, value
}

#[derive(Clone, PartialEq)]
pub struct TagColumn {
    pub tags: HashSet<String>,
}

#[derive(Clone, PartialEq)]
pub struct Unsigned32Column {
    pub value: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub struct Unsigned8Column {
    pub value: Option<u8>,
}

#[derive(Clone, PartialEq)]
pub struct Float64Column {
    pub value: Option<f64>,
}

#[derive(Clone, PartialEq)]
pub struct PercentageColumn {
    pub value: Option<f32>,
    pub decimals: Option<usize>,
}

#[derive(Clone, PartialEq)]
pub struct NicColumn {
    pub nics: Vec<InputNic>,
}

#[derive(Clone, PartialEq)]
pub struct FileColumn {
    pub filename: String,
}

#[derive(Clone, PartialEq)]
pub struct ComparisonColumn {
    pub comparison: Option<Comparison>,
}

#[derive(Clone, PartialEq)]
pub struct VecSelectColumn {
    pub selected: Vec<HashMap<String, ViewString>>, // id, value
}

#[derive(Clone, PartialEq)]
pub struct GroupColumn {
    pub groups: Vec<Vec<Column>>,
}

#[derive(Clone, PartialEq)]
pub struct CheckboxColumn {
    pub status: CheckStatus,
    pub children: Vec<Column>,
    pub display: Vec<String>,
    pub modal: Vec<ModalDisplay>,
    pub theme: Option<Theme>,
}

#[derive(Clone, PartialEq)]
pub struct RadioColumn {
    pub selected: ViewString,
    pub children: Vec<(bool, Vec<Column>)>, // bool = checked
    pub display: Vec<String>,
    pub modal: Vec<ModalDisplay>,
}

#[derive(Clone, PartialEq)]
pub struct ModalDisplay {
    pub title: String,
    pub content: String,
}

#[derive(Clone, PartialEq)]
pub enum Column {
    Text(TextColumn),
    DomainName(DomainNameColumn),
    HostNetworkGroup(HostNetworkGroupColumn),
    SelectSingle(SelectSingleColumn),
    SelectMultiple(SelectMultipleColumn),
    Tag(TagColumn),
    Unsigned32(Unsigned32Column),
    Unsigned8(Unsigned8Column),
    Float64(Float64Column),
    Percentage(PercentageColumn),
    Nic(NicColumn),
    File(FileColumn),
    Comparison(ComparisonColumn),
    VecSelect(VecSelectColumn),
    Group(GroupColumn),
    Checkbox(CheckboxColumn),
    Radio(RadioColumn),
}

impl std::fmt::Display for Column {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Text(d) => write!(formatter, "{}", &d.text),
            Self::DomainName(d) => write!(formatter, "{}", &d.domain),
            Self::HostNetworkGroup(d) => write!(formatter, "{}", d.host_network_group.join(",")),
            Self::SelectSingle(d) => {
                if let Some((_, value)) = d.selected.as_ref() {
                    write!(formatter, "{value}")
                } else {
                    Ok(())
                }
            }
            Self::SelectMultiple(d) => {
                // Since the language is not known here, keys are used.
                let values = d.selected.values().join(",");
                write!(formatter, "{values}")
            }
            Self::Tag(d) => {
                let values = d.tags.iter().join(",");
                write!(formatter, "{values}")
            }
            Self::Unsigned32(d) => {
                let value = d.value.map_or_else(String::new, |d| d.to_string());
                write!(formatter, "{value}")
            }
            Self::Unsigned8(d) => {
                let value = d.value.map_or_else(String::new, |d| d.to_string());
                write!(formatter, "{value}")
            }
            Self::Float64(d) => {
                let value = d.value.map_or_else(String::new, |d| d.to_string());
                write!(formatter, "{value}")
            }
            Self::Percentage(v) => {
                let value = v.value.map_or_else(String::new, |f| {
                    format!(
                        "{0:.1$}%",
                        f * 100.0,
                        v.decimals.unwrap_or(NUM_OF_DECIMALS_DEFAULT)
                    )
                });
                write!(formatter, "{value}")
            }
            Self::Nic(nics) => {
                let mut display = String::new();
                for nic in &nics.nics {
                    write!(
                        display,
                        "{{{}: {}(interface) {}(gateway)}} ",
                        nic.name, nic.interface, nic.gateway
                    )?;
                }
                write!(formatter, "{display}")
            }
            Self::File(d) => write!(formatter, "{}", &d.filename),
            Self::Comparison(d) => {
                if let Some(d) = d.comparison.as_ref() {
                    write!(formatter, "{d}")
                } else {
                    Ok(())
                }
            }
            Self::VecSelect(d) => {
                // Since the language is not known here, keys are used.
                let values = d.selected.iter().map(|s| s.values().join(",")).join(" | ");
                write!(formatter, "{values}")
            }
            Self::Group(_) | Self::Checkbox(_) | Self::Radio(_) => Ok(()),
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
    pub height: u32,
    pub titles: Vec<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_name_column_display() {
        let domain_col = DomainNameColumn {
            domain: ViewString::Raw("example.com".to_string()),
            display: None,
        };
        let column = Column::DomainName(domain_col);

        assert_eq!(column.to_string(), "example.com");
    }

    #[test]
    fn test_domain_name_column_with_display() {
        let domain_col = DomainNameColumn {
            domain: ViewString::Raw("example.com".to_string()),
            display: Some("<b>example.com</b>".to_string()),
        };
        let column = Column::DomainName(domain_col);

        // Display should be ignored in string representation, only domain is used
        assert_eq!(column.to_string(), "example.com");
    }
}
