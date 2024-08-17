use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_timer::Delay;
use futures_util::FutureExt;

/// Future that resolves into a value after a given duration has passed.
///
/// ```rust
/// # use std::time::{Duration, Instant};
/// # tokio_test::block_on(async {
/// use debounced_wasm::Delayed;
///
/// let start = Instant::now();
/// let delayed = Delayed::new(42, Duration::from_secs(1)).await;
/// assert_eq!(delayed, 42);
/// assert!(start.elapsed().as_secs() == 1);
/// # })
/// ```
pub struct Delayed<T> {
    sleep: Pin<Box<Delay>>,
    value: Option<T>,
}

impl<T> Delayed<T> {
    /// Returns a new future that resolves into the given value after the given
    /// duration has passed.
    pub fn new(value: T, duration: Duration) -> Delayed<T> {
        Delayed {
            sleep: Box::pin(Delay::new(duration)),
            value: Some(value),
        }
    }
}

impl<T> Unpin for Delayed<T> {}

impl<T> Future for Delayed<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.sleep.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => Poll::Ready(self.value.take().unwrap()),
        }
    }
}

/// Returns a new future that resolves into the given value after the given
/// duration has passed.
///
/// ```rust
/// # use std::time::{Duration, Instant};
/// # tokio_test::block_on(async {
/// use debounced_wasm::delayed;
///
/// let start = Instant::now();
/// let delayed = delayed(42, Duration::from_secs(1)).await;
/// assert_eq!(delayed, 42);
/// assert!(start.elapsed().as_secs() == 1);
/// # })
/// ```
pub fn delayed<T>(value: T, duration: Duration) -> Delayed<T> {
    Delayed::new(value, duration)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::delayed;

    #[tokio::test]
    async fn test_delay() {
        let start = Instant::now();
        let delayed = delayed(42, Duration::from_secs(1)).await;
        assert_eq!(start.elapsed().as_secs(), 1);
        assert_eq!(delayed, 42);
    }
}
