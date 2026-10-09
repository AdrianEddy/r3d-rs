// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use std::{ future::Future, pin::Pin, sync::{ atomic::{ AtomicBool, Ordering}, mpsc, Arc, Mutex, OnceLock }, task::{ Context, Poll }};
use futures_util::task::AtomicWaker;
use super::*;

/// Shared state between a [`CallbackFuture`] and the SDK completion
/// callback.
///
/// # Lifetime / co-ownership
///
/// The SDK is handed an **owned** reference to this state via
/// `Arc::into_raw` at submit time (see `AsyncDecoder::decode_for_gpu_sdk`
/// / `R3dDecoder::decode`); the callback reclaims it with `Arc::from_raw`.
/// So the `State` (and the `job` it holds, including any output buffer the
/// job owns) stays alive until **both** the future and the pending
/// callback have released their reference. Dropping the future before the
/// callback fires is therefore memory-safe — the SDK still owns a strong
/// reference, so the late callback never writes into freed memory.
///
/// The state also keeps the decoder the job was submitted to alive, so the
/// decoder is never closed while it still has a job in flight.
///
/// When the future is dropped first, the callback holds the last reference.
/// It must not drop it in place: that would destroy the C++ job (`delete` /
/// `R3DDecoder::ReleaseDecodeJob`) from inside the SDK's own callback for that
/// job, on the SDK worker thread. [`release_from_callback`] hands that final
/// drop to another thread instead, which is how RED's samples free jobs.
///
/// `job` lives behind a `Mutex` (not a bare `Option<T>`) precisely because
/// the callback accesses the state through a reclaimed `Arc` (a shared
/// reference): mutating `job`/`status` needs interior mutability, and the
/// `Mutex` also lets `abort()` safely poke the in-flight job from the
/// caller thread while the callback may be running on the SDK thread.
pub(crate) struct State<T> {
    pub(crate) waker: AtomicWaker,
    pub(crate) done: AtomicBool,
    /// The submitted job, kept at a stable address until the future is
    /// polled to completion (or the state is dropped). The callback does
    /// **not** take it — only [`CallbackFuture::poll`] does, so `abort()`
    /// can reach it for as long as the result is unobserved.
    pub(crate) job: Mutex<Option<T>>,
    /// Completion status recorded by the callback; `None` until it fires.
    pub(crate) status: Mutex<Option<Result<(), RedError>>>,
    /// The decoder the job was submitted to. Declared after `job`, so the job
    /// is released before the decoder.
    _decoder: Option<Arc<dyn Send + Sync>>,
}
impl<T> State<T> {
    pub(crate) fn new(job: T, decoder: Option<Arc<dyn Send + Sync>>) -> Self {
        Self {
            waker: AtomicWaker::new(),
            done: AtomicBool::new(false),
            job: Mutex::new(Some(job)),
            status: Mutex::new(None),
            _decoder: decoder,
        }
    }
}
/// Resolves when the SDK completes an asynchronous decode, giving back the
/// submitted job on success.
pub struct CallbackFuture<T> {
    pub(crate) state: Arc<State<T>>,
}
impl<T> Future for CallbackFuture<T> {
    type Output = Result<T, RedError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.state.waker.register(cx.waker());
        if this.state.done.load(Ordering::Acquire) {
            // Take the status first and release its lock before touching the
            // job lock, so the two are never held nested (abort() takes only
            // the job lock, the callback takes only the status lock).
            let status = this.state.status.lock().unwrap().take();
            if let Some(status) = status {
                // The callback has recorded the status; hand back ownership of
                // the job. On error the job is dropped here: ownership is only
                // returned on success.
                let job = this.state.job.lock().unwrap().take()
                    .expect("R3D completion callback fired without a job in the slot");
                return Poll::Ready(status.map(|()| job));
            }
        }
        Poll::Pending
    }
}

/// Release the reference the SDK held on `state`, from inside the completion
/// callback for that job.
///
/// If the future is still alive this only decrements the count. If the
/// future was already dropped, this is the last reference: the `State` (and
/// the C++ job it owns) is then destroyed on the reaper thread rather than
/// inside the SDK's callback. `Arc::into_inner` makes the "am I last?" check
/// atomic with the decrement, so a future dropped concurrently cannot slip
/// the final drop back onto this thread.
pub(crate) fn release_from_callback<T: Send + 'static>(state: Arc<State<T>>) {
    if let Some(orphan) = Arc::into_inner(state) {
        drop_off_thread(Box::new(orphan));
    }
}

type Orphan = Box<dyn Send>;

/// Sender side of the reaper thread, spawned on first use. `None` if the
/// thread could not be spawned.
static REAPER: OnceLock<Option<mpsc::Sender<Orphan>>> = OnceLock::new();

fn drop_off_thread(orphan: Orphan) {
    let reaper = REAPER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Orphan>();
        std::thread::Builder::new()
            .name("r3d-job-reaper".into())
            .spawn(move || rx.into_iter().for_each(drop))
            .map_err(|e| log::error!("Failed to spawn the R3D job reaper thread: {e}"))
            .ok()
            .map(|_| tx)
    });
    let orphan = match reaper {
        Some(tx) => match tx.send(orphan) {
            Ok(()) => return,
            Err(mpsc::SendError(orphan)) => orphan,
        },
        None => orphan,
    };
    // Without the reaper, leaking is the only option that never frees the job
    // inside its own callback.
    log::error!("R3D job reaper unavailable; leaking a completed decode job");
    std::mem::forget(orphan);
}

/// Block until every job handed to the reaper so far has been destroyed.
/// No-op if the reaper never started.
#[cfg(test)]
pub(crate) fn flush_deferred_drops() {
    let Some(Some(tx)) = REAPER.get() else { return };
    struct Marker(mpsc::Sender<()>);
    impl Drop for Marker {
        fn drop(&mut self) { let _ = self.0.send(()); }
    }
    let (done_tx, done_rx) = mpsc::channel();
    // The channel is FIFO, so the marker is dropped after every earlier orphan.
    if tx.send(Box::new(Marker(done_tx))).is_ok() {
        let _ = done_rx.recv();
    }
}

#[cfg(test)]
mod tests;
