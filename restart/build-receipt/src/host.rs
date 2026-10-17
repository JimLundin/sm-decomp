//! Single-use DLL launch boundary for receipt controls and integrity reading.
use std::ffi::{CStr, c_char};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static USED: AtomicBool = AtomicBool::new(false);

/// Run controls DIRECTORY, authenticate RECEIPT or build REQUEST once.
/// Returns 0 on success, 1 on refusal/error (including a null argument),
/// 2 on invalid array extent/pointer, reentry or panic.
///
/// # Safety
/// `argv` points to exactly `argc` readable pointers. Each nonnull pointer points
/// to a null-terminated UTF-8 string readable for this synchronous call. Strings
/// and array must remain immutable; no other thread may modify selected files.
/// The caller retains stdout/stderr and DLL/launcher identities and supervises
/// the entire process tree, including Cargo children. Paths must be absolute.
/// Build callers hold the integration sole-writer lease over declared inputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_restart_run(argc: usize, argv: *const *const c_char) -> i32 {
    if USED.swap(true, Ordering::SeqCst) || argc != 2 || argv.is_null() {
        return 2;
    }
    let result = std::panic::catch_unwind(|| -> super::Result<()> {
        // SAFETY: the caller guarantees exactly argc readable immutable pointers.
        let pointers = unsafe { std::slice::from_raw_parts(argv, argc) };
        let args = pointers
            .iter()
            .map(|&pointer| {
                if pointer.is_null() {
                    return Err("null argument".to_owned());
                }
                // SAFETY: each nonnull pointer is readable and terminated by contract.
                let value = unsafe { CStr::from_ptr(pointer) }.to_str().map_err(|e| e.to_string())?;
                if value.len() > 32768 {
                    return Err("argument length budget exceeded".into());
                }
                Ok(value)
            })
            .collect::<super::Result<Vec<_>>>()?;
        let path = Path::new(args.get(1).ok_or("path missing")?);
        super::absolute(path)?;
        match args.first().copied() {
            Some("controls") => {
                let count = super::focused_controls(path)?;
                println!("build-receipt focused controls: {count} passed");
                Ok(())
            }
            Some("authenticate") => {
                let receipt = super::authenticate(path)?;
                println!("build-receipt authenticated DLL: {}", receipt.dll_sha256);
                Ok(())
            }
            Some("build") => {
                let request = super::read_request(path)?;
                let receipt = super::build(&request)?;
                println!("build receipt: {} DLL: {}", receipt.receipt.display(), receipt.dll_sha256);
                Ok(())
            }
            _ => Err("expected controls DIRECTORY, authenticate RECEIPT or build REQUEST".into()),
        }
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
