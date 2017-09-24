use error::RouterSinkError;
use futures::{Async, AsyncSink, Poll, Sink, StartSend};

/// Marker to decide which route the item has to take
pub enum Route<A, B> {
    /// Marker to indicate that this item is to be routed left
    Left(A),
    /// Marker to indicate that this item is to be routed right
    Right(B),
}

/// A sink capable of routing incoming items to one of two sinks
pub struct RouterSink<A, B> {
    /// The sink for the left route
    left_sink: A,
    /// The sink for the right route
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

    /// Access the inner sink for the left route
    ///
    /// # Example
    ///
    /// ```
    /// # use futures_router_sink::RouterSink;
    /// # let left = Vec::<usize>::new();
    /// # let right = Vec::<usize>::new();
    /// let router = RouterSink::new(left, right);
    /// let left = router.left();
    /// ```
    ///
    /// # Return value
    ///
    /// A reference to the inner left route sink
    pub fn left(&self) -> &A {
        &self.left_sink
    }

    /// Access the inner sink for the right route
    ///
    /// # Example
    ///
    /// ```
    /// # use futures_router_sink::RouterSink;
    /// # let left = Vec::<usize>::new();
    /// # let right = Vec::<usize>::new();
    /// let router = RouterSink::new(left, right);
    /// let right = router.right();
    /// ```
    ///
    /// # Return value
    ///
    /// A reference to the inner right route sink
    pub fn right(&self) -> &B {
        &self.right_sink
    }

    /// Mutable access the inner sink for the left route
    ///
    /// # Example
    ///
    /// ```
    /// # use futures_router_sink::RouterSink;
    /// # let left = Vec::<usize>::new();
    /// # let right = Vec::<usize>::new();
    /// let router = RouterSink::new(left, right);
    /// let right = router.left_mut();
    /// ```
    ///
    /// # Return value
    ///
    /// A mutable reference to the inner left route sink
    pub fn left_mut(&mut self) -> &mut A {
        &mut self.left_sink
    }

    /// Mutable access the inner sink for the right route
    ///
    /// # Example
    ///
    /// ```
    /// # use futures_router_sink::RouterSink;
    /// # let left = Vec::<usize>::new();
    /// # let right = Vec::<usize>::new();
    /// let router = RouterSink::new(left, right);
    /// let right = router.left_mut();
    /// ```
    ///
    /// # Return value
    ///
    /// A mutable reference to the inner right route sink
    pub fn right_mut(&mut self) -> &mut B {
        &mut self.right_sink
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


impl<A,B> Clone for RouterSink<A,B> 
where A: Clone,
      B: Clone
{
    fn clone(&self) -> Self {
        RouterSink::new(self.left_sink.clone(), self.right_sink.clone())
    }
}