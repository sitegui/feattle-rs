# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [feattle 0.6.0] - 2021-05-08

### Changed
- Update `feattle-ui` to 0.6.0

## [feattle-ui 0.6.0] - 2021-05-08

### Added
- Add new methods in `AdminPanel` (`list_feattles_api_v1()`, `show_feattle_api_v1()` and `edit_feattle_api_v1()`) adding
  access to a lower-level API
- Expose new methods in `warp` as a JSON API under `/api/v1/`

## [feattle 0.5.0] - 2021-03-23

### Changed
- Update `feattle-core` to 0.5.0
- Update `feattle-sync` to 0.5.0
- Update `feattle-ui` to 0.5.0

## [feattle-core 0.5.0] - 2021-03-23

### Added
- Update doc on `Feattles::update()` to warn user about consistency guarantees.

### Changed
- `Feattles::last_reload()` now returns `LastReload` that contains more information about the last
  reload operation.

## [feattle-sync 0.5.0] - 2021-03-23

### Changed
- Update `feattle-core` to 0.5.0

## [feattle-ui 0.5.0] - 2021-03-23

### Added
- Show a warning in the UI if the last reload failed

### Changed
- `AdminPanel::list_feattles()` calls `Feattles:reload()` and is now asynchronous
- `AdminPanel::show_feattle()` calls `Feattles:reload()`
- `AdminPanel::edit_feattle()` calls `Feattles:reload()` and may return `RenderError::Reload`
- `AdminPanel::edit_feattle()` takes a new parameter `modified_by`

## [feattle-core 0.4.0] - 2021-03-21

### Changed
- Minimum supported Rust version is now `1.45`

## [feattle-sync 0.4.0] - 2021-03-21

### Changed
- Update `feattle-core` to 0.4.0
- Update `rusoto_core` to 0.46.0
- Update `rusoto_s3` to 0.46.0
- Update `tokio` to 1.4.0
- Minimum supported Rust version is now `1.45`

## [feattle-ui 0.4.0] - 2021-03-21

### Changed
- Update `feattle-core` to 0.4.0
- Update `warp` to 0.3.0
- Minimum supported Rust version is now `1.45`

## [feattle 0.4.0] - 2021-03-21

### Changed
- Update `feattle-core` to 0.4.0
- Update `feattle-sync` to 0.4.0
- Update `feattle-ui` to 0.4.0
- Minimum supported Rust version is now `1.45`

## [feattle-core 0.3.0] - 2021-01-13

### Changed
Instead of adding the bound `Persist` to the trait `Feattles`, only add it to methods that actually
need it. This gives more freedom to code that use methods other than update/reload/etc.

Also remove `Send`, `Sync` and `'static` bounds from `Feattles` and `Persist` traits.

The concrete types (created by `feattles!`) still implement those, but removing from the trait
makes code require the minimum contracts required. However, it makes the code somewhat more
verbose at times.

## [feattle-sync 0.3.0] - 2021-01-13

### Changed
Update `feattle-core` to 0.3.0

## [feattle-ui 0.3.0] - 2021-01-13

### Changed
Update `feattle-core` to 0.3.0

## [feattle 0.3.0] - 2021-01-13

### Changed
Update `feattle-core` to 0.3.0

## [feattle 0.2.5] - 2020-10-23

### Fixed
Fixed a bug in which when updating one feattle, all the others would be reset to their default value.

### Added
When the clipboard API is not available, show a dialog with the content for the user to copy it
manually.

## [feattle-core 0.2.5] - 2020-10-23

### Fixed
Fixed a bug in which when updating one feattle, all the others would be reset to their default value.

## [feattle-ui 0.2.5] - 2020-10-23

### Added
When the clipboard API is not available, show a dialog with the content for the user to copy it
manually.

## [feattle-core 0.2.4] - 2020-10-12
First fully documented and supported version

## [feattle-sync 0.2.4] - 2020-10-12
First fully documented and supported version

## [feattle-ui 0.2.4] - 2020-10-12
First fully documented and supported version

## [feattle 0.2.4] - 2020-10-12
First fully documented and supported version
