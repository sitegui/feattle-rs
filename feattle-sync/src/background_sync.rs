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
                println!("Try to update {:?}", tokio::time::Instant::now());
                match feattles.reload().await {
                    Ok(()) => {
                        println!("Feattles updated");
                        delay_for(dbg!(self.ok_interval)).await;
                    }
                    Err(err) => {
                        println!("Failed to sync Feattles: {:?}", err);
                        delay_for(dbg!(self.err_interval)).await;
                    }
                }
            }

            println!("Stop background sync since Feattles got dropped")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use feattle_core::persist::{CurrentValues, ValueHistory};
    use feattle_core::{feattles, Feattles};
    use parking_lot::Mutex;
    use tokio::time;

    #[derive(Debug, thiserror::Error)]
    #[error("Some error")]
    struct SomeError;

    #[derive(Clone, Default)]
    struct MockPersistence(Arc<Mutex<MockPersistenceInner>>);

    #[derive(Default)]
    struct MockPersistenceInner {
        load_current_calls: i32,
        next_error: Option<SomeError>,
    }

    impl MockPersistence {
        fn load_current_calls(&self) -> i32 {
            self.0.lock().load_current_calls
        }

        fn put_error(&self) {
            let previous = self.0.lock().next_error.replace(SomeError);
            assert!(previous.is_none());
        }
    }

    #[async_trait]
    impl Persist for MockPersistence {
        type Error = SomeError;
        async fn save_current(&self, _value: &CurrentValues) -> Result<(), Self::Error> {
            unimplemented!()
        }
        async fn load_current(&self) -> Result<Option<CurrentValues>, Self::Error> {
            let next_error = {
                let mut inner = self.0.lock();
                inner.load_current_calls += 1;
                inner.next_error.take()
            };
            tokio::task::yield_now().await;
            match next_error {
                None => Ok(None),
                Some(e) => Err(e),
            }
        }
        async fn save_history(&self, _key: &str, _value: &ValueHistory) -> Result<(), Self::Error> {
            unimplemented!()
        }
        async fn load_history(&self, _key: &str) -> Result<Option<ValueHistory>, Self::Error> {
            unimplemented!()
        }
    }

    async fn measure_time(
        persistence: &MockPersistence,
        target_calls: i32,
        min_time: u64,
        max_time: u64,
    ) {
        let start = time::Instant::now();
        while persistence.load_current_calls() != target_calls {
            tokio::task::yield_now().await;
        }
        let seconds = start.elapsed().as_secs();
        assert!(seconds >= min_time, "{} >= {}", seconds, min_time);
        assert!(seconds <= max_time, "{} <= {}", seconds, max_time);
    }

    #[tokio::test]
    async fn test() {
        feattles! {
            struct MyToggles { }
        }

        time::pause();

        let persistence = MockPersistence::default();
        let toggles = Arc::new(MyToggles::new(persistence.clone()));
        BackgroundSync::new(&toggles).spawn();
        assert_eq!(persistence.load_current_calls(), 0);

        // First update: success
        measure_time(&persistence, 1, 0, 1).await;

        // Second update after 30s: fails
        println!("second update");
        persistence.put_error();
        while persistence.load_current_calls() != 2 {
            tokio::task::yield_now().await;
        }

        // Third update after 60s
        println!("third update");
        while persistence.load_current_calls() != 3 {
            tokio::task::yield_now().await;
        }

        // No more updates
        drop(toggles);
        time::advance(Duration::from_secs(30)).await;
        tokio::task::yield_now().await;
        assert_eq!(persistence.load_current_calls(), 3);
    }
}
