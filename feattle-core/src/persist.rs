use crate::models::{CurrentValues, ValueHistory};
use std::error::Error;

pub trait Persist: Send + Sync + 'static {
    fn save_current(&self, value: &CurrentValues) -> Result<(), Box<dyn Error>>;
    fn load_current(&self) -> Result<Option<CurrentValues>, Box<dyn Error>>;

    fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), Box<dyn Error>>;
    fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, Box<dyn Error>>;
}
