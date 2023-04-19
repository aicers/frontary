# Changelog

This file documents recent notable changes to this project. The format of this
file is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and
this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.5.1]: https://github.com/aicers/frontary/compare/0.5.0...0.5.1
[0.5.0]: https://github.com/aicers/frontary/compare/0.4.1...0.5.0
[0.4.1]: https://github.com/aicers/frontary/compare/0.4.0...0.4.1
[0.4.0]: https://github.com/aicers/frontary/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/aicers/frontary/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/aicers/frontary/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/aicers/frontary/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/aicers/frontary/tree/0.1.0
