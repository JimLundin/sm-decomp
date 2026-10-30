#![deny(unsafe_op_in_unsafe_fn)]
#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_imports
)]
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
use sm::types::{FILE, RendererFuncs, Snes, size_t};
/// Config file parsing.
mod config;
/// Window, renderer and OpenGL output.
mod platform {
    pub mod glsl_shader;
    pub mod opengl;
    mod sdl_constants;
    mod texture_png;
}

macro_rules! type_aliases {
    ($visibility:vis $($name:ident),+ = $target:ty;) => {
        $($visibility type $name = $target;)+
    };
}
pub(crate) use type_aliases;

// The replay runner is shared with the sm_verify host.
#[path = "../../../sm/src/verify.rs"]
mod verify;
const kWindowTitle: &[u8] = b"SuperMet\0";
const NULL: *mut c_void = null_mut();
static mut g_pixels: [u8; 245760] = [0; 245760];
static mut g_my_pixels: [u8; 245760] = [0; 245760];

#[link(name = "sm_desktop", kind = "static", modifiers = "-bundle")]
unsafe extern "C-unwind" {
    fn sm_desktop_run(
        argc: i32,
        argv: *mut *mut c_char,
        owner: *const c_void,
        config: *mut c_void,
        audio: *mut c_void,
    ) -> i32;
    fn sm_desktop_close(audio: *mut c_void);
    fn sm_desktop_begin_audio() -> i32;
    fn sm_desktop_end_audio();
    fn sm_desktop_HandleInput(game: *mut c_void, key: i32, modifiers: i32, pressed: bool);
    fn sm_desktop_HandleGamepadInput(game: *mut c_void, button: i32, pressed: bool);
    fn sm_desktop_HandleGamepadAxisInput(game: *mut c_void, id: i32, axis: i32, value: i32);
    fn sm_desktop_HandleCommand(game: *mut c_void, command: u32, pressed: bool);
    fn sm_desktop_DrawPpuFrameWithPerf(game: *mut c_void);
    #[link_name = "sm_desktop_audio_mutex"]
    static mut g_audio_mutex: *mut SDL_mutex;
    fn sm_desktop_matches_source(index: u32, bytes: *const u8, length: usize) -> bool;
}
fn check_dependency() {
    for (index, source) in [
        include_bytes!("main.rs").as_slice(),
        include_bytes!("../../../sm/src/native_desktop.c").as_slice(),
        include_bytes!("../../../sm/src/native_desktop.h").as_slice(),
        include_bytes!("../../../sm/src/types.rs").as_slice(),
        include_bytes!("platform/opengl.rs").as_slice(),
        include_bytes!("../../../tools/install_native_desktop.py").as_slice(),
        include_bytes!("../../../tools/install_native_config.py").as_slice(),
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
    unsafe {
        SDL_ShowSimpleMessageBox(
            SDL_MESSAGEBOX_ERROR as i32 as Uint32,
            kWindowTitle.as_ptr().cast(),
            error,
            null_mut::<SDL_Window>(),
        );
        fprintf(stderr, c"Error: %s\n".as_ptr(), error);
        exit(1);
    }
}

unsafe fn host_warning(error: *const c_char) {
    unsafe {
        SDL_ShowSimpleMessageBox(
            SDL_MESSAGEBOX_WARNING as i32 as Uint32,
            kWindowTitle.as_ptr().cast(),
            error,
            null_mut::<SDL_Window>(),
        );
        fprintf(stderr, c"Warning: %s\n".as_ptr(), error);
    }
}

unsafe fn initialize_verify_audio(game: &mut Game) -> Result<(), String> {
    unsafe {
        g_audio_mutex = SDL_CreateMutex();
        if g_audio_mutex.is_null() {
            return Err("unable to create audio mutex".into());
        }
        game.create_native_player();
        if game.g_spc_player.is_null() {
            SDL_DestroyMutex(g_audio_mutex);
            return Err("unable to initialize sound driver".into());
        }
        game.initialize_native_player();
        Ok(())
    }
}

sm::forward_calls! { unsafe fn ();
    finish_verify_audio => SDL_DestroyMutex(g_audio_mutex);
    host_apu_lock => SDL_LockMutex(g_audio_mutex);
    host_apu_unlock => SDL_UnlockMutex(g_audio_mutex);
}

pub unsafe fn RtlDrawPpuFrame(
    pixel_buffer: *mut u8,
    pitch: size_t,
    _render_flags: u32,
    game: &mut Game,
) {
    unsafe {
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
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_die(message: *const c_char) {
    unsafe {
        Die(message);
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_stderr() -> *mut FILE {
    unsafe { stderr }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_parse_config(config: *mut c_void, name: *const c_char) {
    unsafe {
        crate::config::ParseConfigFile(name, &mut *config.cast::<crate::config::ConfigOwner>());
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_find_key(key: i32, modifiers: i32) -> i32 {
    unsafe { crate::config::FindCmdForSdlKey(key, modifiers as _) }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_find_button(button: i32, modifiers: u32) -> i32 {
    unsafe { crate::config::FindCmdForGamepadButton(button, modifiers) }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_debug(owner: *const GameOwner) {
    unsafe {
        (*owner).with(|game| game.g_debug_flag = true);
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_start_game(
    owner: *const GameOwner,
    filename: *const c_char,
) -> bool {
    unsafe {
        (*owner).with(|game| {
            if let Some(result) = game.start_pending_supplied_session() {
                if let Err(error) = result {
                    eprintln!("Supplied Session start rejected: {error:?}");
                    return false;
                }
                return true;
            }
            !sm::cpu_infra::SnesInit(filename, game).is_null()
        })
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_initialize_renderer(
    owner: *const GameOwner,
    funcs: *const RendererFuncs,
    window: *mut SDL_Window,
) -> bool {
    unsafe {
        (*owner).with(|_game| (*funcs).Initialize.expect("non-null function pointer")(window))
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_begin_draw(
    funcs: *const RendererFuncs,
    width: i32,
    height: i32,
    pixels: *mut *mut u8,
    pitch: *mut i32,
) {
    unsafe {
        (*funcs).BeginDraw.expect("non-null function pointer")(width, height, pixels, pitch);
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_end_draw(funcs: *const RendererFuncs) {
    unsafe {
        (*funcs).EndDraw.expect("non-null function pointer")();
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_create_player(owner: *const GameOwner) {
    unsafe {
        (*owner).with(|game| game.create_native_player());
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_initialize_player(owner: *const GameOwner) {
    unsafe {
        (*owner).with(|game| game.initialize_native_player());
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_prepare_pixels(owner: *const GameOwner) {
    unsafe {
        (*owner).with(|game| {
            let snes = game.g_snes;
            sm::snes::ppu::PpuBeginDrawing((*snes).snes_ppu, (&raw mut g_pixels).cast(), 1024, 0);
            sm::snes::ppu::PpuBeginDrawing((*snes).my_ppu, (&raw mut g_my_pixels).cast(), 1024, 0);
        });
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_read_sram(owner: *const GameOwner) {
    unsafe {
        (*owner).with(|game| RtlReadSram(game));
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_key(
    owner: *const GameOwner,
    key: i32,
    modifiers: i32,
    pressed: bool,
) {
    unsafe {
        (*owner).with(|game| {
            sm_desktop_HandleInput((game as *mut Game).cast(), key, modifiers, pressed)
        });
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_button(owner: *const GameOwner, button: i32, pressed: bool) {
    unsafe {
        (*owner).with(|game| {
            sm_desktop_HandleGamepadInput((game as *mut Game).cast(), button, pressed)
        });
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_axis(owner: *const GameOwner, id: i32, axis: i32, value: i32) {
    unsafe {
        (*owner).with(|game| {
            sm_desktop_HandleGamepadAxisInput((game as *mut Game).cast(), id, axis, value)
        });
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_command(owner: *const GameOwner, command: u32, pressed: bool) {
    unsafe {
        (*owner)
            .with(|game| sm_desktop_HandleCommand((game as *mut Game).cast(), command, pressed));
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_run_frame(owner: *const GameOwner, inputs: i32) -> bool {
    unsafe { (*owner).with(|game| RtlRunFrame(inputs, game)) }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_present(
    owner: *const GameOwner,
    disable_render: *mut bool,
) -> bool {
    unsafe {
        (*owner).with(|game| {
            (*game.g_snes).disableRender = *disable_render;
            if !*disable_render {
                sm_desktop_DrawPpuFrameWithPerf((game as *mut Game).cast());
            }
            *disable_render = (*game.g_snes).disableRender;
            game.g_got_mismatch_count != 0
        })
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_render_scale(game: *mut Game, flags: u32) -> i32 {
    unsafe { sm::snes::ppu::PpuGetCurrentRenderScale((*(*game).g_snes).ppu, flags) }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_copy_frame(
    game: *mut Game,
    pixels: *mut u8,
    pitch: usize,
    flags: u32,
) {
    unsafe {
        RtlDrawPpuFrame(pixels, pitch, flags, &mut *game);
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_mismatch_count(game: *mut Game) -> i32 {
    unsafe { (*game).g_got_mismatch_count }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_fps(value: f32) -> i32 {
    value as i32
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_save_load(game: *mut Game, operation: i32, slot: i32) {
    unsafe {
        RtlSaveLoad(operation, slot, &mut *game);
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_cheat(game: *mut Game, code: c_char) {
    unsafe {
        RtlCheat(code, &mut *game);
    }
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_game_command(game: *mut Game, command: u32, flags: i32) {
    unsafe {
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
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_gamepad_bounds(button: i32) {
    let _ = [0u8; 17][button as usize];
}
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_bad_command() {
    unsafe {
        __assert_fail(
            c"0".as_ptr(),
            c"src/main.c".as_ptr(),
            686,
            c"void HandleCommand(uint32, _Bool)".as_ptr(),
        );
    }
}
// One stable callback context, borrowed until SDL pause/close joins its thread.
// GameOwner remains non-sharing; SDL supplies the serialized callback boundary.
struct DesktopAudio<'a> {
    owner: &'a GameOwner,
    samples: std::cell::UnsafeCell<sm::SampleDelivery>,
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_prepare_audio(audio: *mut c_void, frames: i32) {
    unsafe {
        let audio = &*audio.cast::<DesktopAudio<'_>>();
        (*audio.samples.get())
            .prepare(usize::try_from(frames).expect("audio frames must be positive"));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_host_release_audio(audio: *mut c_void) {
    unsafe {
        let audio = &*audio.cast::<DesktopAudio<'_>>();
        (*audio.samples.get()).release();
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sm_host_audio_callback(userdata: *mut c_void, stream: *mut u8, length: i32) {
    unsafe {
        let audio = &*userdata.cast::<DesktopAudio<'_>>();
        audio.owner.with(|game| {
            // Preserve Game gate -> recursive SDL audio mutex -> producer locks.
            // Native begin/end receive no Game pointer and cannot reborrow it.
            let volume = sm_desktop_begin_audio();
            let length = if length > 0 { length as usize } else { 0 };
            // SDL supplies length writable bytes, which may be uninitialized.
            // The core bounds every index; no destination reference/read is made.
            (*audio.samples.get()).deliver_to(
                length,
                volume as u8,
                |block| RtlRenderAudio(block, game),
                |index, value| stream.add(index).write(value),
            );
            sm_desktop_end_audio();
        });
    }
}

// The transpiled runtime references glibc's stderr and assertion symbols.
// MinGW uses a stream accessor instead; FILE is passed opaquely to the CRT.
#[cfg(windows)]
#[unsafe(no_mangle)]
pub static mut stderr: *mut FILE = std::ptr::null_mut();

#[cfg(windows)]
unsafe extern "C" {
    fn __acrt_iob_func(index: u32) -> *mut FILE;
}

#[cfg(windows)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __assert_fail(
    assertion: *const std::ffi::c_char,
    file: *const std::ffi::c_char,
    line: u32,
    function: *const std::ffi::c_char,
) -> ! {
    unsafe {
        eprintln!(
            "Assertion failed: {} at {}:{line} ({})",
            std::ffi::CStr::from_ptr(assertion).to_string_lossy(),
            std::ffi::CStr::from_ptr(file).to_string_lossy(),
            std::ffi::CStr::from_ptr(function).to_string_lossy()
        );
        std::process::abort();
    }
}

pub fn main() {
    let mut argv = std::env::args();
    let executable = argv.next().unwrap_or_else(|| "sm".into());
    let supplied: Vec<String> = argv.collect();
    let selection = match sm::room_pack::select(&supplied) {
        Ok(selection) => selection,
        Err(error) => {
            eprintln!("sm: {error}");
            std::process::exit(2);
        }
    };
    let mut config = crate::config::ConfigOwner::new();
    let owner = match selection.world {
        Some(world) => GameOwner::with_world(world),
        None => GameOwner::new(),
    };
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
    if selection.arguments.first().map(String::as_str) == Some("verify") {
        let status = owner.with(|game| verify::run_args(&selection.arguments[1..], game));
        drop(owner);
        drop(config);
        ::std::process::exit(status);
    }
    let mut args_strings: Vec<Vec<u8>> = ::core::iter::once(executable)
        .chain(selection.arguments)
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
    // Drop order also applies when a host callback unwinds through the C loop.
    let audio = DesktopAudio {
        owner: &owner,
        samples: std::cell::UnsafeCell::new(sm::SampleDelivery::new()),
    };
    struct Desktop<'a>(&'a DesktopAudio<'a>);
    impl Drop for Desktop<'_> {
        fn drop(&mut self) {
            unsafe { sm_desktop_close((self.0 as *const DesktopAudio<'_>).cast_mut().cast()) }
        }
    }
    let desktop = Desktop(&audio);
    let status = unsafe {
        sm_desktop_run(
            (args_ptrs.len() - 1) as i32,
            args_ptrs.as_mut_ptr(),
            (&*owner as *const GameOwner).cast(),
            (&mut config as *mut crate::config::ConfigOwner).cast(),
            (&audio as *const DesktopAudio<'_>).cast_mut().cast(),
        )
    };
    drop(desktop);
    drop(owner);
    drop(config);
    ::std::process::exit(status);
}
