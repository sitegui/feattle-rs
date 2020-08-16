use feattle_core::models::{CurrentValues, ValueHistory};
use feattle_core::persist::Persist;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter, ErrorKind};
use std::path::PathBuf;

pub struct Disk {
    dir: PathBuf,
}

impl Disk {
    pub fn new<P: Into<PathBuf>>(dir: P) -> Result<Self, Box<dyn Error>> {
        let dir = dir.into();
        create_dir_all(&dir)?;
        Ok(Disk { dir })
    }

    fn save<T: Serialize>(&self, name: &str, value: T) -> Result<(), Box<dyn Error>> {
        let file = BufWriter::new(File::create(self.dir.join(name))?);
        Ok(serde_json::to_writer(file, &value)?)
    }

    fn load<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, Box<dyn Error>> {
        match File::open(self.dir.join(name)) {
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(Box::new(err)),
            Ok(file) => Ok(Some(serde_json::from_reader(BufReader::new(file))?)),
        }
    }
}

impl Persist for Disk {
    fn save_current(&self, value: &CurrentValues) -> Result<(), Box<dyn Error>> {
        self.save("current.json", value)
    }

    fn load_current(&self) -> Result<Option<CurrentValues>, Box<dyn Error>> {
        self.load("current.json")
    }

    fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), Box<dyn Error>> {
        self.save(&format!("history-{}.json", key), value)
    }

    fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, Box<dyn Error>> {
        self.load(&format!("history-{}.json", key))
    }
}
