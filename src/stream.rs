use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::{FutureExt, Stream, StreamExt};

use super::{delayed, Delayed};

/// Stream that delays its items for a given duration and only yields the most
/// recent item afterwards.
///
/// ```rust
/// # use std::time::{Duration, Instant};
/// # use futures_util::{SinkExt, StreamExt};
/// # tokio_test::block_on(async {
/// use debounced::Debounced;
///
/// # let start = Instant::now();
/// let (mut sender, receiver) = futures_channel::mpsc::channel(1024);
/// let mut debounced = Debounced::new(receiver, Duration::from_secs(1));
/// sender.send(21).await;
/// sender.send(42).await;
/// assert_eq!(debounced.next().await, Some(42));
/// assert_eq!(start.elapsed().as_secs(), 1);
/// std::mem::drop(sender);
/// assert_eq!(debounced.next().await, None);
/// # })
pub struct Debounced<S>
where
    S: Stream,
{
    stream: S,
    delay: Duration,
    pending: Option<Delayed<S::Item>>,
}

impl<S> Debounced<S>
where
    S: Stream + Unpin,
{
    /// Returns a new stream that delays its items for a given duration and only
    /// yields the most recent item afterwards.
    pub fn new(stream: S, delay: Duration) -> Debounced<S> {
        Debounced {
            stream,
            delay,
            pending: None,
        }
    }
}

impl<S> Stream for Debounced<S>
where
    S: Stream + Unpin,
{
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        while let Poll::Ready(next) = self.stream.poll_next_unpin(cx) {
            match next {
                Some(next) => self.pending = Some(delayed(next, self.delay)),
                None => {
                    if self.pending.is_none() {
                        return Poll::Ready(None);
                    }
                    break;
                }
            }
        }

        match self.pending.as_mut() {
            Some(pending) => match pending.poll_unpin(cx) {
                Poll::Ready(value) => {
                    let _ = self.pending.take();
                    Poll::Ready(Some(value))
                }
                Poll::Pending => Poll::Pending,
            },
            None => Poll::Pending,
        }
    }
}

/// Returns a new stream that delays its items for a given duration and only
/// yields the most recent item afterwards.
///
/// ```rust
/// # use std::time::{Duration, Instant};
/// # use futures_util::{SinkExt, StreamExt};
/// # tokio_test::block_on(async {
/// use debounced::debounced;
///
/// # let start = Instant::now();
/// let (mut sender, receiver) = futures_channel::mpsc::channel(1024);
/// let mut debounced = debounced(receiver, Duration::from_secs(1));
/// sender.send(21).await;
/// sender.send(42).await;
/// assert_eq!(debounced.next().await, Some(42));
/// assert_eq!(start.elapsed().as_secs(), 1);
/// std::mem::drop(sender);
/// assert_eq!(debounced.next().await, None);
/// # })
pub fn debounced<S>(stream: S, delay: Duration) -> Debounced<S>
where
    S: Stream + Unpin,
{
    Debounced::new(stream, delay)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use futures_channel::mpsc::channel;
    use futures_util::future::join;
    use futures_util::{SinkExt, StreamExt};
    use tokio::time::sleep;

    use super::debounced;

    #[tokio::test]
    async fn test_debounce() {
        let start = Instant::now();
        let (mut sender, receiver) = futures_channel::mpsc::channel(1024);
        let mut debounced = debounced(receiver, Duration::from_secs(1));
        let _ = sender.send(21).await;
        let _ = sender.send(42).await;
        assert_eq!(debounced.next().await, Some(42));
        assert_eq!(start.elapsed().as_secs(), 1);
        std::mem::drop(sender);
        assert_eq!(debounced.next().await, None);
    }

    #[tokio::test]
    async fn test_debounce_order() {
        #[derive(Debug, PartialEq, Eq)]
        pub enum Message {
            Value(u64),
            SenderEnded,
            ReceiverEnded,
        }

        let (mut sender, receiver) = channel(1024);
        let mut receiver = debounced(receiver, Duration::from_millis(100));
        let messages = Arc::new(Mutex::new(vec![]));

        join(
            {
                let messages = messages.clone();
                async move {
                    for i in 0..10u64 {
                        let _ = sleep(Duration::from_millis(23 * i)).await;
                        let _ = sender.send(i).await;
                    }

                    messages.lock().unwrap().push(Message::SenderEnded);
                }
            },
            {
                let messages = messages.clone();

                async move {
                    while let Some(value) = receiver.next().await {
                        messages.lock().unwrap().push(Message::Value(value));
                    }

                    messages.lock().unwrap().push(Message::ReceiverEnded);
                }
            },
        )
        .await;

        assert_eq!(
            messages.lock().unwrap().as_slice(),
            &[
                Message::Value(4),
                Message::Value(5),
                Message::Value(6),
                Message::Value(7),
                Message::Value(8),
                Message::SenderEnded,
                Message::Value(9),
                Message::ReceiverEnded
            ]
        );
    }
}
