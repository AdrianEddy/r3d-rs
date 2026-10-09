// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::{ RedError, RedResult, enums::*, custom_io::{ InstalledIo, IoInterface } };
use cpp::*;
use std::sync::{ Arc, Condvar, Mutex, MutexGuard, PoisonError, Weak };

cpp! {{
    #include "R3DSDK.h"
}}

/// The loaded R3D SDK.
///
/// Load it with [`Sdk::initialize`] before using any other API of this crate,
/// which takes an `&Sdk` to construct anything that needs it. `Sdk` is a cheap
/// handle: every object using the SDK keeps a clone, so the SDK is unloaded
/// only after the last `Sdk` and the last such object are dropped.
///
/// The SDK is process-wide, so only one can be loaded at a time; use
/// [`Sdk::current`] to share it. Once unloaded it can be initialized again.
#[derive(Clone)]
pub struct Sdk {
    inner: Arc<SdkInner>,
}

struct SdkInner {
    flags: InitializeFlags,
    io: Option<InstalledIo>,
}

/// Process-wide SDK state. `loaded` spans `InitializeSdk` to the return of
/// `FinalizeSdk`, so it stays set while the last `Sdk` is being unloaded, after
/// `live` can no longer be upgraded.
struct State {
    loaded: bool,
    live: Weak<SdkInner>,
}

static STATE: Mutex<State> = Mutex::new(State { loaded: false, live: Weak::new() });
/// Notified when the SDK has been unloaded.
static UNLOADED: Condvar = Condvar::new();

fn lock_state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(PoisonError::into_inner)
}

impl std::fmt::Debug for Sdk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sdk").field("flags", &self.inner.flags).finish_non_exhaustive()
    }
}

impl Sdk {
    /// Load the SDK. `path` must point to the folder with the SDK's dynamic
    /// libraries, it cannot be an empty string. `flags` selects the optional
    /// components to load.
    ///
    /// Fails with [`RedError::AlreadyInitialized`] if the SDK is loaded already.
    pub fn initialize(path: &str, flags: InitializeFlags) -> RedResult<Self> {
        Self::init(path, flags, None)
    }

    /// Like [`initialize`](Sdk::initialize), and route all file I/O of the SDK
    /// through `io` until it is unloaded. The SDK requires its I/O interface to
    /// be in place before any clip is opened, so it is set here.
    pub fn initialize_with_io(path: &str, flags: InitializeFlags, io: Arc<dyn IoInterface>) -> RedResult<Self> {
        Self::init(path, flags, Some(io))
    }

    fn init(path: &str, flags: InitializeFlags, io: Option<Arc<dyn IoInterface>>) -> RedResult<Self> {
        let c_path = std::ffi::CString::new(path).map_err(|_| RedError::InvalidPath)?;
        let mut state = lock_state();
        while state.loaded {
            if state.live.strong_count() > 0 {
                return Err(RedError::AlreadyInitialized);
            }
            // The last `Sdk` was just dropped; wait until it is unloaded.
            state = UNLOADED.wait(state).unwrap_or_else(PoisonError::into_inner);
        }

        let c_ptr = c_path.as_ptr();
        let bits = flags.bits();
        let status: i32 = cpp!(unsafe [c_ptr as "const char *", bits as "unsigned int"] -> i32 as "int" {
            return (int)R3DSDK::InitializeSdk(c_ptr, bits);
        });
        if let Err(e) = crate::ffi::check_status::<InitializeStatus>(status) {
            // Unload whatever was loaded, as RED's samples do.
            cpp!(unsafe [] { R3DSDK::FinalizeSdk(); });
            return Err(e);
        }

        let inner = Arc::new(SdkInner { flags, io: io.map(InstalledIo::install) });
        state.loaded = true;
        state.live = Arc::downgrade(&inner);
        Ok(Self { inner })
    }

    /// The loaded SDK, if any.
    pub fn current() -> Option<Self> {
        lock_state().live.upgrade().map(|inner| Self { inner })
    }

    /// The optional components the SDK was initialized with.
    pub fn flags(&self) -> InitializeFlags {
        self.inner.flags
    }

    /// Fail with [`RedError::NotInitializedWith`] unless the SDK was
    /// initialized with `component`. Calls into a component that was not
    /// loaded crash, so constructors that need one check first.
    pub(crate) fn require(&self, component: InitializeFlags) -> RedResult<()> {
        if self.inner.flags.contains(component) {
            Ok(())
        } else {
            Err(RedError::NotInitializedWith(component))
        }
    }

    /// Returns version information for the static/API version as well as any dynamic
    /// libraries successfully loaded. Can be called before [`Sdk::initialize`],
    /// and returns useful information even if initialization failed. If possible log the
    /// output of this function to a log file to help with customer issue debugging.
    pub fn version() -> String {
        let ver: *const core::ffi::c_char = cpp!(unsafe [] -> *const core::ffi::c_char as "const char *" {
            return R3DSDK::GetSdkVersion();
        });
        crate::ffi::string_from_c(ver).unwrap_or_default()
    }
}

impl Drop for SdkInner {
    fn drop(&mut self) {
        // Every object using the SDK holds an `Sdk`, so none is left, and in
        // particular no clip is open, as `ResetIoInterface` requires. A
        // concurrent `initialize` waits for `loaded` to clear.
        let io = self.io.take().map(InstalledIo::uninstall);
        cpp!(unsafe [] { R3DSDK::FinalizeSdk(); });
        lock_state().loaded = false;
        UNLOADED.notify_all();
        // The interface is dropped last, so its own `Drop` may use `Sdk`.
        drop(io);
    }
}
