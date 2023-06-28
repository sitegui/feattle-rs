use async_trait::async_trait;
use feattle_core::persist::{CurrentValues, Persist, ValueHistory};
use feattle_core::BoxError;
use rusoto_core::credential::CredentialsError;
use rusoto_core::request::BufferedHttpResponse;
use rusoto_core::{HttpDispatchError, RusotoError};
use rusoto_s3::{GetObjectError, GetObjectRequest, PutObjectRequest, S3Client, S3 as RusotoS3};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt;
use tokio::io::AsyncReadExt;

/// Persist the data in an [AWS S3](https://aws.amazon.com/s3/) bucket.
///
/// To use it, make sure to activate the cargo feature `"s3"` in your `Cargo.toml`.
///
/// # Example
/// ```
/// use std::sync::Arc;
/// use feattle_core::{feattles, Feattles};
/// use feattle_sync::S3;
/// use rusoto_s3::S3Client;
/// use rusoto_core::Region;
///
/// feattles! {
///     struct MyToggles {
///         a: bool,
///     }
/// }
///
/// // Create a S3 client, read more at the official documentation https://www.rusoto.org
/// let s3_client = S3Client::new(Region::default());
///
/// let persistence = Arc::new(S3::new(s3_client, "my-bucket".to_owned(), "some/s3/prefix/".to_owned()));
/// let my_toggles = MyToggles::new(persistence);
/// ```
#[derive(Clone)]
pub struct S3 {
    client: S3Client,
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

/// Represent what can go wrong when interfacing with AWS. This is based on
/// [`rusoto_core::RusotoError`].
#[derive(Debug, thiserror::Error)]
pub enum S3Error {
    #[error("An error occurred dispatching the HTTP request")]
    HttpDispatch(HttpDispatchError),
    #[error("An error was encountered with AWS credentials.")]
    Credentials(CredentialsError),
    #[error("A validation error occurred.  Details from AWS are provided.")]
    Validation(String),
    #[error("An error occurred parsing the response payload.")]
    ParseError(String),
    #[error("An unknown error occurred.  The raw HTTP response is provided.")]
    Unknown(BufferedHttpResponse),
    #[error("An error occurred when attempting to run a future as blocking")]
    Blocking,
    #[error("Failed to serialize or deserialize JSON")]
    Json(#[from] serde_json::Error),
    #[error("Failed to read from response")]
    Io(#[from] std::io::Error),
}

impl<E> From<RusotoError<E>> for S3Error {
    fn from(error: RusotoError<E>) -> Self {
        match error {
            RusotoError::Service(_) => unreachable!(),
            RusotoError::HttpDispatch(e) => S3Error::HttpDispatch(e),
            RusotoError::Credentials(e) => S3Error::Credentials(e),
            RusotoError::Validation(e) => S3Error::Validation(e),
            RusotoError::ParseError(e) => S3Error::ParseError(e),
            RusotoError::Unknown(e) => S3Error::Unknown(e),
            RusotoError::Blocking => S3Error::Blocking,
        }
    }
}

impl S3 {
    pub fn new(client: S3Client, bucket: String, prefix: String) -> Self {
        S3 {
            client,
            bucket,
            prefix,
        }
    }

    async fn save<T: Serialize>(&self, name: &str, value: T) -> Result<(), BoxError> {
        let key = format!("{}{}", self.prefix, name);
        let contents = serde_json::to_string(&value)?;
        self.client
            .put_object(PutObjectRequest {
                body: Some(contents.into_bytes().into()),
                bucket: self.bucket.clone(),
                key,
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    async fn load<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, BoxError> {
        let key = format!("{}{}", self.prefix, name);
        match self
            .client
            .get_object(GetObjectRequest {
                bucket: self.bucket.clone(),
                key,
                ..Default::default()
            })
            .await
        {
            Err(RusotoError::Service(GetObjectError::NoSuchKey(_))) => Ok(None),
            Ok(response) => match response.body {
                None => Ok(None),
                Some(body) => {
                    let mut contents = String::new();
                    body.into_async_read().read_to_string(&mut contents).await?;
                    Ok(Some(serde_json::from_str(&contents)?))
                }
            },
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

    #[tokio::test]
    async fn s3() {
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
