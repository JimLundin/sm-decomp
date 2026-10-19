use crate::config::{g_config, ParseBool};
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data};
use crate::util::{
    ByteArray_AppendData, ByteArray_Destroy, NextDelim, NextLineStripComments,
    NextPossiblyQuotedString, ReadWholeFile, ReplaceFilenameWithNewPath, SkipPrefix, SplitKeyValue,
    StrSet, StringEqualsNoCase,
};
extern "C" {

    fn strtod(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
    fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    fn strtoul(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn calloc(__nmemb: size_t, __size: size_t) -> *mut ::core::ffi::c_void;
    fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn abs(__x: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;

    static mut stderr: *mut FILE;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn fgetc(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn ungetc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
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
    fn feof(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn ferror(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strdup(__s: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;

}
#[cfg_attr(unix, link(name = "m"))]
extern "C" {
    fn pow(__x: ::core::ffi::c_double, __y: ::core::ffi::c_double) -> ::core::ffi::c_double;
}
pub use crate::types::{
    __off64_t, __off_t, __uint16_t, __uint32_t, __uint8_t, ptrdiff_t, size_t, uint, uint16,
    uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t, GLboolean, GLchar, GLenum, GLfloat,
    GLint, GLsizei, GLsizeiptr, GLuint,
};
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const kGlslMaxTextures: C2RustUnnamed = 10;
pub const kGlslMaxPasses: C2RustUnnamed = 20;
pub use crate::types::GLSLScaleType;
pub const GLSL_ABSOLUTE: GLSLScaleType = 3;
pub const GLSL_VIEWPORT: GLSLScaleType = 2;
pub const GLSL_SOURCE: GLSLScaleType = 1;
pub const GLSL_NONE: GLSLScaleType = 0;
pub use crate::types::{
    _IO_lock_t, stbi_uc, ByteArray, GlTextureWithSize, GlslParam, GlslPass, GlslShader,
    GlslTexture, GlslTextureUniform, GlslUniforms, _IO_FILE, FILE,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__context {
    pub img_x: stbi__uint32,
    pub img_y: stbi__uint32,
    pub img_n: ::core::ffi::c_int,
    pub img_out_n: ::core::ffi::c_int,
    pub io: stbi_io_callbacks,
    pub io_user_data: *mut ::core::ffi::c_void,
    pub read_from_callbacks: ::core::ffi::c_int,
    pub buflen: ::core::ffi::c_int,
    pub buffer_start: [stbi_uc; 128],
    pub callback_already_read: ::core::ffi::c_int,
    pub img_buffer: *mut stbi_uc,
    pub img_buffer_end: *mut stbi_uc,
    pub img_buffer_original: *mut stbi_uc,
    pub img_buffer_original_end: *mut stbi_uc,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi_io_callbacks {
    pub read: Option<
        unsafe fn(
            *mut ::core::ffi::c_void,
            *mut ::core::ffi::c_char,
            ::core::ffi::c_int,
        ) -> ::core::ffi::c_int,
    >,
    pub skip: Option<unsafe fn(*mut ::core::ffi::c_void, ::core::ffi::c_int) -> ()>,
    pub eof: Option<unsafe fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
}
pub use crate::types::stbi__uint32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__result_info {
    pub bits_per_channel: ::core::ffi::c_int,
    pub num_channels: ::core::ffi::c_int,
    pub channel_order: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__png {
    pub s: *mut stbi__context,
    pub idata: *mut stbi_uc,
    pub expanded: *mut stbi_uc,
    pub out: *mut stbi_uc,
    pub depth: ::core::ffi::c_int,
}
pub use crate::types::stbi__uint16;
pub const STBI__SCAN_load: C2RustUnnamed_1 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__pngchunk {
    pub length: stbi__uint32,
    pub type_0: stbi__uint32,
}
pub const STBI__F_paeth_first: C2RustUnnamed_2 = 6;
pub const STBI__F_avg_first: C2RustUnnamed_2 = 5;
pub const STBI__F_paeth: C2RustUnnamed_2 = 4;
pub const STBI__F_avg: C2RustUnnamed_2 = 3;
pub const STBI__F_up: C2RustUnnamed_2 = 2;
pub const STBI__F_sub: C2RustUnnamed_2 = 1;
pub const STBI__F_none: C2RustUnnamed_2 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__zbuf {
    pub zbuffer: *mut stbi_uc,
    pub zbuffer_end: *mut stbi_uc,
    pub num_bits: ::core::ffi::c_int,
    pub code_buffer: stbi__uint32,
    pub zout: *mut ::core::ffi::c_char,
    pub zout_start: *mut ::core::ffi::c_char,
    pub zout_end: *mut ::core::ffi::c_char,
    pub z_expandable: ::core::ffi::c_int,
    pub z_length: stbi__zhuffman,
    pub z_distance: stbi__zhuffman,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__zhuffman {
    pub fast: [stbi__uint16; 512],
    pub firstcode: [stbi__uint16; 16],
    pub maxcode: [::core::ffi::c_int; 17],
    pub firstsymbol: [stbi__uint16; 16],
    pub size: [stbi_uc; 288],
    pub value: [stbi__uint16; 288],
}
pub const STBI__SCAN_header: C2RustUnnamed_1 = 2;
pub const STBI__SCAN_type: C2RustUnnamed_1 = 1;
pub const STBI_ORDER_RGB: C2RustUnnamed_0 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RenderCtx {
    pub texture_unit: uint,
    pub offset: uint,
    pub num_vaos: uint,
    pub vaos: [uint; 51],
}
pub const kMaxVaosInRenderCtx: C2RustUnnamed_3 = 51;
pub use crate::types::{stbi_us, Config};
impl stbi__zhuffman {
    const EMPTY: Self = stbi__zhuffman {
        fast: [0; 512],
        firstcode: [0; 16],
        maxcode: [0; 17],
        firstsymbol: [0; 16],
        size: [0; 288],
        value: [0; 288],
    };
}
impl stbi__png {
    const EMPTY: Self = stbi__png {
        s: ::core::ptr::null_mut::<stbi__context>(),
        idata: ::core::ptr::null_mut::<stbi_uc>(),
        expanded: ::core::ptr::null_mut::<stbi_uc>(),
        out: ::core::ptr::null_mut::<stbi_uc>(),
        depth: 0,
    };
}
impl stbi__context {
    const EMPTY: Self = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: ::core::ptr::null_mut::<stbi_uc>(),
        img_buffer_end: ::core::ptr::null_mut::<stbi_uc>(),
        img_buffer_original: ::core::ptr::null_mut::<stbi_uc>(),
        img_buffer_original_end: ::core::ptr::null_mut::<stbi_uc>(),
    };
}
impl stbi__zbuf {
    const EMPTY: Self = stbi__zbuf {
        zbuffer: ::core::ptr::null_mut::<stbi_uc>(),
        zbuffer_end: ::core::ptr::null_mut::<stbi_uc>(),
        num_bits: 0,
        code_buffer: 0,
        zout: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        zout_start: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        zout_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        z_expandable: 0,
        z_length: stbi__zhuffman::EMPTY,
        z_distance: stbi__zhuffman::EMPTY,
    };
}
pub type C2RustUnnamed_0 = ::core::ffi::c_uint;
pub const STBI_ORDER_BGR: C2RustUnnamed_0 = 1;
pub type C2RustUnnamed_1 = ::core::ffi::c_uint;
pub type C2RustUnnamed_2 = ::core::ffi::c_uint;
pub type C2RustUnnamed_3 = ::core::ffi::c_uint;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const GL_TEXTURE_2D: ::core::ffi::c_int = gl::TEXTURE_2D as ::core::ffi::c_int;
pub const GL_CLAMP: ::core::ffi::c_int = 0x2900;
pub const GL_FALSE: ::core::ffi::c_int = gl::FALSE as ::core::ffi::c_int;
pub const GL_FLOAT: ::core::ffi::c_int = gl::FLOAT as ::core::ffi::c_int;
pub const GL_LINEAR: ::core::ffi::c_int = gl::LINEAR as ::core::ffi::c_int;
pub const GL_LINEAR_MIPMAP_LINEAR: ::core::ffi::c_int =
    gl::LINEAR_MIPMAP_LINEAR as ::core::ffi::c_int;
pub const GL_LUMINANCE: ::core::ffi::c_int = 0x1909;
pub const GL_NEAREST: ::core::ffi::c_int = gl::NEAREST as ::core::ffi::c_int;
pub const GL_NEAREST_MIPMAP_NEAREST: ::core::ffi::c_int =
    gl::NEAREST_MIPMAP_NEAREST as ::core::ffi::c_int;
pub const GL_REPEAT: ::core::ffi::c_int = gl::REPEAT as ::core::ffi::c_int;
pub const GL_RGB: ::core::ffi::c_int = gl::RGB as ::core::ffi::c_int;
pub const GL_RGBA: ::core::ffi::c_int = gl::RGBA as ::core::ffi::c_int;
pub const GL_TEXTURE_MAG_FILTER: ::core::ffi::c_int = gl::TEXTURE_MAG_FILTER as ::core::ffi::c_int;
pub const GL_TEXTURE_MIN_FILTER: ::core::ffi::c_int = gl::TEXTURE_MIN_FILTER as ::core::ffi::c_int;
pub const GL_TEXTURE_WRAP_S: ::core::ffi::c_int = gl::TEXTURE_WRAP_S as ::core::ffi::c_int;
pub const GL_TEXTURE_WRAP_T: ::core::ffi::c_int = gl::TEXTURE_WRAP_T as ::core::ffi::c_int;
pub const GL_TRIANGLE_STRIP: ::core::ffi::c_int = gl::TRIANGLE_STRIP as ::core::ffi::c_int;
pub const GL_TRUE: ::core::ffi::c_int = gl::TRUE as ::core::ffi::c_int;
pub const GL_UNSIGNED_BYTE: ::core::ffi::c_int = gl::UNSIGNED_BYTE as ::core::ffi::c_int;
pub const GL_CLAMP_TO_EDGE: ::core::ffi::c_int = gl::CLAMP_TO_EDGE as ::core::ffi::c_int;
pub const GL_UNSIGNED_INT_8_8_8_8: ::core::ffi::c_int =
    gl::UNSIGNED_INT_8_8_8_8 as ::core::ffi::c_int;
pub const GL_CLAMP_TO_BORDER: ::core::ffi::c_int = gl::CLAMP_TO_BORDER as ::core::ffi::c_int;
pub const GL_TEXTURE0: ::core::ffi::c_int = gl::TEXTURE0 as ::core::ffi::c_int;
pub const GL_ARRAY_BUFFER: ::core::ffi::c_int = gl::ARRAY_BUFFER as ::core::ffi::c_int;
pub const GL_STATIC_DRAW: ::core::ffi::c_int = gl::STATIC_DRAW as ::core::ffi::c_int;
pub const GL_COMPILE_STATUS: ::core::ffi::c_int = gl::COMPILE_STATUS as ::core::ffi::c_int;
pub const GL_FRAGMENT_SHADER: ::core::ffi::c_int = gl::FRAGMENT_SHADER as ::core::ffi::c_int;
pub const GL_LINK_STATUS: ::core::ffi::c_int = gl::LINK_STATUS as ::core::ffi::c_int;
pub const GL_VERTEX_SHADER: ::core::ffi::c_int = gl::VERTEX_SHADER as ::core::ffi::c_int;
pub const GL_SRGB8_ALPHA8: ::core::ffi::c_int = gl::SRGB8_ALPHA8 as ::core::ffi::c_int;
pub const GL_COLOR_ATTACHMENT0: ::core::ffi::c_int = gl::COLOR_ATTACHMENT0 as ::core::ffi::c_int;
pub const GL_FRAMEBUFFER: ::core::ffi::c_int = gl::FRAMEBUFFER as ::core::ffi::c_int;
pub const GL_FRAMEBUFFER_BINDING: ::core::ffi::c_int =
    gl::FRAMEBUFFER_BINDING as ::core::ffi::c_int;
pub const GL_FRAMEBUFFER_SRGB: ::core::ffi::c_int = gl::FRAMEBUFFER_SRGB as ::core::ffi::c_int;
pub const GL_RGBA32F: ::core::ffi::c_int = gl::RGBA32F as ::core::ffi::c_int;
#[inline]
unsafe fn atoi(mut __nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    strtol(
        __nptr,
        NULL as *mut *mut ::core::ffi::c_char,
        10 as ::core::ffi::c_int,
    ) as ::core::ffi::c_int
}
#[inline]
unsafe fn atof(mut __nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_double {
    strtod(__nptr, NULL as *mut *mut ::core::ffi::c_char)
}
pub const EOF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const SEEK_SET: ::core::ffi::c_int = 0;
pub const SEEK_CUR: ::core::ffi::c_int = 1;
unsafe fn stbi__start_mem(s: *mut stbi__context, buffer: *const stbi_uc, len: ::core::ffi::c_int) {
    (*s).io.read = None;
    (*s).read_from_callbacks = 0 as ::core::ffi::c_int;
    (*s).callback_already_read = 0 as ::core::ffi::c_int;
    (*s).img_buffer_original = buffer as *mut stbi_uc;
    (*s).img_buffer = (*s).img_buffer_original;
    (*s).img_buffer_original_end = (buffer as *mut stbi_uc).offset(len as isize);
    (*s).img_buffer_end = (*s).img_buffer_original_end;
}
unsafe fn stbi__start_callbacks(
    s: *mut stbi__context,
    c: *mut stbi_io_callbacks,
    user: *mut ::core::ffi::c_void,
) {
    (*s).io = *c;
    (*s).io_user_data = user;
    (*s).buflen = ::core::mem::size_of::<[stbi_uc; 128]>() as ::core::ffi::c_int;
    (*s).read_from_callbacks = 1 as ::core::ffi::c_int;
    (*s).callback_already_read = 0 as ::core::ffi::c_int;
    (*s).img_buffer_original = &raw mut (*s).buffer_start as *mut stbi_uc;
    (*s).img_buffer = (*s).img_buffer_original;
    stbi__refill_buffer(s);
    (*s).img_buffer_original_end = (*s).img_buffer_end;
}
unsafe fn stbi__stdio_read(
    user: *mut ::core::ffi::c_void,
    data: *mut ::core::ffi::c_char,
    size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    fread(
        data as *mut ::core::ffi::c_void,
        1 as size_t,
        size as size_t,
        user as *mut FILE,
    ) as ::core::ffi::c_int
}
unsafe fn stbi__stdio_skip(user: *mut ::core::ffi::c_void, n: ::core::ffi::c_int) {
    let mut ch: ::core::ffi::c_int = 0;
    fseek(user as *mut FILE, n as ::core::ffi::c_long, SEEK_CUR);
    ch = fgetc(user as *mut FILE);
    if ch != EOF {
        ungetc(ch, user as *mut FILE);
    }
}
unsafe fn stbi__stdio_eof(user: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    (feof(user as *mut FILE) != 0 || ferror(user as *mut FILE) != 0) as ::core::ffi::c_int
}
static mut stbi__stdio_callbacks: stbi_io_callbacks = unsafe {
    stbi_io_callbacks {
        read: Some(
            stbi__stdio_read
                as unsafe fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_char,
                    ::core::ffi::c_int,
                ) -> ::core::ffi::c_int,
        ),
        skip: Some(
            stbi__stdio_skip as unsafe fn(*mut ::core::ffi::c_void, ::core::ffi::c_int) -> (),
        ),
        eof: Some(stbi__stdio_eof as unsafe fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int),
    }
};
unsafe fn stbi__start_file(s: *mut stbi__context, f: *mut FILE) {
    stbi__start_callbacks(
        s,
        &raw mut stbi__stdio_callbacks,
        f as *mut ::core::ffi::c_void,
    );
}
unsafe fn stbi__rewind(s: *mut stbi__context) {
    (*s).img_buffer = (*s).img_buffer_original;
    (*s).img_buffer_end = (*s).img_buffer_original_end;
}
static mut stbi__g_failure_reason: *const ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>();
pub unsafe fn stbi_failure_reason() -> *const ::core::ffi::c_char {
    stbi__g_failure_reason
}
unsafe fn stbi__malloc(size: size_t) -> *mut ::core::ffi::c_void {
    malloc(size)
}
unsafe fn stbi__addsizes_valid(a: ::core::ffi::c_int, b: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if b < 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    (a <= INT_MAX - b) as ::core::ffi::c_int
}
unsafe fn stbi__mul2sizes_valid(
    a: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if a < 0 as ::core::ffi::c_int || b < 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if b == 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    (a <= INT_MAX / b) as ::core::ffi::c_int
}
unsafe fn stbi__mad2sizes_valid(
    a: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
    add: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    (stbi__mul2sizes_valid(a, b) != 0 && stbi__addsizes_valid(a * b, add) != 0)
        as ::core::ffi::c_int
}
unsafe fn stbi__mad3sizes_valid(
    a: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
    c: ::core::ffi::c_int,
    add: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    (stbi__mul2sizes_valid(a, b) != 0
        && stbi__mul2sizes_valid(a * b, c) != 0
        && stbi__addsizes_valid(a * b * c, add) != 0) as ::core::ffi::c_int
}
unsafe fn stbi__mad4sizes_valid(
    a: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
    c: ::core::ffi::c_int,
    d: ::core::ffi::c_int,
    add: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    (stbi__mul2sizes_valid(a, b) != 0
        && stbi__mul2sizes_valid(a * b, c) != 0
        && stbi__mul2sizes_valid(a * b * c, d) != 0
        && stbi__addsizes_valid(a * b * c * d, add) != 0) as ::core::ffi::c_int
}
unsafe fn stbi__malloc_mad2(
    a: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
    add: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    if stbi__mad2sizes_valid(a, b, add) == 0 {
        return NULL_0;
    }
    stbi__malloc((a * b + add) as size_t)
}
unsafe fn stbi__malloc_mad3(
    a: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
    c: ::core::ffi::c_int,
    add: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    if stbi__mad3sizes_valid(a, b, c, add) == 0 {
        return NULL_0;
    }
    stbi__malloc((a * b * c + add) as size_t)
}
unsafe fn stbi__malloc_mad4(
    a: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
    c: ::core::ffi::c_int,
    d: ::core::ffi::c_int,
    add: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    if stbi__mad4sizes_valid(a, b, c, d, add) == 0 {
        return NULL_0;
    }
    stbi__malloc((a * b * c * d + add) as size_t)
}
pub unsafe fn stbi_image_free(retval_from_stbi_load: *mut ::core::ffi::c_void) {
    free(retval_from_stbi_load);
}
static mut stbi__vertically_flip_on_load_global: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub unsafe fn stbi_set_flip_vertically_on_load(flag_true_if_should_flip: ::core::ffi::c_int) {
    stbi__vertically_flip_on_load_global = flag_true_if_should_flip;
}
unsafe fn stbi__load_main(
    s: *mut stbi__context,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
    ri: *mut stbi__result_info,
    bpc: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    memset(
        ri as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<stbi__result_info>() as size_t,
    );
    (*ri).bits_per_channel = 8 as ::core::ffi::c_int;
    (*ri).channel_order = STBI_ORDER_RGB as ::core::ffi::c_int;
    (*ri).num_channels = 0 as ::core::ffi::c_int;
    if stbi__png_test(s) != 0 {
        return stbi__png_load(s, x, y, comp, req_comp, ri);
    }
    ((if 0 as ::core::ffi::c_int != 0 {
        NULL_0
    } else {
        NULL_0
    }) as size_t as *mut ::core::ffi::c_uchar as *mut ::core::ffi::c_void)
}
unsafe fn stbi__convert_16_to_8(
    orig: *mut stbi__uint16,
    w: ::core::ffi::c_int,
    h: ::core::ffi::c_int,
    channels: ::core::ffi::c_int,
) -> *mut stbi_uc {
    let mut i: ::core::ffi::c_int = 0;
    let img_len: ::core::ffi::c_int = w * h * channels;
    let mut reduced: *mut stbi_uc = ::core::ptr::null_mut::<stbi_uc>();
    reduced = stbi__malloc(img_len as size_t) as *mut stbi_uc;
    if reduced.is_null() {
        return (if 0 as ::core::ffi::c_int != 0 {
            NULL_0
        } else {
            NULL_0
        }) as size_t as *mut stbi_uc;
    }
    i = 0 as ::core::ffi::c_int;
    while i < img_len {
        *reduced.offset(i as isize) = (*orig.offset(i as isize) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int) as stbi_uc;
        i += 1;
    }
    free(orig as *mut ::core::ffi::c_void);
    reduced
}
unsafe fn stbi__convert_8_to_16(
    orig: *mut stbi_uc,
    w: ::core::ffi::c_int,
    h: ::core::ffi::c_int,
    channels: ::core::ffi::c_int,
) -> *mut stbi__uint16 {
    let mut i: ::core::ffi::c_int = 0;
    let img_len: ::core::ffi::c_int = w * h * channels;
    let mut enlarged: *mut stbi__uint16 = ::core::ptr::null_mut::<stbi__uint16>();
    enlarged = stbi__malloc((img_len * 2 as ::core::ffi::c_int) as size_t) as *mut stbi__uint16;
    if enlarged.is_null() {
        return (if 0 as ::core::ffi::c_int != 0 {
            NULL_0
        } else {
            NULL_0
        }) as size_t as *mut ::core::ffi::c_uchar as *mut stbi__uint16;
    }
    i = 0 as ::core::ffi::c_int;
    while i < img_len {
        *enlarged.offset(i as isize) =
            (((*orig.offset(i as isize) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                + *orig.offset(i as isize) as ::core::ffi::c_int) as stbi__uint16;
        i += 1;
    }
    free(orig as *mut ::core::ffi::c_void);
    enlarged
}
unsafe fn stbi__vertical_flip(
    image: *mut ::core::ffi::c_void,
    w: ::core::ffi::c_int,
    h: ::core::ffi::c_int,
    bytes_per_pixel: ::core::ffi::c_int,
) {
    let mut row: ::core::ffi::c_int = 0;
    let bytes_per_row: size_t = (w as size_t).wrapping_mul(bytes_per_pixel as size_t);
    let mut temp: [stbi_uc; 2048] = [0; 2048];
    let bytes: *mut stbi_uc = image as *mut stbi_uc;
    row = 0 as ::core::ffi::c_int;
    while row < h >> 1 as ::core::ffi::c_int {
        let mut row0: *mut stbi_uc =
            bytes.offset((row as size_t).wrapping_mul(bytes_per_row) as isize);
        let mut row1: *mut stbi_uc = bytes.offset(
            ((h - row - 1 as ::core::ffi::c_int) as size_t).wrapping_mul(bytes_per_row) as isize,
        );
        let mut bytes_left: size_t = bytes_per_row;
        while bytes_left != 0 {
            let bytes_copy: size_t =
                if bytes_left < ::core::mem::size_of::<[stbi_uc; 2048]>() as usize {
                    bytes_left
                } else {
                    ::core::mem::size_of::<[stbi_uc; 2048]>() as size_t
                };
            memcpy(
                &raw mut temp as *mut stbi_uc as *mut ::core::ffi::c_void,
                row0 as *const ::core::ffi::c_void,
                bytes_copy,
            );
            memcpy(
                row0 as *mut ::core::ffi::c_void,
                row1 as *const ::core::ffi::c_void,
                bytes_copy,
            );
            memcpy(
                row1 as *mut ::core::ffi::c_void,
                &raw mut temp as *mut stbi_uc as *const ::core::ffi::c_void,
                bytes_copy,
            );
            row0 = row0.offset(bytes_copy as isize);
            row1 = row1.offset(bytes_copy as isize);
            bytes_left = bytes_left.wrapping_sub(bytes_copy);
        }
        row += 1;
    }
}
unsafe fn stbi__load_and_postprocess_8bit(
    s: *mut stbi__context,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_uchar {
    let mut ri: stbi__result_info = stbi__result_info {
        bits_per_channel: 0,
        num_channels: 0,
        channel_order: 0,
    };
    let mut result: *mut ::core::ffi::c_void = stbi__load_main(
        s,
        x,
        y,
        comp,
        req_comp,
        &raw mut ri,
        8 as ::core::ffi::c_int,
    );
    if result.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    }
    '_c2rust_label: {
        if ri.bits_per_channel == 8 as ::core::ffi::c_int
            || ri.bits_per_channel == 16 as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"ri.bits_per_channel == 8 || ri.bits_per_channel == 16\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"./third_party/stb/stb_image.h\0" as *const u8
                    as *const ::core::ffi::c_char,
                1249 as ::core::ffi::c_uint,
                b"unsigned char *stbi__load_and_postprocess_8bit(stbi__context *, int *, int *, int *, int)\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    if ri.bits_per_channel != 8 as ::core::ffi::c_int {
        result = stbi__convert_16_to_8(
            result as *mut stbi__uint16,
            *x,
            *y,
            if req_comp == 0 as ::core::ffi::c_int {
                *comp
            } else {
                req_comp
            },
        ) as *mut ::core::ffi::c_void;
        ri.bits_per_channel = 8 as ::core::ffi::c_int;
    }
    if stbi__vertically_flip_on_load_global != 0 {
        let channels: ::core::ffi::c_int = if req_comp != 0 { req_comp } else { *comp };
        stbi__vertical_flip(
            result,
            *x,
            *y,
            (channels as usize).wrapping_mul(::core::mem::size_of::<stbi_uc>() as usize)
                as ::core::ffi::c_int,
        );
    }
    result as *mut ::core::ffi::c_uchar
}
unsafe fn stbi__load_and_postprocess_16bit(
    s: *mut stbi__context,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut stbi__uint16 {
    let mut ri: stbi__result_info = stbi__result_info {
        bits_per_channel: 0,
        num_channels: 0,
        channel_order: 0,
    };
    let mut result: *mut ::core::ffi::c_void = stbi__load_main(
        s,
        x,
        y,
        comp,
        req_comp,
        &raw mut ri,
        16 as ::core::ffi::c_int,
    );
    if result.is_null() {
        return ::core::ptr::null_mut::<stbi__uint16>();
    }
    '_c2rust_label: {
        if ri.bits_per_channel == 8 as ::core::ffi::c_int
            || ri.bits_per_channel == 16 as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"ri.bits_per_channel == 8 || ri.bits_per_channel == 16\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"./third_party/stb/stb_image.h\0" as *const u8
                    as *const ::core::ffi::c_char,
                1275 as ::core::ffi::c_uint,
                b"stbi__uint16 *stbi__load_and_postprocess_16bit(stbi__context *, int *, int *, int *, int)\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    if ri.bits_per_channel != 16 as ::core::ffi::c_int {
        result = stbi__convert_8_to_16(
            result as *mut stbi_uc,
            *x,
            *y,
            if req_comp == 0 as ::core::ffi::c_int {
                *comp
            } else {
                req_comp
            },
        ) as *mut ::core::ffi::c_void;
        ri.bits_per_channel = 16 as ::core::ffi::c_int;
    }
    if stbi__vertically_flip_on_load_global != 0 {
        let channels: ::core::ffi::c_int = if req_comp != 0 { req_comp } else { *comp };
        stbi__vertical_flip(
            result,
            *x,
            *y,
            (channels as usize).wrapping_mul(::core::mem::size_of::<stbi__uint16>() as usize)
                as ::core::ffi::c_int,
        );
    }
    result as *mut stbi__uint16
}
unsafe fn stbi__fopen(
    filename: *const ::core::ffi::c_char,
    mode: *const ::core::ffi::c_char,
) -> *mut FILE {
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    f = fopen(filename, mode) as *mut FILE;
    f
}
pub unsafe fn stbi_load(
    filename: *const ::core::ffi::c_char,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut stbi_uc {
    let f: *mut FILE = stbi__fopen(filename, b"rb\0" as *const u8 as *const ::core::ffi::c_char);
    let mut result: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    if f.is_null() {
        return (if 0 as ::core::ffi::c_int != 0 {
            NULL_0
        } else {
            NULL_0
        }) as size_t as *mut stbi_uc;
    }
    result = stbi_load_from_file(f, x, y, comp, req_comp) as *mut ::core::ffi::c_uchar;
    fclose(f);
    result as *mut stbi_uc
}
pub unsafe fn stbi_load_from_file(
    f: *mut FILE,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut stbi_uc {
    let mut result: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_file(&raw mut s, f);
    result = stbi__load_and_postprocess_8bit(&raw mut s, x, y, comp, req_comp);
    if !result.is_null() {
        fseek(
            f,
            -(s.img_buffer_end.offset_from(s.img_buffer) as ::core::ffi::c_long
                as ::core::ffi::c_int) as ::core::ffi::c_long,
            SEEK_CUR,
        );
    }
    result as *mut stbi_uc
}
pub unsafe fn stbi_load_from_file_16(
    f: *mut FILE,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut stbi_us {
    let mut result: *mut stbi__uint16 = ::core::ptr::null_mut::<stbi__uint16>();
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_file(&raw mut s, f);
    result = stbi__load_and_postprocess_16bit(&raw mut s, x, y, comp, req_comp);
    if !result.is_null() {
        fseek(
            f,
            -(s.img_buffer_end.offset_from(s.img_buffer) as ::core::ffi::c_long
                as ::core::ffi::c_int) as ::core::ffi::c_long,
            SEEK_CUR,
        );
    }
    result as *mut stbi_us
}
pub unsafe fn stbi_load_16(
    filename: *const ::core::ffi::c_char,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut stbi_us {
    let f: *mut FILE = stbi__fopen(filename, b"rb\0" as *const u8 as *const ::core::ffi::c_char);
    let mut result: *mut stbi__uint16 = ::core::ptr::null_mut::<stbi__uint16>();
    if f.is_null() {
        return (if 0 as ::core::ffi::c_int != 0 {
            NULL_0
        } else {
            NULL_0
        }) as size_t as *mut ::core::ffi::c_uchar as *mut stbi_us;
    }
    result = stbi_load_from_file_16(f, x, y, comp, req_comp) as *mut stbi__uint16;
    fclose(f);
    result as *mut stbi_us
}
pub unsafe fn stbi_load_16_from_memory(
    buffer: *const stbi_uc,
    len: ::core::ffi::c_int,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    channels_in_file: *mut ::core::ffi::c_int,
    desired_channels: ::core::ffi::c_int,
) -> *mut stbi_us {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_mem(&raw mut s, buffer, len);
    stbi__load_and_postprocess_16bit(&raw mut s, x, y, channels_in_file, desired_channels)
        as *mut stbi_us
}
pub unsafe fn stbi_load_16_from_callbacks(
    clbk: *const stbi_io_callbacks,
    user: *mut ::core::ffi::c_void,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    channels_in_file: *mut ::core::ffi::c_int,
    desired_channels: ::core::ffi::c_int,
) -> *mut stbi_us {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_callbacks(&raw mut s, clbk as *mut stbi_io_callbacks, user);
    stbi__load_and_postprocess_16bit(&raw mut s, x, y, channels_in_file, desired_channels)
        as *mut stbi_us
}
pub unsafe fn stbi_load_from_memory(
    buffer: *const stbi_uc,
    len: ::core::ffi::c_int,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut stbi_uc {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_mem(&raw mut s, buffer, len);
    stbi__load_and_postprocess_8bit(&raw mut s, x, y, comp, req_comp) as *mut stbi_uc
}
pub unsafe fn stbi_load_from_callbacks(
    clbk: *const stbi_io_callbacks,
    user: *mut ::core::ffi::c_void,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut stbi_uc {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_callbacks(&raw mut s, clbk as *mut stbi_io_callbacks, user);
    stbi__load_and_postprocess_8bit(&raw mut s, x, y, comp, req_comp) as *mut stbi_uc
}
unsafe fn stbi__loadf_main(
    s: *mut stbi__context,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_float {
    let mut data: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    data = stbi__load_and_postprocess_8bit(s, x, y, comp, req_comp);
    if !data.is_null() {
        return stbi__ldr_to_hdr(
            data as *mut stbi_uc,
            *x,
            *y,
            if req_comp != 0 { req_comp } else { *comp },
        );
    }
    ((if 0 as ::core::ffi::c_int != 0 {
        NULL_0
    } else {
        NULL_0
    }) as size_t as *mut ::core::ffi::c_float)
}
pub unsafe fn stbi_loadf_from_memory(
    buffer: *const stbi_uc,
    len: ::core::ffi::c_int,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_float {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_mem(&raw mut s, buffer, len);
    stbi__loadf_main(&raw mut s, x, y, comp, req_comp)
}
pub unsafe fn stbi_loadf_from_callbacks(
    clbk: *const stbi_io_callbacks,
    user: *mut ::core::ffi::c_void,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_float {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_callbacks(&raw mut s, clbk as *mut stbi_io_callbacks, user);
    stbi__loadf_main(&raw mut s, x, y, comp, req_comp)
}
pub unsafe fn stbi_loadf(
    filename: *const ::core::ffi::c_char,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_float {
    let mut result: *mut ::core::ffi::c_float = ::core::ptr::null_mut::<::core::ffi::c_float>();
    let f: *mut FILE = stbi__fopen(filename, b"rb\0" as *const u8 as *const ::core::ffi::c_char);
    if f.is_null() {
        return (if 0 as ::core::ffi::c_int != 0 {
            NULL_0
        } else {
            NULL_0
        }) as size_t as *mut ::core::ffi::c_float;
    }
    result = stbi_loadf_from_file(f, x, y, comp, req_comp);
    fclose(f);
    result
}
pub unsafe fn stbi_loadf_from_file(
    f: *mut FILE,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_float {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_file(&raw mut s, f);
    stbi__loadf_main(&raw mut s, x, y, comp, req_comp)
}
pub fn stbi_is_hdr_from_memory(
    buffer: *const stbi_uc,
    len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    0 as ::core::ffi::c_int
}
pub unsafe fn stbi_is_hdr(filename: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let f: *mut FILE = stbi__fopen(filename, b"rb\0" as *const u8 as *const ::core::ffi::c_char);
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !f.is_null() {
        result = stbi_is_hdr_from_file(f);
        fclose(f);
    }
    result
}
pub fn stbi_is_hdr_from_file(f: *mut FILE) -> ::core::ffi::c_int {
    0 as ::core::ffi::c_int
}
pub fn stbi_is_hdr_from_callbacks(
    clbk: *const stbi_io_callbacks,
    user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    0 as ::core::ffi::c_int
}
static mut stbi__l2h_scale: ::core::ffi::c_float = 1.0f32;
static mut stbi__l2h_gamma: ::core::ffi::c_float = 2.2f32;
pub unsafe fn stbi_ldr_to_hdr_gamma(gamma: ::core::ffi::c_float) {
    stbi__l2h_gamma = gamma;
}
pub unsafe fn stbi_ldr_to_hdr_scale(scale: ::core::ffi::c_float) {
    stbi__l2h_scale = scale;
}
static mut stbi__h2l_scale_i: ::core::ffi::c_float = 1.0f32;
static mut stbi__h2l_gamma_i: ::core::ffi::c_float = 1.0f32 / 2.2f32;
pub unsafe fn stbi_hdr_to_ldr_gamma(gamma: ::core::ffi::c_float) {
    stbi__h2l_gamma_i = 1 as ::core::ffi::c_int as ::core::ffi::c_float / gamma;
}
pub unsafe fn stbi_hdr_to_ldr_scale(scale: ::core::ffi::c_float) {
    stbi__h2l_scale_i = 1 as ::core::ffi::c_int as ::core::ffi::c_float / scale;
}
unsafe fn stbi__refill_buffer(s: *mut stbi__context) {
    let n: ::core::ffi::c_int = (*s).io.read.expect("non-null function pointer")(
        (*s).io_user_data,
        &raw mut (*s).buffer_start as *mut stbi_uc as *mut ::core::ffi::c_char,
        (*s).buflen,
    );
    (*s).callback_already_read += (*s).img_buffer.offset_from((*s).img_buffer_original)
        as ::core::ffi::c_long as ::core::ffi::c_int;
    if n == 0 as ::core::ffi::c_int {
        (*s).read_from_callbacks = 0 as ::core::ffi::c_int;
        (*s).img_buffer = &raw mut (*s).buffer_start as *mut stbi_uc;
        (*s).img_buffer_end = (&raw mut (*s).buffer_start as *mut stbi_uc).offset(1);
        *(*s).img_buffer = 0 as stbi_uc;
    } else {
        (*s).img_buffer = &raw mut (*s).buffer_start as *mut stbi_uc;
        (*s).img_buffer_end = (&raw mut (*s).buffer_start as *mut stbi_uc).offset(n as isize);
    };
}
unsafe fn stbi__get8(s: *mut stbi__context) -> stbi_uc {
    if (*s).img_buffer < (*s).img_buffer_end {
        let fresh17 = (*s).img_buffer;
        (*s).img_buffer = (*s).img_buffer.offset(1);
        return *fresh17;
    }
    if (*s).read_from_callbacks != 0 {
        stbi__refill_buffer(s);
        let fresh18 = (*s).img_buffer;
        (*s).img_buffer = (*s).img_buffer.offset(1);
        return *fresh18;
    }
    0 as stbi_uc
}
unsafe fn stbi__skip(s: *mut stbi__context, n: ::core::ffi::c_int) {
    if n == 0 as ::core::ffi::c_int {
        return;
    }
    if n < 0 as ::core::ffi::c_int {
        (*s).img_buffer = (*s).img_buffer_end;
        return;
    }
    if (*s).io.read.is_some() {
        let blen: ::core::ffi::c_int = (*s).img_buffer_end.offset_from((*s).img_buffer)
            as ::core::ffi::c_long as ::core::ffi::c_int;
        if blen < n {
            (*s).img_buffer = (*s).img_buffer_end;
            (*s).io.skip.expect("non-null function pointer")((*s).io_user_data, n - blen);
            return;
        }
    }
    (*s).img_buffer = (*s).img_buffer.offset(n as isize);
}
unsafe fn stbi__getn(
    s: *mut stbi__context,
    buffer: *mut stbi_uc,
    n: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*s).io.read.is_some() {
        let blen: ::core::ffi::c_int = (*s).img_buffer_end.offset_from((*s).img_buffer)
            as ::core::ffi::c_long as ::core::ffi::c_int;
        if blen < n {
            let mut res: ::core::ffi::c_int = 0;
            let mut count: ::core::ffi::c_int = 0;
            memcpy(
                buffer as *mut ::core::ffi::c_void,
                (*s).img_buffer as *const ::core::ffi::c_void,
                blen as size_t,
            );
            count = (*s).io.read.expect("non-null function pointer")(
                (*s).io_user_data,
                (buffer as *mut ::core::ffi::c_char).offset(blen as isize),
                n - blen,
            );
            res = (count == n - blen) as ::core::ffi::c_int;
            (*s).img_buffer = (*s).img_buffer_end;
            return res;
        }
    }
    if (*s).img_buffer.offset(n as isize) <= (*s).img_buffer_end {
        memcpy(
            buffer as *mut ::core::ffi::c_void,
            (*s).img_buffer as *const ::core::ffi::c_void,
            n as size_t,
        );
        (*s).img_buffer = (*s).img_buffer.offset(n as isize);
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }
}
unsafe fn stbi__get16be(s: *mut stbi__context) -> ::core::ffi::c_int {
    let z: ::core::ffi::c_int = stbi__get8(s) as ::core::ffi::c_int;
    (z << 8 as ::core::ffi::c_int) + stbi__get8(s) as ::core::ffi::c_int
}
unsafe fn stbi__get32be(s: *mut stbi__context) -> stbi__uint32 {
    let z: stbi__uint32 = stbi__get16be(s) as stbi__uint32;
    (z << 16 as ::core::ffi::c_int).wrapping_add(stbi__get16be(s) as stbi__uint32)
}
unsafe fn stbi__compute_y(
    r: ::core::ffi::c_int,
    g: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
) -> stbi_uc {
    (r * 77 as ::core::ffi::c_int + g * 150 as ::core::ffi::c_int + 29 as ::core::ffi::c_int * b
        >> 8 as ::core::ffi::c_int) as stbi_uc
}
unsafe fn stbi__convert_format(
    data: *mut ::core::ffi::c_uchar,
    img_n: ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
    x: ::core::ffi::c_uint,
    y: ::core::ffi::c_uint,
) -> *mut ::core::ffi::c_uchar {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut good: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    if req_comp == img_n {
        return data;
    }
    '_c2rust_label: {
        if req_comp >= 1 as ::core::ffi::c_int && req_comp <= 4 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"req_comp >= 1 && req_comp <= 4\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"./third_party/stb/stb_image.h\0" as *const u8
                    as *const ::core::ffi::c_char,
                1741 as ::core::ffi::c_uint,
                b"unsigned char *stbi__convert_format(unsigned char *, int, int, unsigned int, unsigned int)\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    good = stbi__malloc_mad3(
        req_comp,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ) as *mut ::core::ffi::c_uchar;
    if good.is_null() {
        free(data as *mut ::core::ffi::c_void);
        return (if 0 as ::core::ffi::c_int != 0 {
            NULL_0
        } else {
            NULL_0
        }) as size_t as *mut ::core::ffi::c_uchar;
    }
    j = 0 as ::core::ffi::c_int;
    while j < y as ::core::ffi::c_int {
        let mut src: *mut ::core::ffi::c_uchar = data.offset(
            (j as ::core::ffi::c_uint)
                .wrapping_mul(x)
                .wrapping_mul(img_n as ::core::ffi::c_uint) as isize,
        );
        let mut dest: *mut ::core::ffi::c_uchar = good.offset(
            (j as ::core::ffi::c_uint)
                .wrapping_mul(x)
                .wrapping_mul(req_comp as ::core::ffi::c_uint) as isize,
        );
        match img_n * 8 as ::core::ffi::c_int + req_comp {
            10 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = *src.offset(0);
                    *dest.offset(1) = 255 as ::core::ffi::c_uchar;
                    i -= 1;
                    src = src.offset(1);
                    dest = dest.offset(2);
                }
            }
            11 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    let ref mut fresh9 = *dest.offset(2);
                    *fresh9 = *src.offset(0);
                    let ref mut fresh10 = *dest.offset(1);
                    *fresh10 = *fresh9;
                    *dest.offset(0) = *fresh10;
                    i -= 1;
                    src = src.offset(1);
                    dest = dest.offset(3);
                }
            }
            12 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    let ref mut fresh11 = *dest.offset(2);
                    *fresh11 = *src.offset(0);
                    let ref mut fresh12 = *dest.offset(1);
                    *fresh12 = *fresh11;
                    *dest.offset(0) = *fresh12;
                    *dest.offset(3) = 255 as ::core::ffi::c_uchar;
                    i -= 1;
                    src = src.offset(1);
                    dest = dest.offset(4);
                }
            }
            17 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = *src.offset(0);
                    i -= 1;
                    src = src.offset(2);
                    dest = dest.offset(1);
                }
            }
            19 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    let ref mut fresh13 = *dest.offset(2);
                    *fresh13 = *src.offset(0);
                    let ref mut fresh14 = *dest.offset(1);
                    *fresh14 = *fresh13;
                    *dest.offset(0) = *fresh14;
                    i -= 1;
                    src = src.offset(2);
                    dest = dest.offset(3);
                }
            }
            20 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    let ref mut fresh15 = *dest.offset(2);
                    *fresh15 = *src.offset(0);
                    let ref mut fresh16 = *dest.offset(1);
                    *fresh16 = *fresh15;
                    *dest.offset(0) = *fresh16;
                    *dest.offset(3) = *src.offset(1);
                    i -= 1;
                    src = src.offset(2);
                    dest = dest.offset(4);
                }
            }
            28 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = *src.offset(0);
                    *dest.offset(1) = *src.offset(1);
                    *dest.offset(2) = *src.offset(2);
                    *dest.offset(3) = 255 as ::core::ffi::c_uchar;
                    i -= 1;
                    src = src.offset(3);
                    dest = dest.offset(4);
                }
            }
            25 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = stbi__compute_y(
                        *src.offset(0) as ::core::ffi::c_int,
                        *src.offset(1) as ::core::ffi::c_int,
                        *src.offset(2) as ::core::ffi::c_int,
                    ) as ::core::ffi::c_uchar;
                    i -= 1;
                    src = src.offset(3);
                    dest = dest.offset(1);
                }
            }
            26 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = stbi__compute_y(
                        *src.offset(0) as ::core::ffi::c_int,
                        *src.offset(1) as ::core::ffi::c_int,
                        *src.offset(2) as ::core::ffi::c_int,
                    ) as ::core::ffi::c_uchar;
                    *dest.offset(1) = 255 as ::core::ffi::c_uchar;
                    i -= 1;
                    src = src.offset(3);
                    dest = dest.offset(2);
                }
            }
            33 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = stbi__compute_y(
                        *src.offset(0) as ::core::ffi::c_int,
                        *src.offset(1) as ::core::ffi::c_int,
                        *src.offset(2) as ::core::ffi::c_int,
                    ) as ::core::ffi::c_uchar;
                    i -= 1;
                    src = src.offset(4);
                    dest = dest.offset(1);
                }
            }
            34 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = stbi__compute_y(
                        *src.offset(0) as ::core::ffi::c_int,
                        *src.offset(1) as ::core::ffi::c_int,
                        *src.offset(2) as ::core::ffi::c_int,
                    ) as ::core::ffi::c_uchar;
                    *dest.offset(1) = *src.offset(3);
                    i -= 1;
                    src = src.offset(4);
                    dest = dest.offset(2);
                }
            }
            35 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = *src.offset(0);
                    *dest.offset(1) = *src.offset(1);
                    *dest.offset(2) = *src.offset(2);
                    i -= 1;
                    src = src.offset(4);
                    dest = dest.offset(3);
                }
            }
            _ => {
                '_c2rust_label_0: {
                    __assert_fail(
                        b"0\0" as *const u8 as *const ::core::ffi::c_char,
                        b"./third_party/stb/stb_image.h\0" as *const u8
                            as *const ::core::ffi::c_char,
                        1770 as ::core::ffi::c_uint,
                        b"unsigned char *stbi__convert_format(unsigned char *, int, int, unsigned int, unsigned int)\0"
                            as *const u8 as *const ::core::ffi::c_char,
                    );
                };
                free(data as *mut ::core::ffi::c_void);
                free(good as *mut ::core::ffi::c_void);
                return (if 0 as ::core::ffi::c_int != 0 {
                    NULL_0
                } else {
                    NULL_0
                }) as size_t as *mut ::core::ffi::c_uchar;
            }
        }
        j += 1;
    }
    free(data as *mut ::core::ffi::c_void);
    good
}
unsafe fn stbi__compute_y_16(
    r: ::core::ffi::c_int,
    g: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
) -> stbi__uint16 {
    (r * 77 as ::core::ffi::c_int + g * 150 as ::core::ffi::c_int + 29 as ::core::ffi::c_int * b
        >> 8 as ::core::ffi::c_int) as stbi__uint16
}
unsafe fn stbi__convert_format16(
    data: *mut stbi__uint16,
    img_n: ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
    x: ::core::ffi::c_uint,
    y: ::core::ffi::c_uint,
) -> *mut stbi__uint16 {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut good: *mut stbi__uint16 = ::core::ptr::null_mut::<stbi__uint16>();
    if req_comp == img_n {
        return data;
    }
    '_c2rust_label: {
        if req_comp >= 1 as ::core::ffi::c_int && req_comp <= 4 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"req_comp >= 1 && req_comp <= 4\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"./third_party/stb/stb_image.h\0" as *const u8
                    as *const ::core::ffi::c_char,
                1798 as ::core::ffi::c_uint,
                b"stbi__uint16 *stbi__convert_format16(stbi__uint16 *, int, int, unsigned int, unsigned int)\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    good = stbi__malloc(
        (req_comp as ::core::ffi::c_uint)
            .wrapping_mul(x)
            .wrapping_mul(y)
            .wrapping_mul(2 as ::core::ffi::c_uint) as size_t,
    ) as *mut stbi__uint16;
    if good.is_null() {
        free(data as *mut ::core::ffi::c_void);
        return (if 0 as ::core::ffi::c_int != 0 {
            NULL_0
        } else {
            NULL_0
        }) as size_t as *mut ::core::ffi::c_uchar as *mut stbi__uint16;
    }
    j = 0 as ::core::ffi::c_int;
    while j < y as ::core::ffi::c_int {
        let mut src: *mut stbi__uint16 = data.offset(
            (j as ::core::ffi::c_uint)
                .wrapping_mul(x)
                .wrapping_mul(img_n as ::core::ffi::c_uint) as isize,
        );
        let mut dest: *mut stbi__uint16 = good.offset(
            (j as ::core::ffi::c_uint)
                .wrapping_mul(x)
                .wrapping_mul(req_comp as ::core::ffi::c_uint) as isize,
        );
        match img_n * 8 as ::core::ffi::c_int + req_comp {
            10 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = *src.offset(0);
                    *dest.offset(1) = 0xffff as stbi__uint16;
                    i -= 1;
                    src = src.offset(1);
                    dest = dest.offset(2);
                }
            }
            11 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    let ref mut fresh1 = *dest.offset(2);
                    *fresh1 = *src.offset(0);
                    let ref mut fresh2 = *dest.offset(1);
                    *fresh2 = *fresh1;
                    *dest.offset(0) = *fresh2;
                    i -= 1;
                    src = src.offset(1);
                    dest = dest.offset(3);
                }
            }
            12 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    let ref mut fresh3 = *dest.offset(2);
                    *fresh3 = *src.offset(0);
                    let ref mut fresh4 = *dest.offset(1);
                    *fresh4 = *fresh3;
                    *dest.offset(0) = *fresh4;
                    *dest.offset(3) = 0xffff as stbi__uint16;
                    i -= 1;
                    src = src.offset(1);
                    dest = dest.offset(4);
                }
            }
            17 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = *src.offset(0);
                    i -= 1;
                    src = src.offset(2);
                    dest = dest.offset(1);
                }
            }
            19 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    let ref mut fresh5 = *dest.offset(2);
                    *fresh5 = *src.offset(0);
                    let ref mut fresh6 = *dest.offset(1);
                    *fresh6 = *fresh5;
                    *dest.offset(0) = *fresh6;
                    i -= 1;
                    src = src.offset(2);
                    dest = dest.offset(3);
                }
            }
            20 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    let ref mut fresh7 = *dest.offset(2);
                    *fresh7 = *src.offset(0);
                    let ref mut fresh8 = *dest.offset(1);
                    *fresh8 = *fresh7;
                    *dest.offset(0) = *fresh8;
                    *dest.offset(3) = *src.offset(1);
                    i -= 1;
                    src = src.offset(2);
                    dest = dest.offset(4);
                }
            }
            28 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = *src.offset(0);
                    *dest.offset(1) = *src.offset(1);
                    *dest.offset(2) = *src.offset(2);
                    *dest.offset(3) = 0xffff as stbi__uint16;
                    i -= 1;
                    src = src.offset(3);
                    dest = dest.offset(4);
                }
            }
            25 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = stbi__compute_y_16(
                        *src.offset(0) as ::core::ffi::c_int,
                        *src.offset(1) as ::core::ffi::c_int,
                        *src.offset(2) as ::core::ffi::c_int,
                    );
                    i -= 1;
                    src = src.offset(3);
                    dest = dest.offset(1);
                }
            }
            26 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = stbi__compute_y_16(
                        *src.offset(0) as ::core::ffi::c_int,
                        *src.offset(1) as ::core::ffi::c_int,
                        *src.offset(2) as ::core::ffi::c_int,
                    );
                    *dest.offset(1) = 0xffff as stbi__uint16;
                    i -= 1;
                    src = src.offset(3);
                    dest = dest.offset(2);
                }
            }
            33 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = stbi__compute_y_16(
                        *src.offset(0) as ::core::ffi::c_int,
                        *src.offset(1) as ::core::ffi::c_int,
                        *src.offset(2) as ::core::ffi::c_int,
                    );
                    i -= 1;
                    src = src.offset(4);
                    dest = dest.offset(1);
                }
            }
            34 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = stbi__compute_y_16(
                        *src.offset(0) as ::core::ffi::c_int,
                        *src.offset(1) as ::core::ffi::c_int,
                        *src.offset(2) as ::core::ffi::c_int,
                    );
                    *dest.offset(1) = *src.offset(3);
                    i -= 1;
                    src = src.offset(4);
                    dest = dest.offset(2);
                }
            }
            35 => {
                i = x.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *dest.offset(0) = *src.offset(0);
                    *dest.offset(1) = *src.offset(1);
                    *dest.offset(2) = *src.offset(2);
                    i -= 1;
                    src = src.offset(4);
                    dest = dest.offset(3);
                }
            }
            _ => {
                '_c2rust_label_0: {
                    __assert_fail(
                        b"0\0" as *const u8 as *const ::core::ffi::c_char,
                        b"./third_party/stb/stb_image.h\0" as *const u8
                            as *const ::core::ffi::c_char,
                        1827 as ::core::ffi::c_uint,
                        b"stbi__uint16 *stbi__convert_format16(stbi__uint16 *, int, int, unsigned int, unsigned int)\0"
                            as *const u8 as *const ::core::ffi::c_char,
                    );
                };
                free(data as *mut ::core::ffi::c_void);
                free(good as *mut ::core::ffi::c_void);
                return (if 0 as ::core::ffi::c_int != 0 {
                    NULL_0
                } else {
                    NULL_0
                }) as size_t as *mut ::core::ffi::c_uchar
                    as *mut stbi__uint16;
            }
        }
        j += 1;
    }
    free(data as *mut ::core::ffi::c_void);
    good
}
unsafe fn stbi__ldr_to_hdr(
    data: *mut stbi_uc,
    x: ::core::ffi::c_int,
    y: ::core::ffi::c_int,
    comp: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_float {
    let mut i: ::core::ffi::c_int = 0;
    let mut k: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut output: *mut ::core::ffi::c_float = ::core::ptr::null_mut::<::core::ffi::c_float>();
    if data.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_float>();
    }
    output = stbi__malloc_mad4(
        x,
        y,
        comp,
        ::core::mem::size_of::<::core::ffi::c_float>() as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ) as *mut ::core::ffi::c_float;
    if output.is_null() {
        free(data as *mut ::core::ffi::c_void);
        return (if 0 as ::core::ffi::c_int != 0 {
            NULL_0
        } else {
            NULL_0
        }) as size_t as *mut ::core::ffi::c_float;
    }
    if comp & 1 as ::core::ffi::c_int != 0 {
        n = comp;
    } else {
        n = comp - 1 as ::core::ffi::c_int;
    }
    i = 0 as ::core::ffi::c_int;
    while i < x * y {
        k = 0 as ::core::ffi::c_int;
        while k < n {
            *output.offset((i * comp + k) as isize) = (pow(
                (*data.offset((i * comp + k) as isize) as ::core::ffi::c_int
                    as ::core::ffi::c_float
                    / 255.0f32) as ::core::ffi::c_double,
                stbi__l2h_gamma as ::core::ffi::c_double,
            ) * stbi__l2h_scale as ::core::ffi::c_double)
                as ::core::ffi::c_float;
            k += 1;
        }
        i += 1;
    }
    if n < comp {
        i = 0 as ::core::ffi::c_int;
        while i < x * y {
            *output.offset((i * comp + n) as isize) =
                *data.offset((i * comp + n) as isize) as ::core::ffi::c_int as ::core::ffi::c_float
                    / 255.0f32;
            i += 1;
        }
    }
    free(data as *mut ::core::ffi::c_void);
    output
}
pub const STBI__ZFAST_BITS: ::core::ffi::c_int = 9;
pub const STBI__ZFAST_MASK: ::core::ffi::c_int =
    ((1 as ::core::ffi::c_int) << STBI__ZFAST_BITS) - 1 as ::core::ffi::c_int;
pub const STBI__ZNSYMS: ::core::ffi::c_int = 288;
unsafe fn stbi__bitreverse16(mut n: ::core::ffi::c_int) -> ::core::ffi::c_int {
    n = (n & 0xaaaa as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int
        | (n & 0x5555 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
    n = (n & 0xcccc as ::core::ffi::c_int) >> 2 as ::core::ffi::c_int
        | (n & 0x3333 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
    n = (n & 0xf0f0 as ::core::ffi::c_int) >> 4 as ::core::ffi::c_int
        | (n & 0xf0f as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
    n = (n & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
        | (n & 0xff as ::core::ffi::c_int) << 8 as ::core::ffi::c_int;
    n
}
unsafe fn stbi__bit_reverse(v: ::core::ffi::c_int, bits: ::core::ffi::c_int) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if bits <= 16 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"bits <= 16\0" as *const u8 as *const ::core::ffi::c_char,
                b"./third_party/stb/stb_image.h\0" as *const u8 as *const ::core::ffi::c_char,
                4061 as ::core::ffi::c_uint,
                b"int stbi__bit_reverse(int, int)\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    stbi__bitreverse16(v) >> 16 as ::core::ffi::c_int - bits
}
unsafe fn stbi__zbuild_huffman(
    z: *mut stbi__zhuffman,
    sizelist: *const stbi_uc,
    num: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut code: ::core::ffi::c_int = 0;
    let mut next_code: [::core::ffi::c_int; 16] = [0; 16];
    let mut sizes: [::core::ffi::c_int; 17] = [0; 17];
    memset(
        &raw mut sizes as *mut ::core::ffi::c_int as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[::core::ffi::c_int; 17]>() as size_t,
    );
    memset(
        &raw mut (*z).fast as *mut stbi__uint16 as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[stbi__uint16; 512]>() as size_t,
    );
    i = 0 as ::core::ffi::c_int;
    while i < num {
        sizes[*sizelist.offset(i as isize) as usize] += 1;
        i += 1;
    }
    sizes[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    i = 1 as ::core::ffi::c_int;
    while i < 16 as ::core::ffi::c_int {
        if sizes[i as usize] > (1 as ::core::ffi::c_int) << i {
            return 0 as ::core::ffi::c_int;
        }
        i += 1;
    }
    code = 0 as ::core::ffi::c_int;
    i = 1 as ::core::ffi::c_int;
    while i < 16 as ::core::ffi::c_int {
        next_code[i as usize] = code;
        (*z).firstcode[i as usize] = code as stbi__uint16;
        (*z).firstsymbol[i as usize] = k as stbi__uint16;
        code = code + sizes[i as usize];
        if sizes[i as usize] != 0 {
            if code - 1 as ::core::ffi::c_int >= (1 as ::core::ffi::c_int) << i {
                return 0 as ::core::ffi::c_int;
            }
        }
        (*z).maxcode[i as usize] = code << 16 as ::core::ffi::c_int - i;
        code <<= 1 as ::core::ffi::c_int;
        k += sizes[i as usize];
        i += 1;
    }
    (*z).maxcode[16 as ::core::ffi::c_int as usize] = 0x10000 as ::core::ffi::c_int;
    i = 0 as ::core::ffi::c_int;
    while i < num {
        let s: ::core::ffi::c_int = *sizelist.offset(i as isize) as ::core::ffi::c_int;
        if s != 0 {
            let c: ::core::ffi::c_int = next_code[s as usize]
                - (*z).firstcode[s as usize] as ::core::ffi::c_int
                + (*z).firstsymbol[s as usize] as ::core::ffi::c_int;
            let fastv: stbi__uint16 = (s << 9 as ::core::ffi::c_int | i) as stbi__uint16;
            (*z).size[c as usize] = s as stbi_uc;
            (*z).value[c as usize] = i as stbi__uint16;
            if s <= STBI__ZFAST_BITS {
                let mut j: ::core::ffi::c_int = stbi__bit_reverse(next_code[s as usize], s);
                while j < (1 as ::core::ffi::c_int) << STBI__ZFAST_BITS {
                    (*z).fast[j as usize] = fastv;
                    j += (1 as ::core::ffi::c_int) << s;
                }
            }
            next_code[s as usize] += 1;
        }
        i += 1;
    }
    1 as ::core::ffi::c_int
}
unsafe fn stbi__zeof(z: *mut stbi__zbuf) -> ::core::ffi::c_int {
    ((*z).zbuffer >= (*z).zbuffer_end) as ::core::ffi::c_int
}
unsafe fn stbi__zget8(z: *mut stbi__zbuf) -> stbi_uc {
    ((if stbi__zeof(z) != 0 {
        0 as ::core::ffi::c_int
    } else {
        let fresh49 = (*z).zbuffer;
        (*z).zbuffer = (*z).zbuffer.offset(1);
        *fresh49 as ::core::ffi::c_int
    }) as stbi_uc)
}
unsafe fn stbi__fill_bits(z: *mut stbi__zbuf) {
    loop {
        if (*z).code_buffer >= (1 as stbi__uint32) << (*z).num_bits {
            (*z).zbuffer = (*z).zbuffer_end;
            return;
        }
        (*z).code_buffer = ((*z).code_buffer as ::core::ffi::c_uint
            | (stbi__zget8(z) as ::core::ffi::c_uint) << (*z).num_bits)
            as stbi__uint32;
        (*z).num_bits += 8 as ::core::ffi::c_int;
        if ((*z).num_bits > 24 as ::core::ffi::c_int) {
            break;
        }
    }
}
unsafe fn stbi__zreceive(z: *mut stbi__zbuf, n: ::core::ffi::c_int) -> ::core::ffi::c_uint {
    let mut k: ::core::ffi::c_uint = 0;
    if (*z).num_bits < n {
        stbi__fill_bits(z);
    }
    k = ((*z).code_buffer
        & (((1 as ::core::ffi::c_int) << n) - 1 as ::core::ffi::c_int) as stbi__uint32)
        as ::core::ffi::c_uint;
    (*z).code_buffer >>= n;
    (*z).num_bits -= n;
    k
}
unsafe fn stbi__zhuffman_decode_slowpath(
    a: *mut stbi__zbuf,
    z: *mut stbi__zhuffman,
) -> ::core::ffi::c_int {
    let mut b: ::core::ffi::c_int = 0;
    let mut s: ::core::ffi::c_int = 0;
    let mut k: ::core::ffi::c_int = 0;
    k = stbi__bit_reverse(
        (*a).code_buffer as ::core::ffi::c_int,
        16 as ::core::ffi::c_int,
    );
    s = STBI__ZFAST_BITS + 1 as ::core::ffi::c_int;
    while (k >= (*z).maxcode[s as usize]) {
        s += 1;
    }
    if s >= 16 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    b = (k >> 16 as ::core::ffi::c_int - s) - (*z).firstcode[s as usize] as ::core::ffi::c_int
        + (*z).firstsymbol[s as usize] as ::core::ffi::c_int;
    if b >= STBI__ZNSYMS {
        return -(1 as ::core::ffi::c_int);
    }
    if (*z).size[b as usize] as ::core::ffi::c_int != s {
        return -(1 as ::core::ffi::c_int);
    }
    (*a).code_buffer >>= s;
    (*a).num_bits -= s;
    (*z).value[b as usize] as ::core::ffi::c_int
}
unsafe fn stbi__zhuffman_decode(a: *mut stbi__zbuf, z: *mut stbi__zhuffman) -> ::core::ffi::c_int {
    let mut b: ::core::ffi::c_int = 0;
    let mut s: ::core::ffi::c_int = 0;
    if (*a).num_bits < 16 as ::core::ffi::c_int {
        if stbi__zeof(a) != 0 {
            return -(1 as ::core::ffi::c_int);
        }
        stbi__fill_bits(a);
    }
    b = (*z).fast[((*a).code_buffer & STBI__ZFAST_MASK as stbi__uint32) as usize]
        as ::core::ffi::c_int;
    if b != 0 {
        s = b >> 9 as ::core::ffi::c_int;
        (*a).code_buffer >>= s;
        (*a).num_bits -= s;
        return b & 511 as ::core::ffi::c_int;
    }
    stbi__zhuffman_decode_slowpath(a, z)
}
unsafe fn stbi__zexpand(
    z: *mut stbi__zbuf,
    zout: *mut ::core::ffi::c_char,
    n: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut q: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cur: ::core::ffi::c_uint = 0;
    let mut limit: ::core::ffi::c_uint = 0;
    let mut old_limit: ::core::ffi::c_uint = 0;
    (*z).zout = zout;
    if (*z).z_expandable == 0 {
        return 0 as ::core::ffi::c_int;
    }
    cur = (*z).zout.offset_from((*z).zout_start) as ::core::ffi::c_long as ::core::ffi::c_uint;
    old_limit =
        (*z).zout_end.offset_from((*z).zout_start) as ::core::ffi::c_long as ::core::ffi::c_uint;
    limit = old_limit;
    if UINT_MAX.wrapping_sub(cur) < n as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    while cur.wrapping_add(n as ::core::ffi::c_uint) > limit {
        if limit > UINT_MAX.wrapping_div(2 as ::core::ffi::c_uint) {
            return 0 as ::core::ffi::c_int;
        }
        limit = limit.wrapping_mul(2 as ::core::ffi::c_uint);
    }
    q = realloc((*z).zout_start as *mut ::core::ffi::c_void, limit as size_t)
        as *mut ::core::ffi::c_char;
    if q.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    (*z).zout_start = q;
    (*z).zout = q.offset(cur as isize);
    (*z).zout_end = q.offset(limit as isize);
    1 as ::core::ffi::c_int
}
static mut stbi__zlength_base: [::core::ffi::c_int; 31] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258, 0, 0,
];
static mut stbi__zlength_extra: [::core::ffi::c_int; 31] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 0, 0,
];
static mut stbi__zdist_base: [::core::ffi::c_int; 32] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577, 0, 0,
];
static mut stbi__zdist_extra: [::core::ffi::c_int; 32] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13, 0, 0,
];
unsafe fn stbi__parse_huffman_block(a: *mut stbi__zbuf) -> ::core::ffi::c_int {
    let mut zout: *mut ::core::ffi::c_char = (*a).zout;
    loop {
        let mut z: ::core::ffi::c_int = stbi__zhuffman_decode(a, &raw mut (*a).z_length);
        if z < 256 as ::core::ffi::c_int {
            if z < 0 as ::core::ffi::c_int {
                return 0 as ::core::ffi::c_int;
            }
            if zout >= (*a).zout_end {
                if stbi__zexpand(a, zout, 1 as ::core::ffi::c_int) == 0 {
                    return 0 as ::core::ffi::c_int;
                }
                zout = (*a).zout;
            }
            let fresh45 = zout;
            zout = zout.offset(1);
            *fresh45 = z as ::core::ffi::c_char;
        } else {
            let mut p: *mut stbi_uc = ::core::ptr::null_mut::<stbi_uc>();
            let mut len: ::core::ffi::c_int = 0;
            let mut dist: ::core::ffi::c_int = 0;
            if z == 256 as ::core::ffi::c_int {
                (*a).zout = zout;
                return 1 as ::core::ffi::c_int;
            }
            z -= 257 as ::core::ffi::c_int;
            len = stbi__zlength_base[z as usize];
            if stbi__zlength_extra[z as usize] != 0 {
                len = (len as ::core::ffi::c_uint)
                    .wrapping_add(stbi__zreceive(a, stbi__zlength_extra[z as usize]))
                    as ::core::ffi::c_int as ::core::ffi::c_int;
            }
            z = stbi__zhuffman_decode(a, &raw mut (*a).z_distance);
            if z < 0 as ::core::ffi::c_int {
                return 0 as ::core::ffi::c_int;
            }
            dist = stbi__zdist_base[z as usize];
            if stbi__zdist_extra[z as usize] != 0 {
                dist = (dist as ::core::ffi::c_uint)
                    .wrapping_add(stbi__zreceive(a, stbi__zdist_extra[z as usize]))
                    as ::core::ffi::c_int as ::core::ffi::c_int;
            }
            if (zout.offset_from((*a).zout_start) as ::core::ffi::c_long)
                < dist as ::core::ffi::c_long
            {
                return 0 as ::core::ffi::c_int;
            }
            if zout.offset(len as isize) > (*a).zout_end {
                if stbi__zexpand(a, zout, len) == 0 {
                    return 0 as ::core::ffi::c_int;
                }
                zout = (*a).zout;
            }
            p = zout.offset(-(dist as isize)) as *mut stbi_uc;
            if dist == 1 as ::core::ffi::c_int {
                let v: stbi_uc = *p;
                if len != 0 {
                    loop {
                        let fresh46 = zout;
                        zout = zout.offset(1);
                        *fresh46 = v as ::core::ffi::c_char;
                        len -= 1;
                        if (len == 0) {
                            break;
                        }
                    }
                }
            } else if len != 0 {
                loop {
                    let fresh47 = p;
                    p = p.offset(1);
                    let fresh48 = zout;
                    zout = zout.offset(1);
                    *fresh48 = *fresh47 as ::core::ffi::c_char;
                    len -= 1;
                    if (len == 0) {
                        break;
                    }
                }
            }
        }
    }
}
unsafe fn stbi__compute_huffman_codes(a: *mut stbi__zbuf) -> ::core::ffi::c_int {
    static mut length_dezigzag: [stbi_uc; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let mut z_codelength: stbi__zhuffman = stbi__zhuffman::EMPTY;
    let mut lencodes: [stbi_uc; 455] = [0; 455];
    let mut codelength_sizes: [stbi_uc; 19] = [0; 19];
    let mut i: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let hlit: ::core::ffi::c_int = stbi__zreceive(a, 5 as ::core::ffi::c_int)
        .wrapping_add(257 as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    let hdist: ::core::ffi::c_int = stbi__zreceive(a, 5 as ::core::ffi::c_int)
        .wrapping_add(1 as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    let hclen: ::core::ffi::c_int = stbi__zreceive(a, 4 as ::core::ffi::c_int)
        .wrapping_add(4 as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    let ntot: ::core::ffi::c_int = hlit + hdist;
    memset(
        &raw mut codelength_sizes as *mut stbi_uc as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[stbi_uc; 19]>() as size_t,
    );
    i = 0 as ::core::ffi::c_int;
    while i < hclen {
        let s: ::core::ffi::c_int =
            stbi__zreceive(a, 3 as ::core::ffi::c_int) as ::core::ffi::c_int;
        codelength_sizes[length_dezigzag[i as usize] as usize] = s as stbi_uc;
        i += 1;
    }
    if stbi__zbuild_huffman(
        &raw mut z_codelength,
        &raw mut codelength_sizes as *mut stbi_uc,
        19 as ::core::ffi::c_int,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    n = 0 as ::core::ffi::c_int;
    while n < ntot {
        let mut c: ::core::ffi::c_int = stbi__zhuffman_decode(a, &raw mut z_codelength);
        if c < 0 as ::core::ffi::c_int || c >= 19 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if c < 16 as ::core::ffi::c_int {
            let fresh50 = n;
            n = n + 1;
            lencodes[fresh50 as usize] = c as stbi_uc;
        } else {
            let mut fill: stbi_uc = 0 as stbi_uc;
            if c == 16 as ::core::ffi::c_int {
                c = stbi__zreceive(a, 2 as ::core::ffi::c_int)
                    .wrapping_add(3 as ::core::ffi::c_uint)
                    as ::core::ffi::c_int;
                if n == 0 as ::core::ffi::c_int {
                    return 0 as ::core::ffi::c_int;
                }
                fill = lencodes[(n - 1 as ::core::ffi::c_int) as usize];
            } else if c == 17 as ::core::ffi::c_int {
                c = stbi__zreceive(a, 3 as ::core::ffi::c_int)
                    .wrapping_add(3 as ::core::ffi::c_uint)
                    as ::core::ffi::c_int;
            } else if c == 18 as ::core::ffi::c_int {
                c = stbi__zreceive(a, 7 as ::core::ffi::c_int)
                    .wrapping_add(11 as ::core::ffi::c_uint)
                    as ::core::ffi::c_int;
            } else {
                return 0 as ::core::ffi::c_int;
            }
            if ntot - n < c {
                return 0 as ::core::ffi::c_int;
            }
            memset(
                (&raw mut lencodes as *mut stbi_uc).offset(n as isize) as *mut ::core::ffi::c_void,
                fill as ::core::ffi::c_int,
                c as size_t,
            );
            n += c;
        }
    }
    if n != ntot {
        return 0 as ::core::ffi::c_int;
    }
    if stbi__zbuild_huffman(
        &raw mut (*a).z_length,
        &raw mut lencodes as *mut stbi_uc,
        hlit,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    if stbi__zbuild_huffman(
        &raw mut (*a).z_distance,
        (&raw mut lencodes as *mut stbi_uc).offset(hlit as isize),
        hdist,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    1 as ::core::ffi::c_int
}
unsafe fn stbi__parse_uncompressed_block(a: *mut stbi__zbuf) -> ::core::ffi::c_int {
    let mut header: [stbi_uc; 4] = [0; 4];
    let mut len: ::core::ffi::c_int = 0;
    let mut nlen: ::core::ffi::c_int = 0;
    let mut k: ::core::ffi::c_int = 0;
    if (*a).num_bits & 7 as ::core::ffi::c_int != 0 {
        stbi__zreceive(a, (*a).num_bits & 7 as ::core::ffi::c_int);
    }
    k = 0 as ::core::ffi::c_int;
    while (*a).num_bits > 0 as ::core::ffi::c_int {
        let fresh51 = k;
        k = k + 1;
        header[fresh51 as usize] = ((*a).code_buffer & 255 as stbi__uint32) as stbi_uc;
        (*a).code_buffer >>= 8 as ::core::ffi::c_int;
        (*a).num_bits -= 8 as ::core::ffi::c_int;
    }
    if (*a).num_bits < 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    while k < 4 as ::core::ffi::c_int {
        let fresh52 = k;
        k = k + 1;
        header[fresh52 as usize] = stbi__zget8(a);
    }
    len = header[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        * 256 as ::core::ffi::c_int
        + header[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int;
    nlen = header[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        * 256 as ::core::ffi::c_int
        + header[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int;
    if nlen != len ^ 0xffff as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*a).zbuffer.offset(len as isize) > (*a).zbuffer_end {
        return 0 as ::core::ffi::c_int;
    }
    if (*a).zout.offset(len as isize) > (*a).zout_end {
        if stbi__zexpand(a, (*a).zout, len) == 0 {
            return 0 as ::core::ffi::c_int;
        }
    }
    memcpy(
        (*a).zout as *mut ::core::ffi::c_void,
        (*a).zbuffer as *const ::core::ffi::c_void,
        len as size_t,
    );
    (*a).zbuffer = (*a).zbuffer.offset(len as isize);
    (*a).zout = (*a).zout.offset(len as isize);
    1 as ::core::ffi::c_int
}
unsafe fn stbi__parse_zlib_header(a: *mut stbi__zbuf) -> ::core::ffi::c_int {
    let cmf: ::core::ffi::c_int = stbi__zget8(a) as ::core::ffi::c_int;
    let cm: ::core::ffi::c_int = cmf & 15 as ::core::ffi::c_int;
    let flg: ::core::ffi::c_int = stbi__zget8(a) as ::core::ffi::c_int;
    if stbi__zeof(a) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (cmf * 256 as ::core::ffi::c_int + flg) % 31 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if flg & 32 as ::core::ffi::c_int != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if cm != 8 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    1 as ::core::ffi::c_int
}
static mut stbi__zdefault_length: [stbi_uc; 288] = [
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8,
];
static mut stbi__zdefault_distance: [stbi_uc; 32] = [
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
];
unsafe fn stbi__parse_zlib(
    a: *mut stbi__zbuf,
    parse_header: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut final_0: ::core::ffi::c_int = 0;
    let mut type_0: ::core::ffi::c_int = 0;
    if parse_header != 0 {
        if stbi__parse_zlib_header(a) == 0 {
            return 0 as ::core::ffi::c_int;
        }
    }
    (*a).num_bits = 0 as ::core::ffi::c_int;
    (*a).code_buffer = 0 as stbi__uint32;
    loop {
        final_0 = stbi__zreceive(a, 1 as ::core::ffi::c_int) as ::core::ffi::c_int;
        type_0 = stbi__zreceive(a, 2 as ::core::ffi::c_int) as ::core::ffi::c_int;
        if type_0 == 0 as ::core::ffi::c_int {
            if stbi__parse_uncompressed_block(a) == 0 {
                return 0 as ::core::ffi::c_int;
            }
        } else if type_0 == 3 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        } else {
            if type_0 == 1 as ::core::ffi::c_int {
                if stbi__zbuild_huffman(
                    &raw mut (*a).z_length,
                    &raw const stbi__zdefault_length as *const stbi_uc,
                    STBI__ZNSYMS,
                ) == 0
                {
                    return 0 as ::core::ffi::c_int;
                }
                if stbi__zbuild_huffman(
                    &raw mut (*a).z_distance,
                    &raw const stbi__zdefault_distance as *const stbi_uc,
                    32 as ::core::ffi::c_int,
                ) == 0
                {
                    return 0 as ::core::ffi::c_int;
                }
            } else if stbi__compute_huffman_codes(a) == 0 {
                return 0 as ::core::ffi::c_int;
            }
            if stbi__parse_huffman_block(a) == 0 {
                return 0 as ::core::ffi::c_int;
            }
        }
        if (final_0 != 0) {
            break;
        }
    }
    1 as ::core::ffi::c_int
}
unsafe fn stbi__do_zlib(
    a: *mut stbi__zbuf,
    obuf: *mut ::core::ffi::c_char,
    olen: ::core::ffi::c_int,
    exp: ::core::ffi::c_int,
    parse_header: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    (*a).zout_start = obuf;
    (*a).zout = obuf;
    (*a).zout_end = obuf.offset(olen as isize);
    (*a).z_expandable = exp;
    stbi__parse_zlib(a, parse_header)
}
pub unsafe fn stbi_zlib_decode_malloc_guesssize(
    buffer: *const ::core::ffi::c_char,
    len: ::core::ffi::c_int,
    initial_size: ::core::ffi::c_int,
    outlen: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    stbi_zlib_decode_malloc_guesssize_headerflag(buffer, len, initial_size, outlen, 1)
}
pub unsafe fn stbi_zlib_decode_malloc(
    buffer: *const ::core::ffi::c_char,
    len: ::core::ffi::c_int,
    outlen: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    stbi_zlib_decode_malloc_guesssize(buffer, len, 16384 as ::core::ffi::c_int, outlen)
}
pub unsafe fn stbi_zlib_decode_malloc_guesssize_headerflag(
    buffer: *const ::core::ffi::c_char,
    len: ::core::ffi::c_int,
    initial_size: ::core::ffi::c_int,
    outlen: *mut ::core::ffi::c_int,
    parse_header: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut a: stbi__zbuf = stbi__zbuf::EMPTY;
    let p: *mut ::core::ffi::c_char =
        stbi__malloc(initial_size as size_t) as *mut ::core::ffi::c_char;
    if p.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    a.zbuffer = buffer as *mut stbi_uc;
    a.zbuffer_end = (buffer as *mut stbi_uc).offset(len as isize);
    if stbi__do_zlib(
        &raw mut a,
        p,
        initial_size,
        1 as ::core::ffi::c_int,
        parse_header,
    ) != 0
    {
        if !outlen.is_null() {
            *outlen = a.zout.offset_from(a.zout_start) as ::core::ffi::c_long as ::core::ffi::c_int;
        }
        a.zout_start
    } else {
        free(a.zout_start as *mut ::core::ffi::c_void);
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    }
}
unsafe fn decode_zlib_buffer(
    output: *mut ::core::ffi::c_char,
    output_len: ::core::ffi::c_int,
    input: *const ::core::ffi::c_char,
    input_len: ::core::ffi::c_int,
    parse_header: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut state = stbi__zbuf::EMPTY;
    state.zbuffer = input as *mut stbi_uc;
    state.zbuffer_end = (input as *mut stbi_uc).offset(input_len as isize);
    if stbi__do_zlib(&raw mut state, output, output_len, 0, parse_header) != 0 {
        state.zout.offset_from(state.zout_start) as ::core::ffi::c_long as ::core::ffi::c_int
    } else {
        -1
    }
}
pub unsafe fn stbi_zlib_decode_buffer(
    obuffer: *mut ::core::ffi::c_char,
    olen: ::core::ffi::c_int,
    ibuffer: *const ::core::ffi::c_char,
    ilen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    decode_zlib_buffer(obuffer, olen, ibuffer, ilen, 1)
}
pub unsafe fn stbi_zlib_decode_noheader_malloc(
    buffer: *const ::core::ffi::c_char,
    len: ::core::ffi::c_int,
    outlen: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    stbi_zlib_decode_malloc_guesssize_headerflag(buffer, len, 16384, outlen, 0)
}
pub unsafe fn stbi_zlib_decode_noheader_buffer(
    obuffer: *mut ::core::ffi::c_char,
    olen: ::core::ffi::c_int,
    ibuffer: *const ::core::ffi::c_char,
    ilen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    decode_zlib_buffer(obuffer, olen, ibuffer, ilen, 0)
}
unsafe fn stbi__get_chunk_header(s: *mut stbi__context) -> stbi__pngchunk {
    let mut c: stbi__pngchunk = stbi__pngchunk {
        length: 0,
        type_0: 0,
    };
    c.length = stbi__get32be(s);
    c.type_0 = stbi__get32be(s);
    c
}
unsafe fn stbi__check_png_header(s: *mut stbi__context) -> ::core::ffi::c_int {
    static mut png_sig: [stbi_uc; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        if stbi__get8(s) as ::core::ffi::c_int != png_sig[i as usize] as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        i += 1;
    }
    1 as ::core::ffi::c_int
}
static mut first_row_filter: [stbi_uc; 5] = [
    STBI__F_none as ::core::ffi::c_int as stbi_uc,
    STBI__F_sub as ::core::ffi::c_int as stbi_uc,
    STBI__F_none as ::core::ffi::c_int as stbi_uc,
    STBI__F_avg_first as ::core::ffi::c_int as stbi_uc,
    STBI__F_paeth_first as ::core::ffi::c_int as stbi_uc,
];
unsafe fn stbi__paeth(
    a: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
    c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let p: ::core::ffi::c_int = a + b - c;
    let pa: ::core::ffi::c_int = abs(p - a);
    let pb: ::core::ffi::c_int = abs(p - b);
    let pc: ::core::ffi::c_int = abs(p - c);
    if pa <= pb && pa <= pc {
        return a;
    }
    if pb <= pc {
        return b;
    }
    c
}
static mut stbi__depth_scale_table: [stbi_uc; 9] = [0, 0xff, 0x55, 0, 0x11, 0, 0, 0, 0x1];
unsafe fn stbi__create_png_image_raw(
    a: *mut stbi__png,
    mut raw: *mut stbi_uc,
    raw_len: stbi__uint32,
    out_n: ::core::ffi::c_int,
    x: stbi__uint32,
    y: stbi__uint32,
    depth: ::core::ffi::c_int,
    color: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let bytes: ::core::ffi::c_int = if depth == 16 as ::core::ffi::c_int {
        2 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
    let s: *mut stbi__context = (*a).s;
    let mut i: stbi__uint32 = 0;
    let mut j: stbi__uint32 = 0;
    let stride: stbi__uint32 = x
        .wrapping_mul(out_n as stbi__uint32)
        .wrapping_mul(bytes as stbi__uint32);
    let mut img_len: stbi__uint32 = 0;
    let mut img_width_bytes: stbi__uint32 = 0;
    let mut k: ::core::ffi::c_int = 0;
    let img_n: ::core::ffi::c_int = (*s).img_n;
    let output_bytes: ::core::ffi::c_int = out_n * bytes;
    let mut filter_bytes: ::core::ffi::c_int = img_n * bytes;
    let mut width: ::core::ffi::c_int = x as ::core::ffi::c_int;
    '_c2rust_label: {
        if out_n == (*s).img_n || out_n == (*s).img_n + 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"out_n == s->img_n || out_n == s->img_n+1\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"./third_party/stb/stb_image.h\0" as *const u8
                    as *const ::core::ffi::c_char,
                4607 as ::core::ffi::c_uint,
                b"int stbi__create_png_image_raw(stbi__png *, stbi_uc *, stbi__uint32, int, stbi__uint32, stbi__uint32, int, int)\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    (*a).out = stbi__malloc_mad3(
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        output_bytes,
        0 as ::core::ffi::c_int,
    ) as *mut stbi_uc;
    if (*a).out.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if stbi__mad3sizes_valid(
        img_n,
        x as ::core::ffi::c_int,
        depth,
        7 as ::core::ffi::c_int,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    img_width_bytes = (img_n as stbi__uint32)
        .wrapping_mul(x)
        .wrapping_mul(depth as stbi__uint32)
        .wrapping_add(7 as stbi__uint32)
        >> 3 as ::core::ffi::c_int;
    img_len = img_width_bytes
        .wrapping_add(1 as stbi__uint32)
        .wrapping_mul(y);
    if raw_len < img_len {
        return 0 as ::core::ffi::c_int;
    }
    j = 0 as stbi__uint32;
    while j < y {
        let mut cur: *mut stbi_uc = (*a).out.offset(stride.wrapping_mul(j) as isize);
        let mut prior: *mut stbi_uc = ::core::ptr::null_mut::<stbi_uc>();
        let fresh19 = raw;
        raw = raw.offset(1);
        let mut filter: ::core::ffi::c_int = *fresh19 as ::core::ffi::c_int;
        if filter > 4 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if depth < 8 as ::core::ffi::c_int {
            if img_width_bytes > x {
                return 0 as ::core::ffi::c_int;
            }
            cur = cur.offset(
                x.wrapping_mul(out_n as stbi__uint32)
                    .wrapping_sub(img_width_bytes) as isize,
            );
            filter_bytes = 1 as ::core::ffi::c_int;
            width = img_width_bytes as ::core::ffi::c_int;
        }
        prior = cur.offset(-(stride as isize));
        if j == 0 as stbi__uint32 {
            filter = first_row_filter[filter as usize] as ::core::ffi::c_int;
        }
        k = 0 as ::core::ffi::c_int;
        while k < filter_bytes {
            match filter {
                0 => {
                    *cur.offset(k as isize) = *raw.offset(k as isize);
                }
                1 => {
                    *cur.offset(k as isize) = *raw.offset(k as isize);
                }
                2 => {
                    *cur.offset(k as isize) = (*raw.offset(k as isize) as ::core::ffi::c_int
                        + *prior.offset(k as isize) as ::core::ffi::c_int
                        & 255 as ::core::ffi::c_int)
                        as stbi_uc;
                }
                3 => {
                    *cur.offset(k as isize) = (*raw.offset(k as isize) as ::core::ffi::c_int
                        + (*prior.offset(k as isize) as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int)
                        & 255 as ::core::ffi::c_int)
                        as stbi_uc;
                }
                4 => {
                    *cur.offset(k as isize) = (*raw.offset(k as isize) as ::core::ffi::c_int
                        + stbi__paeth(
                            0 as ::core::ffi::c_int,
                            *prior.offset(k as isize) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        )
                        & 255 as ::core::ffi::c_int)
                        as stbi_uc;
                }
                5 => {
                    *cur.offset(k as isize) = *raw.offset(k as isize);
                }
                6 => {
                    *cur.offset(k as isize) = *raw.offset(k as isize);
                }
                _ => {}
            }
            k += 1;
        }
        if depth == 8 as ::core::ffi::c_int {
            if img_n != out_n {
                *cur.offset(img_n as isize) = 255 as stbi_uc;
            }
            raw = raw.offset(img_n as isize);
            cur = cur.offset(out_n as isize);
            prior = prior.offset(out_n as isize);
        } else if depth == 16 as ::core::ffi::c_int {
            if img_n != out_n {
                *cur.offset(filter_bytes as isize) = 255 as stbi_uc;
                *cur.offset((filter_bytes + 1 as ::core::ffi::c_int) as isize) = 255 as stbi_uc;
            }
            raw = raw.offset(filter_bytes as isize);
            cur = cur.offset(output_bytes as isize);
            prior = prior.offset(output_bytes as isize);
        } else {
            raw = raw.offset(1);
            cur = cur.offset(1);
            prior = prior.offset(1);
        }
        if depth < 8 as ::core::ffi::c_int || img_n == out_n {
            let nk: ::core::ffi::c_int = (width - 1 as ::core::ffi::c_int) * filter_bytes;
            match filter {
                0 => {
                    memcpy(
                        cur as *mut ::core::ffi::c_void,
                        raw as *const ::core::ffi::c_void,
                        nk as size_t,
                    );
                }
                1 => {
                    k = 0 as ::core::ffi::c_int;
                    while k < nk {
                        *cur.offset(k as isize) = (*raw.offset(k as isize) as ::core::ffi::c_int
                            + *cur.offset((k - filter_bytes) as isize) as ::core::ffi::c_int
                            & 255 as ::core::ffi::c_int)
                            as stbi_uc;
                        k += 1;
                    }
                }
                2 => {
                    k = 0 as ::core::ffi::c_int;
                    while k < nk {
                        *cur.offset(k as isize) = (*raw.offset(k as isize) as ::core::ffi::c_int
                            + *prior.offset(k as isize) as ::core::ffi::c_int
                            & 255 as ::core::ffi::c_int)
                            as stbi_uc;
                        k += 1;
                    }
                }
                3 => {
                    k = 0 as ::core::ffi::c_int;
                    while k < nk {
                        *cur.offset(k as isize) = (*raw.offset(k as isize) as ::core::ffi::c_int
                            + (*prior.offset(k as isize) as ::core::ffi::c_int
                                + *cur.offset((k - filter_bytes) as isize) as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int)
                            & 255 as ::core::ffi::c_int)
                            as stbi_uc;
                        k += 1;
                    }
                }
                4 => {
                    k = 0 as ::core::ffi::c_int;
                    while k < nk {
                        *cur.offset(k as isize) = (*raw.offset(k as isize) as ::core::ffi::c_int
                            + stbi__paeth(
                                *cur.offset((k - filter_bytes) as isize) as ::core::ffi::c_int,
                                *prior.offset(k as isize) as ::core::ffi::c_int,
                                *prior.offset((k - filter_bytes) as isize) as ::core::ffi::c_int,
                            )
                            & 255 as ::core::ffi::c_int)
                            as stbi_uc;
                        k += 1;
                    }
                }
                5 => {
                    k = 0 as ::core::ffi::c_int;
                    while k < nk {
                        *cur.offset(k as isize) = (*raw.offset(k as isize) as ::core::ffi::c_int
                            + (*cur.offset((k - filter_bytes) as isize) as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int)
                            & 255 as ::core::ffi::c_int)
                            as stbi_uc;
                        k += 1;
                    }
                }
                6 => {
                    k = 0 as ::core::ffi::c_int;
                    while k < nk {
                        *cur.offset(k as isize) = (*raw.offset(k as isize) as ::core::ffi::c_int
                            + stbi__paeth(
                                *cur.offset((k - filter_bytes) as isize) as ::core::ffi::c_int,
                                0 as ::core::ffi::c_int,
                                0 as ::core::ffi::c_int,
                            )
                            & 255 as ::core::ffi::c_int)
                            as stbi_uc;
                        k += 1;
                    }
                }
                _ => {}
            }
            raw = raw.offset(nk as isize);
        } else {
            '_c2rust_label_0: {
                if img_n + 1 as ::core::ffi::c_int == out_n {
                } else {
                    __assert_fail(
                        b"img_n+1 == out_n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"./third_party/stb/stb_image.h\0" as *const u8
                            as *const ::core::ffi::c_char,
                        4691 as ::core::ffi::c_uint,
                        b"int stbi__create_png_image_raw(stbi__png *, stbi_uc *, stbi__uint32, int, stbi__uint32, stbi__uint32, int, int)\0"
                            as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            };
            match filter {
                0 => {
                    i = x.wrapping_sub(1 as stbi__uint32);
                    while i >= 1 as stbi__uint32 {
                        k = 0 as ::core::ffi::c_int;
                        while k < filter_bytes {
                            *cur.offset(k as isize) = *raw.offset(k as isize);
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur.offset(filter_bytes as isize) = 255 as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                1 => {
                    i = x.wrapping_sub(1 as stbi__uint32);
                    while i >= 1 as stbi__uint32 {
                        k = 0 as ::core::ffi::c_int;
                        while k < filter_bytes {
                            *cur.offset(k as isize) = (*raw.offset(k as isize)
                                as ::core::ffi::c_int
                                + *cur.offset((k - output_bytes) as isize) as ::core::ffi::c_int
                                & 255 as ::core::ffi::c_int)
                                as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur.offset(filter_bytes as isize) = 255 as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                2 => {
                    i = x.wrapping_sub(1 as stbi__uint32);
                    while i >= 1 as stbi__uint32 {
                        k = 0 as ::core::ffi::c_int;
                        while k < filter_bytes {
                            *cur.offset(k as isize) = (*raw.offset(k as isize)
                                as ::core::ffi::c_int
                                + *prior.offset(k as isize) as ::core::ffi::c_int
                                & 255 as ::core::ffi::c_int)
                                as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur.offset(filter_bytes as isize) = 255 as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                3 => {
                    i = x.wrapping_sub(1 as stbi__uint32);
                    while i >= 1 as stbi__uint32 {
                        k = 0 as ::core::ffi::c_int;
                        while k < filter_bytes {
                            *cur.offset(k as isize) = (*raw.offset(k as isize)
                                as ::core::ffi::c_int
                                + (*prior.offset(k as isize) as ::core::ffi::c_int
                                    + *cur.offset((k - output_bytes) as isize)
                                        as ::core::ffi::c_int
                                    >> 1 as ::core::ffi::c_int)
                                & 255 as ::core::ffi::c_int)
                                as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur.offset(filter_bytes as isize) = 255 as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                4 => {
                    i = x.wrapping_sub(1 as stbi__uint32);
                    while i >= 1 as stbi__uint32 {
                        k = 0 as ::core::ffi::c_int;
                        while k < filter_bytes {
                            *cur.offset(k as isize) = (*raw.offset(k as isize)
                                as ::core::ffi::c_int
                                + stbi__paeth(
                                    *cur.offset((k - output_bytes) as isize) as ::core::ffi::c_int,
                                    *prior.offset(k as isize) as ::core::ffi::c_int,
                                    *prior.offset((k - output_bytes) as isize)
                                        as ::core::ffi::c_int,
                                )
                                & 255 as ::core::ffi::c_int)
                                as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur.offset(filter_bytes as isize) = 255 as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                5 => {
                    i = x.wrapping_sub(1 as stbi__uint32);
                    while i >= 1 as stbi__uint32 {
                        k = 0 as ::core::ffi::c_int;
                        while k < filter_bytes {
                            *cur.offset(k as isize) = (*raw.offset(k as isize)
                                as ::core::ffi::c_int
                                + (*cur.offset((k - output_bytes) as isize) as ::core::ffi::c_int
                                    >> 1 as ::core::ffi::c_int)
                                & 255 as ::core::ffi::c_int)
                                as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur.offset(filter_bytes as isize) = 255 as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                6 => {
                    i = x.wrapping_sub(1 as stbi__uint32);
                    while i >= 1 as stbi__uint32 {
                        k = 0 as ::core::ffi::c_int;
                        while k < filter_bytes {
                            *cur.offset(k as isize) = (*raw.offset(k as isize)
                                as ::core::ffi::c_int
                                + stbi__paeth(
                                    *cur.offset((k - output_bytes) as isize) as ::core::ffi::c_int,
                                    0 as ::core::ffi::c_int,
                                    0 as ::core::ffi::c_int,
                                )
                                & 255 as ::core::ffi::c_int)
                                as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur.offset(filter_bytes as isize) = 255 as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                _ => {}
            }
            if depth == 16 as ::core::ffi::c_int {
                cur = (*a).out.offset(stride.wrapping_mul(j) as isize);
                i = 0 as stbi__uint32;
                while i < x {
                    *cur.offset((filter_bytes + 1 as ::core::ffi::c_int) as isize) = 255 as stbi_uc;
                    i = i.wrapping_add(1);
                    cur = cur.offset(output_bytes as isize);
                }
            }
        }
        j = j.wrapping_add(1);
    }
    if depth < 8 as ::core::ffi::c_int {
        j = 0 as stbi__uint32;
        while j < y {
            let mut cur_0: *mut stbi_uc = (*a).out.offset(stride.wrapping_mul(j) as isize);
            let mut in_0: *mut stbi_uc = (*a)
                .out
                .offset(stride.wrapping_mul(j) as isize)
                .offset(x.wrapping_mul(out_n as stbi__uint32) as isize)
                .offset(-(img_width_bytes as isize));
            let scale: stbi_uc = (if color == 0 as ::core::ffi::c_int {
                stbi__depth_scale_table[depth as usize] as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }) as stbi_uc;
            if depth == 4 as ::core::ffi::c_int {
                k = x.wrapping_mul(img_n as stbi__uint32) as ::core::ffi::c_int;
                while k >= 2 as ::core::ffi::c_int {
                    let fresh20 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh20 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int))
                        as stbi_uc;
                    let fresh21 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh21 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
                        as stbi_uc;
                    k -= 2 as ::core::ffi::c_int;
                    in_0 = in_0.offset(1);
                }
                if k > 0 as ::core::ffi::c_int {
                    let fresh22 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh22 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int))
                        as stbi_uc;
                }
            } else if depth == 2 as ::core::ffi::c_int {
                k = x.wrapping_mul(img_n as stbi__uint32) as ::core::ffi::c_int;
                while k >= 4 as ::core::ffi::c_int {
                    let fresh23 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh23 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 6 as ::core::ffi::c_int))
                        as stbi_uc;
                    let fresh24 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh24 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int
                            & 0x3 as ::core::ffi::c_int)) as stbi_uc;
                    let fresh25 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh25 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int
                            & 0x3 as ::core::ffi::c_int)) as stbi_uc;
                    let fresh26 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh26 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int))
                        as stbi_uc;
                    k -= 4 as ::core::ffi::c_int;
                    in_0 = in_0.offset(1);
                }
                if k > 0 as ::core::ffi::c_int {
                    let fresh27 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh27 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 6 as ::core::ffi::c_int))
                        as stbi_uc;
                }
                if k > 1 as ::core::ffi::c_int {
                    let fresh28 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh28 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int
                            & 0x3 as ::core::ffi::c_int)) as stbi_uc;
                }
                if k > 2 as ::core::ffi::c_int {
                    let fresh29 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh29 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int
                            & 0x3 as ::core::ffi::c_int)) as stbi_uc;
                }
            } else if depth == 1 as ::core::ffi::c_int {
                k = x.wrapping_mul(img_n as stbi__uint32) as ::core::ffi::c_int;
                while k >= 8 as ::core::ffi::c_int {
                    let fresh30 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh30 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 7 as ::core::ffi::c_int))
                        as stbi_uc;
                    let fresh31 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh31 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 6 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                    let fresh32 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh32 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 5 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                    let fresh33 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh33 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                    let fresh34 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh34 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 3 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                    let fresh35 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh35 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                    let fresh36 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh36 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                    let fresh37 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh37 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int & 0x1 as ::core::ffi::c_int))
                        as stbi_uc;
                    k -= 8 as ::core::ffi::c_int;
                    in_0 = in_0.offset(1);
                }
                if k > 0 as ::core::ffi::c_int {
                    let fresh38 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh38 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 7 as ::core::ffi::c_int))
                        as stbi_uc;
                }
                if k > 1 as ::core::ffi::c_int {
                    let fresh39 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh39 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 6 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                }
                if k > 2 as ::core::ffi::c_int {
                    let fresh40 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh40 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 5 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                }
                if k > 3 as ::core::ffi::c_int {
                    let fresh41 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh41 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                }
                if k > 4 as ::core::ffi::c_int {
                    let fresh42 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh42 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 3 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                }
                if k > 5 as ::core::ffi::c_int {
                    let fresh43 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh43 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                }
                if k > 6 as ::core::ffi::c_int {
                    let fresh44 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh44 = (scale as ::core::ffi::c_int
                        * (*in_0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                            & 0x1 as ::core::ffi::c_int)) as stbi_uc;
                }
            }
            if img_n != out_n {
                let mut q: ::core::ffi::c_int = 0;
                cur_0 = (*a).out.offset(stride.wrapping_mul(j) as isize);
                if img_n == 1 as ::core::ffi::c_int {
                    q = x.wrapping_sub(1 as stbi__uint32) as ::core::ffi::c_int;
                    while q >= 0 as ::core::ffi::c_int {
                        *cur_0.offset(
                            (q * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                        ) = 255 as stbi_uc;
                        *cur_0.offset(
                            (q * 2 as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as isize,
                        ) = *cur_0.offset(q as isize);
                        q -= 1;
                    }
                } else {
                    '_c2rust_label_1: {
                        if img_n == 3 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"img_n == 3\0" as *const u8 as *const ::core::ffi::c_char,
                                b"./third_party/stb/stb_image.h\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                4780 as ::core::ffi::c_uint,
                                b"int stbi__create_png_image_raw(stbi__png *, stbi_uc *, stbi__uint32, int, stbi__uint32, stbi__uint32, int, int)\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    q = x.wrapping_sub(1 as stbi__uint32) as ::core::ffi::c_int;
                    while q >= 0 as ::core::ffi::c_int {
                        *cur_0.offset(
                            (q * 4 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize,
                        ) = 255 as stbi_uc;
                        *cur_0.offset(
                            (q * 4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize,
                        ) = *cur_0.offset(
                            (q * 3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize,
                        );
                        *cur_0.offset(
                            (q * 4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                        ) = *cur_0.offset(
                            (q * 3 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                        );
                        *cur_0.offset(
                            (q * 4 as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as isize,
                        ) = *cur_0.offset(
                            (q * 3 as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as isize,
                        );
                        q -= 1;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
    } else if depth == 16 as ::core::ffi::c_int {
        let mut cur_1: *mut stbi_uc = (*a).out;
        let mut cur16: *mut stbi__uint16 = cur_1 as *mut stbi__uint16;
        i = 0 as stbi__uint32;
        while i < x.wrapping_mul(y).wrapping_mul(out_n as stbi__uint32) {
            *cur16 = ((*cur_1.offset(0) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                | *cur_1.offset(1) as ::core::ffi::c_int) as stbi__uint16;
            i = i.wrapping_add(1);
            cur16 = cur16.offset(1);
            cur_1 = cur_1.offset(2);
        }
    }
    1 as ::core::ffi::c_int
}
unsafe fn stbi__create_png_image(
    a: *mut stbi__png,
    mut image_data: *mut stbi_uc,
    mut image_data_len: stbi__uint32,
    out_n: ::core::ffi::c_int,
    depth: ::core::ffi::c_int,
    color: ::core::ffi::c_int,
    interlaced: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let bytes: ::core::ffi::c_int = if depth == 16 as ::core::ffi::c_int {
        2 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
    let out_bytes: ::core::ffi::c_int = out_n * bytes;
    let mut final_0: *mut stbi_uc = ::core::ptr::null_mut::<stbi_uc>();
    let mut p: ::core::ffi::c_int = 0;
    if interlaced == 0 {
        return stbi__create_png_image_raw(
            a,
            image_data,
            image_data_len,
            out_n,
            (*(*a).s).img_x,
            (*(*a).s).img_y,
            depth,
            color,
        );
    }
    final_0 = stbi__malloc_mad3(
        (*(*a).s).img_x as ::core::ffi::c_int,
        (*(*a).s).img_y as ::core::ffi::c_int,
        out_bytes,
        0 as ::core::ffi::c_int,
    ) as *mut stbi_uc;
    if final_0.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    p = 0 as ::core::ffi::c_int;
    while p < 7 as ::core::ffi::c_int {
        let xorig: [::core::ffi::c_int; 7] = [
            0 as ::core::ffi::c_int,
            4 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            2 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        ];
        let yorig: [::core::ffi::c_int; 7] = [
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            4 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            2 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        ];
        let xspc: [::core::ffi::c_int; 7] = [
            8 as ::core::ffi::c_int,
            8 as ::core::ffi::c_int,
            4 as ::core::ffi::c_int,
            4 as ::core::ffi::c_int,
            2 as ::core::ffi::c_int,
            2 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        ];
        let yspc: [::core::ffi::c_int; 7] = [
            8 as ::core::ffi::c_int,
            8 as ::core::ffi::c_int,
            8 as ::core::ffi::c_int,
            4 as ::core::ffi::c_int,
            4 as ::core::ffi::c_int,
            2 as ::core::ffi::c_int,
            2 as ::core::ffi::c_int,
        ];
        let mut i: ::core::ffi::c_int = 0;
        let mut j: ::core::ffi::c_int = 0;
        let mut x: ::core::ffi::c_int = 0;
        let mut y: ::core::ffi::c_int = 0;
        x = (*(*a).s)
            .img_x
            .wrapping_sub(xorig[p as usize] as stbi__uint32)
            .wrapping_add(xspc[p as usize] as stbi__uint32)
            .wrapping_sub(1 as stbi__uint32)
            .wrapping_div(xspc[p as usize] as stbi__uint32) as ::core::ffi::c_int;
        y = (*(*a).s)
            .img_y
            .wrapping_sub(yorig[p as usize] as stbi__uint32)
            .wrapping_add(yspc[p as usize] as stbi__uint32)
            .wrapping_sub(1 as stbi__uint32)
            .wrapping_div(yspc[p as usize] as stbi__uint32) as ::core::ffi::c_int;
        if x != 0 && y != 0 {
            let img_len: stbi__uint32 = ((((*(*a).s).img_n * x * depth + 7 as ::core::ffi::c_int
                >> 3 as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int)
                * y) as stbi__uint32;
            if stbi__create_png_image_raw(
                a,
                image_data,
                image_data_len,
                out_n,
                x as stbi__uint32,
                y as stbi__uint32,
                depth,
                color,
            ) == 0
            {
                free(final_0 as *mut ::core::ffi::c_void);
                return 0 as ::core::ffi::c_int;
            }
            j = 0 as ::core::ffi::c_int;
            while j < y {
                i = 0 as ::core::ffi::c_int;
                while i < x {
                    let out_y: ::core::ffi::c_int = j * yspc[p as usize] + yorig[p as usize];
                    let out_x: ::core::ffi::c_int = i * xspc[p as usize] + xorig[p as usize];
                    memcpy(
                        final_0
                            .offset(
                                (out_y as stbi__uint32)
                                    .wrapping_mul((*(*a).s).img_x)
                                    .wrapping_mul(out_bytes as stbi__uint32)
                                    as isize,
                            )
                            .offset((out_x * out_bytes) as isize)
                            as *mut ::core::ffi::c_void,
                        (*a).out.offset(((j * x + i) * out_bytes) as isize)
                            as *const ::core::ffi::c_void,
                        out_bytes as size_t,
                    );
                    i += 1;
                }
                j += 1;
            }
            free((*a).out as *mut ::core::ffi::c_void);
            image_data = image_data.offset(img_len as isize);
            image_data_len = image_data_len.wrapping_sub(img_len);
        }
        p += 1;
    }
    (*a).out = final_0;
    1 as ::core::ffi::c_int
}
unsafe fn stbi__compute_transparency(
    z: *mut stbi__png,
    tc: *mut stbi_uc,
    out_n: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let s: *mut stbi__context = (*z).s;
    let mut i: stbi__uint32 = 0;
    let pixel_count: stbi__uint32 = (*s).img_x.wrapping_mul((*s).img_y);
    let mut p: *mut stbi_uc = (*z).out;
    '_c2rust_label: {
        if out_n == 2 as ::core::ffi::c_int || out_n == 4 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"out_n == 2 || out_n == 4\0" as *const u8 as *const ::core::ffi::c_char,
                b"./third_party/stb/stb_image.h\0" as *const u8 as *const ::core::ffi::c_char,
                4859 as ::core::ffi::c_uint,
                b"int stbi__compute_transparency(stbi__png *, stbi_uc *, int)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if out_n == 2 as ::core::ffi::c_int {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            *p.offset(1) =
                (if *p.offset(0) as ::core::ffi::c_int == *tc.offset(0) as ::core::ffi::c_int {
                    0 as ::core::ffi::c_int
                } else {
                    255 as ::core::ffi::c_int
                }) as stbi_uc;
            p = p.offset(2);
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            if *p.offset(0) as ::core::ffi::c_int == *tc.offset(0) as ::core::ffi::c_int
                && *p.offset(1) as ::core::ffi::c_int == *tc.offset(1) as ::core::ffi::c_int
                && *p.offset(2) as ::core::ffi::c_int == *tc.offset(2) as ::core::ffi::c_int
            {
                *p.offset(3) = 0 as stbi_uc;
            }
            p = p.offset(4);
            i = i.wrapping_add(1);
        }
    }
    1 as ::core::ffi::c_int
}
unsafe fn stbi__compute_transparency16(
    z: *mut stbi__png,
    tc: *mut stbi__uint16,
    out_n: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let s: *mut stbi__context = (*z).s;
    let mut i: stbi__uint32 = 0;
    let pixel_count: stbi__uint32 = (*s).img_x.wrapping_mul((*s).img_y);
    let mut p: *mut stbi__uint16 = (*z).out as *mut stbi__uint16;
    '_c2rust_label: {
        if out_n == 2 as ::core::ffi::c_int || out_n == 4 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"out_n == 2 || out_n == 4\0" as *const u8 as *const ::core::ffi::c_char,
                b"./third_party/stb/stb_image.h\0" as *const u8 as *const ::core::ffi::c_char,
                4884 as ::core::ffi::c_uint,
                b"int stbi__compute_transparency16(stbi__png *, stbi__uint16 *, int)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if out_n == 2 as ::core::ffi::c_int {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            *p.offset(1) =
                (if *p.offset(0) as ::core::ffi::c_int == *tc.offset(0) as ::core::ffi::c_int {
                    0 as ::core::ffi::c_int
                } else {
                    65535 as ::core::ffi::c_int
                }) as stbi__uint16;
            p = p.offset(2);
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            if *p.offset(0) as ::core::ffi::c_int == *tc.offset(0) as ::core::ffi::c_int
                && *p.offset(1) as ::core::ffi::c_int == *tc.offset(1) as ::core::ffi::c_int
                && *p.offset(2) as ::core::ffi::c_int == *tc.offset(2) as ::core::ffi::c_int
            {
                *p.offset(3) = 0 as stbi__uint16;
            }
            p = p.offset(4);
            i = i.wrapping_add(1);
        }
    }
    1 as ::core::ffi::c_int
}
unsafe fn stbi__expand_png_palette(
    a: *mut stbi__png,
    palette: *mut stbi_uc,
    len: ::core::ffi::c_int,
    pal_img_n: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: stbi__uint32 = 0;
    let pixel_count: stbi__uint32 = (*(*a).s).img_x.wrapping_mul((*(*a).s).img_y);
    let mut p: *mut stbi_uc = ::core::ptr::null_mut::<stbi_uc>();
    let mut temp_out: *mut stbi_uc = ::core::ptr::null_mut::<stbi_uc>();
    let orig: *mut stbi_uc = (*a).out;
    p = stbi__malloc_mad2(
        pixel_count as ::core::ffi::c_int,
        pal_img_n,
        0 as ::core::ffi::c_int,
    ) as *mut stbi_uc;
    if p.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    temp_out = p;
    if pal_img_n == 3 as ::core::ffi::c_int {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            let n: ::core::ffi::c_int =
                *orig.offset(i as isize) as ::core::ffi::c_int * 4 as ::core::ffi::c_int;
            *p.offset(0) = *palette.offset(n as isize);
            *p.offset(1) = *palette.offset((n + 1 as ::core::ffi::c_int) as isize);
            *p.offset(2) = *palette.offset((n + 2 as ::core::ffi::c_int) as isize);
            p = p.offset(3);
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            let n_0: ::core::ffi::c_int =
                *orig.offset(i as isize) as ::core::ffi::c_int * 4 as ::core::ffi::c_int;
            *p.offset(0) = *palette.offset(n_0 as isize);
            *p.offset(1) = *palette.offset((n_0 + 1 as ::core::ffi::c_int) as isize);
            *p.offset(2) = *palette.offset((n_0 + 2 as ::core::ffi::c_int) as isize);
            *p.offset(3) = *palette.offset((n_0 + 3 as ::core::ffi::c_int) as isize);
            p = p.offset(4);
            i = i.wrapping_add(1);
        }
    }
    free((*a).out as *mut ::core::ffi::c_void);
    (*a).out = temp_out;
    1 as ::core::ffi::c_int
}
static mut stbi__unpremultiply_on_load_global: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut stbi__de_iphone_flag_global: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub unsafe fn stbi_set_unpremultiply_on_load(
    flag_true_if_should_unpremultiply: ::core::ffi::c_int,
) {
    stbi__unpremultiply_on_load_global = flag_true_if_should_unpremultiply;
}
pub unsafe fn stbi_convert_iphone_png_to_rgb(flag_true_if_should_convert: ::core::ffi::c_int) {
    stbi__de_iphone_flag_global = flag_true_if_should_convert;
}
unsafe fn stbi__de_iphone(z: *mut stbi__png) {
    let s: *mut stbi__context = (*z).s;
    let mut i: stbi__uint32 = 0;
    let pixel_count: stbi__uint32 = (*s).img_x.wrapping_mul((*s).img_y);
    let mut p: *mut stbi_uc = (*z).out;
    if (*s).img_out_n == 3 as ::core::ffi::c_int {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            let t: stbi_uc = *p.offset(0);
            *p.offset(0) = *p.offset(2);
            *p.offset(2) = t;
            p = p.offset(3);
            i = i.wrapping_add(1);
        }
    } else {
        '_c2rust_label: {
            if (*s).img_out_n == 4 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"s->img_out_n == 4\0" as *const u8 as *const ::core::ffi::c_char,
                    b"./third_party/stb/stb_image.h\0" as *const u8 as *const ::core::ffi::c_char,
                    4992 as ::core::ffi::c_uint,
                    b"void stbi__de_iphone(stbi__png *)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if stbi__unpremultiply_on_load_global != 0 {
            i = 0 as stbi__uint32;
            while i < pixel_count {
                let a: stbi_uc = *p.offset(3);
                let t_0: stbi_uc = *p.offset(0);
                if a != 0 {
                    let half: stbi_uc =
                        (a as ::core::ffi::c_int / 2 as ::core::ffi::c_int) as stbi_uc;
                    *p.offset(0) = ((*p.offset(2) as ::core::ffi::c_int
                        * 255 as ::core::ffi::c_int
                        + half as ::core::ffi::c_int)
                        / a as ::core::ffi::c_int) as stbi_uc;
                    *p.offset(1) = ((*p.offset(1) as ::core::ffi::c_int
                        * 255 as ::core::ffi::c_int
                        + half as ::core::ffi::c_int)
                        / a as ::core::ffi::c_int) as stbi_uc;
                    *p.offset(2) = ((t_0 as ::core::ffi::c_int * 255 as ::core::ffi::c_int
                        + half as ::core::ffi::c_int)
                        / a as ::core::ffi::c_int) as stbi_uc;
                } else {
                    *p.offset(0) = *p.offset(2);
                    *p.offset(2) = t_0;
                }
                p = p.offset(4);
                i = i.wrapping_add(1);
            }
        } else {
            i = 0 as stbi__uint32;
            while i < pixel_count {
                let t_1: stbi_uc = *p.offset(0);
                *p.offset(0) = *p.offset(2);
                *p.offset(2) = t_1;
                p = p.offset(4);
                i = i.wrapping_add(1);
            }
        }
    };
}
unsafe fn stbi__parse_png_file(
    z: *mut stbi__png,
    scan: ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut palette: [stbi_uc; 1024] = [0; 1024];
    let mut pal_img_n: stbi_uc = 0 as stbi_uc;
    let mut has_trans: stbi_uc = 0 as stbi_uc;
    let mut tc: [stbi_uc; 3] = [0 as ::core::ffi::c_int as stbi_uc, 0, 0];
    let mut tc16: [stbi__uint16; 3] = [0; 3];
    let mut ioff: stbi__uint32 = 0 as stbi__uint32;
    let mut idata_limit: stbi__uint32 = 0 as stbi__uint32;
    let mut i: stbi__uint32 = 0;
    let mut pal_len: stbi__uint32 = 0 as stbi__uint32;
    let mut first: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut k: ::core::ffi::c_int = 0;
    let mut interlace: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut color: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_iphone: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let s: *mut stbi__context = (*z).s;
    (*z).expanded = ::core::ptr::null_mut::<stbi_uc>();
    (*z).idata = ::core::ptr::null_mut::<stbi_uc>();
    (*z).out = ::core::ptr::null_mut::<stbi_uc>();
    if stbi__check_png_header(s) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if scan == STBI__SCAN_type as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    loop {
        let c: stbi__pngchunk = stbi__get_chunk_header(s);
        match c.type_0 {
            1130840649 => {
                is_iphone = 1 as ::core::ffi::c_int;
                stbi__skip(s, c.length as ::core::ffi::c_int);
            }
            1229472850 => {
                let mut comp: ::core::ffi::c_int = 0;
                let mut filter: ::core::ffi::c_int = 0;
                if first == 0 {
                    return 0 as ::core::ffi::c_int;
                }
                first = 0 as ::core::ffi::c_int;
                if c.length != 13 as stbi__uint32 {
                    return 0 as ::core::ffi::c_int;
                }
                (*s).img_x = stbi__get32be(s);
                (*s).img_y = stbi__get32be(s);
                if (*s).img_y > STBI_MAX_DIMENSIONS as stbi__uint32 {
                    return 0 as ::core::ffi::c_int;
                }
                if (*s).img_x > STBI_MAX_DIMENSIONS as stbi__uint32 {
                    return 0 as ::core::ffi::c_int;
                }
                (*z).depth = stbi__get8(s) as ::core::ffi::c_int;
                if (*z).depth != 1 as ::core::ffi::c_int
                    && (*z).depth != 2 as ::core::ffi::c_int
                    && (*z).depth != 4 as ::core::ffi::c_int
                    && (*z).depth != 8 as ::core::ffi::c_int
                    && (*z).depth != 16 as ::core::ffi::c_int
                {
                    return 0 as ::core::ffi::c_int;
                }
                color = stbi__get8(s) as ::core::ffi::c_int;
                if color > 6 as ::core::ffi::c_int {
                    return 0 as ::core::ffi::c_int;
                }
                if color == 3 as ::core::ffi::c_int && (*z).depth == 16 as ::core::ffi::c_int {
                    return 0 as ::core::ffi::c_int;
                }
                if color == 3 as ::core::ffi::c_int {
                    pal_img_n = 3 as stbi_uc;
                } else if color & 1 as ::core::ffi::c_int != 0 {
                    return 0 as ::core::ffi::c_int;
                }
                comp = stbi__get8(s) as ::core::ffi::c_int;
                if comp != 0 {
                    return 0 as ::core::ffi::c_int;
                }
                filter = stbi__get8(s) as ::core::ffi::c_int;
                if filter != 0 {
                    return 0 as ::core::ffi::c_int;
                }
                interlace = stbi__get8(s) as ::core::ffi::c_int;
                if interlace > 1 as ::core::ffi::c_int {
                    return 0 as ::core::ffi::c_int;
                }
                if (*s).img_x == 0 || (*s).img_y == 0 {
                    return 0 as ::core::ffi::c_int;
                }
                if pal_img_n == 0 {
                    (*s).img_n = (if color & 2 as ::core::ffi::c_int != 0 {
                        3 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    }) + (if color & 4 as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    });
                    if (((1 as ::core::ffi::c_int) << 30 as ::core::ffi::c_int) as stbi__uint32)
                        .wrapping_div((*s).img_x)
                        .wrapping_div((*s).img_n as stbi__uint32)
                        < (*s).img_y
                    {
                        return 0 as ::core::ffi::c_int;
                    }
                    if scan == STBI__SCAN_header as ::core::ffi::c_int {
                        return 1 as ::core::ffi::c_int;
                    }
                } else {
                    (*s).img_n = 1 as ::core::ffi::c_int;
                    if (((1 as ::core::ffi::c_int) << 30 as ::core::ffi::c_int) as stbi__uint32)
                        .wrapping_div((*s).img_x)
                        .wrapping_div(4 as stbi__uint32)
                        < (*s).img_y
                    {
                        return 0 as ::core::ffi::c_int;
                    }
                }
            }
            1347179589 => {
                if first != 0 {
                    return 0 as ::core::ffi::c_int;
                }
                if c.length > (256 as ::core::ffi::c_int * 3 as ::core::ffi::c_int) as stbi__uint32
                {
                    return 0 as ::core::ffi::c_int;
                }
                pal_len = c.length.wrapping_div(3 as stbi__uint32);
                if pal_len.wrapping_mul(3 as stbi__uint32) != c.length {
                    return 0 as ::core::ffi::c_int;
                }
                i = 0 as stbi__uint32;
                while i < pal_len {
                    palette[i
                        .wrapping_mul(4 as stbi__uint32)
                        .wrapping_add(0 as stbi__uint32) as usize] = stbi__get8(s);
                    palette[i
                        .wrapping_mul(4 as stbi__uint32)
                        .wrapping_add(1 as stbi__uint32) as usize] = stbi__get8(s);
                    palette[i
                        .wrapping_mul(4 as stbi__uint32)
                        .wrapping_add(2 as stbi__uint32) as usize] = stbi__get8(s);
                    palette[i
                        .wrapping_mul(4 as stbi__uint32)
                        .wrapping_add(3 as stbi__uint32) as usize] = 255 as stbi_uc;
                    i = i.wrapping_add(1);
                }
            }
            1951551059 => {
                if first != 0 {
                    return 0 as ::core::ffi::c_int;
                }
                if !(*z).idata.is_null() {
                    return 0 as ::core::ffi::c_int;
                }
                if pal_img_n != 0 {
                    if scan == STBI__SCAN_header as ::core::ffi::c_int {
                        (*s).img_n = 4 as ::core::ffi::c_int;
                        return 1 as ::core::ffi::c_int;
                    }
                    if pal_len == 0 as stbi__uint32 {
                        return 0 as ::core::ffi::c_int;
                    }
                    if c.length > pal_len {
                        return 0 as ::core::ffi::c_int;
                    }
                    pal_img_n = 4 as stbi_uc;
                    i = 0 as stbi__uint32;
                    while i < c.length {
                        palette[i
                            .wrapping_mul(4 as stbi__uint32)
                            .wrapping_add(3 as stbi__uint32)
                            as usize] = stbi__get8(s);
                        i = i.wrapping_add(1);
                    }
                } else {
                    if (*s).img_n & 1 as ::core::ffi::c_int == 0 {
                        return 0 as ::core::ffi::c_int;
                    }
                    if c.length != ((*s).img_n as stbi__uint32).wrapping_mul(2 as stbi__uint32) {
                        return 0 as ::core::ffi::c_int;
                    }
                    has_trans = 1 as stbi_uc;
                    if (*z).depth == 16 as ::core::ffi::c_int {
                        k = 0 as ::core::ffi::c_int;
                        while k < (*s).img_n {
                            tc16[k as usize] = stbi__get16be(s) as stbi__uint16;
                            k += 1;
                        }
                    } else {
                        k = 0 as ::core::ffi::c_int;
                        while k < (*s).img_n {
                            tc[k as usize] = ((stbi__get16be(s) & 255 as ::core::ffi::c_int)
                                as stbi_uc
                                as ::core::ffi::c_int
                                * stbi__depth_scale_table[(*z).depth as usize]
                                    as ::core::ffi::c_int)
                                as stbi_uc;
                            k += 1;
                        }
                    }
                }
            }
            1229209940 => {
                if first != 0 {
                    return 0 as ::core::ffi::c_int;
                }
                if pal_img_n as ::core::ffi::c_int != 0 && pal_len == 0 {
                    return 0 as ::core::ffi::c_int;
                }
                if scan == STBI__SCAN_header as ::core::ffi::c_int {
                    (*s).img_n = pal_img_n as ::core::ffi::c_int;
                    return 1 as ::core::ffi::c_int;
                }
                if (ioff.wrapping_add(c.length) as ::core::ffi::c_int) < ioff as ::core::ffi::c_int
                {
                    return 0 as ::core::ffi::c_int;
                }
                if ioff.wrapping_add(c.length) > idata_limit {
                    let idata_limit_old: stbi__uint32 = idata_limit;
                    let mut p: *mut stbi_uc = ::core::ptr::null_mut::<stbi_uc>();
                    if idata_limit == 0 as stbi__uint32 {
                        idata_limit = if c.length > 4096 as stbi__uint32 {
                            c.length
                        } else {
                            4096 as stbi__uint32
                        };
                    }
                    while ioff.wrapping_add(c.length) > idata_limit {
                        idata_limit = idata_limit.wrapping_mul(2 as stbi__uint32);
                    }
                    p = realloc(
                        (*z).idata as *mut ::core::ffi::c_void,
                        idata_limit as size_t,
                    ) as *mut stbi_uc;
                    if p.is_null() {
                        return 0 as ::core::ffi::c_int;
                    }
                    (*z).idata = p;
                }
                if stbi__getn(
                    s,
                    (*z).idata.offset(ioff as isize),
                    c.length as ::core::ffi::c_int,
                ) == 0
                {
                    return 0 as ::core::ffi::c_int;
                }
                ioff = ioff.wrapping_add(c.length);
            }
            1229278788 => {
                let mut raw_len: stbi__uint32 = 0;
                let mut bpl: stbi__uint32 = 0;
                if first != 0 {
                    return 0 as ::core::ffi::c_int;
                }
                if scan != STBI__SCAN_load as ::core::ffi::c_int {
                    return 1 as ::core::ffi::c_int;
                }
                if (*z).idata.is_null() {
                    return 0 as ::core::ffi::c_int;
                }
                bpl = (*s)
                    .img_x
                    .wrapping_mul((*z).depth as stbi__uint32)
                    .wrapping_add(7 as stbi__uint32)
                    .wrapping_div(8 as stbi__uint32);
                raw_len = bpl
                    .wrapping_mul((*s).img_y)
                    .wrapping_mul((*s).img_n as stbi__uint32)
                    .wrapping_add((*s).img_y);
                (*z).expanded = stbi_zlib_decode_malloc_guesssize_headerflag(
                    (*z).idata as *mut ::core::ffi::c_char,
                    ioff as ::core::ffi::c_int,
                    raw_len as ::core::ffi::c_int,
                    &raw mut raw_len as *mut ::core::ffi::c_int,
                    (is_iphone == 0) as ::core::ffi::c_int,
                ) as *mut stbi_uc;
                if (*z).expanded.is_null() {
                    return 0 as ::core::ffi::c_int;
                }
                free((*z).idata as *mut ::core::ffi::c_void);
                (*z).idata = ::core::ptr::null_mut::<stbi_uc>();
                if req_comp == (*s).img_n + 1 as ::core::ffi::c_int
                    && req_comp != 3 as ::core::ffi::c_int
                    && pal_img_n == 0
                    || has_trans as ::core::ffi::c_int != 0
                {
                    (*s).img_out_n = (*s).img_n + 1 as ::core::ffi::c_int;
                } else {
                    (*s).img_out_n = (*s).img_n;
                }
                if stbi__create_png_image(
                    z,
                    (*z).expanded,
                    raw_len,
                    (*s).img_out_n,
                    (*z).depth,
                    color,
                    interlace,
                ) == 0
                {
                    return 0 as ::core::ffi::c_int;
                }
                if has_trans != 0 {
                    if (*z).depth == 16 as ::core::ffi::c_int {
                        if stbi__compute_transparency16(
                            z,
                            &raw mut tc16 as *mut stbi__uint16,
                            (*s).img_out_n,
                        ) == 0
                        {
                            return 0 as ::core::ffi::c_int;
                        }
                    } else if stbi__compute_transparency(
                        z,
                        &raw mut tc as *mut stbi_uc,
                        (*s).img_out_n,
                    ) == 0
                    {
                        return 0 as ::core::ffi::c_int;
                    }
                }
                if is_iphone != 0
                    && stbi__de_iphone_flag_global != 0
                    && (*s).img_out_n > 2 as ::core::ffi::c_int
                {
                    stbi__de_iphone(z);
                }
                if pal_img_n != 0 {
                    (*s).img_n = pal_img_n as ::core::ffi::c_int;
                    (*s).img_out_n = pal_img_n as ::core::ffi::c_int;
                    if req_comp >= 3 as ::core::ffi::c_int {
                        (*s).img_out_n = req_comp;
                    }
                    if stbi__expand_png_palette(
                        z,
                        &raw mut palette as *mut stbi_uc,
                        pal_len as ::core::ffi::c_int,
                        (*s).img_out_n,
                    ) == 0
                    {
                        return 0 as ::core::ffi::c_int;
                    }
                } else if has_trans != 0 {
                    (*s).img_n += 1;
                }
                free((*z).expanded as *mut ::core::ffi::c_void);
                (*z).expanded = ::core::ptr::null_mut::<stbi_uc>();
                stbi__get32be(s);
                return 1 as ::core::ffi::c_int;
            }
            _ => {
                if first != 0 {
                    return 0 as ::core::ffi::c_int;
                }
                if c.type_0
                    & ((1 as ::core::ffi::c_int) << 29 as ::core::ffi::c_int) as stbi__uint32
                    == 0 as stbi__uint32
                {
                    return 0 as ::core::ffi::c_int;
                }
                stbi__skip(s, c.length as ::core::ffi::c_int);
            }
        }
        stbi__get32be(s);
    }
}
unsafe fn stbi__do_png(
    p: *mut stbi__png,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    n: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
    ri: *mut stbi__result_info,
) -> *mut ::core::ffi::c_void {
    let mut result: *mut ::core::ffi::c_void = NULL_0;
    if req_comp < 0 as ::core::ffi::c_int || req_comp > 4 as ::core::ffi::c_int {
        return (if 0 as ::core::ffi::c_int != 0 {
            NULL_0
        } else {
            NULL_0
        }) as size_t as *mut ::core::ffi::c_uchar as *mut ::core::ffi::c_void;
    }
    if stbi__parse_png_file(p, STBI__SCAN_load as ::core::ffi::c_int, req_comp) != 0 {
        if (*p).depth <= 8 as ::core::ffi::c_int {
            (*ri).bits_per_channel = 8 as ::core::ffi::c_int;
        } else if (*p).depth == 16 as ::core::ffi::c_int {
            (*ri).bits_per_channel = 16 as ::core::ffi::c_int;
        } else {
            return (if 0 as ::core::ffi::c_int != 0 {
                NULL_0
            } else {
                NULL_0
            }) as size_t as *mut ::core::ffi::c_uchar
                as *mut ::core::ffi::c_void;
        }
        result = (*p).out as *mut ::core::ffi::c_void;
        (*p).out = ::core::ptr::null_mut::<stbi_uc>();
        if req_comp != 0 && req_comp != (*(*p).s).img_out_n {
            if (*ri).bits_per_channel == 8 as ::core::ffi::c_int {
                result = stbi__convert_format(
                    result as *mut ::core::ffi::c_uchar,
                    (*(*p).s).img_out_n,
                    req_comp,
                    (*(*p).s).img_x as ::core::ffi::c_uint,
                    (*(*p).s).img_y as ::core::ffi::c_uint,
                ) as *mut ::core::ffi::c_void;
            } else {
                result = stbi__convert_format16(
                    result as *mut stbi__uint16,
                    (*(*p).s).img_out_n,
                    req_comp,
                    (*(*p).s).img_x as ::core::ffi::c_uint,
                    (*(*p).s).img_y as ::core::ffi::c_uint,
                ) as *mut ::core::ffi::c_void;
            }
            (*(*p).s).img_out_n = req_comp;
            if result.is_null() {
                return result;
            }
        }
        *x = (*(*p).s).img_x as ::core::ffi::c_int;
        *y = (*(*p).s).img_y as ::core::ffi::c_int;
        if !n.is_null() {
            *n = (*(*p).s).img_n;
        }
    }
    free((*p).out as *mut ::core::ffi::c_void);
    (*p).out = ::core::ptr::null_mut::<stbi_uc>();
    free((*p).expanded as *mut ::core::ffi::c_void);
    (*p).expanded = ::core::ptr::null_mut::<stbi_uc>();
    free((*p).idata as *mut ::core::ffi::c_void);
    (*p).idata = ::core::ptr::null_mut::<stbi_uc>();
    result
}
unsafe fn stbi__png_load(
    s: *mut stbi__context,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
    req_comp: ::core::ffi::c_int,
    ri: *mut stbi__result_info,
) -> *mut ::core::ffi::c_void {
    let mut p: stbi__png = stbi__png::EMPTY;
    p.s = s;
    stbi__do_png(&raw mut p, x, y, comp, req_comp, ri)
}
unsafe fn stbi__png_test(s: *mut stbi__context) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    r = stbi__check_png_header(s);
    stbi__rewind(s);
    r
}
unsafe fn stbi__png_info_raw(
    p: *mut stbi__png,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if stbi__parse_png_file(
        p,
        STBI__SCAN_header as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ) == 0
    {
        stbi__rewind((*p).s);
        return 0 as ::core::ffi::c_int;
    }
    if !x.is_null() {
        *x = (*(*p).s).img_x as ::core::ffi::c_int;
    }
    if !y.is_null() {
        *y = (*(*p).s).img_y as ::core::ffi::c_int;
    }
    if !comp.is_null() {
        *comp = (*(*p).s).img_n;
    }
    1 as ::core::ffi::c_int
}
unsafe fn stbi__png_info(
    s: *mut stbi__context,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: stbi__png = stbi__png::EMPTY;
    p.s = s;
    stbi__png_info_raw(&raw mut p, x, y, comp)
}
unsafe fn stbi__png_is16(s: *mut stbi__context) -> ::core::ffi::c_int {
    let mut p: stbi__png = stbi__png::EMPTY;
    p.s = s;
    if stbi__png_info_raw(
        &raw mut p,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    if p.depth != 16 as ::core::ffi::c_int {
        stbi__rewind(p.s);
        return 0 as ::core::ffi::c_int;
    }
    1 as ::core::ffi::c_int
}
unsafe fn stbi__info_main(
    s: *mut stbi__context,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if stbi__png_info(s, x, y, comp) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    0 as ::core::ffi::c_int
}
unsafe fn stbi__is_16_main(s: *mut stbi__context) -> ::core::ffi::c_int {
    if stbi__png_is16(s) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    0 as ::core::ffi::c_int
}
pub unsafe fn stbi_info(
    filename: *const ::core::ffi::c_char,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let f: *mut FILE = stbi__fopen(filename, b"rb\0" as *const u8 as *const ::core::ffi::c_char);
    let mut result: ::core::ffi::c_int = 0;
    if f.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    result = stbi_info_from_file(f, x, y, comp);
    fclose(f);
    result
}
pub unsafe fn stbi_info_from_file(
    f: *mut FILE,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut s: stbi__context = stbi__context::EMPTY;
    let pos: ::core::ffi::c_long = ftell(f);
    stbi__start_file(&raw mut s, f);
    r = stbi__info_main(&raw mut s, x, y, comp);
    fseek(f, pos, SEEK_SET);
    r
}
pub unsafe fn stbi_is_16_bit(filename: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let f: *mut FILE = stbi__fopen(filename, b"rb\0" as *const u8 as *const ::core::ffi::c_char);
    let mut result: ::core::ffi::c_int = 0;
    if f.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    result = stbi_is_16_bit_from_file(f);
    fclose(f);
    result
}
pub unsafe fn stbi_is_16_bit_from_file(f: *mut FILE) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut s: stbi__context = stbi__context::EMPTY;
    let pos: ::core::ffi::c_long = ftell(f);
    stbi__start_file(&raw mut s, f);
    r = stbi__is_16_main(&raw mut s);
    fseek(f, pos, SEEK_SET);
    r
}
pub unsafe fn stbi_info_from_memory(
    buffer: *const stbi_uc,
    len: ::core::ffi::c_int,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_mem(&raw mut s, buffer, len);
    stbi__info_main(&raw mut s, x, y, comp)
}
pub unsafe fn stbi_info_from_callbacks(
    c: *const stbi_io_callbacks,
    user: *mut ::core::ffi::c_void,
    x: *mut ::core::ffi::c_int,
    y: *mut ::core::ffi::c_int,
    comp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_callbacks(&raw mut s, c as *mut stbi_io_callbacks, user);
    stbi__info_main(&raw mut s, x, y, comp)
}
pub unsafe fn stbi_is_16_bit_from_memory(
    buffer: *const stbi_uc,
    len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_mem(&raw mut s, buffer, len);
    stbi__is_16_main(&raw mut s)
}
pub unsafe fn stbi_is_16_bit_from_callbacks(
    c: *const stbi_io_callbacks,
    user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_callbacks(&raw mut s, c as *mut stbi_io_callbacks, user);
    stbi__is_16_main(&raw mut s)
}
pub const STBI_MAX_DIMENSIONS: ::core::ffi::c_int = 4096;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
unsafe fn ParseConfigKeyPass(
    gs: *mut GlslShader,
    mut key: *const ::core::ffi::c_char,
    mut match_0: *const ::core::ffi::c_char,
) -> *mut GlslPass {
    let mut endp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    while *match_0 != 0 {
        if *key as ::core::ffi::c_int != *match_0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<GlslPass>();
        }
        key = key.offset(1);
        match_0 = match_0.offset(1);
    }
    if (*key as ::core::ffi::c_int - '0' as i32) as uint8 as ::core::ffi::c_int
        >= 10 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<GlslPass>();
    }
    let pass: uint = strtoul(key, &raw mut endp, 10 as ::core::ffi::c_int) as uint;
    if pass >= (*gs).n_pass as uint || *endp as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<GlslPass>();
    }
    (*gs).pass.offset(pass as isize).offset(1)
}
unsafe fn ParseScaleType(s: *const ::core::ffi::c_char) -> uint8 {
    ((if StringEqualsNoCase(s, b"source\0" as *const u8 as *const ::core::ffi::c_char)
        as ::core::ffi::c_int
        != 0
    {
        GLSL_SOURCE as ::core::ffi::c_int
    } else if StringEqualsNoCase(s, b"viewport\0" as *const u8 as *const ::core::ffi::c_char)
        as ::core::ffi::c_int
        != 0
    {
        GLSL_VIEWPORT as ::core::ffi::c_int
    } else if StringEqualsNoCase(s, b"absolute\0" as *const u8 as *const ::core::ffi::c_char)
        as ::core::ffi::c_int
        != 0
    {
        GLSL_ABSOLUTE as ::core::ffi::c_int
    } else {
        GLSL_NONE as ::core::ffi::c_int
    }) as uint8)
}
unsafe fn ParseWrapMode(s: *const ::core::ffi::c_char) -> uint {
    ((if StringEqualsNoCase(s, b"repeat\0" as *const u8 as *const ::core::ffi::c_char)
        as ::core::ffi::c_int
        != 0
    {
        GL_REPEAT
    } else if StringEqualsNoCase(
        s,
        b"clamp_to_edge\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int
        != 0
    {
        GL_CLAMP_TO_EDGE
    } else if StringEqualsNoCase(s, b"clamp\0" as *const u8 as *const ::core::ffi::c_char)
        as ::core::ffi::c_int
        != 0
    {
        GL_CLAMP
    } else {
        GL_CLAMP_TO_BORDER
    }) as uint)
}
unsafe fn GlslPass_Initialize(pass: *mut GlslPass) {
    (*pass).scale_x = 1.0f32;
    (*pass).scale_y = 1.0f32;
    (*pass).wrap_mode = GL_CLAMP_TO_BORDER as uint;
}
unsafe fn ParseTextures(gs: *mut GlslShader, mut value: *mut ::core::ffi::c_char) {
    let mut id: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut nextp: *mut *mut GlslTexture = &raw mut (*gs).first_texture;
    let mut num: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        id = NextDelim(&raw mut value, ';' as i32);
        if !(!id.is_null() && num < kGlslMaxTextures as ::core::ffi::c_int) {
            break;
        }
        let t: *mut GlslTexture =
            calloc(::core::mem::size_of::<GlslTexture>() as size_t, 1 as size_t)
                as *mut GlslTexture;
        (*t).id = strdup(id);
        (*t).wrap_mode = GL_CLAMP_TO_BORDER as uint;
        (*t).filter = GL_NEAREST as uint;
        *nextp = t;
        nextp = &raw mut (*t).next as *mut *mut GlslTexture;
        num += 1;
    }
}
unsafe fn ParseTextureKeyValue(
    gs: *mut GlslShader,
    key: *const ::core::ffi::c_char,
    value: *const ::core::ffi::c_char,
) -> bool {
    let mut t: *mut GlslTexture = (*gs).first_texture;
    while !t.is_null() {
        let key2: *const ::core::ffi::c_char = SkipPrefix(key, (*t).id);
        if !key2.is_null() {
            if *key2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                StrSet(&raw mut (*t).filename, value);
                return true_0 != 0;
            } else if strcmp(
                key2,
                b"_wrap_mode\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0
            {
                (*t).wrap_mode = ParseWrapMode(value);
                return true_0 != 0;
            } else if strcmp(
                key2,
                b"_mipmap\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0
            {
                (*t).mipmap = ParseBool(value, ::core::ptr::null_mut::<bool>());
                return true_0 != 0;
            } else if strcmp(
                key2,
                b"_linear\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0
            {
                (*t).filter = (if ParseBool(value, ::core::ptr::null_mut::<bool>())
                    as ::core::ffi::c_int
                    != 0
                {
                    GL_LINEAR
                } else {
                    GL_NEAREST
                }) as uint;
                return true_0 != 0;
            }
        }
        t = (*t).next as *mut GlslTexture;
    }
    false_0 != 0
}
unsafe fn GlslShader_GetParam(
    gs: *mut GlslShader,
    id: *const ::core::ffi::c_char,
) -> *mut GlslParam {
    let mut pp: *mut *mut GlslParam = &raw mut (*gs).first_param;
    while !(*pp).is_null() {
        if strcmp((**pp).id, id) == 0 {
            return *pp;
        }
        pp = &raw mut (**pp).next as *mut *mut GlslParam;
    }
    let p: *mut GlslParam =
        calloc(1 as size_t, ::core::mem::size_of::<GlslParam>() as size_t) as *mut GlslParam;
    *pp = p;
    (*p).id = strdup(id);
    p
}
unsafe fn ParseParameters(gs: *mut GlslShader, mut value: *mut ::core::ffi::c_char) {
    let mut id: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        id = NextDelim(&raw mut value, ';' as i32);
        if id.is_null() {
            break;
        }
        GlslShader_GetParam(gs, id);
    }
}
unsafe fn ParseParameterKeyValue(
    gs: *mut GlslShader,
    key: *const ::core::ffi::c_char,
    value: *const ::core::ffi::c_char,
) -> bool {
    let mut p: *mut GlslParam = (*gs).first_param;
    while !p.is_null() {
        if strcmp((*p).id, key) == 0 as ::core::ffi::c_int {
            (*p).value = atof(value) as ::core::ffi::c_float;
            (*p).has_value = true_0 != 0;
            return true_0 != 0;
        }
        p = (*p).next as *mut GlslParam;
    }
    false_0 != 0
}
unsafe fn GlslShader_InitializePasses(gs: *mut GlslShader, passes: ::core::ffi::c_int) {
    (*gs).n_pass = passes;
    (*gs).pass = calloc(
        ((*gs).n_pass + 1 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<GlslPass>() as size_t,
    ) as *mut GlslPass;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gs).n_pass {
        GlslPass_Initialize((*gs).pass.offset(i as isize).offset(1));
        i += 1;
    }
}
unsafe fn GlslShader_ReadPresetFile(
    gs: *mut GlslShader,
    filename: *const ::core::ffi::c_char,
) -> bool {
    let mut data: *mut ::core::ffi::c_char =
        ReadWholeFile(filename, ::core::ptr::null_mut::<size_t>()) as *mut ::core::ffi::c_char;
    let data_org: *mut ::core::ffi::c_char = data;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pass: *mut GlslPass = ::core::ptr::null_mut::<GlslPass>();
    if data.is_null() {
        return false_0 != 0;
    }
    let mut lineno: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    loop {
        line = NextLineStripComments(&raw mut data);
        if line.is_null() {
            break;
        }
        let mut value: *mut ::core::ffi::c_char = SplitKeyValue(line);
        let mut t: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if value.is_null() {
            if *line != 0 {
                fprintf(
                    stderr,
                    b"%s:%d: Expecting key=value\n\0" as *const u8 as *const ::core::ffi::c_char,
                    filename,
                    lineno,
                );
            }
        } else {
            if *value as ::core::ffi::c_int == '"' as i32 {
                value = value.offset(1);
                t = value;
                while *t as ::core::ffi::c_int != 0 && *t as ::core::ffi::c_int != '"' as i32 {
                    t = t.offset(1);
                }
                if *t != 0 {
                    *t = 0 as ::core::ffi::c_char;
                }
            }
            if (*gs).n_pass == 0 as ::core::ffi::c_int {
                if strcmp(
                    line,
                    b"shaders\0" as *const u8 as *const ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
                {
                    fprintf(
                        stderr,
                        b"%s:%d: Expecting 'shaders'\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        filename,
                        lineno,
                    );
                    break;
                } else {
                    let passes: ::core::ffi::c_int = strtoul(
                        value,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        10 as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int;
                    if passes < 1 as ::core::ffi::c_int
                        || passes > kGlslMaxPasses as ::core::ffi::c_int
                    {
                        break;
                    }
                    GlslShader_InitializePasses(gs, passes);
                }
            } else {
                pass = ParseConfigKeyPass(
                    gs,
                    line,
                    b"filter_linear\0" as *const u8 as *const ::core::ffi::c_char,
                );
                if !pass.is_null() {
                    (*pass).filter = (if ParseBool(value, ::core::ptr::null_mut::<bool>())
                        as ::core::ffi::c_int
                        != 0
                    {
                        GL_LINEAR
                    } else {
                        GL_NEAREST
                    }) as uint;
                } else {
                    pass = ParseConfigKeyPass(
                        gs,
                        line,
                        b"scale_type\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    if !pass.is_null() {
                        (*pass).scale_type_y = ParseScaleType(value);
                        (*pass).scale_type_x = (*pass).scale_type_y;
                    } else {
                        pass = ParseConfigKeyPass(
                            gs,
                            line,
                            b"scale_type_x\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                        if !pass.is_null() {
                            (*pass).scale_type_x = ParseScaleType(value);
                        } else {
                            pass = ParseConfigKeyPass(
                                gs,
                                line,
                                b"scale_type_y\0" as *const u8 as *const ::core::ffi::c_char,
                            );
                            if !pass.is_null() {
                                (*pass).scale_type_y = ParseScaleType(value);
                            } else {
                                pass = ParseConfigKeyPass(
                                    gs,
                                    line,
                                    b"scale\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                if !pass.is_null() {
                                    (*pass).scale_y = atof(value) as ::core::ffi::c_float;
                                    (*pass).scale_x = (*pass).scale_y;
                                } else {
                                    pass = ParseConfigKeyPass(
                                        gs,
                                        line,
                                        b"scale_x\0" as *const u8 as *const ::core::ffi::c_char,
                                    );
                                    if !pass.is_null() {
                                        (*pass).scale_x = atof(value) as ::core::ffi::c_float;
                                    } else {
                                        pass = ParseConfigKeyPass(
                                            gs,
                                            line,
                                            b"scale_y\0" as *const u8 as *const ::core::ffi::c_char,
                                        );
                                        if !pass.is_null() {
                                            (*pass).scale_y = atof(value) as ::core::ffi::c_float;
                                        } else {
                                            pass = ParseConfigKeyPass(
                                                gs,
                                                line,
                                                b"shader\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                            );
                                            if !pass.is_null() {
                                                StrSet(&raw mut (*pass).filename, value);
                                            } else {
                                                pass = ParseConfigKeyPass(
                                                    gs,
                                                    line,
                                                    b"wrap_mode\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                );
                                                if !pass.is_null() {
                                                    (*pass).wrap_mode = ParseWrapMode(value);
                                                } else {
                                                    pass = ParseConfigKeyPass(
                                                        gs,
                                                        line,
                                                        b"mipmap_input\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                    );
                                                    if !pass.is_null() {
                                                        (*pass).mipmap_input = ParseBool(
                                                            value,
                                                            ::core::ptr::null_mut::<bool>(),
                                                        );
                                                    } else {
                                                        pass = ParseConfigKeyPass(
                                                            gs,
                                                            line,
                                                            b"frame_count_mod\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                        );
                                                        if !pass.is_null() {
                                                            (*pass).frame_count_mod =
                                                                atoi(value) as uint;
                                                        } else {
                                                            pass = ParseConfigKeyPass(
                                                                gs,
                                                                line,
                                                                b"float_framebuffer\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                            if !pass.is_null() {
                                                                (*pass).float_framebuffer =
                                                                    ParseBool(
                                                                        value,
                                                                        ::core::ptr::null_mut::<bool>(
                                                                        ),
                                                                    );
                                                            } else {
                                                                pass = ParseConfigKeyPass(
                                                                    gs,
                                                                    line,
                                                                    b"srgb_framebuffer\0" as *const u8
                                                                        as *const ::core::ffi::c_char,
                                                                );
                                                                if !pass.is_null() {
                                                                    (*pass).srgb_framebuffer =
                                                                        ParseBool(
                                                                            value,
                                                                            ::core::ptr::null_mut::<
                                                                                bool,
                                                                            >(
                                                                            ),
                                                                        );
                                                                } else {
                                                                    pass = ParseConfigKeyPass(
                                                                        gs,
                                                                        line,
                                                                        b"alias\0" as *const u8 as *const ::core::ffi::c_char,
                                                                    );
                                                                    if pass.is_null() {
                                                                        if strcmp(
                                                                            line,
                                                                            b"textures\0" as *const u8 as *const ::core::ffi::c_char,
                                                                        ) == 0 as ::core::ffi::c_int
                                                                            && (*gs).first_texture.is_null()
                                                                        {
                                                                            ParseTextures(gs, value);
                                                                        } else if strcmp(
                                                                            line,
                                                                            b"parameters\0" as *const u8 as *const ::core::ffi::c_char,
                                                                        ) == 0 as ::core::ffi::c_int
                                                                        {
                                                                            ParseParameters(gs, value);
                                                                        } else if !ParseTextureKeyValue(gs, line, value)
                                                                            && !ParseParameterKeyValue(gs, line, value)
                                                                        {
                                                                            fprintf(
                                                                                stderr,
                                                                                b"%s:%d: Unknown key '%s'\n\0" as *const u8
                                                                                    as *const ::core::ffi::c_char,
                                                                                filename,
                                                                                lineno,
                                                                                line,
                                                                            );
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        lineno += 1;
    }
    free(data_org as *mut ::core::ffi::c_void);
    (*gs).n_pass != 0 as ::core::ffi::c_int
}
pub unsafe fn GlslShader_ReadShaderFile(
    gs: *mut GlslShader,
    filename: *const ::core::ffi::c_char,
    result: *mut ByteArray,
) {
    let mut data: *mut ::core::ffi::c_char =
        ReadWholeFile(filename, ::core::ptr::null_mut::<size_t>()) as *mut ::core::ffi::c_char;
    let data_org: *mut ::core::ffi::c_char = data;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if data.is_null() {
        fprintf(
            stderr,
            b"Unable to read file '%s'\n\0" as *const u8 as *const ::core::ffi::c_char,
            filename,
        );
        return;
    }
    loop {
        line = NextDelim(&raw mut data, '\n' as i32);
        if line.is_null() {
            break;
        }
        let linelen: size_t = strlen(line);
        if linelen >= 8 as size_t
            && memcmp(
                line as *const ::core::ffi::c_void,
                b"#include\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            let mut tt: *mut ::core::ffi::c_char = line.offset(8);
            let new_filename: *mut ::core::ffi::c_char =
                ReplaceFilenameWithNewPath(filename, NextPossiblyQuotedString(&raw mut tt));
            GlslShader_ReadShaderFile(gs, new_filename, result);
            free(new_filename as *mut ::core::ffi::c_void);
        } else if linelen >= 17 as size_t
            && memcmp(
                line as *const ::core::ffi::c_void,
                b"#pragma parameter\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                17 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            let mut tt_0: *mut ::core::ffi::c_char = line.offset(17);
            let param: *mut GlslParam =
                GlslShader_GetParam(gs, NextPossiblyQuotedString(&raw mut tt_0));
            NextPossiblyQuotedString(&raw mut tt_0);
            let value: ::core::ffi::c_float =
                atof(NextPossiblyQuotedString(&raw mut tt_0)) as ::core::ffi::c_float;
            if !(*param).has_value {
                (*param).value = value;
            }
            (*param).min = atof(NextPossiblyQuotedString(&raw mut tt_0)) as ::core::ffi::c_float;
            (*param).max = atof(NextPossiblyQuotedString(&raw mut tt_0)) as ::core::ffi::c_float;
        } else {
            *line.offset(linelen as isize) = '\n' as i32 as ::core::ffi::c_char;
            ByteArray_AppendData(
                result,
                line as *mut uint8,
                linelen.wrapping_add(1 as size_t),
            );
        }
    }
    free(data_org as *mut ::core::ffi::c_void);
}
unsafe fn GlslPass_Compile(
    p: *mut GlslPass,
    type_0: uint,
    data: *const uint8,
    size: size_t,
) -> bool {
    static mut kVertexPrefix: [::core::ffi::c_char; 42] = unsafe {
        ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
            *b"#define VERTEX\n#define PARAMETER_UNIFORM\n\0",
        )
    };
    static mut kFragmentPrefix: [::core::ffi::c_char; 44] = unsafe {
        ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
            *b"#define FRAGMENT\n#define PARAMETER_UNIFORM\n\0",
        )
    };
    let mut strings: [*const GLchar; 3] = [::core::ptr::null::<GLchar>(); 3];
    let mut lengths: [GLint; 3] = [0; 3];
    let mut buffer: [::core::ffi::c_char; 256] = [0; 256];
    let mut compile_status: GLint = 0 as GLint;
    let mut skip: size_t = 0 as size_t;
    if size < 8 as size_t
        || memcmp(
            data as *const ::core::ffi::c_void,
            b"#version\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            8 as size_t,
        ) != 0 as ::core::ffi::c_int
    {
        strings[0 as ::core::ffi::c_int as usize] =
            b"#version 330\n\0" as *const u8 as *const ::core::ffi::c_char as *const GLchar;
        lengths[0 as ::core::ffi::c_int as usize] =
            (::core::mem::size_of::<[::core::ffi::c_char; 14]>() as usize).wrapping_sub(1_usize)
                as GLint;
    } else {
        while skip < size && {
            let fresh53 = skip;
            skip = skip.wrapping_add(1);
            *data.offset(fresh53 as isize) as ::core::ffi::c_int != '\n' as i32
        } {}
        strings[0 as ::core::ffi::c_int as usize] = data as *mut ::core::ffi::c_char;
        lengths[0 as ::core::ffi::c_int as usize] = skip as ::core::ffi::c_int as GLint;
    }
    strings[1 as ::core::ffi::c_int as usize] = (if type_0 == GL_VERTEX_SHADER as uint {
        &raw const kVertexPrefix as *const ::core::ffi::c_char as *mut ::core::ffi::c_char
            as *const ::core::ffi::c_char
    } else {
        &raw const kFragmentPrefix as *const ::core::ffi::c_char
    }) as *const GLchar;
    lengths[1 as ::core::ffi::c_int as usize] = (if type_0 == GL_VERTEX_SHADER as uint {
        (::core::mem::size_of::<[::core::ffi::c_char; 42]>() as usize).wrapping_sub(1_usize)
    } else {
        (::core::mem::size_of::<[::core::ffi::c_char; 44]>() as usize).wrapping_sub(1_usize)
    }) as GLint;
    strings[2 as ::core::ffi::c_int as usize] = (data as *mut GLchar).offset(skip as isize);
    lengths[2 as ::core::ffi::c_int as usize] =
        size.wrapping_sub(skip) as ::core::ffi::c_int as GLint;
    let shader: uint = gl::CreateShader(type_0 as GLenum) as uint;
    gl::ShaderSource(
        shader as GLuint,
        3 as GLsizei,
        &raw mut strings as *mut *const GLchar,
        &raw mut lengths as *mut GLint,
    );
    gl::CompileShader(shader as GLuint);
    gl::GetShaderiv(
        shader as GLuint,
        GL_COMPILE_STATUS as GLenum,
        &raw mut compile_status,
    );
    buffer[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
    gl::GetShaderInfoLog(
        shader as GLuint,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as GLsizei,
        ::core::ptr::null_mut::<GLsizei>(),
        &raw mut buffer as *mut GLchar,
    );
    if compile_status != GL_TRUE
        || buffer[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0
    {
        fprintf(
            stderr,
            b"%s compiling %s shader in file '%s':\n%s\n\0" as *const u8
                as *const ::core::ffi::c_char,
            if compile_status != GL_TRUE {
                b"Error\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"While\0" as *const u8 as *const ::core::ffi::c_char
            },
            if type_0 == GL_VERTEX_SHADER as uint {
                b"vertex\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"fragment\0" as *const u8 as *const ::core::ffi::c_char
            },
            (*p).filename,
            &raw mut buffer as *mut ::core::ffi::c_char,
        );
    }
    if compile_status == GL_TRUE {
        gl::AttachShader((*p).gl_program as GLuint, shader as GLuint);
    }
    gl::DeleteShader(shader as GLuint);
    compile_status == GL_TRUE
}
unsafe fn GlslTextureUniform_Read(
    program: uint,
    prefix: *const ::core::ffi::c_char,
    i: ::core::ffi::c_int,
    result: *mut GlslTextureUniform,
) {
    let mut buf: [::core::ffi::c_char; 40] = [0; 40];
    let e: *mut ::core::ffi::c_char = (&raw mut buf as *mut ::core::ffi::c_char).offset((snprintf
        as unsafe extern "C" fn(
            *mut ::core::ffi::c_char,
            size_t,
            *const ::core::ffi::c_char,
            ...
        ) -> ::core::ffi::c_int)(
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 40]>() as size_t,
        if i >= 0 as ::core::ffi::c_int {
            b"%s%u\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"%s\0" as *const u8 as *const ::core::ffi::c_char
        },
        prefix,
        i,
    )
        as isize) as *mut ::core::ffi::c_char;
    memcpy(
        e as *mut ::core::ffi::c_void,
        b"Texture\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        8 as size_t,
    );
    (*result).Texture =
        gl::GetUniformLocation(program as GLuint, &raw mut buf as *mut ::core::ffi::c_char)
            as ::core::ffi::c_int;
    memcpy(
        e as *mut ::core::ffi::c_void,
        b"InputSize\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        10 as size_t,
    );
    (*result).InputSize =
        gl::GetUniformLocation(program as GLuint, &raw mut buf as *mut ::core::ffi::c_char)
            as ::core::ffi::c_int;
    memcpy(
        e as *mut ::core::ffi::c_void,
        b"TextureSize\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        12 as size_t,
    );
    (*result).TextureSize =
        gl::GetUniformLocation(program as GLuint, &raw mut buf as *mut ::core::ffi::c_char)
            as ::core::ffi::c_int;
    memcpy(
        e as *mut ::core::ffi::c_void,
        b"TexCoord\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        9 as size_t,
    );
    (*result).TexCoord =
        gl::GetAttribLocation(program as GLuint, &raw mut buf as *mut ::core::ffi::c_char)
            as ::core::ffi::c_int;
}
static mut kMvpMatrixOrtho: [::core::ffi::c_float; 16] = [
    2.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 2.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, -1.0f32,
    0.0f32, -1.0f32, -1.0f32, 0.0f32, 1.0f32,
];
unsafe fn GlslShader_GetUniforms(gs: *mut GlslShader) {
    let mut pass_idx: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    (*gs).max_prev_frame = 0 as ::core::ffi::c_int;
    let mut p: *mut GlslPass = (*gs).pass.offset(1);
    let p_end: *mut GlslPass = p.offset((*gs).n_pass as isize);
    while p < p_end {
        let program: uint = (*p).gl_program;
        gl::UseProgram(program as GLuint);
        let MVPMatrix: GLint = gl::GetUniformLocation(
            program as GLuint,
            b"MVPMatrix\0" as *const u8 as *const GLchar,
        );
        if MVPMatrix >= 0 as ::core::ffi::c_int {
            gl::UniformMatrix4fv(
                MVPMatrix,
                1 as GLsizei,
                GL_FALSE as GLboolean,
                &raw const kMvpMatrixOrtho as *const GLfloat,
            );
        }
        GlslTextureUniform_Read(
            program,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            -(1 as ::core::ffi::c_int),
            &raw mut (*p).unif.Top,
        );
        (*p).unif.OutputSize = gl::GetUniformLocation(
            program as GLuint,
            b"OutputSize\0" as *const u8 as *const GLchar,
        ) as ::core::ffi::c_int;
        (*p).unif.FrameCount = gl::GetUniformLocation(
            program as GLuint,
            b"FrameCount\0" as *const u8 as *const GLchar,
        ) as ::core::ffi::c_int;
        (*p).unif.FrameDirection = gl::GetUniformLocation(
            program as GLuint,
            b"FrameDirection\0" as *const u8 as *const GLchar,
        ) as ::core::ffi::c_int;
        (*p).unif.LUTTexCoord = gl::GetAttribLocation(
            program as GLuint,
            b"LUTTexCoord\0" as *const u8 as *const GLchar,
        ) as ::core::ffi::c_int;
        (*p).unif.VertexCoord = gl::GetAttribLocation(
            program as GLuint,
            b"VertexCoord\0" as *const u8 as *const GLchar,
        ) as ::core::ffi::c_int;
        GlslTextureUniform_Read(
            program,
            b"Orig\0" as *const u8 as *const ::core::ffi::c_char,
            -(1 as ::core::ffi::c_int),
            &raw mut (*p).unif.Orig,
        );
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < 7 as ::core::ffi::c_int {
            GlslTextureUniform_Read(
                program,
                b"Prev\0" as *const u8 as *const ::core::ffi::c_char,
                if j != 0 {
                    j
                } else {
                    -(1 as ::core::ffi::c_int)
                },
                (&raw mut (*p).unif.Prev as *mut GlslTextureUniform).offset(j as isize)
                    as *mut GlslTextureUniform,
            );
            if (*p).unif.Prev[j as usize].Texture >= 0 as ::core::ffi::c_int {
                (*gs).max_prev_frame = j + 1 as ::core::ffi::c_int;
            }
            j += 1;
        }
        let mut j_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j_0 < (*gs).n_pass {
            GlslTextureUniform_Read(
                program,
                b"Pass\0" as *const u8 as *const ::core::ffi::c_char,
                j_0,
                (&raw mut (*p).unif.Pass as *mut GlslTextureUniform).offset(j_0 as isize)
                    as *mut GlslTextureUniform,
            );
            GlslTextureUniform_Read(
                program,
                b"PassPrev\0" as *const u8 as *const ::core::ffi::c_char,
                j_0,
                (&raw mut (*p).unif.PassPrev as *mut GlslTextureUniform).offset(j_0 as isize)
                    as *mut GlslTextureUniform,
            );
            j_0 += 1;
        }
        let mut t: *mut GlslTexture = (*gs).first_texture;
        let mut j_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !t.is_null() {
            (*p).unif.Texture[j_1 as usize] =
                gl::GetUniformLocation(program as GLuint, (*t).id) as ::core::ffi::c_int;
            t = (*t).next as *mut GlslTexture;
            j_1 += 1;
        }
        let mut pa: *mut GlslParam = (*gs).first_param;
        while !pa.is_null() {
            (*pa).uniform[pass_idx as usize] =
                gl::GetUniformLocation(program as GLuint, (*pa).id) as uint;
            pa = (*pa).next as *mut GlslParam;
        }
        p = p.offset(1);
        pass_idx += 1;
    }
    gl::UseProgram(0 as GLuint);
}
unsafe fn IsGlslFilename(filename: *const ::core::ffi::c_char) -> bool {
    let len: size_t = strlen(filename);
    len >= 5 as size_t
        && memcmp(
            filename
                .offset(len as isize)
                .offset(-(5 as ::core::ffi::c_int as isize))
                as *const ::core::ffi::c_void,
            b".glsl\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
}
pub unsafe fn GlslShader_CreateFromFile(
    mut filename: *const ::core::ffi::c_char,
) -> *mut GlslShader {
    static mut kTexCoords: [GLfloat; 16] = [
        0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32, 1.0f32, 0.0f32, 1.0f32, 1.0f32,
        1.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32,
    ];
    let mut current_block: u64;
    let mut buffer: [::core::ffi::c_char; 256] = [0; 256];
    let mut link_status: GLint = 0;
    let mut shader_code: ByteArray = ByteArray {
        data: ::core::ptr::null_mut::<uint8>(),
        size: 0,
        capacity: 0,
    };
    let mut success: bool = false_0 != 0;
    let mut gs: *mut GlslShader =
        calloc(::core::mem::size_of::<GlslShader>() as size_t, 1 as size_t) as *mut GlslShader;
    if gs.is_null() {
        return gs;
    }
    if IsGlslFilename(filename) {
        GlslShader_InitializePasses(gs, 1 as ::core::ffi::c_int);
        let ref mut fresh0 = (*(*gs).pass.offset(1)).filename;
        *fresh0 = strdup(filename);
        filename = b"\0" as *const u8 as *const ::core::ffi::c_char;
        current_block = 1394248824506584008;
    } else if !GlslShader_ReadPresetFile(gs, filename) {
        fprintf(
            stderr,
            b"Unable to read file '%s'\n\0" as *const u8 as *const ::core::ffi::c_char,
            filename,
        );
        current_block = 13851546192323845927;
    } else {
        current_block = 1394248824506584008;
    }
    match current_block {
        1394248824506584008 => {
            let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            loop {
                if (i > (*gs).n_pass) {
                    current_block = 7056779235015430508;
                    break;
                }
                let p: *mut GlslPass = (*gs).pass.offset(i as isize);
                shader_code.size = 0 as size_t;
                if (*p).filename.is_null() {
                    fprintf(
                        stderr,
                        b"shader%d attribute missing\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        i - 1 as ::core::ffi::c_int,
                    );
                    current_block = 13851546192323845927;
                    break;
                } else {
                    let new_filename: *mut ::core::ffi::c_char =
                        ReplaceFilenameWithNewPath(filename, (*p).filename);
                    GlslShader_ReadShaderFile(gs, new_filename, &raw mut shader_code);
                    free(new_filename as *mut ::core::ffi::c_void);
                    if shader_code.size == 0 as size_t {
                        fprintf(
                            stderr,
                            b"Couldn't read shader in file '%s'\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*p).filename,
                        );
                        current_block = 13851546192323845927;
                        break;
                    } else {
                        (*p).gl_program = gl::CreateProgram() as uint;
                        if !GlslPass_Compile(
                            p,
                            GL_VERTEX_SHADER as uint,
                            shader_code.data,
                            shader_code.size,
                        ) || !GlslPass_Compile(
                            p,
                            GL_FRAGMENT_SHADER as uint,
                            shader_code.data,
                            shader_code.size,
                        ) {
                            current_block = 13851546192323845927;
                            break;
                        }
                        gl::LinkProgram((*p).gl_program as GLuint);
                        gl::GetProgramiv(
                            (*p).gl_program as GLuint,
                            GL_LINK_STATUS as GLenum,
                            &raw mut link_status,
                        );
                        buffer[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
                        gl::GetProgramInfoLog(
                            (*p).gl_program as GLuint,
                            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as GLsizei,
                            ::core::ptr::null_mut::<GLsizei>(),
                            &raw mut buffer as *mut GLchar,
                        );
                        if link_status != GL_TRUE
                            || buffer[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0
                        {
                            fprintf(
                                stderr,
                                b"%s linking shader in file '%s':\n%s\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                if link_status != GL_TRUE {
                                    b"Error\0" as *const u8 as *const ::core::ffi::c_char
                                } else {
                                    b"While\0" as *const u8 as *const ::core::ffi::c_char
                                },
                                (*p).filename,
                                &raw mut buffer as *mut ::core::ffi::c_char,
                            );
                        }
                        if link_status != GL_TRUE {
                            current_block = 13851546192323845927;
                            break;
                        }
                        gl::GenFramebuffers(1 as GLsizei, &raw mut (*p).gl_fbo);
                        gl::GenTextures(1 as GLsizei, &raw mut (*p).gl_texture);
                        i += 1;
                    }
                }
            }
            match current_block {
                13851546192323845927 => {}
                _ => {
                    let mut t: *mut GlslTexture = (*gs).first_texture;
                    while !t.is_null() {
                        gl::GenTextures(1 as GLsizei, &raw mut (*t).gl_texture);
                        gl::BindTexture(GL_TEXTURE_2D as GLenum, (*t).gl_texture as GLuint);
                        gl::TexParameteri(
                            GL_TEXTURE_2D as GLenum,
                            GL_TEXTURE_WRAP_S as GLenum,
                            (*t).wrap_mode as GLint,
                        );
                        gl::TexParameteri(
                            GL_TEXTURE_2D as GLenum,
                            GL_TEXTURE_WRAP_T as GLenum,
                            (*t).wrap_mode as GLint,
                        );
                        gl::TexParameteri(
                            GL_TEXTURE_2D as GLenum,
                            GL_TEXTURE_MAG_FILTER as GLenum,
                            (*t).filter as GLint,
                        );
                        gl::TexParameteri(
                            GL_TEXTURE_2D as GLenum,
                            GL_TEXTURE_MIN_FILTER as GLenum,
                            (if (*t).mipmap as ::core::ffi::c_int != 0 {
                                (if (*t).filter == GL_LINEAR as uint {
                                    GL_LINEAR_MIPMAP_LINEAR
                                } else {
                                    GL_NEAREST_MIPMAP_NEAREST
                                }) as uint
                            } else {
                                (*t).filter
                            }) as GLint,
                        );
                        if !(*t).filename.is_null() {
                            let new_filename_0: *mut ::core::ffi::c_char =
                                ReplaceFilenameWithNewPath(filename, (*t).filename);
                            let mut imw: ::core::ffi::c_int = 0;
                            let mut imh: ::core::ffi::c_int = 0;
                            let mut imn: ::core::ffi::c_int = 0;
                            let data: *mut ::core::ffi::c_uchar = stbi_load(
                                new_filename_0,
                                &raw mut imw,
                                &raw mut imh,
                                &raw mut imn,
                                0 as ::core::ffi::c_int,
                            )
                                as *mut ::core::ffi::c_uchar;
                            if data.is_null() {
                                fprintf(
                                    stderr,
                                    b"Unable to read PNG '%s'\n\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    new_filename_0,
                                );
                            } else {
                                gl::TexImage2D(
                                    GL_TEXTURE_2D as GLenum,
                                    0 as GLint,
                                    GL_RGBA,
                                    imw as GLsizei,
                                    imh as GLsizei,
                                    0 as GLint,
                                    (if imn == 4 as ::core::ffi::c_int {
                                        GL_RGBA
                                    } else if imn == 3 as ::core::ffi::c_int {
                                        GL_RGB
                                    } else {
                                        GL_LUMINANCE
                                    }) as GLenum,
                                    GL_UNSIGNED_BYTE as GLenum,
                                    data as *const ::core::ffi::c_void,
                                );
                            }
                            free(data as *mut ::core::ffi::c_void);
                            free(new_filename_0 as *mut ::core::ffi::c_void);
                        }
                        if (*t).mipmap {
                            gl::GenerateMipmap(GL_TEXTURE_2D as GLenum);
                        }
                        t = (*t).next as *mut GlslTexture;
                    }
                    let mut p_0: *mut GlslParam = (*gs).first_param;
                    while !p_0.is_null() {
                        (*p_0).value = if (*p_0).value < (*p_0).min {
                            (*p_0).min
                        } else if (*p_0).value > (*p_0).max {
                            (*p_0).max
                        } else {
                            (*p_0).value
                        };
                        p_0 = (*p_0).next as *mut GlslParam;
                    }
                    GlslShader_GetUniforms(gs);
                    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i_0 < (*gs).max_prev_frame {
                        gl::GenTextures(
                            1 as GLsizei,
                            &raw mut (*(&raw mut (*gs).prev_frame as *mut GlTextureWithSize)
                                .offset(
                                    (-i_0 - 1 as ::core::ffi::c_int & 7 as ::core::ffi::c_int)
                                        as isize,
                                ))
                            .gl_texture,
                        );
                        i_0 += 1;
                    }
                    gl::GenBuffers(1 as GLsizei, &raw mut (*gs).gl_vbo);
                    gl::BindBuffer(GL_ARRAY_BUFFER as GLenum, (*gs).gl_vbo as GLuint);
                    gl::BufferData(
                        GL_ARRAY_BUFFER as GLenum,
                        ::core::mem::size_of::<[GLfloat; 16]>() as GLsizeiptr,
                        &raw const kTexCoords as *const GLfloat as *const ::core::ffi::c_void,
                        GL_STATIC_DRAW as GLenum,
                    );
                    gl::BindBuffer(GL_ARRAY_BUFFER as GLenum, 0 as GLuint);
                    success = true_0 != 0;
                }
            }
        }
        _ => {}
    }
    if !success {
        GlslShader_Destroy(gs);
        gs = ::core::ptr::null_mut::<GlslShader>();
    }
    ByteArray_Destroy(&raw mut shader_code);
    gs
}
pub unsafe fn GlslShader_Destroy(gs: *mut GlslShader) {
    let mut p: *mut GlslPass = (*gs).pass.offset(1);
    let p_end: *mut GlslPass = p.offset((*gs).n_pass as isize);
    while p < p_end {
        gl::DeleteProgram((*p).gl_program as GLuint);
        gl::DeleteTextures(1 as GLsizei, &raw mut (*p).gl_texture);
        gl::DeleteFramebuffers(1 as GLsizei, &raw mut (*p).gl_fbo);
        free((*p).filename as *mut ::core::ffi::c_void);
        p = p.offset(1);
    }
    free((*gs).pass as *mut ::core::ffi::c_void);
    let mut t: *mut GlslTexture = ::core::ptr::null_mut::<GlslTexture>();
    loop {
        t = (*gs).first_texture;
        if t.is_null() {
            break;
        }
        (*gs).first_texture = (*t).next as *mut GlslTexture;
        gl::DeleteTextures(1 as GLsizei, &raw mut (*t).gl_texture);
        free((*t).id as *mut ::core::ffi::c_void);
        free((*t).filename as *mut ::core::ffi::c_void);
        free(t as *mut ::core::ffi::c_void);
    }
    let mut pp: *mut GlslParam = ::core::ptr::null_mut::<GlslParam>();
    loop {
        pp = (*gs).first_param;
        if pp.is_null() {
            break;
        }
        (*gs).first_param = (*pp).next as *mut GlslParam;
        free((*pp).id as *mut ::core::ffi::c_void);
        free(pp as *mut ::core::ffi::c_void);
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        gl::DeleteTextures(
            1 as GLsizei,
            &raw mut (*(&raw mut (*gs).prev_frame as *mut GlTextureWithSize).offset(i as isize))
                .gl_texture,
        );
        i += 1;
    }
    gl::DeleteBuffers(1 as GLsizei, &raw mut (*gs).gl_vbo);
    free(gs as *mut ::core::ffi::c_void);
}
unsafe fn RenderCtx_SetTexture(
    ctx: *mut RenderCtx,
    textureu: ::core::ffi::c_int,
    texture_id: uint,
) {
    if textureu >= 0 as ::core::ffi::c_int {
        gl::ActiveTexture((GL_TEXTURE0 as GLenum).wrapping_add((*ctx).texture_unit as GLenum));
        gl::BindTexture(GL_TEXTURE_2D as GLenum, texture_id as GLuint);
        let fresh55 = (*ctx).texture_unit;
        (*ctx).texture_unit = (*ctx).texture_unit.wrapping_add(1);
        gl::Uniform1i(textureu as GLint, fresh55 as GLint);
    }
}
unsafe fn RenderCtx_SetTexCoords(
    ctx: *mut RenderCtx,
    tex_coord: ::core::ffi::c_int,
    offset: uintptr_t,
) {
    if tex_coord >= 0 as ::core::ffi::c_int {
        '_c2rust_label: {
            if (*ctx).num_vaos < kMaxVaosInRenderCtx as ::core::ffi::c_int as uint {
            } else {
                __assert_fail(
                    b"ctx->num_vaos < kMaxVaosInRenderCtx\0" as *const u8
                        as *const ::core::ffi::c_char,
                    b"src/glsl_shader.c\0" as *const u8 as *const ::core::ffi::c_char,
                    467 as ::core::ffi::c_uint,
                    b"void RenderCtx_SetTexCoords(RenderCtx *, int, uintptr_t)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let fresh54 = (*ctx).num_vaos;
        (*ctx).num_vaos = (*ctx).num_vaos.wrapping_add(1);
        (*ctx).vaos[fresh54 as usize] = tex_coord as uint;
        gl::VertexAttribPointer(
            tex_coord as GLuint,
            2 as GLint,
            GL_FLOAT as GLenum,
            GL_FALSE as GLboolean,
            0 as GLsizei,
            offset as *mut ::core::ffi::c_void,
        );
        gl::EnableVertexAttribArray(tex_coord as GLuint);
    }
}
unsafe fn RenderCtx_SetGlslTextureUniform(
    ctx: *mut RenderCtx,
    u: *mut GlslTextureUniform,
    width: ::core::ffi::c_int,
    height: ::core::ffi::c_int,
    texture: uint,
) {
    let mut size: [::core::ffi::c_float; 2] = [
        width as ::core::ffi::c_float,
        height as ::core::ffi::c_float,
    ];
    RenderCtx_SetTexture(ctx, (*u).Texture, texture);
    if (*u).InputSize >= 0 as ::core::ffi::c_int {
        gl::Uniform2fv(
            (*u).InputSize as GLint,
            1 as GLsizei,
            &raw mut size as *mut ::core::ffi::c_float,
        );
    }
    if (*u).TextureSize >= 0 as ::core::ffi::c_int {
        gl::Uniform2fv(
            (*u).TextureSize as GLint,
            1 as GLsizei,
            &raw mut size as *mut ::core::ffi::c_float,
        );
    }
    RenderCtx_SetTexCoords(ctx, (*u).TexCoord, (*ctx).offset as uintptr_t);
}
unsafe fn GlslShader_SetShaderVars(
    gs: *mut GlslShader,
    ctx: *mut RenderCtx,
    pass: ::core::ffi::c_int,
) {
    let p: *mut GlslPass = (*gs).pass.offset(pass as isize) as *mut GlslPass;
    RenderCtx_SetGlslTextureUniform(
        ctx,
        &raw mut (*p).unif.Top,
        (*p.offset(-(1 as ::core::ffi::c_int) as isize)).width as ::core::ffi::c_int,
        (*p.offset(-(1 as ::core::ffi::c_int) as isize)).height as ::core::ffi::c_int,
        (*p.offset(-(1 as ::core::ffi::c_int) as isize)).gl_texture,
    );
    if (*p).unif.OutputSize >= 0 as ::core::ffi::c_int {
        let mut output_size: [::core::ffi::c_float; 2] = [
            (*p.offset(0)).width as ::core::ffi::c_float,
            (*p.offset(0)).height as ::core::ffi::c_float,
        ];
        gl::Uniform2fv(
            (*p).unif.OutputSize as GLint,
            1 as GLsizei,
            &raw mut output_size as *mut ::core::ffi::c_float,
        );
    }
    if (*p).unif.FrameCount >= 0 as ::core::ffi::c_int {
        gl::Uniform1i(
            (*p).unif.FrameCount as GLint,
            (if (*p).frame_count_mod != 0 {
                (*gs).frame_count.wrapping_rem((*p).frame_count_mod)
            } else {
                (*gs).frame_count
            }) as GLint,
        );
    }
    if (*p).unif.FrameDirection >= 0 as ::core::ffi::c_int {
        gl::Uniform1i((*p).unif.FrameDirection as GLint, 1 as GLint);
    }
    RenderCtx_SetTexCoords(ctx, (*p).unif.LUTTexCoord, (*ctx).offset as uintptr_t);
    RenderCtx_SetTexCoords(ctx, (*p).unif.VertexCoord, 0 as uintptr_t);
    RenderCtx_SetGlslTextureUniform(
        ctx,
        &raw mut (*p).unif.Orig,
        (*(*gs).pass.offset(0)).width as ::core::ffi::c_int,
        (*(*gs).pass.offset(0)).height as ::core::ffi::c_int,
        (*(*gs).pass.offset(0)).gl_texture,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gs).max_prev_frame {
        let t: *mut GlTextureWithSize = (&raw mut (*gs).prev_frame as *mut GlTextureWithSize)
            .offset(
                ((*gs)
                    .frame_count
                    .wrapping_sub(1 as uint)
                    .wrapping_sub(i as uint)
                    & 7 as uint) as isize,
            ) as *mut GlTextureWithSize;
        '_c2rust_label: {
            if (*t).gl_texture != 0 as uint {
            } else {
                __assert_fail(
                    b"t->gl_texture != 0\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/glsl_shader.c\0" as *const u8 as *const ::core::ffi::c_char,
                    504 as ::core::ffi::c_uint,
                    b"void GlslShader_SetShaderVars(GlslShader *, RenderCtx *, int)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if (*t).width != 0 {
            RenderCtx_SetGlslTextureUniform(
                ctx,
                (&raw mut (*p).unif.Prev as *mut GlslTextureUniform).offset(i as isize)
                    as *mut GlslTextureUniform,
                (*t).width as ::core::ffi::c_int,
                (*t).height as ::core::ffi::c_int,
                (*t).gl_texture,
            );
        }
        i += 1;
    }
    let mut tctr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut t_0: *mut GlslTexture = (*gs).first_texture;
    while !t_0.is_null() {
        RenderCtx_SetTexture(ctx, (*p).unif.Texture[tctr as usize], (*t_0).gl_texture);
        t_0 = (*t_0).next as *mut GlslTexture;
        tctr += 1;
    }
    let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i_0 < pass {
        RenderCtx_SetGlslTextureUniform(
            ctx,
            (&raw mut (*p).unif.Pass as *mut GlslTextureUniform).offset(i_0 as isize)
                as *mut GlslTextureUniform,
            (*(*gs).pass.offset(i_0 as isize)).width as ::core::ffi::c_int,
            (*(*gs).pass.offset(i_0 as isize)).height as ::core::ffi::c_int,
            (*(*gs).pass.offset(i_0 as isize)).gl_texture,
        );
        i_0 += 1;
    }
    let mut i_1: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i_1 < pass {
        RenderCtx_SetGlslTextureUniform(
            ctx,
            (&raw mut (*p).unif.PassPrev as *mut GlslTextureUniform).offset((pass - i_1) as isize)
                as *mut GlslTextureUniform,
            (*(*gs).pass.offset(i_1 as isize)).width as ::core::ffi::c_int,
            (*(*gs).pass.offset(i_1 as isize)).height as ::core::ffi::c_int,
            (*(*gs).pass.offset(i_1 as isize)).gl_texture,
        );
        i_1 += 1;
    }
    let mut pa: *mut GlslParam = (*gs).first_param;
    while !pa.is_null() {
        if (*pa).uniform[pass as usize] >= 0 as uint {
            gl::Uniform1f(
                (*pa).uniform[pass as usize] as GLint,
                (*pa).value as GLfloat,
            );
        }
        pa = (*pa).next as *mut GlslParam;
    }
    gl::ActiveTexture(GL_TEXTURE0 as GLenum);
}
pub unsafe fn GlslShader_Render(
    gs: *mut GlslShader,
    tex: *mut GlTextureWithSize,
    viewport_x: ::core::ffi::c_int,
    viewport_y: ::core::ffi::c_int,
    viewport_width: ::core::ffi::c_int,
    viewport_height: ::core::ffi::c_int,
) {
    (*(*gs).pass.offset(0)).gl_texture = (*tex).gl_texture;
    (*(*gs).pass.offset(0)).width = (*tex).width;
    (*(*gs).pass.offset(0)).height = (*tex).height;
    let mut previous_framebuffer: GLint = 0;
    gl::GetIntegerv(
        GL_FRAMEBUFFER_BINDING as GLenum,
        &raw mut previous_framebuffer,
    );
    gl::BindBuffer(GL_ARRAY_BUFFER as GLenum, (*gs).gl_vbo as GLuint);
    let mut pass: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while pass <= (*gs).n_pass {
        let last_pass: bool = pass == (*gs).n_pass;
        let p: *mut GlslPass = (*gs).pass.offset(pass as isize);
        match (*p).scale_type_x as ::core::ffi::c_int {
            3 => {
                (*p).width = (*p).scale_x as uint16;
            }
            1 => {
                (*p).width = ((*p.offset(-(1 as ::core::ffi::c_int) as isize)).width
                    as ::core::ffi::c_int as ::core::ffi::c_float
                    * (*p).scale_x) as uint16;
            }
            2 => {
                (*p).width = (viewport_width as ::core::ffi::c_float * (*p).scale_x) as uint16;
            }
            _ => {
                (*p).width = (if last_pass as ::core::ffi::c_int != 0 {
                    viewport_width as ::core::ffi::c_float
                } else {
                    (*p.offset(-(1 as ::core::ffi::c_int) as isize)).width as ::core::ffi::c_int
                        as ::core::ffi::c_float
                        * (*p).scale_x
                }) as uint16;
            }
        }
        match (*p).scale_type_y as ::core::ffi::c_int {
            3 => {
                (*p).height = (*p).scale_y as uint16;
            }
            1 => {
                (*p).height = ((*p.offset(-(1 as ::core::ffi::c_int) as isize)).height
                    as ::core::ffi::c_int as ::core::ffi::c_float
                    * (*p).scale_y) as uint16;
            }
            2 => {
                (*p).height = (viewport_height as ::core::ffi::c_float * (*p).scale_y) as uint16;
            }
            _ => {
                (*p).height = (if last_pass as ::core::ffi::c_int != 0 {
                    viewport_height as ::core::ffi::c_float
                } else {
                    (*p.offset(-(1 as ::core::ffi::c_int) as isize)).height as ::core::ffi::c_int
                        as ::core::ffi::c_float
                        * (*p).scale_y
                }) as uint16;
            }
        }
        if !last_pass {
            gl::BindTexture(GL_TEXTURE_2D as GLenum, (*p).gl_texture as GLuint);
            if (*p).srgb_framebuffer {
                gl::Enable(GL_FRAMEBUFFER_SRGB as GLenum);
                gl::TexImage2D(
                    GL_TEXTURE_2D as GLenum,
                    0 as GLint,
                    GL_SRGB8_ALPHA8,
                    (*p).width as GLsizei,
                    (*p).height as GLsizei,
                    0 as GLint,
                    GL_RGBA as GLenum,
                    GL_UNSIGNED_INT_8_8_8_8 as GLenum,
                    ::core::ptr::null::<::core::ffi::c_void>(),
                );
            } else {
                gl::TexImage2D(
                    GL_TEXTURE_2D as GLenum,
                    0 as GLint,
                    if (*p).float_framebuffer as ::core::ffi::c_int != 0 {
                        GL_RGBA32F
                    } else {
                        GL_RGBA
                    },
                    (*p).width as GLsizei,
                    (*p).height as GLsizei,
                    0 as GLint,
                    GL_RGBA as GLenum,
                    (if (*p).float_framebuffer as ::core::ffi::c_int != 0 {
                        GL_FLOAT
                    } else {
                        GL_UNSIGNED_INT_8_8_8_8
                    }) as GLenum,
                    ::core::ptr::null::<::core::ffi::c_void>(),
                );
            }
            gl::Viewport(
                0 as GLint,
                0 as GLint,
                (*p).width as GLsizei,
                (*p).height as GLsizei,
            );
            gl::BindFramebuffer(GL_FRAMEBUFFER as GLenum, (*p).gl_fbo as GLuint);
            gl::FramebufferTexture2D(
                GL_FRAMEBUFFER as GLenum,
                GL_COLOR_ATTACHMENT0 as GLenum,
                GL_TEXTURE_2D as GLenum,
                (*p).gl_texture as GLuint,
                0 as GLint,
            );
        } else {
            gl::BindFramebuffer(GL_FRAMEBUFFER as GLenum, previous_framebuffer as GLuint);
            gl::Viewport(
                viewport_x as GLint,
                viewport_y as GLint,
                viewport_width as GLsizei,
                viewport_height as GLsizei,
            );
        }
        gl::BindTexture(
            GL_TEXTURE_2D as GLenum,
            (*p.offset(-(1 as ::core::ffi::c_int) as isize)).gl_texture as GLuint,
        );
        let filter: uint = if (*p).filter != 0 {
            (*p).filter
        } else {
            (if last_pass as ::core::ffi::c_int != 0
                && g_config.linear_filtering as ::core::ffi::c_int != 0
            {
                GL_LINEAR
            } else {
                GL_NEAREST
            }) as uint
        };
        gl::TexParameteri(
            GL_TEXTURE_2D as GLenum,
            GL_TEXTURE_MAG_FILTER as GLenum,
            filter as GLint,
        );
        gl::TexParameteri(
            GL_TEXTURE_2D as GLenum,
            GL_TEXTURE_MIN_FILTER as GLenum,
            (if (*p).mipmap_input as ::core::ffi::c_int != 0 {
                (if filter == GL_LINEAR as uint {
                    GL_LINEAR_MIPMAP_LINEAR
                } else {
                    GL_NEAREST_MIPMAP_NEAREST
                }) as uint
            } else {
                filter
            }) as GLint,
        );
        gl::TexParameteri(
            GL_TEXTURE_2D as GLenum,
            GL_TEXTURE_WRAP_S as GLenum,
            (*p).wrap_mode as GLint,
        );
        gl::TexParameteri(
            GL_TEXTURE_2D as GLenum,
            GL_TEXTURE_WRAP_T as GLenum,
            (*p).wrap_mode as GLint,
        );
        if (*p).mipmap_input {
            gl::GenerateMipmap(GL_TEXTURE_2D as GLenum);
        }
        gl::UseProgram((*p).gl_program as GLuint);
        let mut ctx: RenderCtx = RenderCtx {
            texture_unit: 0,
            offset: 0,
            num_vaos: 0,
            vaos: [0; 51],
        };
        ctx.num_vaos = 0 as uint;
        ctx.texture_unit = ctx.num_vaos;
        ctx.offset = (if last_pass as ::core::ffi::c_int != 0 {
            (::core::mem::size_of::<::core::ffi::c_float>() as usize).wrapping_mul(8_usize)
        } else {
            0_usize
        }) as uint;
        GlslShader_SetShaderVars(gs, &raw mut ctx, pass);
        gl::DrawArrays(GL_TRIANGLE_STRIP as GLenum, 0 as GLint, 4 as GLsizei);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < ctx.num_vaos as ::core::ffi::c_int {
            gl::DisableVertexAttribArray(ctx.vaos[i as usize]);
            i += 1;
        }
        if (*p).srgb_framebuffer {
            gl::Disable(GL_FRAMEBUFFER_SRGB as GLenum);
        }
        pass += 1;
    }
    gl::BindFramebuffer(GL_FRAMEBUFFER as GLenum, previous_framebuffer as GLuint);
    gl::UseProgram(0 as GLuint);
    gl::BindTexture(GL_TEXTURE_2D as GLenum, 0 as GLuint);
    gl::BindBuffer(GL_ARRAY_BUFFER as GLenum, 0 as GLuint);
    if (*gs).max_prev_frame != 0 as ::core::ffi::c_int {
        let store_pos: *mut GlTextureWithSize = (&raw mut (*gs).prev_frame
            as *mut GlTextureWithSize)
            .offset(((*gs).frame_count & 7 as uint) as isize)
            as *mut GlTextureWithSize;
        let load_pos: *mut GlTextureWithSize = (&raw mut (*gs).prev_frame as *mut GlTextureWithSize)
            .offset(
                ((*gs).frame_count.wrapping_sub((*gs).max_prev_frame as uint) & 7 as uint) as isize,
            ) as *mut GlTextureWithSize;
        '_c2rust_label: {
            if (*store_pos).gl_texture == 0 as uint {
            } else {
                __assert_fail(
                    b"store_pos->gl_texture == 0\0" as *const u8
                        as *const ::core::ffi::c_char,
                    b"src/glsl_shader.c\0" as *const u8 as *const ::core::ffi::c_char,
                    609 as ::core::ffi::c_uint,
                    b"void GlslShader_Render(GlslShader *, GlTextureWithSize *, int, int, int, int)\0"
                        as *const u8 as *const ::core::ffi::c_char,
                );
            }
        };
        *store_pos = *tex;
        *tex = *load_pos;
        memset(
            load_pos as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<GlTextureWithSize>() as size_t,
        );
    }
    (*gs).frame_count = (*gs).frame_count.wrapping_add(1);
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647;
pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
