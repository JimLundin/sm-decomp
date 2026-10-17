//! Windows headless host, built directly with rustc and loaded by Python.
//! Each process runs exactly one replay: the game owns global mutable state.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use std::ffi::{c_char, c_int, c_void, CStr};
use std::sync::atomic::{AtomicBool, Ordering};

mod verify;

#[no_mangle]
pub static mut g_spc_player: *mut sm::spc_player::SpcPlayer = std::ptr::null_mut();
#[no_mangle]
pub static mut g_got_mismatch_count: c_int = 0;
#[no_mangle]
pub static mut g_other_image: bool = false;
#[no_mangle]
pub static mut g_debug_flag: bool = false;
#[no_mangle]
pub static mut g_new_ppu: bool = true;
static mut g_pixels: [u8; 245760] = [0; 245760];
static mut g_my_pixels: [u8; 245760] = [0; 245760];
static STARTED: AtomicBool = AtomicBool::new(false);

// Transpiled code expects glibc's stderr global; MinGW exposes its FILE via
// __acrt_iob_func instead. FILE remains opaque to this host and the game code.
#[no_mangle]
pub static mut stderr: *mut c_void = std::ptr::null_mut();
extern "C" {
    fn __acrt_iob_func(index: u32) -> *mut c_void;
}

unsafe fn initialize_verify_audio() -> Result<(), String> {
    g_spc_player = sm::spc_player::SpcPlayer_Create();
    if g_spc_player.is_null() {
        return Err("unable to initialize sound driver".into());
    }
    sm::spc_player::SpcPlayer_Initialize(g_spc_player);
    Ok(())
}

unsafe fn finish_verify_audio() {}

// No audio thread exists in this host; the shared replay loop is synchronous.
#[no_mangle]
pub extern "C" fn RtlApuLock() {}
#[no_mangle]
pub extern "C" fn RtlApuUnlock() {}

#[no_mangle]
pub unsafe extern "C" fn Die(error: *const c_char) {
    eprintln!("Error: {}", CStr::from_ptr(error).to_string_lossy());
    std::process::exit(2);
}

#[no_mangle]
pub unsafe extern "C" fn Warning(error: *const c_char) {
    eprintln!("Warning: {}", CStr::from_ptr(error).to_string_lossy());
}

#[no_mangle]
pub unsafe extern "C" fn __assert_fail(
    assertion: *const c_char,
    file: *const c_char,
    line: u32,
    function: *const c_char,
) -> ! {
    eprintln!(
        "Assertion failed: {} at {}:{line} ({})",
        CStr::from_ptr(assertion).to_string_lossy(),
        CStr::from_ptr(file).to_string_lossy(),
        CStr::from_ptr(function).to_string_lossy()
    );
    std::process::exit(2);
}

/// Called by ctypes with UTF-8 paths; zero frames means the entire movie.
#[no_mangle]
pub unsafe extern "C" fn sm_verify(
    rom: *const c_char,
    inputs: *const c_char,
    frames: u64,
) -> c_int {
    if STARTED.swap(true, Ordering::SeqCst) {
        eprintln!("verify: use a separate Python process for each replay");
        return 2;
    }
    if rom.is_null() || inputs.is_null() {
        eprintln!("verify: missing ROM or input path");
        return 2;
    }
    stderr = __acrt_iob_func(2);
    let mut args = Vec::new();
    for path in [rom, inputs] {
        match CStr::from_ptr(path).to_str() {
            Ok(path) => args.push(path.to_owned()),
            Err(_) => {
                eprintln!("verify: path is not valid UTF-8");
                return 2;
            }
        }
    }
    if frames != 0 {
        args.extend(["--frames".to_owned(), frames.to_string()]);
    }
    verify::run_args(&args)
}
