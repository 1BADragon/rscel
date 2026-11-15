use std::cell::RefCell;

use chrono::{DateTime, Utc};

use crate::interp::Interpreter;

/// Per-execution state that built-in functions can read from anywhere in a run.
///
/// It lives in a thread local for the duration of a run. Passing a previously captured
/// runtime to `CelContext::exec_with_rt` replays a run with the same fixed time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpRuntime {
    start_time: DateTime<Utc>,
}

/// Restores the thread's runtime on drop.
///
/// `restore` is `None` when the guard didn't change anything (a nested run reusing the
/// outer runtime); otherwise it holds the value to put back, which is `Some(None)` for
/// an outermost run.
pub struct InterpRuntimeGuard {
    restore: Option<Option<InterpRuntime>>,
}

thread_local! {
    static INTERP_RUNTIME: RefCell<Option<InterpRuntime>> = const { RefCell::new(None) };
}

/// Installs an explicitly provided runtime, even inside a nested run. The previous
/// runtime, if any, is restored when the guard drops.
pub fn start_runtime_prebuilt(_interp: &Interpreter, runtime: InterpRuntime) -> InterpRuntimeGuard {
    let prev = INTERP_RUNTIME.replace(Some(runtime));

    InterpRuntimeGuard {
        restore: Some(prev),
    }
}

/// Starts a fresh runtime for an outermost run. A nested run with no explicit runtime
/// reuses the outer one so `now()` stays consistent across the whole execution.
pub fn start_runtime(interp: &Interpreter) -> InterpRuntimeGuard {
    if INTERP_RUNTIME.with(|rt| rt.borrow().is_some()) {
        return InterpRuntimeGuard { restore: None };
    }

    start_runtime_prebuilt(interp, InterpRuntime::from_interp(interp))
}

impl InterpRuntime {
    pub fn new(start_time: DateTime<Utc>) -> Self {
        InterpRuntime { start_time }
    }

    /// The time `now()` returns during a run using this runtime.
    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    fn from_interp(_interp: &Interpreter) -> Self {
        InterpRuntime::default()
    }
}

impl Default for InterpRuntime {
    /// A runtime starting at the current time.
    fn default() -> Self {
        InterpRuntime::new(Utc::now())
    }
}

impl Drop for InterpRuntimeGuard {
    fn drop(&mut self) {
        if let Some(prev) = self.restore.take() {
            INTERP_RUNTIME.set(prev);
        }
    }
}

/// Start time of the current execution, or the current time if no
/// runtime is active on this thread.
pub fn runtime_start_time() -> DateTime<Utc> {
    INTERP_RUNTIME
        .with(|rt| rt.borrow().as_ref().map(|r| r.start_time))
        .unwrap_or_else(Utc::now)
}

/// The active runtime. Only call while an `InterpRuntimeGuard` is held.
pub(crate) fn runtime_copy() -> InterpRuntime {
    INTERP_RUNTIME.with(|rt| {
        rt.borrow()
            .clone()
            .expect("runtime_copy called with no active runtime")
    })
}
