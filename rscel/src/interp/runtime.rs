use std::cell::{Cell, RefCell};

use chrono::{DateTime, Utc};

use crate::interp::Interpreter;

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

pub fn start_runtime(_interp: &Interpreter) -> InterpRuntimeGuard {
    INTERP_RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        if rt.is_some() {
            return InterpRuntimeGuard { owns: false };
        }
        *rt = Some(InterpRuntime {
            start_time: Utc::now(),
        });
        InterpRuntimeGuard { owns: true }
    })
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

thread_local! {
    static CONST_EVAL: Cell<bool> = const { Cell::new(false) };
}

/// Marks the current thread as doing compile-time constant folding. Impure
/// functions (e.g. `now()`) check `in_const_eval()` and refuse to fold.
pub struct ConstEvalGuard {
    prev: bool,
}

pub fn start_const_eval() -> ConstEvalGuard {
    ConstEvalGuard {
        prev: CONST_EVAL.replace(true),
    }
}

impl Drop for ConstEvalGuard {
    fn drop(&mut self) {
        CONST_EVAL.set(self.prev);
    }
}

pub fn in_const_eval() -> bool {
    CONST_EVAL.get()
}
