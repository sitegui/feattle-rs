pub mod persist;

use feattle_core::persist::Persist;
use feattle_core::Feattles;
use std::sync::{Arc, Weak};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;

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

    pub fn spawn<P>(self) -> JoinHandle<()>
    where
        F: Feattles<P>,
        P: Persist,
    {
        spawn(move || {
            while let Some(feattles) = self.feattles.upgrade() {
                match feattles.reload() {
                    Ok(()) => {
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
