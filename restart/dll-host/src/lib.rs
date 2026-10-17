//! Thin external launch boundary for the production Rust runtime worker.
use std::ffi::{CStr, OsString, c_char};
use std::sync::atomic::{AtomicBool, Ordering};

static USED: AtomicBool = AtomicBool::new(false);

/// Execute `prepare` or `run-worker` once in a dedicated process, or the
/// feature-gated fixed private assertion suites without creating a Game.
/// Returns 0 on success, 1 on argument/runtime refusal, 2 on reentry/panic/invalid array.
/// Native fatal exits can terminate the process without returning.
///
/// # Safety
/// `argv` must point to `argc` readable pointers (1..=64), each pointing to a
/// readable null-terminated UTF-8 string for this entire synchronous call.
/// All paths must be absolute. No other thread may use native runtime state or
/// the working directory. The caller must retain stdout/stderr, enforce an outer
/// process-tree deadline and record actual DLL/launcher identities separately.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_restart_run(argc: usize, argv: *const *const c_char) -> i32 {
    if USED.swap(true, Ordering::SeqCst) || !(1..=64).contains(&argc) || argv.is_null() {
        return 2;
    }
    let result = std::panic::catch_unwind(|| {
        // SAFETY: the caller guarantees the array's size, lifetime and readability.
        let pointers = unsafe { std::slice::from_raw_parts(argv, argc) };
        let args = pointers
            .iter()
            .map(|&pointer| {
                if pointer.is_null() {
                    return Err("null argument".to_owned());
                }
                // SAFETY: the caller guarantees a readable terminated string.
                let argument = unsafe { CStr::from_ptr(pointer) };
                argument.to_str().map(OsString::from).map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        sm_restart::runtime_cli::run_external(args)
    });
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            eprintln!("{error}");
            1
        }
        Err(_) => 2,
    }
}
