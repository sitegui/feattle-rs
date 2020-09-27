use async_trait::async_trait;
use feattle_core::persist::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::ErrorKind;
use std::path::PathBuf;
use tokio::fs::{create_dir_all, File};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Disk {
    dir: PathBuf,
}

impl Disk {
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        let dir = dir.into();
        Disk { dir }
    }

    async fn save<T: Serialize>(&self, name: &str, value: T) -> io::Result<()> {
        create_dir_all(&self.dir).await?;

        let contents = serde_json::to_string(&value)?;
        let mut file = File::create(self.dir.join(name)).await?;
        Ok(file.write_all(contents.as_bytes()).await?)
    }

    async fn load<T: DeserializeOwned>(&self, name: &str) -> io::Result<Option<T>> {
        match File::open(self.dir.join(name)).await {
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err),
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents).await?;
                Ok(Some(serde_json::from_str(&contents)?))
            }
        }
    }
}

#[async_trait]
impl Persist for Disk {
    type Error = io::Error;

    async fn save_current(&self, value: &CurrentValues) -> Result<(), Self::Error> {
        self.save("current.json", value).await
    }

    async fn load_current(&self) -> Result<Option<CurrentValues>, Self::Error> {
        self.load("current.json").await
    }

    async fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), Self::Error> {
        self.save(&format!("history-{}.json", key), value).await
    }

    async fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, Self::Error> {
        self.load(&format!("history-{}.json", key)).await
    }
}
