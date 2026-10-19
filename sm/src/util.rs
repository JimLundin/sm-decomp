use crate::host::Die;
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data};
extern "C" {

    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
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
    fn fseek(
        __stream: *mut FILE,
        __off: ::core::ffi::c_long,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ftell(__stream: *mut FILE) -> ::core::ffi::c_long;
    fn rewind(__stream: *mut FILE);
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strdup(__s: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub use crate::types::{
    _IO_lock_t, __off64_t, __off_t, __uint8_t, size_t, uint8, uint8_t, ByteArray, _IO_FILE, FILE,
};
pub type va_list = __gnuc_va_list;
pub type __gnuc_va_list = __builtin_va_list;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
pub const SEEK_END: ::core::ffi::c_int = 2;
pub unsafe fn NextDelim(
    mut s: *mut *mut ::core::ffi::c_char,
    mut sep: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut r: *mut ::core::ffi::c_char = *s;
    if !r.is_null() {
        while *r.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == ' ' as i32
            || *r.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\t' as i32
        {
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
    return r;
}
#[inline]
unsafe fn ToLower(mut a: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return a
        + (a >= 'A' as i32 && a <= 'Z' as i32) as ::core::ffi::c_int * 32 as ::core::ffi::c_int;
}
pub unsafe fn StringEqualsNoCase(
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) -> bool {
    loop {
        let fresh2 = a;
        a = a.offset(1);
        let mut aa: ::core::ffi::c_int = ToLower(*fresh2 as ::core::ffi::c_int);
        let fresh3 = b;
        b = b.offset(1);
        let mut bb: ::core::ffi::c_int = ToLower(*fresh3 as ::core::ffi::c_int);
        if aa != bb {
            return false_0 != 0;
        }
        if aa == 0 as ::core::ffi::c_int {
            return true_0 != 0;
        }
    }
}
pub unsafe fn StringStartsWithNoCase(
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    loop {
        let mut aa: ::core::ffi::c_int = ToLower(*a as ::core::ffi::c_int);
        let mut bb: ::core::ffi::c_int = ToLower(*b as ::core::ffi::c_int);
        if bb == 0 as ::core::ffi::c_int {
            return a;
        }
        if aa != bb {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        a = a.offset(1);
        b = b.offset(1);
    }
}
pub unsafe fn ReadWholeFile(
    mut name: *const ::core::ffi::c_char,
    mut length: *mut size_t,
) -> *mut uint8 {
    let mut f: *mut FILE =
        fopen(name, b"rb\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    if f.is_null() {
        return ::core::ptr::null_mut::<uint8>();
    }
    fseek(f, 0 as ::core::ffi::c_long, SEEK_END);
    let mut size: size_t = ftell(f) as size_t;
    rewind(f);
    let mut buffer: *mut uint8 = malloc(size.wrapping_add(1 as size_t)) as *mut uint8;
    if buffer.is_null() {
        Die(b"malloc failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    *buffer.offset(size as isize) = 0 as uint8;
    if fread(buffer as *mut ::core::ffi::c_void, 1 as size_t, size, f) as size_t != size {
        Die(b"fread failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    fclose(f);
    if !length.is_null() {
        *length = size;
    }
    return buffer;
}
pub unsafe fn NextLineStripComments(
    mut s: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = *s;
    if p.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut eol: *mut ::core::ffi::c_char = strchr(p, '\n' as i32);
    *s = if !eol.is_null() {
        eol.offset(1 as ::core::ffi::c_int as isize)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    eol = if !eol.is_null() {
        eol
    } else {
        p.offset(strlen(p) as isize)
    };
    let mut comment: *mut ::core::ffi::c_char = memchr(
        p as *const ::core::ffi::c_void,
        '#' as i32,
        eol.offset_from(p) as ::core::ffi::c_long as size_t,
    ) as *mut ::core::ffi::c_char;
    eol = if !comment.is_null() { comment } else { eol };
    while eol > p
        && (*eol.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int == '\r' as i32
            || *eol.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int == ' ' as i32
            || *eol.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == '\t' as i32)
    {
        eol = eol.offset(-1);
    }
    *eol = 0 as ::core::ffi::c_char;
    while *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == ' ' as i32
        || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\t' as i32
    {
        p = p.offset(1);
    }
    return p;
}
pub unsafe fn NextPossiblyQuotedString(
    mut s: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut r: *mut ::core::ffi::c_char = *s;
    let mut t: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    while *r as ::core::ffi::c_int == ' ' as i32 || *r as ::core::ffi::c_int == '\t' as i32 {
        r = r.offset(1);
    }
    if *r as ::core::ffi::c_int == '"' as i32 {
        r = r.offset(1);
        t = r;
        while *t as ::core::ffi::c_int != 0 && *t as ::core::ffi::c_int != '"' as i32 {
            t = t.offset(1);
        }
    } else {
        t = r;
        while *t as ::core::ffi::c_int != 0
            && *t as ::core::ffi::c_int != ' ' as i32
            && *t as ::core::ffi::c_int != '\t' as i32
        {
            t = t.offset(1);
        }
    }
    if *t != 0 {
        let fresh1 = t;
        t = t.offset(1);
        *fresh1 = 0 as ::core::ffi::c_char;
    }
    while *t as ::core::ffi::c_int == ' ' as i32 || *t as ::core::ffi::c_int == '\t' as i32 {
        t = t.offset(1);
    }
    *s = t;
    return r;
}
pub unsafe fn ReplaceFilenameWithNewPath(
    mut old_path: *const ::core::ffi::c_char,
    mut new_path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut olen: size_t = strlen(old_path);
    let mut nlen: size_t = strlen(new_path).wrapping_add(1 as size_t);
    while olen != 0
        && *old_path.offset(olen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            != '/' as i32
        && *old_path.offset(olen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            != '\\' as i32
    {
        olen = olen.wrapping_sub(1);
    }
    let mut result: *mut ::core::ffi::c_char =
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
    return result;
}
pub unsafe fn SplitKeyValue(mut p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut equals: *mut ::core::ffi::c_char = strchr(p, '=' as i32);
    if equals.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut kr: *mut ::core::ffi::c_char = equals;
    while kr > p
        && (*kr.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int == ' ' as i32
            || *kr.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int == '\t' as i32)
    {
        kr = kr.offset(-1);
    }
    *kr = 0 as ::core::ffi::c_char;
    let mut v: *mut ::core::ffi::c_char = equals.offset(1 as ::core::ffi::c_int as isize);
    while *v.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == ' ' as i32
        || *v.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\t' as i32
    {
        v = v.offset(1);
    }
    return v;
}
pub unsafe fn SkipPrefix(
    mut big: *const ::core::ffi::c_char,
    mut little: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    while *little != 0 {
        if *little as ::core::ffi::c_int != *big as ::core::ffi::c_int {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        big = big.offset(1);
        little = little.offset(1);
    }
    return big;
}
pub unsafe fn StrSet(mut rv: *mut *mut ::core::ffi::c_char, mut s: *const ::core::ffi::c_char) {
    let mut news: *mut ::core::ffi::c_char = strdup(s);
    let mut old: *mut ::core::ffi::c_char = *rv;
    *rv = news;
    free(old as *mut ::core::ffi::c_void);
}
pub unsafe fn ByteArray_Resize(mut arr: *mut ByteArray, mut new_size: size_t) {
    (*arr).size = new_size;
    if new_size > (*arr).capacity {
        let mut minsize: size_t = (*arr)
            .capacity
            .wrapping_add((*arr).capacity >> 1 as ::core::ffi::c_int)
            .wrapping_add(8 as size_t);
        (*arr).capacity = if new_size < minsize {
            minsize
        } else {
            new_size
        };
        let mut data: *mut uint8 =
            realloc((*arr).data as *mut ::core::ffi::c_void, (*arr).capacity) as *mut uint8;
        if data.is_null() {
            Die(b"memory allocation failed\0" as *const u8 as *const ::core::ffi::c_char);
        }
        (*arr).data = data;
    }
}
pub unsafe fn ByteArray_Destroy(mut arr: *mut ByteArray) {
    let mut data: *mut ::core::ffi::c_void = (*arr).data as *mut ::core::ffi::c_void;
    (*arr).data = ::core::ptr::null_mut::<uint8>();
    free(data);
}
pub unsafe fn ByteArray_AppendData(
    mut arr: *mut ByteArray,
    mut data: *const uint8,
    mut data_size: size_t,
) {
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
pub unsafe fn ByteArray_AppendByte(mut arr: *mut ByteArray, mut v: uint8) {
    ByteArray_Resize(arr, (*arr).size.wrapping_add(1 as size_t));
    *(*arr)
        .data
        .offset((*arr).size.wrapping_sub(1 as size_t) as isize) = v;
}
