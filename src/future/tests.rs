// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Lifecycle tests for the `CallbackFuture` / `State` co-ownership model.
//!
//! These exercise the real `State` type and the real `Arc::into_raw` /
//! `Arc::from_raw` balance used by `AsyncDecoder::decode_for_gpu_sdk`
//! (and `R3dDecoder::decode`) — without the SDK ever running a decode. They
//! are the hardware-free proof that dropping a pending future does NOT free
//! the state out from under a not-yet-fired completion callback.

use crate::future::{flush_deferred_drops, release_from_callback, CallbackFuture, State};
use crate::{AsyncDecompressJob, InitializeFlags, Sdk};
use std::sync::{mpsc, Arc, OnceLock};
use std::thread::{self, ThreadId};

/// Stands in for a job or a decoder: reports its name and the thread it was
/// dropped on.
struct DropProbe(&'static str, mpsc::Sender<(&'static str, ThreadId)>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        let _ = self.1.send((self.0, thread::current().id()));
    }
}

/// The SDK, loaded once for the tests that need it, from the redistributable
/// folder of `$R3DSDK_DIR` (default `sdk` in the crate root, as in `build.rs`).
fn sdk() -> &'static Sdk {
    static SDK: OnceLock<Sdk> = OnceLock::new();
    SDK.get_or_init(|| {
        let root = std::env::var("R3DSDK_DIR").unwrap_or_else(|_| format!("{}/sdk", env!("CARGO_MANIFEST_DIR")));
        let os = if cfg!(windows) { "win" } else if cfg!(target_os = "macos") { "mac" } else { "linux" };
        Sdk::initialize(&format!("{root}/Redistributable/{os}"), InitializeFlags::None).expect("R3D SDK init")
    })
}

/// When the future is already gone, the callback's reference is the last
/// one. Releasing it must not destroy the job on the callback's own thread
/// (inside the SDK's callback), but on the reaper thread.
#[test]
fn callback_release_of_orphaned_state_drops_off_thread() {
    let (tx, rx) = mpsc::channel();
    let state = Arc::new(State::new(DropProbe("job", tx), None));
    let sdk_ptr = Arc::into_raw(state.clone());
    drop(state); // the future is dropped before the callback fires

    // The callback, as in `async_decode_callback` / `decode_callback`.
    let reclaimed = unsafe { Arc::from_raw(sdk_ptr) };
    release_from_callback(reclaimed);

    flush_deferred_drops();
    let (_, dropped_on) = rx.try_recv().expect("orphaned job dropped by the time the flush returns");
    assert_ne!(dropped_on, thread::current().id(), "job must not be dropped inside the callback");
}

/// While the future is alive, the callback only gives up its reference:
/// the job stays put for `poll` to hand back.
#[test]
fn callback_release_with_live_future_keeps_state() {
    let (tx, rx) = mpsc::channel();
    let state = Arc::new(State::new(DropProbe("job", tx), None));
    let sdk_ptr = Arc::into_raw(state.clone());

    release_from_callback(unsafe { Arc::from_raw(sdk_ptr) });
    flush_deferred_drops();
    assert!(rx.try_recv().is_err(), "job dropped while the future still owns it");
    assert_eq!(Arc::strong_count(&state), 1);
    assert!(state.job.lock().unwrap().is_some());

    drop(state);
    assert_eq!(rx.try_recv().unwrap(), ("job", thread::current().id()));
}

/// Dropping the future while the SDK still holds its handed-out refcount
/// must keep `State` — the job and the decoder it was submitted to — alive.
/// This is the property the `Arc::into_raw` co-ownership buys us: a bare
/// `Arc::as_ptr` would have freed the state on `drop(state)` below, and the
/// later callback's `Arc::from_raw` would have been a use-after-free. The job
/// is released before its decoder.
#[test]
fn dropping_future_keeps_job_and_decoder_alive_until_callback() {
    let (tx, rx) = mpsc::channel();
    let decoder: Arc<dyn Send + Sync> = Arc::new(DropProbe("decoder", tx.clone()));
    let state = Arc::new(State::new(DropProbe("job", tx), Some(decoder)));

    // submit(): hand the SDK an owned refcount.
    let sdk_ptr = Arc::into_raw(state.clone());
    assert_eq!(Arc::strong_count(&state), 2, "future + SDK both own the state");

    // The caller drops the future before the callback fires (e.g. a seek).
    drop(state);
    assert!(rx.try_recv().is_err(), "nothing released while the SDK holds the state");

    // The SDK's reference still keeps the state alive — reconstruct it the
    // way the completion callback does and confirm it is fully intact.
    let reclaimed = unsafe { Arc::from_raw(sdk_ptr) };
    assert_eq!(Arc::strong_count(&reclaimed), 1, "SDK holds the last reference");
    assert!(reclaimed.job.lock().unwrap().is_some(), "job survived the future drop");

    drop(reclaimed);
    let order: Vec<_> = rx.try_iter().map(|(name, _)| name).collect();
    assert_eq!(order, ["job", "decoder"]);
}

/// `abort()` must reach the in-flight job, and must be a safe no-op once the
/// result has been observed (the job slot is empty).
#[test]
fn abort_reaches_job_and_is_safe_after_drain() {
    let job = AsyncDecompressJob::new(sdk());
    let fut = CallbackFuture { state: Arc::new(State::new(job, None)) };

    // In-flight: abort() locks the slot and sets AbortDecode on the job.
    fut.abort();
    assert!(fut.state.job.lock().unwrap().is_some());

    // Simulate `poll` having handed the job back to the caller.
    let _drained = fut.state.job.lock().unwrap().take();

    // Aborting an already-drained future must not panic.
    fut.abort();
}
