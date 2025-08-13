#![allow(clippy::module_name_repetitions)]
mod whole;

use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Write},
};

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
                let values = d
                    .selected
                    .values()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                write!(formatter, "{values}")
            }
            Self::Tag(d) => {
                let values = d
                    .tags
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(",");
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
                let values = d
                    .selected
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

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, Default, PartialEq, Debug)]
pub struct DisplayInfo {
    pub widths: Vec<ColWidths>, // The first row, widths[0] should be ColWidths::Pixel
    pub width_full: u32,        // sum of column widths
    pub height: u32,
    pub titles: Vec<&'static str>,
}

impl DisplayInfo {
    /// Creates a new `DisplayInfo` with validation.
    ///
    /// # Arguments
    ///
    /// * `widths` - Vector of column width specifications
    /// * `width_full` - Total width for all columns
    /// * `width_view` - Display viewport width
    /// * `height` - Row height
    /// * `titles` - Column titles
    ///
    /// # Returns
    ///
    /// Returns the validated `DisplayInfo` on success.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The sum of fixed column widths exceeds `width_full`
    /// * Width values are invalid (zero or excessive)
    /// * The first row of widths is not of type `ColWidths::Pixel` as required
    pub fn new(
        widths: Vec<ColWidths>,
        width_full: u32,
        width_view: u32,
        height: u32,
        titles: Vec<&'static str>,
    ) -> Result<Self, String> {
        // Basic validation first
        if width_full == 0 {
            return Err("width_full must be greater than zero".to_string());
        }

        // Validate that the first row should be ColWidths::Pixel as per documentation
        if let Some(first_row) = widths.first()
            && !matches!(first_row, ColWidths::Pixel(_))
        {
            return Err("The first row of widths must be ColWidths::Pixel".to_string());
        }

        // Validate width consistency for Pixel columns
        for (row_idx, col_widths) in widths.iter().enumerate() {
            if let ColWidths::Pixel(pixel_widths) = col_widths {
                let fixed_width_sum: u32 = pixel_widths.iter().filter_map(|&w| w).sum();
                if fixed_width_sum > width_full {
                    return Err(format!(
                        "Sum of fixed column widths ({fixed_width_sum}) exceeds total width ({width_full}) in row {row_idx}"
                    ));
                }
            }
        }

        Ok(Self {
            widths,
            width_full,
            width_view,
            height,
            titles,
        })
    }

    /// Creates a new `DisplayInfo` with the same validation, but returns a default on error.
    /// This is useful for maintaining backward compatibility where validation failures should not panic.
    #[must_use]
    pub fn new_or_default(
        widths: Vec<ColWidths>,
        width_full: u32,
        width_view: u32,
        height: u32,
        titles: Vec<&'static str>,
    ) -> Self {
        Self::new(widths, width_full, width_view, height, titles).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_info_valid_creation() {
        let widths = vec![ColWidths::Pixel(vec![Some(400), Some(300), None])];
        let result = DisplayInfo::new(widths, 1200, 1000, 30, vec!["Col1", "Col2", "Col3"]);

        assert!(result.is_ok());
        let display_info = result.unwrap();
        assert_eq!(display_info.width_full, 1200);
        assert_eq!(display_info.width_view, 1000);
    }

    #[test]
    fn test_display_info_width_sum_exceeds_full_width() {
        let widths = vec![ColWidths::Pixel(vec![Some(1000), Some(1000)])];
        let result = DisplayInfo::new(widths, 1200, 1000, 30, vec!["Col1", "Col2"]);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Sum of fixed column widths (2000) exceeds total width (1200)"));
    }

    #[test]
    fn test_display_info_zero_width_full() {
        let widths = vec![ColWidths::Pixel(vec![Some(100)])];
        let result = DisplayInfo::new(widths, 0, 1000, 30, vec!["Col1"]);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error, "width_full must be greater than zero");
    }

    #[test]
    fn test_display_info_first_row_not_pixel() {
        let widths = vec![ColWidths::Ratio(vec![Some(0.5), Some(0.5)])];
        let result = DisplayInfo::new(widths, 1200, 1000, 30, vec!["Col1", "Col2"]);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error, "The first row of widths must be ColWidths::Pixel");
    }

    #[test]
    fn test_display_info_mixed_pixel_and_ratio_rows() {
        let widths = vec![
            ColWidths::Pixel(vec![Some(400), Some(300)]),
            ColWidths::Ratio(vec![Some(0.5), Some(0.5)]),
        ];
        let result = DisplayInfo::new(widths, 1200, 1000, 30, vec!["Col1", "Col2"]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_display_info_with_none_pixel_widths() {
        let widths = vec![ColWidths::Pixel(vec![Some(400), None, Some(300)])];
        let result = DisplayInfo::new(widths, 1200, 1000, 30, vec!["Col1", "Col2", "Col3"]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_display_info_new_or_default_with_valid_data() {
        let widths = vec![ColWidths::Pixel(vec![Some(400), Some(300)])];
        let display_info =
            DisplayInfo::new_or_default(widths, 1200, 1000, 30, vec!["Col1", "Col2"]);

        assert_eq!(display_info.width_full, 1200);
        assert_eq!(display_info.width_view, 1000);
    }

    #[test]
    fn test_display_info_new_or_default_with_invalid_data() {
        let widths = vec![ColWidths::Pixel(vec![Some(1000), Some(1000)])];
        let display_info =
            DisplayInfo::new_or_default(widths, 1200, 1000, 30, vec!["Col1", "Col2"]);

        // Should return default values on validation failure
        assert_eq!(display_info, DisplayInfo::default());
    }
}
