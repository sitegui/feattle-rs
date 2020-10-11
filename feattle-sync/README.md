# feattle-sync

[![Crates.io](https://img.shields.io/crates/v/feattle-sync.svg)](https://crates.io/crates/feattle-sync)
[![Docs.rs](https://docs.rs/feattle-sync/badge.svg)](https://docs.rs/feattle-sync)
[![CI](https://github.com/sitegui/feattle-rs/workflows/Continuous%20Integration/badge.svg)](https://github.com/sitegui/feattle-rs/actions)
[![Coverage Status](https://coveralls.io/repos/github/sitegui/feattle-rs/badge.svg?branch=master)](https://coveralls.io/github/sitegui/feattle-rs?branch=master)

This crate is the implementation for some synchronization strategies for the feature flags
(called "feattles", for short).

The crate [`feattle_core`] provides the trait [`feattle_core::persist::Persist`] as the
extension point to implementors of the persistence layer logic. This crates has some useful
concrete implementations: [`Disk`] and [`S3`]. Please refer to the
[main package - `feattle`](https://crates.io/crates/feattle) for more information.

It also provides a simple way to poll the persistence layer for updates in [`BackgroundSync`].

## Optional features

- **s3**: provides [`S3`] to integrate with AWS' S3

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
