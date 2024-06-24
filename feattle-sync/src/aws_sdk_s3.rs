use async_trait::async_trait;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use aws_types::SdkConfig;
use feattle_core::persist::{CurrentValues, Persist, ValueHistory};
use feattle_core::BoxError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt;

/// Persist the data in an [AWS S3](https://aws.amazon.com/s3/) bucket.
///
/// To use it, make sure to activate the cargo feature `"aws_sdk_s3"` in your `Cargo.toml`.
///
/// # Example
/// ```
/// use std::sync::Arc;
/// use std::time::Duration;
/// use feattle_core::{feattles, Feattles};
/// use rusoto_core::Region;
///
/// feattles! {
///     struct MyToggles {
///         a: bool,
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     // Create an AWS config, read more at the official documentation <https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html>
///     let config = aws_config::load_from_env().await;
///    
///     let timeout = Duration::from_secs(10);
///     let persistence = Arc::new(S3::new(
///         &config,
///         "my-bucket".to_owned(),
///         "some/s3/prefix/".to_owned(),
///         timeout,
///     ));
///     let my_toggles = MyToggles::new(persistence);
/// }
/// ```
#[derive(Clone)]
pub struct S3 {
    client: Client,
    bucket: String,
    prefix: String,
}

impl fmt::Debug for S3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("S3")
            .field("client", &"S3Client")
            .field("bucket", &self.bucket)
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl S3 {
    pub fn new(config: &SdkConfig, bucket: String, prefix: String) -> Self {
        S3 {
            client: Client::new(config),
            bucket,
            prefix,
        }
    }

    async fn save<T: Serialize>(&self, name: &str, value: T) -> Result<(), BoxError> {
        let key = format!("{}{}", self.prefix, name);
        let contents = serde_json::to_vec(&value)?;
        self.client
            .put_object()
            .bucket(self.bucket.clone())
            .key(key)
            .body(ByteStream::from(contents))
            .send()
            .await?;

        Ok(())
    }

    async fn load<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, BoxError> {
        let key = format!("{}{}", self.prefix, name);
        let get_object = self
            .client
            .get_object()
            .bucket(self.bucket.clone())
            .key(key)
            .send()
            .await
            .map_err(|x| x.into_service_error());
        match get_object {
            Err(GetObjectError::NoSuchKey(_)) => Ok(None),
            Ok(response) => {
                let contents = response.body.collect().await?.to_vec();
                Ok(Some(serde_json::from_slice(&contents)?))
            }
            Err(error) => Err(error.into()),
        }
    }
}

#[async_trait]
impl Persist for S3 {
    async fn save_current(&self, value: &CurrentValues) -> Result<(), BoxError> {
        self.save("current.json", value).await
    }

    async fn load_current(&self) -> Result<Option<CurrentValues>, BoxError> {
        self.load("current.json").await
    }

    async fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), BoxError> {
        self.save(&format!("history-{}.json", key), value).await
    }

    async fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, BoxError> {
        self.load(&format!("history-{}.json", key)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_persistence;
    use aws_sdk_s3::types::{Delete, ObjectIdentifier};

    #[tokio::test]
    async fn s3() {
        use std::env;

        dotenv::dotenv().ok();

        // Please set the environment variables AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY,
        // AWS_REGION, S3_BUCKET and S3_KEY_PREFIX accordingly
        let config = aws_config::load_from_env().await;
        let bucket = env::var("S3_BUCKET").unwrap();
        let prefix = format!("{}/aws-sdk-s3", env::var("S3_KEY_PREFIX").unwrap());
        let client = Client::new(&config);

        // Clear all previous objects
        let objects_to_delete = client
            .list_objects_v2()
            .bucket(&bucket)
            .prefix(&prefix)
            .send()
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

            let mut delete_builder = Delete::builder();
            for key in keys_to_delete {
                delete_builder =
                    delete_builder.objects(ObjectIdentifier::builder().key(key).build().unwrap());
            }

            client
                .delete_objects()
                .bucket(&bucket)
                .delete(delete_builder.build().unwrap())
                .send()
                .await
                .unwrap();
        }

        test_persistence(S3::new(&config, bucket, prefix)).await;
    }
}
