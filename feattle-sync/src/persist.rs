use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

pub trait Persist: Send + 'static {
    fn save<T: Serialize>(&self, name: String, value: T) -> Result<(), Box<dyn Error>>;
    fn load<T: DeserializeOwned>(&self, name: String) -> Result<T, Box<dyn Error>>;
}

pub struct Disk {
    dir: PathBuf,
}

impl Disk {
    pub fn new<P: Into<PathBuf>>(dir: P) -> Result<Self, Box<dyn Error>> {
        let dir = dir.into();
        create_dir_all(&dir)?;
        Ok(Disk { dir })
    }
}

impl Persist for Disk {
    fn save<T: Serialize>(&self, mut name: String, value: T) -> Result<(), Box<dyn Error>> {
        name += ".json";
        let file = BufWriter::new(File::create(self.dir.join(name))?);
        Ok(serde_json::to_writer(file, &value)?)
    }

    fn load<T: DeserializeOwned>(&self, mut name: String) -> Result<T, Box<dyn Error>> {
        name += ".json";
        let file = BufReader::new(File::open(self.dir.join(name))?);
        Ok(serde_json::from_reader(file)?)
    }
}
