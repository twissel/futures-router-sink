use std::fmt::{Debug, Display, Error, Formatter};

/// Encapsulate errors from both Sinks
pub enum RouterSinkError<A, B> {
    /// An error occured in the left route sink
    Left(A),
    /// An error occured in the right route sink
    Right(B),
}

impl<A, B> Display for RouterSinkError<A, B>
    where A: Display,
          B: Display
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            &RouterSinkError::Left(ref x) => x.fmt(f),
            &RouterSinkError::Right(ref x) => x.fmt(f),
        }
    }
}

impl<A, B> Debug for RouterSinkError<A, B>
    where A: Debug,
          B: Debug
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            &RouterSinkError::Left(ref x) => x.fmt(f),
            &RouterSinkError::Right(ref x) => x.fmt(f),
        }
    }
}
