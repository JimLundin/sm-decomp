use crate::c_library::{fprintf, free, realloc, stderr, strtol};
use crate::host::Die;
use crate::sdl_constants;
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data};
pub use crate::types::{
    _IO_lock_t, __off64_t, __off_t, size_t, SDL_Keycode, Sint32, _IO_FILE, FILE,
};
use crate::util::{
    NextDelim, NextLineStripComments, NextPossiblyQuotedString, ReadWholeFile,
    ReplaceFilenameWithNewPath, SkipPrefix, SplitKeyValue, StringEqualsNoCase,
    StringStartsWithNoCase,
};
use ::core::ffi::{c_char, c_long, c_void};
use ::core::mem::size_of;
use ::core::ptr::{null, null_mut};
pub use sdl2_sys::{
    SDL_GameController, SDL_GetKeyFromName, SDL_Renderer, SDL_SysWMmsg, SDL_Texture, SDL_mutex,
};
pub type C2RustUnnamed = u32;
sdl_constants! {
    SDL_KeyCode as C2RustUnnamed;
    SDLK_ENDCALL, SDLK_CALL, SDLK_SOFTRIGHT, SDLK_SOFTLEFT, SDLK_AUDIOFASTFORWARD,
    SDLK_AUDIOREWIND, SDLK_APP2, SDLK_APP1, SDLK_SLEEP, SDLK_EJECT, SDLK_KBDILLUMUP,
    SDLK_KBDILLUMDOWN, SDLK_KBDILLUMTOGGLE, SDLK_DISPLAYSWITCH, SDLK_BRIGHTNESSUP,
    SDLK_BRIGHTNESSDOWN, SDLK_AC_BOOKMARKS, SDLK_AC_REFRESH, SDLK_AC_STOP, SDLK_AC_FORWARD,
    SDLK_AC_BACK, SDLK_AC_HOME, SDLK_AC_SEARCH, SDLK_COMPUTER, SDLK_CALCULATOR, SDLK_MAIL,
    SDLK_WWW, SDLK_MEDIASELECT, SDLK_AUDIOMUTE, SDLK_AUDIOPLAY, SDLK_AUDIOSTOP, SDLK_AUDIOPREV,
    SDLK_AUDIONEXT, SDLK_MODE, SDLK_RGUI, SDLK_RALT, SDLK_RSHIFT, SDLK_RCTRL, SDLK_LGUI,
    SDLK_LALT, SDLK_LSHIFT, SDLK_LCTRL, SDLK_KP_HEXADECIMAL, SDLK_KP_DECIMAL, SDLK_KP_OCTAL,
    SDLK_KP_BINARY, SDLK_KP_CLEARENTRY, SDLK_KP_CLEAR, SDLK_KP_PLUSMINUS, SDLK_KP_MEMDIVIDE,
    SDLK_KP_MEMMULTIPLY, SDLK_KP_MEMSUBTRACT, SDLK_KP_MEMADD, SDLK_KP_MEMCLEAR,
    SDLK_KP_MEMRECALL, SDLK_KP_MEMSTORE, SDLK_KP_EXCLAM, SDLK_KP_AT, SDLK_KP_SPACE,
    SDLK_KP_HASH, SDLK_KP_COLON, SDLK_KP_DBLVERTICALBAR, SDLK_KP_VERTICALBAR,
    SDLK_KP_DBLAMPERSAND, SDLK_KP_AMPERSAND, SDLK_KP_GREATER, SDLK_KP_LESS, SDLK_KP_PERCENT,
    SDLK_KP_POWER, SDLK_KP_XOR, SDLK_KP_F, SDLK_KP_E, SDLK_KP_D, SDLK_KP_C, SDLK_KP_B,
    SDLK_KP_A, SDLK_KP_BACKSPACE, SDLK_KP_TAB, SDLK_KP_RIGHTBRACE, SDLK_KP_LEFTBRACE,
    SDLK_KP_RIGHTPAREN, SDLK_KP_LEFTPAREN, SDLK_CURRENCYSUBUNIT, SDLK_CURRENCYUNIT,
    SDLK_DECIMALSEPARATOR, SDLK_THOUSANDSSEPARATOR, SDLK_KP_000, SDLK_KP_00, SDLK_EXSEL,
    SDLK_CRSEL, SDLK_CLEARAGAIN, SDLK_OPER, SDLK_OUT, SDLK_SEPARATOR, SDLK_RETURN2, SDLK_PRIOR,
    SDLK_CLEAR, SDLK_CANCEL, SDLK_SYSREQ, SDLK_ALTERASE, SDLK_KP_EQUALSAS400, SDLK_KP_COMMA,
    SDLK_VOLUMEDOWN, SDLK_VOLUMEUP, SDLK_MUTE, SDLK_FIND, SDLK_PASTE, SDLK_COPY, SDLK_CUT,
    SDLK_UNDO, SDLK_AGAIN, SDLK_STOP, SDLK_SELECT, SDLK_MENU, SDLK_HELP, SDLK_EXECUTE, SDLK_F24,
    SDLK_F23, SDLK_F22, SDLK_F21, SDLK_F20, SDLK_F19, SDLK_F18, SDLK_F17, SDLK_F16, SDLK_F15,
    SDLK_F14, SDLK_F13, SDLK_KP_EQUALS, SDLK_POWER, SDLK_APPLICATION, SDLK_KP_PERIOD, SDLK_KP_0,
    SDLK_KP_9, SDLK_KP_8, SDLK_KP_7, SDLK_KP_6, SDLK_KP_5, SDLK_KP_4, SDLK_KP_3, SDLK_KP_2,
    SDLK_KP_1, SDLK_KP_ENTER, SDLK_KP_PLUS, SDLK_KP_MINUS, SDLK_KP_MULTIPLY, SDLK_KP_DIVIDE,
    SDLK_NUMLOCKCLEAR, SDLK_UP, SDLK_DOWN, SDLK_LEFT, SDLK_RIGHT, SDLK_PAGEDOWN, SDLK_END,
    SDLK_DELETE, SDLK_PAGEUP, SDLK_HOME, SDLK_INSERT, SDLK_PAUSE, SDLK_SCROLLLOCK,
    SDLK_PRINTSCREEN, SDLK_F12, SDLK_F11, SDLK_F10, SDLK_F9, SDLK_F8, SDLK_F7, SDLK_F6, SDLK_F5,
    SDLK_F4, SDLK_F3, SDLK_F2, SDLK_F1, SDLK_CAPSLOCK, SDLK_z, SDLK_y, SDLK_x, SDLK_w, SDLK_v,
    SDLK_u, SDLK_t, SDLK_s, SDLK_r, SDLK_q, SDLK_p, SDLK_o, SDLK_n, SDLK_m, SDLK_l, SDLK_k,
    SDLK_j, SDLK_i, SDLK_h, SDLK_g, SDLK_f, SDLK_e, SDLK_d, SDLK_c, SDLK_b, SDLK_a,
    SDLK_BACKQUOTE, SDLK_UNDERSCORE, SDLK_CARET, SDLK_RIGHTBRACKET, SDLK_BACKSLASH,
    SDLK_LEFTBRACKET, SDLK_AT, SDLK_QUESTION, SDLK_GREATER, SDLK_EQUALS, SDLK_LESS,
    SDLK_SEMICOLON, SDLK_COLON, SDLK_9, SDLK_8, SDLK_7, SDLK_6, SDLK_5, SDLK_4, SDLK_3, SDLK_2,
    SDLK_1, SDLK_0, SDLK_SLASH, SDLK_PERIOD, SDLK_MINUS, SDLK_COMMA, SDLK_PLUS, SDLK_ASTERISK,
    SDLK_RIGHTPAREN, SDLK_LEFTPAREN, SDLK_QUOTE, SDLK_AMPERSAND, SDLK_DOLLAR, SDLK_PERCENT,
    SDLK_HASH, SDLK_QUOTEDBL, SDLK_EXCLAIM, SDLK_SPACE, SDLK_TAB, SDLK_BACKSPACE, SDLK_ESCAPE,
    SDLK_RETURN, SDLK_UNKNOWN,
}
pub use crate::types::SDL_Keymod;
pub const KMOD_RESERVED: SDL_Keymod = 32768;
sdl_constants! {
    SDL_Keymod as SDL_Keymod;
    KMOD_GUI, KMOD_ALT, KMOD_SHIFT, KMOD_CTRL, KMOD_SCROLL, KMOD_MODE, KMOD_CAPS, KMOD_NUM,
    KMOD_RGUI, KMOD_LGUI, KMOD_RALT, KMOD_LALT, KMOD_RCTRL, KMOD_LCTRL, KMOD_RSHIFT,
    KMOD_LSHIFT, KMOD_NONE,
}
pub use crate::types::KeyAction as C2RustUnnamed_0;
pub const kKeys_Total: C2RustUnnamed_0 = 130;
pub const kKeys_VolumeDown: C2RustUnnamed_0 = 129;
pub const kKeys_VolumeUp: C2RustUnnamed_0 = 128;
pub const kKeys_ToggleRenderer: C2RustUnnamed_0 = 127;
pub const kKeys_DisplayPerf: C2RustUnnamed_0 = 126;
pub const kKeys_WindowSmaller: C2RustUnnamed_0 = 125;
pub const kKeys_WindowBigger: C2RustUnnamed_0 = 124;
pub const kKeys_ReplayTurbo: C2RustUnnamed_0 = 123;
pub const kKeys_Turbo: C2RustUnnamed_0 = 122;
pub const kKeys_PauseDimmed: C2RustUnnamed_0 = 121;
pub const kKeys_Pause: C2RustUnnamed_0 = 120;
pub const kKeys_Reset: C2RustUnnamed_0 = 119;
pub const kKeys_Fullscreen: C2RustUnnamed_0 = 118;
pub const kKeys_StopReplay: C2RustUnnamed_0 = 117;
pub const kKeys_ClearKeyLog: C2RustUnnamed_0 = 116;
pub const kKeys_ToggleWhichFrame: C2RustUnnamed_0 = 115;
pub const kKeys_CheatJump: C2RustUnnamed_0 = 114;
pub const kKeys_CheatLife: C2RustUnnamed_0 = 113;
pub const kKeys_ReplayRef_Last: C2RustUnnamed_0 = 112;
pub const kKeys_ReplayRef: C2RustUnnamed_0 = 93;
pub const kKeys_LoadRef_Last: C2RustUnnamed_0 = 92;
pub const kKeys_LoadRef: C2RustUnnamed_0 = 73;
pub const kKeys_Replay_Last: C2RustUnnamed_0 = 72;
pub const kKeys_Replay: C2RustUnnamed_0 = 53;
pub const kKeys_Save_Last: C2RustUnnamed_0 = 52;
pub const kKeys_Save: C2RustUnnamed_0 = 33;
pub const kKeys_Load_Last: C2RustUnnamed_0 = 32;
pub const kKeys_Load: C2RustUnnamed_0 = 13;
pub const kKeys_Controls_Last: C2RustUnnamed_0 = 12;
pub const kKeys_Controls: C2RustUnnamed_0 = 1;
pub const kKeys_Null: C2RustUnnamed_0 = 0;
pub use crate::types::OutputMethod as C2RustUnnamed_1;
pub const kOutputMethod_OpenGL: C2RustUnnamed_1 = 2;
pub const kOutputMethod_SDLSoftware: C2RustUnnamed_1 = 1;
pub const kOutputMethod_SDL: C2RustUnnamed_1 = 0;
pub use crate::types::Config;
pub type C2RustUnnamed_2 = u32;
pub const kMsuEnabled_Opuz: C2RustUnnamed_2 = 4;
pub const kMsuEnabled_MsuDeluxe: C2RustUnnamed_2 = 2;
pub const kMsuEnabled_Msu: C2RustUnnamed_2 = 1;
pub use crate::types::GamepadButton as C2RustUnnamed_3;
pub const kGamepadBtn_Count: C2RustUnnamed_3 = 17;
pub const kGamepadBtn_R2: C2RustUnnamed_3 = 16;
pub const kGamepadBtn_L2: C2RustUnnamed_3 = 15;
pub const kGamepadBtn_DpadRight: C2RustUnnamed_3 = 14;
pub const kGamepadBtn_DpadLeft: C2RustUnnamed_3 = 13;
pub const kGamepadBtn_DpadDown: C2RustUnnamed_3 = 12;
pub const kGamepadBtn_DpadUp: C2RustUnnamed_3 = 11;
pub const kGamepadBtn_R1: C2RustUnnamed_3 = 10;
pub const kGamepadBtn_L1: C2RustUnnamed_3 = 9;
pub const kGamepadBtn_R3: C2RustUnnamed_3 = 8;
pub const kGamepadBtn_L3: C2RustUnnamed_3 = 7;
pub const kGamepadBtn_Start: C2RustUnnamed_3 = 6;
pub const kGamepadBtn_Guide: C2RustUnnamed_3 = 5;
pub const kGamepadBtn_Back: C2RustUnnamed_3 = 4;
pub const kGamepadBtn_Y: C2RustUnnamed_3 = 3;
pub const kGamepadBtn_X: C2RustUnnamed_3 = 2;
pub const kGamepadBtn_B: C2RustUnnamed_3 = 1;
pub const kGamepadBtn_A: C2RustUnnamed_3 = 0;
pub const kGamepadBtn_Invalid: C2RustUnnamed_3 = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GamepadMapEnt {
    pub modifiers: u32,
    pub cmd: u16,
    pub next: u16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyNameId {
    pub name: *const c_char,
    pub id: u16,
    pub size: u16,
}
pub const kKeyMod_ScanCode: C2RustUnnamed_4 = 512;
pub const kKeyMod_Shift: C2RustUnnamed_4 = 2048;
pub const kKeyMod_Ctrl: C2RustUnnamed_4 = 4096;
pub const kKeyMod_Alt: C2RustUnnamed_4 = 1024;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyMapHashEnt {
    pub key: u16,
    pub cmd: u16,
    pub next: u16,
}
pub type C2RustUnnamed_4 = u32;
#[inline]
unsafe fn atoi(mut __nptr: *const c_char) -> i32 {
    strtol(__nptr, NULL as *mut *mut c_char, 10i32) as i32
}
pub const SDLK_SCANCODE_MASK: i32 = (1i32) << 30i32;
pub static mut g_config: Config = Config {
    window_width: 0,
    window_height: 0,
    enhanced_mode7: false,
    new_renderer: false,
    ignore_aspect_ratio: false,
    fullscreen: 0,
    window_scale: 0,
    enable_audio: false,
    linear_filtering: false,
    output_method: 0,
    audio_freq: 0,
    audio_channels: 0,
    audio_samples: 0,
    autosave: false,
    extended_aspect_ratio: 0,
    extend_y: false,
    no_sprite_limits: false,
    display_perf_title: false,
    enable_msu: 0,
    resume_msu: false,
    disable_frame_delay: false,
    msuvolume: 0,
    features0: 0,
    link_graphics: null::<c_char>(),
    memory_buffer: null::<c_char>() as *mut c_char,
    shader: null::<c_char>(),
    msu_path: null::<c_char>(),
};
pub const N: i32 = 0;
static mut kDefaultKbdControls: [u16; 130] = [0; 130];
static mut kKeyNameId: [KeyNameId; 24] = [
    KeyNameId {
        name: b"Null\0" as *const u8 as *const c_char,
        id: kKeys_Null as i32 as u16,
        size: 65535u16,
    },
    KeyNameId {
        name: b"Controls\0" as *const u8 as *const c_char,
        id: kKeys_Controls as i32 as u16,
        size: (kKeys_Controls_Last as i32 - kKeys_Controls as i32 + 1i32) as u16,
    },
    KeyNameId {
        name: b"Load\0" as *const u8 as *const c_char,
        id: kKeys_Load as i32 as u16,
        size: (kKeys_Load_Last as i32 - kKeys_Load as i32 + 1i32) as u16,
    },
    KeyNameId {
        name: b"Save\0" as *const u8 as *const c_char,
        id: kKeys_Save as i32 as u16,
        size: (kKeys_Save_Last as i32 - kKeys_Save as i32 + 1i32) as u16,
    },
    KeyNameId {
        name: b"Replay\0" as *const u8 as *const c_char,
        id: kKeys_Replay as i32 as u16,
        size: (kKeys_Replay_Last as i32 - kKeys_Replay as i32 + 1i32) as u16,
    },
    KeyNameId {
        name: b"LoadRef\0" as *const u8 as *const c_char,
        id: kKeys_LoadRef as i32 as u16,
        size: (kKeys_LoadRef_Last as i32 - kKeys_LoadRef as i32 + 1i32) as u16,
    },
    KeyNameId {
        name: b"ReplayRef\0" as *const u8 as *const c_char,
        id: kKeys_ReplayRef as i32 as u16,
        size: (kKeys_ReplayRef_Last as i32 - kKeys_ReplayRef as i32 + 1i32) as u16,
    },
    KeyNameId {
        name: b"CheatLife\0" as *const u8 as *const c_char,
        id: kKeys_CheatLife as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"CheatJump\0" as *const u8 as *const c_char,
        id: kKeys_CheatJump as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"ToggleWhichFrame\0" as *const u8 as *const c_char,
        id: kKeys_ToggleWhichFrame as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"ClearKeyLog\0" as *const u8 as *const c_char,
        id: kKeys_ClearKeyLog as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"StopReplay\0" as *const u8 as *const c_char,
        id: kKeys_StopReplay as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"Fullscreen\0" as *const u8 as *const c_char,
        id: kKeys_Fullscreen as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"Reset\0" as *const u8 as *const c_char,
        id: kKeys_Reset as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"Pause\0" as *const u8 as *const c_char,
        id: kKeys_Pause as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"PauseDimmed\0" as *const u8 as *const c_char,
        id: kKeys_PauseDimmed as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"Turbo\0" as *const u8 as *const c_char,
        id: kKeys_Turbo as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"ReplayTurbo\0" as *const u8 as *const c_char,
        id: kKeys_ReplayTurbo as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"WindowBigger\0" as *const u8 as *const c_char,
        id: kKeys_WindowBigger as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"WindowSmaller\0" as *const u8 as *const c_char,
        id: kKeys_WindowSmaller as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"VolumeUp\0" as *const u8 as *const c_char,
        id: kKeys_VolumeUp as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"VolumeDown\0" as *const u8 as *const c_char,
        id: kKeys_VolumeDown as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"DisplayPerf\0" as *const u8 as *const c_char,
        id: kKeys_DisplayPerf as i32 as u16,
        size: 1u16,
    },
    KeyNameId {
        name: b"ToggleRenderer\0" as *const u8 as *const c_char,
        id: kKeys_ToggleRenderer as i32 as u16,
        size: 1u16,
    },
];
static mut keymap_hash_first: [u16; 255] = [0; 255];
static mut keymap_hash: *mut KeyMapHashEnt = null::<KeyMapHashEnt>() as *mut KeyMapHashEnt;
static mut keymap_hash_size: i32 = 0;
static mut has_keynameid: [bool; 24] = [false; 24];
unsafe fn KeyMapHash_Add(key: u16, cmd: u16) -> bool {
    if keymap_hash_size & 0xffi32 == 0i32 {
        if keymap_hash_size > 10000i32 {
            Die(b"Too many keys\0" as *const u8 as *const c_char);
        }
        keymap_hash = realloc(
            keymap_hash as *mut c_void,
            (size_of::<KeyMapHashEnt>() as size_t)
                .wrapping_mul((keymap_hash_size + 256i32) as size_t),
        ) as *mut KeyMapHashEnt;
    }
    let fresh1 = keymap_hash_size;
    keymap_hash_size = keymap_hash_size + 1;
    let i: i32 = fresh1;
    let ent: *mut KeyMapHashEnt = keymap_hash.offset(i as isize);
    (*ent).key = key;
    (*ent).cmd = cmd;
    (*ent).next = 0u16;
    let j: i32 = (key as u32).wrapping_rem(255u32) as i32;
    let mut cur: *mut u16 = (&raw mut keymap_hash_first as *mut u16).offset(j as isize);
    while *cur != 0 {
        let ent_0: *mut KeyMapHashEnt = keymap_hash.offset((*cur as i32 - 1i32) as isize);
        if (*ent_0).key as i32 == key as i32 {
            return false_0 != 0;
        }
        cur = &raw mut (*ent_0).next;
    }
    *cur = (i + 1i32) as u16;
    true_0 != 0
}
unsafe fn KeyMapHash_Find(key: u16) -> i32 {
    let mut i: i32 = keymap_hash_first[(key as i32 % 255i32) as usize] as i32;
    while i != 0 {
        let ent: *mut KeyMapHashEnt = keymap_hash.offset((i - 1i32) as isize);
        if (*ent).key as i32 == key as i32 {
            return (*ent).cmd as i32;
        }
        i = (*ent).next as i32;
    }
    0i32
}
pub unsafe fn FindCmdForSdlKey(code: SDL_Keycode, mod_0: SDL_Keymod) -> i32 {
    if code & !(SDLK_SCANCODE_MASK as SDL_Keycode | 0x1ff as SDL_Keycode) != 0 {
        return 0i32;
    }
    let mut key: i32 = 0i32;
    if code != SDLK_LALT as i32 as SDL_Keycode && code != SDLK_RALT as i32 as SDL_Keycode {
        key |= if mod_0 & KMOD_ALT as i32 as u32 != 0 {
            kKeyMod_Alt as i32
        } else {
            0i32
        };
    }
    if code != SDLK_LCTRL as i32 as SDL_Keycode && code != SDLK_RCTRL as i32 as SDL_Keycode {
        key |= if mod_0 & KMOD_CTRL as i32 as u32 != 0 {
            kKeyMod_Ctrl as i32
        } else {
            0i32
        };
    }
    if code != SDLK_LSHIFT as i32 as SDL_Keycode && code != SDLK_RSHIFT as i32 as SDL_Keycode {
        key |= if mod_0 & KMOD_SHIFT as i32 as u32 != 0 {
            kKeyMod_Shift as i32
        } else {
            0i32
        };
    }
    key |= ((if code & SDLK_SCANCODE_MASK as SDL_Keycode != 0 {
        kKeyMod_ScanCode as i32 as SDL_Keycode
    } else {
        0 as SDL_Keycode
    }) | code & kKeyMod_ScanCode as i32 as SDL_Keycode - 1 as SDL_Keycode);
    KeyMapHash_Find(key as u16)
}
unsafe fn ParseKeyArray(mut value: *mut c_char, mut cmd: i32, size: i32) {
    let mut s: *mut c_char = null_mut::<c_char>();
    let mut i: i32 = 0i32;
    while i < size && {
        s = NextDelim(&raw mut value, ',' as i32);
        !s.is_null()
    } {
        if (*s as i32 != 0i32) {
            let mut key_with_mod: i32 = 0i32;
            loop {
                if !StringStartsWithNoCase(s, b"Shift+\0" as *const u8 as *const c_char).is_null() {
                    key_with_mod |= kKeyMod_Shift as i32;
                    s = s.offset(6);
                } else if !StringStartsWithNoCase(s, b"Ctrl+\0" as *const u8 as *const c_char)
                    .is_null()
                {
                    key_with_mod |= kKeyMod_Ctrl as i32;
                    s = s.offset(5);
                } else {
                    if StringStartsWithNoCase(s, b"Alt+\0" as *const u8 as *const c_char).is_null()
                    {
                        break;
                    }
                    key_with_mod |= kKeyMod_Alt as i32;
                    s = s.offset(4);
                }
            }
            let key: SDL_Keycode = SDL_GetKeyFromName(s);
            if key == SDLK_UNKNOWN as i32 as SDL_Keycode {
                fprintf(
                    stderr,
                    b"Unknown key: '%s'\n\0" as *const u8 as *const c_char,
                    s,
                );
            } else if !KeyMapHash_Add(
                (key_with_mod as SDL_Keycode
                    | (if key & SDLK_SCANCODE_MASK as SDL_Keycode != 0 {
                        kKeyMod_ScanCode as i32 as SDL_Keycode
                    } else {
                        0 as SDL_Keycode
                    })
                    | key & kKeyMod_ScanCode as i32 as SDL_Keycode - 1 as SDL_Keycode)
                    as u16,
                cmd as u16,
            ) {
                fprintf(
                    stderr,
                    b"Duplicate key: '%s'\n\0" as *const u8 as *const c_char,
                    s,
                );
            }
        }
        i += 1;
        cmd += (cmd != 0i32) as i32;
    }
}
static mut joymap_first: [u16; 17] = [0; 17];
static mut joymap_ents: *mut GamepadMapEnt = null::<GamepadMapEnt>() as *mut GamepadMapEnt;
static mut joymap_size: i32 = 0;
static mut has_joypad_controls: bool = false;
unsafe fn CountBits32(mut n: u32) -> i32 {
    let mut count: i32 = 0i32;
    while n != 0u32 {
        n &= n.wrapping_sub(1u32);
        count += 1;
    }
    count
}
unsafe fn GamepadMap_Add(button: i32, modifiers: u32, cmd: u16) {
    if joymap_size & 0xffi32 == 0i32 {
        if joymap_size > 1000i32 {
            Die(b"Too many joypad keys\0" as *const u8 as *const c_char);
        }
        joymap_ents = realloc(
            joymap_ents as *mut c_void,
            (size_of::<GamepadMapEnt>() as size_t).wrapping_mul((joymap_size + 64i32) as size_t),
        ) as *mut GamepadMapEnt;
        if joymap_ents.is_null() {
            Die(b"realloc failure\0" as *const u8 as *const c_char);
        }
    }
    let mut p: *mut u16 = (&raw mut joymap_first as *mut u16).offset(button as isize);
    let cb: i32 = CountBits32(modifiers);
    while *p as i32 != 0
        && cb < CountBits32((*joymap_ents.offset((*p as i32 - 1i32) as isize)).modifiers)
    {
        p = &raw mut (*joymap_ents.offset((*p as i32 - 1i32) as isize)).next;
    }
    let fresh0 = joymap_size;
    joymap_size = joymap_size + 1;
    let i: i32 = fresh0;
    let ent: *mut GamepadMapEnt = joymap_ents.offset(i as isize);
    (*ent).modifiers = modifiers;
    (*ent).cmd = cmd;
    (*ent).next = *p;
    *p = (i + 1i32) as u16;
}
pub unsafe fn FindCmdForGamepadButton(button: i32, modifiers: u32) -> i32 {
    let mut ent: *mut GamepadMapEnt = null_mut::<GamepadMapEnt>();
    let mut e: i32 = joymap_first[button as usize] as i32;
    while e != 0i32 {
        ent = joymap_ents.offset((e - 1i32) as isize);
        if modifiers & (*ent).modifiers == (*ent).modifiers {
            return (*ent).cmd as i32;
        }
        e = (*ent).next as i32;
    }
    0i32
}
unsafe fn ParseGamepadButtonName(value: *mut *const c_char) -> i32 {
    let s: *const c_char = *value;
    static mut kGamepadKeyNames: [*const c_char; 19] = [
        b"Back\0" as *const u8 as *const c_char,
        b"Guide\0" as *const u8 as *const c_char,
        b"Start\0" as *const u8 as *const c_char,
        b"L3\0" as *const u8 as *const c_char,
        b"R3\0" as *const u8 as *const c_char,
        b"L1\0" as *const u8 as *const c_char,
        b"R1\0" as *const u8 as *const c_char,
        b"DpadUp\0" as *const u8 as *const c_char,
        b"DpadDown\0" as *const u8 as *const c_char,
        b"DpadLeft\0" as *const u8 as *const c_char,
        b"DpadRight\0" as *const u8 as *const c_char,
        b"L2\0" as *const u8 as *const c_char,
        b"R2\0" as *const u8 as *const c_char,
        b"Lb\0" as *const u8 as *const c_char,
        b"Rb\0" as *const u8 as *const c_char,
        b"A\0" as *const u8 as *const c_char,
        b"B\0" as *const u8 as *const c_char,
        b"X\0" as *const u8 as *const c_char,
        b"Y\0" as *const u8 as *const c_char,
    ];
    static mut kGamepadKeyIds: [u8; 19] = [
        kGamepadBtn_Back as u8,
        kGamepadBtn_Guide as u8,
        kGamepadBtn_Start as u8,
        kGamepadBtn_L3 as u8,
        kGamepadBtn_R3 as u8,
        kGamepadBtn_L1 as u8,
        kGamepadBtn_R1 as u8,
        kGamepadBtn_DpadUp as u8,
        kGamepadBtn_DpadDown as u8,
        kGamepadBtn_DpadLeft as u8,
        kGamepadBtn_DpadRight as u8,
        kGamepadBtn_L2 as u8,
        kGamepadBtn_R2 as u8,
        kGamepadBtn_L1 as u8,
        kGamepadBtn_R1 as u8,
        kGamepadBtn_A as u8,
        kGamepadBtn_B as u8,
        kGamepadBtn_X as u8,
        kGamepadBtn_Y as u8,
    ];
    let mut i: size_t = 0 as size_t;
    while i
        != (size_of::<[*const c_char; 19]>() as usize)
            .wrapping_div(size_of::<*const c_char>() as usize)
    {
        let r: *const c_char = StringStartsWithNoCase(s, kGamepadKeyNames[i as usize]);
        if !r.is_null() {
            *value = r;
            return kGamepadKeyIds[i as usize] as i32;
        }
        i = i.wrapping_add(1);
    }
    kGamepadBtn_Invalid
}
static mut kDefaultGamepadCmds: [u8; 12] = [
    kGamepadBtn_DpadUp as u8,
    kGamepadBtn_DpadDown as u8,
    kGamepadBtn_DpadLeft as u8,
    kGamepadBtn_DpadRight as u8,
    kGamepadBtn_Back as u8,
    kGamepadBtn_Start as u8,
    kGamepadBtn_B as u8,
    kGamepadBtn_A as u8,
    kGamepadBtn_Y as u8,
    kGamepadBtn_X as u8,
    kGamepadBtn_L1 as u8,
    kGamepadBtn_R1 as u8,
];
unsafe fn ParseGamepadArray(mut value: *mut c_char, mut cmd: i32, size: i32) {
    let mut s: *mut c_char = null_mut::<c_char>();
    let mut i: i32 = 0i32;
    while i < size && {
        s = NextDelim(&raw mut value, ',' as i32);
        !s.is_null()
    } {
        if (*s as i32 != 0i32) {
            let mut modifiers: u32 = 0u32;
            let mut ss: *const c_char = s;
            loop {
                let button: i32 = ParseGamepadButtonName(&raw mut ss);
                if (button != kGamepadBtn_Invalid) {
                    while *ss as i32 == ' ' as i32 || *ss as i32 == '\t' as i32 {
                        ss = ss.offset(1);
                    }
                    if *ss as i32 == '+' as i32 {
                        ss = ss.offset(1);
                        modifiers |= ((1i32) << button) as u32;
                        continue;
                    } else if *ss as i32 == 0i32 {
                        GamepadMap_Add(button, modifiers, cmd as u16);
                        break;
                    }
                }
                fprintf(
                    stderr,
                    b"Unknown gamepad button: '%s'\n\0" as *const u8 as *const c_char,
                    s,
                );
                break;
            }
        }
        i += 1;
        cmd += (cmd != 0i32) as i32;
    }
}
unsafe fn RegisterDefaultKeys() {
    let mut i: i32 = 1i32;
    while (i as usize)
        < (size_of::<[KeyNameId; 24]>() as usize).wrapping_div(size_of::<KeyNameId>() as usize)
    {
        if !has_keynameid[i as usize] {
            let size: i32 = kKeyNameId[i as usize].size as i32;
            let mut k: i32 = kKeyNameId[i as usize].id as i32;
            let mut j: i32 = 0i32;
            while j < size {
                KeyMapHash_Add(kDefaultKbdControls[k as usize], k as u16);
                j += 1;
                k += 1;
            }
        }
        i += 1;
    }
    if !has_joypad_controls {
        let mut i_0: i32 = 0i32;
        while (i_0 as usize)
            < (size_of::<[u8; 12]>() as usize).wrapping_div(size_of::<u8>() as usize)
        {
            GamepadMap_Add(
                kDefaultGamepadCmds[i_0 as usize] as i32,
                0u32,
                (kKeys_Controls as i32 + i_0) as u16,
            );
            i_0 += 1;
        }
    }
}
unsafe fn GetIniSection(s: *const c_char) -> i32 {
    if StringEqualsNoCase(s, b"[KeyMap]\0" as *const u8 as *const c_char) {
        return 0i32;
    }
    if StringEqualsNoCase(s, b"[Graphics]\0" as *const u8 as *const c_char) {
        return 1i32;
    }
    if StringEqualsNoCase(s, b"[Sound]\0" as *const u8 as *const c_char) {
        return 2i32;
    }
    if StringEqualsNoCase(s, b"[General]\0" as *const u8 as *const c_char) {
        return 3i32;
    }
    if StringEqualsNoCase(s, b"[Features]\0" as *const u8 as *const c_char) {
        return 4i32;
    }
    if StringEqualsNoCase(s, b"[GamepadMap]\0" as *const u8 as *const c_char) {
        return 5i32;
    }
    -1i32
}
pub unsafe fn ParseBool(mut value: *const c_char, result: *mut bool) -> bool {
    let mut rv: bool = false_0 != 0;
    let fresh2 = value;
    value = value.offset(1);
    match *fresh2 as i32 | 32i32 {
        48 => {
            if (*value as i32 != 0i32) {
                return false_0 != 0;
            }
        }
        102 => {
            if !StringEqualsNoCase(value, b"alse\0" as *const u8 as *const c_char) {
                return false_0 != 0;
            }
        }
        110 => {
            if !StringEqualsNoCase(value, b"o\0" as *const u8 as *const c_char) {
                return false_0 != 0;
            }
        }
        111 => {
            rv = *value as i32 | 32i32 == 'n' as i32;
            if !StringEqualsNoCase(
                value,
                if rv as i32 != 0 {
                    b"n\0" as *const u8 as *const c_char
                } else {
                    b"ff\0" as *const u8 as *const c_char
                },
            ) {
                return false_0 != 0;
            }
        }
        49 => {
            rv = true_0 != 0;
            if (*value as i32 != 0i32) {
                return false_0 != 0;
            }
        }
        121 => {
            rv = true_0 != 0;
            if !StringEqualsNoCase(value, b"es\0" as *const u8 as *const c_char) {
                return false_0 != 0;
            }
        }
        116 => {
            rv = true_0 != 0;
            if !StringEqualsNoCase(value, b"rue\0" as *const u8 as *const c_char) {
                return false_0 != 0;
            }
        }
        _ => return false_0 != 0,
    }
    if !result.is_null() {
        *result = rv;
        return true_0 != 0;
    }
    rv
}
unsafe fn HandleIniConfig(section: i32, key: *const c_char, mut value: *mut c_char) -> bool {
    if section == 0i32 {
        let mut i: i32 = 0i32;
        while (i as usize)
            < (size_of::<[KeyNameId; 24]>() as usize).wrapping_div(size_of::<KeyNameId>() as usize)
        {
            if StringEqualsNoCase(key, kKeyNameId[i as usize].name) {
                has_keynameid[i as usize] = true_0 != 0;
                ParseKeyArray(
                    value,
                    kKeyNameId[i as usize].id as i32,
                    kKeyNameId[i as usize].size as i32,
                );
                return true_0 != 0;
            }
            i += 1;
        }
    } else if section == 5i32 {
        let mut i_0: i32 = 0i32;
        while (i_0 as usize)
            < (size_of::<[KeyNameId; 24]>() as usize).wrapping_div(size_of::<KeyNameId>() as usize)
        {
            if StringEqualsNoCase(key, kKeyNameId[i_0 as usize].name) {
                if i_0 == 1i32 {
                    has_joypad_controls = true_0 != 0;
                }
                ParseGamepadArray(
                    value,
                    kKeyNameId[i_0 as usize].id as i32,
                    kKeyNameId[i_0 as usize].size as i32,
                );
                return true_0 != 0;
            }
            i_0 += 1;
        }
    } else if section == 1i32 {
        if StringEqualsNoCase(key, b"WindowSize\0" as *const u8 as *const c_char) {
            let mut s: *mut c_char = null_mut::<c_char>();
            if StringEqualsNoCase(value, b"Auto\0" as *const u8 as *const c_char) {
                g_config.window_width = 0i32;
                g_config.window_height = 0i32;
                return true_0 != 0;
            }
            loop {
                s = NextDelim(&raw mut value, 'x' as i32);
                if s.is_null() {
                    break;
                }
                if g_config.window_width == 0i32 {
                    g_config.window_width = atoi(s);
                } else {
                    g_config.window_height = atoi(s);
                    return true_0 != 0;
                }
            }
        } else if StringEqualsNoCase(key, b"EnhancedMode7\0" as *const u8 as *const c_char) {
            return ParseBool(value, &raw mut g_config.enhanced_mode7);
        } else if StringEqualsNoCase(key, b"NewRenderer\0" as *const u8 as *const c_char) {
            return ParseBool(value, &raw mut g_config.new_renderer);
        } else if StringEqualsNoCase(key, b"IgnoreAspectRatio\0" as *const u8 as *const c_char) {
            return ParseBool(value, &raw mut g_config.ignore_aspect_ratio);
        } else if StringEqualsNoCase(key, b"Fullscreen\0" as *const u8 as *const c_char) {
            g_config.fullscreen = strtol(value, NULL_0 as *mut *mut c_char, 10i32) as u8;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"WindowScale\0" as *const u8 as *const c_char) {
            g_config.window_scale = strtol(value, NULL_0 as *mut *mut c_char, 10i32) as u8;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"OutputMethod\0" as *const u8 as *const c_char) {
            g_config.output_method =
                (if StringEqualsNoCase(value, b"SDL-Software\0" as *const u8 as *const c_char)
                    as i32
                    != 0
                {
                    kOutputMethod_SDLSoftware as i32
                } else if StringEqualsNoCase(value, b"OpenGL\0" as *const u8 as *const c_char)
                    as i32
                    != 0
                {
                    kOutputMethod_OpenGL as i32
                } else {
                    kOutputMethod_SDL as i32
                }) as u8;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"LinearFiltering\0" as *const u8 as *const c_char) {
            return ParseBool(value, &raw mut g_config.linear_filtering);
        } else if StringEqualsNoCase(key, b"NoSpriteLimits\0" as *const u8 as *const c_char) {
            return ParseBool(value, &raw mut g_config.no_sprite_limits);
        } else if StringEqualsNoCase(key, b"LinkGraphics\0" as *const u8 as *const c_char) {
            g_config.link_graphics = value;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"Shader\0" as *const u8 as *const c_char) {
            g_config.shader = if *value as i32 != 0 {
                value
            } else {
                null_mut::<c_char>()
            };
            return true_0 != 0;
        }
    } else if section == 2i32 {
        if StringEqualsNoCase(key, b"EnableAudio\0" as *const u8 as *const c_char) {
            return ParseBool(value, &raw mut g_config.enable_audio);
        } else if StringEqualsNoCase(key, b"AudioFreq\0" as *const u8 as *const c_char) {
            g_config.audio_freq = strtol(value, NULL_0 as *mut *mut c_char, 10i32) as u16;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"AudioChannels\0" as *const u8 as *const c_char) {
            g_config.audio_channels = strtol(value, NULL_0 as *mut *mut c_char, 10i32) as u8;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"AudioSamples\0" as *const u8 as *const c_char) {
            g_config.audio_samples = strtol(value, NULL_0 as *mut *mut c_char, 10i32) as u16;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"EnableMSU\0" as *const u8 as *const c_char) {
            if StringEqualsNoCase(value, b"opuz\0" as *const u8 as *const c_char) {
                g_config.enable_msu = kMsuEnabled_Opuz as i32 as u8;
            } else if StringEqualsNoCase(value, b"deluxe\0" as *const u8 as *const c_char) {
                g_config.enable_msu = kMsuEnabled_MsuDeluxe as i32 as u8;
            } else if StringEqualsNoCase(value, b"deluxe-opuz\0" as *const u8 as *const c_char) {
                g_config.enable_msu =
                    (kMsuEnabled_MsuDeluxe as i32 | kMsuEnabled_Opuz as i32) as u8;
            } else {
                return ParseBool(value, &raw mut g_config.enable_msu as *mut bool);
            }
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"MSUPath\0" as *const u8 as *const c_char) {
            g_config.msu_path = value;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"MSUVolume\0" as *const u8 as *const c_char) {
            g_config.msuvolume = atoi(value) as u8;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"ResumeMSU\0" as *const u8 as *const c_char) {
            return ParseBool(value, &raw mut g_config.resume_msu);
        }
    } else if section == 3i32 {
        if StringEqualsNoCase(key, b"Autosave\0" as *const u8 as *const c_char) {
            g_config.autosave = strtol(value, NULL_0 as *mut *mut c_char, 10i32) != 0;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"DisplayPerfInTitle\0" as *const u8 as *const c_char) {
            return ParseBool(value, &raw mut g_config.display_perf_title);
        } else if StringEqualsNoCase(key, b"DisableFrameDelay\0" as *const u8 as *const c_char) {
            return ParseBool(value, &raw mut g_config.disable_frame_delay);
        }
    } else {
        section == 4i32;
    }
    false_0 != 0
}
unsafe fn ParseOneConfigFile(filename: *const c_char, depth: i32) -> bool {
    let mut filedata: *mut c_char = ReadWholeFile(filename, null_mut::<size_t>()) as *mut c_char;
    let mut p: *mut c_char = null_mut::<c_char>();
    if filedata.is_null() {
        return false_0 != 0;
    }
    let mut section: i32 = -2i32;
    g_config.memory_buffer = filedata;
    let mut lineno: i32 = 1i32;
    loop {
        p = NextLineStripComments(&raw mut filedata);
        if p.is_null() {
            break;
        }
        if (*p as i32 != 0i32) {
            if *p as i32 == '[' as i32 {
                section = GetIniSection(p);
                if section < 0i32 {
                    fprintf(
                        stderr,
                        b"%s:%d: Invalid .ini section %s\n\0" as *const u8 as *const c_char,
                        filename,
                        lineno,
                        p,
                    );
                }
            } else if *p as i32 == '!' as i32
                && !SkipPrefix(p.offset(1), b"include \0" as *const u8 as *const c_char).is_null()
            {
                let mut tt: *mut c_char = p.offset(8);
                let new_filename: *mut c_char =
                    ReplaceFilenameWithNewPath(filename, NextPossiblyQuotedString(&raw mut tt));
                if depth > 10i32 || !ParseOneConfigFile(new_filename, depth + 1i32) {
                    fprintf(
                        stderr,
                        b"Warning: Unable to read %s\n\0" as *const u8 as *const c_char,
                        new_filename,
                    );
                }
                free(new_filename as *mut c_void);
            } else if section == -2i32 {
                fprintf(
                    stderr,
                    b"%s:%d: Expecting [section]\n\0" as *const u8 as *const c_char,
                    filename,
                    lineno,
                );
            } else {
                let v: *mut c_char = SplitKeyValue(p);
                if v.is_null() {
                    fprintf(
                        stderr,
                        b"%s:%d: Expecting 'key=value'\n\0" as *const u8 as *const c_char,
                        filename,
                        lineno,
                    );
                } else if section >= 0i32 && !HandleIniConfig(section, p, v) {
                    fprintf(
                        stderr,
                        b"%s:%d: Can't parse '%s'\n\0" as *const u8 as *const c_char,
                        filename,
                        lineno,
                        p,
                    );
                }
            }
        }
        lineno += 1;
    }
    true_0 != 0
}
pub unsafe fn ParseConfigFile(mut filename: *const c_char) {
    g_config.msuvolume = 100u8;
    if !filename.is_null()
        || !ParseOneConfigFile(b"sm.user.ini\0" as *const u8 as *const c_char, 0i32)
    {
        if filename.is_null() {
            filename = b"sm.ini\0" as *const u8 as *const c_char;
        }
        if !ParseOneConfigFile(filename, 0i32) {
            fprintf(
                stderr,
                b"Warning: Unable to read config file %s\n\0" as *const u8 as *const c_char,
                filename,
            );
        }
    }
    RegisterDefaultKeys();
}
pub const NULL: *mut c_void = null_mut::<c_void>();
pub const NULL_0: *mut c_void = null_mut::<c_void>();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
unsafe fn run_static_initializers() {
    kDefaultKbdControls = [
        0i32 as u16,
        ((if SDLK_UP as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_UP as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_DOWN as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_DOWN as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_LEFT as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_LEFT as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_RIGHT as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_RIGHT as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_RSHIFT as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_RSHIFT as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_RETURN as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_RETURN as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_x as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_x as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_z as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_z as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_s as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_s as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_a as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_a as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_c as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_c as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_v as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_v as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_F1 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F1 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_F2 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F2 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_F3 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F3 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_F4 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F4 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_F5 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F5 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_F6 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F6 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_F7 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F7 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_F8 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F8 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_F9 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F9 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_F10 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F10 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        ((if SDLK_F1 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F1 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_F2 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F2 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_F3 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F3 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_F4 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F4 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_F5 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F5 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_F6 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F6 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_F7 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F7 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_F8 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F8 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_F9 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F9 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_F10 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F10 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        ((if SDLK_F1 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F1 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_F2 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F2 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_F3 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F3 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_F4 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F4 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_F5 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F5 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_F6 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F6 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_F7 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F7 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_F8 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F8 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_F9 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F9 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_F10 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_F10 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        ((if SDLK_1 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_1 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_2 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_2 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_3 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_3 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_4 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_4 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_5 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_5 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_6 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_6 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_7 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_7 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_8 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_8 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_9 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_9 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_0 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_0 as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_MINUS as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_MINUS as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_EQUALS as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_EQUALS as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_BACKSPACE as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_BACKSPACE as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        ((if SDLK_1 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_1 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_2 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_2 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_3 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_3 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_4 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_4 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_5 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_5 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_6 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_6 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_7 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_7 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_8 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_8 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_9 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_9 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_0 as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_0 as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_MINUS as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_MINUS as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_EQUALS as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_EQUALS as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_BACKSPACE as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_BACKSPACE as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        N as u16,
        ((if SDLK_w as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_w as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_q as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_q as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_r as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_r as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_k as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_k as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_l as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_l as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_RETURN as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_RETURN as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Alt as i32) as u16,
        ((if SDLK_r as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_r as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Ctrl as i32) as u16,
        ((if SDLK_p as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_p as i32 & kKeyMod_ScanCode as i32 - 1i32
            | kKeyMod_Shift as i32) as u16,
        ((if SDLK_p as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_p as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_TAB as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_TAB as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_t as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_t as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        N as u16,
        N as u16,
        ((if SDLK_f as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_f as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        ((if SDLK_r as i32 & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as i32
        } else {
            0i32
        }) | SDLK_r as i32 & kKeyMod_ScanCode as i32 - 1i32) as u16,
        0i32 as u16,
        0i32 as u16,
    ];
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe fn(); 1] = [run_static_initializers];
