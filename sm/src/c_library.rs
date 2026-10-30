//! Native C-library boundary. Keep the original ABI types and symbols here.

use crate::types::{FILE, size_t, time_t};
use ::core::ffi::{c_char, c_double, c_long, c_void};

/// The original C assertion failure for NUL-terminated literal diagnostics.
pub fn assertion_failed(
    assertion: &'static [u8],
    file: &'static [u8],
    line: u32,
    function: &'static [u8],
) -> ! {
    assert!(
        [assertion, file, function]
            .iter()
            .all(|text| text.last() == Some(&0)),
        "assertion diagnostics must be NUL-terminated"
    );
    // SAFETY: each pointer is a NUL-terminated static byte string.
    unsafe {
        __assert_fail(
            assertion.as_ptr().cast(),
            file.as_ptr().cast(),
            line,
            function.as_ptr().cast(),
        )
    }
}

unsafe extern "C" {
    pub fn __assert_fail(
        __assertion: *const c_char,
        __file: *const c_char,
        __line: u32,
        __function: *const c_char,
    ) -> !;
    pub fn abs(__x: i32) -> i32;
    pub fn exit(__status: i32) -> !;
    pub fn fprintf(__stream: *mut FILE, __format: *const c_char, ...) -> i32;
    pub fn free(__ptr: *mut c_void);
    pub fn getcwd(__buf: *mut c_char, __size: size_t) -> *mut c_char;
    pub fn memcmp(__s1: *const c_void, __s2: *const c_void, __n: size_t) -> i32;
    pub fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: size_t) -> *mut c_void;
    pub fn memset(__s: *mut c_void, __c: i32, __n: size_t) -> *mut c_void;
    pub fn printf(__format: *const c_char, ...) -> i32;
    pub fn puts(__s: *const c_char) -> i32;
    pub fn rename(__old: *const c_char, __new: *const c_char) -> i32;
    pub fn snprintf(__s: *mut c_char, __maxlen: size_t, __format: *const c_char, ...) -> i32;
    pub fn sprintf(__s: *mut c_char, __format: *const c_char, ...) -> i32;
    pub static mut stderr: *mut FILE;
    pub fn strtod(__nptr: *const c_char, __endptr: *mut *mut c_char) -> c_double;
    pub fn strtol(__nptr: *const c_char, __endptr: *mut *mut c_char, __base: i32) -> c_long;
    pub fn time(__timer: *mut time_t) -> time_t;
}

/// Print the fixed `message` through the C library's stdout, as the
/// original's debug lines do.
pub(crate) fn print(message: &core::ffi::CStr) {
    // The message has no conversions, so printf reads only the message.
    unsafe { printf(message.as_ptr()) };
}
