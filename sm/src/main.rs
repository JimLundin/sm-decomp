#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
#[allow(unused_imports)]
use ::sm as _;
use sm::game_state::{Game, GameOwner};
mod verify;
pub use sdl2_sys::{
    SDL_AudioDeviceEvent, SDL_AudioSpec, SDL_CloseAudioDevice, SDL_CommonEvent,
    SDL_ControllerAxisEvent, SDL_ControllerButtonEvent, SDL_ControllerDeviceEvent,
    SDL_ControllerSensorEvent, SDL_ControllerTouchpadEvent, SDL_CreateMutex, SDL_CreateRenderer,
    SDL_CreateTexture, SDL_CreateWindow, SDL_Delay, SDL_DestroyMutex, SDL_DestroyRenderer,
    SDL_DestroyTexture, SDL_DestroyWindow, SDL_DisplayEvent, SDL_DollarGestureEvent, SDL_DropEvent,
    SDL_Event, SDL_GameController, SDL_GameControllerOpen, SDL_GetDisplayUsableBounds,
    SDL_GetError, SDL_GetGlobalMouseState, SDL_GetModState, SDL_GetPerformanceCounter,
    SDL_GetPerformanceFrequency, SDL_GetPixelFormatName, SDL_GetRendererInfo, SDL_GetTicks,
    SDL_GetWindowBordersSize, SDL_GetWindowDisplayIndex, SDL_GetWindowFlags, SDL_GetWindowSize,
    SDL_HitTest, SDL_HitTestResult, SDL_Init, SDL_IsGameController, SDL_JoyAxisEvent,
    SDL_JoyBallEvent, SDL_JoyBatteryEvent, SDL_JoyButtonEvent, SDL_JoyDeviceEvent, SDL_JoyHatEvent,
    SDL_KeyboardEvent, SDL_Keysym, SDL_LockMutex, SDL_LockTexture, SDL_MixAudioFormat,
    SDL_MouseButtonEvent, SDL_MouseMotionEvent, SDL_MouseWheelEvent, SDL_MultiGestureEvent,
    SDL_NumJoysticks, SDL_OpenAudioDevice, SDL_PauseAudioDevice, SDL_Point, SDL_PollEvent,
    SDL_Quit, SDL_QuitEvent, SDL_Rect, SDL_RenderClear, SDL_RenderCopy, SDL_RenderPresent,
    SDL_RenderSetLogicalSize, SDL_Renderer, SDL_RendererInfo, SDL_SensorEvent, SDL_SetHint,
    SDL_SetWindowBordered, SDL_SetWindowFullscreen, SDL_SetWindowHitTest, SDL_SetWindowPosition,
    SDL_SetWindowSize, SDL_SetWindowTitle, SDL_ShowCursor, SDL_ShowSimpleMessageBox,
    SDL_SysWMEvent, SDL_SysWMmsg, SDL_TextEditingEvent, SDL_TextEditingExtEvent,
    SDL_TextInputEvent, SDL_Texture, SDL_TouchFingerEvent, SDL_UnlockMutex, SDL_UnlockTexture,
    SDL_UserEvent, SDL_WindowEvent, SDL_bool, SDL_memset, SDL_mutex,
};
use sm::config::{g_config, FindCmdForGamepadButton, FindCmdForSdlKey, ParseConfigFile};
use sm::cpu_infra::SnesInit;
use sm::host::{Die, Warning};
use sm::platform::opengl::OpenGLRenderer_Create;
use sm::rtl::{
    RtlCheat, RtlClearKeyLog, RtlReadSram, RtlRenderAudio, RtlReset, RtlRunFrame, RtlSaveLoad,
    RtlStopReplay,
};
use sm::snes::ppu::{PpuBeginDrawing, PpuGetCurrentRenderScale};
use sm::spc_player::{SpcPlayer_Create, SpcPlayer_Initialize};
pub use sm::types::{_IO_codecvt, _IO_marker, _IO_wide_data, SDL_Window};
extern "C" {

    #[cfg(not(windows))]
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
    fn mkdir(__path: *const ::core::ffi::c_char, __mode: __mode_t) -> ::core::ffi::c_int;
    fn chdir(__path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn getcwd(__buf: *mut ::core::ffi::c_char, __size: size_t) -> *mut ::core::ffi::c_char;

    #[cfg(not(windows))]
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;

}
pub use sm::types::{
    _IO_lock_t, __int16_t, __int32_t, __int64_t, __int8_t, __mode_t, __off64_t, __off_t,
    __uint16_t, __uint32_t, __uint64_t, __uint8_t, int16_t, int32_t, int64_t, int8_t, size_t,
    uint16_t, uint32_t, uint64_t, uint8_t, _IO_FILE, FILE,
};
pub const SDL_TRUE: SDL_bool = sdl2_sys::SDL_bool::SDL_TRUE;
pub const SDL_FALSE: SDL_bool = sdl2_sys::SDL_bool::SDL_FALSE;
pub use sm::types::SDL_AudioDeviceID;
pub use sm::types::{
    SDL_AudioCallback, SDL_AudioFormat, Sint16, Sint32, Sint64, Uint16, Uint32, Uint64, Uint8,
};
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const SDL_PIXELFORMAT_EXTERNAL_OES: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_EXTERNAL_OES as C2RustUnnamed;
pub const SDL_PIXELFORMAT_NV21: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_NV21 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_NV12: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_NV12 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_YVYU: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_YVYU as C2RustUnnamed;
pub const SDL_PIXELFORMAT_UYVY: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_UYVY as C2RustUnnamed;
pub const SDL_PIXELFORMAT_YUY2: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_YUY2 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_IYUV: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_IYUV as C2RustUnnamed;
pub const SDL_PIXELFORMAT_YV12: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_YV12 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_XBGR32: C2RustUnnamed = 371595268;
pub const SDL_PIXELFORMAT_BGRX32: C2RustUnnamed = 370546692;
pub const SDL_PIXELFORMAT_XRGB32: C2RustUnnamed = 375789572;
pub const SDL_PIXELFORMAT_RGBX32: C2RustUnnamed = 374740996;
pub const SDL_PIXELFORMAT_ABGR32: C2RustUnnamed = 373694468;
pub const SDL_PIXELFORMAT_BGRA32: C2RustUnnamed = 372645892;
pub const SDL_PIXELFORMAT_ARGB32: C2RustUnnamed = 377888772;
pub const SDL_PIXELFORMAT_RGBA32: C2RustUnnamed = 376840196;
pub const SDL_PIXELFORMAT_ARGB2101010: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB2101010 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_BGRA8888: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_BGRA8888 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_ABGR8888: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_ABGR8888 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_RGBA8888: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGBA8888 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_ARGB8888: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_BGRX8888: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_BGRX8888 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_BGR888: C2RustUnnamed = 374740996;
pub const SDL_PIXELFORMAT_XBGR8888: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_XBGR8888 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_RGBX8888: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGBX8888 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_RGB888: C2RustUnnamed = 370546692;
pub const SDL_PIXELFORMAT_XRGB8888: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_XRGB8888 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_BGR24: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_BGR24 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_RGB24: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGB24 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_BGR565: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_BGR565 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_RGB565: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGB565 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_BGRA5551: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_BGRA5551 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_ABGR1555: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_ABGR1555 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_RGBA5551: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGBA5551 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_ARGB1555: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB1555 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_BGRA4444: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_BGRA4444 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_ABGR4444: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_ABGR4444 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_RGBA4444: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGBA4444 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_ARGB4444: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB4444 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_BGR555: C2RustUnnamed = 357764866;
pub const SDL_PIXELFORMAT_XBGR1555: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_XBGR1555 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_RGB555: C2RustUnnamed = 353570562;
pub const SDL_PIXELFORMAT_XRGB1555: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_XRGB1555 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_BGR444: C2RustUnnamed = 357698562;
pub const SDL_PIXELFORMAT_XBGR4444: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_XBGR4444 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_RGB444: C2RustUnnamed = 353504258;
pub const SDL_PIXELFORMAT_XRGB4444: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_XRGB4444 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_RGB332: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGB332 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_INDEX8: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_INDEX8 as C2RustUnnamed;
pub const SDL_PIXELFORMAT_INDEX4MSB: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_INDEX4MSB as C2RustUnnamed;
pub const SDL_PIXELFORMAT_INDEX4LSB: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_INDEX4LSB as C2RustUnnamed;
pub const SDL_PIXELFORMAT_INDEX2MSB: C2RustUnnamed = 471859712;
pub const SDL_PIXELFORMAT_INDEX2LSB: C2RustUnnamed = 470811136;
pub const SDL_PIXELFORMAT_INDEX1MSB: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_INDEX1MSB as C2RustUnnamed;
pub const SDL_PIXELFORMAT_INDEX1LSB: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_INDEX1LSB as C2RustUnnamed;
pub const SDL_PIXELFORMAT_UNKNOWN: C2RustUnnamed =
    sdl2_sys::SDL_PixelFormatEnum::SDL_PIXELFORMAT_UNKNOWN as C2RustUnnamed;
pub type C2RustUnnamed_0 = ::core::ffi::c_uint;
pub const SDL_WINDOW_INPUT_GRABBED: C2RustUnnamed_0 = 256;
pub const SDL_WINDOW_METAL: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_METAL as C2RustUnnamed_0;
pub const SDL_WINDOW_VULKAN: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_VULKAN as C2RustUnnamed_0;
pub const SDL_WINDOW_KEYBOARD_GRABBED: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_KEYBOARD_GRABBED as C2RustUnnamed_0;
pub const SDL_WINDOW_POPUP_MENU: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_POPUP_MENU as C2RustUnnamed_0;
pub const SDL_WINDOW_TOOLTIP: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_TOOLTIP as C2RustUnnamed_0;
pub const SDL_WINDOW_UTILITY: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_UTILITY as C2RustUnnamed_0;
pub const SDL_WINDOW_SKIP_TASKBAR: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_SKIP_TASKBAR as C2RustUnnamed_0;
pub const SDL_WINDOW_ALWAYS_ON_TOP: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as C2RustUnnamed_0;
pub const SDL_WINDOW_MOUSE_CAPTURE: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_MOUSE_CAPTURE as C2RustUnnamed_0;
pub const SDL_WINDOW_ALLOW_HIGHDPI: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_ALLOW_HIGHDPI as C2RustUnnamed_0;
pub const SDL_WINDOW_FOREIGN: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_FOREIGN as C2RustUnnamed_0;
pub const SDL_WINDOW_FULLSCREEN_DESKTOP: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_FULLSCREEN_DESKTOP as C2RustUnnamed_0;
pub const SDL_WINDOW_MOUSE_FOCUS: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_MOUSE_FOCUS as C2RustUnnamed_0;
pub const SDL_WINDOW_INPUT_FOCUS: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_INPUT_FOCUS as C2RustUnnamed_0;
pub const SDL_WINDOW_MOUSE_GRABBED: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_MOUSE_GRABBED as C2RustUnnamed_0;
pub const SDL_WINDOW_MAXIMIZED: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_MAXIMIZED as C2RustUnnamed_0;
pub const SDL_WINDOW_MINIMIZED: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_MINIMIZED as C2RustUnnamed_0;
pub const SDL_WINDOW_RESIZABLE: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_RESIZABLE as C2RustUnnamed_0;
pub const SDL_WINDOW_BORDERLESS: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_BORDERLESS as C2RustUnnamed_0;
pub const SDL_WINDOW_HIDDEN: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_HIDDEN as C2RustUnnamed_0;
pub const SDL_WINDOW_SHOWN: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_SHOWN as C2RustUnnamed_0;
pub const SDL_WINDOW_OPENGL: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_OPENGL as C2RustUnnamed_0;
pub const SDL_WINDOW_FULLSCREEN: C2RustUnnamed_0 =
    sdl2_sys::SDL_WindowFlags::SDL_WINDOW_FULLSCREEN as C2RustUnnamed_0;
pub const SDL_HITTEST_RESIZE_LEFT: SDL_HitTestResult =
    sdl2_sys::SDL_HitTestResult::SDL_HITTEST_RESIZE_LEFT;
pub const SDL_HITTEST_RESIZE_BOTTOMLEFT: SDL_HitTestResult =
    sdl2_sys::SDL_HitTestResult::SDL_HITTEST_RESIZE_BOTTOMLEFT;
pub const SDL_HITTEST_RESIZE_BOTTOM: SDL_HitTestResult =
    sdl2_sys::SDL_HitTestResult::SDL_HITTEST_RESIZE_BOTTOM;
pub const SDL_HITTEST_RESIZE_BOTTOMRIGHT: SDL_HitTestResult =
    sdl2_sys::SDL_HitTestResult::SDL_HITTEST_RESIZE_BOTTOMRIGHT;
pub const SDL_HITTEST_RESIZE_RIGHT: SDL_HitTestResult =
    sdl2_sys::SDL_HitTestResult::SDL_HITTEST_RESIZE_RIGHT;
pub const SDL_HITTEST_RESIZE_TOPRIGHT: SDL_HitTestResult =
    sdl2_sys::SDL_HitTestResult::SDL_HITTEST_RESIZE_TOPRIGHT;
pub const SDL_HITTEST_RESIZE_TOP: SDL_HitTestResult =
    sdl2_sys::SDL_HitTestResult::SDL_HITTEST_RESIZE_TOP;
pub const SDL_HITTEST_RESIZE_TOPLEFT: SDL_HitTestResult =
    sdl2_sys::SDL_HitTestResult::SDL_HITTEST_RESIZE_TOPLEFT;
pub const SDL_HITTEST_DRAGGABLE: SDL_HitTestResult =
    sdl2_sys::SDL_HitTestResult::SDL_HITTEST_DRAGGABLE;
pub const SDL_HITTEST_NORMAL: SDL_HitTestResult = sdl2_sys::SDL_HitTestResult::SDL_HITTEST_NORMAL;
pub use sm::types::SDL_Scancode;
pub const SDL_NUM_SCANCODES: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_NUM_SCANCODES as SDL_Scancode;
pub const SDL_SCANCODE_ENDCALL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_ENDCALL as SDL_Scancode;
pub const SDL_SCANCODE_CALL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_CALL as SDL_Scancode;
pub const SDL_SCANCODE_SOFTRIGHT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_SOFTRIGHT as SDL_Scancode;
pub const SDL_SCANCODE_SOFTLEFT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_SOFTLEFT as SDL_Scancode;
pub const SDL_SCANCODE_AUDIOFASTFORWARD: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AUDIOFASTFORWARD as SDL_Scancode;
pub const SDL_SCANCODE_AUDIOREWIND: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AUDIOREWIND as SDL_Scancode;
pub const SDL_SCANCODE_APP2: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_APP2 as SDL_Scancode;
pub const SDL_SCANCODE_APP1: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_APP1 as SDL_Scancode;
pub const SDL_SCANCODE_SLEEP: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_SLEEP as SDL_Scancode;
pub const SDL_SCANCODE_EJECT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_EJECT as SDL_Scancode;
pub const SDL_SCANCODE_KBDILLUMUP: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KBDILLUMUP as SDL_Scancode;
pub const SDL_SCANCODE_KBDILLUMDOWN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KBDILLUMDOWN as SDL_Scancode;
pub const SDL_SCANCODE_KBDILLUMTOGGLE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KBDILLUMTOGGLE as SDL_Scancode;
pub const SDL_SCANCODE_DISPLAYSWITCH: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_DISPLAYSWITCH as SDL_Scancode;
pub const SDL_SCANCODE_BRIGHTNESSUP: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_BRIGHTNESSUP as SDL_Scancode;
pub const SDL_SCANCODE_BRIGHTNESSDOWN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_BRIGHTNESSDOWN as SDL_Scancode;
pub const SDL_SCANCODE_AC_BOOKMARKS: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AC_BOOKMARKS as SDL_Scancode;
pub const SDL_SCANCODE_AC_REFRESH: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AC_REFRESH as SDL_Scancode;
pub const SDL_SCANCODE_AC_STOP: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AC_STOP as SDL_Scancode;
pub const SDL_SCANCODE_AC_FORWARD: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AC_FORWARD as SDL_Scancode;
pub const SDL_SCANCODE_AC_BACK: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AC_BACK as SDL_Scancode;
pub const SDL_SCANCODE_AC_HOME: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AC_HOME as SDL_Scancode;
pub const SDL_SCANCODE_AC_SEARCH: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AC_SEARCH as SDL_Scancode;
pub const SDL_SCANCODE_COMPUTER: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_COMPUTER as SDL_Scancode;
pub const SDL_SCANCODE_CALCULATOR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_CALCULATOR as SDL_Scancode;
pub const SDL_SCANCODE_MAIL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_MAIL as SDL_Scancode;
pub const SDL_SCANCODE_WWW: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_WWW as SDL_Scancode;
pub const SDL_SCANCODE_MEDIASELECT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_MEDIASELECT as SDL_Scancode;
pub const SDL_SCANCODE_AUDIOMUTE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AUDIOMUTE as SDL_Scancode;
pub const SDL_SCANCODE_AUDIOPLAY: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AUDIOPLAY as SDL_Scancode;
pub const SDL_SCANCODE_AUDIOSTOP: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AUDIOSTOP as SDL_Scancode;
pub const SDL_SCANCODE_AUDIOPREV: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AUDIOPREV as SDL_Scancode;
pub const SDL_SCANCODE_AUDIONEXT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AUDIONEXT as SDL_Scancode;
pub const SDL_SCANCODE_MODE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_MODE as SDL_Scancode;
pub const SDL_SCANCODE_RGUI: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_RGUI as SDL_Scancode;
pub const SDL_SCANCODE_RALT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_RALT as SDL_Scancode;
pub const SDL_SCANCODE_RSHIFT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_RSHIFT as SDL_Scancode;
pub const SDL_SCANCODE_RCTRL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_RCTRL as SDL_Scancode;
pub const SDL_SCANCODE_LGUI: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LGUI as SDL_Scancode;
pub const SDL_SCANCODE_LALT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LALT as SDL_Scancode;
pub const SDL_SCANCODE_LSHIFT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LSHIFT as SDL_Scancode;
pub const SDL_SCANCODE_LCTRL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LCTRL as SDL_Scancode;
pub const SDL_SCANCODE_KP_HEXADECIMAL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_HEXADECIMAL as SDL_Scancode;
pub const SDL_SCANCODE_KP_DECIMAL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_DECIMAL as SDL_Scancode;
pub const SDL_SCANCODE_KP_OCTAL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_OCTAL as SDL_Scancode;
pub const SDL_SCANCODE_KP_BINARY: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_BINARY as SDL_Scancode;
pub const SDL_SCANCODE_KP_CLEARENTRY: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_CLEARENTRY as SDL_Scancode;
pub const SDL_SCANCODE_KP_CLEAR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_CLEAR as SDL_Scancode;
pub const SDL_SCANCODE_KP_PLUSMINUS: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_PLUSMINUS as SDL_Scancode;
pub const SDL_SCANCODE_KP_MEMDIVIDE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_MEMDIVIDE as SDL_Scancode;
pub const SDL_SCANCODE_KP_MEMMULTIPLY: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_MEMMULTIPLY as SDL_Scancode;
pub const SDL_SCANCODE_KP_MEMSUBTRACT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_MEMSUBTRACT as SDL_Scancode;
pub const SDL_SCANCODE_KP_MEMADD: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_MEMADD as SDL_Scancode;
pub const SDL_SCANCODE_KP_MEMCLEAR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_MEMCLEAR as SDL_Scancode;
pub const SDL_SCANCODE_KP_MEMRECALL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_MEMRECALL as SDL_Scancode;
pub const SDL_SCANCODE_KP_MEMSTORE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_MEMSTORE as SDL_Scancode;
pub const SDL_SCANCODE_KP_EXCLAM: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_EXCLAM as SDL_Scancode;
pub const SDL_SCANCODE_KP_AT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_AT as SDL_Scancode;
pub const SDL_SCANCODE_KP_SPACE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_SPACE as SDL_Scancode;
pub const SDL_SCANCODE_KP_HASH: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_HASH as SDL_Scancode;
pub const SDL_SCANCODE_KP_COLON: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_COLON as SDL_Scancode;
pub const SDL_SCANCODE_KP_DBLVERTICALBAR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_DBLVERTICALBAR as SDL_Scancode;
pub const SDL_SCANCODE_KP_VERTICALBAR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_VERTICALBAR as SDL_Scancode;
pub const SDL_SCANCODE_KP_DBLAMPERSAND: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_DBLAMPERSAND as SDL_Scancode;
pub const SDL_SCANCODE_KP_AMPERSAND: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_AMPERSAND as SDL_Scancode;
pub const SDL_SCANCODE_KP_GREATER: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_GREATER as SDL_Scancode;
pub const SDL_SCANCODE_KP_LESS: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_LESS as SDL_Scancode;
pub const SDL_SCANCODE_KP_PERCENT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_PERCENT as SDL_Scancode;
pub const SDL_SCANCODE_KP_POWER: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_POWER as SDL_Scancode;
pub const SDL_SCANCODE_KP_XOR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_XOR as SDL_Scancode;
pub const SDL_SCANCODE_KP_F: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_F as SDL_Scancode;
pub const SDL_SCANCODE_KP_E: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_E as SDL_Scancode;
pub const SDL_SCANCODE_KP_D: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_D as SDL_Scancode;
pub const SDL_SCANCODE_KP_C: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_C as SDL_Scancode;
pub const SDL_SCANCODE_KP_B: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_B as SDL_Scancode;
pub const SDL_SCANCODE_KP_A: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_A as SDL_Scancode;
pub const SDL_SCANCODE_KP_BACKSPACE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_BACKSPACE as SDL_Scancode;
pub const SDL_SCANCODE_KP_TAB: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_TAB as SDL_Scancode;
pub const SDL_SCANCODE_KP_RIGHTBRACE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_RIGHTBRACE as SDL_Scancode;
pub const SDL_SCANCODE_KP_LEFTBRACE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_LEFTBRACE as SDL_Scancode;
pub const SDL_SCANCODE_KP_RIGHTPAREN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_RIGHTPAREN as SDL_Scancode;
pub const SDL_SCANCODE_KP_LEFTPAREN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_LEFTPAREN as SDL_Scancode;
pub const SDL_SCANCODE_CURRENCYSUBUNIT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_CURRENCYSUBUNIT as SDL_Scancode;
pub const SDL_SCANCODE_CURRENCYUNIT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_CURRENCYUNIT as SDL_Scancode;
pub const SDL_SCANCODE_DECIMALSEPARATOR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_DECIMALSEPARATOR as SDL_Scancode;
pub const SDL_SCANCODE_THOUSANDSSEPARATOR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_THOUSANDSSEPARATOR as SDL_Scancode;
pub const SDL_SCANCODE_KP_000: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_000 as SDL_Scancode;
pub const SDL_SCANCODE_KP_00: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_00 as SDL_Scancode;
pub const SDL_SCANCODE_EXSEL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_EXSEL as SDL_Scancode;
pub const SDL_SCANCODE_CRSEL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_CRSEL as SDL_Scancode;
pub const SDL_SCANCODE_CLEARAGAIN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_CLEARAGAIN as SDL_Scancode;
pub const SDL_SCANCODE_OPER: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_OPER as SDL_Scancode;
pub const SDL_SCANCODE_OUT: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_OUT as SDL_Scancode;
pub const SDL_SCANCODE_SEPARATOR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_SEPARATOR as SDL_Scancode;
pub const SDL_SCANCODE_RETURN2: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_RETURN2 as SDL_Scancode;
pub const SDL_SCANCODE_PRIOR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_PRIOR as SDL_Scancode;
pub const SDL_SCANCODE_CLEAR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_CLEAR as SDL_Scancode;
pub const SDL_SCANCODE_CANCEL: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_CANCEL as SDL_Scancode;
pub const SDL_SCANCODE_SYSREQ: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_SYSREQ as SDL_Scancode;
pub const SDL_SCANCODE_ALTERASE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_ALTERASE as SDL_Scancode;
pub const SDL_SCANCODE_LANG9: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LANG9 as SDL_Scancode;
pub const SDL_SCANCODE_LANG8: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LANG8 as SDL_Scancode;
pub const SDL_SCANCODE_LANG7: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LANG7 as SDL_Scancode;
pub const SDL_SCANCODE_LANG6: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LANG6 as SDL_Scancode;
pub const SDL_SCANCODE_LANG5: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LANG5 as SDL_Scancode;
pub const SDL_SCANCODE_LANG4: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LANG4 as SDL_Scancode;
pub const SDL_SCANCODE_LANG3: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LANG3 as SDL_Scancode;
pub const SDL_SCANCODE_LANG2: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LANG2 as SDL_Scancode;
pub const SDL_SCANCODE_LANG1: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LANG1 as SDL_Scancode;
pub const SDL_SCANCODE_INTERNATIONAL9: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_INTERNATIONAL9 as SDL_Scancode;
pub const SDL_SCANCODE_INTERNATIONAL8: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_INTERNATIONAL8 as SDL_Scancode;
pub const SDL_SCANCODE_INTERNATIONAL7: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_INTERNATIONAL7 as SDL_Scancode;
pub const SDL_SCANCODE_INTERNATIONAL6: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_INTERNATIONAL6 as SDL_Scancode;
pub const SDL_SCANCODE_INTERNATIONAL5: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_INTERNATIONAL5 as SDL_Scancode;
pub const SDL_SCANCODE_INTERNATIONAL4: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_INTERNATIONAL4 as SDL_Scancode;
pub const SDL_SCANCODE_INTERNATIONAL3: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_INTERNATIONAL3 as SDL_Scancode;
pub const SDL_SCANCODE_INTERNATIONAL2: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_INTERNATIONAL2 as SDL_Scancode;
pub const SDL_SCANCODE_INTERNATIONAL1: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_INTERNATIONAL1 as SDL_Scancode;
pub const SDL_SCANCODE_KP_EQUALSAS400: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_EQUALSAS400 as SDL_Scancode;
pub const SDL_SCANCODE_KP_COMMA: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_COMMA as SDL_Scancode;
pub const SDL_SCANCODE_VOLUMEDOWN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_VOLUMEDOWN as SDL_Scancode;
pub const SDL_SCANCODE_VOLUMEUP: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_VOLUMEUP as SDL_Scancode;
pub const SDL_SCANCODE_MUTE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_MUTE as SDL_Scancode;
pub const SDL_SCANCODE_FIND: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_FIND as SDL_Scancode;
pub const SDL_SCANCODE_PASTE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_PASTE as SDL_Scancode;
pub const SDL_SCANCODE_COPY: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_COPY as SDL_Scancode;
pub const SDL_SCANCODE_CUT: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_CUT as SDL_Scancode;
pub const SDL_SCANCODE_UNDO: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_UNDO as SDL_Scancode;
pub const SDL_SCANCODE_AGAIN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_AGAIN as SDL_Scancode;
pub const SDL_SCANCODE_STOP: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_STOP as SDL_Scancode;
pub const SDL_SCANCODE_SELECT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_SELECT as SDL_Scancode;
pub const SDL_SCANCODE_MENU: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_MENU as SDL_Scancode;
pub const SDL_SCANCODE_HELP: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_HELP as SDL_Scancode;
pub const SDL_SCANCODE_EXECUTE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_EXECUTE as SDL_Scancode;
pub const SDL_SCANCODE_F24: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F24 as SDL_Scancode;
pub const SDL_SCANCODE_F23: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F23 as SDL_Scancode;
pub const SDL_SCANCODE_F22: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F22 as SDL_Scancode;
pub const SDL_SCANCODE_F21: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F21 as SDL_Scancode;
pub const SDL_SCANCODE_F20: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F20 as SDL_Scancode;
pub const SDL_SCANCODE_F19: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F19 as SDL_Scancode;
pub const SDL_SCANCODE_F18: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F18 as SDL_Scancode;
pub const SDL_SCANCODE_F17: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F17 as SDL_Scancode;
pub const SDL_SCANCODE_F16: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F16 as SDL_Scancode;
pub const SDL_SCANCODE_F15: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F15 as SDL_Scancode;
pub const SDL_SCANCODE_F14: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F14 as SDL_Scancode;
pub const SDL_SCANCODE_F13: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F13 as SDL_Scancode;
pub const SDL_SCANCODE_KP_EQUALS: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_EQUALS as SDL_Scancode;
pub const SDL_SCANCODE_POWER: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_POWER as SDL_Scancode;
pub const SDL_SCANCODE_APPLICATION: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_APPLICATION as SDL_Scancode;
pub const SDL_SCANCODE_NONUSBACKSLASH: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_NONUSBACKSLASH as SDL_Scancode;
pub const SDL_SCANCODE_KP_PERIOD: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_PERIOD as SDL_Scancode;
pub const SDL_SCANCODE_KP_0: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_0 as SDL_Scancode;
pub const SDL_SCANCODE_KP_9: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_9 as SDL_Scancode;
pub const SDL_SCANCODE_KP_8: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_8 as SDL_Scancode;
pub const SDL_SCANCODE_KP_7: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_7 as SDL_Scancode;
pub const SDL_SCANCODE_KP_6: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_6 as SDL_Scancode;
pub const SDL_SCANCODE_KP_5: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_5 as SDL_Scancode;
pub const SDL_SCANCODE_KP_4: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_4 as SDL_Scancode;
pub const SDL_SCANCODE_KP_3: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_3 as SDL_Scancode;
pub const SDL_SCANCODE_KP_2: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_2 as SDL_Scancode;
pub const SDL_SCANCODE_KP_1: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_1 as SDL_Scancode;
pub const SDL_SCANCODE_KP_ENTER: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_ENTER as SDL_Scancode;
pub const SDL_SCANCODE_KP_PLUS: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_PLUS as SDL_Scancode;
pub const SDL_SCANCODE_KP_MINUS: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_MINUS as SDL_Scancode;
pub const SDL_SCANCODE_KP_MULTIPLY: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_MULTIPLY as SDL_Scancode;
pub const SDL_SCANCODE_KP_DIVIDE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_KP_DIVIDE as SDL_Scancode;
pub const SDL_SCANCODE_NUMLOCKCLEAR: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_NUMLOCKCLEAR as SDL_Scancode;
pub const SDL_SCANCODE_UP: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_UP as SDL_Scancode;
pub const SDL_SCANCODE_DOWN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_DOWN as SDL_Scancode;
pub const SDL_SCANCODE_LEFT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LEFT as SDL_Scancode;
pub const SDL_SCANCODE_RIGHT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_RIGHT as SDL_Scancode;
pub const SDL_SCANCODE_PAGEDOWN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_PAGEDOWN as SDL_Scancode;
pub const SDL_SCANCODE_END: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_END as SDL_Scancode;
pub const SDL_SCANCODE_DELETE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_DELETE as SDL_Scancode;
pub const SDL_SCANCODE_PAGEUP: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_PAGEUP as SDL_Scancode;
pub const SDL_SCANCODE_HOME: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_HOME as SDL_Scancode;
pub const SDL_SCANCODE_INSERT: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_INSERT as SDL_Scancode;
pub const SDL_SCANCODE_PAUSE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_PAUSE as SDL_Scancode;
pub const SDL_SCANCODE_SCROLLLOCK: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_SCROLLLOCK as SDL_Scancode;
pub const SDL_SCANCODE_PRINTSCREEN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_PRINTSCREEN as SDL_Scancode;
pub const SDL_SCANCODE_F12: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F12 as SDL_Scancode;
pub const SDL_SCANCODE_F11: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F11 as SDL_Scancode;
pub const SDL_SCANCODE_F10: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F10 as SDL_Scancode;
pub const SDL_SCANCODE_F9: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F9 as SDL_Scancode;
pub const SDL_SCANCODE_F8: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F8 as SDL_Scancode;
pub const SDL_SCANCODE_F7: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F7 as SDL_Scancode;
pub const SDL_SCANCODE_F6: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F6 as SDL_Scancode;
pub const SDL_SCANCODE_F5: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F5 as SDL_Scancode;
pub const SDL_SCANCODE_F4: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F4 as SDL_Scancode;
pub const SDL_SCANCODE_F3: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F3 as SDL_Scancode;
pub const SDL_SCANCODE_F2: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F2 as SDL_Scancode;
pub const SDL_SCANCODE_F1: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F1 as SDL_Scancode;
pub const SDL_SCANCODE_CAPSLOCK: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_CAPSLOCK as SDL_Scancode;
pub const SDL_SCANCODE_SLASH: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_SLASH as SDL_Scancode;
pub const SDL_SCANCODE_PERIOD: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_PERIOD as SDL_Scancode;
pub const SDL_SCANCODE_COMMA: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_COMMA as SDL_Scancode;
pub const SDL_SCANCODE_GRAVE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_GRAVE as SDL_Scancode;
pub const SDL_SCANCODE_APOSTROPHE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_APOSTROPHE as SDL_Scancode;
pub const SDL_SCANCODE_SEMICOLON: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_SEMICOLON as SDL_Scancode;
pub const SDL_SCANCODE_NONUSHASH: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_NONUSHASH as SDL_Scancode;
pub const SDL_SCANCODE_BACKSLASH: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_BACKSLASH as SDL_Scancode;
pub const SDL_SCANCODE_RIGHTBRACKET: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_RIGHTBRACKET as SDL_Scancode;
pub const SDL_SCANCODE_LEFTBRACKET: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_LEFTBRACKET as SDL_Scancode;
pub const SDL_SCANCODE_EQUALS: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_EQUALS as SDL_Scancode;
pub const SDL_SCANCODE_MINUS: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_MINUS as SDL_Scancode;
pub const SDL_SCANCODE_SPACE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_SPACE as SDL_Scancode;
pub const SDL_SCANCODE_TAB: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_TAB as SDL_Scancode;
pub const SDL_SCANCODE_BACKSPACE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_BACKSPACE as SDL_Scancode;
pub const SDL_SCANCODE_ESCAPE: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_ESCAPE as SDL_Scancode;
pub const SDL_SCANCODE_RETURN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_RETURN as SDL_Scancode;
pub const SDL_SCANCODE_0: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_0 as SDL_Scancode;
pub const SDL_SCANCODE_9: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_9 as SDL_Scancode;
pub const SDL_SCANCODE_8: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_8 as SDL_Scancode;
pub const SDL_SCANCODE_7: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_7 as SDL_Scancode;
pub const SDL_SCANCODE_6: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_6 as SDL_Scancode;
pub const SDL_SCANCODE_5: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_5 as SDL_Scancode;
pub const SDL_SCANCODE_4: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_4 as SDL_Scancode;
pub const SDL_SCANCODE_3: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_3 as SDL_Scancode;
pub const SDL_SCANCODE_2: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_2 as SDL_Scancode;
pub const SDL_SCANCODE_1: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_1 as SDL_Scancode;
pub const SDL_SCANCODE_Z: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_Z as SDL_Scancode;
pub const SDL_SCANCODE_Y: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_Y as SDL_Scancode;
pub const SDL_SCANCODE_X: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_X as SDL_Scancode;
pub const SDL_SCANCODE_W: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_W as SDL_Scancode;
pub const SDL_SCANCODE_V: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_V as SDL_Scancode;
pub const SDL_SCANCODE_U: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_U as SDL_Scancode;
pub const SDL_SCANCODE_T: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_T as SDL_Scancode;
pub const SDL_SCANCODE_S: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_S as SDL_Scancode;
pub const SDL_SCANCODE_R: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_R as SDL_Scancode;
pub const SDL_SCANCODE_Q: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_Q as SDL_Scancode;
pub const SDL_SCANCODE_P: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_P as SDL_Scancode;
pub const SDL_SCANCODE_O: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_O as SDL_Scancode;
pub const SDL_SCANCODE_N: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_N as SDL_Scancode;
pub const SDL_SCANCODE_M: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_M as SDL_Scancode;
pub const SDL_SCANCODE_L: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_L as SDL_Scancode;
pub const SDL_SCANCODE_K: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_K as SDL_Scancode;
pub const SDL_SCANCODE_J: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_J as SDL_Scancode;
pub const SDL_SCANCODE_I: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_I as SDL_Scancode;
pub const SDL_SCANCODE_H: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_H as SDL_Scancode;
pub const SDL_SCANCODE_G: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_G as SDL_Scancode;
pub const SDL_SCANCODE_F: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_F as SDL_Scancode;
pub const SDL_SCANCODE_E: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_E as SDL_Scancode;
pub const SDL_SCANCODE_D: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_D as SDL_Scancode;
pub const SDL_SCANCODE_C: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_C as SDL_Scancode;
pub const SDL_SCANCODE_B: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_B as SDL_Scancode;
pub const SDL_SCANCODE_A: SDL_Scancode = sdl2_sys::SDL_Scancode::SDL_SCANCODE_A as SDL_Scancode;
pub const SDL_SCANCODE_UNKNOWN: SDL_Scancode =
    sdl2_sys::SDL_Scancode::SDL_SCANCODE_UNKNOWN as SDL_Scancode;
pub use sm::types::{SDL_Keycode, SDL_Keymod};
pub const KMOD_RESERVED: SDL_Keymod = 32768;
pub const KMOD_GUI: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_GUI as SDL_Keymod;
pub const KMOD_ALT: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_ALT as SDL_Keymod;
pub const KMOD_SHIFT: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_SHIFT as SDL_Keymod;
pub const KMOD_CTRL: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_CTRL as SDL_Keymod;
pub const KMOD_SCROLL: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_SCROLL as SDL_Keymod;
pub const KMOD_MODE: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_MODE as SDL_Keymod;
pub const KMOD_CAPS: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_CAPS as SDL_Keymod;
pub const KMOD_NUM: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_NUM as SDL_Keymod;
pub const KMOD_RGUI: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_RGUI as SDL_Keymod;
pub const KMOD_LGUI: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_LGUI as SDL_Keymod;
pub const KMOD_RALT: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_RALT as SDL_Keymod;
pub const KMOD_LALT: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_LALT as SDL_Keymod;
pub const KMOD_RCTRL: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_RCTRL as SDL_Keymod;
pub const KMOD_LCTRL: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_LCTRL as SDL_Keymod;
pub const KMOD_RSHIFT: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_RSHIFT as SDL_Keymod;
pub const KMOD_LSHIFT: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_LSHIFT as SDL_Keymod;
pub const KMOD_NONE: SDL_Keymod = sdl2_sys::SDL_Keymod::KMOD_NONE as SDL_Keymod;
pub use sm::types::{SDL_JoystickID, SDL_JoystickPowerLevel};
pub const SDL_JOYSTICK_POWER_MAX: SDL_JoystickPowerLevel =
    sdl2_sys::SDL_JoystickPowerLevel::SDL_JOYSTICK_POWER_MAX as SDL_JoystickPowerLevel;
pub const SDL_JOYSTICK_POWER_WIRED: SDL_JoystickPowerLevel =
    sdl2_sys::SDL_JoystickPowerLevel::SDL_JOYSTICK_POWER_WIRED as SDL_JoystickPowerLevel;
pub const SDL_JOYSTICK_POWER_FULL: SDL_JoystickPowerLevel =
    sdl2_sys::SDL_JoystickPowerLevel::SDL_JOYSTICK_POWER_FULL as SDL_JoystickPowerLevel;
pub const SDL_JOYSTICK_POWER_MEDIUM: SDL_JoystickPowerLevel =
    sdl2_sys::SDL_JoystickPowerLevel::SDL_JOYSTICK_POWER_MEDIUM as SDL_JoystickPowerLevel;
pub const SDL_JOYSTICK_POWER_LOW: SDL_JoystickPowerLevel =
    sdl2_sys::SDL_JoystickPowerLevel::SDL_JOYSTICK_POWER_LOW as SDL_JoystickPowerLevel;
pub const SDL_JOYSTICK_POWER_EMPTY: SDL_JoystickPowerLevel =
    sdl2_sys::SDL_JoystickPowerLevel::SDL_JOYSTICK_POWER_EMPTY as SDL_JoystickPowerLevel;
pub const SDL_JOYSTICK_POWER_UNKNOWN: SDL_JoystickPowerLevel =
    sdl2_sys::SDL_JoystickPowerLevel::SDL_JOYSTICK_POWER_UNKNOWN as SDL_JoystickPowerLevel;
pub type C2RustUnnamed_1 = ::core::ffi::c_int;
pub const SDL_CONTROLLER_AXIS_MAX: C2RustUnnamed_1 =
    sdl2_sys::SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_MAX as C2RustUnnamed_1;
pub const SDL_CONTROLLER_AXIS_TRIGGERRIGHT: C2RustUnnamed_1 =
    sdl2_sys::SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERRIGHT as C2RustUnnamed_1;
pub const SDL_CONTROLLER_AXIS_TRIGGERLEFT: C2RustUnnamed_1 =
    sdl2_sys::SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERLEFT as C2RustUnnamed_1;
pub const SDL_CONTROLLER_AXIS_RIGHTY: C2RustUnnamed_1 =
    sdl2_sys::SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTY as C2RustUnnamed_1;
pub const SDL_CONTROLLER_AXIS_RIGHTX: C2RustUnnamed_1 =
    sdl2_sys::SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTX as C2RustUnnamed_1;
pub const SDL_CONTROLLER_AXIS_LEFTY: C2RustUnnamed_1 =
    sdl2_sys::SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY as C2RustUnnamed_1;
pub const SDL_CONTROLLER_AXIS_LEFTX: C2RustUnnamed_1 =
    sdl2_sys::SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTX as C2RustUnnamed_1;
pub const SDL_CONTROLLER_AXIS_INVALID: C2RustUnnamed_1 =
    sdl2_sys::SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_INVALID as C2RustUnnamed_1;
pub type C2RustUnnamed_2 = ::core::ffi::c_int;
pub const SDL_CONTROLLER_BUTTON_MAX: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_MAX as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_TOUCHPAD: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_TOUCHPAD as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_PADDLE4: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_PADDLE4 as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_PADDLE3: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_PADDLE3 as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_PADDLE2: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_PADDLE2 as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_PADDLE1: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_PADDLE1 as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_MISC1: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_MISC1 as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_DPAD_RIGHT: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_RIGHT as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_DPAD_LEFT: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_LEFT as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_DPAD_DOWN: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_DOWN as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_DPAD_UP: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_UP as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_RIGHTSHOULDER: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSHOULDER as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_LEFTSHOULDER: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSHOULDER as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_RIGHTSTICK: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSTICK as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_LEFTSTICK: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSTICK as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_START: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_START as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_GUIDE: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_GUIDE as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_BACK: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_BACK as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_Y: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_Y as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_X: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_X as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_B: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_B as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_A: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_A as C2RustUnnamed_2;
pub const SDL_CONTROLLER_BUTTON_INVALID: C2RustUnnamed_2 =
    sdl2_sys::SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_INVALID as C2RustUnnamed_2;
pub use sm::types::{SDL_FingerID, SDL_GestureID, SDL_TouchID};
pub type C2RustUnnamed_3 = ::core::ffi::c_uint;
pub const SDL_LASTEVENT: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_LASTEVENT as C2RustUnnamed_3;
pub const SDL_USEREVENT: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_USEREVENT as C2RustUnnamed_3;
pub const SDL_POLLSENTINEL: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_POLLSENTINEL as C2RustUnnamed_3;
pub const SDL_RENDER_DEVICE_RESET: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_RENDER_DEVICE_RESET as C2RustUnnamed_3;
pub const SDL_RENDER_TARGETS_RESET: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_RENDER_TARGETS_RESET as C2RustUnnamed_3;
pub const SDL_SENSORUPDATE: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_SENSORUPDATE as C2RustUnnamed_3;
pub const SDL_AUDIODEVICEREMOVED: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_AUDIODEVICEREMOVED as C2RustUnnamed_3;
pub const SDL_AUDIODEVICEADDED: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_AUDIODEVICEADDED as C2RustUnnamed_3;
pub const SDL_DROPCOMPLETE: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_DROPCOMPLETE as C2RustUnnamed_3;
pub const SDL_DROPBEGIN: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_DROPBEGIN as C2RustUnnamed_3;
pub const SDL_DROPTEXT: C2RustUnnamed_3 = sdl2_sys::SDL_EventType::SDL_DROPTEXT as C2RustUnnamed_3;
pub const SDL_DROPFILE: C2RustUnnamed_3 = sdl2_sys::SDL_EventType::SDL_DROPFILE as C2RustUnnamed_3;
pub const SDL_CLIPBOARDUPDATE: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CLIPBOARDUPDATE as C2RustUnnamed_3;
pub const SDL_MULTIGESTURE: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_MULTIGESTURE as C2RustUnnamed_3;
pub const SDL_DOLLARRECORD: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_DOLLARRECORD as C2RustUnnamed_3;
pub const SDL_DOLLARGESTURE: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_DOLLARGESTURE as C2RustUnnamed_3;
pub const SDL_FINGERMOTION: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_FINGERMOTION as C2RustUnnamed_3;
pub const SDL_FINGERUP: C2RustUnnamed_3 = sdl2_sys::SDL_EventType::SDL_FINGERUP as C2RustUnnamed_3;
pub const SDL_FINGERDOWN: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_FINGERDOWN as C2RustUnnamed_3;
pub const SDL_CONTROLLERSTEAMHANDLEUPDATED: C2RustUnnamed_3 = 1627;
pub const SDL_CONTROLLERUPDATECOMPLETE_RESERVED_FOR_SDL3: C2RustUnnamed_3 = 1626;
pub const SDL_CONTROLLERSENSORUPDATE: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CONTROLLERSENSORUPDATE as C2RustUnnamed_3;
pub const SDL_CONTROLLERTOUCHPADUP: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CONTROLLERTOUCHPADUP as C2RustUnnamed_3;
pub const SDL_CONTROLLERTOUCHPADMOTION: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CONTROLLERTOUCHPADMOTION as C2RustUnnamed_3;
pub const SDL_CONTROLLERTOUCHPADDOWN: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CONTROLLERTOUCHPADDOWN as C2RustUnnamed_3;
pub const SDL_CONTROLLERDEVICEREMAPPED: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CONTROLLERDEVICEREMAPPED as C2RustUnnamed_3;
pub const SDL_CONTROLLERDEVICEREMOVED: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CONTROLLERDEVICEREMOVED as C2RustUnnamed_3;
pub const SDL_CONTROLLERDEVICEADDED: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CONTROLLERDEVICEADDED as C2RustUnnamed_3;
pub const SDL_CONTROLLERBUTTONUP: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CONTROLLERBUTTONUP as C2RustUnnamed_3;
pub const SDL_CONTROLLERBUTTONDOWN: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CONTROLLERBUTTONDOWN as C2RustUnnamed_3;
pub const SDL_CONTROLLERAXISMOTION: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_CONTROLLERAXISMOTION as C2RustUnnamed_3;
pub const SDL_JOYBATTERYUPDATED: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_JOYBATTERYUPDATED as C2RustUnnamed_3;
pub const SDL_JOYDEVICEREMOVED: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_JOYDEVICEREMOVED as C2RustUnnamed_3;
pub const SDL_JOYDEVICEADDED: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_JOYDEVICEADDED as C2RustUnnamed_3;
pub const SDL_JOYBUTTONUP: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_JOYBUTTONUP as C2RustUnnamed_3;
pub const SDL_JOYBUTTONDOWN: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_JOYBUTTONDOWN as C2RustUnnamed_3;
pub const SDL_JOYHATMOTION: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_JOYHATMOTION as C2RustUnnamed_3;
pub const SDL_JOYBALLMOTION: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_JOYBALLMOTION as C2RustUnnamed_3;
pub const SDL_JOYAXISMOTION: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_JOYAXISMOTION as C2RustUnnamed_3;
pub const SDL_MOUSEWHEEL: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_MOUSEWHEEL as C2RustUnnamed_3;
pub const SDL_MOUSEBUTTONUP: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_MOUSEBUTTONUP as C2RustUnnamed_3;
pub const SDL_MOUSEBUTTONDOWN: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_MOUSEBUTTONDOWN as C2RustUnnamed_3;
pub const SDL_MOUSEMOTION: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_MOUSEMOTION as C2RustUnnamed_3;
pub const SDL_TEXTEDITING_EXT: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_TEXTEDITING_EXT as C2RustUnnamed_3;
pub const SDL_KEYMAPCHANGED: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_KEYMAPCHANGED as C2RustUnnamed_3;
pub const SDL_TEXTINPUT: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_TEXTINPUT as C2RustUnnamed_3;
pub const SDL_TEXTEDITING: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_TEXTEDITING as C2RustUnnamed_3;
pub const SDL_KEYUP: C2RustUnnamed_3 = sdl2_sys::SDL_EventType::SDL_KEYUP as C2RustUnnamed_3;
pub const SDL_KEYDOWN: C2RustUnnamed_3 = sdl2_sys::SDL_EventType::SDL_KEYDOWN as C2RustUnnamed_3;
pub const SDL_SYSWMEVENT: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_SYSWMEVENT as C2RustUnnamed_3;
pub const SDL_WINDOWEVENT: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_WINDOWEVENT as C2RustUnnamed_3;
pub const SDL_DISPLAYEVENT: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_DISPLAYEVENT as C2RustUnnamed_3;
pub const SDL_LOCALECHANGED: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_LOCALECHANGED as C2RustUnnamed_3;
pub const SDL_APP_DIDENTERFOREGROUND: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_APP_DIDENTERFOREGROUND as C2RustUnnamed_3;
pub const SDL_APP_WILLENTERFOREGROUND: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_APP_WILLENTERFOREGROUND as C2RustUnnamed_3;
pub const SDL_APP_DIDENTERBACKGROUND: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_APP_DIDENTERBACKGROUND as C2RustUnnamed_3;
pub const SDL_APP_WILLENTERBACKGROUND: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_APP_WILLENTERBACKGROUND as C2RustUnnamed_3;
pub const SDL_APP_LOWMEMORY: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_APP_LOWMEMORY as C2RustUnnamed_3;
pub const SDL_APP_TERMINATING: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_APP_TERMINATING as C2RustUnnamed_3;
pub const SDL_QUIT: C2RustUnnamed_3 = sdl2_sys::SDL_EventType::SDL_QUIT as C2RustUnnamed_3;
pub const SDL_FIRSTEVENT: C2RustUnnamed_3 =
    sdl2_sys::SDL_EventType::SDL_FIRSTEVENT as C2RustUnnamed_3;
pub type C2RustUnnamed_4 = ::core::ffi::c_uint;
pub const SDL_MESSAGEBOX_BUTTONS_RIGHT_TO_LEFT: C2RustUnnamed_4 =
    sdl2_sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_BUTTONS_RIGHT_TO_LEFT as C2RustUnnamed_4;
pub const SDL_MESSAGEBOX_BUTTONS_LEFT_TO_RIGHT: C2RustUnnamed_4 =
    sdl2_sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_BUTTONS_LEFT_TO_RIGHT as C2RustUnnamed_4;
pub const SDL_MESSAGEBOX_INFORMATION: C2RustUnnamed_4 =
    sdl2_sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_INFORMATION as C2RustUnnamed_4;
pub const SDL_MESSAGEBOX_WARNING: C2RustUnnamed_4 =
    sdl2_sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_WARNING as C2RustUnnamed_4;
pub const SDL_MESSAGEBOX_ERROR: C2RustUnnamed_4 =
    sdl2_sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_ERROR as C2RustUnnamed_4;
pub type C2RustUnnamed_5 = ::core::ffi::c_uint;
pub const SDL_RENDERER_TARGETTEXTURE: C2RustUnnamed_5 =
    sdl2_sys::SDL_RendererFlags::SDL_RENDERER_TARGETTEXTURE as C2RustUnnamed_5;
pub const SDL_RENDERER_PRESENTVSYNC: C2RustUnnamed_5 =
    sdl2_sys::SDL_RendererFlags::SDL_RENDERER_PRESENTVSYNC as C2RustUnnamed_5;
pub const SDL_RENDERER_ACCELERATED: C2RustUnnamed_5 =
    sdl2_sys::SDL_RendererFlags::SDL_RENDERER_ACCELERATED as C2RustUnnamed_5;
pub const SDL_RENDERER_SOFTWARE: C2RustUnnamed_5 =
    sdl2_sys::SDL_RendererFlags::SDL_RENDERER_SOFTWARE as C2RustUnnamed_5;
pub type C2RustUnnamed_6 = ::core::ffi::c_uint;
pub const SDL_TEXTUREACCESS_TARGET: C2RustUnnamed_6 =
    sdl2_sys::SDL_TextureAccess::SDL_TEXTUREACCESS_TARGET as C2RustUnnamed_6;
pub const SDL_TEXTUREACCESS_STREAMING: C2RustUnnamed_6 =
    sdl2_sys::SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as C2RustUnnamed_6;
pub const SDL_TEXTUREACCESS_STATIC: C2RustUnnamed_6 =
    sdl2_sys::SDL_TextureAccess::SDL_TEXTUREACCESS_STATIC as C2RustUnnamed_6;
pub use sm::types::{
    Apu, ApuHistory as C2RustUnnamed_7, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer,
    WindowLayer,
};
pub type C2RustUnnamed_8 = ::core::ffi::c_uint;
pub const kPpuRenderFlags_NoSpriteLimits: C2RustUnnamed_8 = 8;
pub const kPpuRenderFlags_Height240: C2RustUnnamed_8 = 4;
pub const kPpuRenderFlags_4x4Mode7: C2RustUnnamed_8 = 2;
pub const kPpuRenderFlags_NewRenderer: C2RustUnnamed_8 = 1;
pub use sm::types::{int16, uint16, uint32, uint64, uint8, SaveLoadOperation as C2RustUnnamed_9};
pub const kSaveLoad_Replay: C2RustUnnamed_9 = 3;
pub const kSaveLoad_Load: C2RustUnnamed_9 = 2;
pub const kSaveLoad_Save: C2RustUnnamed_9 = 1;
pub use sm::types::{Channel, KeyAction as C2RustUnnamed_10, SfxChan, SfxRegs, SpcPlayer};
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
pub use sm::types::OutputMethod as C2RustUnnamed_11;
pub const kOutputMethod_OpenGL: C2RustUnnamed_11 = 2;
pub const kOutputMethod_SDLSoftware: C2RustUnnamed_11 = 1;
pub const kOutputMethod_SDL: C2RustUnnamed_11 = 0;
pub use sm::types::{Config, GamepadButton as C2RustUnnamed_12};
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
pub use sm::types::RendererFuncs;
pub const kMaxWindowScale: C2RustUnnamed_13 = 10;
pub type C2RustUnnamed_13 = ::core::ffi::c_uint;
pub const kDefaultSamples: C2RustUnnamed_13 = 2048;
pub const kDefaultChannels: C2RustUnnamed_13 = 2;
pub const kDefaultFreq: C2RustUnnamed_13 = 44100;
pub const kDefaultFullscreen: C2RustUnnamed_13 = 0;
pub const AUDIO_S16LSB: ::core::ffi::c_int = sdl2_sys::AUDIO_S16LSB as ::core::ffi::c_int;
pub const AUDIO_S16: ::core::ffi::c_int = sdl2_sys::AUDIO_S16 as ::core::ffi::c_int;
pub const SDL_MIX_MAXVOLUME: ::core::ffi::c_int = sdl2_sys::SDL_MIX_MAXVOLUME as ::core::ffi::c_int;
pub const SDL_WINDOWPOS_UNDEFINED_MASK: ::core::ffi::c_uint =
    sdl2_sys::SDL_WINDOWPOS_UNDEFINED_MASK as ::core::ffi::c_uint;
pub const SDL_WINDOWPOS_UNDEFINED: ::core::ffi::c_uint =
    SDL_WINDOWPOS_UNDEFINED_MASK | 0 as ::core::ffi::c_uint;
pub const SDL_WINDOWPOS_CENTERED_MASK: ::core::ffi::c_uint =
    sdl2_sys::SDL_WINDOWPOS_CENTERED_MASK as ::core::ffi::c_uint;
pub const SDL_WINDOWPOS_CENTERED: ::core::ffi::c_uint =
    SDL_WINDOWPOS_CENTERED_MASK | 0 as ::core::ffi::c_uint;
pub const SDL_BUTTON_LEFT: ::core::ffi::c_int = sdl2_sys::SDL_BUTTON_LEFT as ::core::ffi::c_int;
pub const SDL_PRESSED: ::core::ffi::c_int = sdl2_sys::SDL_PRESSED as ::core::ffi::c_int;
pub const SDL_INIT_AUDIO: ::core::ffi::c_uint = sdl2_sys::SDL_INIT_AUDIO as ::core::ffi::c_uint;
pub const SDL_INIT_VIDEO: ::core::ffi::c_uint = sdl2_sys::SDL_INIT_VIDEO as ::core::ffi::c_uint;
pub const SDL_INIT_GAMECONTROLLER: ::core::ffi::c_uint =
    sdl2_sys::SDL_INIT_GAMECONTROLLER as ::core::ffi::c_uint;
pub const kDebugFlag: ::core::ffi::c_int = 0;
#[inline]
unsafe fn IntMin(a: ::core::ffi::c_int, b: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
unsafe fn IntMax(a: ::core::ffi::c_int, b: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if a > b {
        a
    } else {
        b
    }
}
pub static mut g_is_turbo: bool = false;
pub static mut g_want_dump_memmap_flags: bool = false;
static mut g_pixels: [uint8_t; 245760] = [0; 245760];
static mut g_my_pixels: [uint8_t; 245760] = [0; 245760];
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
unsafe fn host_die(error: *const ::core::ffi::c_char) {
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
unsafe fn host_warning(error: *const ::core::ffi::c_char) {
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
pub unsafe fn ChangeWindowScale(scale_step: ::core::ffi::c_int) {
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
        let mw: ::core::ffi::c_int =
            (bounds.w - bl - br + g_snes_width / 4 as ::core::ffi::c_int) / g_snes_width;
        let mh: ::core::ffi::c_int =
            (bounds.h - bt - bb + g_snes_height / 4 as ::core::ffi::c_int) / g_snes_height;
        max_scale = IntMin(mw, mh);
    }
    let new_scale: ::core::ffi::c_int = IntMax(
        IntMin(
            g_current_window_scale as ::core::ffi::c_int + scale_step,
            max_scale,
        ),
        1 as ::core::ffi::c_int,
    );
    g_current_window_scale = new_scale as uint8;
    let w: ::core::ffi::c_int = new_scale * g_snes_width;
    let h: ::core::ffi::c_int = new_scale * g_snes_height;
    SDL_SetWindowSize(g_window as *mut SDL_Window, w, h);
    if bt >= 0 as ::core::ffi::c_int {
        let mut mx: ::core::ffi::c_int = 0;
        let mut my: ::core::ffi::c_int = 0;
        SDL_GetGlobalMouseState(&raw mut mx, &raw mut my);
        let wx: ::core::ffi::c_int = IntMax(
            IntMin(
                mx - w / 2 as ::core::ffi::c_int,
                bounds.x + bounds.w - bl - br - w,
            ),
            bounds.x + bl,
        );
        let wy: ::core::ffi::c_int = IntMax(
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
pub const RESIZE_BORDER: ::core::ffi::c_int = 20;
unsafe extern "C" fn HitTestCallback(
    win: *mut SDL_Window,
    pt: *const SDL_Point,
    data: *mut ::core::ffi::c_void,
) -> SDL_HitTestResult {
    let flags: uint32 = SDL_GetWindowFlags(win as *mut SDL_Window) as uint32;
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
        return if (*pt).x < RESIZE_BORDER {
            SDL_HITTEST_RESIZE_TOPLEFT
        } else if (*pt).x >= w - RESIZE_BORDER {
            SDL_HITTEST_RESIZE_TOPRIGHT
        } else {
            SDL_HITTEST_RESIZE_TOP
        };
    } else if (*pt).y >= h - RESIZE_BORDER {
        return if (*pt).x < RESIZE_BORDER {
            SDL_HITTEST_RESIZE_BOTTOMLEFT
        } else if (*pt).x >= w - RESIZE_BORDER {
            SDL_HITTEST_RESIZE_BOTTOMRIGHT
        } else {
            SDL_HITTEST_RESIZE_BOTTOM
        };
    } else if (*pt).x < RESIZE_BORDER {
        return SDL_HITTEST_RESIZE_LEFT;
    } else if (*pt).x >= w - RESIZE_BORDER {
        return SDL_HITTEST_RESIZE_RIGHT;
    }
    SDL_HITTEST_NORMAL
}
pub unsafe fn RtlDrawPpuFrame(
    pixel_buffer: *mut uint8,
    pitch: size_t,
    render_flags: uint32,
    game: &mut Game,
) {
    let ppu_pixels: *mut uint8 = if game.g_other_image as ::core::ffi::c_int != 0 {
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
unsafe fn DrawPpuFrameWithPerf(game: &mut Game) {
    let render_scale: ::core::ffi::c_int =
        PpuGetCurrentRenderScale((*game.g_snes).ppu, g_ppu_render_flags as uint32_t);
    let mut pixel_buffer: *mut uint8 = ::core::ptr::null_mut::<uint8>();
    let mut pitch: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    g_renderer_funcs
        .BeginDraw
        .expect("non-null function pointer")(
        g_snes_width * render_scale,
        g_snes_height * render_scale,
        &raw mut pixel_buffer,
        &raw mut pitch,
        game,
    );
    if g_display_perf as ::core::ffi::c_int != 0
        || g_config.display_perf_title as ::core::ffi::c_int != 0
    {
        static mut history: [::core::ffi::c_float; 64] = [0.; 64];
        static mut average: ::core::ffi::c_float = 0.;
        static mut history_pos: ::core::ffi::c_int = 0;
        let before: uint64 = SDL_GetPerformanceCounter() as uint64;
        RtlDrawPpuFrame(
            pixel_buffer,
            pitch as size_t,
            g_ppu_render_flags as uint32,
            game,
        );
        let after: uint64 = SDL_GetPerformanceCounter() as uint64;
        let v: ::core::ffi::c_float = (SDL_GetPerformanceFrequency() as ::core::ffi::c_double
            / after.wrapping_sub(before) as ::core::ffi::c_double)
            as ::core::ffi::c_float;
        average += v - history[history_pos as usize];
        history[history_pos as usize] = v;
        history_pos = history_pos + 1 as ::core::ffi::c_int & 63 as ::core::ffi::c_int;
        g_curr_fps = (average * (1.0f32 / 64 as ::core::ffi::c_int as ::core::ffi::c_float))
            as ::core::ffi::c_int;
    } else {
        RtlDrawPpuFrame(
            pixel_buffer,
            pitch as size_t,
            g_ppu_render_flags as uint32,
            game,
        );
    }
    if g_display_perf {
        RenderNumber(
            pixel_buffer.offset((pitch * render_scale) as isize),
            pitch as size_t,
            g_curr_fps,
            (render_scale == 4 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8,
        );
    }
    if game.g_got_mismatch_count != 0 {
        RenderNumber(
            pixel_buffer.offset((pitch * render_scale) as isize),
            pitch as size_t,
            game.g_got_mismatch_count,
            (render_scale == 4 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8,
        );
    }
    g_renderer_funcs.EndDraw.expect("non-null function pointer")(game);
}
static mut g_audio_mutex: *mut SDL_mutex = ::core::ptr::null::<SDL_mutex>() as *mut SDL_mutex;
static mut g_audiobuffer_cur: *mut uint8 = ::core::ptr::null::<uint8>() as *mut uint8;
static mut g_audiobuffer_end: *mut uint8 = ::core::ptr::null::<uint8>() as *mut uint8;
static mut g_audiobuffer: *mut uint8 = ::core::ptr::null::<uint8>() as *mut uint8;
static mut g_frames_per_block: ::core::ffi::c_int = 0;
static mut g_audio_channels: uint8 = 0;
static mut g_audio_device: SDL_AudioDeviceID = 0;
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

unsafe fn finish_verify_audio() {
    SDL_DestroyMutex(g_audio_mutex);
}

unsafe fn host_apu_lock() {
    SDL_LockMutex(g_audio_mutex);
}
unsafe fn host_apu_unlock() {
    SDL_UnlockMutex(g_audio_mutex);
}
unsafe extern "C" fn AudioCallback(
    userdata: *mut ::core::ffi::c_void,
    mut stream: *mut Uint8,
    mut len: ::core::ffi::c_int,
) {
    let owner = &*(userdata as *const GameOwner);
    owner.with(|game| {
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
                    game,
                );
                g_audiobuffer_cur = g_audiobuffer;
                g_audiobuffer_end = g_audiobuffer.offset(
                    ((g_frames_per_block * g_audio_channels as ::core::ffi::c_int) as usize)
                        .wrapping_mul(::core::mem::size_of::<int16>() as usize)
                        as isize,
                );
            }
            let n: ::core::ffi::c_int = IntMin(
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
    });
}
static mut g_renderer: *mut SDL_Renderer = ::core::ptr::null::<SDL_Renderer>() as *mut SDL_Renderer;
static mut g_texture: *mut SDL_Texture = ::core::ptr::null::<SDL_Texture>() as *mut SDL_Texture;
static mut g_sdl_renderer_rect: SDL_Rect = SDL_Rect {
    x: 0,
    y: 0,
    w: 0,
    h: 0,
};
unsafe fn SdlRenderer_Init(window: *mut SDL_Window, game: &mut Game) -> bool {
    if !g_config.shader.is_null() {
        fprintf(
            stderr,
            b"Warning: Shaders are supported only with the OpenGL backend\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    }
    let renderer: *mut SDL_Renderer = SDL_CreateRenderer(
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
            sdl2_sys::SDL_HINT_RENDER_SCALE_QUALITY.as_ptr() as *const ::core::ffi::c_char,
            b"best\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    let tex_mult: ::core::ffi::c_int =
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
    true_0 != 0
}
unsafe fn SdlRenderer_Destroy(game: &mut Game) {
    SDL_DestroyTexture(g_texture);
    SDL_DestroyRenderer(g_renderer);
}
unsafe fn SdlRenderer_BeginDraw(
    width: ::core::ffi::c_int,
    height: ::core::ffi::c_int,
    pixels: *mut *mut uint8,
    pitch: *mut ::core::ffi::c_int,
    game: &mut Game,
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
    }
}
unsafe fn SdlRenderer_EndDraw(game: &mut Game) {
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
    Initialize: Some(SdlRenderer_Init as unsafe fn(*mut SDL_Window, &mut Game) -> bool),
    Destroy: Some(SdlRenderer_Destroy as unsafe fn(&mut Game) -> ()),
    BeginDraw: Some(
        SdlRenderer_BeginDraw
            as unsafe fn(
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                *mut *mut uint8,
                *mut ::core::ffi::c_int,
                &mut Game,
            ) -> (),
    ),
    EndDraw: Some(SdlRenderer_EndDraw as unsafe fn(&mut Game) -> ()),
};
unsafe fn main_0(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    owner: &GameOwner,
) -> ::core::ffi::c_int {
    argc -= 1;
    argv = argv.offset(1);
    let mut config_file: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if argc >= 2 as ::core::ffi::c_int
        && strcmp(
            *argv.offset(0),
            b"--config\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        config_file = *argv.offset(1);
        argc -= 2 as ::core::ffi::c_int;
        argv = argv.offset(2);
    } else {
        SwitchDirectory();
    }
    if argc >= 1 as ::core::ffi::c_int
        && strcmp(
            *argv.offset(0),
            b"--debug\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        owner.with(|game| game.g_debug_flag = true_0 != 0);
        argc -= 1 as ::core::ffi::c_int;
        argv = argv.offset(1);
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
    let custom_size: bool = g_config.window_width != 0 as ::core::ffi::c_int
        && g_config.window_height != 0 as ::core::ffi::c_int;
    let window_width: ::core::ffi::c_int = if custom_size as ::core::ffi::c_int != 0 {
        g_config.window_width
    } else {
        g_current_window_scale as ::core::ffi::c_int * g_snes_width
    };
    let window_height: ::core::ffi::c_int = if custom_size as ::core::ffi::c_int != 0 {
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
    let filename: *const ::core::ffi::c_char = if !(*argv.offset(0)).is_null() {
        *argv.offset(0) as *const ::core::ffi::c_char
    } else {
        b"sm.smc\0" as *const u8 as *const ::core::ffi::c_char
    };
    let snes: *mut Snes = owner.with(|game| SnesInit(filename, game));
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
    let window: *mut SDL_Window = SDL_CreateWindow(
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
    if !owner.with(|game| {
        g_renderer_funcs
            .Initialize
            .expect("non-null function pointer")(window, game)
    }) {
        return 1 as ::core::ffi::c_int;
    }
    g_audio_mutex = SDL_CreateMutex();
    if g_audio_mutex.is_null() {
        Die(b"No mutex\0" as *const u8 as *const ::core::ffi::c_char);
    }
    owner.with(|game| game.g_spc_player = SpcPlayer_Create() as *mut SpcPlayer);
    owner.with(|game| SpcPlayer_Initialize(game.g_spc_player as *mut SpcPlayer));
    let enable_audio: bool = true_0 != 0;
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
            userdata: owner as *const GameOwner as *mut ::core::ffi::c_void,
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
            userdata: owner as *const GameOwner as *mut ::core::ffi::c_void,
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
    owner.with(|game| RtlReadSram(game));
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < SDL_NumJoysticks() {
        OpenOneGamepad(i);
        i += 1;
    }
    if g_config.autosave {
        owner.with(|game| {
            HandleCommand(
                (kKeys_Load as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as uint32,
                true_0 != 0,
                game,
            )
        });
    }
    let mut running: bool = true_0 != 0;
    let mut lastTick: uint32 = SDL_GetTicks() as uint32;
    let mut curTick: uint32 = 0 as uint32;
    let mut frameCtr: uint32 = 0 as uint32;
    let mut audiopaused: uint8 = true_0 as uint8;
    let mut has_bug_in_title: bool = false_0 != 0;
    while running {
        let mut event: SDL_Event = SDL_Event { type_: 0 };
        while SDL_PollEvent(&raw mut event) != 0 {
            match event.type_ {
                1619 => {
                    OpenOneGamepad(event.cdevice.which as ::core::ffi::c_int);
                }
                1616 => {
                    owner.with(|game| {
                        HandleGamepadAxisInput(
                            event.caxis.which as ::core::ffi::c_int,
                            event.caxis.axis as ::core::ffi::c_int,
                            event.caxis.value as ::core::ffi::c_int,
                            game,
                        )
                    });
                }
                1617 | 1618 => {
                    let b: ::core::ffi::c_int =
                        RemapSdlButton(event.cbutton.button as ::core::ffi::c_int);
                    if b >= 0 as ::core::ffi::c_int {
                        owner.with(|game| {
                            HandleGamepadInput(
                                b,
                                event.type_
                                    == SDL_CONTROLLERBUTTONDOWN as ::core::ffi::c_int as Uint32,
                                game,
                            )
                        });
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
                        && g_win_flags
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
                            if g_win_flags & SDL_WINDOW_BORDERLESS as uint32 == 0 as uint32 {
                                SDL_TRUE
                            } else {
                                SDL_FALSE
                            },
                        );
                    }
                }
                768 => {
                    owner.with(|game| {
                        HandleInput(
                            event.key.keysym.sym as ::core::ffi::c_int,
                            event.key.keysym.mod_ as ::core::ffi::c_int,
                            true_0 != 0,
                            game,
                        )
                    });
                }
                769 => {
                    owner.with(|game| {
                        HandleInput(
                            event.key.keysym.sym as ::core::ffi::c_int,
                            event.key.keysym.mod_ as ::core::ffi::c_int,
                            false_0 != 0,
                            game,
                        )
                    });
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
            let is_replay: uint8 = owner.with(|game| RtlRunFrame(inputs, game)) as uint8;
            frameCtr = frameCtr.wrapping_add(1);
            let (want_bug_in_title, disable_render) = owner.with(|game| {
                (*game.g_snes).disableRender = g_turbo as ::core::ffi::c_int
                    ^ is_replay as ::core::ffi::c_int & g_replay_turbo as ::core::ffi::c_int
                    != 0
                    && frameCtr
                        & (if g_turbo as ::core::ffi::c_int != 0 {
                            0xf as ::core::ffi::c_int
                        } else {
                            0x7f as ::core::ffi::c_int
                        }) as uint32
                        != 0 as uint32;
                if !(*game.g_snes).disableRender {
                    DrawPpuFrameWithPerf(game);
                }
                (
                    game.g_got_mismatch_count != 0 as ::core::ffi::c_int,
                    (*game.g_snes).disableRender,
                )
            });
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
            if !disable_render && !g_config.disable_frame_delay {
                static mut delays: [uint8; 3] = [17, 17, 16];
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
        owner.with(|game| {
            HandleCommand(
                (kKeys_Save as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as uint32,
                true_0 != 0,
                game,
            )
        });
    }
    SDL_PauseAudioDevice(g_audio_device, 1 as ::core::ffi::c_int);
    SDL_CloseAudioDevice(g_audio_device);
    SDL_DestroyMutex(g_audio_mutex);
    free(g_audiobuffer as *mut ::core::ffi::c_void);
    owner.with(|game| g_renderer_funcs.Destroy.expect("non-null function pointer")(game));
    SDL_DestroyWindow(window as *mut SDL_Window);
    SDL_Quit();
    0 as ::core::ffi::c_int
}
unsafe fn RenderDigit(
    mut dst: *mut uint8,
    pitch: size_t,
    digit: ::core::ffi::c_int,
    color: uint32,
    big: bool,
) {
    static mut kFont: [uint8; 100] = [
        0x1c, 0x36, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x36, 0x1c, 0x18, 0x1c, 0x1e, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x7e, 0x3e, 0x63, 0x60, 0x30, 0x18, 0xc, 0x6, 0x3, 0x63, 0x7f,
        0x3e, 0x63, 0x60, 0x60, 0x3c, 0x60, 0x60, 0x60, 0x63, 0x3e, 0x30, 0x38, 0x3c, 0x36, 0x33,
        0x7f, 0x30, 0x30, 0x30, 0x78, 0x7f, 0x3, 0x3, 0x3, 0x3f, 0x60, 0x60, 0x60, 0x63, 0x3e,
        0x1c, 0x6, 0x3, 0x3, 0x3f, 0x63, 0x63, 0x63, 0x63, 0x3e, 0x7f, 0x63, 0x60, 0x60, 0x30,
        0x18, 0xc, 0xc, 0xc, 0xc, 0x3e, 0x63, 0x63, 0x63, 0x3e, 0x63, 0x63, 0x63, 0x63, 0x3e, 0x3e,
        0x63, 0x63, 0x63, 0x7e, 0x60, 0x60, 0x60, 0x30, 0x1e,
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
unsafe fn RenderNumber(dst: *mut uint8, pitch: size_t, n: ::core::ffi::c_int, big: uint8) {
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
unsafe fn HandleCommand(j: uint32, pressed: bool, game: &mut Game) {
    if j <= kKeys_Controls_Last as ::core::ffi::c_int as uint32 {
        static mut kKbdRemap: [uint8; 13] = [0, 4, 5, 6, 7, 2, 3, 8, 0, 9, 1, 10, 11];
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
            game,
        );
    } else if j <= kKeys_Save_Last as ::core::ffi::c_int as uint32 {
        RtlSaveLoad(
            kSaveLoad_Save as ::core::ffi::c_int,
            j.wrapping_sub(kKeys_Save as ::core::ffi::c_int as uint32) as ::core::ffi::c_int,
            game,
        );
    } else if j <= kKeys_Replay_Last as ::core::ffi::c_int as uint32 {
        RtlSaveLoad(
            kSaveLoad_Replay as ::core::ffi::c_int,
            j.wrapping_sub(kKeys_Replay as ::core::ffi::c_int as uint32) as ::core::ffi::c_int,
            game,
        );
    } else if j <= kKeys_LoadRef_Last as ::core::ffi::c_int as uint32 {
        RtlSaveLoad(
            kSaveLoad_Load as ::core::ffi::c_int,
            (256 as uint32)
                .wrapping_add(j)
                .wrapping_sub(kKeys_LoadRef as ::core::ffi::c_int as uint32)
                as ::core::ffi::c_int,
            game,
        );
    } else if j <= kKeys_ReplayRef_Last as ::core::ffi::c_int as uint32 {
        RtlSaveLoad(
            kSaveLoad_Replay as ::core::ffi::c_int,
            (256 as uint32)
                .wrapping_add(j)
                .wrapping_sub(kKeys_ReplayRef as ::core::ffi::c_int as uint32)
                as ::core::ffi::c_int,
            game,
        );
    } else {
        match j {
            113 => {
                RtlCheat('w' as i32 as ::core::ffi::c_char, game);
            }
            114 => {
                RtlCheat('q' as i32 as ::core::ffi::c_char, game);
            }
            115 => {
                game.g_other_image = !game.g_other_image;
            }
            116 => {
                RtlClearKeyLog(game);
            }
            117 => {
                RtlStopReplay(game);
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
                RtlReset(1 as ::core::ffi::c_int, game);
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
                game.g_new_ppu = g_ppu_render_flags
                    & kPpuRenderFlags_NewRenderer as ::core::ffi::c_int
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
unsafe fn HandleInput(
    keyCode: ::core::ffi::c_int,
    keyMod: ::core::ffi::c_int,
    pressed: bool,
    game: &mut Game,
) {
    let j: ::core::ffi::c_int = FindCmdForSdlKey(keyCode as SDL_Keycode, keyMod as SDL_Keymod);
    if j != 0 as ::core::ffi::c_int {
        HandleCommand(j as uint32, pressed, game);
    }
}
unsafe fn OpenOneGamepad(i: ::core::ffi::c_int) {
    if SDL_IsGameController(i) as u64 != 0 {
        let controller: *mut SDL_GameController = SDL_GameControllerOpen(i);
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
unsafe fn RemapSdlButton(button: ::core::ffi::c_int) -> ::core::ffi::c_int {
    match button {
        0 => kGamepadBtn_A as ::core::ffi::c_int,
        1 => kGamepadBtn_B as ::core::ffi::c_int,
        2 => kGamepadBtn_X as ::core::ffi::c_int,
        3 => kGamepadBtn_Y as ::core::ffi::c_int,
        4 => kGamepadBtn_Back as ::core::ffi::c_int,
        5 => kGamepadBtn_Guide as ::core::ffi::c_int,
        6 => kGamepadBtn_Start as ::core::ffi::c_int,
        7 => kGamepadBtn_L3 as ::core::ffi::c_int,
        8 => kGamepadBtn_R3 as ::core::ffi::c_int,
        9 => kGamepadBtn_L1 as ::core::ffi::c_int,
        10 => kGamepadBtn_R1 as ::core::ffi::c_int,
        11 => kGamepadBtn_DpadUp as ::core::ffi::c_int,
        12 => kGamepadBtn_DpadDown as ::core::ffi::c_int,
        13 => kGamepadBtn_DpadLeft as ::core::ffi::c_int,
        14 => kGamepadBtn_DpadRight as ::core::ffi::c_int,
        _ => -(1 as ::core::ffi::c_int),
    }
}
unsafe fn HandleGamepadInput(button: ::core::ffi::c_int, pressed: bool, game: &mut Game) {
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
        HandleCommand(g_gamepad_last_cmd[button as usize] as uint32, pressed, game);
    }
}
unsafe fn HandleVolumeAdjustment(volume_adjustment: ::core::ffi::c_int) {
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
unsafe fn ApproximateAtan2(
    mut y: ::core::ffi::c_float,
    mut x: ::core::ffi::c_float,
) -> ::core::ffi::c_float {
    let sign_mask: uint32 = 0x80000000 as uint32;
    let b: ::core::ffi::c_float = 0.596227f32;
    let ux_s: uint32 = sign_mask & *(&raw mut x as *mut uint32);
    let uy_s: uint32 = sign_mask & *(&raw mut y as *mut uint32);
    let q: ::core::ffi::c_float = ((!ux_s & uy_s) >> 29 as ::core::ffi::c_int
        | ux_s >> 30 as ::core::ffi::c_int)
        as ::core::ffi::c_float;
    let mut bxy_a: ::core::ffi::c_float = b * x * y;
    if bxy_a < 0.0f32 {
        bxy_a = -bxy_a;
    }
    let num: ::core::ffi::c_float = bxy_a + y * y;
    let mut atan_1q: ::core::ffi::c_float = num / (x * x + bxy_a + num + 0.000001f32);
    let mut uatan_2q: uint32_t =
        ux_s as uint32_t ^ uy_s as uint32_t | *(&raw mut atan_1q as *mut uint32);
    q + *(&raw mut uatan_2q as *mut ::core::ffi::c_float)
}
unsafe fn HandleGamepadAxisInput(
    gamepad_id: ::core::ffi::c_int,
    axis: ::core::ffi::c_int,
    value: ::core::ffi::c_int,
    game: &mut Game,
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
            let angle: uint8 = (ApproximateAtan2(
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
    } else if (axis == SDL_CONTROLLER_AXIS_TRIGGERLEFT as ::core::ffi::c_int
        || axis == SDL_CONTROLLER_AXIS_TRIGGERRIGHT as ::core::ffi::c_int)
        && (value < 12000 as ::core::ffi::c_int || value >= 16000 as ::core::ffi::c_int)
    {
        HandleGamepadInput(
            if axis == SDL_CONTROLLER_AXIS_TRIGGERLEFT as ::core::ffi::c_int {
                kGamepadBtn_L2 as ::core::ffi::c_int
            } else {
                kGamepadBtn_R2 as ::core::ffi::c_int
            },
            value >= 12000 as ::core::ffi::c_int,
            game,
        );
    }
}
unsafe fn SwitchDirectory() {
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
        let f: *mut FILE = fopen(
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
                let err: ::core::ffi::c_int = chdir(&raw mut buf as *mut ::core::ffi::c_char);
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
pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
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
    let mut args_ptrs: Vec<*mut ::core::ffi::c_char> = args_strings
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut ::core::ffi::c_char)
        .chain(::core::iter::once(::core::ptr::null_mut()))
        .collect();
    unsafe {
        ::std::process::exit(main_0(
            (args_ptrs.len() - 1) as ::core::ffi::c_int,
            args_ptrs.as_mut_ptr() as *mut *mut ::core::ffi::c_char,
            &owner,
        ) as i32)
    }
}
