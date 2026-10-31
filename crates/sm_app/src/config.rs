use ::core::ffi::{c_char, c_void};
use ::core::ptr::null_mut;
use sm::c_library::strtol;
pub use sm::types::{Config, SdlKeycode, SdlKeymod};
use std::ffi::CStr;

#[link(name = "sm_config", kind = "static", modifiers = "-bundle")]
unsafe extern "C-unwind" {
    #[link_name = "sm_config_state"]
    pub static mut CONFIG: Config;
    fn sm_config_parse_file(owner: *mut c_void, filename: *const c_char);
    fn sm_config_parse_one_file(owner: *mut c_void, filename: *const c_char, depth: i32) -> bool;
    fn sm_config_find_key(code: SdlKeycode, modifiers: SdlKeymod) -> i32;
    fn sm_config_find_button(button: i32, modifiers: u32) -> i32;
    fn sm_config_matches_source(index: u32, bytes: *const u8, length: usize) -> bool;
}

// Validate the archive at its actual frontend consumer. Headless replay never
// links this module; every public entry also works before parse_config_file.
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
                CONFIG.link_graphics = null_mut();
                CONFIG.memory_buffer = null_mut();
                CONFIG.shader = null_mut();
                CONFIG.msu_path = null_mut();
            }
        }
    }
}

/// # Safety
/// All live native configuration views must belong to this owner. Parsing requires
/// exclusive access to native config/maps; all consumers must stop before it drops.
pub unsafe fn parse_config_file(filename: *const c_char, owner: &mut ConfigOwner) {
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
                sm::host::fatal_error(c"unable to read the configuration file".as_ptr());
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

pub unsafe fn find_cmd_for_sdl_key(code: SdlKeycode, modifiers: SdlKeymod) -> i32 {
    unsafe {
        check_dependency();
        sm_config_find_key(code, modifiers)
    }
}

pub unsafe fn find_cmd_for_gamepad_button(button: i32, modifiers: u32) -> i32 {
    unsafe {
        check_dependency();
        // The native public lookup bounds-checks before consulting its linked list.
        let _ = [0u16; 17][button as usize];
        sm_config_find_button(button, modifiers)
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
