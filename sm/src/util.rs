use crate::host::Die;
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data};
extern "C" {

    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn fclose(__stream: *mut FILE) -> i32;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn fread(
        __ptr: *mut ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __stream: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn fseek(__stream: *mut FILE, __off: ::core::ffi::c_long, __whence: i32) -> i32;
    fn ftell(__stream: *mut FILE) -> ::core::ffi::c_long;
    fn rewind(__stream: *mut FILE);
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memchr(__s: *const ::core::ffi::c_void, __c: i32, __n: size_t) -> *mut ::core::ffi::c_void;
    fn strdup(__s: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strchr(__s: *const ::core::ffi::c_char, __c: i32) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: u32,
    pub fp_offset: u32,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub use crate::types::{_IO_lock_t, __off64_t, __off_t, size_t, ByteArray, _IO_FILE, FILE};
pub type va_list = __gnuc_va_list;
pub type __gnuc_va_list = __builtin_va_list;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
pub const SEEK_END: i32 = 2;
pub unsafe fn NextDelim(s: *mut *mut ::core::ffi::c_char, sep: i32) -> *mut ::core::ffi::c_char {
    let mut r: *mut ::core::ffi::c_char = *s;
    if !r.is_null() {
        while *r.offset(0) as i32 == ' ' as i32 || *r.offset(0) as i32 == '\t' as i32 {
            r = r.offset(1);
        }
        let mut t: *mut ::core::ffi::c_char = strchr(r, sep);
        *s = if !t.is_null() {
            let fresh0 = t;
            t = t.offset(1);
            *fresh0 = 0 as ::core::ffi::c_char;
            t
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
    }
    r
}
#[inline]
unsafe fn ToLower(a: i32) -> i32 {
    a + (a >= 'A' as i32 && a <= 'Z' as i32) as i32 * 32 as i32
}
pub unsafe fn StringEqualsNoCase(
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) -> bool {
    loop {
        let fresh2 = a;
        a = a.offset(1);
        let aa: i32 = ToLower(*fresh2 as i32);
        let fresh3 = b;
        b = b.offset(1);
        let bb: i32 = ToLower(*fresh3 as i32);
        if aa != bb {
            return false_0 != 0;
        }
        if aa == 0 as i32 {
            return true_0 != 0;
        }
    }
}
pub unsafe fn StringStartsWithNoCase(
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    loop {
        let aa: i32 = ToLower(*a as i32);
        let bb: i32 = ToLower(*b as i32);
        if bb == 0 as i32 {
            return a;
        }
        if aa != bb {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        a = a.offset(1);
        b = b.offset(1);
    }
}
pub unsafe fn ReadWholeFile(name: *const ::core::ffi::c_char, length: *mut size_t) -> *mut u8 {
    let f: *mut FILE = fopen(name, b"rb\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    if f.is_null() {
        return ::core::ptr::null_mut::<u8>();
    }
    fseek(f, 0 as ::core::ffi::c_long, SEEK_END);
    let size: size_t = ftell(f) as size_t;
    rewind(f);
    let buffer: *mut u8 = malloc(size.wrapping_add(1 as size_t)) as *mut u8;
    if buffer.is_null() {
        Die(b"malloc failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    *buffer.offset(size as isize) = 0 as u8;
    if fread(buffer as *mut ::core::ffi::c_void, 1 as size_t, size, f) as size_t != size {
        Die(b"fread failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    fclose(f);
    if !length.is_null() {
        *length = size;
    }
    buffer
}
pub unsafe fn NextLineStripComments(s: *mut *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = *s;
    if p.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut eol: *mut ::core::ffi::c_char = strchr(p, '\n' as i32);
    *s = if !eol.is_null() {
        eol.offset(1)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    eol = if !eol.is_null() {
        eol
    } else {
        p.offset(strlen(p) as isize)
    };
    let comment: *mut ::core::ffi::c_char = memchr(
        p as *const ::core::ffi::c_void,
        '#' as i32,
        eol.offset_from(p) as ::core::ffi::c_long as size_t,
    ) as *mut ::core::ffi::c_char;
    eol = if !comment.is_null() { comment } else { eol };
    while eol > p
        && (*eol.offset(-(1 as i32) as isize) as i32 == '\r' as i32
            || *eol.offset(-(1 as i32) as isize) as i32 == ' ' as i32
            || *eol.offset(-(1 as i32) as isize) as i32 == '\t' as i32)
    {
        eol = eol.offset(-1);
    }
    *eol = 0 as ::core::ffi::c_char;
    while *p.offset(0) as i32 == ' ' as i32 || *p.offset(0) as i32 == '\t' as i32 {
        p = p.offset(1);
    }
    p
}
pub unsafe fn NextPossiblyQuotedString(
    s: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut r: *mut ::core::ffi::c_char = *s;
    let mut t: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    while *r as i32 == ' ' as i32 || *r as i32 == '\t' as i32 {
        r = r.offset(1);
    }
    if *r as i32 == '"' as i32 {
        r = r.offset(1);
        t = r;
        while *t as i32 != 0 && *t as i32 != '"' as i32 {
            t = t.offset(1);
        }
    } else {
        t = r;
        while *t as i32 != 0 && *t as i32 != ' ' as i32 && *t as i32 != '\t' as i32 {
            t = t.offset(1);
        }
    }
    if *t != 0 {
        let fresh1 = t;
        t = t.offset(1);
        *fresh1 = 0 as ::core::ffi::c_char;
    }
    while *t as i32 == ' ' as i32 || *t as i32 == '\t' as i32 {
        t = t.offset(1);
    }
    *s = t;
    r
}
pub unsafe fn ReplaceFilenameWithNewPath(
    old_path: *const ::core::ffi::c_char,
    new_path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut olen: size_t = strlen(old_path);
    let nlen: size_t = strlen(new_path).wrapping_add(1 as size_t);
    while olen != 0
        && *old_path.offset(olen.wrapping_sub(1 as size_t) as isize) as i32 != '/' as i32
        && *old_path.offset(olen.wrapping_sub(1 as size_t) as isize) as i32 != '\\' as i32
    {
        olen = olen.wrapping_sub(1);
    }
    let result: *mut ::core::ffi::c_char =
        malloc(olen.wrapping_add(nlen)) as *mut ::core::ffi::c_char;
    memcpy(
        result as *mut ::core::ffi::c_void,
        old_path as *const ::core::ffi::c_void,
        olen,
    );
    memcpy(
        result.offset(olen as isize) as *mut ::core::ffi::c_void,
        new_path as *const ::core::ffi::c_void,
        nlen,
    );
    result
}
pub unsafe fn SplitKeyValue(p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let equals: *mut ::core::ffi::c_char = strchr(p, '=' as i32);
    if equals.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut kr: *mut ::core::ffi::c_char = equals;
    while kr > p
        && (*kr.offset(-(1 as i32) as isize) as i32 == ' ' as i32
            || *kr.offset(-(1 as i32) as isize) as i32 == '\t' as i32)
    {
        kr = kr.offset(-1);
    }
    *kr = 0 as ::core::ffi::c_char;
    let mut v: *mut ::core::ffi::c_char = equals.offset(1);
    while *v.offset(0) as i32 == ' ' as i32 || *v.offset(0) as i32 == '\t' as i32 {
        v = v.offset(1);
    }
    v
}
pub unsafe fn SkipPrefix(
    mut big: *const ::core::ffi::c_char,
    mut little: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    while *little != 0 {
        if *little as i32 != *big as i32 {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        big = big.offset(1);
        little = little.offset(1);
    }
    big
}
pub unsafe fn StrSet(rv: *mut *mut ::core::ffi::c_char, s: *const ::core::ffi::c_char) {
    let news: *mut ::core::ffi::c_char = strdup(s);
    let old: *mut ::core::ffi::c_char = *rv;
    *rv = news;
    free(old as *mut ::core::ffi::c_void);
}
pub unsafe fn ByteArray_Resize(arr: *mut ByteArray, new_size: size_t) {
    (*arr).size = new_size;
    if new_size > (*arr).capacity {
        let minsize: size_t = (*arr)
            .capacity
            .wrapping_add((*arr).capacity >> 1 as i32)
            .wrapping_add(8 as size_t);
        (*arr).capacity = if new_size < minsize {
            minsize
        } else {
            new_size
        };
        let data: *mut u8 =
            realloc((*arr).data as *mut ::core::ffi::c_void, (*arr).capacity) as *mut u8;
        if data.is_null() {
            Die(b"memory allocation failed\0" as *const u8 as *const ::core::ffi::c_char);
        }
        (*arr).data = data;
    }
}
pub unsafe fn ByteArray_Destroy(arr: *mut ByteArray) {
    let data: *mut ::core::ffi::c_void = (*arr).data as *mut ::core::ffi::c_void;
    (*arr).data = ::core::ptr::null_mut::<u8>();
    free(data);
}
pub unsafe fn ByteArray_AppendData(arr: *mut ByteArray, data: *const u8, data_size: size_t) {
    ByteArray_Resize(arr, (*arr).size.wrapping_add(data_size));
    memcpy(
        (*arr)
            .data
            .offset((*arr).size as isize)
            .offset(-(data_size as isize)) as *mut ::core::ffi::c_void,
        data as *const ::core::ffi::c_void,
        data_size,
    );
}
pub unsafe fn ByteArray_AppendByte(arr: *mut ByteArray, v: u8) {
    ByteArray_Resize(arr, (*arr).size.wrapping_add(1 as size_t));
    *(*arr)
        .data
        .offset((*arr).size.wrapping_sub(1 as size_t) as isize) = v;
}
