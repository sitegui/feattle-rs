//! Featture toggles for Rust, extensible and with background synchronization and administration UI.
//!
//! ## Example
//!
//! ```no_run
//! use rusoto_s3::S3Client;
//! use rusoto_core::Region;
//! use feattle::*;
//! use std::sync::Arc;
//!
//! /// A struct with your feature toggles
//! feattles! {
//!     struct MyFeattles {
//!         /// Is this usage considered cool?
//!         is_cool: bool = true,
//!         /// Limit the number of "blings" available.
//!         /// This will not change the number of "blengs", though!
//!         max_blings: i32,
//!         /// List the actions that should not be available
//!         blocked_actions: Vec<String>,
//!     }
//! }
//!
//! // Store their values and history in AWS' S3
//! let s3_client = S3Client::new(Region::default());
//! let persistence = S3::new(s3_client, "my-bucket".to_owned(), "some/s3/prefix/".to_owned());
//!
//! // Create a new instance
//! let my_feattles = Arc::new(MyFeattles::new(persistence));
//!
//! // Poll the storage in the background
//! BackgroundSync::new(&my_feattles).spawn();
//!
//! // Start the admin UI with `warp`
//! let admin_panel = Arc::new(AdminPanel::new(my_feattles.clone(), "Project Panda - DEV".to_owned()));
//! tokio::spawn(run_warp_server(admin_panel, ([127, 0, 0, 1], 3030)));
//!
//! // Read values (note the use of `*`)
//! assert_eq!(*my_feattles.is_cool(), true);
//! assert_eq!(*my_feattles.max_blings(), 0);
//! assert_eq!(*my_feattles.blocked_actions(), Vec::<String>::new());
//! ```
//!
//! # How it works
//!
//! The macro will generate a struct with the given name and visibility modifier (assuming private
//! by default). The generated struct implements [`Feattles`] and also exposes one method for each
//! feattle.
//!
//! The methods created for each feattle allow reading their current value. For example, for a
//! feattle `is_cool: bool`, there will be a method like
//! `pub fn is_cool(&self) -> MappedRwLockReadGuard<bool>`. Note the use of
//! [`parking_lot::MappedRwLockReadGuard`] because the interior of the struct is stored behind a `RwLock` to
//! control concurrent access.
//!
//! A feattle is created with the syntax `$key: $type [= $default]`. You can use doc coments (
//! starting with `///`) to describe nicely what they do in your system. You can use any type that
//! implements [`FeattleValue`] and optionally provide a default. If not provided, the default
//! will be created with `Default::default()`.
//!
//! # Updating values
//! This crate only disposes of low-level methods to load current feattles with [`Feattles::reload()`]
//! and update their values with [`Feattles::update()`]. Please look for the crates
//! [feattle-sync](https://crates.io/crates/feattle-sync) and
//! [feattle-ui](https://crates.io/crates/feattle-ui) for higher-level functionalities.
//!
//! # Limitations
//! Due to some restrictions on how the macro is written, you can only use [`feattles!`] once per
//! module. For example, the following does not compile:
//!
//! ```compile_fail
//! use feattle_core::feattles;
//!
//! feattles! { struct A { } }
//! feattles! { struct B { } }
//! ```
//!
//! You can work around this limitation by creating a sub-module and then re-exporting the generated
//! struct. Note the use of `pub struct` in the second case.
//! ```
//! use feattle_core::feattles;
//!
//! feattles! { struct A { } }
//!
//! mod b {
//!     use feattle_core::feattles;
//!     feattles! { pub struct B { } }
//! }
//!
//! use b::B;
//! ```
//!
//! # Optional features
//!
//! - **uuid**: will add support for [`uuid::Uuid`].
//! - **s3**: provides [`S3`] to integrate with AWS' S3
//! - **warp**: provides [`run_warp_server`] for a read-to-use integration with [`warp`]
//!
//! ## Organization
//!
//! * `feattle-core`: [![Crates.io](https://img.shields.io/crates/v/feattle-core.svg)](https://crates.io/crates/feattle-core)
//! * `feattle-sync`: [![Crates.io](https://img.shields.io/crates/v/feattle-sync.svg)](https://crates.io/crates/feattle-sync)
//! * `feattle-ui`: [![Crates.io](https://img.shields.io/crates/v/feattle-ui.svg)](https://crates.io/crates/feattle-ui)
//! * `feattle`: [![Crates.io](https://img.shields.io/crates/v/feattle.svg)](https://crates.io/crates/feattle)

pub use feattle_core::*;
pub use feattle_sync::*;
pub use feattle_ui::*;
