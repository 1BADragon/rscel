use std::cell::RefCell;

use chrono::{DateTime, Utc};

use crate::interp::Interpreter;

// Thread local global context to allow the specific runtime information of a run to
// be pulled from any point in the execution. This will also allow a user to "reply"
// an old run with the fixed time.
#[derive(Debug, Clone)]
pub struct InterpRuntime {
    start_time: DateTime<Utc>,
}

/// Clears the thread's runtime on drop, but only if this guard created it.
/// Nested runs on the same thread reuse the outer runtime.
pub struct InterpRuntimeGuard {
    owns: bool,
}

thread_local! {
    static INTERP_RUNTIME: RefCell<Option<InterpRuntime>> = const { RefCell::new(None) };
}

pub fn start_runtime_prebuilt(_interp: &Interpreter, runtime: InterpRuntime) -> InterpRuntimeGuard {
    INTERP_RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        if rt.is_some() {
            return InterpRuntimeGuard { owns: false };
        }
        *rt = Some(runtime);
        InterpRuntimeGuard { owns: true }
    })
}

pub fn start_runtime(interp: &Interpreter) -> InterpRuntimeGuard {
    let rt = InterpRuntime::from_interp(interp);

    start_runtime_prebuilt(interp, rt)
}

impl InterpRuntime {
    pub fn new(start_time: chrono::DateTime<Utc>) -> Self {
        InterpRuntime { start_time }
    }

    fn from_interp(_interp: &Interpreter) -> Self {
        InterpRuntime {
            start_time: chrono::Utc::now(),
        }
    }
}

impl Drop for InterpRuntimeGuard {
    fn drop(&mut self) {
        if self.owns {
            INTERP_RUNTIME.set(None);
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

pub fn runtime_copy() -> InterpRuntime {
    INTERP_RUNTIME.with(|rt| rt.borrow().as_ref().unwrap().clone())
}
