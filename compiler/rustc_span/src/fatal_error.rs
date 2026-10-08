/// Used as a return value to signify a fatal error occurred.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct FatalError;

use std::panic;

pub use rustc_data_structures::FatalErrorMarker;

// Don't implement Send on FatalError. This makes it impossible to `panic_any!(FatalError)`.
// We don't want to invoke the panic handler and print a backtrace for fatal errors.
impl !Send for FatalError {}

#[cfg(all(target_family = "wasm", panic = "abort"))]
thread_local! {
    static EXIT_HOOK: std::cell::Cell<Option<&'static dyn Fn()>> = const { std::cell::Cell::new(None) };
}

/// Runs `f`; if a fatal error ends the process without unwinding (wasm-hosted rustc), `hook`
/// runs first, standing in for the cleanup the caller would have done after catching the unwind.
pub fn with_fatal_exit_hook<R>(hook: &dyn Fn(), f: impl FnOnce() -> R) -> R {
    #[cfg(all(target_family = "wasm", panic = "abort"))]
    {
        // SAFETY: the hook is unregistered before `hook` goes out of scope; without unwinding,
        // `f` either returns here or the process exits inside `raise`.
        let hook: &'static dyn Fn() = unsafe { std::mem::transmute(hook) };
        let prev = EXIT_HOOK.replace(Some(hook));
        let r = f();
        EXIT_HOOK.set(prev);
        r
    }
    #[cfg(not(all(target_family = "wasm", panic = "abort")))]
    {
        let _ = hook;
        f()
    }
}

impl FatalError {
    pub fn raise(self) -> ! {
        // Without unwinding (wasm-hosted rustc) the marker would abort with a trap; the
        // diagnostics are already emitted, so exit the way `catch_with_exit_code` would.
        #[cfg(all(target_family = "wasm", panic = "abort"))]
        {
            if let Some(hook) = EXIT_HOOK.take() {
                hook();
            }
            let _ = std::io::Write::flush(&mut std::io::stderr());
            std::process::exit(1);
        }
        #[allow(unreachable_code)]
        std::panic::resume_unwind(Box::new(FatalErrorMarker))
    }
}

impl std::fmt::Display for FatalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fatal error")
    }
}

impl std::error::Error for FatalError {}

/// Runs a closure and catches unwinds triggered by fatal errors.
///
/// The compiler currently unwinds with a special sentinel value to abort
/// compilation on fatal errors. This function catches that sentinel and turns
/// the panic into a `Result` instead.
pub fn catch_fatal_errors<F: FnOnce() -> R, R>(f: F) -> Result<R, FatalError> {
    panic::catch_unwind(panic::AssertUnwindSafe(f)).map_err(|value| {
        if value.is::<FatalErrorMarker>() {
            FatalError
        } else {
            panic::resume_unwind(value);
        }
    })
}
