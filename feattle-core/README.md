# feattle-core

[![Crates.io](https://img.shields.io/crates/v/feattle-core.svg)](https://crates.io/crates/feattle-core)
[![Docs.rs](https://docs.rs/feattle-core/badge.svg)](https://docs.rs/feattle-core)
[![CI](https://github.com/sitegui/feattle-rs/workflows/Continuous%20Integration/badge.svg)](https://github.com/sitegui/feattle-rs/actions)
[![Coverage Status](https://coveralls.io/repos/github/sitegui/feattle-rs/badge.svg?branch=master)](https://coveralls.io/github/sitegui/feattle-rs?branch=master)

This crate is the core implementation of the feature flags (called "feattles", for short).

Its main parts are the macro [`feattles!`] together with the trait [`Feattles`]. Please refer to
the [main package - `feattle`](https://crates.io/crates/feattle) for more information.

## Usage example
```rust
use feattle_core::{feattles, Feattles};
use feattle_core::persist::NoPersistence;

// Declare the struct
feattles! {
    struct MyFeattles {
        /// Is this usage considered cool?
        is_cool: bool = true,
        /// Limit the number of "blings" available.
        /// This will not change the number of "blengs", though!
        max_blings: i32,
        /// List the actions that should not be available
        blocked_actions: Vec<String>,
    }
}

// Create a new instance (`NoPersistence` is just a mock for the persistence layer)
let my_feattles = MyFeattles::new(NoPersistence);

// Read values (note the use of `*`)
assert_eq!(*my_feattles.is_cool(), true);
assert_eq!(*my_feattles.max_blings(), 0);
assert_eq!(*my_feattles.blocked_actions(), Vec::<String>::new());
```

## How it works

The macro will generate a struct with the given name and visibility modifier (assuming private
by default). The generated struct implements [`Feattles`] and also exposes one method for each
feattle.

The methods created for each feattle allow reading their current value. For example, for a
feattle `is_cool: bool`, there will be a method like
`pub fn is_cool(&self) -> MappedRwLockReadGuard<bool>`. Note the use of
[`parking_lot::MappedRwLockReadGuard`] because the interior of the struct is stored behind a `RwLock` to
control concurrent access.

A feattle is created with the syntax `$key: $type [= $default]`. You can use doc coments (
starting with `///`) to describe nicely what they do in your system. You can use any type that
implements [`FeattleValue`] and optionally provide a default. If not provided, the default
will be created with `Default::default()`.

## Updating values
This crate only disposes of low-level methods to load current feattles with [`Feattles::reload()`]
and update their values with [`Feattles::update()`]. Please look for the crates
[feattle-sync](https://crates.io/crates/feattle-sync) and
[feattle-ui](https://crates.io/crates/feattle-ui) for higher-level functionalities.

## Limitations
Due to some restrictions on how the macro is written, you can only use [`feattles!`] once per
module. For example, the following does not compile:

```compile_fail
use feattle_core::feattles;

feattles! { struct A { } }
feattles! { struct B { } }
```

You can work around this limitation by creating a sub-module and then re-exporting the generated
struct. Note the use of `pub struct` in the second case.
```rust
use feattle_core::feattles;

feattles! { struct A { } }

mod b {
    use feattle_core::feattles;
    feattles! { pub struct B { } }
}

use b::B;
```

## Optional features

- **uuid**: will add support for [`uuid::Uuid`].

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
