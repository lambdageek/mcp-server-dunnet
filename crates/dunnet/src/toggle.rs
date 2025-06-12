//! Implements a Toggle type which is a single source multiple consumer 1-bit one-shot channel.
//! The Toggle can be signaled by the owner to inform all the consumers that a change has occurred.

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use futures::FutureExt;

#[derive(Debug)]
struct ToggleState {
    toggled: AtomicBool,
}

#[derive(Debug)]
pub struct ToggleSender {
    state: Arc<ToggleState>,
}


#[derive(Debug, Clone)]
pub struct ToggleReceiver {
    state: Arc<ToggleState>,
}

pub fn new_toggle() -> (ToggleSender, ToggleReceiver) {
    let state = Arc::new(ToggleState {
        toggled: AtomicBool::new(false),
    });
    (
        ToggleSender { state: state.clone() },
        ToggleReceiver { state },
    )
}

impl ToggleSender {
    /// Notify all the receivers that a toggle has occurred.
    pub fn toggle(&mut self) {
        self.state.toggled.store(true, Ordering::SeqCst);
    }
}

impl ToggleReceiver {
    /// Check if the toggle has occurred.
    pub fn is_toggled(&self) -> bool {
        self.state.toggled.load(Ordering::SeqCst)
    }

    pub fn wait(&self) -> ToggleWaiter {
        if (self.is_toggled()) {
            ToggleWaiter::Done
        } else {
            ToggleWaiter::Waiting {
                state: self.state.clone(),
            }
        }
    }
}

#[derive(Debug)]
pub enum ToggleWaiter {
    Done,
    Waiting {
        state: Arc<ToggleState>,
    },
}

impl std::future::Future for ToggleWaiter {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Self::Output> {
        let self_mut = self.get_mut();
        match self_mut {
            ToggleWaiter::Done => std::task::Poll::Ready(()),
            ToggleWaiter::Waiting { state } => {
                if state.toggled.load(Ordering::SeqCst) {
                    *self_mut = ToggleWaiter::Done;
                    std::task::Poll::Ready(())
                } else {
                    cx.waker().clone().wake();
                    std::task::Poll::Pending
                }
            }
        }
    }
}

impl futures::future::FusedFuture for ToggleWaiter {
    fn is_terminated(&self) -> bool {
        matches!(self, ToggleWaiter::Done)
    }
}