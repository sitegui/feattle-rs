use async_trait::async_trait;
use feattle_core::persist::{CurrentValues, Persist, ValueHistory};
use rusoto_core::credential::CredentialsError;
use rusoto_core::request::BufferedHttpResponse;
use rusoto_core::{HttpDispatchError, RusotoError};
use rusoto_s3::{GetObjectError, GetObjectRequest, PutObjectRequest, S3Client, S3 as RusotoS3};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::AsyncReadExt;

pub struct S3 {
    client: S3Client,
    bucket: String,
    prefix: String,
}

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

    async fn save<T: Serialize>(&self, name: &str, value: T) -> Result<(), S3Error> {
        let key = format!("{}/{}", self.prefix, name);
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

    async fn load<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, S3Error> {
        let key = format!("{}/{}", self.prefix, name);
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
    type Error = S3Error;

    async fn save_current(&self, value: &CurrentValues) -> Result<(), S3Error> {
        self.save("current.json", value).await
    }

    async fn load_current(&self) -> Result<Option<CurrentValues>, S3Error> {
        self.load("current.json").await
    }

    async fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), S3Error> {
        self.save(&format!("history-{}.json", key), value).await
    }

    async fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, S3Error> {
        self.load(&format!("history-{}.json", key)).await
    }
}
