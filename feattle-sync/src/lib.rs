use std::sync::{Arc, Weak};
use std::time::Duration;

use tokio::time::delay_for;

use feattle_core::persist::Persist;
use feattle_core::Feattles;

pub mod disk;
#[cfg(feature = "s3")]
pub mod s3;

pub struct BackgroundSync<F> {
    ok_interval: Duration,
    err_interval: Duration,
    feattles: Weak<F>,
}

impl<F> BackgroundSync<F> {
    pub fn new(feattles: &Arc<F>) -> Self {
        BackgroundSync {
            ok_interval: Duration::from_secs(30),
            err_interval: Duration::from_secs(30),
            feattles: Arc::downgrade(feattles),
        }
    }

    pub fn interval(&mut self, value: Duration) -> &mut Self {
        self.ok_interval = value;
        self.err_interval = value;
        self
    }

    pub fn ok_interval(&mut self, value: Duration) -> &mut Self {
        self.ok_interval = value;
        self
    }

    pub fn err_interval(&mut self, value: Duration) -> &mut Self {
        self.err_interval = value;
        self
    }

    pub async fn run<P>(self)
    where
        F: Feattles<P>,
        P: Persist,
    {
        while let Some(feattles) = self.feattles.upgrade() {
            match feattles.reload().await {
                Ok(()) => {
                    log::debug!("Feattles updated");
                    delay_for(self.ok_interval).await;
                }
                Err(err) => {
                    log::error!("Failed to sync Feattles: {:?}", err);
                    delay_for(self.err_interval).await;
                }
            }
        }

        log::info!("Stop background sync since Feattles got dropped")
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use feattle_core::persist::{CurrentValue, CurrentValues, HistoryEntry, ValueHistory};

    use crate::disk::Disk;

    use super::*;

    async fn test_persistence<P: Persist>(persistence: P) {
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

    #[tokio::test]
    async fn disk() {
        let dir = tempfile::TempDir::new().unwrap();
        test_persistence(Disk::new(dir.path())).await;
    }

    #[tokio::test]
    #[cfg(feature = "s3")]
    async fn s3() {
        use crate::s3::S3;
        use rusoto_core::Region;
        use rusoto_s3::{
            Delete, DeleteObjectsRequest, ListObjectsV2Request, ObjectIdentifier, S3Client,
            S3 as RusotoS3,
        };
        use std::env;

        dotenv::dotenv().ok();

        // Please set the environment variables AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY,
        // AWS_REGION, S3_BUCKET and S3_KEY_PREFIX accordingly
        let client = S3Client::new(Region::default());
        let bucket = env::var("S3_BUCKET").unwrap();
        let prefix = env::var("S3_KEY_PREFIX").unwrap();

        // Clear all previous objects
        let objects_to_delete = client
            .list_objects_v2(ListObjectsV2Request {
                bucket: bucket.clone(),
                prefix: Some(prefix.clone()),
                ..Default::default()
            })
            .await
            .unwrap()
            .contents
            .unwrap_or_default();
        let keys_to_delete: Vec<_> = objects_to_delete
            .into_iter()
            .filter_map(|o| o.key)
            .collect();

        if !keys_to_delete.is_empty() {
            println!(
                "Will first clear previous objects in S3: {:?}",
                keys_to_delete
            );
            client
                .delete_objects(DeleteObjectsRequest {
                    bucket: bucket.clone(),
                    delete: Delete {
                        objects: keys_to_delete
                            .into_iter()
                            .map(|key| ObjectIdentifier {
                                key,
                                version_id: None,
                            })
                            .collect(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .await
                .unwrap();
        }

        test_persistence(S3::new(client, bucket, prefix)).await;
    }
}
