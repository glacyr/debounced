#![warn(missing_docs)]
//! Utility for building delayed [`Future`](std::future::Future)s and debounced
//! `Stream`s that wait a given duration before yielding the most recent item.
//!
//! # Usage
//! The functionality that is implemented by this crate is pretty simple.
//! Depending on whether you want to delay a single future, or debounce an
//! entire stream, you should either use [`delayed`] or [`debounced`].
//!
//! ## Delaying a Single Value
//! If you want to delay a single future from resolving to a known value, you
//! can use [`delayed`].
//!
//! ```rust
//! # use std::time::{Duration, Instant};
//! # tokio_test::block_on(async {
//! use debounced_wasm::delayed;
//!
//! # let start = Instant::now();
//! let delayed = delayed(42, Duration::from_secs(1)).await;
//! assert_eq!(start.elapsed().as_secs(), 1);
//! assert_eq!(delayed, 42);
//! # })
//! ```
//!
//! ## Debouncing a Stream
//! If you want to debounce an entire stream, you can use [`debounced`].
//!
//! ```rust
//! # use std::time::{Duration, Instant};
//! # use futures_util::{SinkExt, StreamExt};
//! # tokio_test::block_on(async {
//! use debounced_wasm::debounced;
//!
//! # let start = Instant::now();
//! let (mut sender, receiver) = futures_channel::mpsc::channel(1024);
//! let mut debounced = debounced(receiver, Duration::from_secs(1));
//! sender.send(21).await;
//! sender.send(42).await;
//! assert_eq!(debounced.next().await, Some(42));
//! assert_eq!(start.elapsed().as_secs(), 1);
//! # })
//! ```
//!
//! # Limitations
//! - __Leading debounce.__ This library currently only implements trailing
//!   debounce. It does not implement leading debounce.

mod future;
mod stream;

pub use future::{delayed, Delayed};
pub use stream::{debounced, Debounced};
