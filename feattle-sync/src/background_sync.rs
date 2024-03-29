use feattle_core::{BoxError, Feattles};
use std::sync::{Arc, Weak};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

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
/// let toggles = Arc::new(MyToggles::new(Arc::new(NoPersistence)));
///
/// BackgroundSync::new(&toggles).start().await;
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
}

impl<F: Feattles + Sync + Send + 'static> BackgroundSync<F> {
    /// Spawn a new tokio task, returning its handle. Usually you do not want to anything with the
    /// returned handle, since the task will run by itself until the feattles instance gets dropped.
    ///
    /// Operational logs are generated with the crate [`log`].
    #[deprecated = "use `start_sync()` that will try a first update right away"]
    pub fn spawn(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(feattles) = self.feattles.upgrade() {
                match feattles.reload().await {
                    Ok(()) => {
                        log::debug!("Feattles updated");
                        sleep(self.ok_interval).await;
                    }
                    Err(err) => {
                        log::warn!("Failed to sync Feattles: {:?}", err);
                        sleep(self.err_interval).await;
                    }
                }
            }

            log::info!("Stop background sync since Feattles got dropped")
        })
    }

    /// Start the sync operation by executing an update right now and then spawning a new tokio
    /// task.
    ///
    /// This call will block until the first update returns. If it fails, the obtained error will be
    /// returned.
    ///
    /// Note that the return type is `Option<_>` and not `Result<_>`, to avoid confusion: even if
    /// the first update fails, the sync process will continue in the background.
    ///
    /// The tokio task will run by itself until the feattles instance gets dropped.
    ///
    /// Operational logs are generated with the crate [`log`].
    pub async fn start(self) -> Option<BoxError> {
        let feattles = self.feattles.upgrade()?;

        let first_error = feattles.reload().await.err();
        let first_sleep = match &first_error {
            Some(err) => {
                log::warn!("Failed to sync Feattles: {:?}", err);
                self.err_interval
            }
            None => {
                log::debug!("Feattles updated");
                self.ok_interval
            }
        };

        tokio::spawn(async move {
            sleep(first_sleep).await;

            while let Some(feattles) = self.feattles.upgrade() {
                match feattles.reload().await {
                    Ok(()) => {
                        log::debug!("Feattles updated");
                        sleep(self.ok_interval).await;
                    }
                    Err(err) => {
                        log::warn!("Failed to sync Feattles: {:?}", err);
                        sleep(self.err_interval).await;
                    }
                }
            }

            log::info!("Stop background sync since Feattles got dropped")
        });

        first_error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use feattle_core::persist::{CurrentValues, Persist, ValueHistory};
    use feattle_core::{feattles, BoxError, Feattles};
    use parking_lot::Mutex;
    use tokio::time;
    use tokio::time::Instant;

    #[derive(Debug, thiserror::Error)]
    #[error("Some error")]
    struct SomeError;

    #[derive(Clone)]
    struct MockPersistence {
        call_instants: Arc<Mutex<Vec<Instant>>>,
    }

    impl MockPersistence {
        fn new() -> Self {
            MockPersistence {
                call_instants: Arc::new(Mutex::new(vec![Instant::now()])),
            }
        }

        fn call_intervals(&self) -> Vec<Duration> {
            self.call_instants
                .lock()
                .windows(2)
                .map(|instants| instants[1] - instants[0])
                .collect()
        }
    }

    #[async_trait]
    impl Persist for MockPersistence {
        async fn save_current(&self, _value: &CurrentValues) -> Result<(), BoxError> {
            unimplemented!()
        }
        async fn load_current(&self) -> Result<Option<CurrentValues>, BoxError> {
            let mut call_instants = self.call_instants.lock();
            call_instants.push(Instant::now());
            if call_instants.len() == 3 {
                // Second call returns an error
                Err(Box::new(SomeError))
            } else {
                Ok(None)
            }
        }
        async fn save_history(&self, _key: &str, _value: &ValueHistory) -> Result<(), BoxError> {
            unimplemented!()
        }
        async fn load_history(&self, _key: &str) -> Result<Option<ValueHistory>, BoxError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test() {
        feattles! {
            struct MyToggles { }
        }

        time::pause();

        let persistence = Arc::new(MockPersistence::new());
        let toggles = Arc::new(MyToggles::new(persistence.clone()));
        BackgroundSync::new(&toggles).start().await;

        // First update: success
        // Second update after 30s: fails
        // Third update after 60s: success
        // Forth update after 30s
        loop {
            let call_intervals = persistence.call_intervals();
            if call_intervals.len() == 4 {
                assert_eq!(call_intervals[0].as_secs_f32().round() as i32, 0);
                assert_eq!(call_intervals[1].as_secs_f32().round() as i32, 30);
                assert_eq!(call_intervals[2].as_secs_f32().round() as i32, 60);
                assert_eq!(call_intervals[3].as_secs_f32().round() as i32, 30);
                break;
            }
            tokio::task::yield_now().await;
            time::sleep(Duration::from_millis(100)).await;
        }

        // No more updates
        drop(toggles);
        for _ in 0..5 {
            tokio::task::yield_now().await;
        }
        assert_eq!(persistence.call_intervals().len(), 4);
    }
}
