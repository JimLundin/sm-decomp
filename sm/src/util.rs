//! File and lexical helpers from the pinned native utility dependency.
pub use crate::types::{FILE, size_t};
use core::ffi::{c_char, c_void};

pub type __builtin_va_list = [__va_list_tag; 1];
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct __va_list_tag {
        pub gp_offset, fp_offset: u32;
        pub overflow_arg_area, reg_save_area: *mut c_void;
    }
}
pub type va_list = __gnuc_va_list;
pub type __gnuc_va_list = __builtin_va_list;
pub const NULL: *mut c_void = core::ptr::null_mut::<c_void>();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
pub const SEEK_END: i32 = 2;

// ROM and configuration readers close the actual CRT stream on return or unwind.
pub(crate) struct InputFile(pub(crate) *mut FILE);
impl Drop for InputFile {
    fn drop(&mut self) {
        unsafe {
            crate::c_library::fclose(self.0);
        }
    }
}

#[link(name = "sm_config", kind = "static", modifiers = "-bundle")]
unsafe extern "C-unwind" {
    #[link_name = "sm_util_NextDelim"]
    pub fn NextDelim(cursor: *mut *mut c_char, separator: i32) -> *mut c_char;
    #[link_name = "sm_util_StringEqualsNoCase"]
    pub fn StringEqualsNoCase(value: *const c_char, expected: *const c_char) -> bool;
    #[link_name = "sm_util_StringStartsWithNoCase"]
    pub fn StringStartsWithNoCase(value: *const c_char, prefix: *const c_char) -> *const c_char;
    #[link_name = "sm_util_ReadWholeFile"]
    pub fn ReadWholeFile(name: *const c_char, length: *mut size_t) -> *mut u8;
    #[link_name = "sm_util_NextLineStripComments"]
    pub fn NextLineStripComments(cursor: *mut *mut c_char) -> *mut c_char;
    #[link_name = "sm_util_NextPossiblyQuotedString"]
    pub fn NextPossiblyQuotedString(cursor: *mut *mut c_char) -> *mut c_char;
    #[link_name = "sm_util_ReplaceFilenameWithNewPath"]
    pub fn ReplaceFilenameWithNewPath(base: *const c_char, name: *const c_char) -> *mut c_char;
    #[link_name = "sm_util_SplitKeyValue"]
    pub fn SplitKeyValue(value: *mut c_char) -> *mut c_char;
    #[link_name = "sm_util_SkipPrefix"]
    pub fn SkipPrefix(value: *const c_char, prefix: *const c_char) -> *const c_char;
    #[link_name = "sm_util_StrSet"]
    pub fn StrSet(target: *mut *mut c_char, value: *const c_char);
    fn sm_util_matches_source(index: u32, bytes: *const u8, length: usize) -> bool;
}

// The installed host callback may return or unwind. Do not declare it noreturn.
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_util_die(message: *const c_char) {
    unsafe {
        crate::host::Die(message);
    }
}

#[doc(hidden)]
pub fn native_source_matches() -> bool {
    [
        include_bytes!("util.rs").as_slice(),
        include_bytes!("native_util.c").as_slice(),
        include_bytes!("native_util.h").as_slice(),
        include_bytes!("types.rs").as_slice(),
        include_bytes!("../../tools/install_native_config.py").as_slice(),
    ]
    .into_iter()
    .enumerate()
    .all(|(index, source)| unsafe {
        sm_util_matches_source(index as u32, source.as_ptr(), source.len())
    })
}
