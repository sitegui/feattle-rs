pub mod disk;
#[cfg(feature = "s3")]
pub mod s3;

use feattle_core::persist::Persist;
use feattle_core::Feattles;
use std::sync::{Arc, Weak};
use std::time::Duration;
use tokio::time::delay_for;

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
