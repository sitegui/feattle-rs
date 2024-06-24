//! This crate is the implementation for some synchronization strategies for the feature flags
//! (called "feattles", for short).
//!
//! The crate [`feattle_core`] provides the trait [`feattle_core::persist::Persist`] as the
//! extension point to implementors of the persistence layer logic. This crates has some useful
//! concrete implementations: [`Disk`] and [`S3`]. Please refer to the
//! [main package - `feattle`](https://crates.io/crates/feattle) for more information.
//!
//! It also provides a simple way to poll the persistence layer for updates in [`BackgroundSync`].
//!
//! # Optional features
//!
//! - **aws_sdk_s3**: provides [`S3`] to integrate with AWS' S3 using the crate `aws-sdk-s3` crate
//! - **rusoto_s3**: provides [`RusotoS3`] to integrate with AWS' S3 using the crate `rusoto` crate

#[cfg(feature = "aws_sdk_s3")]
mod aws_sdk_s3;
mod background_sync;
mod disk;
#[cfg(feature = "rusoto_s3")]
mod rusoto_s3;

#[cfg(feature = "aws_sdk_s3")]
pub use aws_sdk_s3::*;
pub use background_sync::*;
pub use disk::*;
#[cfg(feature = "rusoto_s3")]
pub use rusoto_s3::*;

#[cfg(test)]
pub mod tests {
    use chrono::Utc;
    use serde_json::json;

    use feattle_core::persist::{CurrentValue, CurrentValues, HistoryEntry, Persist, ValueHistory};

    pub async fn test_persistence<P: Persist>(persistence: P) {
        // Empty state
        assert_eq!(persistence.load_current().await.unwrap(), None);
        assert_eq!(persistence.load_history("key").await.unwrap(), None);

        // Save new values and check if correctly saved
        let feattles = vec![(
            "key".to_string(),
            CurrentValue {
                modified_at: Utc::now(),
                modified_by: "someone".to_owned(),
                value: json!(17i32),
            },
        )]
        .into_iter()
        .collect();
        let current_values = CurrentValues {
            version: 17,
            date: Utc::now(),
            feattles,
        };
        persistence.save_current(&current_values).await.unwrap();
        assert_eq!(
            persistence.load_current().await.unwrap(),
            Some(current_values)
        );

        // Save history and check if correctly saved
        let history = ValueHistory {
            entries: vec![HistoryEntry {
                value: json!(17i32),
                value_overview: "overview".to_owned(),
                modified_at: Utc::now(),
                modified_by: "someone else".to_owned(),
            }],
        };
        persistence.save_history("key", &history).await.unwrap();
        assert_eq!(
            persistence.load_history("key").await.unwrap(),
            Some(history)
        );
        assert_eq!(persistence.load_history("key2").await.unwrap(), None);
    }
}
