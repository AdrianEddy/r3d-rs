// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use std::{ future::Future, pin::Pin, sync::{ atomic::{ AtomicBool, Ordering}, Arc, Mutex }, task::{ Context, Poll }};
use futures_util::task::AtomicWaker;
use super::*;

pub(crate) struct State<T> {
    pub(crate) waker: AtomicWaker,
    pub(crate) done: AtomicBool,
    pub(crate) result: Mutex<Option<Result<T, RedError>>>,
    pub(crate) job: Option<T>,
}
impl<T> State<T> {
    pub(crate) fn new(job: T) -> Self {
        Self {
            waker: AtomicWaker::new(),
            done: AtomicBool::new(false),
            result: Mutex::new(None),
            job: Some(job),
        }
    }
}
pub struct CallbackFuture<T> {
    pub(crate) state: Arc<State<T>>,
}
impl<T> Future for CallbackFuture<T> {
    type Output = Result<T, RedError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.state.waker.register(cx.waker());
        if this.state.done.load(Ordering::Acquire) {
            if let Some(res) = this.state.result.lock().unwrap().take() {
                return Poll::Ready(res);
            }
        }
        Poll::Pending
    }
}
