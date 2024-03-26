use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

pub fn cancellation_signal() -> (CancellationSender, CancellationFuture) {
    let signal = Arc::new(Mutex::new(CancellationSignal {
        on: false,
        waker: None,
    }));
    (
        CancellationSender {
            signal: Arc::clone(&signal),
        },
        CancellationFuture { signal },
    )
}

#[derive(Debug)]
struct CancellationSignal {
    on: bool,
    waker: Option<Waker>,
}

#[derive(Debug)]
pub struct CancellationSender {
    signal: Arc<Mutex<CancellationSignal>>,
}
impl CancellationSender {
    pub fn send(&mut self) {
        let mut signal = self.signal.lock().unwrap();
        signal.on = true;
        if let Some(waker) = signal.waker.take() {
            waker.wake();
        }
    }

    pub fn is_on(&self) -> bool {
        self.signal.lock().unwrap().on
    }
}

#[derive(Debug, Clone)]
pub struct CancellationFuture {
    signal: Arc<Mutex<CancellationSignal>>,
}
impl Future for CancellationFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut signal = self.signal.lock().unwrap();
        if signal.on {
            Poll::Ready(())
        } else {
            signal.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
