// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Lifecycle tests for the `CallbackFuture` / `State` co-ownership model.
//!
//! These exercise the real `State` type and the real `Arc::into_raw` /
//! `Arc::from_raw` balance used by `AsyncDecoder::decode_for_gpu_sdk`
//! (and `R3dDecoder::decode`) — without the SDK ever running a decode, so
//! they need only the SDK link, not a clip or a GPU. They are the
//! hardware-free proof that dropping a pending future does NOT free the
//! state out from under a not-yet-fired completion callback.

use crate::future::{flush_deferred_drops, release_from_callback, CallbackFuture, State};
use crate::AsyncDecompressJob;
use std::sync::{mpsc, Arc};
use std::thread::{self, ThreadId};

/// Stands in for a job: reports the thread it was dropped on.
struct DropProbe(mpsc::Sender<ThreadId>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        let _ = self.0.send(thread::current().id());
    }
}

/// When the future is already gone, the callback's reference is the last
/// one. Releasing it must not destroy the job on the callback's own thread
/// (inside the SDK's callback), but on the reaper thread.
#[test]
fn callback_release_of_orphaned_state_drops_off_thread() {
    let (tx, rx) = mpsc::channel();
    let state = Arc::new(State::new(DropProbe(tx)));
    let sdk_ptr = Arc::into_raw(state.clone());
    drop(state); // the future is dropped before the callback fires

    // The callback, as in `async_decode_callback` / `decode_callback`.
    let reclaimed = unsafe { Arc::from_raw(sdk_ptr) };
    release_from_callback(reclaimed);

    flush_deferred_drops();
    let dropped_on = rx.try_recv().expect("orphaned job dropped by the time the flush returns");
    assert_ne!(dropped_on, thread::current().id(), "job must not be dropped inside the callback");
}

/// While the future is alive, the callback only gives up its reference:
/// the job stays put for `poll` to hand back.
#[test]
fn callback_release_with_live_future_keeps_state() {
    let (tx, rx) = mpsc::channel();
    let state = Arc::new(State::new(DropProbe(tx)));
    let sdk_ptr = Arc::into_raw(state.clone());

    release_from_callback(unsafe { Arc::from_raw(sdk_ptr) });
    flush_deferred_drops();
    assert!(rx.try_recv().is_err(), "job dropped while the future still owns it");
    assert_eq!(Arc::strong_count(&state), 1);
    assert!(state.job.lock().unwrap().is_some());

    drop(state);
    assert_eq!(rx.try_recv().unwrap(), thread::current().id());
}

/// Dropping the future while the SDK still holds its handed-out refcount
/// must keep `State` (and the job it owns) alive. This is the property the
/// `Arc::into_raw` co-ownership buys us — a bare `Arc::as_ptr` would have
/// freed the state on `drop(state)` below, and the later callback's
/// `Arc::from_raw` would have been a use-after-free.
#[test]
fn dropping_future_keeps_state_alive_until_callback() {
    let job = AsyncDecompressJob::new();
    let state = Arc::new(State::new(job));

    // submit(): hand the SDK an owned refcount.
    let sdk_ptr = Arc::into_raw(state.clone());
    assert_eq!(Arc::strong_count(&state), 2, "future + SDK both own the state");

    // The caller drops the future before the callback fires (e.g. a seek).
    drop(state);

    // The SDK's reference still keeps the state alive — reconstruct it the
    // way the completion callback does and confirm it is fully intact.
    let reclaimed = unsafe { Arc::from_raw(sdk_ptr) };
    assert_eq!(Arc::strong_count(&reclaimed), 1, "SDK holds the last reference");
    assert!(
        reclaimed.job.lock().unwrap().is_some(),
        "job (and its output buffer) survived the future drop",
    );

    // Final release: frees State and deletes the C++ AsyncDecompressJob.
    drop(reclaimed);
}

/// `abort()` must reach the in-flight job, and must be a safe no-op once the
/// result has been observed (the job slot is empty).
#[test]
fn abort_reaches_job_and_is_safe_after_drain() {
    let job = AsyncDecompressJob::new();
    let fut = CallbackFuture { state: Arc::new(State::new(job)) };

    // In-flight: abort() locks the slot and sets AbortDecode on the job.
    fut.abort();
    assert!(fut.state.job.lock().unwrap().is_some());

    // Simulate `poll` having handed the job back to the caller.
    let _drained = fut.state.job.lock().unwrap().take();

    // Aborting an already-drained future must not panic.
    fut.abort();
}
