# feattle-ui

[![Crates.io](https://img.shields.io/crates/v/feattle-ui.svg)](https://crates.io/crates/feattle-ui)
[![Docs.rs](https://docs.rs/feattle-ui/badge.svg)](https://docs.rs/feattle-ui)
[![CI](https://github.com/sitegui/feattle-rs/workflows/Continuous%20Integration/badge.svg)](https://github.com/sitegui/feattle-rs/actions)
[![Coverage Status](https://coveralls.io/repos/github/sitegui/feattle-rs/badge.svg?branch=master)](https://coveralls.io/github/sitegui/feattle-rs?branch=master)

This crate implements an administration Web Interface for visualizing and modifying the feature
flags (called "feattles", for short).

It provides a web-framework-agnostic implementation in [`AdminPanel`] and a ready-to-use binding
to `warp` in [`run_warp_server`]. Please refer to the
[main package - `feattle`](https://crates.io/crates/feattle) for more information.

Note that authentication is **not** provided out-of-the-box and you're the one responsible for
controlling and protecting the access.

## Optional features

- **warp**: provides [`run_warp_server`] for a read-to-use integration with [`warp`]

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
