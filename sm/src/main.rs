#![allow(dead_code, non_snake_case, non_upper_case_globals, unused_imports)]
use core::ffi::{c_char, c_long, c_void};
use core::ptr::{null, null_mut};
use sdl2_sys::SDL_MessageBoxFlags::{SDL_MESSAGEBOX_ERROR, SDL_MESSAGEBOX_WARNING};
use sdl2_sys::{
    SDL_CreateMutex, SDL_DestroyMutex, SDL_LockMutex, SDL_ShowSimpleMessageBox, SDL_UnlockMutex,
    SDL_Window, SDL_mutex, Uint32,
};
#[cfg(not(windows))]
use sm::c_library::{__assert_fail, stderr};
use sm::c_library::{exit, fprintf, memcpy};
use sm::game_state::{Game, GameOwner};
use sm::host::Die;
use sm::rtl::*;
use sm::spc_player::{SpcPlayer_Create, SpcPlayer_Initialize};
use sm::types::{size_t, RendererFuncs, Snes, FILE};
mod verify;
const kWindowTitle: &[u8] = b"SuperMet\0";
const NULL: *mut c_void = null_mut();
static mut g_pixels: [u8; 245760] = [0; 245760];
static mut g_my_pixels: [u8; 245760] = [0; 245760];

#[link(name = "sm_desktop", kind = "static", modifiers = "-bundle")]
extern "C-unwind" {
    fn sm_desktop_run(argc: i32, argv: *mut *mut c_char, owner: *const GameOwner) -> i32;
    fn sm_desktop_HandleInput(game: *mut Game, key: i32, modifiers: i32, pressed: bool);
    fn sm_desktop_HandleGamepadInput(game: *mut Game, button: i32, pressed: bool);
    fn sm_desktop_HandleGamepadAxisInput(game: *mut Game, id: i32, axis: i32, value: i32);
    fn sm_desktop_HandleCommand(game: *mut Game, command: u32, pressed: bool);
    fn sm_desktop_DrawPpuFrameWithPerf(game: *mut Game);
    fn sm_desktop_AudioCallback(game: *mut Game, stream: *mut u8, length: i32);
    #[link_name = "sm_desktop_audio_mutex"]
    static mut g_audio_mutex: *mut SDL_mutex;
    fn sm_desktop_matches_source(index: u32, bytes: *const u8, length: usize) -> bool;
}
fn check_dependency() {
    for (index, source) in [
        include_bytes!("main.rs").as_slice(),
        include_bytes!("native_desktop.c").as_slice(),
        include_bytes!("native_desktop.h").as_slice(),
        include_bytes!("types.rs").as_slice(),
        include_bytes!("platform/opengl.rs").as_slice(),
        include_bytes!("../../tools/install_native_desktop.py").as_slice(),
        include_bytes!("../../tools/install_native_config.py").as_slice(),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            unsafe { sm_desktop_matches_source(index as u32, source.as_ptr(), source.len()) },
            "stale native desktop; rerun tools/install_native_desktop.py and rebuild"
        );
    }
}

unsafe fn host_die(error: *const c_char) {
    SDL_ShowSimpleMessageBox(
        SDL_MESSAGEBOX_ERROR as i32 as Uint32,
        kWindowTitle.as_ptr().cast(),
        error,
        null_mut::<SDL_Window>(),
    );
    fprintf(stderr, c"Error: %s\n".as_ptr(), error);
    exit(1);
}

unsafe fn host_warning(error: *const c_char) {
    SDL_ShowSimpleMessageBox(
        SDL_MESSAGEBOX_WARNING as i32 as Uint32,
        kWindowTitle.as_ptr().cast(),
        error,
        null_mut::<SDL_Window>(),
    );
    fprintf(stderr, c"Warning: %s\n".as_ptr(), error);
}

unsafe fn initialize_verify_audio(game: &mut Game) -> Result<(), String> {
    g_audio_mutex = SDL_CreateMutex();
    if g_audio_mutex.is_null() {
        return Err("unable to create audio mutex".into());
    }
    game.g_spc_player = SpcPlayer_Create();
    if game.g_spc_player.is_null() {
        SDL_DestroyMutex(g_audio_mutex);
        return Err("unable to initialize sound driver".into());
    }
    SpcPlayer_Initialize(game.g_spc_player);
    Ok(())
}

sm::forward_calls! { unsafe fn ();
    finish_verify_audio => SDL_DestroyMutex(g_audio_mutex);
    host_apu_lock => SDL_LockMutex(g_audio_mutex);
    host_apu_unlock => SDL_UnlockMutex(g_audio_mutex);
}

pub unsafe fn RtlDrawPpuFrame(
    pixel_buffer: *mut u8,
    pitch: size_t,
    render_flags: u32,
    game: &mut Game,
) {
    let ppu_pixels = if game.g_other_image as i32 != 0 {
        &raw mut g_my_pixels as *mut u8
    } else {
        &raw mut g_pixels as *mut u8
    };
    for y in 0usize..240 {
        memcpy(
            (pixel_buffer).offset(y.wrapping_mul(pitch) as isize) as *mut c_void,
            ppu_pixels.offset(y.wrapping_mul(256 as size_t).wrapping_mul(4 as size_t) as isize)
                as *const c_void,
            (256 * 4) as size_t,
        );
    }
}

#[no_mangle]
unsafe extern "C-unwind" fn sm_host_die(message: *const c_char) {
    Die(message);
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_stderr() -> *mut FILE {
    stderr
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_parse_config(name: *const c_char) {
    sm::config::ParseConfigFile(name);
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_find_key(key: i32, modifiers: i32) -> i32 {
    sm::config::FindCmdForSdlKey(key, modifiers as _)
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_find_button(button: i32, modifiers: u32) -> i32 {
    sm::config::FindCmdForGamepadButton(button, modifiers)
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_debug(owner: *const GameOwner) {
    (*owner).with(|game| game.g_debug_flag = true);
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_start_game(
    owner: *const GameOwner,
    filename: *const c_char,
) -> *mut Snes {
    (*owner).with(|game| sm::cpu_infra::SnesInit(filename, game))
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_initialize_renderer(
    owner: *const GameOwner,
    funcs: *const RendererFuncs,
    window: *mut SDL_Window,
) -> bool {
    (*owner).with(|_game| (*funcs).Initialize.expect("non-null function pointer")(window))
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_destroy_renderer(
    owner: *const GameOwner,
    funcs: *const RendererFuncs,
) {
    (*owner).with(|_game| (*funcs).Destroy.expect("non-null function pointer")());
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_begin_draw(
    funcs: *const RendererFuncs,
    width: i32,
    height: i32,
    pixels: *mut *mut u8,
    pitch: *mut i32,
) {
    (*funcs).BeginDraw.expect("non-null function pointer")(width, height, pixels, pitch);
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_end_draw(funcs: *const RendererFuncs) {
    (*funcs).EndDraw.expect("non-null function pointer")();
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_create_player(owner: *const GameOwner) {
    (*owner).with(|game| game.g_spc_player = SpcPlayer_Create());
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_initialize_player(owner: *const GameOwner) {
    (*owner).with(|game| SpcPlayer_Initialize(game.g_spc_player));
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_prepare_pixels(snes: *mut Snes) {
    sm::snes::ppu::PpuBeginDrawing((*snes).snes_ppu, (&raw mut g_pixels).cast(), 1024, 0);
    sm::snes::ppu::PpuBeginDrawing((*snes).my_ppu, (&raw mut g_my_pixels).cast(), 1024, 0);
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_read_sram(owner: *const GameOwner) {
    (*owner).with(|game| RtlReadSram(game));
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_key(
    owner: *const GameOwner,
    key: i32,
    modifiers: i32,
    pressed: bool,
) {
    (*owner).with(|game| sm_desktop_HandleInput(game, key, modifiers, pressed));
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_button(owner: *const GameOwner, button: i32, pressed: bool) {
    (*owner).with(|game| sm_desktop_HandleGamepadInput(game, button, pressed));
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_axis(owner: *const GameOwner, id: i32, axis: i32, value: i32) {
    (*owner).with(|game| sm_desktop_HandleGamepadAxisInput(game, id, axis, value));
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_command(owner: *const GameOwner, command: u32, pressed: bool) {
    (*owner).with(|game| sm_desktop_HandleCommand(game, command, pressed));
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_run_frame(owner: *const GameOwner, inputs: i32) -> bool {
    (*owner).with(|game| RtlRunFrame(inputs, game))
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_present(
    owner: *const GameOwner,
    disable_render: *mut bool,
) -> bool {
    (*owner).with(|game| {
        (*game.g_snes).disableRender = *disable_render;
        if !*disable_render {
            sm_desktop_DrawPpuFrameWithPerf(game);
        }
        *disable_render = (*game.g_snes).disableRender;
        game.g_got_mismatch_count != 0
    })
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_render_scale(game: *mut Game, flags: u32) -> i32 {
    sm::snes::ppu::PpuGetCurrentRenderScale((*(*game).g_snes).ppu, flags)
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_copy_frame(
    game: *mut Game,
    pixels: *mut u8,
    pitch: usize,
    flags: u32,
) {
    RtlDrawPpuFrame(pixels, pitch, flags, &mut *game);
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_mismatch_count(game: *mut Game) -> i32 {
    (*game).g_got_mismatch_count
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_fps(value: f32) -> i32 {
    value as i32
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_save_load(game: *mut Game, operation: i32, slot: i32) {
    RtlSaveLoad(operation, slot, &mut *game);
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_cheat(game: *mut Game, code: c_char) {
    RtlCheat(code, &mut *game);
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_game_command(game: *mut Game, command: u32, flags: i32) {
    let game = &mut *game;
    match command {
        115 => game.g_other_image = !game.g_other_image,
        116 => RtlClearKeyLog(game),
        117 => RtlStopReplay(game),
        119 => RtlReset(1, game),
        127 => game.g_new_ppu = flags & 1 != 0,
        _ => unreachable!(),
    }
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_gamepad_bounds(button: i32) {
    let _ = [0u8; 17][button as usize];
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_bad_command() {
    __assert_fail(
        c"0".as_ptr(),
        c"src/main.c".as_ptr(),
        686,
        c"void HandleCommand(uint32, _Bool)".as_ptr(),
    );
}
#[no_mangle]
unsafe extern "C-unwind" fn sm_host_render_audio(
    game: *mut Game,
    buffer: *mut i16,
    frames: i32,
    channels: i32,
) {
    RtlRenderAudio(buffer, frames, channels, &mut *game);
}
#[no_mangle]
unsafe extern "C" fn sm_host_audio_callback(userdata: *mut c_void, stream: *mut u8, length: i32) {
    (*(userdata as *const GameOwner)).with(|game| sm_desktop_AudioCallback(game, stream, length));
}

// The transpiled runtime references glibc's stderr and assertion symbols.
// MinGW uses a stream accessor instead; FILE is passed opaquely to the CRT.
#[cfg(windows)]
#[no_mangle]
pub static mut stderr: *mut FILE = std::ptr::null_mut();

#[cfg(windows)]
extern "C" {
    fn __acrt_iob_func(index: u32) -> *mut FILE;
}

#[cfg(windows)]
#[no_mangle]
pub unsafe extern "C" fn __assert_fail(
    assertion: *const std::ffi::c_char,
    file: *const std::ffi::c_char,
    line: u32,
    function: *const std::ffi::c_char,
) -> ! {
    eprintln!(
        "Assertion failed: {} at {}:{line} ({})",
        std::ffi::CStr::from_ptr(assertion).to_string_lossy(),
        std::ffi::CStr::from_ptr(file).to_string_lossy(),
        std::ffi::CStr::from_ptr(function).to_string_lossy()
    );
    std::process::abort();
}

pub fn main() {
    let owner = GameOwner::new();
    check_dependency();

    #[cfg(windows)]
    unsafe {
        stderr = __acrt_iob_func(2);
    }
    sm::host::install_host_hooks(sm::host::HostHooks {
        die: host_die,
        warning: host_warning,
        apu_lock: host_apu_lock,
        apu_unlock: host_apu_unlock,
    });
    if std::env::args().nth(1).as_deref() == Some("verify") {
        ::std::process::exit(owner.with(|game| verify::run(game)));
    }
    let mut args_strings: Vec<Vec<u8>> = ::std::env::args()
        .map(|arg| {
            ::std::ffi::CString::new(arg)
                .expect("Failed to convert argument into CString.")
                .into_bytes_with_nul()
        })
        .collect();
    let mut args_ptrs: Vec<*mut c_char> = args_strings
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut c_char)
        .chain(::core::iter::once(null_mut()))
        .collect();
    unsafe {
        ::std::process::exit(sm_desktop_run(
            (args_ptrs.len() - 1) as i32,
            args_ptrs.as_mut_ptr(),
            &*owner,
        ) as i32)
    }
}
