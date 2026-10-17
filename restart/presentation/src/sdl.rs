//! The small SDL2 ABI surface used by the presentation consumer.
//!
//! SDL2 public declarations: SDL_video.h, SDL_render.h, SDL_events.h and
//! SDL_audio.h. These layouts target SDL2 (not SDL3); no C build is required.

use std::ffi::{CStr, c_char, c_int, c_void};
use std::path::Path;

use libloading::Library;

use crate::Error;

pub const INIT: u32 = 0x10 | 0x20 | 0x4000; // Audio, video, events.
pub const ARGB8888: u32 = 0x16362004;
pub const QUIT: u32 = 0x100;
pub const WINDOW_EVENT: u32 = 0x200;
pub const KEY_DOWN: u32 = 0x300;
pub const KEY_UP: u32 = 0x301;
pub const FOCUS_LOST: u8 = 13;
pub const WINDOW_CLOSE: u8 = 14;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Keysym {
    pub scancode: c_int,
    pub symbol: c_int,
    pub modifiers: u16,
    pub unused: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeyboardEvent {
    pub kind: u32,
    pub timestamp: u32,
    pub window_id: u32,
    pub state: u8,
    pub repeat: u8,
    pub padding: [u8; 2],
    pub keysym: Keysym,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WindowEvent {
    pub kind: u32,
    pub timestamp: u32,
    pub window_id: u32,
    pub event: u8,
    pub padding: [u8; 3],
    pub data1: i32,
    pub data2: i32,
}

// SDL2 fixes SDL_Event to 56 bytes on the supported 32/64-bit targets. The
// pointer-bearing event variants require pointer alignment even though this
// consumer reads only keyboard/window/quit events.
#[repr(C)]
pub union Event {
    pub kind: u32,
    pub key: KeyboardEvent,
    pub window: WindowEvent,
    pub padding: [u8; 56],
    pub align: *mut c_void,
}

#[repr(C)]
pub struct AudioSpec {
    pub freq: c_int,
    pub format: u16,
    pub channels: u8,
    pub silence: u8,
    pub samples: u16,
    pub padding: u16,
    pub size: u32,
    pub callback: Option<unsafe extern "C" fn(*mut c_void, *mut u8, c_int)>,
    pub userdata: *mut c_void,
}

macro_rules! api {
    ($($field:ident: $symbol:literal => $signature:ty),+ $(,)?) => {
        pub struct Api {
            $(pub $field: $signature,)+
            // Retain the shared library until every function/resource is done.
            _library: Library,
        }

        impl Api {
            pub fn load(path: &Path) -> Result<Self, Error> {
                // SAFETY: caller selects a trusted SDL2 runtime. Symbols are
                // loaded with the signatures from SDL2's public C headers.
                unsafe {
                    let library = Library::new(path).map_err(|error|
                        Error(format!("load SDL2 {}: {error}", path.display())))?;
                    Ok(Self {
                        $($field: *library.get::<$signature>(concat!($symbol, "\0").as_bytes())
                            .map_err(|error| Error(format!("load {}: {error}", $symbol)))?,)+
                        _library: library,
                    })
                }
            }
        }
    };
}

api! {
    init: "SDL_Init" => unsafe extern "C" fn(u32) -> c_int,
    quit: "SDL_Quit" => unsafe extern "C" fn(),
    get_error: "SDL_GetError" => unsafe extern "C" fn() -> *const c_char,
    create_window: "SDL_CreateWindow" => unsafe extern "C" fn(*const c_char, c_int, c_int, c_int, c_int, u32) -> *mut c_void,
    destroy_window: "SDL_DestroyWindow" => unsafe extern "C" fn(*mut c_void),
    create_renderer: "SDL_CreateRenderer" => unsafe extern "C" fn(*mut c_void, c_int, u32) -> *mut c_void,
    destroy_renderer: "SDL_DestroyRenderer" => unsafe extern "C" fn(*mut c_void),
    set_logical_size: "SDL_RenderSetLogicalSize" => unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int,
    create_texture: "SDL_CreateTexture" => unsafe extern "C" fn(*mut c_void, u32, c_int, c_int, c_int) -> *mut c_void,
    destroy_texture: "SDL_DestroyTexture" => unsafe extern "C" fn(*mut c_void),
    set_blend_mode: "SDL_SetTextureBlendMode" => unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    update_texture: "SDL_UpdateTexture" => unsafe extern "C" fn(*mut c_void, *const c_void, *const c_void, c_int) -> c_int,
    render_clear: "SDL_RenderClear" => unsafe extern "C" fn(*mut c_void) -> c_int,
    render_copy: "SDL_RenderCopy" => unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_void, *const c_void) -> c_int,
    render_present: "SDL_RenderPresent" => unsafe extern "C" fn(*mut c_void),
    poll_event: "SDL_PollEvent" => unsafe extern "C" fn(*mut Event) -> c_int,
    open_audio: "SDL_OpenAudioDevice" => unsafe extern "C" fn(*const c_char, c_int, *const AudioSpec, *mut AudioSpec, c_int) -> u32,
    close_audio: "SDL_CloseAudioDevice" => unsafe extern "C" fn(u32),
    pause_audio: "SDL_PauseAudioDevice" => unsafe extern "C" fn(u32, c_int),
    queue_audio: "SDL_QueueAudio" => unsafe extern "C" fn(u32, *const c_void, u32) -> c_int,
    queued_audio_size: "SDL_GetQueuedAudioSize" => unsafe extern "C" fn(u32) -> u32,
    clear_audio: "SDL_ClearQueuedAudio" => unsafe extern "C" fn(u32),
}

impl Api {
    pub fn error(&self, operation: &str) -> Error {
        // SAFETY: SDL_GetError returns a valid NUL-terminated thread-local string.
        let detail = unsafe { CStr::from_ptr((self.get_error)()) };
        Error(format!("{operation}: {}", detail.to_string_lossy()))
    }
}
