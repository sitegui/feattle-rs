use feattle_core::persist::Persist;
use feattle_core::Feattles;
use std::sync::{Arc, Weak};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::delay_for;

/// Spawn a tokio task to poll [`Feattles::reload()`] continuously
///
/// A feattles instance will only ask the persistence layer for the current values when the
/// [`Feattles::reload()`] method is called. This type would do so regularly for you, until the
/// [`Feattles`] instance is dropped.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() {
/// use feattle_core::{feattles, Feattles};
/// use feattle_sync::BackgroundSync;
/// use feattle_core::persist::NoPersistence;
/// use std::sync::Arc;
///
/// feattles! {
///     struct MyToggles {
///         a: bool,
///     }
/// }
///
/// // `NoPersistence` here is just a mock for the sake of the example
/// let toggles = Arc::new(MyToggles::new(NoPersistence));
///
/// BackgroundSync::new(&toggles).spawn();
/// # }
/// ```
#[derive(Debug)]
pub struct BackgroundSync<F> {
    ok_interval: Duration,
    err_interval: Duration,
    feattles: Weak<F>,
}

impl<F> BackgroundSync<F> {
    /// Create a new poller for the given feattles instance. It will call [`Arc::downgrade()`] to
    /// detect when the value is dropped.
    pub fn new(feattles: &Arc<F>) -> Self {
        BackgroundSync {
            ok_interval: Duration::from_secs(30),
            err_interval: Duration::from_secs(60),
            feattles: Arc::downgrade(feattles),
        }
    }

    /// Set both [`Self::ok_interval`] and [`Self::err_interval`]
    pub fn interval(&mut self, value: Duration) -> &mut Self {
        self.ok_interval = value;
        self.err_interval = value;
        self
    }

    /// After a successful reload, will wait for this long before starting the next one. By default
    /// this is 30 seconds.
    pub fn ok_interval(&mut self, value: Duration) -> &mut Self {
        self.ok_interval = value;
        self
    }

    /// After a failed reload, will wait for this long before starting the next one. By default
    /// this is 60 seconds.
    pub fn err_interval(&mut self, value: Duration) -> &mut Self {
        self.err_interval = value;
        self
    }

    /// Spawn a new tokio task, returning its handle. Usually you do not want to anything with the
    /// returned handle, since the task will run by itself until the feattles instance gets dropped.
    ///
    /// Operational logs are generated with the crate [`log`].
    pub fn spawn<P>(self) -> JoinHandle<()>
    where
        F: Feattles<P>,
        P: Persist,
    {
        tokio::spawn(async move {
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
        })
    }
}
