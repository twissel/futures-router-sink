//! Router Sink
//!
//! # Example
//!
//! ```no_run
//! # extern crate futures;
//! # extern crate futures_router_sink;
//! # fn main() {
//! use futures::{stream, Stream};
//! use futures_router_sink::{Route, RouterSink, RouterSinkError};
//!
//! let even = Vec::<usize>::new();
//! let odd = Vec::<usize>::new();
//!
//! let router = RouterSink::new(even, odd);
//!
//! let input = (0..10).map(|x| {
//!     if x % 2 == 0 {
//!         Route::Left(x)
//!     } else {
//!         Route::Right(x)
//!     }
//! })
//! .map(Ok::<_, ()>)
//! .collect::<Vec<_>>();
//!
//! let stream = stream::iter(input);
//! stream
//!     .map_err(|_| RouterSinkError::Left(()))
//!     .forward(router);
//! # }
//! ```

extern crate futures;

use futures::{Async, AsyncSink, Poll, Sink, StartSend};

/// Encapsulate errors from both Sinks
pub enum RouterSinkError<A, B> {
    Left(A),
    Right(B),
}

/// Marker to decide which route the item has to take
pub enum Route<A, B> {
    Left(A),
    Right(B),
}

/// A sink capable of routing incoming items to one of two sinks
pub struct RouterSink<A, B> {
    left_sink: A,
    right_sink: B,
}

/// Poll the given sink and map the error to an appropriate type with
/// the given conversion function
fn poll_complete<S, F, E>(sink: &mut S, f: F) -> Poll<(), E>
    where S: Sink,
          F: Fn(S::SinkError) -> E
{
    match sink.poll_complete() {
            Ok(Async::Ready(x)) => Ok(Async::Ready(x)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
        .map_err(f)
}

/// Start sending an item on the given sink, map the item back to its route
/// if `NotReady` and map the error to an appropriate type
/// with the given conversion function
fn start_send<S, F, E, G, I>(sink: &mut S, item: S::SinkItem, f: F, g: G) -> StartSend<I, E>
    where S: Sink,
          F: Fn(S::SinkError) -> E,
          G: Fn(S::SinkItem) -> I
{
    sink.start_send(item)
        .map(|x| match x {
                 AsyncSink::Ready => AsyncSink::Ready,
                 AsyncSink::NotReady(x) => AsyncSink::NotReady(g(x)),
             })
        .map_err(f)
}

impl<A, B> RouterSink<A, B> {
    /// Create a new RouterrSink for the two given sinks
    ///
    /// # Example
    ///
    /// ```
    /// use futures_router_sink::RouterSink;
    ///
    /// let left = Vec::<usize>::new();
    /// let right = Vec::<usize>::new();
    ///
    /// let router = RouterSink::new(left, right);
    /// ```
    ///
    /// # Arguments
    ///
    /// - `left_sink`: The sink chosen by the router if an item is tagged as `Left`
    /// - `right_sink`: The sink chosen by the router if an item is tagged as `Right`
    pub fn new(left_sink: A, right_sink: B) -> RouterSink<A, B> {
        RouterSink {
            left_sink,
            right_sink,
        }
    }
}

impl<A, B> Sink for RouterSink<A, B>
    where A: Sink,
          B: Sink
{
    type SinkItem = Route<A::SinkItem, B::SinkItem>;
    type SinkError = RouterSinkError<A::SinkError, B::SinkError>;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        match item {
            Route::Left(x) => {
                start_send(&mut self.left_sink, x, RouterSinkError::Left, Route::Left)
            }
            Route::Right(x) => {
                start_send(&mut self.right_sink,
                           x,
                           RouterSinkError::Right,
                           Route::Right)
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        match (poll_complete(&mut self.left_sink, RouterSinkError::Left),
               poll_complete(&mut self.right_sink, RouterSinkError::Right)) {
            (Ok(Async::Ready(())), Ok(Async::Ready(()))) => Ok(Async::Ready(())),
            (Err(e), _) | (_, Err(e)) => Err(e),
            (Ok(Async::NotReady), _) |
            (_, Ok(Async::NotReady)) => Ok(Async::NotReady),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Route, RouterSink, RouterSinkError};
    use futures::{Future, stream, Stream};

    #[test]
    fn poll_all() {
        let a: Vec<u32> = Vec::new();
        let b: Vec<u32> = Vec::new();

        let input: Vec<Result<_, ()>> = vec![Ok(Route::Left(23)), Ok(Route::Right(42))];
        let stream = stream::iter(input);

        let router = RouterSink::new(a, b);

        match stream
                  .map_err(|_| RouterSinkError::Left(()))
                  .forward(router)
                  .wait() {
            Ok((_, router)) => {
                assert_eq!(router.left_sink[0], 23);
                assert_eq!(router.right_sink[0], 42);
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn poll_err() {
        let a: Vec<u32> = Vec::new();
        let b: Vec<u32> = Vec::new();

        let input = vec![Ok(Route::Left(23)), Err(())];
        let stream = stream::iter(input);

        let router = RouterSink::new(a, b);

        assert_eq!(true,
                   stream
                       .map_err(|_| RouterSinkError::Left(()))
                       .forward(router)
                       .wait()
                       .is_err());
    }
}
