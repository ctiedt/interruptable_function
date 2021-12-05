//! This crate provides the [Interruptable] trait and [Executor] struct.
//! An [Interruptable] can be used in cases where partial results
//! may be useful.
//!
//! The API is inspired by Rust's futures.

/// The default items to import
pub mod prelude {
    pub use crate::{exec_interruptable, Executor, Interruptable, Status};
}

use std::time::{Duration, Instant};

/// Error if the deadline of an [Interruptable] is missed.
/// Contains the amount of time by which the deadline was missed
/// as well as a partial result if one exists.
#[derive(Debug)]
pub struct TimeoutError<P>(Duration, Option<P>);

impl<P> TimeoutError<P> {
    /// Returns the amount by which the deadline was missed.
    pub fn late_by(&self) -> Duration {
        self.0
    }

    /// Returns the partial result of the function that caused
    /// the error.
    pub fn partial_result(&self) -> Option<&P> {
        self.1.as_ref()
    }
}

/// An [Interruptable] wraps a function that can be interrupted.
/// At this point, no preemption is possible and functions have
/// to give up control voluntarily.
pub trait Interruptable {
    /// The return type of the function.
    type Output;

    /// Check if the function has finished execution.
    fn poll(&mut self) -> Status<Self::Output>;

    /// Return the partial result, if one is available.
    fn partial_result(&self) -> Option<Self::Output>;
}

/// Used to run an [Interruptable] until done or the deadline is missed.
pub struct Executor<I, T>
where
    I: Interruptable<Output = T>,
{
    func: I,
    deadline: Duration,
}

impl<I, T> Executor<I, T>
where
    I: Interruptable<Output = T>,
{
    pub fn new(func: I, deadline: Duration) -> Self {
        Self { func, deadline }
    }

    pub fn run(&mut self) -> Result<T, TimeoutError<T>> {
        let start = Instant::now();
        loop {
            match self.func.poll() {
                Status::Done(t) => return Ok(t),
                Status::Pending => {
                    let current_time = start.elapsed();
                    if current_time >= self.deadline {
                        return Err(TimeoutError(
                            current_time - self.deadline,
                            self.func.partial_result(),
                        ));
                    }
                }
            }
        }
    }

    pub fn partial_result(&self) -> Option<T> {
        self.func.partial_result()
    }
}

/// The status of an [Interruptable]. Should be [Status::Pending] while
/// the [Interruptable] is still doing work, [Status::Done] when it has finished.
pub enum Status<T> {
    Done(T),
    Pending,
}

#[doc = "Convenience macro to run an [Interruptable] in an [Executor]."]
#[macro_export]
macro_rules! exec_interruptable {
    ($func:ident, $duration:expr) => {
        Executor::new($func, $duration).run()
    };
}
