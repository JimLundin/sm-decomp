//! Windows headless host, built by Cargo or rustc and loaded by Python.
//! Each process runs one replay with explicitly owned Game state.
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg(target_os = "windows")]
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use sm::game_state::Game;
use std::ffi::{CStr, c_char, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

mod verify;

#[unsafe(no_mangle)]
pub extern "C" fn sm_verify_room_level_copy(case: u32) -> bool {
    std::panic::catch_unwind(|| sm::verify_room_level_copy(case)).is_ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn sm_verify_player_projectiles() -> u32 {
    std::panic::catch_unwind(sm::verify_player_projectiles).unwrap_or(0)
}

/// Copy a synthetic projectile scenario's RAM for independent byte comparison.
///
/// # Safety
/// A non-null output must be writable for at least `capacity` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_player_projectile_ram(
    id: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    if output.is_null() {
        return 0;
    }
    match std::panic::catch_unwind(|| sm::verify_player_projectile_case(id)) {
        Ok(Some(bytes)) if bytes.len() <= capacity => {
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len()) };
            bytes.len()
        }
        _ => 0,
    }
}

/// Copy a synthetic mechanism scenario's RAM and return observation.
///
/// # Safety
/// A non-null output must be writable for at least `capacity` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_room_mechanism_case(
    id: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    if output.is_null() {
        return 0;
    }
    match std::panic::catch_unwind(|| sm::verify_room_mechanism_case(id)) {
        Ok(Some(bytes)) if bytes.len() <= capacity => {
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len()) };
            bytes.len()
        }
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_backend_source_hash() -> *const c_char {
    unsafe {
        unsafe extern "C" {
            fn sm_backend_source_hash() -> *const c_char;
        }
        sm_backend_source_hash()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_music_source_hash() -> *const c_char {
    unsafe {
        unsafe extern "C" {
            fn sm_music_source_hash() -> *const c_char;
        }
        sm_music_source_hash()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sm_verify_util_source_matches() -> bool {
    sm::util::native_source_matches()
}

static mut g_pixels: [u8; 245760] = [0; 245760];
static mut g_my_pixels: [u8; 245760] = [0; 245760];
static STARTED: AtomicBool = AtomicBool::new(false);

/// The launcher stamps the source contents when it builds this host.
/// Cargo tracks this environment value without rebuilding the game library.
#[unsafe(no_mangle)]
pub extern "C" fn sm_verify_source_hash() -> *const u8 {
    match option_env!("SM_VERIFY_SOURCE_HASH") {
        Some(hash) if hash.len() == 64 => hash.as_ptr(),
        _ => std::ptr::null(),
    }
}

#[repr(C)]
pub struct SourceFile {
    path: *const u8,
    path_len: usize,
    data: *const u8,
    data_len: usize,
}

/// These bytes come from sm's rlib, independently of the host's source stamp.
#[unsafe(no_mangle)]
pub extern "C" fn sm_verify_source_file(index: usize) -> SourceFile {
    let (path, data) = sm::verify_source::FILES
        .get(index)
        .copied()
        .unwrap_or(("", &[]));
    SourceFile {
        path: path.as_ptr(),
        path_len: path.len(),
        data: data.as_ptr(),
        data_len: data.len(),
    }
}

// Transpiled code expects glibc's stderr global; MinGW exposes its FILE via
// __acrt_iob_func instead. FILE remains opaque to this host and the game code.
#[unsafe(no_mangle)]
pub static mut stderr: *mut c_void = std::ptr::null_mut();
unsafe extern "C" {
    fn __acrt_iob_func(index: u32) -> *mut c_void;
}

unsafe fn initialize_verify_audio(game: &mut Game) -> Result<(), String> {
    unsafe {
        game.g_spc_player = sm::spc_player::SpcPlayer_Create();
        if game.g_spc_player.is_null() {
            return Err("unable to initialize sound driver".into());
        }
        sm::spc_player::SpcPlayer_Initialize(game.g_spc_player);
        Ok(())
    }
}

unsafe fn finish_verify_audio() {}

// No audio thread exists in this host; the shared replay loop is synchronous.
unsafe fn host_apu_lock() {}
unsafe fn host_apu_unlock() {}

unsafe fn host_die(error: *const c_char) {
    unsafe {
        eprintln!("Error: {}", CStr::from_ptr(error).to_string_lossy());
        std::process::exit(2);
    }
}

unsafe fn host_warning(error: *const c_char) {
    unsafe {
        eprintln!("Warning: {}", CStr::from_ptr(error).to_string_lossy());
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __assert_fail(
    assertion: *const c_char,
    file: *const c_char,
    line: u32,
    function: *const c_char,
) -> ! {
    unsafe {
        eprintln!(
            "Assertion failed: {} at {}:{line} ({})",
            CStr::from_ptr(assertion).to_string_lossy(),
            CStr::from_ptr(file).to_string_lossy(),
            CStr::from_ptr(function).to_string_lossy()
        );
        std::process::exit(2);
    }
}

/// Called by ctypes with UTF-8 paths; zero frames means the entire movie.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify(
    rom: *const c_char,
    inputs: *const c_char,
    frames: u64,
) -> c_int {
    unsafe {
        if rom.is_null() || inputs.is_null() {
            eprintln!("verify: missing ROM or input path");
            return 2;
        }
        let mut args = Vec::new();
        for path in [rom, inputs] {
            if let Ok(path) = CStr::from_ptr(path).to_str() {
                args.push(path.to_owned())
            } else {
                eprintln!("verify: path is not valid UTF-8");
                return 2;
            }
        }
        if frames != 0 {
            args.extend(["--frames".to_owned(), frames.to_string()]);
        }
        run(&args)
    }
}

/// The normal CLI argument path, including verifier checkpoint options.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_args(argc: u32, argv: *const *const c_char) -> c_int {
    unsafe {
        if argv.is_null() || argc > 1024 {
            eprintln!("verify: invalid arguments");
            return 2;
        }
        let mut args = Vec::new();
        for &arg in std::slice::from_raw_parts(argv, argc as usize) {
            if arg.is_null() {
                eprintln!("verify: null argument");
                return 2;
            }
            if let Ok(arg) = CStr::from_ptr(arg).to_str() {
                args.push(arg.to_owned())
            } else {
                eprintln!("verify: argument is not valid UTF-8");
                return 2;
            }
        }
        run(&args)
    }
}

unsafe fn run(args: &[String]) -> c_int {
    unsafe {
        if STARTED.swap(true, Ordering::SeqCst) {
            eprintln!("verify: use a separate Python process for each replay");
            return 2;
        }
        let mut game = Game::new();
        sm::host::install_host_hooks(sm::host::HostHooks {
            die: host_die,
            warning: host_warning,
            apu_lock: host_apu_lock,
            apu_unlock: host_apu_unlock,
        });
        stderr = __acrt_iob_func(2);
        verify::run_args(args, &mut game)
    }
}
