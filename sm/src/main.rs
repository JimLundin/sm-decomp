#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
#![feature(extern_types)]
#[allow(unused_imports)]
use ::sm as _;
mod verify;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type SDL_mutex;
    pub type SDL_Window;
    pub type _SDL_GameController;
    pub type SDL_SysWMmsg;
    pub type SDL_Renderer;
    pub type SDL_Texture;
    static mut stderr: *mut FILE;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn sprintf(
        __s: *mut ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn exit(__status: ::core::ffi::c_int) -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn SDL_memset(
        dst: *mut ::core::ffi::c_void,
        c: ::core::ffi::c_int,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn SDL_GetError() -> *const ::core::ffi::c_char;
    fn SDL_CreateMutex() -> *mut SDL_mutex;
    fn SDL_LockMutex(mutex: *mut SDL_mutex) -> ::core::ffi::c_int;
    fn SDL_UnlockMutex(mutex: *mut SDL_mutex) -> ::core::ffi::c_int;
    fn SDL_DestroyMutex(mutex: *mut SDL_mutex);
    fn SDL_OpenAudioDevice(
        device: *const ::core::ffi::c_char,
        iscapture: ::core::ffi::c_int,
        desired: *const SDL_AudioSpec,
        obtained: *mut SDL_AudioSpec,
        allowed_changes: ::core::ffi::c_int,
    ) -> SDL_AudioDeviceID;
    fn SDL_PauseAudioDevice(dev: SDL_AudioDeviceID, pause_on: ::core::ffi::c_int);
    fn SDL_MixAudioFormat(
        dst: *mut Uint8,
        src: *const Uint8,
        format: SDL_AudioFormat,
        len: Uint32,
        volume: ::core::ffi::c_int,
    );
    fn SDL_CloseAudioDevice(dev: SDL_AudioDeviceID);
    fn SDL_GetPixelFormatName(format: Uint32) -> *const ::core::ffi::c_char;
    fn SDL_GetDisplayUsableBounds(
        displayIndex: ::core::ffi::c_int,
        rect: *mut SDL_Rect,
    ) -> ::core::ffi::c_int;
    fn SDL_GetWindowDisplayIndex(window: *mut SDL_Window) -> ::core::ffi::c_int;
    fn SDL_CreateWindow(
        title: *const ::core::ffi::c_char,
        x: ::core::ffi::c_int,
        y: ::core::ffi::c_int,
        w: ::core::ffi::c_int,
        h: ::core::ffi::c_int,
        flags: Uint32,
    ) -> *mut SDL_Window;
    fn SDL_GetWindowFlags(window: *mut SDL_Window) -> Uint32;
    fn SDL_SetWindowTitle(window: *mut SDL_Window, title: *const ::core::ffi::c_char);
    fn SDL_SetWindowPosition(window: *mut SDL_Window, x: ::core::ffi::c_int, y: ::core::ffi::c_int);
    fn SDL_SetWindowSize(window: *mut SDL_Window, w: ::core::ffi::c_int, h: ::core::ffi::c_int);
    fn SDL_GetWindowSize(
        window: *mut SDL_Window,
        w: *mut ::core::ffi::c_int,
        h: *mut ::core::ffi::c_int,
    );
    fn SDL_GetWindowBordersSize(
        window: *mut SDL_Window,
        top: *mut ::core::ffi::c_int,
        left: *mut ::core::ffi::c_int,
        bottom: *mut ::core::ffi::c_int,
        right: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn SDL_SetWindowBordered(window: *mut SDL_Window, bordered: SDL_bool);
    fn SDL_SetWindowFullscreen(window: *mut SDL_Window, flags: Uint32) -> ::core::ffi::c_int;
    fn SDL_SetWindowHitTest(
        window: *mut SDL_Window,
        callback: SDL_HitTest,
        callback_data: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn SDL_DestroyWindow(window: *mut SDL_Window);
    fn SDL_GetModState() -> SDL_Keymod;
    fn SDL_GetGlobalMouseState(x: *mut ::core::ffi::c_int, y: *mut ::core::ffi::c_int) -> Uint32;
    fn SDL_ShowCursor(toggle: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn SDL_NumJoysticks() -> ::core::ffi::c_int;
    fn SDL_IsGameController(joystick_index: ::core::ffi::c_int) -> SDL_bool;
    fn SDL_GameControllerOpen(joystick_index: ::core::ffi::c_int) -> *mut SDL_GameController;
    fn SDL_PollEvent(event: *mut SDL_Event) -> ::core::ffi::c_int;
    fn SDL_SetHint(name: *const ::core::ffi::c_char, value: *const ::core::ffi::c_char)
        -> SDL_bool;
    fn SDL_ShowSimpleMessageBox(
        flags: Uint32,
        title: *const ::core::ffi::c_char,
        message: *const ::core::ffi::c_char,
        window: *mut SDL_Window,
    ) -> ::core::ffi::c_int;
    fn SDL_CreateRenderer(
        window: *mut SDL_Window,
        index: ::core::ffi::c_int,
        flags: Uint32,
    ) -> *mut SDL_Renderer;
    fn SDL_GetRendererInfo(
        renderer: *mut SDL_Renderer,
        info: *mut SDL_RendererInfo,
    ) -> ::core::ffi::c_int;
    fn SDL_CreateTexture(
        renderer: *mut SDL_Renderer,
        format: Uint32,
        access: ::core::ffi::c_int,
        w: ::core::ffi::c_int,
        h: ::core::ffi::c_int,
    ) -> *mut SDL_Texture;
    fn SDL_LockTexture(
        texture: *mut SDL_Texture,
        rect: *const SDL_Rect,
        pixels: *mut *mut ::core::ffi::c_void,
        pitch: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn SDL_UnlockTexture(texture: *mut SDL_Texture);
    fn SDL_RenderSetLogicalSize(
        renderer: *mut SDL_Renderer,
        w: ::core::ffi::c_int,
        h: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn SDL_RenderClear(renderer: *mut SDL_Renderer) -> ::core::ffi::c_int;
    fn SDL_RenderCopy(
        renderer: *mut SDL_Renderer,
        texture: *mut SDL_Texture,
        srcrect: *const SDL_Rect,
        dstrect: *const SDL_Rect,
    ) -> ::core::ffi::c_int;
    fn SDL_RenderPresent(renderer: *mut SDL_Renderer);
    fn SDL_DestroyTexture(texture: *mut SDL_Texture);
    fn SDL_DestroyRenderer(renderer: *mut SDL_Renderer);
    fn SDL_GetTicks() -> Uint32;
    fn SDL_GetPerformanceCounter() -> Uint64;
    fn SDL_GetPerformanceFrequency() -> Uint64;
    fn SDL_Delay(ms: Uint32);
    fn SDL_Init(flags: Uint32) -> ::core::ffi::c_int;
    fn SDL_Quit();
    fn mkdir(__path: *const ::core::ffi::c_char, __mode: __mode_t) -> ::core::ffi::c_int;
    fn chdir(__path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn getcwd(__buf: *mut ::core::ffi::c_char, __size: size_t) -> *mut ::core::ffi::c_char;
    fn PpuBeginDrawing(ppu: *mut Ppu, pixels: *mut uint8_t, pitch: size_t, render_flags: uint32_t);
    fn PpuGetCurrentRenderScale(ppu: *mut Ppu, render_flags: uint32_t) -> ::core::ffi::c_int;
    fn RtlReset(mode: ::core::ffi::c_int);
    fn RtlClearKeyLog();
    fn RtlStopReplay();
    fn RtlSaveLoad(cmd: ::core::ffi::c_int, slot: ::core::ffi::c_int);
    fn RtlCheat(c: ::core::ffi::c_char);
    fn RtlRenderAudio(
        audio_buffer: *mut int16,
        samples: ::core::ffi::c_int,
        channels: ::core::ffi::c_int,
    );
    fn RtlRunFrame(inputs: ::core::ffi::c_int) -> bool;
    fn RtlReadSram();
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn OpenGLRenderer_Create(funcs: *mut RendererFuncs);
    fn SpcPlayer_Create() -> *mut SpcPlayer;
    fn SpcPlayer_Initialize(p: *mut SpcPlayer);
    static mut g_snes: *mut Snes;
    fn SnesInit(filename: *const ::core::ffi::c_char) -> *mut Snes;
    static mut g_config: Config;
    fn ParseConfigFile(filename: *const ::core::ffi::c_char);
    fn FindCmdForSdlKey(code: SDL_Keycode, mod_0: SDL_Keymod) -> ::core::ffi::c_int;
    fn FindCmdForGamepadButton(button: ::core::ffi::c_int, modifiers: uint32)
        -> ::core::ffi::c_int;
}
pub use sm::types::{
    __int16_t, __int32_t, __int64_t, __int8_t, __mode_t, __off64_t, __off_t, __uint16_t,
    __uint32_t, __uint64_t, __uint8_t, size_t,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub __pad5: size_t,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub use sm::types::_IO_lock_t;
pub type FILE = _IO_FILE;
pub use sm::types::{
    int16_t, int32_t, int64_t, int8_t, uint16_t, uint32_t, uint64_t, uint8_t, SDL_bool,
};
pub const SDL_TRUE: SDL_bool = 1;
pub const SDL_FALSE: SDL_bool = 0;
pub use sm::types::{
    SDL_AudioCallback, SDL_AudioFormat, Sint16, Sint32, Sint64, Uint16, Uint32, Uint64, Uint8,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_AudioSpec {
    pub freq: ::core::ffi::c_int,
    pub format: SDL_AudioFormat,
    pub channels: Uint8,
    pub silence: Uint8,
    pub samples: Uint16,
    pub padding: Uint16,
    pub size: Uint32,
    pub callback: SDL_AudioCallback,
    pub userdata: *mut ::core::ffi::c_void,
}
pub use sm::types::SDL_AudioDeviceID;
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const SDL_PIXELFORMAT_EXTERNAL_OES: C2RustUnnamed = 542328143;
pub const SDL_PIXELFORMAT_NV21: C2RustUnnamed = 825382478;
pub const SDL_PIXELFORMAT_NV12: C2RustUnnamed = 842094158;
pub const SDL_PIXELFORMAT_YVYU: C2RustUnnamed = 1431918169;
pub const SDL_PIXELFORMAT_UYVY: C2RustUnnamed = 1498831189;
pub const SDL_PIXELFORMAT_YUY2: C2RustUnnamed = 844715353;
pub const SDL_PIXELFORMAT_IYUV: C2RustUnnamed = 1448433993;
pub const SDL_PIXELFORMAT_YV12: C2RustUnnamed = 842094169;
pub const SDL_PIXELFORMAT_XBGR32: C2RustUnnamed = 371595268;
pub const SDL_PIXELFORMAT_BGRX32: C2RustUnnamed = 370546692;
pub const SDL_PIXELFORMAT_XRGB32: C2RustUnnamed = 375789572;
pub const SDL_PIXELFORMAT_RGBX32: C2RustUnnamed = 374740996;
pub const SDL_PIXELFORMAT_ABGR32: C2RustUnnamed = 373694468;
pub const SDL_PIXELFORMAT_BGRA32: C2RustUnnamed = 372645892;
pub const SDL_PIXELFORMAT_ARGB32: C2RustUnnamed = 377888772;
pub const SDL_PIXELFORMAT_RGBA32: C2RustUnnamed = 376840196;
pub const SDL_PIXELFORMAT_ARGB2101010: C2RustUnnamed = 372711428;
pub const SDL_PIXELFORMAT_BGRA8888: C2RustUnnamed = 377888772;
pub const SDL_PIXELFORMAT_ABGR8888: C2RustUnnamed = 376840196;
pub const SDL_PIXELFORMAT_RGBA8888: C2RustUnnamed = 373694468;
pub const SDL_PIXELFORMAT_ARGB8888: C2RustUnnamed = 372645892;
pub const SDL_PIXELFORMAT_BGRX8888: C2RustUnnamed = 375789572;
pub const SDL_PIXELFORMAT_BGR888: C2RustUnnamed = 374740996;
pub const SDL_PIXELFORMAT_XBGR8888: C2RustUnnamed = 374740996;
pub const SDL_PIXELFORMAT_RGBX8888: C2RustUnnamed = 371595268;
pub const SDL_PIXELFORMAT_RGB888: C2RustUnnamed = 370546692;
pub const SDL_PIXELFORMAT_XRGB8888: C2RustUnnamed = 370546692;
pub const SDL_PIXELFORMAT_BGR24: C2RustUnnamed = 390076419;
pub const SDL_PIXELFORMAT_RGB24: C2RustUnnamed = 386930691;
pub const SDL_PIXELFORMAT_BGR565: C2RustUnnamed = 357896194;
pub const SDL_PIXELFORMAT_RGB565: C2RustUnnamed = 353701890;
pub const SDL_PIXELFORMAT_BGRA5551: C2RustUnnamed = 360976386;
pub const SDL_PIXELFORMAT_ABGR1555: C2RustUnnamed = 359862274;
pub const SDL_PIXELFORMAT_RGBA5551: C2RustUnnamed = 356782082;
pub const SDL_PIXELFORMAT_ARGB1555: C2RustUnnamed = 355667970;
pub const SDL_PIXELFORMAT_BGRA4444: C2RustUnnamed = 360845314;
pub const SDL_PIXELFORMAT_ABGR4444: C2RustUnnamed = 359796738;
pub const SDL_PIXELFORMAT_RGBA4444: C2RustUnnamed = 356651010;
pub const SDL_PIXELFORMAT_ARGB4444: C2RustUnnamed = 355602434;
pub const SDL_PIXELFORMAT_BGR555: C2RustUnnamed = 357764866;
pub const SDL_PIXELFORMAT_XBGR1555: C2RustUnnamed = 357764866;
pub const SDL_PIXELFORMAT_RGB555: C2RustUnnamed = 353570562;
pub const SDL_PIXELFORMAT_XRGB1555: C2RustUnnamed = 353570562;
pub const SDL_PIXELFORMAT_BGR444: C2RustUnnamed = 357698562;
pub const SDL_PIXELFORMAT_XBGR4444: C2RustUnnamed = 357698562;
pub const SDL_PIXELFORMAT_RGB444: C2RustUnnamed = 353504258;
pub const SDL_PIXELFORMAT_XRGB4444: C2RustUnnamed = 353504258;
pub const SDL_PIXELFORMAT_RGB332: C2RustUnnamed = 336660481;
pub const SDL_PIXELFORMAT_INDEX8: C2RustUnnamed = 318769153;
pub const SDL_PIXELFORMAT_INDEX4MSB: C2RustUnnamed = 304088064;
pub const SDL_PIXELFORMAT_INDEX4LSB: C2RustUnnamed = 303039488;
pub const SDL_PIXELFORMAT_INDEX2MSB: C2RustUnnamed = 471859712;
pub const SDL_PIXELFORMAT_INDEX2LSB: C2RustUnnamed = 470811136;
pub const SDL_PIXELFORMAT_INDEX1MSB: C2RustUnnamed = 287310080;
pub const SDL_PIXELFORMAT_INDEX1LSB: C2RustUnnamed = 286261504;
pub const SDL_PIXELFORMAT_UNKNOWN: C2RustUnnamed = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_Point {
    pub x: ::core::ffi::c_int,
    pub y: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_Rect {
    pub x: ::core::ffi::c_int,
    pub y: ::core::ffi::c_int,
    pub w: ::core::ffi::c_int,
    pub h: ::core::ffi::c_int,
}
pub type C2RustUnnamed_0 = ::core::ffi::c_uint;
pub const SDL_WINDOW_INPUT_GRABBED: C2RustUnnamed_0 = 256;
pub const SDL_WINDOW_METAL: C2RustUnnamed_0 = 536870912;
pub const SDL_WINDOW_VULKAN: C2RustUnnamed_0 = 268435456;
pub const SDL_WINDOW_KEYBOARD_GRABBED: C2RustUnnamed_0 = 1048576;
pub const SDL_WINDOW_POPUP_MENU: C2RustUnnamed_0 = 524288;
pub const SDL_WINDOW_TOOLTIP: C2RustUnnamed_0 = 262144;
pub const SDL_WINDOW_UTILITY: C2RustUnnamed_0 = 131072;
pub const SDL_WINDOW_SKIP_TASKBAR: C2RustUnnamed_0 = 65536;
pub const SDL_WINDOW_ALWAYS_ON_TOP: C2RustUnnamed_0 = 32768;
pub const SDL_WINDOW_MOUSE_CAPTURE: C2RustUnnamed_0 = 16384;
pub const SDL_WINDOW_ALLOW_HIGHDPI: C2RustUnnamed_0 = 8192;
pub const SDL_WINDOW_FOREIGN: C2RustUnnamed_0 = 2048;
pub const SDL_WINDOW_FULLSCREEN_DESKTOP: C2RustUnnamed_0 = 4097;
pub const SDL_WINDOW_MOUSE_FOCUS: C2RustUnnamed_0 = 1024;
pub const SDL_WINDOW_INPUT_FOCUS: C2RustUnnamed_0 = 512;
pub const SDL_WINDOW_MOUSE_GRABBED: C2RustUnnamed_0 = 256;
pub const SDL_WINDOW_MAXIMIZED: C2RustUnnamed_0 = 128;
pub const SDL_WINDOW_MINIMIZED: C2RustUnnamed_0 = 64;
pub const SDL_WINDOW_RESIZABLE: C2RustUnnamed_0 = 32;
pub const SDL_WINDOW_BORDERLESS: C2RustUnnamed_0 = 16;
pub const SDL_WINDOW_HIDDEN: C2RustUnnamed_0 = 8;
pub const SDL_WINDOW_SHOWN: C2RustUnnamed_0 = 4;
pub const SDL_WINDOW_OPENGL: C2RustUnnamed_0 = 2;
pub const SDL_WINDOW_FULLSCREEN: C2RustUnnamed_0 = 1;
pub use sm::types::SDL_HitTestResult;
pub const SDL_HITTEST_RESIZE_LEFT: SDL_HitTestResult = 9;
pub const SDL_HITTEST_RESIZE_BOTTOMLEFT: SDL_HitTestResult = 8;
pub const SDL_HITTEST_RESIZE_BOTTOM: SDL_HitTestResult = 7;
pub const SDL_HITTEST_RESIZE_BOTTOMRIGHT: SDL_HitTestResult = 6;
pub const SDL_HITTEST_RESIZE_RIGHT: SDL_HitTestResult = 5;
pub const SDL_HITTEST_RESIZE_TOPRIGHT: SDL_HitTestResult = 4;
pub const SDL_HITTEST_RESIZE_TOP: SDL_HitTestResult = 3;
pub const SDL_HITTEST_RESIZE_TOPLEFT: SDL_HitTestResult = 2;
pub const SDL_HITTEST_DRAGGABLE: SDL_HitTestResult = 1;
pub const SDL_HITTEST_NORMAL: SDL_HitTestResult = 0;
pub type SDL_HitTest = Option<
    unsafe extern "C" fn(
        *mut SDL_Window,
        *const SDL_Point,
        *mut ::core::ffi::c_void,
    ) -> SDL_HitTestResult,
>;
pub use sm::types::SDL_Scancode;
pub const SDL_NUM_SCANCODES: SDL_Scancode = 512;
pub const SDL_SCANCODE_ENDCALL: SDL_Scancode = 290;
pub const SDL_SCANCODE_CALL: SDL_Scancode = 289;
pub const SDL_SCANCODE_SOFTRIGHT: SDL_Scancode = 288;
pub const SDL_SCANCODE_SOFTLEFT: SDL_Scancode = 287;
pub const SDL_SCANCODE_AUDIOFASTFORWARD: SDL_Scancode = 286;
pub const SDL_SCANCODE_AUDIOREWIND: SDL_Scancode = 285;
pub const SDL_SCANCODE_APP2: SDL_Scancode = 284;
pub const SDL_SCANCODE_APP1: SDL_Scancode = 283;
pub const SDL_SCANCODE_SLEEP: SDL_Scancode = 282;
pub const SDL_SCANCODE_EJECT: SDL_Scancode = 281;
pub const SDL_SCANCODE_KBDILLUMUP: SDL_Scancode = 280;
pub const SDL_SCANCODE_KBDILLUMDOWN: SDL_Scancode = 279;
pub const SDL_SCANCODE_KBDILLUMTOGGLE: SDL_Scancode = 278;
pub const SDL_SCANCODE_DISPLAYSWITCH: SDL_Scancode = 277;
pub const SDL_SCANCODE_BRIGHTNESSUP: SDL_Scancode = 276;
pub const SDL_SCANCODE_BRIGHTNESSDOWN: SDL_Scancode = 275;
pub const SDL_SCANCODE_AC_BOOKMARKS: SDL_Scancode = 274;
pub const SDL_SCANCODE_AC_REFRESH: SDL_Scancode = 273;
pub const SDL_SCANCODE_AC_STOP: SDL_Scancode = 272;
pub const SDL_SCANCODE_AC_FORWARD: SDL_Scancode = 271;
pub const SDL_SCANCODE_AC_BACK: SDL_Scancode = 270;
pub const SDL_SCANCODE_AC_HOME: SDL_Scancode = 269;
pub const SDL_SCANCODE_AC_SEARCH: SDL_Scancode = 268;
pub const SDL_SCANCODE_COMPUTER: SDL_Scancode = 267;
pub const SDL_SCANCODE_CALCULATOR: SDL_Scancode = 266;
pub const SDL_SCANCODE_MAIL: SDL_Scancode = 265;
pub const SDL_SCANCODE_WWW: SDL_Scancode = 264;
pub const SDL_SCANCODE_MEDIASELECT: SDL_Scancode = 263;
pub const SDL_SCANCODE_AUDIOMUTE: SDL_Scancode = 262;
pub const SDL_SCANCODE_AUDIOPLAY: SDL_Scancode = 261;
pub const SDL_SCANCODE_AUDIOSTOP: SDL_Scancode = 260;
pub const SDL_SCANCODE_AUDIOPREV: SDL_Scancode = 259;
pub const SDL_SCANCODE_AUDIONEXT: SDL_Scancode = 258;
pub const SDL_SCANCODE_MODE: SDL_Scancode = 257;
pub const SDL_SCANCODE_RGUI: SDL_Scancode = 231;
pub const SDL_SCANCODE_RALT: SDL_Scancode = 230;
pub const SDL_SCANCODE_RSHIFT: SDL_Scancode = 229;
pub const SDL_SCANCODE_RCTRL: SDL_Scancode = 228;
pub const SDL_SCANCODE_LGUI: SDL_Scancode = 227;
pub const SDL_SCANCODE_LALT: SDL_Scancode = 226;
pub const SDL_SCANCODE_LSHIFT: SDL_Scancode = 225;
pub const SDL_SCANCODE_LCTRL: SDL_Scancode = 224;
pub const SDL_SCANCODE_KP_HEXADECIMAL: SDL_Scancode = 221;
pub const SDL_SCANCODE_KP_DECIMAL: SDL_Scancode = 220;
pub const SDL_SCANCODE_KP_OCTAL: SDL_Scancode = 219;
pub const SDL_SCANCODE_KP_BINARY: SDL_Scancode = 218;
pub const SDL_SCANCODE_KP_CLEARENTRY: SDL_Scancode = 217;
pub const SDL_SCANCODE_KP_CLEAR: SDL_Scancode = 216;
pub const SDL_SCANCODE_KP_PLUSMINUS: SDL_Scancode = 215;
pub const SDL_SCANCODE_KP_MEMDIVIDE: SDL_Scancode = 214;
pub const SDL_SCANCODE_KP_MEMMULTIPLY: SDL_Scancode = 213;
pub const SDL_SCANCODE_KP_MEMSUBTRACT: SDL_Scancode = 212;
pub const SDL_SCANCODE_KP_MEMADD: SDL_Scancode = 211;
pub const SDL_SCANCODE_KP_MEMCLEAR: SDL_Scancode = 210;
pub const SDL_SCANCODE_KP_MEMRECALL: SDL_Scancode = 209;
pub const SDL_SCANCODE_KP_MEMSTORE: SDL_Scancode = 208;
pub const SDL_SCANCODE_KP_EXCLAM: SDL_Scancode = 207;
pub const SDL_SCANCODE_KP_AT: SDL_Scancode = 206;
pub const SDL_SCANCODE_KP_SPACE: SDL_Scancode = 205;
pub const SDL_SCANCODE_KP_HASH: SDL_Scancode = 204;
pub const SDL_SCANCODE_KP_COLON: SDL_Scancode = 203;
pub const SDL_SCANCODE_KP_DBLVERTICALBAR: SDL_Scancode = 202;
pub const SDL_SCANCODE_KP_VERTICALBAR: SDL_Scancode = 201;
pub const SDL_SCANCODE_KP_DBLAMPERSAND: SDL_Scancode = 200;
pub const SDL_SCANCODE_KP_AMPERSAND: SDL_Scancode = 199;
pub const SDL_SCANCODE_KP_GREATER: SDL_Scancode = 198;
pub const SDL_SCANCODE_KP_LESS: SDL_Scancode = 197;
pub const SDL_SCANCODE_KP_PERCENT: SDL_Scancode = 196;
pub const SDL_SCANCODE_KP_POWER: SDL_Scancode = 195;
pub const SDL_SCANCODE_KP_XOR: SDL_Scancode = 194;
pub const SDL_SCANCODE_KP_F: SDL_Scancode = 193;
pub const SDL_SCANCODE_KP_E: SDL_Scancode = 192;
pub const SDL_SCANCODE_KP_D: SDL_Scancode = 191;
pub const SDL_SCANCODE_KP_C: SDL_Scancode = 190;
pub const SDL_SCANCODE_KP_B: SDL_Scancode = 189;
pub const SDL_SCANCODE_KP_A: SDL_Scancode = 188;
pub const SDL_SCANCODE_KP_BACKSPACE: SDL_Scancode = 187;
pub const SDL_SCANCODE_KP_TAB: SDL_Scancode = 186;
pub const SDL_SCANCODE_KP_RIGHTBRACE: SDL_Scancode = 185;
pub const SDL_SCANCODE_KP_LEFTBRACE: SDL_Scancode = 184;
pub const SDL_SCANCODE_KP_RIGHTPAREN: SDL_Scancode = 183;
pub const SDL_SCANCODE_KP_LEFTPAREN: SDL_Scancode = 182;
pub const SDL_SCANCODE_CURRENCYSUBUNIT: SDL_Scancode = 181;
pub const SDL_SCANCODE_CURRENCYUNIT: SDL_Scancode = 180;
pub const SDL_SCANCODE_DECIMALSEPARATOR: SDL_Scancode = 179;
pub const SDL_SCANCODE_THOUSANDSSEPARATOR: SDL_Scancode = 178;
pub const SDL_SCANCODE_KP_000: SDL_Scancode = 177;
pub const SDL_SCANCODE_KP_00: SDL_Scancode = 176;
pub const SDL_SCANCODE_EXSEL: SDL_Scancode = 164;
pub const SDL_SCANCODE_CRSEL: SDL_Scancode = 163;
pub const SDL_SCANCODE_CLEARAGAIN: SDL_Scancode = 162;
pub const SDL_SCANCODE_OPER: SDL_Scancode = 161;
pub const SDL_SCANCODE_OUT: SDL_Scancode = 160;
pub const SDL_SCANCODE_SEPARATOR: SDL_Scancode = 159;
pub const SDL_SCANCODE_RETURN2: SDL_Scancode = 158;
pub const SDL_SCANCODE_PRIOR: SDL_Scancode = 157;
pub const SDL_SCANCODE_CLEAR: SDL_Scancode = 156;
pub const SDL_SCANCODE_CANCEL: SDL_Scancode = 155;
pub const SDL_SCANCODE_SYSREQ: SDL_Scancode = 154;
pub const SDL_SCANCODE_ALTERASE: SDL_Scancode = 153;
pub const SDL_SCANCODE_LANG9: SDL_Scancode = 152;
pub const SDL_SCANCODE_LANG8: SDL_Scancode = 151;
pub const SDL_SCANCODE_LANG7: SDL_Scancode = 150;
pub const SDL_SCANCODE_LANG6: SDL_Scancode = 149;
pub const SDL_SCANCODE_LANG5: SDL_Scancode = 148;
pub const SDL_SCANCODE_LANG4: SDL_Scancode = 147;
pub const SDL_SCANCODE_LANG3: SDL_Scancode = 146;
pub const SDL_SCANCODE_LANG2: SDL_Scancode = 145;
pub const SDL_SCANCODE_LANG1: SDL_Scancode = 144;
pub const SDL_SCANCODE_INTERNATIONAL9: SDL_Scancode = 143;
pub const SDL_SCANCODE_INTERNATIONAL8: SDL_Scancode = 142;
pub const SDL_SCANCODE_INTERNATIONAL7: SDL_Scancode = 141;
pub const SDL_SCANCODE_INTERNATIONAL6: SDL_Scancode = 140;
pub const SDL_SCANCODE_INTERNATIONAL5: SDL_Scancode = 139;
pub const SDL_SCANCODE_INTERNATIONAL4: SDL_Scancode = 138;
pub const SDL_SCANCODE_INTERNATIONAL3: SDL_Scancode = 137;
pub const SDL_SCANCODE_INTERNATIONAL2: SDL_Scancode = 136;
pub const SDL_SCANCODE_INTERNATIONAL1: SDL_Scancode = 135;
pub const SDL_SCANCODE_KP_EQUALSAS400: SDL_Scancode = 134;
pub const SDL_SCANCODE_KP_COMMA: SDL_Scancode = 133;
pub const SDL_SCANCODE_VOLUMEDOWN: SDL_Scancode = 129;
pub const SDL_SCANCODE_VOLUMEUP: SDL_Scancode = 128;
pub const SDL_SCANCODE_MUTE: SDL_Scancode = 127;
pub const SDL_SCANCODE_FIND: SDL_Scancode = 126;
pub const SDL_SCANCODE_PASTE: SDL_Scancode = 125;
pub const SDL_SCANCODE_COPY: SDL_Scancode = 124;
pub const SDL_SCANCODE_CUT: SDL_Scancode = 123;
pub const SDL_SCANCODE_UNDO: SDL_Scancode = 122;
pub const SDL_SCANCODE_AGAIN: SDL_Scancode = 121;
pub const SDL_SCANCODE_STOP: SDL_Scancode = 120;
pub const SDL_SCANCODE_SELECT: SDL_Scancode = 119;
pub const SDL_SCANCODE_MENU: SDL_Scancode = 118;
pub const SDL_SCANCODE_HELP: SDL_Scancode = 117;
pub const SDL_SCANCODE_EXECUTE: SDL_Scancode = 116;
pub const SDL_SCANCODE_F24: SDL_Scancode = 115;
pub const SDL_SCANCODE_F23: SDL_Scancode = 114;
pub const SDL_SCANCODE_F22: SDL_Scancode = 113;
pub const SDL_SCANCODE_F21: SDL_Scancode = 112;
pub const SDL_SCANCODE_F20: SDL_Scancode = 111;
pub const SDL_SCANCODE_F19: SDL_Scancode = 110;
pub const SDL_SCANCODE_F18: SDL_Scancode = 109;
pub const SDL_SCANCODE_F17: SDL_Scancode = 108;
pub const SDL_SCANCODE_F16: SDL_Scancode = 107;
pub const SDL_SCANCODE_F15: SDL_Scancode = 106;
pub const SDL_SCANCODE_F14: SDL_Scancode = 105;
pub const SDL_SCANCODE_F13: SDL_Scancode = 104;
pub const SDL_SCANCODE_KP_EQUALS: SDL_Scancode = 103;
pub const SDL_SCANCODE_POWER: SDL_Scancode = 102;
pub const SDL_SCANCODE_APPLICATION: SDL_Scancode = 101;
pub const SDL_SCANCODE_NONUSBACKSLASH: SDL_Scancode = 100;
pub const SDL_SCANCODE_KP_PERIOD: SDL_Scancode = 99;
pub const SDL_SCANCODE_KP_0: SDL_Scancode = 98;
pub const SDL_SCANCODE_KP_9: SDL_Scancode = 97;
pub const SDL_SCANCODE_KP_8: SDL_Scancode = 96;
pub const SDL_SCANCODE_KP_7: SDL_Scancode = 95;
pub const SDL_SCANCODE_KP_6: SDL_Scancode = 94;
pub const SDL_SCANCODE_KP_5: SDL_Scancode = 93;
pub const SDL_SCANCODE_KP_4: SDL_Scancode = 92;
pub const SDL_SCANCODE_KP_3: SDL_Scancode = 91;
pub const SDL_SCANCODE_KP_2: SDL_Scancode = 90;
pub const SDL_SCANCODE_KP_1: SDL_Scancode = 89;
pub const SDL_SCANCODE_KP_ENTER: SDL_Scancode = 88;
pub const SDL_SCANCODE_KP_PLUS: SDL_Scancode = 87;
pub const SDL_SCANCODE_KP_MINUS: SDL_Scancode = 86;
pub const SDL_SCANCODE_KP_MULTIPLY: SDL_Scancode = 85;
pub const SDL_SCANCODE_KP_DIVIDE: SDL_Scancode = 84;
pub const SDL_SCANCODE_NUMLOCKCLEAR: SDL_Scancode = 83;
pub const SDL_SCANCODE_UP: SDL_Scancode = 82;
pub const SDL_SCANCODE_DOWN: SDL_Scancode = 81;
pub const SDL_SCANCODE_LEFT: SDL_Scancode = 80;
pub const SDL_SCANCODE_RIGHT: SDL_Scancode = 79;
pub const SDL_SCANCODE_PAGEDOWN: SDL_Scancode = 78;
pub const SDL_SCANCODE_END: SDL_Scancode = 77;
pub const SDL_SCANCODE_DELETE: SDL_Scancode = 76;
pub const SDL_SCANCODE_PAGEUP: SDL_Scancode = 75;
pub const SDL_SCANCODE_HOME: SDL_Scancode = 74;
pub const SDL_SCANCODE_INSERT: SDL_Scancode = 73;
pub const SDL_SCANCODE_PAUSE: SDL_Scancode = 72;
pub const SDL_SCANCODE_SCROLLLOCK: SDL_Scancode = 71;
pub const SDL_SCANCODE_PRINTSCREEN: SDL_Scancode = 70;
pub const SDL_SCANCODE_F12: SDL_Scancode = 69;
pub const SDL_SCANCODE_F11: SDL_Scancode = 68;
pub const SDL_SCANCODE_F10: SDL_Scancode = 67;
pub const SDL_SCANCODE_F9: SDL_Scancode = 66;
pub const SDL_SCANCODE_F8: SDL_Scancode = 65;
pub const SDL_SCANCODE_F7: SDL_Scancode = 64;
pub const SDL_SCANCODE_F6: SDL_Scancode = 63;
pub const SDL_SCANCODE_F5: SDL_Scancode = 62;
pub const SDL_SCANCODE_F4: SDL_Scancode = 61;
pub const SDL_SCANCODE_F3: SDL_Scancode = 60;
pub const SDL_SCANCODE_F2: SDL_Scancode = 59;
pub const SDL_SCANCODE_F1: SDL_Scancode = 58;
pub const SDL_SCANCODE_CAPSLOCK: SDL_Scancode = 57;
pub const SDL_SCANCODE_SLASH: SDL_Scancode = 56;
pub const SDL_SCANCODE_PERIOD: SDL_Scancode = 55;
pub const SDL_SCANCODE_COMMA: SDL_Scancode = 54;
pub const SDL_SCANCODE_GRAVE: SDL_Scancode = 53;
pub const SDL_SCANCODE_APOSTROPHE: SDL_Scancode = 52;
pub const SDL_SCANCODE_SEMICOLON: SDL_Scancode = 51;
pub const SDL_SCANCODE_NONUSHASH: SDL_Scancode = 50;
pub const SDL_SCANCODE_BACKSLASH: SDL_Scancode = 49;
pub const SDL_SCANCODE_RIGHTBRACKET: SDL_Scancode = 48;
pub const SDL_SCANCODE_LEFTBRACKET: SDL_Scancode = 47;
pub const SDL_SCANCODE_EQUALS: SDL_Scancode = 46;
pub const SDL_SCANCODE_MINUS: SDL_Scancode = 45;
pub const SDL_SCANCODE_SPACE: SDL_Scancode = 44;
pub const SDL_SCANCODE_TAB: SDL_Scancode = 43;
pub const SDL_SCANCODE_BACKSPACE: SDL_Scancode = 42;
pub const SDL_SCANCODE_ESCAPE: SDL_Scancode = 41;
pub const SDL_SCANCODE_RETURN: SDL_Scancode = 40;
pub const SDL_SCANCODE_0: SDL_Scancode = 39;
pub const SDL_SCANCODE_9: SDL_Scancode = 38;
pub const SDL_SCANCODE_8: SDL_Scancode = 37;
pub const SDL_SCANCODE_7: SDL_Scancode = 36;
pub const SDL_SCANCODE_6: SDL_Scancode = 35;
pub const SDL_SCANCODE_5: SDL_Scancode = 34;
pub const SDL_SCANCODE_4: SDL_Scancode = 33;
pub const SDL_SCANCODE_3: SDL_Scancode = 32;
pub const SDL_SCANCODE_2: SDL_Scancode = 31;
pub const SDL_SCANCODE_1: SDL_Scancode = 30;
pub const SDL_SCANCODE_Z: SDL_Scancode = 29;
pub const SDL_SCANCODE_Y: SDL_Scancode = 28;
pub const SDL_SCANCODE_X: SDL_Scancode = 27;
pub const SDL_SCANCODE_W: SDL_Scancode = 26;
pub const SDL_SCANCODE_V: SDL_Scancode = 25;
pub const SDL_SCANCODE_U: SDL_Scancode = 24;
pub const SDL_SCANCODE_T: SDL_Scancode = 23;
pub const SDL_SCANCODE_S: SDL_Scancode = 22;
pub const SDL_SCANCODE_R: SDL_Scancode = 21;
pub const SDL_SCANCODE_Q: SDL_Scancode = 20;
pub const SDL_SCANCODE_P: SDL_Scancode = 19;
pub const SDL_SCANCODE_O: SDL_Scancode = 18;
pub const SDL_SCANCODE_N: SDL_Scancode = 17;
pub const SDL_SCANCODE_M: SDL_Scancode = 16;
pub const SDL_SCANCODE_L: SDL_Scancode = 15;
pub const SDL_SCANCODE_K: SDL_Scancode = 14;
pub const SDL_SCANCODE_J: SDL_Scancode = 13;
pub const SDL_SCANCODE_I: SDL_Scancode = 12;
pub const SDL_SCANCODE_H: SDL_Scancode = 11;
pub const SDL_SCANCODE_G: SDL_Scancode = 10;
pub const SDL_SCANCODE_F: SDL_Scancode = 9;
pub const SDL_SCANCODE_E: SDL_Scancode = 8;
pub const SDL_SCANCODE_D: SDL_Scancode = 7;
pub const SDL_SCANCODE_C: SDL_Scancode = 6;
pub const SDL_SCANCODE_B: SDL_Scancode = 5;
pub const SDL_SCANCODE_A: SDL_Scancode = 4;
pub const SDL_SCANCODE_UNKNOWN: SDL_Scancode = 0;
pub use sm::types::{SDL_Keycode, SDL_Keymod};
pub const KMOD_RESERVED: SDL_Keymod = 32768;
pub const KMOD_GUI: SDL_Keymod = 3072;
pub const KMOD_ALT: SDL_Keymod = 768;
pub const KMOD_SHIFT: SDL_Keymod = 3;
pub const KMOD_CTRL: SDL_Keymod = 192;
pub const KMOD_SCROLL: SDL_Keymod = 32768;
pub const KMOD_MODE: SDL_Keymod = 16384;
pub const KMOD_CAPS: SDL_Keymod = 8192;
pub const KMOD_NUM: SDL_Keymod = 4096;
pub const KMOD_RGUI: SDL_Keymod = 2048;
pub const KMOD_LGUI: SDL_Keymod = 1024;
pub const KMOD_RALT: SDL_Keymod = 512;
pub const KMOD_LALT: SDL_Keymod = 256;
pub const KMOD_RCTRL: SDL_Keymod = 128;
pub const KMOD_LCTRL: SDL_Keymod = 64;
pub const KMOD_RSHIFT: SDL_Keymod = 2;
pub const KMOD_LSHIFT: SDL_Keymod = 1;
pub const KMOD_NONE: SDL_Keymod = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_Keysym {
    pub scancode: SDL_Scancode,
    pub sym: SDL_Keycode,
    pub mod_0: Uint16,
    pub unused: Uint32,
}
pub use sm::types::{SDL_JoystickID, SDL_JoystickPowerLevel};
pub const SDL_JOYSTICK_POWER_MAX: SDL_JoystickPowerLevel = 5;
pub const SDL_JOYSTICK_POWER_WIRED: SDL_JoystickPowerLevel = 4;
pub const SDL_JOYSTICK_POWER_FULL: SDL_JoystickPowerLevel = 3;
pub const SDL_JOYSTICK_POWER_MEDIUM: SDL_JoystickPowerLevel = 2;
pub const SDL_JOYSTICK_POWER_LOW: SDL_JoystickPowerLevel = 1;
pub const SDL_JOYSTICK_POWER_EMPTY: SDL_JoystickPowerLevel = 0;
pub const SDL_JOYSTICK_POWER_UNKNOWN: SDL_JoystickPowerLevel = -1;
pub type SDL_GameController = _SDL_GameController;
pub type C2RustUnnamed_1 = ::core::ffi::c_int;
pub const SDL_CONTROLLER_AXIS_MAX: C2RustUnnamed_1 = 6;
pub const SDL_CONTROLLER_AXIS_TRIGGERRIGHT: C2RustUnnamed_1 = 5;
pub const SDL_CONTROLLER_AXIS_TRIGGERLEFT: C2RustUnnamed_1 = 4;
pub const SDL_CONTROLLER_AXIS_RIGHTY: C2RustUnnamed_1 = 3;
pub const SDL_CONTROLLER_AXIS_RIGHTX: C2RustUnnamed_1 = 2;
pub const SDL_CONTROLLER_AXIS_LEFTY: C2RustUnnamed_1 = 1;
pub const SDL_CONTROLLER_AXIS_LEFTX: C2RustUnnamed_1 = 0;
pub const SDL_CONTROLLER_AXIS_INVALID: C2RustUnnamed_1 = -1;
pub type C2RustUnnamed_2 = ::core::ffi::c_int;
pub const SDL_CONTROLLER_BUTTON_MAX: C2RustUnnamed_2 = 21;
pub const SDL_CONTROLLER_BUTTON_TOUCHPAD: C2RustUnnamed_2 = 20;
pub const SDL_CONTROLLER_BUTTON_PADDLE4: C2RustUnnamed_2 = 19;
pub const SDL_CONTROLLER_BUTTON_PADDLE3: C2RustUnnamed_2 = 18;
pub const SDL_CONTROLLER_BUTTON_PADDLE2: C2RustUnnamed_2 = 17;
pub const SDL_CONTROLLER_BUTTON_PADDLE1: C2RustUnnamed_2 = 16;
pub const SDL_CONTROLLER_BUTTON_MISC1: C2RustUnnamed_2 = 15;
pub const SDL_CONTROLLER_BUTTON_DPAD_RIGHT: C2RustUnnamed_2 = 14;
pub const SDL_CONTROLLER_BUTTON_DPAD_LEFT: C2RustUnnamed_2 = 13;
pub const SDL_CONTROLLER_BUTTON_DPAD_DOWN: C2RustUnnamed_2 = 12;
pub const SDL_CONTROLLER_BUTTON_DPAD_UP: C2RustUnnamed_2 = 11;
pub const SDL_CONTROLLER_BUTTON_RIGHTSHOULDER: C2RustUnnamed_2 = 10;
pub const SDL_CONTROLLER_BUTTON_LEFTSHOULDER: C2RustUnnamed_2 = 9;
pub const SDL_CONTROLLER_BUTTON_RIGHTSTICK: C2RustUnnamed_2 = 8;
pub const SDL_CONTROLLER_BUTTON_LEFTSTICK: C2RustUnnamed_2 = 7;
pub const SDL_CONTROLLER_BUTTON_START: C2RustUnnamed_2 = 6;
pub const SDL_CONTROLLER_BUTTON_GUIDE: C2RustUnnamed_2 = 5;
pub const SDL_CONTROLLER_BUTTON_BACK: C2RustUnnamed_2 = 4;
pub const SDL_CONTROLLER_BUTTON_Y: C2RustUnnamed_2 = 3;
pub const SDL_CONTROLLER_BUTTON_X: C2RustUnnamed_2 = 2;
pub const SDL_CONTROLLER_BUTTON_B: C2RustUnnamed_2 = 1;
pub const SDL_CONTROLLER_BUTTON_A: C2RustUnnamed_2 = 0;
pub const SDL_CONTROLLER_BUTTON_INVALID: C2RustUnnamed_2 = -1;
pub use sm::types::{SDL_FingerID, SDL_GestureID, SDL_TouchID};
pub type C2RustUnnamed_3 = ::core::ffi::c_uint;
pub const SDL_LASTEVENT: C2RustUnnamed_3 = 65535;
pub const SDL_USEREVENT: C2RustUnnamed_3 = 32768;
pub const SDL_POLLSENTINEL: C2RustUnnamed_3 = 32512;
pub const SDL_RENDER_DEVICE_RESET: C2RustUnnamed_3 = 8193;
pub const SDL_RENDER_TARGETS_RESET: C2RustUnnamed_3 = 8192;
pub const SDL_SENSORUPDATE: C2RustUnnamed_3 = 4608;
pub const SDL_AUDIODEVICEREMOVED: C2RustUnnamed_3 = 4353;
pub const SDL_AUDIODEVICEADDED: C2RustUnnamed_3 = 4352;
pub const SDL_DROPCOMPLETE: C2RustUnnamed_3 = 4099;
pub const SDL_DROPBEGIN: C2RustUnnamed_3 = 4098;
pub const SDL_DROPTEXT: C2RustUnnamed_3 = 4097;
pub const SDL_DROPFILE: C2RustUnnamed_3 = 4096;
pub const SDL_CLIPBOARDUPDATE: C2RustUnnamed_3 = 2304;
pub const SDL_MULTIGESTURE: C2RustUnnamed_3 = 2050;
pub const SDL_DOLLARRECORD: C2RustUnnamed_3 = 2049;
pub const SDL_DOLLARGESTURE: C2RustUnnamed_3 = 2048;
pub const SDL_FINGERMOTION: C2RustUnnamed_3 = 1794;
pub const SDL_FINGERUP: C2RustUnnamed_3 = 1793;
pub const SDL_FINGERDOWN: C2RustUnnamed_3 = 1792;
pub const SDL_CONTROLLERSTEAMHANDLEUPDATED: C2RustUnnamed_3 = 1627;
pub const SDL_CONTROLLERUPDATECOMPLETE_RESERVED_FOR_SDL3: C2RustUnnamed_3 = 1626;
pub const SDL_CONTROLLERSENSORUPDATE: C2RustUnnamed_3 = 1625;
pub const SDL_CONTROLLERTOUCHPADUP: C2RustUnnamed_3 = 1624;
pub const SDL_CONTROLLERTOUCHPADMOTION: C2RustUnnamed_3 = 1623;
pub const SDL_CONTROLLERTOUCHPADDOWN: C2RustUnnamed_3 = 1622;
pub const SDL_CONTROLLERDEVICEREMAPPED: C2RustUnnamed_3 = 1621;
pub const SDL_CONTROLLERDEVICEREMOVED: C2RustUnnamed_3 = 1620;
pub const SDL_CONTROLLERDEVICEADDED: C2RustUnnamed_3 = 1619;
pub const SDL_CONTROLLERBUTTONUP: C2RustUnnamed_3 = 1618;
pub const SDL_CONTROLLERBUTTONDOWN: C2RustUnnamed_3 = 1617;
pub const SDL_CONTROLLERAXISMOTION: C2RustUnnamed_3 = 1616;
pub const SDL_JOYBATTERYUPDATED: C2RustUnnamed_3 = 1543;
pub const SDL_JOYDEVICEREMOVED: C2RustUnnamed_3 = 1542;
pub const SDL_JOYDEVICEADDED: C2RustUnnamed_3 = 1541;
pub const SDL_JOYBUTTONUP: C2RustUnnamed_3 = 1540;
pub const SDL_JOYBUTTONDOWN: C2RustUnnamed_3 = 1539;
pub const SDL_JOYHATMOTION: C2RustUnnamed_3 = 1538;
pub const SDL_JOYBALLMOTION: C2RustUnnamed_3 = 1537;
pub const SDL_JOYAXISMOTION: C2RustUnnamed_3 = 1536;
pub const SDL_MOUSEWHEEL: C2RustUnnamed_3 = 1027;
pub const SDL_MOUSEBUTTONUP: C2RustUnnamed_3 = 1026;
pub const SDL_MOUSEBUTTONDOWN: C2RustUnnamed_3 = 1025;
pub const SDL_MOUSEMOTION: C2RustUnnamed_3 = 1024;
pub const SDL_TEXTEDITING_EXT: C2RustUnnamed_3 = 773;
pub const SDL_KEYMAPCHANGED: C2RustUnnamed_3 = 772;
pub const SDL_TEXTINPUT: C2RustUnnamed_3 = 771;
pub const SDL_TEXTEDITING: C2RustUnnamed_3 = 770;
pub const SDL_KEYUP: C2RustUnnamed_3 = 769;
pub const SDL_KEYDOWN: C2RustUnnamed_3 = 768;
pub const SDL_SYSWMEVENT: C2RustUnnamed_3 = 513;
pub const SDL_WINDOWEVENT: C2RustUnnamed_3 = 512;
pub const SDL_DISPLAYEVENT: C2RustUnnamed_3 = 336;
pub const SDL_LOCALECHANGED: C2RustUnnamed_3 = 263;
pub const SDL_APP_DIDENTERFOREGROUND: C2RustUnnamed_3 = 262;
pub const SDL_APP_WILLENTERFOREGROUND: C2RustUnnamed_3 = 261;
pub const SDL_APP_DIDENTERBACKGROUND: C2RustUnnamed_3 = 260;
pub const SDL_APP_WILLENTERBACKGROUND: C2RustUnnamed_3 = 259;
pub const SDL_APP_LOWMEMORY: C2RustUnnamed_3 = 258;
pub const SDL_APP_TERMINATING: C2RustUnnamed_3 = 257;
pub const SDL_QUIT: C2RustUnnamed_3 = 256;
pub const SDL_FIRSTEVENT: C2RustUnnamed_3 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_CommonEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_DisplayEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub display: Uint32,
    pub event: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub data1: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_WindowEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub event: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub data1: Sint32,
    pub data2: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_KeyboardEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub state: Uint8,
    pub repeat: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub keysym: SDL_Keysym,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_TextEditingEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub text: [::core::ffi::c_char; 32],
    pub start: Sint32,
    pub length: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_TextEditingExtEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub text: *mut ::core::ffi::c_char,
    pub start: Sint32,
    pub length: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_TextInputEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub text: [::core::ffi::c_char; 32],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_MouseMotionEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub which: Uint32,
    pub state: Uint32,
    pub x: Sint32,
    pub y: Sint32,
    pub xrel: Sint32,
    pub yrel: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_MouseButtonEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub which: Uint32,
    pub button: Uint8,
    pub state: Uint8,
    pub clicks: Uint8,
    pub padding1: Uint8,
    pub x: Sint32,
    pub y: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_MouseWheelEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub which: Uint32,
    pub x: Sint32,
    pub y: Sint32,
    pub direction: Uint32,
    pub preciseX: ::core::ffi::c_float,
    pub preciseY: ::core::ffi::c_float,
    pub mouseX: Sint32,
    pub mouseY: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyAxisEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub axis: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub value: Sint16,
    pub padding4: Uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyBallEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub ball: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub xrel: Sint16,
    pub yrel: Sint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyHatEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub hat: Uint8,
    pub value: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyButtonEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub button: Uint8,
    pub state: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyDeviceEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyBatteryEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub level: SDL_JoystickPowerLevel,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_ControllerAxisEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub axis: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub value: Sint16,
    pub padding4: Uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_ControllerButtonEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub button: Uint8,
    pub state: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_ControllerDeviceEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_ControllerTouchpadEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub touchpad: Sint32,
    pub finger: Sint32,
    pub x: ::core::ffi::c_float,
    pub y: ::core::ffi::c_float,
    pub pressure: ::core::ffi::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_ControllerSensorEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub sensor: Sint32,
    pub data: [::core::ffi::c_float; 3],
    pub timestamp_us: Uint64,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_AudioDeviceEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: Uint32,
    pub iscapture: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_TouchFingerEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub touchId: SDL_TouchID,
    pub fingerId: SDL_FingerID,
    pub x: ::core::ffi::c_float,
    pub y: ::core::ffi::c_float,
    pub dx: ::core::ffi::c_float,
    pub dy: ::core::ffi::c_float,
    pub pressure: ::core::ffi::c_float,
    pub windowID: Uint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_MultiGestureEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub touchId: SDL_TouchID,
    pub dTheta: ::core::ffi::c_float,
    pub dDist: ::core::ffi::c_float,
    pub x: ::core::ffi::c_float,
    pub y: ::core::ffi::c_float,
    pub numFingers: Uint16,
    pub padding: Uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_DollarGestureEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub touchId: SDL_TouchID,
    pub gestureId: SDL_GestureID,
    pub numFingers: Uint32,
    pub error: ::core::ffi::c_float,
    pub x: ::core::ffi::c_float,
    pub y: ::core::ffi::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_DropEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub file: *mut ::core::ffi::c_char,
    pub windowID: Uint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_SensorEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: Sint32,
    pub data: [::core::ffi::c_float; 6],
    pub timestamp_us: Uint64,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_QuitEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_UserEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub code: Sint32,
    pub data1: *mut ::core::ffi::c_void,
    pub data2: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_SysWMEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub msg: *mut SDL_SysWMmsg,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union SDL_Event {
    pub type_0: Uint32,
    pub common: SDL_CommonEvent,
    pub display: SDL_DisplayEvent,
    pub window: SDL_WindowEvent,
    pub key: SDL_KeyboardEvent,
    pub edit: SDL_TextEditingEvent,
    pub editExt: SDL_TextEditingExtEvent,
    pub text: SDL_TextInputEvent,
    pub motion: SDL_MouseMotionEvent,
    pub button: SDL_MouseButtonEvent,
    pub wheel: SDL_MouseWheelEvent,
    pub jaxis: SDL_JoyAxisEvent,
    pub jball: SDL_JoyBallEvent,
    pub jhat: SDL_JoyHatEvent,
    pub jbutton: SDL_JoyButtonEvent,
    pub jdevice: SDL_JoyDeviceEvent,
    pub jbattery: SDL_JoyBatteryEvent,
    pub caxis: SDL_ControllerAxisEvent,
    pub cbutton: SDL_ControllerButtonEvent,
    pub cdevice: SDL_ControllerDeviceEvent,
    pub ctouchpad: SDL_ControllerTouchpadEvent,
    pub csensor: SDL_ControllerSensorEvent,
    pub adevice: SDL_AudioDeviceEvent,
    pub sensor: SDL_SensorEvent,
    pub quit: SDL_QuitEvent,
    pub user: SDL_UserEvent,
    pub syswm: SDL_SysWMEvent,
    pub tfinger: SDL_TouchFingerEvent,
    pub mgesture: SDL_MultiGestureEvent,
    pub dgesture: SDL_DollarGestureEvent,
    pub drop: SDL_DropEvent,
    pub padding: [Uint8; 56],
}
pub type C2RustUnnamed_4 = ::core::ffi::c_uint;
pub const SDL_MESSAGEBOX_BUTTONS_RIGHT_TO_LEFT: C2RustUnnamed_4 = 256;
pub const SDL_MESSAGEBOX_BUTTONS_LEFT_TO_RIGHT: C2RustUnnamed_4 = 128;
pub const SDL_MESSAGEBOX_INFORMATION: C2RustUnnamed_4 = 64;
pub const SDL_MESSAGEBOX_WARNING: C2RustUnnamed_4 = 32;
pub const SDL_MESSAGEBOX_ERROR: C2RustUnnamed_4 = 16;
pub type C2RustUnnamed_5 = ::core::ffi::c_uint;
pub const SDL_RENDERER_TARGETTEXTURE: C2RustUnnamed_5 = 8;
pub const SDL_RENDERER_PRESENTVSYNC: C2RustUnnamed_5 = 4;
pub const SDL_RENDERER_ACCELERATED: C2RustUnnamed_5 = 2;
pub const SDL_RENDERER_SOFTWARE: C2RustUnnamed_5 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_RendererInfo {
    pub name: *const ::core::ffi::c_char,
    pub flags: Uint32,
    pub num_texture_formats: Uint32,
    pub texture_formats: [Uint32; 16],
    pub max_texture_width: ::core::ffi::c_int,
    pub max_texture_height: ::core::ffi::c_int,
}
pub type C2RustUnnamed_6 = ::core::ffi::c_uint;
pub const SDL_TEXTUREACCESS_TARGET: C2RustUnnamed_6 = 2;
pub const SDL_TEXTUREACCESS_STREAMING: C2RustUnnamed_6 = 1;
pub const SDL_TEXTUREACCESS_STATIC: C2RustUnnamed_6 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ppu {
    pub snes: *mut Snes,
    pub vram: [uint16_t; 32768],
    pub vramPointer: uint16_t,
    pub vramIncrementOnHigh: bool,
    pub vramIncrement: uint16_t,
    pub vramRemapMode: uint8_t,
    pub vramReadBuffer: uint16_t,
    pub cgram: [uint16_t; 256],
    pub cgramPointer: uint8_t,
    pub cgramSecondWrite: bool,
    pub cgramBuffer: uint8_t,
    pub oam: [uint16_t; 256],
    pub highOam: [uint8_t; 32],
    pub oamAdr: uint8_t,
    pub oamAdrWritten: uint8_t,
    pub oamInHigh: bool,
    pub oamInHighWritten: bool,
    pub oamSecondWrite: bool,
    pub oamBuffer: uint8_t,
    pub objPriority: bool,
    pub objTileAdr1: uint16_t,
    pub objTileAdr2: uint16_t,
    pub objSize: uint8_t,
    pub objPixelBufferXX: [uint8_t; 256],
    pub objPriorityBufferXX: [uint8_t; 256],
    pub timeOver: bool,
    pub rangeOver: bool,
    pub objInterlace: bool,
    pub bgLayer: [BgLayer; 4],
    pub scrollPrev: uint8_t,
    pub scrollPrev2: uint8_t,
    pub mosaicSize: uint8_t,
    pub mosaicStartLine: uint8_t,
    pub layer: [Layer; 5],
    pub m7matrix: [int16_t; 8],
    pub m7prev: uint8_t,
    pub m7largeField: bool,
    pub m7charFill: bool,
    pub m7xFlip: bool,
    pub m7yFlip: bool,
    pub m7extBg: bool,
    pub m7startX: int32_t,
    pub m7startY: int32_t,
    pub windowLayer: [WindowLayer; 6],
    pub window1left: uint8_t,
    pub window1right: uint8_t,
    pub window2left: uint8_t,
    pub window2right: uint8_t,
    pub clipMode: uint8_t,
    pub preventMathMode: uint8_t,
    pub addSubscreen: bool,
    pub subtractColor: bool,
    pub halfColor: bool,
    pub mathEnabled: [bool; 6],
    pub fixedColorR: uint8_t,
    pub fixedColorG: uint8_t,
    pub fixedColorB: uint8_t,
    pub forcedBlank: bool,
    pub brightness: uint8_t,
    pub mode: uint8_t,
    pub bg3priority: bool,
    pub evenFrame: bool,
    pub pseudoHires: bool,
    pub overscan: bool,
    pub frameOverscan: bool,
    pub interlace: bool,
    pub frameInterlace: bool,
    pub directColor: bool,
    pub hCount: uint16_t,
    pub vCount: uint16_t,
    pub hCountSecond: bool,
    pub vCountSecond: bool,
    pub countersLatched: bool,
    pub ppu1openBus: uint8_t,
    pub ppu2openBus: uint8_t,
    pub pixelbuffer_placeholder: uint8_t,
    pub windowsel: uint32_t,
    pub extraLeftCur: uint8_t,
    pub extraRightCur: uint8_t,
    pub extraLeftRight: uint8_t,
    pub screenEnabled: [uint8_t; 2],
    pub screenWindowed: [uint8_t; 2],
    pub mosaicEnabled: uint8_t,
    pub lastBrightnessMult: uint8_t,
    pub lineHasSprites: bool,
    pub bgBuffers: [PpuPixelPrioBufs; 2],
    pub objBuffer: PpuPixelPrioBufs,
    pub renderPitch: uint32_t,
    pub renderBuffer: *mut uint8_t,
    pub brightnessMult: [uint8_t; 63],
    pub brightnessMultHalf: [uint8_t; 64],
    pub mosaicModulo: [uint8_t; 256],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PpuPixelPrioBufs {
    pub data: [PpuZbufType; 256],
}
pub use sm::types::PpuZbufType;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WindowLayer {
    pub window1enabled: bool,
    pub window2enabled: bool,
    pub window1inversed: bool,
    pub window2inversed: bool,
    pub maskLogic: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Layer {
    pub mainScreenEnabled: bool,
    pub subScreenEnabled: bool,
    pub mainScreenWindowed: bool,
    pub subScreenWindowed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BgLayer {
    pub hScroll: uint16_t,
    pub vScroll: uint16_t,
    pub tilemapWider: bool,
    pub tilemapHigher: bool,
    pub tilemapAdr: uint16_t,
    pub tileAdr: uint16_t,
    pub bigTiles: bool,
    pub mosaicEnabled: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Snes {
    pub cpu: *mut Cpu,
    pub apu: *mut Apu,
    pub ppu: *mut Ppu,
    pub snes_ppu: *mut Ppu,
    pub my_ppu: *mut Ppu,
    pub dma: *mut Dma,
    pub cart: *mut Cart,
    pub input1: *mut Input,
    pub input2: *mut Input,
    pub debug_cycles: bool,
    pub debug_apu_cycles: bool,
    pub disableRender: bool,
    pub runningWhichVersion: uint8_t,
    pub ramAdr: uint32_t,
    pub ram: *mut uint8_t,
    pub padx: [uint8_t; 4],
    pub hPos: uint16_t,
    pub vPos: uint16_t,
    pub frames: uint32_t,
    pub cpuCyclesLeft: uint8_t,
    pub cpuMemOps: uint8_t,
    pub padpad: [uint8_t; 2],
    pub apuCatchupCycles: ::core::ffi::c_double,
    pub hIrqEnabled: bool,
    pub vIrqEnabled: bool,
    pub nmiEnabled: bool,
    pub hTimer: uint16_t,
    pub vTimer: uint16_t,
    pub inNmi: bool,
    pub inIrq: bool,
    pub inVblank: bool,
    pub portAutoReadX: [uint16_t; 4],
    pub autoJoyRead: bool,
    pub autoJoyTimer: uint16_t,
    pub ppuLatch: bool,
    pub multiplyA: uint8_t,
    pub multiplyResult: uint16_t,
    pub divideA: uint16_t,
    pub divideResult: uint16_t,
    pub fastMem: bool,
    pub openBus: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Input {
    pub snes: *mut Snes,
    pub type_0: uint8_t,
    pub currentState: uint16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cart {
    pub snes: *mut Snes,
    pub type_0: uint8_t,
    pub rom: *mut uint8_t,
    pub romSize: uint32_t,
    pub ram: *mut uint8_t,
    pub ramSize: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dma {
    pub snes: *mut Snes,
    pub channel: [DmaChannel; 8],
    pub hdmaTimer: uint16_t,
    pub dmaTimer: uint32_t,
    pub dmaBusy: bool,
    pub pad: [uint8_t; 7],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DmaChannel {
    pub bAdr: uint8_t,
    pub aAdr: uint16_t,
    pub aBank: uint8_t,
    pub size: uint16_t,
    pub indBank: uint8_t,
    pub tableAdr: uint16_t,
    pub repCount: uint8_t,
    pub unusedByte: uint8_t,
    pub dmaActive: bool,
    pub hdmaActive: bool,
    pub mode: uint8_t,
    pub fixed: bool,
    pub decrement: bool,
    pub indirect: bool,
    pub fromB: bool,
    pub unusedBit: bool,
    pub doTransfer: bool,
    pub terminated: bool,
    pub offIndex: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Apu {
    pub spc: *mut Spc,
    pub dsp: *mut Dsp,
    pub ram: [uint8_t; 65536],
    pub romReadable: bool,
    pub dspAdr: uint8_t,
    pub cycles: uint32_t,
    pub inPorts: [uint8_t; 6],
    pub outPorts: [uint8_t; 4],
    pub timer: [Timer; 3],
    pub cpuCyclesLeft: uint8_t,
    pub pad: [uint8_t; 6],
    pub c2rust_unnamed: C2RustUnnamed_7,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_7 {
    pub hist: DspRegWriteHistory,
    pub padpad: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DspRegWriteHistory {
    pub count: uint32_t,
    pub addr: [uint8_t; 256],
    pub val: [uint8_t; 256],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub cycles: uint8_t,
    pub divider: uint8_t,
    pub target: uint8_t,
    pub counter: uint8_t,
    pub enabled: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dsp {
    pub apu_ram: *mut uint8_t,
    pub ram: [uint8_t; 128],
    pub channel: [DspChannel; 8],
    pub dirPage: uint16_t,
    pub evenCycle: bool,
    pub mute: bool,
    pub reset: bool,
    pub masterVolumeL: int8_t,
    pub masterVolumeR: int8_t,
    pub noiseSample: int16_t,
    pub noiseRate: uint16_t,
    pub noiseCounter: uint16_t,
    pub echoWrites: bool,
    pub echoVolumeL: int8_t,
    pub echoVolumeR: int8_t,
    pub feedbackVolume: int8_t,
    pub echoBufferAdr: uint16_t,
    pub echoDelay: uint16_t,
    pub echoRemain: uint16_t,
    pub echoBufferIndex: uint16_t,
    pub firBufferIndex: uint8_t,
    pub firValues: [int8_t; 8],
    pub firBufferL: [int16_t; 8],
    pub firBufferR: [int16_t; 8],
    pub sampleBuffer: [int16_t; 1068],
    pub sampleOffset: uint16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DspChannel {
    pub pitch: uint16_t,
    pub pitchCounter: uint16_t,
    pub pitchModulation: bool,
    pub decodeBuffer: [int16_t; 19],
    pub srcn: uint8_t,
    pub decodeOffset: uint16_t,
    pub previousFlags: uint8_t,
    pub old: int16_t,
    pub older: int16_t,
    pub useNoise: bool,
    pub adsrRates: [uint16_t; 4],
    pub rateCounter: uint16_t,
    pub adsrState: uint8_t,
    pub sustainLevel: uint16_t,
    pub useGain: bool,
    pub gainMode: uint8_t,
    pub directGain: bool,
    pub gainValue: uint16_t,
    pub gain: uint16_t,
    pub keyOn: bool,
    pub keyOff: bool,
    pub sampleOut: int16_t,
    pub volumeL: int8_t,
    pub volumeR: int8_t,
    pub echoEnable: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Spc {
    pub apu: *mut Apu,
    pub a: uint8_t,
    pub x: uint8_t,
    pub y: uint8_t,
    pub sp: uint8_t,
    pub pc: uint16_t,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub h: bool,
    pub p: bool,
    pub b: bool,
    pub stopped: bool,
    pub cyclesUsed: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cpu {
    pub mem: *mut ::core::ffi::c_void,
    pub memType: uint8_t,
    pub a: uint16_t,
    pub x: uint16_t,
    pub y: uint16_t,
    pub sp: uint16_t,
    pub pc: uint16_t,
    pub dp: uint16_t,
    pub k: uint8_t,
    pub db: uint8_t,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub d: bool,
    pub xf: bool,
    pub mf: bool,
    pub e: bool,
    pub irqWanted: bool,
    pub nmiWanted: bool,
    pub waiting: bool,
    pub stopped: bool,
    pub cyclesUsed: uint8_t,
    pub spBreakpoint: uint16_t,
    pub in_emu: bool,
}
pub type C2RustUnnamed_8 = ::core::ffi::c_uint;
pub const kPpuRenderFlags_NoSpriteLimits: C2RustUnnamed_8 = 8;
pub const kPpuRenderFlags_Height240: C2RustUnnamed_8 = 4;
pub const kPpuRenderFlags_4x4Mode7: C2RustUnnamed_8 = 2;
pub const kPpuRenderFlags_NewRenderer: C2RustUnnamed_8 = 1;
pub use sm::types::{int16, uint16, uint32, uint64, uint8};
pub type C2RustUnnamed_9 = ::core::ffi::c_uint;
pub const kSaveLoad_Replay: C2RustUnnamed_9 = 3;
pub const kSaveLoad_Load: C2RustUnnamed_9 = 2;
pub const kSaveLoad_Save: C2RustUnnamed_9 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SpcPlayer {
    pub reg_write_history: *mut DspRegWriteHistory,
    pub timer_cycles: uint8,
    pub dsp: *mut Dsp,
    pub port_to_snes: [uint8; 4],
    pub last_value_from_snes: [uint8; 4],
    pub counter_sf0c: uint16,
    pub always_zero: uint16,
    pub temp_accum: uint16,
    pub ttt: uint8,
    pub did_affect_volumepitch_flag: uint8,
    pub addr0: uint16,
    pub addr1: uint16,
    pub lfsr_value: uint16,
    pub is_chan_on: uint8,
    pub fast_forward: uint8,
    pub word_20: uint16,
    pub music_ptr_toplevel: uint16,
    pub block_count: uint8,
    pub sfx_timer_accum: uint8,
    pub chn: uint8,
    pub key_ON: uint8,
    pub key_OFF: uint8,
    pub cur_chan_bit: uint8,
    pub reg_FLG: uint8,
    pub reg_NON: uint8,
    pub reg_EON: uint8,
    pub echo_stored_time: uint8,
    pub echo_parameter_EDL: uint8,
    pub reg_EFB: uint8,
    pub global_transposition: uint8,
    pub main_tempo_accum: uint8,
    pub tempo: uint16,
    pub tempo_fade_num_ticks: uint8,
    pub tempo_fade_final: uint8,
    pub tempo_fade_add: uint16,
    pub master_volume: uint16,
    pub master_volume_fade_ticks: uint8,
    pub master_volume_fade_target: uint8,
    pub master_volume_fade_add_per_tick: uint16,
    pub vol_dirty: uint8,
    pub percussion_base_id: uint8,
    pub echo_volume_left: uint16,
    pub echo_volume_right: uint16,
    pub echo_volume_fade_add_left: uint16,
    pub echo_volume_fade_add_right: uint16,
    pub echo_volume_fade_ticks: uint8,
    pub echo_volume_fade_target_left: uint8,
    pub echo_volume_fade_target_right: uint8,
    pub p_echoBuffer: uint16,
    pub memset_ptr: uint16,
    pub memset_ctr: uint8,
    pub disable_sfx2: uint8,
    pub byte_4B1: uint8,
    pub channel: [Channel; 8],
    pub sfx1: SfxRegs,
    pub sfx2: SfxRegs,
    pub sfx3: SfxRegs,
    pub sfx_chans_1: [SfxChan; 4],
    pub sfx_chans_2: [SfxChan; 2],
    pub sfx_chans_3: [SfxChan; 2],
    pub last_written_edl: uint8,
    pub input_ports: [uint8; 4],
    pub ram: [uint8; 65536],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SfxChan {
    pub voice_bitset: uint8,
    pub instr_timer: uint8,
    pub subnote: uint8,
    pub repeat_pt: uint8,
    pub note: uint8,
    pub ptr_idx: uint8,
    pub voice_index: uint8,
    pub voice_index_mult8: uint8,
    pub release_timer: uint8,
    pub volume: uint8,
    pub target_note: uint8,
    pub phase_invert: uint8,
    pub ptr: uint16,
    pub repeat_ctr: uint8,
    pub enable_pitch_slide: uint8,
    pub release_flag: uint8,
    pub enable_pitch_slide_legato: uint8,
    pub adsr_settings: uint16,
    pub channel_mask: uint8,
    pub legato: uint8,
    pub disable: uint8,
    pub subnote_delta: uint8,
    pub update_adsr: uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SfxRegs {
    pub priority: uint8,
    pub chan_idx_x2: uint8,
    pub chan_voice_bitset_ptr: uint16,
    pub cur_sound: uint8,
    pub chan_voice_mask_ptr: uint16,
    pub chan_voice_index_ptr: uint16,
    pub some_value: uint8,
    pub cur_ptr: uint16,
    pub enabled_voices: uint8,
    pub voices_to_setup: uint8,
    pub chan_idx: uint8,
    pub voices_remaining: uint8,
    pub cur_sound_index: uint8,
    pub voice_id: uint8,
    pub init_flag: uint8,
    pub voice_index: uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Channel {
    pub pattern_order_ptr_for_chan: uint16,
    pub index: uint8,
    pub note_ticks_left: uint8,
    pub note_keyoff_ticks_left: uint8,
    pub subroutine_num_loops: uint8,
    pub volume_fade_ticks: uint8,
    pub pan_num_ticks: uint8,
    pub pitch_slide_length: uint8,
    pub pitch_slide_delay_left: uint8,
    pub vibrato_hold_count: uint8,
    pub vib_depth: uint8,
    pub tremolo_hold_count: uint8,
    pub tremolo_depth: uint8,
    pub vibrato_change_count: uint8,
    pub note_length: uint8,
    pub note_gate_off_fixedpt: uint8,
    pub channel_volume_master: uint8,
    pub instrument_id: uint8,
    pub instrument_pitch_base: uint16,
    pub saved_pattern_ptr: uint16,
    pub pattern_start_ptr: uint16,
    pub pitch_envelope_num_ticks: uint8,
    pub pitch_envelope_delay: uint8,
    pub pitch_envelope_direction: uint8,
    pub pitch_envelope_slide_value: uint8,
    pub vibrato_count: uint8,
    pub vibrato_rate: uint8,
    pub vibrato_delay_ticks: uint8,
    pub vibrato_fade_num_ticks: uint8,
    pub vibrato_fade_add_per_tick: uint8,
    pub vibrato_depth_target: uint8,
    pub tremolo_count: uint8,
    pub tremolo_rate: uint8,
    pub tremolo_delay_ticks: uint8,
    pub channel_transposition: uint8,
    pub channel_volume: uint16,
    pub volume_fade_addpertick: uint16,
    pub volume_fade_target: uint8,
    pub final_volume: uint8,
    pub pan_value: uint16,
    pub pan_add_per_tick: uint16,
    pub pan_target_value: uint8,
    pub pan_flag_with_phase_invert: uint8,
    pub pitch: uint16,
    pub pitch_add_per_tick: uint16,
    pub pitch_target: uint8,
    pub fine_tune: uint8,
    pub cutk: uint8,
}
pub type C2RustUnnamed_10 = ::core::ffi::c_uint;
pub const kKeys_Total: C2RustUnnamed_10 = 130;
pub const kKeys_VolumeDown: C2RustUnnamed_10 = 129;
pub const kKeys_VolumeUp: C2RustUnnamed_10 = 128;
pub const kKeys_ToggleRenderer: C2RustUnnamed_10 = 127;
pub const kKeys_DisplayPerf: C2RustUnnamed_10 = 126;
pub const kKeys_WindowSmaller: C2RustUnnamed_10 = 125;
pub const kKeys_WindowBigger: C2RustUnnamed_10 = 124;
pub const kKeys_ReplayTurbo: C2RustUnnamed_10 = 123;
pub const kKeys_Turbo: C2RustUnnamed_10 = 122;
pub const kKeys_PauseDimmed: C2RustUnnamed_10 = 121;
pub const kKeys_Pause: C2RustUnnamed_10 = 120;
pub const kKeys_Reset: C2RustUnnamed_10 = 119;
pub const kKeys_Fullscreen: C2RustUnnamed_10 = 118;
pub const kKeys_StopReplay: C2RustUnnamed_10 = 117;
pub const kKeys_ClearKeyLog: C2RustUnnamed_10 = 116;
pub const kKeys_ToggleWhichFrame: C2RustUnnamed_10 = 115;
pub const kKeys_CheatJump: C2RustUnnamed_10 = 114;
pub const kKeys_CheatLife: C2RustUnnamed_10 = 113;
pub const kKeys_ReplayRef_Last: C2RustUnnamed_10 = 112;
pub const kKeys_ReplayRef: C2RustUnnamed_10 = 93;
pub const kKeys_LoadRef_Last: C2RustUnnamed_10 = 92;
pub const kKeys_LoadRef: C2RustUnnamed_10 = 73;
pub const kKeys_Replay_Last: C2RustUnnamed_10 = 72;
pub const kKeys_Replay: C2RustUnnamed_10 = 53;
pub const kKeys_Save_Last: C2RustUnnamed_10 = 52;
pub const kKeys_Save: C2RustUnnamed_10 = 33;
pub const kKeys_Load_Last: C2RustUnnamed_10 = 32;
pub const kKeys_Load: C2RustUnnamed_10 = 13;
pub const kKeys_Controls_Last: C2RustUnnamed_10 = 12;
pub const kKeys_Controls: C2RustUnnamed_10 = 1;
pub const kKeys_Null: C2RustUnnamed_10 = 0;
pub type C2RustUnnamed_11 = ::core::ffi::c_uint;
pub const kOutputMethod_OpenGL: C2RustUnnamed_11 = 2;
pub const kOutputMethod_SDLSoftware: C2RustUnnamed_11 = 1;
pub const kOutputMethod_SDL: C2RustUnnamed_11 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Config {
    pub window_width: ::core::ffi::c_int,
    pub window_height: ::core::ffi::c_int,
    pub enhanced_mode7: bool,
    pub new_renderer: bool,
    pub ignore_aspect_ratio: bool,
    pub fullscreen: uint8,
    pub window_scale: uint8,
    pub enable_audio: bool,
    pub linear_filtering: bool,
    pub output_method: uint8,
    pub audio_freq: uint16,
    pub audio_channels: uint8,
    pub audio_samples: uint16,
    pub autosave: bool,
    pub extended_aspect_ratio: uint8,
    pub extend_y: bool,
    pub no_sprite_limits: bool,
    pub display_perf_title: bool,
    pub enable_msu: uint8,
    pub resume_msu: bool,
    pub disable_frame_delay: bool,
    pub msuvolume: uint8,
    pub features0: uint32,
    pub link_graphics: *const ::core::ffi::c_char,
    pub memory_buffer: *mut ::core::ffi::c_char,
    pub shader: *const ::core::ffi::c_char,
    pub msu_path: *const ::core::ffi::c_char,
}
pub type C2RustUnnamed_12 = ::core::ffi::c_int;
pub const kGamepadBtn_Count: C2RustUnnamed_12 = 17;
pub const kGamepadBtn_R2: C2RustUnnamed_12 = 16;
pub const kGamepadBtn_L2: C2RustUnnamed_12 = 15;
pub const kGamepadBtn_DpadRight: C2RustUnnamed_12 = 14;
pub const kGamepadBtn_DpadLeft: C2RustUnnamed_12 = 13;
pub const kGamepadBtn_DpadDown: C2RustUnnamed_12 = 12;
pub const kGamepadBtn_DpadUp: C2RustUnnamed_12 = 11;
pub const kGamepadBtn_R1: C2RustUnnamed_12 = 10;
pub const kGamepadBtn_L1: C2RustUnnamed_12 = 9;
pub const kGamepadBtn_R3: C2RustUnnamed_12 = 8;
pub const kGamepadBtn_L3: C2RustUnnamed_12 = 7;
pub const kGamepadBtn_Start: C2RustUnnamed_12 = 6;
pub const kGamepadBtn_Guide: C2RustUnnamed_12 = 5;
pub const kGamepadBtn_Back: C2RustUnnamed_12 = 4;
pub const kGamepadBtn_Y: C2RustUnnamed_12 = 3;
pub const kGamepadBtn_X: C2RustUnnamed_12 = 2;
pub const kGamepadBtn_B: C2RustUnnamed_12 = 1;
pub const kGamepadBtn_A: C2RustUnnamed_12 = 0;
pub const kGamepadBtn_Invalid: C2RustUnnamed_12 = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RendererFuncs {
    pub Initialize: Option<unsafe extern "C" fn(*mut SDL_Window) -> bool>,
    pub Destroy: Option<unsafe extern "C" fn() -> ()>,
    pub BeginDraw: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut *mut uint8,
            *mut ::core::ffi::c_int,
        ) -> (),
    >,
    pub EndDraw: Option<unsafe extern "C" fn() -> ()>,
}
pub const kMaxWindowScale: C2RustUnnamed_13 = 10;
pub type C2RustUnnamed_13 = ::core::ffi::c_uint;
pub const kDefaultSamples: C2RustUnnamed_13 = 2048;
pub const kDefaultChannels: C2RustUnnamed_13 = 2;
pub const kDefaultFreq: C2RustUnnamed_13 = 44100;
pub const kDefaultFullscreen: C2RustUnnamed_13 = 0;
pub const AUDIO_S16LSB: ::core::ffi::c_int = 0x8010 as ::core::ffi::c_int;
pub const AUDIO_S16: ::core::ffi::c_int = AUDIO_S16LSB;
pub const SDL_MIX_MAXVOLUME: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
pub const SDL_WINDOWPOS_UNDEFINED_MASK: ::core::ffi::c_uint = 0x1fff0000 as ::core::ffi::c_uint;
pub const SDL_WINDOWPOS_UNDEFINED: ::core::ffi::c_uint =
    SDL_WINDOWPOS_UNDEFINED_MASK | 0 as ::core::ffi::c_uint;
pub const SDL_WINDOWPOS_CENTERED_MASK: ::core::ffi::c_uint = 0x2fff0000 as ::core::ffi::c_uint;
pub const SDL_WINDOWPOS_CENTERED: ::core::ffi::c_uint =
    SDL_WINDOWPOS_CENTERED_MASK | 0 as ::core::ffi::c_uint;
pub const SDL_BUTTON_LEFT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SDL_PRESSED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SDL_HINT_RENDER_SCALE_QUALITY: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"SDL_RENDER_SCALE_QUALITY\0")
};
pub const SDL_INIT_AUDIO: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const SDL_INIT_VIDEO: ::core::ffi::c_uint = 0x20 as ::core::ffi::c_uint;
pub const SDL_INIT_GAMECONTROLLER: ::core::ffi::c_uint = 0x2000 as ::core::ffi::c_uint;
pub const kDebugFlag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn IntMin(
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn IntMax(
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return if a > b { a } else { b };
}
#[no_mangle]
pub static mut g_debug_flag: bool = false;
#[no_mangle]
pub static mut g_is_turbo: bool = false;
#[no_mangle]
pub static mut g_want_dump_memmap_flags: bool = false;
#[no_mangle]
pub static mut g_new_ppu: bool = true_0 != 0;
#[no_mangle]
pub static mut g_other_image: bool = false;
#[no_mangle]
pub static mut g_spc_player: *mut SpcPlayer = ::core::ptr::null::<SpcPlayer>() as *mut SpcPlayer;
static mut g_pixels: [uint8_t; 245760] = [0; 245760];
static mut g_my_pixels: [uint8_t; 245760] = [0; 245760];
#[no_mangle]
pub static mut g_got_mismatch_count: ::core::ffi::c_int = 0;
static mut kWindowTitle: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"SuperMet\0") };
static mut g_win_flags: uint32 = SDL_WINDOW_RESIZABLE as ::core::ffi::c_int as uint32;
static mut g_window: *mut SDL_Window = ::core::ptr::null::<SDL_Window>() as *mut SDL_Window;
static mut g_cursor: uint8 = true_0 as uint8;
static mut g_paused: uint8 = 0;
static mut g_turbo: uint8 = 0;
static mut g_replay_turbo: uint8 = true_0 as uint8;
static mut g_current_window_scale: uint8 = 0;
static mut g_gamepad_buttons: uint8 = 0;
static mut g_input1_state: ::core::ffi::c_int = 0;
static mut g_display_perf: bool = false;
static mut g_curr_fps: ::core::ffi::c_int = 0;
static mut g_ppu_render_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut g_snes_height: ::core::ffi::c_int = 0;
static mut g_snes_width: ::core::ffi::c_int = 0;
static mut g_sdl_audio_mixer_volume: ::core::ffi::c_int = SDL_MIX_MAXVOLUME;
static mut g_renderer_funcs: RendererFuncs = RendererFuncs {
    Initialize: None,
    Destroy: None,
    BeginDraw: None,
    EndDraw: None,
};
static mut g_gamepad_modifiers: uint32 = 0;
static mut g_gamepad_last_cmd: [uint16; 17] = [0; 17];
#[no_mangle]
pub unsafe extern "C" fn Die(mut error: *const ::core::ffi::c_char) {
    SDL_ShowSimpleMessageBox(
        SDL_MESSAGEBOX_ERROR as ::core::ffi::c_int as Uint32,
        &raw const kWindowTitle as *const ::core::ffi::c_char,
        error,
        ::core::ptr::null_mut::<SDL_Window>(),
    );
    fprintf(
        stderr,
        b"Error: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
        error,
    );
    exit(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn Warning(mut error: *const ::core::ffi::c_char) {
    SDL_ShowSimpleMessageBox(
        SDL_MESSAGEBOX_WARNING as ::core::ffi::c_int as Uint32,
        &raw const kWindowTitle as *const ::core::ffi::c_char,
        error,
        ::core::ptr::null_mut::<SDL_Window>(),
    );
    fprintf(
        stderr,
        b"Warning: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
        error,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ChangeWindowScale(mut scale_step: ::core::ffi::c_int) {
    if SDL_GetWindowFlags(g_window as *mut SDL_Window)
        & (SDL_WINDOW_FULLSCREEN_DESKTOP as ::core::ffi::c_int
            | SDL_WINDOW_FULLSCREEN as ::core::ffi::c_int
            | SDL_WINDOW_MINIMIZED as ::core::ffi::c_int
            | SDL_WINDOW_MAXIMIZED as ::core::ffi::c_int) as Uint32
        != 0 as Uint32
    {
        return;
    }
    let mut screen: ::core::ffi::c_int = SDL_GetWindowDisplayIndex(g_window as *mut SDL_Window);
    if screen < 0 as ::core::ffi::c_int {
        screen = 0 as ::core::ffi::c_int;
    }
    let mut max_scale: ::core::ffi::c_int = kMaxWindowScale as ::core::ffi::c_int;
    let mut bounds: SDL_Rect = SDL_Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };
    let mut bt: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut bl: ::core::ffi::c_int = 0;
    let mut bb: ::core::ffi::c_int = 0;
    let mut br: ::core::ffi::c_int = 0;
    if SDL_GetDisplayUsableBounds(screen, &raw mut bounds) == 0 as ::core::ffi::c_int {
        if SDL_GetWindowBordersSize(
            g_window as *mut SDL_Window,
            &raw mut bt,
            &raw mut bl,
            &raw mut bb,
            &raw mut br,
        ) != 0 as ::core::ffi::c_int
        {
            bb = 1 as ::core::ffi::c_int;
            br = bb;
            bl = br;
            bt = 31 as ::core::ffi::c_int;
        }
        let mut mw: ::core::ffi::c_int =
            (bounds.w - bl - br + g_snes_width / 4 as ::core::ffi::c_int) / g_snes_width;
        let mut mh: ::core::ffi::c_int =
            (bounds.h - bt - bb + g_snes_height / 4 as ::core::ffi::c_int) / g_snes_height;
        max_scale = IntMin(mw, mh);
    }
    let mut new_scale: ::core::ffi::c_int = IntMax(
        IntMin(
            g_current_window_scale as ::core::ffi::c_int + scale_step,
            max_scale,
        ),
        1 as ::core::ffi::c_int,
    );
    g_current_window_scale = new_scale as uint8;
    let mut w: ::core::ffi::c_int = new_scale * g_snes_width;
    let mut h: ::core::ffi::c_int = new_scale * g_snes_height;
    SDL_SetWindowSize(g_window as *mut SDL_Window, w, h);
    if bt >= 0 as ::core::ffi::c_int {
        let mut mx: ::core::ffi::c_int = 0;
        let mut my: ::core::ffi::c_int = 0;
        SDL_GetGlobalMouseState(&raw mut mx, &raw mut my);
        let mut wx: ::core::ffi::c_int = IntMax(
            IntMin(
                mx - w / 2 as ::core::ffi::c_int,
                bounds.x + bounds.w - bl - br - w,
            ),
            bounds.x + bl,
        );
        let mut wy: ::core::ffi::c_int = IntMax(
            IntMin(
                my - h / 2 as ::core::ffi::c_int,
                bounds.y + bounds.h - bt - bb - h,
            ),
            bounds.y + bt,
        );
        SDL_SetWindowPosition(g_window as *mut SDL_Window, wx, wy);
    } else {
        SDL_SetWindowPosition(
            g_window as *mut SDL_Window,
            SDL_WINDOWPOS_CENTERED as ::core::ffi::c_int,
            SDL_WINDOWPOS_CENTERED as ::core::ffi::c_int,
        );
    };
}
pub const RESIZE_BORDER: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
unsafe extern "C" fn HitTestCallback(
    mut win: *mut SDL_Window,
    mut pt: *const SDL_Point,
    mut data: *mut ::core::ffi::c_void,
) -> SDL_HitTestResult {
    let mut flags: uint32 = SDL_GetWindowFlags(win as *mut SDL_Window) as uint32;
    if flags & SDL_WINDOW_FULLSCREEN_DESKTOP as ::core::ffi::c_int as uint32 != 0 as uint32
        || flags & SDL_WINDOW_FULLSCREEN as ::core::ffi::c_int as uint32 != 0 as uint32
    {
        return SDL_HITTEST_NORMAL;
    }
    if SDL_GetModState() as ::core::ffi::c_uint
        & KMOD_CTRL as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint
    {
        return SDL_HITTEST_DRAGGABLE;
    }
    let mut w: ::core::ffi::c_int = 0;
    let mut h: ::core::ffi::c_int = 0;
    SDL_GetWindowSize(win as *mut SDL_Window, &raw mut w, &raw mut h);
    if (*pt).y < RESIZE_BORDER {
        return (if (*pt).x < RESIZE_BORDER {
            SDL_HITTEST_RESIZE_TOPLEFT as ::core::ffi::c_int
        } else if (*pt).x >= w - RESIZE_BORDER {
            SDL_HITTEST_RESIZE_TOPRIGHT as ::core::ffi::c_int
        } else {
            SDL_HITTEST_RESIZE_TOP as ::core::ffi::c_int
        }) as SDL_HitTestResult;
    } else if (*pt).y >= h - RESIZE_BORDER {
        return (if (*pt).x < RESIZE_BORDER {
            SDL_HITTEST_RESIZE_BOTTOMLEFT as ::core::ffi::c_int
        } else if (*pt).x >= w - RESIZE_BORDER {
            SDL_HITTEST_RESIZE_BOTTOMRIGHT as ::core::ffi::c_int
        } else {
            SDL_HITTEST_RESIZE_BOTTOM as ::core::ffi::c_int
        }) as SDL_HitTestResult;
    } else if (*pt).x < RESIZE_BORDER {
        return SDL_HITTEST_RESIZE_LEFT;
    } else if (*pt).x >= w - RESIZE_BORDER {
        return SDL_HITTEST_RESIZE_RIGHT;
    }
    return SDL_HITTEST_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn RtlDrawPpuFrame(
    mut pixel_buffer: *mut uint8,
    mut pitch: size_t,
    mut render_flags: uint32,
) {
    let mut ppu_pixels: *mut uint8 = if g_other_image as ::core::ffi::c_int != 0 {
        &raw mut g_my_pixels as *mut uint8
    } else {
        &raw mut g_pixels as *mut uint8
    };
    let mut y: size_t = 0 as size_t;
    while y < 240 as size_t {
        memcpy(
            (pixel_buffer as *mut uint8_t).offset(y.wrapping_mul(pitch) as isize)
                as *mut ::core::ffi::c_void,
            ppu_pixels.offset(y.wrapping_mul(256 as size_t).wrapping_mul(4 as size_t) as isize)
                as *const ::core::ffi::c_void,
            (256 as ::core::ffi::c_int * 4 as ::core::ffi::c_int) as size_t,
        );
        y = y.wrapping_add(1);
    }
}
unsafe extern "C" fn DrawPpuFrameWithPerf() {
    let mut render_scale: ::core::ffi::c_int =
        PpuGetCurrentRenderScale((*g_snes).ppu, g_ppu_render_flags as uint32_t);
    let mut pixel_buffer: *mut uint8 = ::core::ptr::null_mut::<uint8>();
    let mut pitch: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    g_renderer_funcs
        .BeginDraw
        .expect("non-null function pointer")(
        g_snes_width * render_scale,
        g_snes_height * render_scale,
        &raw mut pixel_buffer,
        &raw mut pitch,
    );
    if g_display_perf as ::core::ffi::c_int != 0
        || g_config.display_perf_title as ::core::ffi::c_int != 0
    {
        static mut history: [::core::ffi::c_float; 64] = [0.; 64];
        static mut average: ::core::ffi::c_float = 0.;
        static mut history_pos: ::core::ffi::c_int = 0;
        let mut before: uint64 = SDL_GetPerformanceCounter() as uint64;
        RtlDrawPpuFrame(pixel_buffer, pitch as size_t, g_ppu_render_flags as uint32);
        let mut after: uint64 = SDL_GetPerformanceCounter() as uint64;
        let mut v: ::core::ffi::c_float = (SDL_GetPerformanceFrequency() as ::core::ffi::c_double
            / after.wrapping_sub(before) as ::core::ffi::c_double)
            as ::core::ffi::c_float;
        average += v - history[history_pos as usize];
        history[history_pos as usize] = v;
        history_pos = history_pos + 1 as ::core::ffi::c_int & 63 as ::core::ffi::c_int;
        g_curr_fps = (average * (1.0f32 / 64 as ::core::ffi::c_int as ::core::ffi::c_float))
            as ::core::ffi::c_int;
    } else {
        RtlDrawPpuFrame(pixel_buffer, pitch as size_t, g_ppu_render_flags as uint32);
    }
    if g_display_perf {
        RenderNumber(
            pixel_buffer.offset((pitch * render_scale) as isize),
            pitch as size_t,
            g_curr_fps,
            (render_scale == 4 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8,
        );
    }
    if g_got_mismatch_count != 0 {
        RenderNumber(
            pixel_buffer.offset((pitch * render_scale) as isize),
            pitch as size_t,
            g_got_mismatch_count,
            (render_scale == 4 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8,
        );
    }
    ::core::mem::transmute::<_, fn()>(g_renderer_funcs.EndDraw.expect("non-null function pointer"))(
    );
}
static mut g_audio_mutex: *mut SDL_mutex = ::core::ptr::null::<SDL_mutex>() as *mut SDL_mutex;
static mut g_audiobuffer_cur: *mut uint8 = ::core::ptr::null::<uint8>() as *mut uint8;
static mut g_audiobuffer_end: *mut uint8 = ::core::ptr::null::<uint8>() as *mut uint8;
static mut g_audiobuffer: *mut uint8 = ::core::ptr::null::<uint8>() as *mut uint8;
static mut g_frames_per_block: ::core::ffi::c_int = 0;
static mut g_audio_channels: uint8 = 0;
static mut g_audio_device: SDL_AudioDeviceID = 0;
unsafe fn initialize_verify_audio() -> Result<(), String> {
    g_audio_mutex = SDL_CreateMutex();
    if g_audio_mutex.is_null() {
        return Err("unable to create audio mutex".into());
    }
    g_spc_player = SpcPlayer_Create();
    if g_spc_player.is_null() {
        SDL_DestroyMutex(g_audio_mutex);
        return Err("unable to initialize sound driver".into());
    }
    SpcPlayer_Initialize(g_spc_player);
    Ok(())
}

unsafe fn finish_verify_audio() {
    SDL_DestroyMutex(g_audio_mutex);
}

#[no_mangle]
pub unsafe extern "C" fn RtlApuLock() {
    SDL_LockMutex(g_audio_mutex);
}
#[no_mangle]
pub unsafe extern "C" fn RtlApuUnlock() {
    SDL_UnlockMutex(g_audio_mutex);
}
unsafe extern "C" fn AudioCallback(
    mut userdata: *mut ::core::ffi::c_void,
    mut stream: *mut Uint8,
    mut len: ::core::ffi::c_int,
) {
    if SDL_LockMutex(g_audio_mutex) != 0 {
        Die(b"Mutex lock failed!\0" as *const u8 as *const ::core::ffi::c_char);
    }
    while len != 0 as ::core::ffi::c_int {
        if g_audiobuffer_end.offset_from(g_audiobuffer_cur) as ::core::ffi::c_long
            == 0 as ::core::ffi::c_long
        {
            RtlRenderAudio(
                g_audiobuffer as *mut int16,
                g_frames_per_block,
                g_audio_channels as ::core::ffi::c_int,
            );
            g_audiobuffer_cur = g_audiobuffer;
            g_audiobuffer_end = g_audiobuffer.offset(
                ((g_frames_per_block * g_audio_channels as ::core::ffi::c_int) as usize)
                    .wrapping_mul(::core::mem::size_of::<int16>() as usize)
                    as isize,
            );
        }
        let mut n: ::core::ffi::c_int = IntMin(
            len,
            g_audiobuffer_end.offset_from(g_audiobuffer_cur) as ::core::ffi::c_long
                as ::core::ffi::c_int,
        );
        if g_sdl_audio_mixer_volume == SDL_MIX_MAXVOLUME {
            memcpy(
                stream as *mut ::core::ffi::c_void,
                g_audiobuffer_cur as *const ::core::ffi::c_void,
                n as size_t,
            );
        } else {
            SDL_memset(
                stream as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                n as size_t,
            );
            SDL_MixAudioFormat(
                stream,
                g_audiobuffer_cur,
                AUDIO_S16 as SDL_AudioFormat,
                n as Uint32,
                g_sdl_audio_mixer_volume,
            );
        }
        g_audiobuffer_cur = g_audiobuffer_cur.offset(n as isize);
        stream = stream.offset(n as isize);
        len -= n;
    }
    SDL_UnlockMutex(g_audio_mutex);
}
static mut g_renderer: *mut SDL_Renderer = ::core::ptr::null::<SDL_Renderer>() as *mut SDL_Renderer;
static mut g_texture: *mut SDL_Texture = ::core::ptr::null::<SDL_Texture>() as *mut SDL_Texture;
static mut g_sdl_renderer_rect: SDL_Rect = SDL_Rect {
    x: 0,
    y: 0,
    w: 0,
    h: 0,
};
unsafe extern "C" fn SdlRenderer_Init(mut window: *mut SDL_Window) -> bool {
    if !g_config.shader.is_null() {
        fprintf(
            stderr,
            b"Warning: Shaders are supported only with the OpenGL backend\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    }
    let mut renderer: *mut SDL_Renderer = SDL_CreateRenderer(
        g_window as *mut SDL_Window,
        -(1 as ::core::ffi::c_int),
        (if g_config.output_method as ::core::ffi::c_int
            == kOutputMethod_SDLSoftware as ::core::ffi::c_int
        {
            SDL_RENDERER_SOFTWARE as ::core::ffi::c_int
        } else {
            SDL_RENDERER_ACCELERATED as ::core::ffi::c_int
                | SDL_RENDERER_PRESENTVSYNC as ::core::ffi::c_int
        }) as Uint32,
    );
    if renderer.is_null() {
        printf(
            b"Failed to create renderer: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            SDL_GetError(),
        );
        return false_0 != 0;
    }
    let mut renderer_info: SDL_RendererInfo = SDL_RendererInfo {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
        num_texture_formats: 0,
        texture_formats: [0; 16],
        max_texture_width: 0,
        max_texture_height: 0,
    };
    SDL_GetRendererInfo(renderer, &raw mut renderer_info);
    g_renderer = renderer;
    if !g_config.ignore_aspect_ratio {
        SDL_RenderSetLogicalSize(renderer, g_snes_width, g_snes_height);
    }
    if g_config.linear_filtering {
        SDL_SetHint(
            SDL_HINT_RENDER_SCALE_QUALITY.as_ptr(),
            b"best\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    let mut tex_mult: ::core::ffi::c_int =
        if g_ppu_render_flags & kPpuRenderFlags_4x4Mode7 as ::core::ffi::c_int != 0 {
            4 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    g_texture = SDL_CreateTexture(
        renderer,
        SDL_PIXELFORMAT_ARGB8888 as ::core::ffi::c_int as Uint32,
        SDL_TEXTUREACCESS_STREAMING as ::core::ffi::c_int,
        g_snes_width * tex_mult,
        g_snes_height * tex_mult,
    );
    if g_texture.is_null() {
        printf(
            b"Failed to create texture: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            SDL_GetError(),
        );
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn SdlRenderer_Destroy() {
    SDL_DestroyTexture(g_texture);
    SDL_DestroyRenderer(g_renderer);
}
unsafe extern "C" fn SdlRenderer_BeginDraw(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
    mut pixels: *mut *mut uint8,
    mut pitch: *mut ::core::ffi::c_int,
) {
    g_sdl_renderer_rect.w = width;
    g_sdl_renderer_rect.h = height;
    if SDL_LockTexture(
        g_texture,
        &raw mut g_sdl_renderer_rect,
        pixels as *mut *mut ::core::ffi::c_void,
        pitch,
    ) != 0 as ::core::ffi::c_int
    {
        printf(
            b"Failed to lock texture: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            SDL_GetError(),
        );
        return;
    }
}
unsafe extern "C" fn SdlRenderer_EndDraw() {
    SDL_UnlockTexture(g_texture);
    SDL_RenderClear(g_renderer);
    SDL_RenderCopy(
        g_renderer,
        g_texture,
        &raw mut g_sdl_renderer_rect,
        ::core::ptr::null::<SDL_Rect>(),
    );
    SDL_RenderPresent(g_renderer);
}
static mut kSdlRendererFuncs: RendererFuncs = RendererFuncs {
    Initialize: Some(SdlRenderer_Init as unsafe extern "C" fn(*mut SDL_Window) -> bool),
    Destroy: Some(SdlRenderer_Destroy as unsafe extern "C" fn() -> ()),
    BeginDraw: Some(
        SdlRenderer_BeginDraw
            as unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                *mut *mut uint8,
                *mut ::core::ffi::c_int,
            ) -> (),
    ),
    EndDraw: Some(SdlRenderer_EndDraw as unsafe extern "C" fn() -> ()),
};
unsafe fn main_0(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    argc -= 1;
    argv = argv.offset(1);
    let mut config_file: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if argc >= 2 as ::core::ffi::c_int
        && strcmp(
            *argv.offset(0 as ::core::ffi::c_int as isize),
            b"--config\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        config_file = *argv.offset(1 as ::core::ffi::c_int as isize);
        argc -= 2 as ::core::ffi::c_int;
        argv = argv.offset(2 as ::core::ffi::c_int as isize);
    } else {
        SwitchDirectory();
    }
    if argc >= 1 as ::core::ffi::c_int
        && strcmp(
            *argv.offset(0 as ::core::ffi::c_int as isize),
            b"--debug\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        g_debug_flag = true_0 != 0;
        argc -= 1 as ::core::ffi::c_int;
        argv = argv.offset(1 as ::core::ffi::c_int as isize);
    }
    ParseConfigFile(config_file);
    g_snes_width = g_config.extended_aspect_ratio as ::core::ffi::c_int * 2 as ::core::ffi::c_int
        + 256 as ::core::ffi::c_int;
    g_snes_height = 240 as ::core::ffi::c_int;
    g_ppu_render_flags = g_config.new_renderer as ::core::ffi::c_int
        * kPpuRenderFlags_NewRenderer as ::core::ffi::c_int
        | g_config.enhanced_mode7 as ::core::ffi::c_int
            * kPpuRenderFlags_4x4Mode7 as ::core::ffi::c_int
        | g_config.extend_y as ::core::ffi::c_int * kPpuRenderFlags_Height240 as ::core::ffi::c_int
        | g_config.no_sprite_limits as ::core::ffi::c_int
            * kPpuRenderFlags_NoSpriteLimits as ::core::ffi::c_int;
    if g_config.fullscreen as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        g_win_flags ^= SDL_WINDOW_FULLSCREEN_DESKTOP as ::core::ffi::c_int as uint32;
    } else if g_config.fullscreen as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
        g_win_flags ^= SDL_WINDOW_FULLSCREEN as ::core::ffi::c_int as uint32;
    }
    g_current_window_scale =
        (if g_config.window_scale as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            2 as ::core::ffi::c_int
        } else {
            IntMin(
                g_config.window_scale as ::core::ffi::c_int,
                kMaxWindowScale as ::core::ffi::c_int,
            )
        }) as uint8;
    if (g_config.audio_freq as ::core::ffi::c_int) < 11025 as ::core::ffi::c_int
        || g_config.audio_freq as ::core::ffi::c_int > 48000 as ::core::ffi::c_int
    {
        g_config.audio_freq = kDefaultFreq as ::core::ffi::c_int as uint16;
    }
    if (g_config.audio_channels as ::core::ffi::c_int) < 1 as ::core::ffi::c_int
        || g_config.audio_channels as ::core::ffi::c_int > 2 as ::core::ffi::c_int
    {
        g_config.audio_channels = kDefaultChannels as ::core::ffi::c_int as uint8;
    }
    if g_config.audio_samples as ::core::ffi::c_int <= 0 as ::core::ffi::c_int
        || g_config.audio_samples as ::core::ffi::c_int
            & g_config.audio_samples as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        g_config.audio_samples = kDefaultSamples as ::core::ffi::c_int as uint16;
    }
    if SDL_Init(
        SDL_INIT_VIDEO as Uint32 | SDL_INIT_AUDIO as Uint32 | SDL_INIT_GAMECONTROLLER as Uint32,
    ) != 0 as ::core::ffi::c_int
    {
        printf(
            b"Failed to init SDL: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            SDL_GetError(),
        );
        return 1 as ::core::ffi::c_int;
    }
    let mut custom_size: bool = g_config.window_width != 0 as ::core::ffi::c_int
        && g_config.window_height != 0 as ::core::ffi::c_int;
    let mut window_width: ::core::ffi::c_int = if custom_size as ::core::ffi::c_int != 0 {
        g_config.window_width
    } else {
        g_current_window_scale as ::core::ffi::c_int * g_snes_width
    };
    let mut window_height: ::core::ffi::c_int = if custom_size as ::core::ffi::c_int != 0 {
        g_config.window_height
    } else {
        g_current_window_scale as ::core::ffi::c_int * g_snes_height
    };
    if g_config.output_method as ::core::ffi::c_int == kOutputMethod_OpenGL as ::core::ffi::c_int {
        g_win_flags |= SDL_WINDOW_OPENGL as ::core::ffi::c_int as uint32;
        OpenGLRenderer_Create(&raw mut g_renderer_funcs);
    } else {
        g_renderer_funcs = kSdlRendererFuncs;
    }
    let mut filename: *const ::core::ffi::c_char =
        if !(*argv.offset(0 as ::core::ffi::c_int as isize)).is_null() {
            *argv.offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char
        } else {
            b"sm.smc\0" as *const u8 as *const ::core::ffi::c_char
        };
    let mut snes: *mut Snes = SnesInit(filename);
    if snes.is_null() {
        let mut buf: [::core::ffi::c_char; 256] = [0; 256];
        snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
            b"unable to load rom: %s\0" as *const u8 as *const ::core::ffi::c_char,
            filename,
        );
        Die(&raw mut buf as *mut ::core::ffi::c_char);
        return 1 as ::core::ffi::c_int;
    }
    let mut window: *mut SDL_Window = SDL_CreateWindow(
        &raw const kWindowTitle as *const ::core::ffi::c_char,
        SDL_WINDOWPOS_UNDEFINED as ::core::ffi::c_int,
        SDL_WINDOWPOS_UNDEFINED as ::core::ffi::c_int,
        window_width,
        window_height,
        g_win_flags as Uint32,
    ) as *mut SDL_Window;
    if window.is_null() {
        printf(
            b"Failed to create window: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            SDL_GetError(),
        );
        return 1 as ::core::ffi::c_int;
    }
    g_window = window;
    SDL_SetWindowHitTest(
        window as *mut SDL_Window,
        Some(
            HitTestCallback
                as unsafe extern "C" fn(
                    *mut SDL_Window,
                    *const SDL_Point,
                    *mut ::core::ffi::c_void,
                ) -> SDL_HitTestResult,
        ),
        NULL,
    );
    if !g_renderer_funcs
        .Initialize
        .expect("non-null function pointer")(window)
    {
        return 1 as ::core::ffi::c_int;
    }
    g_audio_mutex = SDL_CreateMutex();
    if g_audio_mutex.is_null() {
        Die(b"No mutex\0" as *const u8 as *const ::core::ffi::c_char);
    }
    g_spc_player = SpcPlayer_Create() as *mut SpcPlayer;
    SpcPlayer_Initialize(g_spc_player as *mut SpcPlayer);
    let mut enable_audio: bool = true_0 != 0;
    if enable_audio {
        let mut want: SDL_AudioSpec = SDL_AudioSpec {
            freq: 0 as ::core::ffi::c_int,
            format: 0,
            channels: 0,
            silence: 0,
            samples: 0,
            padding: 0,
            size: 0,
            callback: None,
            userdata: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut have: SDL_AudioSpec = SDL_AudioSpec {
            freq: 0,
            format: 0,
            channels: 0,
            silence: 0,
            samples: 0,
            padding: 0,
            size: 0,
            callback: None,
            userdata: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        want.freq = 44100 as ::core::ffi::c_int;
        want.format = AUDIO_S16 as SDL_AudioFormat;
        want.channels = 2 as Uint8;
        want.samples = 2048 as Uint16;
        want.callback = Some(
            AudioCallback
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut Uint8,
                    ::core::ffi::c_int,
                ) -> (),
        ) as SDL_AudioCallback;
        g_audio_device = SDL_OpenAudioDevice(
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            &raw mut want,
            &raw mut have,
            0 as ::core::ffi::c_int,
        );
        if g_audio_device == 0 as SDL_AudioDeviceID {
            printf(
                b"Failed to open audio device: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                SDL_GetError(),
            );
            return 1 as ::core::ffi::c_int;
        }
        g_audio_channels = 2 as uint8;
        g_frames_per_block = 534 as ::core::ffi::c_int * have.freq / 32000 as ::core::ffi::c_int;
        g_audiobuffer = malloc(
            ((g_frames_per_block * have.channels as ::core::ffi::c_int) as size_t)
                .wrapping_mul(::core::mem::size_of::<int16>() as size_t),
        ) as *mut uint8;
    }
    PpuBeginDrawing(
        (*snes).snes_ppu,
        &raw mut g_pixels as *mut uint8_t,
        (256 as ::core::ffi::c_int * 4 as ::core::ffi::c_int) as size_t,
        0 as uint32_t,
    );
    PpuBeginDrawing(
        (*snes).my_ppu,
        &raw mut g_my_pixels as *mut uint8_t,
        (256 as ::core::ffi::c_int * 4 as ::core::ffi::c_int) as size_t,
        0 as uint32_t,
    );
    mkdir(
        b"saves\0" as *const u8 as *const ::core::ffi::c_char,
        0o755 as __mode_t,
    );
    RtlReadSram();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < SDL_NumJoysticks() {
        OpenOneGamepad(i);
        i += 1;
    }
    if g_config.autosave {
        HandleCommand(
            (kKeys_Load as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as uint32,
            true_0 != 0,
        );
    }
    let mut running: bool = true_0 != 0;
    let mut lastTick: uint32 = SDL_GetTicks() as uint32;
    let mut curTick: uint32 = 0 as uint32;
    let mut frameCtr: uint32 = 0 as uint32;
    let mut audiopaused: uint8 = true_0 as uint8;
    let mut has_bug_in_title: bool = false_0 != 0;
    while running {
        let mut event: SDL_Event = SDL_Event { type_0: 0 };
        while SDL_PollEvent(&raw mut event) != 0 {
            match event.type_0 {
                1619 => {
                    OpenOneGamepad(event.cdevice.which as ::core::ffi::c_int);
                }
                1616 => {
                    HandleGamepadAxisInput(
                        event.caxis.which as ::core::ffi::c_int,
                        event.caxis.axis as ::core::ffi::c_int,
                        event.caxis.value as ::core::ffi::c_int,
                    );
                }
                1617 | 1618 => {
                    let mut b: ::core::ffi::c_int =
                        RemapSdlButton(event.cbutton.button as ::core::ffi::c_int);
                    if b >= 0 as ::core::ffi::c_int {
                        HandleGamepadInput(
                            b,
                            event.type_0
                                == SDL_CONTROLLERBUTTONDOWN as ::core::ffi::c_int as Uint32,
                        );
                    }
                }
                1027 => {
                    if SDL_GetModState() as ::core::ffi::c_uint
                        & KMOD_CTRL as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                        && event.wheel.y != 0 as Sint32
                    {
                        ChangeWindowScale(if event.wheel.y > 0 as Sint32 {
                            1 as ::core::ffi::c_int
                        } else {
                            -(1 as ::core::ffi::c_int)
                        });
                    }
                }
                1025 => {
                    if event.button.button as ::core::ffi::c_int == SDL_BUTTON_LEFT
                        && event.button.state as ::core::ffi::c_int == SDL_PRESSED
                        && event.button.clicks as ::core::ffi::c_int == 2 as ::core::ffi::c_int
                    {
                        if g_win_flags
                            & SDL_WINDOW_FULLSCREEN_DESKTOP as ::core::ffi::c_int as uint32
                            == 0 as uint32
                            && g_win_flags & SDL_WINDOW_FULLSCREEN as ::core::ffi::c_int as uint32
                                == 0 as uint32
                            && SDL_GetModState() as ::core::ffi::c_uint
                                & KMOD_SHIFT as ::core::ffi::c_int as ::core::ffi::c_uint
                                != 0
                        {
                            g_win_flags ^= SDL_WINDOW_BORDERLESS as ::core::ffi::c_int as uint32;
                            SDL_SetWindowBordered(
                                g_window as *mut SDL_Window,
                                (if g_win_flags
                                    & SDL_WINDOW_BORDERLESS as ::core::ffi::c_int as uint32
                                    == 0 as uint32
                                {
                                    SDL_TRUE as ::core::ffi::c_int
                                } else {
                                    SDL_FALSE as ::core::ffi::c_int
                                }) as SDL_bool,
                            );
                        }
                    }
                }
                768 => {
                    HandleInput(
                        event.key.keysym.sym as ::core::ffi::c_int,
                        event.key.keysym.mod_0 as ::core::ffi::c_int,
                        true_0 != 0,
                    );
                }
                769 => {
                    HandleInput(
                        event.key.keysym.sym as ::core::ffi::c_int,
                        event.key.keysym.mod_0 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                }
                256 => {
                    running = false_0 != 0;
                }
                _ => {}
            }
        }
        if g_paused as ::core::ffi::c_int != audiopaused as ::core::ffi::c_int {
            audiopaused = g_paused;
            if g_audio_device != 0 {
                SDL_PauseAudioDevice(g_audio_device, audiopaused as ::core::ffi::c_int);
            }
        }
        if g_paused != 0 {
            SDL_Delay(16 as Uint32);
        } else {
            let mut inputs: ::core::ffi::c_int = g_input1_state;
            if g_input1_state & 0xf0 as ::core::ffi::c_int != 0 {
                g_gamepad_buttons = 0 as uint8;
            }
            inputs |= g_gamepad_buttons as ::core::ffi::c_int;
            let mut is_replay: uint8 = RtlRunFrame(inputs) as uint8;
            frameCtr = frameCtr.wrapping_add(1);
            (*g_snes).disableRender = g_turbo as ::core::ffi::c_int
                ^ is_replay as ::core::ffi::c_int & g_replay_turbo as ::core::ffi::c_int
                != 0
                && frameCtr
                    & (if g_turbo as ::core::ffi::c_int != 0 {
                        0xf as ::core::ffi::c_int
                    } else {
                        0x7f as ::core::ffi::c_int
                    }) as uint32
                    != 0 as uint32;
            if !(*g_snes).disableRender {
                DrawPpuFrameWithPerf();
            }
            let mut want_bug_in_title: bool = g_got_mismatch_count != 0 as ::core::ffi::c_int;
            if want_bug_in_title as ::core::ffi::c_int != has_bug_in_title as ::core::ffi::c_int {
                has_bug_in_title = want_bug_in_title;
                let mut title: [::core::ffi::c_char; 60] = [0; 60];
                if want_bug_in_title {
                    snprintf(
                        &raw mut title as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 60]>() as size_t,
                        b"%s | BUG FOUND!\0" as *const u8 as *const ::core::ffi::c_char,
                        &raw const kWindowTitle as *const ::core::ffi::c_char,
                    );
                    SDL_SetWindowTitle(
                        g_window as *mut SDL_Window,
                        &raw mut title as *mut ::core::ffi::c_char,
                    );
                } else {
                    SDL_SetWindowTitle(
                        g_window as *mut SDL_Window,
                        &raw const kWindowTitle as *const ::core::ffi::c_char,
                    );
                }
            }
            curTick = SDL_GetTicks() as uint32;
            if !(*g_snes).disableRender && !g_config.disable_frame_delay {
                static mut delays: [uint8; 3] = [
                    17 as ::core::ffi::c_int as uint8,
                    17 as ::core::ffi::c_int as uint8,
                    16 as ::core::ffi::c_int as uint8,
                ];
                lastTick = lastTick
                    .wrapping_add(delays[frameCtr.wrapping_rem(3 as uint32) as usize] as uint32);
                if lastTick > curTick {
                    let mut delta: uint32 = lastTick.wrapping_sub(curTick);
                    if delta > 500 as uint32 {
                        lastTick = curTick.wrapping_sub(500 as uint32);
                        delta = 500 as uint32;
                    }
                    SDL_Delay(delta as Uint32);
                } else if curTick.wrapping_sub(lastTick) > 500 as uint32 {
                    lastTick = curTick;
                }
            }
        }
    }
    if g_config.autosave {
        HandleCommand(
            (kKeys_Save as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as uint32,
            true_0 != 0,
        );
    }
    SDL_PauseAudioDevice(g_audio_device, 1 as ::core::ffi::c_int);
    SDL_CloseAudioDevice(g_audio_device);
    SDL_DestroyMutex(g_audio_mutex);
    free(g_audiobuffer as *mut ::core::ffi::c_void);
    ::core::mem::transmute::<_, fn()>(g_renderer_funcs.Destroy.expect("non-null function pointer"))(
    );
    SDL_DestroyWindow(window as *mut SDL_Window);
    SDL_Quit();
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn RenderDigit(
    mut dst: *mut uint8,
    mut pitch: size_t,
    mut digit: ::core::ffi::c_int,
    mut color: uint32,
    mut big: bool,
) {
    static mut kFont: [uint8; 100] = [
        0x1c as ::core::ffi::c_int as uint8,
        0x36 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x36 as ::core::ffi::c_int as uint8,
        0x1c as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x1c as ::core::ffi::c_int as uint8,
        0x1e as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0x7e as ::core::ffi::c_int as uint8,
        0x3e as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x30 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0xc as ::core::ffi::c_int as uint8,
        0x6 as ::core::ffi::c_int as uint8,
        0x3 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x7f as ::core::ffi::c_int as uint8,
        0x3e as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x3c as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x3e as ::core::ffi::c_int as uint8,
        0x30 as ::core::ffi::c_int as uint8,
        0x38 as ::core::ffi::c_int as uint8,
        0x3c as ::core::ffi::c_int as uint8,
        0x36 as ::core::ffi::c_int as uint8,
        0x33 as ::core::ffi::c_int as uint8,
        0x7f as ::core::ffi::c_int as uint8,
        0x30 as ::core::ffi::c_int as uint8,
        0x30 as ::core::ffi::c_int as uint8,
        0x30 as ::core::ffi::c_int as uint8,
        0x78 as ::core::ffi::c_int as uint8,
        0x7f as ::core::ffi::c_int as uint8,
        0x3 as ::core::ffi::c_int as uint8,
        0x3 as ::core::ffi::c_int as uint8,
        0x3 as ::core::ffi::c_int as uint8,
        0x3f as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x3e as ::core::ffi::c_int as uint8,
        0x1c as ::core::ffi::c_int as uint8,
        0x6 as ::core::ffi::c_int as uint8,
        0x3 as ::core::ffi::c_int as uint8,
        0x3 as ::core::ffi::c_int as uint8,
        0x3f as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x3e as ::core::ffi::c_int as uint8,
        0x7f as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x30 as ::core::ffi::c_int as uint8,
        0x18 as ::core::ffi::c_int as uint8,
        0xc as ::core::ffi::c_int as uint8,
        0xc as ::core::ffi::c_int as uint8,
        0xc as ::core::ffi::c_int as uint8,
        0xc as ::core::ffi::c_int as uint8,
        0x3e as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x3e as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x3e as ::core::ffi::c_int as uint8,
        0x3e as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x63 as ::core::ffi::c_int as uint8,
        0x7e as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x30 as ::core::ffi::c_int as uint8,
        0x1e as ::core::ffi::c_int as uint8,
    ];
    let mut p: *const uint8 =
        (&raw const kFont as *const uint8).offset((digit * 10 as ::core::ffi::c_int) as isize);
    if !big {
        let mut y: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while y < 10 as ::core::ffi::c_int {
            let fresh0 = p;
            p = p.offset(1);
            let mut v: ::core::ffi::c_int = *fresh0 as ::core::ffi::c_int;
            let mut x: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while v != 0 {
                if v & 1 as ::core::ffi::c_int != 0 {
                    *(dst as *mut uint32).offset(x as isize) = color;
                }
                x += 1;
                v >>= 1 as ::core::ffi::c_int;
            }
            y += 1;
            dst = dst.offset(pitch as isize);
        }
    } else {
        let mut y_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while y_0 < 10 as ::core::ffi::c_int {
            let fresh1 = p;
            p = p.offset(1);
            let mut v_0: ::core::ffi::c_int = *fresh1 as ::core::ffi::c_int;
            let mut x_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while v_0 != 0 {
                if v_0 & 1 as ::core::ffi::c_int != 0 {
                    let ref mut fresh2 =
                        *(dst as *mut uint32).offset((x_0 * 2 as ::core::ffi::c_int) as isize);
                    *fresh2 = color;
                    *(dst as *mut uint32).offset(
                        (x_0 * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                    ) = *fresh2;
                    let ref mut fresh3 = *(dst.offset(pitch as isize) as *mut uint32)
                        .offset((x_0 * 2 as ::core::ffi::c_int) as isize);
                    *fresh3 = color;
                    *(dst.offset(pitch as isize) as *mut uint32).offset(
                        (x_0 * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                    ) = *fresh3;
                }
                x_0 += 1;
                v_0 >>= 1 as ::core::ffi::c_int;
            }
            y_0 += 1;
            dst = dst.offset(pitch.wrapping_mul(2 as size_t) as isize);
        }
    };
}
unsafe extern "C" fn RenderNumber(
    mut dst: *mut uint8,
    mut pitch: size_t,
    mut n: ::core::ffi::c_int,
    mut big: uint8,
) {
    let mut buf: [::core::ffi::c_char; 32] = [0; 32];
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    sprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    s = &raw mut buf as *mut ::core::ffi::c_char;
    i = 2 as ::core::ffi::c_int * 4 as ::core::ffi::c_int;
    while *s != 0 {
        RenderDigit(
            dst.offset(
                (pitch.wrapping_add(i as size_t).wrapping_add(4 as size_t)
                    << big as ::core::ffi::c_int) as isize,
            ),
            pitch,
            *s as ::core::ffi::c_int - '0' as i32,
            0x404040 as uint32,
            big != 0,
        );
        s = s.offset(1);
        i += 8 as ::core::ffi::c_int * 4 as ::core::ffi::c_int;
    }
    s = &raw mut buf as *mut ::core::ffi::c_char;
    i = 2 as ::core::ffi::c_int * 4 as ::core::ffi::c_int;
    while *s != 0 {
        RenderDigit(
            dst.offset((i << big as ::core::ffi::c_int) as isize),
            pitch,
            *s as ::core::ffi::c_int - '0' as i32,
            0xffffff as uint32,
            big != 0,
        );
        s = s.offset(1);
        i += 8 as ::core::ffi::c_int * 4 as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn HandleCommand(mut j: uint32, mut pressed: bool) {
    if j <= kKeys_Controls_Last as ::core::ffi::c_int as uint32 {
        static mut kKbdRemap: [uint8; 13] = [
            0 as ::core::ffi::c_int as uint8,
            4 as ::core::ffi::c_int as uint8,
            5 as ::core::ffi::c_int as uint8,
            6 as ::core::ffi::c_int as uint8,
            7 as ::core::ffi::c_int as uint8,
            2 as ::core::ffi::c_int as uint8,
            3 as ::core::ffi::c_int as uint8,
            8 as ::core::ffi::c_int as uint8,
            0 as ::core::ffi::c_int as uint8,
            9 as ::core::ffi::c_int as uint8,
            1 as ::core::ffi::c_int as uint8,
            10 as ::core::ffi::c_int as uint8,
            11 as ::core::ffi::c_int as uint8,
        ];
        if pressed {
            g_input1_state |=
                (1 as ::core::ffi::c_int) << kKbdRemap[j as usize] as ::core::ffi::c_int;
        } else {
            g_input1_state &=
                !((1 as ::core::ffi::c_int) << kKbdRemap[j as usize] as ::core::ffi::c_int);
        }
        return;
    }
    if j == kKeys_Turbo as ::core::ffi::c_int as uint32 {
        g_turbo = pressed as uint8;
        return;
    }
    if !pressed {
        return;
    }
    if j <= kKeys_Load_Last as ::core::ffi::c_int as uint32 {
        RtlSaveLoad(
            kSaveLoad_Load as ::core::ffi::c_int,
            j.wrapping_sub(kKeys_Load as ::core::ffi::c_int as uint32) as ::core::ffi::c_int,
        );
    } else if j <= kKeys_Save_Last as ::core::ffi::c_int as uint32 {
        RtlSaveLoad(
            kSaveLoad_Save as ::core::ffi::c_int,
            j.wrapping_sub(kKeys_Save as ::core::ffi::c_int as uint32) as ::core::ffi::c_int,
        );
    } else if j <= kKeys_Replay_Last as ::core::ffi::c_int as uint32 {
        RtlSaveLoad(
            kSaveLoad_Replay as ::core::ffi::c_int,
            j.wrapping_sub(kKeys_Replay as ::core::ffi::c_int as uint32) as ::core::ffi::c_int,
        );
    } else if j <= kKeys_LoadRef_Last as ::core::ffi::c_int as uint32 {
        RtlSaveLoad(
            kSaveLoad_Load as ::core::ffi::c_int,
            (256 as uint32)
                .wrapping_add(j)
                .wrapping_sub(kKeys_LoadRef as ::core::ffi::c_int as uint32)
                as ::core::ffi::c_int,
        );
    } else if j <= kKeys_ReplayRef_Last as ::core::ffi::c_int as uint32 {
        RtlSaveLoad(
            kSaveLoad_Replay as ::core::ffi::c_int,
            (256 as uint32)
                .wrapping_add(j)
                .wrapping_sub(kKeys_ReplayRef as ::core::ffi::c_int as uint32)
                as ::core::ffi::c_int,
        );
    } else {
        match j {
            113 => {
                RtlCheat('w' as i32 as ::core::ffi::c_char);
            }
            114 => {
                RtlCheat('q' as i32 as ::core::ffi::c_char);
            }
            115 => {
                g_other_image = !g_other_image;
            }
            116 => {
                RtlClearKeyLog();
            }
            117 => {
                RtlStopReplay();
            }
            118 => {
                g_win_flags ^= SDL_WINDOW_FULLSCREEN_DESKTOP as ::core::ffi::c_int as uint32;
                SDL_SetWindowFullscreen(
                    g_window as *mut SDL_Window,
                    g_win_flags as Uint32
                        & SDL_WINDOW_FULLSCREEN_DESKTOP as ::core::ffi::c_int as Uint32,
                );
                g_cursor = (g_cursor == 0) as ::core::ffi::c_int as uint8;
                SDL_ShowCursor(g_cursor as ::core::ffi::c_int);
            }
            119 => {
                RtlReset(1 as ::core::ffi::c_int);
            }
            120 => {
                g_paused = (g_paused == 0) as ::core::ffi::c_int as uint8;
            }
            121 => {
                g_paused = (g_paused == 0) as ::core::ffi::c_int as uint8;
            }
            123 => {
                g_replay_turbo = (g_replay_turbo == 0) as ::core::ffi::c_int as uint8;
            }
            124 => {
                ChangeWindowScale(1 as ::core::ffi::c_int);
            }
            125 => {
                ChangeWindowScale(-(1 as ::core::ffi::c_int));
            }
            126 => {
                g_display_perf =
                    (g_display_perf as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) != 0;
            }
            127 => {
                g_ppu_render_flags ^= kPpuRenderFlags_NewRenderer as ::core::ffi::c_int;
                g_new_ppu = g_ppu_render_flags & kPpuRenderFlags_NewRenderer as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int;
            }
            128 | 129 => {
                HandleVolumeAdjustment(if j == kKeys_VolumeUp as ::core::ffi::c_int as uint32 {
                    1 as ::core::ffi::c_int
                } else {
                    -(1 as ::core::ffi::c_int)
                });
            }
            _ => {
                '_c2rust_label: {
                    __assert_fail(
                        b"0\0" as *const u8 as *const ::core::ffi::c_char,
                        b"src/main.c\0" as *const u8 as *const ::core::ffi::c_char,
                        686 as ::core::ffi::c_uint,
                        b"void HandleCommand(uint32, _Bool)\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
            }
        }
    };
}
unsafe extern "C" fn HandleInput(
    mut keyCode: ::core::ffi::c_int,
    mut keyMod: ::core::ffi::c_int,
    mut pressed: bool,
) {
    let mut j: ::core::ffi::c_int = FindCmdForSdlKey(keyCode as SDL_Keycode, keyMod as SDL_Keymod);
    if j != 0 as ::core::ffi::c_int {
        HandleCommand(j as uint32, pressed);
    }
}
unsafe extern "C" fn OpenOneGamepad(mut i: ::core::ffi::c_int) {
    if SDL_IsGameController(i) as u64 != 0 {
        let mut controller: *mut SDL_GameController = SDL_GameControllerOpen(i);
        if controller.is_null() {
            fprintf(
                stderr,
                b"Could not open gamepad %d: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                i,
                SDL_GetError(),
            );
        }
    }
}
unsafe extern "C" fn RemapSdlButton(mut button: ::core::ffi::c_int) -> ::core::ffi::c_int {
    match button {
        0 => return kGamepadBtn_A as ::core::ffi::c_int,
        1 => return kGamepadBtn_B as ::core::ffi::c_int,
        2 => return kGamepadBtn_X as ::core::ffi::c_int,
        3 => return kGamepadBtn_Y as ::core::ffi::c_int,
        4 => return kGamepadBtn_Back as ::core::ffi::c_int,
        5 => return kGamepadBtn_Guide as ::core::ffi::c_int,
        6 => return kGamepadBtn_Start as ::core::ffi::c_int,
        7 => return kGamepadBtn_L3 as ::core::ffi::c_int,
        8 => return kGamepadBtn_R3 as ::core::ffi::c_int,
        9 => return kGamepadBtn_L1 as ::core::ffi::c_int,
        10 => return kGamepadBtn_R1 as ::core::ffi::c_int,
        11 => return kGamepadBtn_DpadUp as ::core::ffi::c_int,
        12 => return kGamepadBtn_DpadDown as ::core::ffi::c_int,
        13 => return kGamepadBtn_DpadLeft as ::core::ffi::c_int,
        14 => return kGamepadBtn_DpadRight as ::core::ffi::c_int,
        _ => return -(1 as ::core::ffi::c_int),
    };
}
unsafe extern "C" fn HandleGamepadInput(mut button: ::core::ffi::c_int, mut pressed: bool) {
    if (g_gamepad_modifiers & ((1 as ::core::ffi::c_int) << button) as uint32 != 0)
        as ::core::ffi::c_int
        == pressed as ::core::ffi::c_int
    {
        return;
    }
    g_gamepad_modifiers ^= ((1 as ::core::ffi::c_int) << button) as uint32;
    if pressed {
        g_gamepad_last_cmd[button as usize] =
            FindCmdForGamepadButton(button, g_gamepad_modifiers) as uint16;
    }
    if g_gamepad_last_cmd[button as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        HandleCommand(g_gamepad_last_cmd[button as usize] as uint32, pressed);
    }
}
unsafe extern "C" fn HandleVolumeAdjustment(mut volume_adjustment: ::core::ffi::c_int) {
    g_sdl_audio_mixer_volume = IntMin(
        IntMax(
            0 as ::core::ffi::c_int,
            g_sdl_audio_mixer_volume
                + volume_adjustment * (SDL_MIX_MAXVOLUME >> 4 as ::core::ffi::c_int),
        ),
        SDL_MIX_MAXVOLUME,
    );
    printf(
        b"[SDL mixer volume]=%i\n\0" as *const u8 as *const ::core::ffi::c_char,
        g_sdl_audio_mixer_volume,
    );
}
unsafe extern "C" fn ApproximateAtan2(
    mut y: ::core::ffi::c_float,
    mut x: ::core::ffi::c_float,
) -> ::core::ffi::c_float {
    let mut sign_mask: uint32 = 0x80000000 as uint32;
    let mut b: ::core::ffi::c_float = 0.596227f32;
    let mut ux_s: uint32 = sign_mask & *(&raw mut x as *mut uint32);
    let mut uy_s: uint32 = sign_mask & *(&raw mut y as *mut uint32);
    let mut q: ::core::ffi::c_float = ((!ux_s & uy_s) >> 29 as ::core::ffi::c_int
        | ux_s >> 30 as ::core::ffi::c_int)
        as ::core::ffi::c_float;
    let mut bxy_a: ::core::ffi::c_float = b * x * y;
    if bxy_a < 0.0f32 {
        bxy_a = -bxy_a;
    }
    let mut num: ::core::ffi::c_float = bxy_a + y * y;
    let mut atan_1q: ::core::ffi::c_float = num / (x * x + bxy_a + num + 0.000001f32);
    let mut uatan_2q: uint32_t =
        ux_s as uint32_t ^ uy_s as uint32_t | *(&raw mut atan_1q as *mut uint32);
    return q + *(&raw mut uatan_2q as *mut ::core::ffi::c_float);
}
unsafe extern "C" fn HandleGamepadAxisInput(
    mut gamepad_id: ::core::ffi::c_int,
    mut axis: ::core::ffi::c_int,
    mut value: ::core::ffi::c_int,
) {
    static mut last_gamepad_id: ::core::ffi::c_int = 0;
    static mut last_x: ::core::ffi::c_int = 0;
    static mut last_y: ::core::ffi::c_int = 0;
    if axis == SDL_CONTROLLER_AXIS_LEFTX as ::core::ffi::c_int
        || axis == SDL_CONTROLLER_AXIS_LEFTY as ::core::ffi::c_int
    {
        if last_gamepad_id != gamepad_id {
            if value > -(16000 as ::core::ffi::c_int) && value < 16000 as ::core::ffi::c_int {
                return;
            }
            last_gamepad_id = gamepad_id;
            last_y = 0 as ::core::ffi::c_int;
            last_x = last_y;
        }
        *if axis == SDL_CONTROLLER_AXIS_LEFTX as ::core::ffi::c_int {
            &raw mut last_x
        } else {
            &raw mut last_y
        } = value;
        let mut buttons: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if last_x * last_x + last_y * last_y
            >= 10000 as ::core::ffi::c_int * 10000 as ::core::ffi::c_int
        {
            static mut kSegmentToButtons: [uint8; 8] = [
                ((1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int) as uint8,
                ((1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int)
                    as uint8,
                ((1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as uint8,
                ((1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int)
                    as uint8,
                ((1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int) as uint8,
                ((1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int)
                    as uint8,
                ((1 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int) as uint8,
                ((1 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
                    as uint8,
            ];
            let mut angle: uint8 = (ApproximateAtan2(
                last_y as ::core::ffi::c_float,
                last_x as ::core::ffi::c_float,
            ) * 64.0f32
                + 0.5f32) as ::core::ffi::c_int as uint8;
            buttons = kSegmentToButtons[((angle as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int) as uint8
                as ::core::ffi::c_int
                >> 5 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int;
        }
        g_gamepad_buttons = buttons as uint8;
    } else if axis == SDL_CONTROLLER_AXIS_TRIGGERLEFT as ::core::ffi::c_int
        || axis == SDL_CONTROLLER_AXIS_TRIGGERRIGHT as ::core::ffi::c_int
    {
        if value < 12000 as ::core::ffi::c_int || value >= 16000 as ::core::ffi::c_int {
            HandleGamepadInput(
                if axis == SDL_CONTROLLER_AXIS_TRIGGERLEFT as ::core::ffi::c_int {
                    kGamepadBtn_L2 as ::core::ffi::c_int
                } else {
                    kGamepadBtn_R2 as ::core::ffi::c_int
                },
                value >= 12000 as ::core::ffi::c_int,
            );
        }
    }
}
unsafe extern "C" fn SwitchDirectory() {
    let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    if getcwd(
        &raw mut buf as *mut ::core::ffi::c_char,
        (::core::mem::size_of::<[::core::ffi::c_char; 4096]>() as size_t)
            .wrapping_sub(32 as size_t),
    )
    .is_null()
    {
        return;
    }
    let mut pos: size_t = strlen(&raw mut buf as *mut ::core::ffi::c_char);
    let mut step: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while pos != 0 as size_t && step < 3 as ::core::ffi::c_int {
        memcpy(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(pos as isize)
                as *mut ::core::ffi::c_void,
            b"/sm.ini\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            8 as size_t,
        );
        let mut f: *mut FILE = fopen(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"rb\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut FILE;
        if !f.is_null() {
            fclose(f);
            buf[pos as usize] = 0 as ::core::ffi::c_char;
            if step != 0 as ::core::ffi::c_int {
                printf(
                    b"Found sm.ini in %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut buf as *mut ::core::ffi::c_char,
                );
                let mut err: ::core::ffi::c_int = chdir(&raw mut buf as *mut ::core::ffi::c_char);
            }
            return;
        }
        pos = pos.wrapping_sub(1);
        while pos != 0 as size_t
            && buf[pos as usize] as ::core::ffi::c_int != '/' as i32
            && buf[pos as usize] as ::core::ffi::c_int != '\\' as i32
        {
            pos = pos.wrapping_sub(1);
        }
        step += 1;
    }
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub fn main() {
    if std::env::args().nth(1).as_deref() == Some("verify") {
        ::std::process::exit(verify::run());
    }
    let mut args_strings: Vec<Vec<u8>> = ::std::env::args()
        .map(|arg| {
            ::std::ffi::CString::new(arg)
                .expect("Failed to convert argument into CString.")
                .into_bytes_with_nul()
        })
        .collect();
    let mut args_ptrs: Vec<*mut ::core::ffi::c_char> = args_strings
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut ::core::ffi::c_char)
        .chain(::core::iter::once(::core::ptr::null_mut()))
        .collect();
    unsafe {
        ::std::process::exit(main_0(
            (args_ptrs.len() - 1) as ::core::ffi::c_int,
            args_ptrs.as_mut_ptr() as *mut *mut ::core::ffi::c_char,
        ) as i32)
    }
}
