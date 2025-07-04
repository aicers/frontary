# Changelog

This file documents recent notable changes to this project. The format of this
file is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and
this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `DomainNameItem` and `DomainNameConfig` for domain name input validation.
  Provides built-in validation according to RFC standards including label length
  limits, allowed characters, and overall domain name structure validation.

### Changed

- Refined layout and validation styling in `user_input_composite`,
  `user_input_comparison`, `user_input_select`, `user_input` and
  `vec_searchable` modules to align with updated design standards.

## [0.11.0] - 2025-06-25

### Changed

- Changed `PortRange` struct fields from `Option<i64>` to `Option<u16>` for
  improved type safety and accuracy, as port numbers range from 0 to 65535.
- Replaced `pumpkin-dark` and `pumpkin-light` features with a single
  `pumpkin` feature for unified theme handling.
- Updated to Rust 2024 edition.
- Add height to `Modal` and support HTML in `TextStyle`.
- Add `MsgType::None` for modal and set title header when not pumpkin.
- Added an `immutable` flag to `TextConfig` — when true, disables the associated
  input field in the edit modal.
- Adjusted `notification` z-index from 99999 to 90 to ensure better layering and
  compatibility with other UI components.
- Added `overflow-y: auto` to `input-inner` to enhance vertical scrolling and
  layout behavior.
- Added `UInteger`, `Vector`, `IpAddr`, `Bool` to `ValueKind`.
- `creation_time` in `ListItem` no longer uses `DateTime<Utc>` from `chrono`; it
  now uses `Timestamp` from `jiff`.

### Fixed

- Fixed overflow error when `NotificationType::ErrorList` has an empty errors.
- Fixed notifications not re-rendering when a new notification is given without
  a timeout.
- Fixed missing duplicate validation in modals and tags caused by incorrect index
  calculation from an extra `+1` offset.

### Removed

- Removed unnecessary inline styles from `input-contents` to simplify layout adjustments.

## [0.10.1]

### Changed

- Minor style changes related to the `SelectMini` component and user inputs.

## [0.10.0]

### Added

- Enabled the radio component have child items recursively.
- Added the `invalid_password` method to validate user passwords based on
  different criteria depending on whether the `cc-password` feature is enabled.
- Added a method to validate all password requirements.
- Added extensions validation to allow any extensions and automatically add a
  `.` if necessary
- Added `Theme` enum with light/dark mode support and local storage integration.

### Changed

- Adjusted Clumit notification component's styling for consistency with the
  design.
- Renamed `CheckBox` to `Checkbox` across all related types
- Renamed `InputType` to `InputConfig`, and changed the variants of
  `InputConfig` to be composed of the corresponding structs.
- Changed the variants of `InputItem` to be composed of the corresponding
  structs.
- Removed `Essential::default` and introduced `preset`:
  - Some `InputConfig` variants, that need preset values for users, now have the
    `preset` field.
- Moved `Essential::unique` to `TextConfig`, because only `TextConfig` needs it.
- Replaced deprecated Clumit color variable names with new color naming
  conventions.
- Moved Clumit images to the pumpkin directory, removed the `clumit-` prefix
  for consistency, and updated references accordingly.
- Changed file input extension handling to avoid hardcoding extension values.
- Changed the variants of `Column` to use structs instead of tuples.
- Changed styling for `user_input_composite.rs` and `user_input.rs` to enahnce
  design and funcionality for `view_group`.
- Removed `br_separator` for `Column::{Checkbox | Radio}`.
- Introduced `{CheckboxColumn | RadioColumn}::modal`, which displays a modal
  window corresponding to each of the `{CheckboxColumn | RadioColumn}::display`
  elements.
- The content of both `modal` and `display` is directly injected into the HTML
  stream using `Html::from_html_unchecked` from Yew.
- Added `TextColumn::display` element.
- Added `SelectSingleColumn::display` element.
- Added `Unsigned8` to `Column`, `InputConfig`, `InputItem`.

### Removed

- Removed `Svg`

### Fixed

- Fixed `delete-trash` and `close-white` icon.
- Fixed overlapping placeholder text.
- Fixed search box width and border radius.
- Fixed `SelectMini` component to ensure long data entries are display on a
  single line.
- Adjusted alignment for example text in input fields.
- Fixed the layout of tags to wrap properly and fit within their container
  without overflowing.
- Fixed to show on the same line instead of two lines in `Checkbox`.
- Fixed to handle `File` properly in `Column`.

## [0.9.4] - 2024-10-16

### Fixed

- Adjusted the height of `SelectSearchable` to ensure all items are visible,
  according to the features.

## [0.9.3] - 2024-10-16

### Changed

- Adjusted the space between `CheckBox` items.
- Replaced PNG images with SVG images for Clumit theme.

### Deprecated

- `Svg` component will be removed, because it is no longer needed.

## [0.9.2] - 2024-10-07

### Added

- Applied Clumit theme to `WholeList`,`Tag`, `HostNetworkHtml`, `Modal`, and
  `SelectComplex`.
- Added support for `InputType::Text` in the `view_checkbox` method.
- Applied the Clumit theme to `Input`, `WholeList`, `SelectMini`, and `Radio`
  components.

### Changed

- Changed to hide the `Essential::title` when it is empty.

### Fixed

- Fixed a bug with incorrect page number display in pagination.
- Added the `role` attribute to the `radio` and `checkbox` elements implemented
  with `div` tags, which will enhance web accessibility.

## [0.9.1] - 2024-07-18

### Added

- Added `MoreActionBasic` to `SelectMiniKind` in `SelectMini`.
- Added correct CSS styling for `MoreActionBasic` for `SelectMini`.
- Applied Clumit theme to `Radio`, `CheckBox`, `TabMenu`, `Modal`.
- Added `Svg` to fetch and display a SVG file.
- Added `cc-password` feature in `Cargo.toml`.
- Applied Clumit theme to `Sort`, `SelectMini`, `SelectSearchable`, `Input`,
  `view_asterisk`, `Notification`.

### Fixed

- Fixed the wrong directory of delete-x.png file from `frotary`
  to `frontary` in `theme.css`.
- Fixed `Modal` error for title header.

### Changed

- Applied `cargo fmt` with `--config group_imports=StdExternalCrate` for
  consistent import grouping.
- Updated CI process to include `--config group_imports=StdExternalCrate`
  with the `cargo fmt -- --check` command for enforcing import grouping rules.
- Changed `expect` message in `text` macro to show the invalid key name
  if `test` feature is enabled
- Removed the scrollbar in `WholeList` and adjusted the width size for
  `TabMenu`.

## [0.9.0] - 2024-04-11

### Changed

- Updated Yew to 0.21.0

## [0.8.2] - 2024-02-13

### Changed

- Add `Debug` to `PagesInfo` in order to support unit tests.

## [0.8.1] - 2023-11-20

### Changed

- Support dynamic titles of `CheckBox` which are not included language files.

## [0.8.0] - 2023-11-17

### Changed

- Changed `InputEssential::title` to a `String` type.

## [0.7.5] - 2023-09-19

### Fixed

- Fixed `SelectSingle` and `SelectMini` component to compare what value the
  user has selected with the pre-shortened value of the item selected instead
  of the shortend value.

## [0.7.4] - 2023-09-12

### Changed

- Changed `SelectSingle` and `SelectMini` component to display what value
  the user has selected and adjusted height.

## [0.7.3] - 2023-09-07

### Fixed

- Use `readonly` instead of `disabled` in `input` for others than Safari.

## [0.7.2] - 2023-07-21

### Added

- Added `disable` prop in `Pages` component.

## [0.7.1] - 2023-07-05

### Fixed

- Check if inputs of `IpRangeInput` and `PortRangeInput` have valid ranges.

## [0.7.0] - 2023-05-17

### Added

- Added `IpRangeInput`, `PortRangeInput`, and `TextInput` to get range and
  single inputs and not rely on `Input` component with `enable_bool_pop_up`
  prop.

### Removed

- Removed `enable_bool_pop_up` prop from `Input` component.

## [0.6.0] - 2023-05-02

### Changed

- Separate `NetworkItem` from `Item`.

## [0.5.2] - 2023-04-19

### Fixed

- Check if an HTML element still exists to avoid a null pointer in JS code.

## [0.5.1] - 2023-04-19

### Fixed

- Fix a bug where tag area gets hidden when edit-related icons are clicked.

## [0.5.0] - 2023-03-28

### Changed

- Updated Yew to 0.20.

## [0.4.1] - 2023-03-02

### Changed

- Make multiple `<Tag>`s available at the same time without `<Input>`.

### Security

- Turned off the default features of chrono that might casue SEGFAULT. See
  [RUSTSEC-2020-0071](https://rustsec.org/advisories/RUSTSEC-2020-0071)
  for details.

## [0.4.0] - 2023-01-24

### Changed

- The main colors are defined as global variables in css for applying themes.

## [0.3.0] - 2023-01-09

### Added

- `InputType::Group`, `InputItem::Group`, and `ListItem::Group`
- `InputType::Comparison`, `InputItem::Comparison`, and `ListItem::Comparison`
- `InputType::VecSelect`, `InputItem::VecSelect`, and `ListItem::VecSelect`

### Changed

- Change `String` with `ViewString` in `SelectSingle` and `SelectMultiple`.

## [0.2.0] - 2022-11-27

### Changed

- Make `WholeList::Props::data_type` optionable.
- Enable `DisplayInfo` to include information of multi lines.

## [0.1.1] - 2022-11-02

### Fixed

- Fixed build with Rust 1.65.

## [0.1.0] - 2022-11-01

### Added

- Initial release.

[Unreleased]: https://github.com/aicers/frontary/compare/0.11.0...main
[0.11.0]: https://github.com/aicers/frontary/compare/0.10.1...0.11.0
[0.10.1]: https://github.com/aicers/frontary/compare/0.10.0...0.10.1
[0.10.0]: https://github.com/aicers/frontary/compare/0.9.4...0.10.0
[0.9.4]: https://github.com/aicers/frontary/compare/0.9.3...0.9.4
[0.9.3]: https://github.com/aicers/frontary/compare/0.9.2...0.9.3
[0.9.2]: https://github.com/aicers/frontary/compare/0.9.1...0.9.2
[0.9.1]: https://github.com/aicers/frontary/compare/0.9.0...0.9.1
[0.9.0]: https://github.com/aicers/frontary/compare/0.8.2...0.9.0
[0.8.2]: https://github.com/aicers/frontary/compare/0.8.1...0.8.2
[0.8.1]: https://github.com/aicers/frontary/compare/0.8.0...0.8.1
[0.8.0]: https://github.com/aicers/frontary/compare/0.7.5...0.8.0
[0.7.5]: https://github.com/aicers/frontary/compare/0.7.4...0.7.5
[0.7.4]: https://github.com/aicers/frontary/compare/0.7.3...0.7.4
[0.7.3]: https://github.com/aicers/frontary/compare/0.7.2...0.7.3
[0.7.2]: https://github.com/aicers/frontary/compare/0.7.1...0.7.2
[0.7.1]: https://github.com/aicers/frontary/compare/0.7.0...0.7.1
[0.7.0]: https://github.com/aicers/frontary/compare/0.6.0...0.7.0
[0.6.0]: https://github.com/aicers/frontary/compare/0.5.1...0.6.0
[0.5.1]: https://github.com/aicers/frontary/compare/0.5.0...0.5.1
[0.5.0]: https://github.com/aicers/frontary/compare/0.4.1...0.5.0
[0.4.1]: https://github.com/aicers/frontary/compare/0.4.0...0.4.1
[0.4.0]: https://github.com/aicers/frontary/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/aicers/frontary/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/aicers/frontary/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/aicers/frontary/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/aicers/frontary/tree/0.1.0
