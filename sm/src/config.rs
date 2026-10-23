use crate::c_library::strtol;
use crate::sdl_constants;
pub use crate::types::{SDL_Keycode, size_t};
use ::core::ffi::{c_char, c_void};
use ::core::ptr::null_mut;
pub use sdl2_sys::SDL_GetKeyFromName;
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

pub const SDLK_SCANCODE_MASK: i32 = 1 << 30;
pub const N: i32 = 0;
pub const NULL: *mut c_void = null_mut();
pub const NULL_0: *mut c_void = null_mut();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;

#[link(name = "sm_config", kind = "static", modifiers = "-bundle")]
unsafe extern "C-unwind" {
    #[link_name = "sm_config_state"]
    pub static mut g_config: Config;
    fn sm_config_parse_file(filename: *const c_char);
    fn sm_config_find_key(code: SDL_Keycode, modifiers: SDL_Keymod) -> i32;
    fn sm_config_find_button(button: i32, modifiers: u32) -> i32;
    fn sm_config_parse_bool(value: *const c_char, result: *mut bool) -> bool;
    fn sm_config_matches_source(index: u32, bytes: *const u8, length: usize) -> bool;
}

// Validate the archive at its actual frontend consumer. Headless replay never
// links this module; every public entry also works before ParseConfigFile.
pub(crate) fn check_dependency() {
    static CHECKED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    CHECKED.get_or_init(|| {
        assert!(
            crate::util::native_source_matches(),
            "stale native utilities; rerun tools/install_native_config.py and rebuild"
        );
        for (index, source) in [
            include_bytes!("config.rs").as_slice(),
            include_bytes!("native_config.c").as_slice(),
            include_bytes!("native_config.h").as_slice(),
            include_bytes!("native_util.h").as_slice(),
            include_bytes!("types.rs").as_slice(),
            include_bytes!("../../tools/install_native_config.py").as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                unsafe { sm_config_matches_source(index as u32, source.as_ptr(), source.len()) },
                "stale native configuration; rerun tools/install_native_config.py and rebuild"
            );
        }
    });
}

pub unsafe fn ParseConfigFile(filename: *const c_char) {
    unsafe {
        check_dependency();
        sm_config_parse_file(filename);
    }
}

pub unsafe fn FindCmdForSdlKey(code: SDL_Keycode, modifiers: SDL_Keymod) -> i32 {
    unsafe {
        check_dependency();
        sm_config_find_key(code, modifiers)
    }
}

pub unsafe fn FindCmdForGamepadButton(button: i32, modifiers: u32) -> i32 {
    unsafe {
        check_dependency();
        // The native public lookup bounds-checks before consulting its linked list.
        let _ = [0u16; 17][button as usize];
        sm_config_find_button(button, modifiers)
    }
}

pub unsafe fn ParseBool(value: *const c_char, result: *mut bool) -> bool {
    unsafe {
        check_dependency();
        sm_config_parse_bool(value, result)
    }
}

// Preserve explicit numeric conversion and the frontend FILE pointer.
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_config_atoi(value: *const c_char) -> i32 {
    unsafe { strtol(value, null_mut(), 10) as i32 }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_config_stderr() -> *mut crate::types::FILE {
    unsafe { crate::c_library::stderr }
}
