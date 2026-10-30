//! Native C-library boundary. Keep the original ABI types and symbols here.

use crate::types::{__mode_t, FILE, size_t, time_t};
use ::core::ffi::{c_char, c_double, c_long, c_ulong, c_void};

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
    pub fn calloc(__nmemb: size_t, __size: size_t) -> *mut c_void;
    pub fn chdir(__path: *const c_char) -> i32;
    pub fn exit(__status: i32) -> !;
    pub fn fprintf(__stream: *mut FILE, __format: *const c_char, ...) -> i32;
    pub fn free(__ptr: *mut c_void);
    pub fn getcwd(__buf: *mut c_char, __size: size_t) -> *mut c_char;
    pub fn malloc(__size: size_t) -> *mut c_void;
    pub fn memchr(__s: *const c_void, __c: i32, __n: size_t) -> *mut c_void;
    pub fn memcmp(__s1: *const c_void, __s2: *const c_void, __n: size_t) -> i32;
    pub fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: size_t) -> *mut c_void;
    pub fn memset(__s: *mut c_void, __c: i32, __n: size_t) -> *mut c_void;
    pub fn mkdir(__path: *const c_char, __mode: __mode_t) -> i32;
    pub fn pow(__x: c_double, __y: c_double) -> c_double;
    pub fn printf(__format: *const c_char, ...) -> i32;
    pub fn puts(__s: *const c_char) -> i32;
    pub fn realloc(__ptr: *mut c_void, __size: size_t) -> *mut c_void;
    pub fn rename(__old: *const c_char, __new: *const c_char) -> i32;
    pub fn snprintf(__s: *mut c_char, __maxlen: size_t, __format: *const c_char, ...) -> i32;
    pub fn sprintf(__s: *mut c_char, __format: *const c_char, ...) -> i32;
    pub static mut stderr: *mut FILE;
    pub fn strchr(__s: *const c_char, __c: i32) -> *mut c_char;
    pub fn strcmp(__s1: *const c_char, __s2: *const c_char) -> i32;
    pub fn strcpy(__dest: *mut c_char, __src: *const c_char) -> *mut c_char;
    pub fn strdup(__s: *const c_char) -> *mut c_char;
    pub fn strlen(__s: *const c_char) -> size_t;
    pub fn strtod(__nptr: *const c_char, __endptr: *mut *mut c_char) -> c_double;
    pub fn strtol(__nptr: *const c_char, __endptr: *mut *mut c_char, __base: i32) -> c_long;
    pub fn strtoul(__nptr: *const c_char, __endptr: *mut *mut c_char, __base: i32) -> c_ulong;
    pub fn time(__timer: *mut time_t) -> time_t;
}
