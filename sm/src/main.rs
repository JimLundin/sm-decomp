#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
use ::core::ffi::{c_char, c_double, c_float, c_long, c_void};
use ::core::mem::{size_of, transmute};
use ::core::ptr::{null, null_mut};
#[allow(unused_imports)]
use ::sm as _;
use sm::game_state::{Game, GameOwner};
use sm::sdl_constants;
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
#[cfg(not(windows))]
use sm::c_library::{__assert_fail, stderr};
use sm::c_library::{
    chdir, exit, fclose, fopen, fprintf, free, getcwd, malloc, memcpy, mkdir, printf, snprintf,
    sprintf, strcmp, strlen,
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
pub use sm::types::{_IO_lock_t, __mode_t, __off64_t, __off_t, size_t, _IO_FILE, FILE};
sdl_constants! {
    SDL_bool: SDL_bool;
    SDL_TRUE, SDL_FALSE,
}
pub use sm::types::SDL_AudioDeviceID;
pub use sm::types::{
    SDL_AudioCallback, SDL_AudioFormat, Sint16, Sint32, Sint64, Uint16, Uint32, Uint64, Uint8,
};
pub type C2RustUnnamed = u32;
sdl_constants! {
    SDL_PixelFormatEnum as C2RustUnnamed;
    SDL_PIXELFORMAT_EXTERNAL_OES, SDL_PIXELFORMAT_NV21, SDL_PIXELFORMAT_NV12,
    SDL_PIXELFORMAT_YVYU, SDL_PIXELFORMAT_UYVY, SDL_PIXELFORMAT_YUY2, SDL_PIXELFORMAT_IYUV,
    SDL_PIXELFORMAT_YV12,
}
pub const SDL_PIXELFORMAT_XBGR32: C2RustUnnamed = 371595268;
pub const SDL_PIXELFORMAT_BGRX32: C2RustUnnamed = 370546692;
pub const SDL_PIXELFORMAT_XRGB32: C2RustUnnamed = 375789572;
pub const SDL_PIXELFORMAT_RGBX32: C2RustUnnamed = 374740996;
pub const SDL_PIXELFORMAT_ABGR32: C2RustUnnamed = 373694468;
pub const SDL_PIXELFORMAT_BGRA32: C2RustUnnamed = 372645892;
pub const SDL_PIXELFORMAT_ARGB32: C2RustUnnamed = 377888772;
pub const SDL_PIXELFORMAT_RGBA32: C2RustUnnamed = 376840196;
sdl_constants! {
    SDL_PixelFormatEnum as C2RustUnnamed;
    SDL_PIXELFORMAT_ARGB2101010, SDL_PIXELFORMAT_BGRA8888, SDL_PIXELFORMAT_ABGR8888,
    SDL_PIXELFORMAT_RGBA8888, SDL_PIXELFORMAT_ARGB8888, SDL_PIXELFORMAT_BGRX8888,
}
pub const SDL_PIXELFORMAT_BGR888: C2RustUnnamed = 374740996;
sdl_constants! {
    SDL_PixelFormatEnum as C2RustUnnamed;
    SDL_PIXELFORMAT_XBGR8888, SDL_PIXELFORMAT_RGBX8888,
}
pub const SDL_PIXELFORMAT_RGB888: C2RustUnnamed = 370546692;
sdl_constants! {
    SDL_PixelFormatEnum as C2RustUnnamed;
    SDL_PIXELFORMAT_XRGB8888, SDL_PIXELFORMAT_BGR24, SDL_PIXELFORMAT_RGB24,
    SDL_PIXELFORMAT_BGR565, SDL_PIXELFORMAT_RGB565, SDL_PIXELFORMAT_BGRA5551,
    SDL_PIXELFORMAT_ABGR1555, SDL_PIXELFORMAT_RGBA5551, SDL_PIXELFORMAT_ARGB1555,
    SDL_PIXELFORMAT_BGRA4444, SDL_PIXELFORMAT_ABGR4444, SDL_PIXELFORMAT_RGBA4444,
    SDL_PIXELFORMAT_ARGB4444,
}
pub const SDL_PIXELFORMAT_BGR555: C2RustUnnamed = 357764866;
sdl_constants! {
    SDL_PixelFormatEnum as C2RustUnnamed;
    SDL_PIXELFORMAT_XBGR1555,
}
pub const SDL_PIXELFORMAT_RGB555: C2RustUnnamed = 353570562;
sdl_constants! {
    SDL_PixelFormatEnum as C2RustUnnamed;
    SDL_PIXELFORMAT_XRGB1555,
}
pub const SDL_PIXELFORMAT_BGR444: C2RustUnnamed = 357698562;
sdl_constants! {
    SDL_PixelFormatEnum as C2RustUnnamed;
    SDL_PIXELFORMAT_XBGR4444,
}
pub const SDL_PIXELFORMAT_RGB444: C2RustUnnamed = 353504258;
sdl_constants! {
    SDL_PixelFormatEnum as C2RustUnnamed;
    SDL_PIXELFORMAT_XRGB4444, SDL_PIXELFORMAT_RGB332, SDL_PIXELFORMAT_INDEX8,
    SDL_PIXELFORMAT_INDEX4MSB, SDL_PIXELFORMAT_INDEX4LSB,
}
pub const SDL_PIXELFORMAT_INDEX2MSB: C2RustUnnamed = 471859712;
pub const SDL_PIXELFORMAT_INDEX2LSB: C2RustUnnamed = 470811136;
sdl_constants! {
    SDL_PixelFormatEnum as C2RustUnnamed;
    SDL_PIXELFORMAT_INDEX1MSB, SDL_PIXELFORMAT_INDEX1LSB, SDL_PIXELFORMAT_UNKNOWN,
}
pub type C2RustUnnamed_0 = u32;
pub const SDL_WINDOW_INPUT_GRABBED: C2RustUnnamed_0 = 256;
sdl_constants! {
    SDL_WindowFlags as C2RustUnnamed_0;
    SDL_WINDOW_METAL, SDL_WINDOW_VULKAN, SDL_WINDOW_KEYBOARD_GRABBED, SDL_WINDOW_POPUP_MENU,
    SDL_WINDOW_TOOLTIP, SDL_WINDOW_UTILITY, SDL_WINDOW_SKIP_TASKBAR, SDL_WINDOW_ALWAYS_ON_TOP,
    SDL_WINDOW_MOUSE_CAPTURE, SDL_WINDOW_ALLOW_HIGHDPI, SDL_WINDOW_FOREIGN,
    SDL_WINDOW_FULLSCREEN_DESKTOP, SDL_WINDOW_MOUSE_FOCUS, SDL_WINDOW_INPUT_FOCUS,
    SDL_WINDOW_MOUSE_GRABBED, SDL_WINDOW_MAXIMIZED, SDL_WINDOW_MINIMIZED, SDL_WINDOW_RESIZABLE,
    SDL_WINDOW_BORDERLESS, SDL_WINDOW_HIDDEN, SDL_WINDOW_SHOWN, SDL_WINDOW_OPENGL,
    SDL_WINDOW_FULLSCREEN,
}
sdl_constants! {
    SDL_HitTestResult: SDL_HitTestResult;
    SDL_HITTEST_RESIZE_LEFT, SDL_HITTEST_RESIZE_BOTTOMLEFT, SDL_HITTEST_RESIZE_BOTTOM,
    SDL_HITTEST_RESIZE_BOTTOMRIGHT, SDL_HITTEST_RESIZE_RIGHT, SDL_HITTEST_RESIZE_TOPRIGHT,
    SDL_HITTEST_RESIZE_TOP, SDL_HITTEST_RESIZE_TOPLEFT, SDL_HITTEST_DRAGGABLE,
    SDL_HITTEST_NORMAL,
}
pub use sm::types::SDL_Scancode;
sdl_constants! {
    SDL_Scancode as SDL_Scancode;
    SDL_NUM_SCANCODES, SDL_SCANCODE_ENDCALL, SDL_SCANCODE_CALL, SDL_SCANCODE_SOFTRIGHT,
    SDL_SCANCODE_SOFTLEFT, SDL_SCANCODE_AUDIOFASTFORWARD, SDL_SCANCODE_AUDIOREWIND,
    SDL_SCANCODE_APP2, SDL_SCANCODE_APP1, SDL_SCANCODE_SLEEP, SDL_SCANCODE_EJECT,
    SDL_SCANCODE_KBDILLUMUP, SDL_SCANCODE_KBDILLUMDOWN, SDL_SCANCODE_KBDILLUMTOGGLE,
    SDL_SCANCODE_DISPLAYSWITCH, SDL_SCANCODE_BRIGHTNESSUP, SDL_SCANCODE_BRIGHTNESSDOWN,
    SDL_SCANCODE_AC_BOOKMARKS, SDL_SCANCODE_AC_REFRESH, SDL_SCANCODE_AC_STOP,
    SDL_SCANCODE_AC_FORWARD, SDL_SCANCODE_AC_BACK, SDL_SCANCODE_AC_HOME, SDL_SCANCODE_AC_SEARCH,
    SDL_SCANCODE_COMPUTER, SDL_SCANCODE_CALCULATOR, SDL_SCANCODE_MAIL, SDL_SCANCODE_WWW,
    SDL_SCANCODE_MEDIASELECT, SDL_SCANCODE_AUDIOMUTE, SDL_SCANCODE_AUDIOPLAY,
    SDL_SCANCODE_AUDIOSTOP, SDL_SCANCODE_AUDIOPREV, SDL_SCANCODE_AUDIONEXT, SDL_SCANCODE_MODE,
    SDL_SCANCODE_RGUI, SDL_SCANCODE_RALT, SDL_SCANCODE_RSHIFT, SDL_SCANCODE_RCTRL,
    SDL_SCANCODE_LGUI, SDL_SCANCODE_LALT, SDL_SCANCODE_LSHIFT, SDL_SCANCODE_LCTRL,
    SDL_SCANCODE_KP_HEXADECIMAL, SDL_SCANCODE_KP_DECIMAL, SDL_SCANCODE_KP_OCTAL,
    SDL_SCANCODE_KP_BINARY, SDL_SCANCODE_KP_CLEARENTRY, SDL_SCANCODE_KP_CLEAR,
    SDL_SCANCODE_KP_PLUSMINUS, SDL_SCANCODE_KP_MEMDIVIDE, SDL_SCANCODE_KP_MEMMULTIPLY,
    SDL_SCANCODE_KP_MEMSUBTRACT, SDL_SCANCODE_KP_MEMADD, SDL_SCANCODE_KP_MEMCLEAR,
    SDL_SCANCODE_KP_MEMRECALL, SDL_SCANCODE_KP_MEMSTORE, SDL_SCANCODE_KP_EXCLAM,
    SDL_SCANCODE_KP_AT, SDL_SCANCODE_KP_SPACE, SDL_SCANCODE_KP_HASH, SDL_SCANCODE_KP_COLON,
    SDL_SCANCODE_KP_DBLVERTICALBAR, SDL_SCANCODE_KP_VERTICALBAR, SDL_SCANCODE_KP_DBLAMPERSAND,
    SDL_SCANCODE_KP_AMPERSAND, SDL_SCANCODE_KP_GREATER, SDL_SCANCODE_KP_LESS,
    SDL_SCANCODE_KP_PERCENT, SDL_SCANCODE_KP_POWER, SDL_SCANCODE_KP_XOR, SDL_SCANCODE_KP_F,
    SDL_SCANCODE_KP_E, SDL_SCANCODE_KP_D, SDL_SCANCODE_KP_C, SDL_SCANCODE_KP_B,
    SDL_SCANCODE_KP_A, SDL_SCANCODE_KP_BACKSPACE, SDL_SCANCODE_KP_TAB,
    SDL_SCANCODE_KP_RIGHTBRACE, SDL_SCANCODE_KP_LEFTBRACE, SDL_SCANCODE_KP_RIGHTPAREN,
    SDL_SCANCODE_KP_LEFTPAREN, SDL_SCANCODE_CURRENCYSUBUNIT, SDL_SCANCODE_CURRENCYUNIT,
    SDL_SCANCODE_DECIMALSEPARATOR, SDL_SCANCODE_THOUSANDSSEPARATOR, SDL_SCANCODE_KP_000,
    SDL_SCANCODE_KP_00, SDL_SCANCODE_EXSEL, SDL_SCANCODE_CRSEL, SDL_SCANCODE_CLEARAGAIN,
    SDL_SCANCODE_OPER, SDL_SCANCODE_OUT, SDL_SCANCODE_SEPARATOR, SDL_SCANCODE_RETURN2,
    SDL_SCANCODE_PRIOR, SDL_SCANCODE_CLEAR, SDL_SCANCODE_CANCEL, SDL_SCANCODE_SYSREQ,
    SDL_SCANCODE_ALTERASE, SDL_SCANCODE_LANG9, SDL_SCANCODE_LANG8, SDL_SCANCODE_LANG7,
    SDL_SCANCODE_LANG6, SDL_SCANCODE_LANG5, SDL_SCANCODE_LANG4, SDL_SCANCODE_LANG3,
    SDL_SCANCODE_LANG2, SDL_SCANCODE_LANG1, SDL_SCANCODE_INTERNATIONAL9,
    SDL_SCANCODE_INTERNATIONAL8, SDL_SCANCODE_INTERNATIONAL7, SDL_SCANCODE_INTERNATIONAL6,
    SDL_SCANCODE_INTERNATIONAL5, SDL_SCANCODE_INTERNATIONAL4, SDL_SCANCODE_INTERNATIONAL3,
    SDL_SCANCODE_INTERNATIONAL2, SDL_SCANCODE_INTERNATIONAL1, SDL_SCANCODE_KP_EQUALSAS400,
    SDL_SCANCODE_KP_COMMA, SDL_SCANCODE_VOLUMEDOWN, SDL_SCANCODE_VOLUMEUP, SDL_SCANCODE_MUTE,
    SDL_SCANCODE_FIND, SDL_SCANCODE_PASTE, SDL_SCANCODE_COPY, SDL_SCANCODE_CUT,
    SDL_SCANCODE_UNDO, SDL_SCANCODE_AGAIN, SDL_SCANCODE_STOP, SDL_SCANCODE_SELECT,
    SDL_SCANCODE_MENU, SDL_SCANCODE_HELP, SDL_SCANCODE_EXECUTE, SDL_SCANCODE_F24,
    SDL_SCANCODE_F23, SDL_SCANCODE_F22, SDL_SCANCODE_F21, SDL_SCANCODE_F20, SDL_SCANCODE_F19,
    SDL_SCANCODE_F18, SDL_SCANCODE_F17, SDL_SCANCODE_F16, SDL_SCANCODE_F15, SDL_SCANCODE_F14,
    SDL_SCANCODE_F13, SDL_SCANCODE_KP_EQUALS, SDL_SCANCODE_POWER, SDL_SCANCODE_APPLICATION,
    SDL_SCANCODE_NONUSBACKSLASH, SDL_SCANCODE_KP_PERIOD, SDL_SCANCODE_KP_0, SDL_SCANCODE_KP_9,
    SDL_SCANCODE_KP_8, SDL_SCANCODE_KP_7, SDL_SCANCODE_KP_6, SDL_SCANCODE_KP_5,
    SDL_SCANCODE_KP_4, SDL_SCANCODE_KP_3, SDL_SCANCODE_KP_2, SDL_SCANCODE_KP_1,
    SDL_SCANCODE_KP_ENTER, SDL_SCANCODE_KP_PLUS, SDL_SCANCODE_KP_MINUS,
    SDL_SCANCODE_KP_MULTIPLY, SDL_SCANCODE_KP_DIVIDE, SDL_SCANCODE_NUMLOCKCLEAR,
    SDL_SCANCODE_UP, SDL_SCANCODE_DOWN, SDL_SCANCODE_LEFT, SDL_SCANCODE_RIGHT,
    SDL_SCANCODE_PAGEDOWN, SDL_SCANCODE_END, SDL_SCANCODE_DELETE, SDL_SCANCODE_PAGEUP,
    SDL_SCANCODE_HOME, SDL_SCANCODE_INSERT, SDL_SCANCODE_PAUSE, SDL_SCANCODE_SCROLLLOCK,
    SDL_SCANCODE_PRINTSCREEN, SDL_SCANCODE_F12, SDL_SCANCODE_F11, SDL_SCANCODE_F10,
    SDL_SCANCODE_F9, SDL_SCANCODE_F8, SDL_SCANCODE_F7, SDL_SCANCODE_F6, SDL_SCANCODE_F5,
    SDL_SCANCODE_F4, SDL_SCANCODE_F3, SDL_SCANCODE_F2, SDL_SCANCODE_F1, SDL_SCANCODE_CAPSLOCK,
    SDL_SCANCODE_SLASH, SDL_SCANCODE_PERIOD, SDL_SCANCODE_COMMA, SDL_SCANCODE_GRAVE,
    SDL_SCANCODE_APOSTROPHE, SDL_SCANCODE_SEMICOLON, SDL_SCANCODE_NONUSHASH,
    SDL_SCANCODE_BACKSLASH, SDL_SCANCODE_RIGHTBRACKET, SDL_SCANCODE_LEFTBRACKET,
    SDL_SCANCODE_EQUALS, SDL_SCANCODE_MINUS, SDL_SCANCODE_SPACE, SDL_SCANCODE_TAB,
    SDL_SCANCODE_BACKSPACE, SDL_SCANCODE_ESCAPE, SDL_SCANCODE_RETURN, SDL_SCANCODE_0,
    SDL_SCANCODE_9, SDL_SCANCODE_8, SDL_SCANCODE_7, SDL_SCANCODE_6, SDL_SCANCODE_5,
    SDL_SCANCODE_4, SDL_SCANCODE_3, SDL_SCANCODE_2, SDL_SCANCODE_1, SDL_SCANCODE_Z,
    SDL_SCANCODE_Y, SDL_SCANCODE_X, SDL_SCANCODE_W, SDL_SCANCODE_V, SDL_SCANCODE_U,
    SDL_SCANCODE_T, SDL_SCANCODE_S, SDL_SCANCODE_R, SDL_SCANCODE_Q, SDL_SCANCODE_P,
    SDL_SCANCODE_O, SDL_SCANCODE_N, SDL_SCANCODE_M, SDL_SCANCODE_L, SDL_SCANCODE_K,
    SDL_SCANCODE_J, SDL_SCANCODE_I, SDL_SCANCODE_H, SDL_SCANCODE_G, SDL_SCANCODE_F,
    SDL_SCANCODE_E, SDL_SCANCODE_D, SDL_SCANCODE_C, SDL_SCANCODE_B, SDL_SCANCODE_A,
    SDL_SCANCODE_UNKNOWN,
}
pub use sm::types::{SDL_Keycode, SDL_Keymod};
pub const KMOD_RESERVED: SDL_Keymod = 32768;
sdl_constants! {
    SDL_Keymod as SDL_Keymod;
    KMOD_GUI, KMOD_ALT, KMOD_SHIFT, KMOD_CTRL, KMOD_SCROLL, KMOD_MODE, KMOD_CAPS, KMOD_NUM,
    KMOD_RGUI, KMOD_LGUI, KMOD_RALT, KMOD_LALT, KMOD_RCTRL, KMOD_LCTRL, KMOD_RSHIFT,
    KMOD_LSHIFT, KMOD_NONE,
}
pub use sm::types::{SDL_JoystickID, SDL_JoystickPowerLevel};
sdl_constants! {
    SDL_JoystickPowerLevel as SDL_JoystickPowerLevel;
    SDL_JOYSTICK_POWER_MAX, SDL_JOYSTICK_POWER_WIRED, SDL_JOYSTICK_POWER_FULL,
    SDL_JOYSTICK_POWER_MEDIUM, SDL_JOYSTICK_POWER_LOW, SDL_JOYSTICK_POWER_EMPTY,
    SDL_JOYSTICK_POWER_UNKNOWN,
}
pub type C2RustUnnamed_1 = i32;
sdl_constants! {
    SDL_GameControllerAxis as C2RustUnnamed_1;
    SDL_CONTROLLER_AXIS_MAX, SDL_CONTROLLER_AXIS_TRIGGERRIGHT, SDL_CONTROLLER_AXIS_TRIGGERLEFT,
    SDL_CONTROLLER_AXIS_RIGHTY, SDL_CONTROLLER_AXIS_RIGHTX, SDL_CONTROLLER_AXIS_LEFTY,
    SDL_CONTROLLER_AXIS_LEFTX, SDL_CONTROLLER_AXIS_INVALID,
}
pub type C2RustUnnamed_2 = i32;
sdl_constants! {
    SDL_GameControllerButton as C2RustUnnamed_2;
    SDL_CONTROLLER_BUTTON_MAX, SDL_CONTROLLER_BUTTON_TOUCHPAD, SDL_CONTROLLER_BUTTON_PADDLE4,
    SDL_CONTROLLER_BUTTON_PADDLE3, SDL_CONTROLLER_BUTTON_PADDLE2, SDL_CONTROLLER_BUTTON_PADDLE1,
    SDL_CONTROLLER_BUTTON_MISC1, SDL_CONTROLLER_BUTTON_DPAD_RIGHT,
    SDL_CONTROLLER_BUTTON_DPAD_LEFT, SDL_CONTROLLER_BUTTON_DPAD_DOWN,
    SDL_CONTROLLER_BUTTON_DPAD_UP, SDL_CONTROLLER_BUTTON_RIGHTSHOULDER,
    SDL_CONTROLLER_BUTTON_LEFTSHOULDER, SDL_CONTROLLER_BUTTON_RIGHTSTICK,
    SDL_CONTROLLER_BUTTON_LEFTSTICK, SDL_CONTROLLER_BUTTON_START, SDL_CONTROLLER_BUTTON_GUIDE,
    SDL_CONTROLLER_BUTTON_BACK, SDL_CONTROLLER_BUTTON_Y, SDL_CONTROLLER_BUTTON_X,
    SDL_CONTROLLER_BUTTON_B, SDL_CONTROLLER_BUTTON_A, SDL_CONTROLLER_BUTTON_INVALID,
}
pub use sm::types::{SDL_FingerID, SDL_GestureID, SDL_TouchID};
pub type C2RustUnnamed_3 = u32;
sdl_constants! {
    SDL_EventType as C2RustUnnamed_3;
    SDL_LASTEVENT, SDL_USEREVENT, SDL_POLLSENTINEL, SDL_RENDER_DEVICE_RESET,
    SDL_RENDER_TARGETS_RESET, SDL_SENSORUPDATE, SDL_AUDIODEVICEREMOVED, SDL_AUDIODEVICEADDED,
    SDL_DROPCOMPLETE, SDL_DROPBEGIN, SDL_DROPTEXT, SDL_DROPFILE, SDL_CLIPBOARDUPDATE,
    SDL_MULTIGESTURE, SDL_DOLLARRECORD, SDL_DOLLARGESTURE, SDL_FINGERMOTION, SDL_FINGERUP,
    SDL_FINGERDOWN,
}
pub const SDL_CONTROLLERSTEAMHANDLEUPDATED: C2RustUnnamed_3 = 1627;
pub const SDL_CONTROLLERUPDATECOMPLETE_RESERVED_FOR_SDL3: C2RustUnnamed_3 = 1626;
sdl_constants! {
    SDL_EventType as C2RustUnnamed_3;
    SDL_CONTROLLERSENSORUPDATE, SDL_CONTROLLERTOUCHPADUP, SDL_CONTROLLERTOUCHPADMOTION,
    SDL_CONTROLLERTOUCHPADDOWN, SDL_CONTROLLERDEVICEREMAPPED, SDL_CONTROLLERDEVICEREMOVED,
    SDL_CONTROLLERDEVICEADDED, SDL_CONTROLLERBUTTONUP, SDL_CONTROLLERBUTTONDOWN,
    SDL_CONTROLLERAXISMOTION, SDL_JOYBATTERYUPDATED, SDL_JOYDEVICEREMOVED, SDL_JOYDEVICEADDED,
    SDL_JOYBUTTONUP, SDL_JOYBUTTONDOWN, SDL_JOYHATMOTION, SDL_JOYBALLMOTION, SDL_JOYAXISMOTION,
    SDL_MOUSEWHEEL, SDL_MOUSEBUTTONUP, SDL_MOUSEBUTTONDOWN, SDL_MOUSEMOTION,
    SDL_TEXTEDITING_EXT, SDL_KEYMAPCHANGED, SDL_TEXTINPUT, SDL_TEXTEDITING, SDL_KEYUP,
    SDL_KEYDOWN, SDL_SYSWMEVENT, SDL_WINDOWEVENT, SDL_DISPLAYEVENT, SDL_LOCALECHANGED,
    SDL_APP_DIDENTERFOREGROUND, SDL_APP_WILLENTERFOREGROUND, SDL_APP_DIDENTERBACKGROUND,
    SDL_APP_WILLENTERBACKGROUND, SDL_APP_LOWMEMORY, SDL_APP_TERMINATING, SDL_QUIT,
    SDL_FIRSTEVENT,
}
pub type C2RustUnnamed_4 = u32;
sdl_constants! {
    SDL_MessageBoxFlags as C2RustUnnamed_4;
    SDL_MESSAGEBOX_BUTTONS_RIGHT_TO_LEFT, SDL_MESSAGEBOX_BUTTONS_LEFT_TO_RIGHT,
    SDL_MESSAGEBOX_INFORMATION, SDL_MESSAGEBOX_WARNING, SDL_MESSAGEBOX_ERROR,
}
pub type C2RustUnnamed_5 = u32;
sdl_constants! {
    SDL_RendererFlags as C2RustUnnamed_5;
    SDL_RENDERER_TARGETTEXTURE, SDL_RENDERER_PRESENTVSYNC, SDL_RENDERER_ACCELERATED,
    SDL_RENDERER_SOFTWARE,
}
pub type C2RustUnnamed_6 = u32;
sdl_constants! {
    SDL_TextureAccess as C2RustUnnamed_6;
    SDL_TEXTUREACCESS_TARGET, SDL_TEXTUREACCESS_STREAMING, SDL_TEXTUREACCESS_STATIC,
}
pub use sm::types::{
    Apu, ApuHistory as C2RustUnnamed_7, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer,
    WindowLayer,
};
pub type C2RustUnnamed_8 = u32;
pub const kPpuRenderFlags_NoSpriteLimits: C2RustUnnamed_8 = 8;
pub const kPpuRenderFlags_Height240: C2RustUnnamed_8 = 4;
pub const kPpuRenderFlags_4x4Mode7: C2RustUnnamed_8 = 2;
pub const kPpuRenderFlags_NewRenderer: C2RustUnnamed_8 = 1;
pub use sm::types::SaveLoadOperation as C2RustUnnamed_9;
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
pub type C2RustUnnamed_13 = u32;
pub const kDefaultSamples: C2RustUnnamed_13 = 2048;
pub const kDefaultChannels: C2RustUnnamed_13 = 2;
pub const kDefaultFreq: C2RustUnnamed_13 = 44100;
pub const kDefaultFullscreen: C2RustUnnamed_13 = 0;
pub const AUDIO_S16LSB: i32 = sdl2_sys::AUDIO_S16LSB as i32;
pub const AUDIO_S16: i32 = sdl2_sys::AUDIO_S16 as i32;
pub const SDL_MIX_MAXVOLUME: i32 = sdl2_sys::SDL_MIX_MAXVOLUME as i32;
pub const SDL_WINDOWPOS_UNDEFINED_MASK: u32 = sdl2_sys::SDL_WINDOWPOS_UNDEFINED_MASK;
pub const SDL_WINDOWPOS_UNDEFINED: u32 = SDL_WINDOWPOS_UNDEFINED_MASK | 0u32;
pub const SDL_WINDOWPOS_CENTERED_MASK: u32 = sdl2_sys::SDL_WINDOWPOS_CENTERED_MASK;
pub const SDL_WINDOWPOS_CENTERED: u32 = SDL_WINDOWPOS_CENTERED_MASK | 0u32;
pub const SDL_BUTTON_LEFT: i32 = sdl2_sys::SDL_BUTTON_LEFT as i32;
pub const SDL_PRESSED: i32 = sdl2_sys::SDL_PRESSED as i32;
pub const SDL_INIT_AUDIO: u32 = sdl2_sys::SDL_INIT_AUDIO;
pub const SDL_INIT_VIDEO: u32 = sdl2_sys::SDL_INIT_VIDEO;
pub const SDL_INIT_GAMECONTROLLER: u32 = sdl2_sys::SDL_INIT_GAMECONTROLLER;
pub const kDebugFlag: i32 = 0;
#[inline]
unsafe fn IntMin(a: i32, b: i32) -> i32 {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
unsafe fn IntMax(a: i32, b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}
pub static mut g_is_turbo: bool = false;
pub static mut g_want_dump_memmap_flags: bool = false;
static mut g_pixels: [u8; 245760] = [0; 245760];
static mut g_my_pixels: [u8; 245760] = [0; 245760];
static mut kWindowTitle: [c_char; 9] = unsafe { transmute::<[u8; 9], [c_char; 9]>(*b"SuperMet\0") };
static mut g_win_flags: u32 = SDL_WINDOW_RESIZABLE as i32 as u32;
static mut g_window: *mut SDL_Window = null::<SDL_Window>() as *mut SDL_Window;
static mut g_cursor: u8 = true_0 as u8;
static mut g_paused: u8 = 0;
static mut g_turbo: u8 = 0;
static mut g_replay_turbo: u8 = true_0 as u8;
static mut g_current_window_scale: u8 = 0;
static mut g_gamepad_buttons: u8 = 0;
static mut g_input1_state: i32 = 0;
static mut g_display_perf: bool = false;
static mut g_curr_fps: i32 = 0;
static mut g_ppu_render_flags: i32 = 0i32;
static mut g_snes_height: i32 = 0;
static mut g_snes_width: i32 = 0;
static mut g_sdl_audio_mixer_volume: i32 = SDL_MIX_MAXVOLUME;
static mut g_renderer_funcs: RendererFuncs = RendererFuncs {
    Initialize: None,
    Destroy: None,
    BeginDraw: None,
    EndDraw: None,
};
static mut g_gamepad_modifiers: u32 = 0;
static mut g_gamepad_last_cmd: [u16; 17] = [0; 17];
unsafe fn host_die(error: *const c_char) {
    SDL_ShowSimpleMessageBox(
        SDL_MESSAGEBOX_ERROR as i32 as Uint32,
        &raw const kWindowTitle as *const c_char,
        error,
        null_mut::<SDL_Window>(),
    );
    fprintf(
        stderr,
        b"Error: %s\n\0" as *const u8 as *const c_char,
        error,
    );
    exit(1i32);
}
unsafe fn host_warning(error: *const c_char) {
    SDL_ShowSimpleMessageBox(
        SDL_MESSAGEBOX_WARNING as i32 as Uint32,
        &raw const kWindowTitle as *const c_char,
        error,
        null_mut::<SDL_Window>(),
    );
    fprintf(
        stderr,
        b"Warning: %s\n\0" as *const u8 as *const c_char,
        error,
    );
}
pub unsafe fn ChangeWindowScale(scale_step: i32) {
    if SDL_GetWindowFlags(g_window)
        & (SDL_WINDOW_FULLSCREEN_DESKTOP as i32
            | SDL_WINDOW_FULLSCREEN as i32
            | SDL_WINDOW_MINIMIZED as i32
            | SDL_WINDOW_MAXIMIZED as i32) as Uint32
        != 0 as Uint32
    {
        return;
    }
    let mut screen: i32 = SDL_GetWindowDisplayIndex(g_window);
    if screen < 0i32 {
        screen = 0i32;
    }
    let mut max_scale: i32 = kMaxWindowScale as i32;
    let mut bounds: SDL_Rect = SDL_Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };
    let mut bt: i32 = -1i32;
    let mut bl: i32 = 0;
    let mut bb: i32 = 0;
    let mut br: i32 = 0;
    if SDL_GetDisplayUsableBounds(screen, &raw mut bounds) == 0i32 {
        if SDL_GetWindowBordersSize(g_window, &raw mut bt, &raw mut bl, &raw mut bb, &raw mut br)
            != 0i32
        {
            bb = 1i32;
            br = bb;
            bl = br;
            bt = 31i32;
        }
        let mw: i32 = (bounds.w - bl - br + g_snes_width / 4i32) / g_snes_width;
        let mh: i32 = (bounds.h - bt - bb + g_snes_height / 4i32) / g_snes_height;
        max_scale = IntMin(mw, mh);
    }
    let new_scale: i32 = IntMax(
        IntMin(g_current_window_scale as i32 + scale_step, max_scale),
        1i32,
    );
    g_current_window_scale = new_scale as u8;
    let w: i32 = new_scale * g_snes_width;
    let h: i32 = new_scale * g_snes_height;
    SDL_SetWindowSize(g_window, w, h);
    if bt >= 0i32 {
        let mut mx: i32 = 0;
        let mut my: i32 = 0;
        SDL_GetGlobalMouseState(&raw mut mx, &raw mut my);
        let wx: i32 = IntMax(
            IntMin(mx - w / 2i32, bounds.x + bounds.w - bl - br - w),
            bounds.x + bl,
        );
        let wy: i32 = IntMax(
            IntMin(my - h / 2i32, bounds.y + bounds.h - bt - bb - h),
            bounds.y + bt,
        );
        SDL_SetWindowPosition(g_window, wx, wy);
    } else {
        SDL_SetWindowPosition(
            g_window,
            SDL_WINDOWPOS_CENTERED as i32,
            SDL_WINDOWPOS_CENTERED as i32,
        );
    };
}
pub const RESIZE_BORDER: i32 = 20;
unsafe extern "C" fn HitTestCallback(
    win: *mut SDL_Window,
    pt: *const SDL_Point,
    data: *mut c_void,
) -> SDL_HitTestResult {
    let flags: u32 = SDL_GetWindowFlags(win) as u32;
    if flags & SDL_WINDOW_FULLSCREEN_DESKTOP as i32 as u32 != 0u32
        || flags & SDL_WINDOW_FULLSCREEN as i32 as u32 != 0u32
    {
        return SDL_HITTEST_NORMAL;
    }
    if SDL_GetModState() as u32 & KMOD_CTRL as i32 as u32 != 0u32 {
        return SDL_HITTEST_DRAGGABLE;
    }
    let mut w: i32 = 0;
    let mut h: i32 = 0;
    SDL_GetWindowSize(win, &raw mut w, &raw mut h);
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
    pixel_buffer: *mut u8,
    pitch: size_t,
    render_flags: u32,
    game: &mut Game,
) {
    let ppu_pixels: *mut u8 = if game.g_other_image as i32 != 0 {
        &raw mut g_my_pixels as *mut u8
    } else {
        &raw mut g_pixels as *mut u8
    };
    let mut y: size_t = 0 as size_t;
    while y < 240 as size_t {
        memcpy(
            (pixel_buffer).offset(y.wrapping_mul(pitch) as isize) as *mut c_void,
            ppu_pixels.offset(y.wrapping_mul(256 as size_t).wrapping_mul(4 as size_t) as isize)
                as *const c_void,
            (256i32 * 4i32) as size_t,
        );
        y = y.wrapping_add(1);
    }
}
unsafe fn DrawPpuFrameWithPerf(game: &mut Game) {
    let render_scale: i32 = PpuGetCurrentRenderScale((*game.g_snes).ppu, g_ppu_render_flags as u32);
    let mut pixel_buffer: *mut u8 = null_mut::<u8>();
    let mut pitch: i32 = 0i32;
    g_renderer_funcs
        .BeginDraw
        .expect("non-null function pointer")(
        g_snes_width * render_scale,
        g_snes_height * render_scale,
        &raw mut pixel_buffer,
        &raw mut pitch,
        game,
    );
    if g_display_perf as i32 != 0 || g_config.display_perf_title as i32 != 0 {
        static mut history: [c_float; 64] = [0.; 64];
        static mut average: c_float = 0.;
        static mut history_pos: i32 = 0;
        let before: u64 = SDL_GetPerformanceCounter() as u64;
        RtlDrawPpuFrame(
            pixel_buffer,
            pitch as size_t,
            g_ppu_render_flags as u32,
            game,
        );
        let after: u64 = SDL_GetPerformanceCounter() as u64;
        let v: c_float = (SDL_GetPerformanceFrequency() as c_double
            / after.wrapping_sub(before) as c_double) as c_float;
        average += v - history[history_pos as usize];
        history[history_pos as usize] = v;
        history_pos = history_pos + 1i32 & 63i32;
        g_curr_fps = (average * (1.0f32 / 64i32 as c_float)) as i32;
    } else {
        RtlDrawPpuFrame(
            pixel_buffer,
            pitch as size_t,
            g_ppu_render_flags as u32,
            game,
        );
    }
    if g_display_perf {
        RenderNumber(
            pixel_buffer.offset((pitch * render_scale) as isize),
            pitch as size_t,
            g_curr_fps,
            (render_scale == 4i32) as i32 as u8,
        );
    }
    if game.g_got_mismatch_count != 0 {
        RenderNumber(
            pixel_buffer.offset((pitch * render_scale) as isize),
            pitch as size_t,
            game.g_got_mismatch_count,
            (render_scale == 4i32) as i32 as u8,
        );
    }
    g_renderer_funcs.EndDraw.expect("non-null function pointer")(game);
}
static mut g_audio_mutex: *mut SDL_mutex = null::<SDL_mutex>() as *mut SDL_mutex;
static mut g_audiobuffer_cur: *mut u8 = null::<u8>() as *mut u8;
static mut g_audiobuffer_end: *mut u8 = null::<u8>() as *mut u8;
static mut g_audiobuffer: *mut u8 = null::<u8>() as *mut u8;
static mut g_frames_per_block: i32 = 0;
static mut g_audio_channels: u8 = 0;
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
unsafe extern "C" fn AudioCallback(userdata: *mut c_void, mut stream: *mut Uint8, mut len: i32) {
    let owner = &*(userdata as *const GameOwner);
    owner.with(|game| {
        if SDL_LockMutex(g_audio_mutex) != 0 {
            Die(b"Mutex lock failed!\0" as *const u8 as *const c_char);
        }
        while len != 0i32 {
            if g_audiobuffer_end.offset_from(g_audiobuffer_cur) as c_long == 0 as c_long {
                RtlRenderAudio(
                    g_audiobuffer as *mut i16,
                    g_frames_per_block,
                    g_audio_channels as i32,
                    game,
                );
                g_audiobuffer_cur = g_audiobuffer;
                g_audiobuffer_end = g_audiobuffer.offset(
                    ((g_frames_per_block * g_audio_channels as i32) as usize)
                        .wrapping_mul(size_of::<i16>() as usize) as isize,
                );
            }
            let n: i32 = IntMin(
                len,
                g_audiobuffer_end.offset_from(g_audiobuffer_cur) as c_long as i32,
            );
            if g_sdl_audio_mixer_volume == SDL_MIX_MAXVOLUME {
                memcpy(
                    stream as *mut c_void,
                    g_audiobuffer_cur as *const c_void,
                    n as size_t,
                );
            } else {
                SDL_memset(stream as *mut c_void, 0i32, n as size_t);
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
static mut g_renderer: *mut SDL_Renderer = null::<SDL_Renderer>() as *mut SDL_Renderer;
static mut g_texture: *mut SDL_Texture = null::<SDL_Texture>() as *mut SDL_Texture;
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
                as *const c_char,
        );
    }
    let renderer: *mut SDL_Renderer = SDL_CreateRenderer(
        g_window,
        -1i32,
        (if g_config.output_method as i32 == kOutputMethod_SDLSoftware as i32 {
            SDL_RENDERER_SOFTWARE as i32
        } else {
            SDL_RENDERER_ACCELERATED as i32 | SDL_RENDERER_PRESENTVSYNC as i32
        }) as Uint32,
    );
    if renderer.is_null() {
        printf(
            b"Failed to create renderer: %s\n\0" as *const u8 as *const c_char,
            SDL_GetError(),
        );
        return false_0 != 0;
    }
    let mut renderer_info: SDL_RendererInfo = SDL_RendererInfo {
        name: null::<c_char>(),
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
            sdl2_sys::SDL_HINT_RENDER_SCALE_QUALITY.as_ptr() as *const c_char,
            b"best\0" as *const u8 as *const c_char,
        );
    }
    let tex_mult: i32 = if g_ppu_render_flags & kPpuRenderFlags_4x4Mode7 as i32 != 0 {
        4i32
    } else {
        1i32
    };
    g_texture = SDL_CreateTexture(
        renderer,
        SDL_PIXELFORMAT_ARGB8888 as i32 as Uint32,
        SDL_TEXTUREACCESS_STREAMING as i32,
        g_snes_width * tex_mult,
        g_snes_height * tex_mult,
    );
    if g_texture.is_null() {
        printf(
            b"Failed to create texture: %s\n\0" as *const u8 as *const c_char,
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
    width: i32,
    height: i32,
    pixels: *mut *mut u8,
    pitch: *mut i32,
    game: &mut Game,
) {
    g_sdl_renderer_rect.w = width;
    g_sdl_renderer_rect.h = height;
    if SDL_LockTexture(
        g_texture,
        &raw mut g_sdl_renderer_rect,
        pixels as *mut *mut c_void,
        pitch,
    ) != 0i32
    {
        printf(
            b"Failed to lock texture: %s\n\0" as *const u8 as *const c_char,
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
        null::<SDL_Rect>(),
    );
    SDL_RenderPresent(g_renderer);
}
static mut kSdlRendererFuncs: RendererFuncs = RendererFuncs {
    Initialize: Some(SdlRenderer_Init as unsafe fn(*mut SDL_Window, &mut Game) -> bool),
    Destroy: Some(SdlRenderer_Destroy as unsafe fn(&mut Game) -> ()),
    BeginDraw: Some(
        SdlRenderer_BeginDraw as unsafe fn(i32, i32, *mut *mut u8, *mut i32, &mut Game) -> (),
    ),
    EndDraw: Some(SdlRenderer_EndDraw as unsafe fn(&mut Game) -> ()),
};
unsafe fn main_0(mut argc: i32, mut argv: *mut *mut c_char, owner: &GameOwner) -> i32 {
    argc -= 1;
    argv = argv.offset(1);
    let mut config_file: *const c_char = null::<c_char>();
    if argc >= 2i32 && strcmp(*argv.offset(0), b"--config\0" as *const u8 as *const c_char) == 0i32
    {
        config_file = *argv.offset(1);
        argc -= 2i32;
        argv = argv.offset(2);
    } else {
        SwitchDirectory();
    }
    if argc >= 1i32 && strcmp(*argv.offset(0), b"--debug\0" as *const u8 as *const c_char) == 0i32 {
        owner.with(|game| game.g_debug_flag = true_0 != 0);
        argc -= 1i32;
        argv = argv.offset(1);
    }
    ParseConfigFile(config_file);
    g_snes_width = g_config.extended_aspect_ratio as i32 * 2i32 + 256i32;
    g_snes_height = 240i32;
    g_ppu_render_flags = g_config.new_renderer as i32 * kPpuRenderFlags_NewRenderer as i32
        | g_config.enhanced_mode7 as i32 * kPpuRenderFlags_4x4Mode7 as i32
        | g_config.extend_y as i32 * kPpuRenderFlags_Height240 as i32
        | g_config.no_sprite_limits as i32 * kPpuRenderFlags_NoSpriteLimits as i32;
    if g_config.fullscreen as i32 == 1i32 {
        g_win_flags ^= SDL_WINDOW_FULLSCREEN_DESKTOP as i32 as u32;
    } else if g_config.fullscreen as i32 == 2i32 {
        g_win_flags ^= SDL_WINDOW_FULLSCREEN as i32 as u32;
    }
    g_current_window_scale = (if g_config.window_scale as i32 == 0i32 {
        2i32
    } else {
        IntMin(g_config.window_scale as i32, kMaxWindowScale as i32)
    }) as u8;
    if (g_config.audio_freq as i32) < 11025i32 || g_config.audio_freq as i32 > 48000i32 {
        g_config.audio_freq = kDefaultFreq as i32 as u16;
    }
    if (g_config.audio_channels as i32) < 1i32 || g_config.audio_channels as i32 > 2i32 {
        g_config.audio_channels = kDefaultChannels as i32 as u8;
    }
    if g_config.audio_samples as i32 <= 0i32
        || g_config.audio_samples as i32 & g_config.audio_samples as i32 - 1i32 != 0i32
    {
        g_config.audio_samples = kDefaultSamples as i32 as u16;
    }
    if SDL_Init(
        SDL_INIT_VIDEO as Uint32 | SDL_INIT_AUDIO as Uint32 | SDL_INIT_GAMECONTROLLER as Uint32,
    ) != 0i32
    {
        printf(
            b"Failed to init SDL: %s\n\0" as *const u8 as *const c_char,
            SDL_GetError(),
        );
        return 1i32;
    }
    let custom_size: bool = g_config.window_width != 0i32 && g_config.window_height != 0i32;
    let window_width: i32 = if custom_size as i32 != 0 {
        g_config.window_width
    } else {
        g_current_window_scale as i32 * g_snes_width
    };
    let window_height: i32 = if custom_size as i32 != 0 {
        g_config.window_height
    } else {
        g_current_window_scale as i32 * g_snes_height
    };
    if g_config.output_method as i32 == kOutputMethod_OpenGL as i32 {
        g_win_flags |= SDL_WINDOW_OPENGL as i32 as u32;
        OpenGLRenderer_Create(&raw mut g_renderer_funcs);
    } else {
        g_renderer_funcs = kSdlRendererFuncs;
    }
    let filename: *const c_char = if !(*argv.offset(0)).is_null() {
        *argv.offset(0) as *const c_char
    } else {
        b"sm.smc\0" as *const u8 as *const c_char
    };
    let snes: *mut Snes = owner.with(|game| SnesInit(filename, game));
    if snes.is_null() {
        let mut buf: [c_char; 256] = [0; 256];
        snprintf(
            &raw mut buf as *mut c_char,
            size_of::<[c_char; 256]>() as size_t,
            b"unable to load rom: %s\0" as *const u8 as *const c_char,
            filename,
        );
        Die(&raw mut buf as *mut c_char);
        return 1i32;
    }
    let window: *mut SDL_Window = SDL_CreateWindow(
        &raw const kWindowTitle as *const c_char,
        SDL_WINDOWPOS_UNDEFINED as i32,
        SDL_WINDOWPOS_UNDEFINED as i32,
        window_width,
        window_height,
        g_win_flags as Uint32,
    );
    if window.is_null() {
        printf(
            b"Failed to create window: %s\n\0" as *const u8 as *const c_char,
            SDL_GetError(),
        );
        return 1i32;
    }
    g_window = window;
    SDL_SetWindowHitTest(
        window,
        Some(
            HitTestCallback
                as unsafe extern "C" fn(
                    *mut SDL_Window,
                    *const SDL_Point,
                    *mut c_void,
                ) -> SDL_HitTestResult,
        ),
        NULL,
    );
    if !owner.with(|game| {
        g_renderer_funcs
            .Initialize
            .expect("non-null function pointer")(window, game)
    }) {
        return 1i32;
    }
    g_audio_mutex = SDL_CreateMutex();
    if g_audio_mutex.is_null() {
        Die(b"No mutex\0" as *const u8 as *const c_char);
    }
    owner.with(|game| game.g_spc_player = SpcPlayer_Create());
    owner.with(|game| SpcPlayer_Initialize(game.g_spc_player));
    let enable_audio: bool = true_0 != 0;
    if enable_audio {
        let mut want: SDL_AudioSpec = SDL_AudioSpec {
            freq: 0i32,
            format: 0,
            channels: 0,
            silence: 0,
            samples: 0,
            padding: 0,
            size: 0,
            callback: None,
            userdata: owner as *const GameOwner as *mut c_void,
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
            userdata: owner as *const GameOwner as *mut c_void,
        };
        want.freq = 44100i32;
        want.format = AUDIO_S16 as SDL_AudioFormat;
        want.channels = 2 as Uint8;
        want.samples = 2048 as Uint16;
        want.callback =
            Some(AudioCallback as unsafe extern "C" fn(*mut c_void, *mut Uint8, i32) -> ())
                as SDL_AudioCallback;
        g_audio_device =
            SDL_OpenAudioDevice(null::<c_char>(), 0i32, &raw mut want, &raw mut have, 0i32);
        if g_audio_device == 0 as SDL_AudioDeviceID {
            printf(
                b"Failed to open audio device: %s\n\0" as *const u8 as *const c_char,
                SDL_GetError(),
            );
            return 1i32;
        }
        g_audio_channels = 2u8;
        g_frames_per_block = 534i32 * have.freq / 32000i32;
        g_audiobuffer = malloc(
            ((g_frames_per_block * have.channels as i32) as size_t)
                .wrapping_mul(size_of::<i16>() as size_t),
        ) as *mut u8;
    }
    PpuBeginDrawing(
        (*snes).snes_ppu,
        &raw mut g_pixels as *mut u8,
        (256i32 * 4i32) as size_t,
        0u32,
    );
    PpuBeginDrawing(
        (*snes).my_ppu,
        &raw mut g_my_pixels as *mut u8,
        (256i32 * 4i32) as size_t,
        0u32,
    );
    mkdir(b"saves\0" as *const u8 as *const c_char, 0o755 as __mode_t);
    owner.with(|game| RtlReadSram(game));
    let mut i: i32 = 0i32;
    while i < SDL_NumJoysticks() {
        OpenOneGamepad(i);
        i += 1;
    }
    if g_config.autosave {
        owner.with(|game| HandleCommand((kKeys_Load as i32 + 0i32) as u32, true_0 != 0, game));
    }
    let mut running: bool = true_0 != 0;
    let mut lastTick: u32 = SDL_GetTicks() as u32;
    let mut frameCtr: u32 = 0u32;
    let mut audiopaused: u8 = true_0 as u8;
    let mut has_bug_in_title: bool = false_0 != 0;
    while running {
        let mut event: SDL_Event = SDL_Event { type_: 0 };
        while SDL_PollEvent(&raw mut event) != 0 {
            match event.type_ {
                1619 => {
                    OpenOneGamepad(event.cdevice.which);
                }
                1616 => {
                    owner.with(|game| {
                        HandleGamepadAxisInput(
                            event.caxis.which,
                            event.caxis.axis as i32,
                            event.caxis.value as i32,
                            game,
                        )
                    });
                }
                1617 | 1618 => {
                    let b: i32 = RemapSdlButton(event.cbutton.button as i32);
                    if b >= 0i32 {
                        owner.with(|game| {
                            HandleGamepadInput(
                                b,
                                event.type_ == SDL_CONTROLLERBUTTONDOWN as i32 as Uint32,
                                game,
                            )
                        });
                    }
                }
                1027 => {
                    if SDL_GetModState() as u32 & KMOD_CTRL as i32 as u32 != 0
                        && event.wheel.y != 0 as Sint32
                    {
                        ChangeWindowScale(if event.wheel.y > 0 as Sint32 {
                            1i32
                        } else {
                            -1i32
                        });
                    }
                }
                1025 => {
                    if event.button.button as i32 == SDL_BUTTON_LEFT
                        && event.button.state as i32 == SDL_PRESSED
                        && event.button.clicks as i32 == 2i32
                        && g_win_flags & SDL_WINDOW_FULLSCREEN_DESKTOP as i32 as u32 == 0u32
                        && g_win_flags & SDL_WINDOW_FULLSCREEN as i32 as u32 == 0u32
                        && SDL_GetModState() as u32 & KMOD_SHIFT as i32 as u32 != 0
                    {
                        g_win_flags ^= SDL_WINDOW_BORDERLESS as i32 as u32;
                        SDL_SetWindowBordered(
                            g_window,
                            if g_win_flags & SDL_WINDOW_BORDERLESS == 0u32 {
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
                            event.key.keysym.sym,
                            event.key.keysym.mod_ as i32,
                            true_0 != 0,
                            game,
                        )
                    });
                }
                769 => {
                    owner.with(|game| {
                        HandleInput(
                            event.key.keysym.sym,
                            event.key.keysym.mod_ as i32,
                            false_0 != 0,
                            game,
                        )
                    });
                }
                256 => running = false_0 != 0,
                _ => {}
            }
        }
        if g_paused as i32 != audiopaused as i32 {
            audiopaused = g_paused;
            if g_audio_device != 0 {
                SDL_PauseAudioDevice(g_audio_device, audiopaused as i32);
            }
        }
        if g_paused != 0 {
            SDL_Delay(16 as Uint32);
        } else {
            let mut inputs: i32 = g_input1_state;
            if g_input1_state & 0xf0i32 != 0 {
                g_gamepad_buttons = 0u8;
            }
            inputs |= g_gamepad_buttons as i32;
            let is_replay: u8 = owner.with(|game| RtlRunFrame(inputs, game)) as u8;
            frameCtr = frameCtr.wrapping_add(1);
            let (want_bug_in_title, disable_render) = owner.with(|game| {
                (*game.g_snes).disableRender = g_turbo ^ is_replay & g_replay_turbo != 0
                    && frameCtr & (if g_turbo as i32 != 0 { 0xfi32 } else { 0x7fi32 }) as u32
                        != 0u32;
                if !(*game.g_snes).disableRender {
                    DrawPpuFrameWithPerf(game);
                }
                (
                    game.g_got_mismatch_count != 0i32,
                    (*game.g_snes).disableRender,
                )
            });
            if want_bug_in_title as i32 != has_bug_in_title as i32 {
                has_bug_in_title = want_bug_in_title;
                let mut title: [c_char; 60] = [0; 60];
                if want_bug_in_title {
                    snprintf(
                        &raw mut title as *mut c_char,
                        size_of::<[c_char; 60]>() as size_t,
                        b"%s | BUG FOUND!\0" as *const u8 as *const c_char,
                        &raw const kWindowTitle as *const c_char,
                    );
                    SDL_SetWindowTitle(g_window, &raw mut title as *mut c_char);
                } else {
                    SDL_SetWindowTitle(g_window, &raw const kWindowTitle as *const c_char);
                }
            }
            let mut curTick: u32 = SDL_GetTicks() as u32;
            if !disable_render && !g_config.disable_frame_delay {
                static mut delays: [u8; 3] = [17, 17, 16];
                lastTick =
                    lastTick.wrapping_add(delays[frameCtr.wrapping_rem(3u32) as usize] as u32);
                if lastTick > curTick {
                    let mut delta: u32 = lastTick.wrapping_sub(curTick);
                    if delta > 500u32 {
                        lastTick = curTick.wrapping_sub(500u32);
                        delta = 500u32;
                    }
                    SDL_Delay(delta as Uint32);
                } else if curTick.wrapping_sub(lastTick) > 500u32 {
                    lastTick = curTick;
                }
            }
        }
    }
    if g_config.autosave {
        owner.with(|game| HandleCommand((kKeys_Save as i32 + 0i32) as u32, true_0 != 0, game));
    }
    SDL_PauseAudioDevice(g_audio_device, 1i32);
    SDL_CloseAudioDevice(g_audio_device);
    SDL_DestroyMutex(g_audio_mutex);
    free(g_audiobuffer as *mut c_void);
    owner.with(|game| g_renderer_funcs.Destroy.expect("non-null function pointer")(game));
    SDL_DestroyWindow(window);
    SDL_Quit();
    0i32
}
unsafe fn RenderDigit(mut dst: *mut u8, pitch: size_t, digit: i32, color: u32, big: bool) {
    static mut kFont: [u8; 100] = [
        0x1c, 0x36, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x36, 0x1c, 0x18, 0x1c, 0x1e, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x7e, 0x3e, 0x63, 0x60, 0x30, 0x18, 0xc, 0x6, 0x3, 0x63, 0x7f,
        0x3e, 0x63, 0x60, 0x60, 0x3c, 0x60, 0x60, 0x60, 0x63, 0x3e, 0x30, 0x38, 0x3c, 0x36, 0x33,
        0x7f, 0x30, 0x30, 0x30, 0x78, 0x7f, 0x3, 0x3, 0x3, 0x3f, 0x60, 0x60, 0x60, 0x63, 0x3e,
        0x1c, 0x6, 0x3, 0x3, 0x3f, 0x63, 0x63, 0x63, 0x63, 0x3e, 0x7f, 0x63, 0x60, 0x60, 0x30,
        0x18, 0xc, 0xc, 0xc, 0xc, 0x3e, 0x63, 0x63, 0x63, 0x3e, 0x63, 0x63, 0x63, 0x63, 0x3e, 0x3e,
        0x63, 0x63, 0x63, 0x7e, 0x60, 0x60, 0x60, 0x30, 0x1e,
    ];
    let mut p: *const u8 = (&raw const kFont as *const u8).offset((digit * 10i32) as isize);
    if !big {
        let mut y: i32 = 0i32;
        while y < 10i32 {
            let fresh0 = p;
            p = p.offset(1);
            let mut v: i32 = *fresh0 as i32;
            let mut x: i32 = 0i32;
            while v != 0 {
                if v & 1i32 != 0 {
                    *(dst as *mut u32).offset(x as isize) = color;
                }
                x += 1;
                v >>= 1i32;
            }
            y += 1;
            dst = dst.offset(pitch as isize);
        }
    } else {
        let mut y_0: i32 = 0i32;
        while y_0 < 10i32 {
            let fresh1 = p;
            p = p.offset(1);
            let mut v_0: i32 = *fresh1 as i32;
            let mut x_0: i32 = 0i32;
            while v_0 != 0 {
                if v_0 & 1i32 != 0 {
                    let ref mut fresh2 = *(dst as *mut u32).offset((x_0 * 2i32) as isize);
                    *fresh2 = color;
                    *(dst as *mut u32).offset((x_0 * 2i32 + 1i32) as isize) = *fresh2;
                    let ref mut fresh3 =
                        *(dst.offset(pitch as isize) as *mut u32).offset((x_0 * 2i32) as isize);
                    *fresh3 = color;
                    *(dst.offset(pitch as isize) as *mut u32)
                        .offset((x_0 * 2i32 + 1i32) as isize) = *fresh3;
                }
                x_0 += 1;
                v_0 >>= 1i32;
            }
            y_0 += 1;
            dst = dst.offset(pitch.wrapping_mul(2 as size_t) as isize);
        }
    };
}
unsafe fn RenderNumber(dst: *mut u8, pitch: size_t, n: i32, big: u8) {
    let mut buf: [c_char; 32] = [0; 32];
    let mut s: *mut c_char = null_mut::<c_char>();
    sprintf(
        &raw mut buf as *mut c_char,
        b"%d\0" as *const u8 as *const c_char,
        n,
    );
    s = &raw mut buf as *mut c_char;
    let mut i: i32 = 2i32 * 4i32;
    while *s != 0 {
        RenderDigit(
            dst.offset(
                (pitch.wrapping_add(i as size_t).wrapping_add(4 as size_t) << big as i32) as isize,
            ),
            pitch,
            *s as i32 - '0' as i32,
            0x404040u32,
            big != 0,
        );
        s = s.offset(1);
        i += 8i32 * 4i32;
    }
    s = &raw mut buf as *mut c_char;
    i = 2i32 * 4i32;
    while *s != 0 {
        RenderDigit(
            dst.offset((i << big as i32) as isize),
            pitch,
            *s as i32 - '0' as i32,
            0xffffffu32,
            big != 0,
        );
        s = s.offset(1);
        i += 8i32 * 4i32;
    }
}
unsafe fn HandleCommand(j: u32, pressed: bool, game: &mut Game) {
    if j <= kKeys_Controls_Last as i32 as u32 {
        static mut kKbdRemap: [u8; 13] = [0, 4, 5, 6, 7, 2, 3, 8, 0, 9, 1, 10, 11];
        if pressed {
            g_input1_state |= (1i32) << kKbdRemap[j as usize] as i32;
        } else {
            g_input1_state &= !((1i32) << kKbdRemap[j as usize] as i32);
        }
        return;
    }
    if j == kKeys_Turbo as i32 as u32 {
        g_turbo = pressed as u8;
        return;
    }
    if !pressed {
        return;
    }
    if j <= kKeys_Load_Last as i32 as u32 {
        RtlSaveLoad(
            kSaveLoad_Load as i32,
            j.wrapping_sub(kKeys_Load as i32 as u32) as i32,
            game,
        );
    } else if j <= kKeys_Save_Last as i32 as u32 {
        RtlSaveLoad(
            kSaveLoad_Save as i32,
            j.wrapping_sub(kKeys_Save as i32 as u32) as i32,
            game,
        );
    } else if j <= kKeys_Replay_Last as i32 as u32 {
        RtlSaveLoad(
            kSaveLoad_Replay as i32,
            j.wrapping_sub(kKeys_Replay as i32 as u32) as i32,
            game,
        );
    } else if j <= kKeys_LoadRef_Last as i32 as u32 {
        RtlSaveLoad(
            kSaveLoad_Load as i32,
            (256u32)
                .wrapping_add(j)
                .wrapping_sub(kKeys_LoadRef as i32 as u32) as i32,
            game,
        );
    } else if j <= kKeys_ReplayRef_Last as i32 as u32 {
        RtlSaveLoad(
            kSaveLoad_Replay as i32,
            (256u32)
                .wrapping_add(j)
                .wrapping_sub(kKeys_ReplayRef as i32 as u32) as i32,
            game,
        );
    } else {
        match j {
            113 => RtlCheat('w' as i32 as c_char, game),
            114 => RtlCheat('q' as i32 as c_char, game),
            115 => game.g_other_image = !game.g_other_image,
            116 => RtlClearKeyLog(game),
            117 => RtlStopReplay(game),
            118 => {
                g_win_flags ^= SDL_WINDOW_FULLSCREEN_DESKTOP as i32 as u32;
                SDL_SetWindowFullscreen(
                    g_window,
                    g_win_flags as Uint32 & SDL_WINDOW_FULLSCREEN_DESKTOP as i32 as Uint32,
                );
                g_cursor = (g_cursor == 0) as i32 as u8;
                SDL_ShowCursor(g_cursor as i32);
            }
            119 => RtlReset(1i32, game),
            120 => g_paused = (g_paused == 0) as i32 as u8,
            121 => g_paused = (g_paused == 0) as i32 as u8,
            123 => g_replay_turbo = (g_replay_turbo == 0) as i32 as u8,
            124 => ChangeWindowScale(1i32),
            125 => ChangeWindowScale(-1i32),
            126 => g_display_perf = (g_display_perf as i32 ^ 1i32) != 0,
            127 => {
                g_ppu_render_flags ^= kPpuRenderFlags_NewRenderer as i32;
                game.g_new_ppu = g_ppu_render_flags & kPpuRenderFlags_NewRenderer as i32 != 0i32;
            }
            128 | 129 => HandleVolumeAdjustment(if j == kKeys_VolumeUp as i32 as u32 {
                1i32
            } else {
                -1i32
            }),
            _ => {
                sm::native_assertion! {
                    '_c2rust_label;
                    b"0\0", b"src/main.c\0", 686u32, b"void HandleCommand(uint32, _Bool)\0"
                };
            }
        }
    };
}
unsafe fn HandleInput(keyCode: i32, keyMod: i32, pressed: bool, game: &mut Game) {
    let j: i32 = FindCmdForSdlKey(keyCode as SDL_Keycode, keyMod as SDL_Keymod);
    if j != 0i32 {
        HandleCommand(j as u32, pressed, game);
    }
}
unsafe fn OpenOneGamepad(i: i32) {
    if SDL_IsGameController(i) as u64 != 0 {
        let controller: *mut SDL_GameController = SDL_GameControllerOpen(i);
        if controller.is_null() {
            fprintf(
                stderr,
                b"Could not open gamepad %d: %s\n\0" as *const u8 as *const c_char,
                i,
                SDL_GetError(),
            );
        }
    }
}
unsafe fn RemapSdlButton(button: i32) -> i32 {
    match button {
        0 => kGamepadBtn_A,
        1 => kGamepadBtn_B,
        2 => kGamepadBtn_X,
        3 => kGamepadBtn_Y,
        4 => kGamepadBtn_Back,
        5 => kGamepadBtn_Guide,
        6 => kGamepadBtn_Start,
        7 => kGamepadBtn_L3,
        8 => kGamepadBtn_R3,
        9 => kGamepadBtn_L1,
        10 => kGamepadBtn_R1,
        11 => kGamepadBtn_DpadUp,
        12 => kGamepadBtn_DpadDown,
        13 => kGamepadBtn_DpadLeft,
        14 => kGamepadBtn_DpadRight,
        _ => -1i32,
    }
}
unsafe fn HandleGamepadInput(button: i32, pressed: bool, game: &mut Game) {
    if (g_gamepad_modifiers & ((1i32) << button) as u32 != 0) as i32 == pressed as i32 {
        return;
    }
    g_gamepad_modifiers ^= ((1i32) << button) as u32;
    if pressed {
        g_gamepad_last_cmd[button as usize] =
            FindCmdForGamepadButton(button, g_gamepad_modifiers) as u16;
    }
    if g_gamepad_last_cmd[button as usize] as i32 != 0i32 {
        HandleCommand(g_gamepad_last_cmd[button as usize] as u32, pressed, game);
    }
}
unsafe fn HandleVolumeAdjustment(volume_adjustment: i32) {
    g_sdl_audio_mixer_volume = IntMin(
        IntMax(
            0i32,
            g_sdl_audio_mixer_volume + volume_adjustment * (SDL_MIX_MAXVOLUME >> 4i32),
        ),
        SDL_MIX_MAXVOLUME,
    );
    printf(
        b"[SDL mixer volume]=%i\n\0" as *const u8 as *const c_char,
        g_sdl_audio_mixer_volume,
    );
}
unsafe fn ApproximateAtan2(mut y: c_float, mut x: c_float) -> c_float {
    let sign_mask: u32 = 0x80000000u32;
    let b: c_float = 0.596227f32;
    let ux_s: u32 = sign_mask & *(&raw mut x as *mut u32);
    let uy_s: u32 = sign_mask & *(&raw mut y as *mut u32);
    let q: c_float = ((!ux_s & uy_s) >> 29i32 | ux_s >> 30i32) as c_float;
    let mut bxy_a: c_float = b * x * y;
    if bxy_a < 0.0f32 {
        bxy_a = -bxy_a;
    }
    let num: c_float = bxy_a + y * y;
    let mut atan_1q: c_float = num / (x * x + bxy_a + num + 0.000001f32);
    let mut uatan_2q: u32 = ux_s ^ uy_s | *(&raw mut atan_1q as *mut u32);
    q + *(&raw mut uatan_2q as *mut c_float)
}
unsafe fn HandleGamepadAxisInput(gamepad_id: i32, axis: i32, value: i32, game: &mut Game) {
    static mut last_gamepad_id: i32 = 0;
    static mut last_x: i32 = 0;
    static mut last_y: i32 = 0;
    if axis == SDL_CONTROLLER_AXIS_LEFTX || axis == SDL_CONTROLLER_AXIS_LEFTY {
        if last_gamepad_id != gamepad_id {
            if value > -16000i32 && value < 16000i32 {
                return;
            }
            last_gamepad_id = gamepad_id;
            last_y = 0i32;
            last_x = last_y;
        }
        *if axis == SDL_CONTROLLER_AXIS_LEFTX {
            &raw mut last_x
        } else {
            &raw mut last_y
        } = value;
        let mut buttons: i32 = 0i32;
        if last_x * last_x + last_y * last_y >= 10000i32 * 10000i32 {
            static mut kSegmentToButtons: [u8; 8] = [
                ((1i32) << 4i32) as u8,
                ((1i32) << 4i32 | (1i32) << 7i32) as u8,
                ((1i32) << 7i32) as u8,
                ((1i32) << 7i32 | (1i32) << 5i32) as u8,
                ((1i32) << 5i32) as u8,
                ((1i32) << 5i32 | (1i32) << 6i32) as u8,
                ((1i32) << 6i32) as u8,
                ((1i32) << 6i32 | (1i32) << 4i32) as u8,
            ];
            let angle: u8 = (ApproximateAtan2(last_y as c_float, last_x as c_float) * 64.0f32
                + 0.5f32) as i32 as u8;
            buttons = kSegmentToButtons
                [((angle as i32 + 16i32 + 64i32) as u8 as i32 >> 5i32) as usize]
                as i32;
        }
        g_gamepad_buttons = buttons as u8;
    } else if (axis == SDL_CONTROLLER_AXIS_TRIGGERLEFT || axis == SDL_CONTROLLER_AXIS_TRIGGERRIGHT)
        && (value < 12000i32 || value >= 16000i32)
    {
        HandleGamepadInput(
            if axis == SDL_CONTROLLER_AXIS_TRIGGERLEFT {
                kGamepadBtn_L2
            } else {
                kGamepadBtn_R2
            },
            value >= 12000i32,
            game,
        );
    }
}
unsafe fn SwitchDirectory() {
    let mut buf: [c_char; 4096] = [0; 4096];
    if getcwd(
        &raw mut buf as *mut c_char,
        (size_of::<[c_char; 4096]>() as size_t).wrapping_sub(32 as size_t),
    )
    .is_null()
    {
        return;
    }
    let mut pos: size_t = strlen(&raw mut buf as *mut c_char);
    let mut step: i32 = 0i32;
    while pos != 0 as size_t && step < 3i32 {
        memcpy(
            (&raw mut buf as *mut c_char).offset(pos as isize) as *mut c_void,
            b"/sm.ini\0" as *const u8 as *const c_char as *const c_void,
            8 as size_t,
        );
        let f: *mut FILE = fopen(
            &raw mut buf as *mut c_char,
            b"rb\0" as *const u8 as *const c_char,
        ) as *mut FILE;
        if !f.is_null() {
            fclose(f);
            buf[pos as usize] = 0 as c_char;
            if step != 0i32 {
                printf(
                    b"Found sm.ini in %s\n\0" as *const u8 as *const c_char,
                    &raw mut buf as *mut c_char,
                );
                let err: i32 = chdir(&raw mut buf as *mut c_char);
            }
            return;
        }
        pos = pos.wrapping_sub(1);
        while pos != 0 as size_t
            && buf[pos as usize] as i32 != '/' as i32
            && buf[pos as usize] as i32 != '\\' as i32
        {
            pos = pos.wrapping_sub(1);
        }
        step += 1;
    }
}
pub const NULL: *mut c_void = null_mut::<c_void>();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
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
    let mut args_ptrs: Vec<*mut c_char> = args_strings
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut c_char)
        .chain(::core::iter::once(null_mut()))
        .collect();
    unsafe {
        ::std::process::exit(
            main_0((args_ptrs.len() - 1) as i32, args_ptrs.as_mut_ptr(), &owner) as i32,
        )
    }
}
