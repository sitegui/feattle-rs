# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [feattle 2.0.0] - 2024-06-26

### Changed

- BREAKING: Renamed feature "s3" to "rusoto_s3" in `feattle` and `feattle-sync`
- BREAKING: Renamed `feattle_sync::S3` to `feattle_sync::RusotoS3`
- BREAKING: Upgraded from `axum` 0.6 to 0.7
- BREAKING: Upgraded from `handlebars` 4 to 5
- BREAKING: Minimum Rust version is now 1.76

### Added

- Added feature "aws_sdk_s3" to `feattle` and `feattle-sync`

## [feattle 1.0.0] - 2023-06-28

### Changed

- Remove generic type from `Feattles` trait

## [feattle-core 1.0.0] - 2023-06-28

### Changed

- Remove generic type from `Feattles` trait  
  This generic represented the persistence layer, because each concrete implementation was free to declare their own
  associate error type.
  However, this feature caused "generics contamination" in the API, forcing users to carry the generic type parameter
  around.
  Instead, we can force persistent implementation to use a boxed error, removing this syntax complexity.  
  This means that the constructor now takes `Arc<dyn Persist>` instead of a direct instance of `Persist`.

## [feattle-sync 1.0.0] - 2023-06-28

### Changed

- Remove generic type from `Feattles` trait
- Added `BackgroundSync::start()` that waits for the first update to complete
- Deprecate `BackgroundSync::spawn()` since it will be replaced in favor of `start()`, that is more flexible.
- Added a new parameter to `S3::new()`: the `timeout`. Any operation will return an error after this time has elapsed.

## [feattle-ui 1.0.0] - 2023-06-28

### Changed

- Remove generic type from `Feattles` trait

## [feattle 0.10.0] - 2023-04-21

### Changed

- Update `feattle-ui` to 0.10.0
- Minimum supported Rust version is now 1.60

## [feattle-ui 0.10.0] - 2023-04-21

### Changed

- Add optional support for `axum`
- Minimum supported Rust version is now 1.60

## [feattle 0.9.0] - 2022-07-11

### Changed

- Update `feattle-core` to 0.9.0
- Update `feattle-sync` to 0.9.0
- Update `feattle-ui` to 0.9.0
- Update rusoto to `0.48.0`
- Update uuid to `1.1.2`
- Minimum supported Rust version is now 1.57

## [feattle-core 0.9.0] - 2022-07-11

### Changed

- Update uuid to `1.1.2`
- Minimum supported Rust version is now 1.57

## [feattle-sync 0.9.0] - 2022-07-11

### Changed

- Update rusoto to `0.48.0`
- Update `feattle-core` to 0.9.0
- Minimum supported Rust version is now 1.57

## [feattle-ui 0.9.0] - 2022-07-11

### Changed

- Update `feattle-core` to 0.9.0
- Change `pagecdn` with `cdnjs`
- Minimum supported Rust version is now 1.57

## [feattle 0.8.0] - 2022-03-16

### Changed

- Update `feattle-core` to 0.8.0
- Update `feattle-sync` to 0.8.0
- Update `feattle-ui` to 0.8.0
- Update parking_lot to `0.12.0`

## [feattle-core 0.8.0] - 2022-03-16

### Changed

- Update parking_lot to `0.12.0`

## [feattle-sync 0.8.0] - 2022-03-16

### Changed

- Update parking_lot to `0.12.0`

## [feattle-ui 0.8.0] - 2022-03-16

### Changed

- Update parking_lot to `0.12.0`

## [feattle 0.7.0] - 2021-09-09

### Changed

- Update `feattle-core` to 0.7.0
- Update `feattle-sync` to 0.7.0
- Update `feattle-ui` to 0.7.0
- Minimum supported Rust version is now 1.51

## [feattle-core 0.7.0] - 2021-09-09

### Changed

- Minimum supported Rust version is now 1.51

## [feattle-sync 0.7.0] - 2021-09-09

### Changed

- Update `rusoto` to 0.47.0
- Minimum supported Rust version is now 1.51

## [feattle-ui 0.7.0] - 2021-09-09

### Changed

- Update `handlebars` to 4.1.2
- Minimum supported Rust version is now 1.51

## [feattle 0.6.0] - 2021-05-08

### Changed

- Update `feattle-core` to 0.6.0
- Update `feattle-sync` to 0.6.0
- Update `feattle-ui` to 0.6.0
- Minimum supported Rust version is now 1.51

## [feattle-core 0.6.0] - 2021-05-08

### Added

- Implement `serde::Serialize` for `LastReload`

### Changed

- Minimum supported Rust version is now 1.51

## [feattle-sync 0.6.0] - 2021-03-23

### Changed

- Minimum supported Rust version is now 1.51
- Update `feattle-core` to 0.6.0

## [feattle-ui 0.6.0] - 2021-05-08

### Added

- Add new methods in `AdminPanel` (`list_feattles_api_v1()`, `show_feattle_api_v1()` and `edit_feattle_api_v1()`) adding
  access to a lower-level API
- Expose new methods in `warp` as a JSON API under `/api/v1/`

### Changed

- Minimum supported Rust version is now 1.51
- Update `feattle-core` to 0.6.0

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
