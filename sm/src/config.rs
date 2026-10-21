use crate::c_library::{fprintf, free, realloc, stderr, strtol};
use crate::host::Die;
use crate::sdl_constants;
use crate::set_fields;
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
crate::type_aliases! {
    pub C2RustUnnamed, C2RustUnnamed_2, C2RustUnnamed_4 = u32;
}
sdl_constants! {
    SDL_KeyCode as C2RustUnnamed;
    SDLK_RALT, SDLK_RSHIFT, SDLK_RCTRL, SDLK_LALT, SDLK_LSHIFT, SDLK_LCTRL, SDLK_UP,
    SDLK_DOWN, SDLK_LEFT, SDLK_RIGHT, SDLK_F10, SDLK_F9, SDLK_F8, SDLK_F7, SDLK_F6, SDLK_F5,
    SDLK_F4, SDLK_F3, SDLK_F2, SDLK_F1, SDLK_z, SDLK_x, SDLK_w, SDLK_v, SDLK_t, SDLK_s,
    SDLK_r, SDLK_q, SDLK_p, SDLK_l, SDLK_k, SDLK_f, SDLK_c, SDLK_a, SDLK_EQUALS, SDLK_9,
    SDLK_8, SDLK_7, SDLK_6, SDLK_5, SDLK_4, SDLK_3, SDLK_2, SDLK_1, SDLK_0, SDLK_MINUS,
    SDLK_TAB, SDLK_BACKSPACE, SDLK_RETURN, SDLK_UNKNOWN,
}
pub use crate::types::SDL_Keymod;

sdl_constants! {
    SDL_Keymod as SDL_Keymod;
    KMOD_ALT, KMOD_SHIFT, KMOD_CTRL,
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
crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C)]
    ];
    pub struct GamepadMapEnt {
        pub modifiers: u32;
        pub cmd, next: u16;
    }

    pub struct KeyNameId {
        pub name: *const c_char;
        pub id, size: u16;
    }

    pub struct KeyMapHashEnt {
        pub key, cmd, next: u16;
    }
}

pub const kKeyMod_ScanCode: C2RustUnnamed_4 = 512;
pub const kKeyMod_Shift: C2RustUnnamed_4 = 2048;
pub const kKeyMod_Ctrl: C2RustUnnamed_4 = 4096;
pub const kKeyMod_Alt: C2RustUnnamed_4 = 1024;

#[inline]
unsafe fn atoi(mut __nptr: *const c_char) -> i32 {
    strtol(__nptr, NULL as *mut *mut c_char, 10) as i32
}
pub const SDLK_SCANCODE_MASK: i32 = (1i32) << 30;
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
const fn key_name(name: &'static core::ffi::CStr, id: u32, size: u16) -> KeyNameId {
    KeyNameId {
        name: name.as_ptr(),
        id: id as u16,
        size,
    }
}
static mut kKeyNameId: [KeyNameId; 24] = [
    key_name(c"Null", kKeys_Null, 65535),
    key_name(
        c"Controls",
        kKeys_Controls,
        (kKeys_Controls_Last as i32 - kKeys_Controls as i32 + 1) as u16,
    ),
    key_name(
        c"Load",
        kKeys_Load,
        (kKeys_Load_Last as i32 - kKeys_Load as i32 + 1) as u16,
    ),
    key_name(
        c"Save",
        kKeys_Save,
        (kKeys_Save_Last as i32 - kKeys_Save as i32 + 1) as u16,
    ),
    key_name(
        c"Replay",
        kKeys_Replay,
        (kKeys_Replay_Last as i32 - kKeys_Replay as i32 + 1) as u16,
    ),
    key_name(
        c"LoadRef",
        kKeys_LoadRef,
        (kKeys_LoadRef_Last as i32 - kKeys_LoadRef as i32 + 1) as u16,
    ),
    key_name(
        c"ReplayRef",
        kKeys_ReplayRef,
        (kKeys_ReplayRef_Last as i32 - kKeys_ReplayRef as i32 + 1) as u16,
    ),
    key_name(c"CheatLife", kKeys_CheatLife, 1),
    key_name(c"CheatJump", kKeys_CheatJump, 1),
    key_name(c"ToggleWhichFrame", kKeys_ToggleWhichFrame, 1),
    key_name(c"ClearKeyLog", kKeys_ClearKeyLog, 1),
    key_name(c"StopReplay", kKeys_StopReplay, 1),
    key_name(c"Fullscreen", kKeys_Fullscreen, 1),
    key_name(c"Reset", kKeys_Reset, 1),
    key_name(c"Pause", kKeys_Pause, 1),
    key_name(c"PauseDimmed", kKeys_PauseDimmed, 1),
    key_name(c"Turbo", kKeys_Turbo, 1),
    key_name(c"ReplayTurbo", kKeys_ReplayTurbo, 1),
    key_name(c"WindowBigger", kKeys_WindowBigger, 1),
    key_name(c"WindowSmaller", kKeys_WindowSmaller, 1),
    key_name(c"VolumeUp", kKeys_VolumeUp, 1),
    key_name(c"VolumeDown", kKeys_VolumeDown, 1),
    key_name(c"DisplayPerf", kKeys_DisplayPerf, 1),
    key_name(c"ToggleRenderer", kKeys_ToggleRenderer, 1),
];
static mut keymap_hash_first: [u16; 255] = [0; 255];
static mut keymap_hash: *mut KeyMapHashEnt = null::<KeyMapHashEnt>() as *mut KeyMapHashEnt;
static mut keymap_hash_size: i32 = 0;
static mut has_keynameid: [bool; 24] = [false; 24];
unsafe fn KeyMapHash_Add(key: u16, cmd: u16) -> bool {
    if keymap_hash_size & 0xff == 0 {
        if keymap_hash_size > 10000 {
            Die(c"Too many keys".as_ptr());
        }
        keymap_hash = realloc(
            keymap_hash as *mut c_void,
            (size_of::<KeyMapHashEnt>() as size_t).wrapping_mul((keymap_hash_size + 256) as size_t),
        ) as *mut KeyMapHashEnt;
    }
    crate::update_place! { previous fresh1 = keymap_hash_size => keymap_hash_size + 1; }
    let i: i32 = fresh1;
    let ent = keymap_hash.offset(i as isize);
    (*ent).key = key;
    (*ent).cmd = cmd;
    (*ent).next = 0;
    let j: i32 = (key as u32).wrapping_rem(255) as i32;
    let mut cur = (&raw mut keymap_hash_first as *mut u16).offset(j as isize);
    while *cur != 0 {
        let ent_0 = keymap_hash.offset((*cur as i32 - 1i32) as isize);
        if (*ent_0).key as i32 == key as i32 {
            return false_0 != 0;
        }
        cur = &raw mut (*ent_0).next;
    }
    *cur = (i + 1) as u16;
    true_0 != 0
}
unsafe fn KeyMapHash_Find(key: u16) -> i32 {
    let mut i: i32 = keymap_hash_first[(key as i32 % 255) as usize] as i32;
    while i != 0 {
        let ent = keymap_hash.offset((i - 1i32) as isize);
        if (*ent).key as i32 == key as i32 {
            return (*ent).cmd as i32;
        }
        i = (*ent).next as i32;
    }
    0
}
// Pack SDL's low nine key bits and scancode marker into the key-map format.
// Modifier bits and each caller's validation remain separate.
const fn encode_keycode(code: SDL_Keycode) -> i32 {
    let scancode = if code & SDLK_SCANCODE_MASK != 0 {
        kKeyMod_ScanCode as i32
    } else {
        0
    };
    scancode | (code & (kKeyMod_ScanCode as i32 - 1i32))
}

pub unsafe fn FindCmdForSdlKey(code: SDL_Keycode, mod_0: SDL_Keymod) -> i32 {
    if code & !(SDLK_SCANCODE_MASK as SDL_Keycode | 0x1ff as SDL_Keycode) != 0 {
        return 0;
    }
    let mut key: i32 = 0;
    if code != SDLK_LALT as i32 as SDL_Keycode && code != SDLK_RALT as i32 as SDL_Keycode {
        key |= if mod_0 & KMOD_ALT as i32 as u32 != 0 {
            kKeyMod_Alt as i32
        } else {
            0
        };
    }
    if code != SDLK_LCTRL as i32 as SDL_Keycode && code != SDLK_RCTRL as i32 as SDL_Keycode {
        key |= if mod_0 & KMOD_CTRL as i32 as u32 != 0 {
            kKeyMod_Ctrl as i32
        } else {
            0
        };
    }
    if code != SDLK_LSHIFT as i32 as SDL_Keycode && code != SDLK_RSHIFT as i32 as SDL_Keycode {
        key |= if mod_0 & KMOD_SHIFT as i32 as u32 != 0 {
            kKeyMod_Shift as i32
        } else {
            0
        };
    }
    key |= encode_keycode(code);
    KeyMapHash_Find(key as u16)
}
unsafe fn ParseKeyArray(mut value: *mut c_char, mut cmd: i32, size: i32) {
    let mut s = null_mut::<c_char>();
    let mut i: i32 = 0;
    while i < size && {
        s = NextDelim(&raw mut value, ',' as i32);
        !s.is_null()
    } {
        if (*s as i32 != 0) {
            let mut key_with_mod: i32 = 0;
            loop {
                if !StringStartsWithNoCase(s, c"Shift+".as_ptr()).is_null() {
                    key_with_mod |= kKeyMod_Shift as i32;
                    s = s.offset(6);
                } else if !StringStartsWithNoCase(s, c"Ctrl+".as_ptr()).is_null() {
                    key_with_mod |= kKeyMod_Ctrl as i32;
                    s = s.offset(5);
                } else {
                    if StringStartsWithNoCase(s, c"Alt+".as_ptr()).is_null() {
                        break;
                    }
                    key_with_mod |= kKeyMod_Alt as i32;
                    s = s.offset(4);
                }
            }
            let key: SDL_Keycode = SDL_GetKeyFromName(s);
            if key == SDLK_UNKNOWN as i32 as SDL_Keycode {
                fprintf(stderr, c"Unknown key: '%s'\n".as_ptr(), s);
            } else if !KeyMapHash_Add(
                (key_with_mod as SDL_Keycode | encode_keycode(key)) as u16,
                cmd as u16,
            ) {
                fprintf(stderr, c"Duplicate key: '%s'\n".as_ptr(), s);
            }
        }
        i += 1;
        cmd += (cmd != 0) as i32;
    }
}
static mut joymap_first: [u16; 17] = [0; 17];
static mut joymap_ents: *mut GamepadMapEnt = null::<GamepadMapEnt>() as *mut GamepadMapEnt;
static mut joymap_size: i32 = 0;
static mut has_joypad_controls: bool = false;
unsafe fn CountBits32(mut n: u32) -> i32 {
    let mut count: i32 = 0;
    while n != 0 {
        n &= n.wrapping_sub(1);
        count += 1;
    }
    count
}
unsafe fn GamepadMap_Add(button: i32, modifiers: u32, cmd: u16) {
    if joymap_size & 0xff == 0 {
        if joymap_size > 1000 {
            Die(c"Too many joypad keys".as_ptr());
        }
        joymap_ents = realloc(
            joymap_ents as *mut c_void,
            (size_of::<GamepadMapEnt>() as size_t).wrapping_mul((joymap_size + 64) as size_t),
        ) as *mut GamepadMapEnt;
        if joymap_ents.is_null() {
            Die(c"realloc failure".as_ptr());
        }
    }
    let mut p = (&raw mut joymap_first as *mut u16).offset(button as isize);
    let cb: i32 = CountBits32(modifiers);
    while *p as i32 != 0
        && cb < CountBits32((*joymap_ents.offset((*p as i32 - 1i32) as isize)).modifiers)
    {
        p = &raw mut (*joymap_ents.offset((*p as i32 - 1i32) as isize)).next;
    }
    crate::update_place! { previous fresh0 = joymap_size => joymap_size + 1; }
    let i: i32 = fresh0;
    let ent = joymap_ents.offset(i as isize);
    (*ent).modifiers = modifiers;
    (*ent).cmd = cmd;
    (*ent).next = *p;
    *p = (i + 1) as u16;
}
pub unsafe fn FindCmdForGamepadButton(button: i32, modifiers: u32) -> i32 {
    let mut e: i32 = joymap_first[button as usize] as i32;
    while e != 0 {
        let mut ent = joymap_ents.offset((e - 1i32) as isize);
        if modifiers & (*ent).modifiers == (*ent).modifiers {
            return (*ent).cmd as i32;
        }
        e = (*ent).next as i32;
    }
    0
}
unsafe fn ParseGamepadButtonName(value: *mut *const c_char) -> i32 {
    const BUTTONS: &[(&core::ffi::CStr, i32)] = &[
        (c"Back", kGamepadBtn_Back),
        (c"Guide", kGamepadBtn_Guide),
        (c"Start", kGamepadBtn_Start),
        (c"L3", kGamepadBtn_L3),
        (c"R3", kGamepadBtn_R3),
        (c"L1", kGamepadBtn_L1),
        (c"R1", kGamepadBtn_R1),
        (c"DpadUp", kGamepadBtn_DpadUp),
        (c"DpadDown", kGamepadBtn_DpadDown),
        (c"DpadLeft", kGamepadBtn_DpadLeft),
        (c"DpadRight", kGamepadBtn_DpadRight),
        (c"L2", kGamepadBtn_L2),
        (c"R2", kGamepadBtn_R2),
        (c"Lb", kGamepadBtn_L1),
        (c"Rb", kGamepadBtn_R1),
        (c"A", kGamepadBtn_A),
        (c"B", kGamepadBtn_B),
        (c"X", kGamepadBtn_X),
        (c"Y", kGamepadBtn_Y),
    ];
    for &(name, button) in BUTTONS {
        let rest = StringStartsWithNoCase(*value, name.as_ptr());
        if !rest.is_null() {
            *value = rest;
            return button;
        }
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
    let mut s = null_mut::<c_char>();
    let mut i: i32 = 0;
    while i < size && {
        s = NextDelim(&raw mut value, ',' as i32);
        !s.is_null()
    } {
        if (*s as i32 != 0) {
            let mut modifiers: u32 = 0;
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
                    } else if *ss as i32 == 0 {
                        GamepadMap_Add(button, modifiers, cmd as u16);
                        break;
                    }
                }
                fprintf(stderr, c"Unknown gamepad button: '%s'\n".as_ptr(), s);
                break;
            }
        }
        i += 1;
        cmd += (cmd != 0) as i32;
    }
}
unsafe fn RegisterDefaultKeys() {
    let mut i: i32 = 1;
    while (i as usize)
        < (size_of::<[KeyNameId; 24]>() as usize).wrapping_div(size_of::<KeyNameId>() as usize)
    {
        if !has_keynameid[i as usize] {
            let size: i32 = kKeyNameId[i as usize].size as i32;
            let mut k: i32 = kKeyNameId[i as usize].id as i32;
            let mut j: i32 = 0;
            while j < size {
                KeyMapHash_Add(kDefaultKbdControls[k as usize], k as u16);
                j += 1;
                k += 1;
            }
        }
        i += 1;
    }
    if !has_joypad_controls {
        let mut i_0: i32 = 0;
        while (i_0 as usize)
            < (size_of::<[u8; 12]>() as usize).wrapping_div(size_of::<u8>() as usize)
        {
            GamepadMap_Add(
                kDefaultGamepadCmds[i_0 as usize] as i32,
                0,
                (kKeys_Controls as i32 + i_0) as u16,
            );
            i_0 += 1;
        }
    }
}
unsafe fn GetIniSection(s: *const c_char) -> i32 {
    [
        c"[KeyMap]",
        c"[Graphics]",
        c"[Sound]",
        c"[General]",
        c"[Features]",
        c"[GamepadMap]",
    ]
    .iter()
    .position(|name| StringEqualsNoCase(s, name.as_ptr()))
    .map_or(-1, |index| index as i32)
}
pub unsafe fn ParseBool(value: *const c_char, result: *mut bool) -> bool {
    // The original first-byte OR also accepts control bytes $10/$11 as 0/1.
    let suffix = value.add(1);
    let (parsed, expected) = match *value as i32 | 32 {
        48 => (false, c""),
        49 => (true, c""),
        102 => (false, c"alse"),
        110 => (false, c"o"),
        111 => {
            let on = *suffix as i32 | 32 == 'n' as i32;
            (on, if on { c"n" } else { c"ff" })
        }
        121 => (true, c"es"),
        116 => (true, c"rue"),
        _ => return false,
    };
    if !StringEqualsNoCase(suffix, expected.as_ptr()) {
        return false;
    }
    if result.is_null() {
        parsed
    } else {
        *result = parsed;
        true
    }
}
unsafe fn HandleIniConfig(section: i32, key: *const c_char, mut value: *mut c_char) -> bool {
    if section == 0 || section == 5 {
        for i in 0..24 {
            if StringEqualsNoCase(key, kKeyNameId[i].name) {
                if section == 0 {
                    has_keynameid[i] = true;
                    ParseKeyArray(value, kKeyNameId[i].id as i32, kKeyNameId[i].size as i32);
                } else {
                    if i == 1 {
                        has_joypad_controls = true;
                    }
                    ParseGamepadArray(value, kKeyNameId[i].id as i32, kKeyNameId[i].size as i32);
                }
                return true;
            }
        }
    } else if section == 1 {
        if StringEqualsNoCase(key, c"WindowSize".as_ptr()) {
            let mut s = null_mut::<c_char>();
            if StringEqualsNoCase(value, c"Auto".as_ptr()) {
                set_fields! {[g_config], 0; window_width, window_height}
                return true;
            }
            loop {
                s = NextDelim(&raw mut value, 'x' as i32);
                if s.is_null() {
                    break;
                }
                if g_config.window_width == 0 {
                    g_config.window_width = atoi(s);
                } else {
                    g_config.window_height = atoi(s);
                    return true;
                }
            }
        } else if StringEqualsNoCase(key, c"EnhancedMode7".as_ptr()) {
            return ParseBool(value, &raw mut g_config.enhanced_mode7);
        } else if StringEqualsNoCase(key, c"NewRenderer".as_ptr()) {
            return ParseBool(value, &raw mut g_config.new_renderer);
        } else if StringEqualsNoCase(key, c"IgnoreAspectRatio".as_ptr()) {
            return ParseBool(value, &raw mut g_config.ignore_aspect_ratio);
        } else if StringEqualsNoCase(key, c"Fullscreen".as_ptr()) {
            g_config.fullscreen = strtol(value, NULL_0 as *mut *mut c_char, 10) as u8;
            return true;
        } else if StringEqualsNoCase(key, c"WindowScale".as_ptr()) {
            g_config.window_scale = strtol(value, NULL_0 as *mut *mut c_char, 10) as u8;
            return true;
        } else if StringEqualsNoCase(key, c"OutputMethod".as_ptr()) {
            g_config.output_method =
                (if StringEqualsNoCase(value, c"SDL-Software".as_ptr()) as i32 != 0 {
                    kOutputMethod_SDLSoftware as i32
                } else if StringEqualsNoCase(value, c"OpenGL".as_ptr()) as i32 != 0 {
                    kOutputMethod_OpenGL as i32
                } else {
                    kOutputMethod_SDL as i32
                }) as u8;
            return true;
        } else if StringEqualsNoCase(key, c"LinearFiltering".as_ptr()) {
            return ParseBool(value, &raw mut g_config.linear_filtering);
        } else if StringEqualsNoCase(key, c"NoSpriteLimits".as_ptr()) {
            return ParseBool(value, &raw mut g_config.no_sprite_limits);
        } else if StringEqualsNoCase(key, c"LinkGraphics".as_ptr()) {
            g_config.link_graphics = value;
            return true;
        } else if StringEqualsNoCase(key, c"Shader".as_ptr()) {
            g_config.shader = if *value as i32 != 0 {
                value
            } else {
                null_mut::<c_char>()
            };
            return true;
        }
    } else if section == 2 {
        if StringEqualsNoCase(key, c"EnableAudio".as_ptr()) {
            return ParseBool(value, &raw mut g_config.enable_audio);
        } else if StringEqualsNoCase(key, c"AudioFreq".as_ptr()) {
            g_config.audio_freq = strtol(value, NULL_0 as *mut *mut c_char, 10) as u16;
            return true;
        } else if StringEqualsNoCase(key, c"AudioChannels".as_ptr()) {
            g_config.audio_channels = strtol(value, NULL_0 as *mut *mut c_char, 10) as u8;
            return true;
        } else if StringEqualsNoCase(key, c"AudioSamples".as_ptr()) {
            g_config.audio_samples = strtol(value, NULL_0 as *mut *mut c_char, 10) as u16;
            return true;
        } else if StringEqualsNoCase(key, c"EnableMSU".as_ptr()) {
            if StringEqualsNoCase(value, c"opuz".as_ptr()) {
                g_config.enable_msu = kMsuEnabled_Opuz as i32 as u8;
            } else if StringEqualsNoCase(value, c"deluxe".as_ptr()) {
                g_config.enable_msu = kMsuEnabled_MsuDeluxe as i32 as u8;
            } else if StringEqualsNoCase(value, c"deluxe-opuz".as_ptr()) {
                g_config.enable_msu =
                    (kMsuEnabled_MsuDeluxe as i32 | kMsuEnabled_Opuz as i32) as u8;
            } else {
                return ParseBool(value, &raw mut g_config.enable_msu as *mut bool);
            }
            return true;
        } else if StringEqualsNoCase(key, c"MSUPath".as_ptr()) {
            g_config.msu_path = value;
            return true;
        } else if StringEqualsNoCase(key, c"MSUVolume".as_ptr()) {
            g_config.msuvolume = atoi(value) as u8;
            return true;
        } else if StringEqualsNoCase(key, c"ResumeMSU".as_ptr()) {
            return ParseBool(value, &raw mut g_config.resume_msu);
        }
    } else if section == 3 {
        if StringEqualsNoCase(key, c"Autosave".as_ptr()) {
            g_config.autosave = strtol(value, NULL_0 as *mut *mut c_char, 10) != 0;
            return true;
        } else if StringEqualsNoCase(key, c"DisplayPerfInTitle".as_ptr()) {
            return ParseBool(value, &raw mut g_config.display_perf_title);
        } else if StringEqualsNoCase(key, c"DisableFrameDelay".as_ptr()) {
            return ParseBool(value, &raw mut g_config.disable_frame_delay);
        }
    } else {
        section == 4;
    }
    false
}
unsafe fn ParseOneConfigFile(filename: *const c_char, depth: i32) -> bool {
    let mut filedata = ReadWholeFile(filename, null_mut::<size_t>()) as *mut c_char;
    let mut p = null_mut::<c_char>();
    if filedata.is_null() {
        return false_0 != 0;
    }
    let mut section: i32 = -2i32;
    g_config.memory_buffer = filedata;
    let mut lineno: i32 = 1;
    loop {
        p = NextLineStripComments(&raw mut filedata);
        if p.is_null() {
            break;
        }
        if (*p as i32 != 0) {
            if *p as i32 == '[' as i32 {
                section = GetIniSection(p);
                if section < 0 {
                    fprintf(
                        stderr,
                        c"%s:%d: Invalid .ini section %s\n".as_ptr(),
                        filename,
                        lineno,
                        p,
                    );
                }
            } else if *p as i32 == '!' as i32
                && !SkipPrefix(p.offset(1), c"include ".as_ptr()).is_null()
            {
                let mut tt = p.offset(8);
                let new_filename =
                    ReplaceFilenameWithNewPath(filename, NextPossiblyQuotedString(&raw mut tt));
                if depth > 10 || !ParseOneConfigFile(new_filename, depth + 1) {
                    fprintf(
                        stderr,
                        c"Warning: Unable to read %s\n".as_ptr(),
                        new_filename,
                    );
                }
                free(new_filename as *mut c_void);
            } else if section == -2i32 {
                fprintf(
                    stderr,
                    c"%s:%d: Expecting [section]\n".as_ptr(),
                    filename,
                    lineno,
                );
            } else {
                let v = SplitKeyValue(p);
                if v.is_null() {
                    fprintf(
                        stderr,
                        c"%s:%d: Expecting 'key=value'\n".as_ptr(),
                        filename,
                        lineno,
                    );
                } else if section >= 0 && !HandleIniConfig(section, p, v) {
                    fprintf(
                        stderr,
                        c"%s:%d: Can't parse '%s'\n".as_ptr(),
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
    g_config.msuvolume = 100;
    if !filename.is_null() || !ParseOneConfigFile(c"sm.user.ini".as_ptr(), 0) {
        if filename.is_null() {
            filename = c"sm.ini".as_ptr();
        }
        if !ParseOneConfigFile(filename, 0) {
            fprintf(
                stderr,
                c"Warning: Unable to read config file %s\n".as_ptr(),
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
    let mut bindings = [N as u16; 130];
    for (i, key) in [
        SDLK_UP,
        SDLK_DOWN,
        SDLK_LEFT,
        SDLK_RIGHT,
        SDLK_RSHIFT,
        SDLK_RETURN,
        SDLK_x,
        SDLK_z,
        SDLK_s,
        SDLK_a,
        SDLK_c,
        SDLK_v,
    ]
    .into_iter()
    .enumerate()
    {
        bindings[kKeys_Controls as usize + i] = encode_keycode(key as SDL_Keycode) as u16;
    }
    // Each save-state command group has twenty slots; only F1..F10 are bound.
    for (command, modifier) in [
        (kKeys_Load, 0),
        (kKeys_Save, kKeyMod_Shift),
        (kKeys_Replay, kKeyMod_Ctrl),
    ] {
        for (i, key) in [
            SDLK_F1, SDLK_F2, SDLK_F3, SDLK_F4, SDLK_F5, SDLK_F6, SDLK_F7, SDLK_F8, SDLK_F9,
            SDLK_F10,
        ]
        .into_iter()
        .enumerate()
        {
            bindings[command as usize + i] =
                (encode_keycode(key as SDL_Keycode) | modifier as i32) as u16;
        }
    }
    for (command, modifier) in [(kKeys_LoadRef, 0), (kKeys_ReplayRef, kKeyMod_Ctrl)] {
        for (i, key) in [
            SDLK_1,
            SDLK_2,
            SDLK_3,
            SDLK_4,
            SDLK_5,
            SDLK_6,
            SDLK_7,
            SDLK_8,
            SDLK_9,
            SDLK_0,
            SDLK_MINUS,
            SDLK_EQUALS,
            SDLK_BACKSPACE,
        ]
        .into_iter()
        .enumerate()
        {
            bindings[command as usize + i] =
                (encode_keycode(key as SDL_Keycode) | modifier as i32) as u16;
        }
    }
    for (command, key, modifier) in [
        (kKeys_CheatLife, SDLK_w, 0),
        (kKeys_CheatJump, SDLK_q, 0),
        (kKeys_ToggleWhichFrame, SDLK_r, kKeyMod_Shift),
        (kKeys_ClearKeyLog, SDLK_k, 0),
        (kKeys_StopReplay, SDLK_l, 0),
        (kKeys_Fullscreen, SDLK_RETURN, kKeyMod_Alt),
        (kKeys_Reset, SDLK_r, kKeyMod_Ctrl),
        (kKeys_Pause, SDLK_p, kKeyMod_Shift),
        (kKeys_PauseDimmed, SDLK_p, 0),
        (kKeys_Turbo, SDLK_TAB, 0),
        (kKeys_ReplayTurbo, SDLK_t, 0),
        (kKeys_DisplayPerf, SDLK_f, 0),
        (kKeys_ToggleRenderer, SDLK_r, 0),
    ] {
        bindings[command as usize] = (encode_keycode(key as SDL_Keycode) | modifier as i32) as u16;
    }
    kDefaultKbdControls = bindings;
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe fn(); 1] = [run_static_initializers];
