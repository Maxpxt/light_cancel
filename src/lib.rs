pub mod signal;

use signal::{cancellation_signal, CancellationFuture, CancellationSender};
use std::{
    error, fmt,
    future::Future,
    pin::Pin,
    task::{self, Poll},
};

pub fn cancellable<F: Future, C: Future<Output = ()>>(cancel: C, future: F) -> Cancellable<F, C> {
    Cancellable::new(cancel, future)
}

pub fn cancellable_with_signal<F: Future>(
    future: F,
) -> (Cancellable<F, CancellationFuture>, CancellationSender) {
    Cancellable::new_with_signal(future)
}

pub struct Cancellable<F, C> {
    cancel: C,
    future: F,
}
impl<F, C: Future<Output = ()>> Cancellable<F, C> {
    pub fn new(cancel: C, future: F) -> Self {
        Self { cancel, future }
    }
}
impl<F> Cancellable<F, CancellationFuture> {
    pub fn new_with_signal(future: F) -> (Self, CancellationSender) {
        let (cancel_tx, cancel) = cancellation_signal();

        (Self { cancel, future }, cancel_tx)
    }
}
impl<F: Future, C: Future<Output = ()>> Future for Cancellable<F, C> {
    type Output = Result<F::Output, Cancelled>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        // SAFETY:
        //   The `future` and `cancellation_rx` fields can be pinned
        //   because they are fields of a pinned value.
        let (cancel, future) = unsafe {
            let Self { cancel, future } = self.get_unchecked_mut();

            (Pin::new_unchecked(cancel), Pin::new_unchecked(future))
        };

        if let Poll::Ready(()) = cancel.poll(cx) {
            Poll::Ready(Err(Cancelled))
        } else if let Poll::Ready(result) = future.poll(cx) {
            Poll::Ready(Ok(result))
        } else {
            Poll::Pending
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cancelled;
impl fmt::Display for Cancelled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the task was cancelled")
    }
}
impl error::Error for Cancelled {}
