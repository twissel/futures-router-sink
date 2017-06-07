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
