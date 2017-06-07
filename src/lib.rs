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
//! // Create the two sinks that we are going to route into
//! let even = Vec::<usize>::new();
//! let odd = Vec::<usize>::new();
//!
//! // Create the router sink
//! let router = RouterSink::new(even, odd);
//!
//! // Some made up data where we route even numbers
//! // to the left and odd numbers to the right sink
//! let input = (0..10)
//!     .map(|x| {
//!         if x % 2 == 0 {
//!             Route::Left(x)
//!         } else {
//!             Route::Right(x)
//!         }
//!     })
//!     .map(Ok::<_, ()>)
//!     .collect::<Vec<_>>();
//!
//! stream::iter(input)
//!     .map_err(|_| RouterSinkError::Left(()))
//!     .forward(router);
//! # }
//! ```

extern crate futures;

mod error;
mod router_sink;

pub use error::RouterSinkError;
pub use router_sink::{Route, RouterSink};


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
                assert_eq!(router.left()[0], 23);
                assert_eq!(router.right()[0], 42);
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
