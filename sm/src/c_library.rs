//! Native C-library boundary. Keep the original ABI types and symbols here.

use crate::types::{__mode_t, FILE, size_t, time_t};
use ::core::ffi::{c_char, c_double, c_long, c_ulong, c_void};

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
    pub fn fclose(__stream: *mut FILE) -> i32;
    pub fn feof(__stream: *mut FILE) -> i32;
    pub fn ferror(__stream: *mut FILE) -> i32;
    pub fn fgetc(__stream: *mut FILE) -> i32;
    pub fn fopen(__filename: *const c_char, __modes: *const c_char) -> *mut FILE;
    pub fn fprintf(__stream: *mut FILE, __format: *const c_char, ...) -> i32;
    pub fn fread(__ptr: *mut c_void, __size: size_t, __n: size_t, __stream: *mut FILE) -> c_ulong;
    pub fn free(__ptr: *mut c_void);
    pub fn fseek(__stream: *mut FILE, __off: c_long, __whence: i32) -> i32;
    pub fn ftell(__stream: *mut FILE) -> c_long;
    pub fn fwrite(__ptr: *const c_void, __size: size_t, __n: size_t, __s: *mut FILE) -> c_ulong;
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
    pub fn rewind(__stream: *mut FILE);
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
    pub fn ungetc(__c: i32, __stream: *mut FILE) -> i32;
}
