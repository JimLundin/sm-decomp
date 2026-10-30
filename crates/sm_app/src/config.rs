use crate::sdl_constants;
use ::core::ffi::{c_char, c_void};
use ::core::ptr::null_mut;
pub use sdl2_sys::SDL_GetKeyFromName;
use sm::c_library::strtol;
pub use sm::types::{SDL_Keycode, size_t};
use std::ffi::CStr;
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
pub use sm::types::SDL_Keymod;

sdl_constants! {
    SDL_Keymod as SDL_Keymod;
    KMOD_ALT, KMOD_SHIFT, KMOD_CTRL,
}
pub use sm::types::KeyAction as C2RustUnnamed_0;
pub const kKeys_ToggleRenderer: C2RustUnnamed_0 = 127;
pub const kKeys_ReplayTurbo: C2RustUnnamed_0 = 123;
pub const kKeys_PauseDimmed: C2RustUnnamed_0 = 121;
pub const kKeys_StopReplay: C2RustUnnamed_0 = 117;
pub const kKeys_ClearKeyLog: C2RustUnnamed_0 = 116;
pub const kKeys_ToggleWhichFrame: C2RustUnnamed_0 = 115;
pub const kKeys_CheatJump: C2RustUnnamed_0 = 114;
pub const kKeys_CheatLife: C2RustUnnamed_0 = 113;
pub const kKeys_Load: C2RustUnnamed_0 = 13;
pub use sm::types::OutputMethod as C2RustUnnamed_1;
pub const kOutputMethod_OpenGL: C2RustUnnamed_1 = 2;
pub use sm::types::Config;
pub use sm::types::GamepadButton as C2RustUnnamed_3;
pub const N: i32 = 0;
pub const NULL: *mut c_void = null_mut();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;

#[link(name = "sm_config", kind = "static", modifiers = "-bundle")]
unsafe extern "C-unwind" {
    #[link_name = "sm_config_state"]
    pub static mut g_config: Config;
    fn sm_config_parse_file(owner: *mut c_void, filename: *const c_char);
    fn sm_config_parse_one_file(owner: *mut c_void, filename: *const c_char, depth: i32) -> bool;
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
            sm::util::native_source_matches(),
            "stale native utilities; rerun tools/install_native_config.py and rebuild"
        );
        for (index, source) in [
            include_bytes!("config.rs").as_slice(),
            include_bytes!("../../../sm/src/native_config.c").as_slice(),
            include_bytes!("../../../sm/src/native_config.h").as_slice(),
            include_bytes!("../../../sm/src/native_util.h").as_slice(),
            include_bytes!("../../../sm/src/types.rs").as_slice(),
            include_bytes!("../../../tools/install_native_config.py").as_slice(),
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

// Native views borrow these stable mutable input allocations across includes/reloads.
// Keyboard/gamepad numeric maps have separate native lifetimes.
pub struct ConfigOwner {
    inputs: Vec<Vec<u8>>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl ConfigOwner {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            _thread: std::marker::PhantomData,
        }
    }
}
impl Drop for ConfigOwner {
    fn drop(&mut self) {
        if !self.inputs.is_empty() {
            unsafe {
                g_config.link_graphics = null_mut();
                g_config.memory_buffer = null_mut();
                g_config.shader = null_mut();
                g_config.msu_path = null_mut();
            }
        }
    }
}

/// # Safety
/// All live native configuration views must belong to this owner. Parsing requires
/// exclusive access to native config/maps; all consumers must stop before it drops.
pub unsafe fn ParseConfigFile(filename: *const c_char, owner: &mut ConfigOwner) {
    unsafe {
        check_dependency();
        sm_config_parse_file((owner as *mut ConfigOwner).cast(), filename);
    }
}

// Retain native read/seek/narrowing and mutable sentinel storage in one Rust owner.
// A successful input remains owned even if parsing unwinds.
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_config_read_file(
    owner: *mut c_void,
    name: *const c_char,
) -> *mut c_char {
    unsafe {
        let mut buffer = match sm::util::read_whole_file(name) {
            Ok(buffer) => buffer,
            Err(sm::util::ReadFailure::Open) => return null_mut(),
            Err(sm::util::ReadFailure::Read) => {
                sm::host::Die(c"unable to read the configuration file".as_ptr());
                return null_mut();
            }
        };
        let data = buffer.as_mut_ptr();
        (*owner.cast::<ConfigOwner>()).inputs.push(buffer);
        data.cast()
    }
}

// The temporary owned include path drops after recursion and diagnostics.
// Rust owns this recursive scope, including callbacks unwinding through native C.
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_config_include(
    owner: *mut c_void,
    filename: *const c_char,
    name: *const c_char,
    depth: i32,
) {
    unsafe {
        let path = sm::util::include_path(CStr::from_ptr(filename), CStr::from_ptr(name));
        if depth > 10 || !sm_config_parse_one_file(owner, path.as_ptr(), depth + 1) {
            sm::c_library::fprintf(
                sm::c_library::stderr,
                c"Warning: Unable to read %s\n".as_ptr(),
                path.as_ptr(),
            );
        }
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
unsafe extern "C-unwind" fn sm_config_stderr() -> *mut sm::types::FILE {
    unsafe { sm::c_library::stderr }
}
