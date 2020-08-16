mod models;
pub mod persist;

use crate::persist::Persist;
use feattle_core::Feattles;
use std::sync::{Arc, Weak};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;

pub struct BackgroundSync<P, F> {
    ok_interval: Duration,
    err_interval: Duration,
    persist: P,
    feattles: Weak<F>,
}

impl<P: Persist, F: Feattles> BackgroundSync<P, F> {
    pub fn new(persist: P, feattles: &Arc<F>) -> Self {
        BackgroundSync {
            ok_interval: Duration::from_secs(30),
            err_interval: Duration::from_secs(30),
            persist,
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

    pub fn spawn(self) -> JoinHandle<()> {
        spawn(move || {
            while let Some(feattles) = self.feattles.upgrade() {
                match self.persist.load("current".to_owned()) {
                    Ok(value) => {
                        feattles.update(value);
                        log::debug!("Feattles updated");
                        sleep(self.ok_interval);
                    }
                    Err(err) => {
                        log::error!("Failed to sync Feattles: {:?}", err);
                        sleep(self.err_interval);
                    }
                }
            }

            log::info!("Stop background sync since Feattles got dropped")
        })
    }
}
