//! Windows headless host, built directly with rustc and loaded by Python.
//! Each process runs one replay with explicitly owned Game state.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use sm::game_state::Game;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::sync::atomic::{AtomicBool, Ordering};

mod verify;

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

unsafe fn initialize_verify_audio(game: &mut Game) -> Result<(), String> {
    game.g_spc_player = sm::spc_player::SpcPlayer_Create();
    if game.g_spc_player.is_null() {
        return Err("unable to initialize sound driver".into());
    }
    sm::spc_player::SpcPlayer_Initialize(game.g_spc_player);
    Ok(())
}

unsafe fn finish_verify_audio() {}

// No audio thread exists in this host; the shared replay loop is synchronous.
unsafe fn host_apu_lock() {}
unsafe fn host_apu_unlock() {}

unsafe fn host_die(error: *const c_char) {
    eprintln!("Error: {}", CStr::from_ptr(error).to_string_lossy());
    std::process::exit(2);
}

unsafe fn host_warning(error: *const c_char) {
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
    let mut owned_game = sm::game_state::Game::new();
    let game = &mut *owned_game;

    if STARTED.swap(true, Ordering::SeqCst) {
        eprintln!("verify: use a separate Python process for each replay");
        return 2;
    }
    if rom.is_null() || inputs.is_null() {
        eprintln!("verify: missing ROM or input path");
        return 2;
    }
    sm::host::install_host_hooks(sm::host::HostHooks {
        die: host_die,
        warning: host_warning,
        apu_lock: host_apu_lock,
        apu_unlock: host_apu_unlock,
    });
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
    verify::run_args(&args, game)
}
