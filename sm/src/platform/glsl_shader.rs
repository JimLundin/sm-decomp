#[cfg_attr(unix, link(name = "m"))]
use crate::c_library::pow;
use crate::c_library::{
    __assert_fail, abs, calloc, fclose, feof, ferror, fgetc, fopen, fprintf, fread, free, fseek,
    ftell, malloc, memcmp, memcpy, memset, realloc, snprintf, stderr, strcmp, strdup, strlen,
    strtod, strtol, strtoul, ungetc,
};
use crate::config::{g_config, ParseBool};
use crate::set_fields;
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data};
pub use crate::types::{
    __off64_t, __off_t, ptrdiff_t, size_t, GLboolean, GLchar, GLenum, GLfloat, GLint, GLsizei,
    GLsizeiptr, GLuint,
};
use crate::util::{
    ByteArray_AppendData, ByteArray_Destroy, NextDelim, NextLineStripComments,
    NextPossiblyQuotedString, ReadWholeFile, ReplaceFilenameWithNewPath, SkipPrefix, SplitKeyValue,
    StrSet, StringEqualsNoCase,
};
use ::core::ffi::{c_char, c_double, c_float, c_long, c_uchar, c_ulong, c_void};
use ::core::mem::{size_of, transmute};
use ::core::ptr::{null, null_mut};
pub type C2RustUnnamed = u32;
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
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct stbi__context {
        pub img_x, img_y: stbi__uint32;
        pub img_n, img_out_n: i32;
        pub io: stbi_io_callbacks;
        pub io_user_data: *mut c_void;
        pub read_from_callbacks, buflen: i32;
        pub buffer_start: [stbi_uc; 128];
        pub callback_already_read: i32;
        pub img_buffer, img_buffer_end, img_buffer_original, img_buffer_original_end: *mut stbi_uc;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi_io_callbacks {
    pub read: Option<unsafe fn(*mut c_void, *mut c_char, i32) -> i32>,
    pub skip: Option<unsafe fn(*mut c_void, i32) -> ()>,
    pub eof: Option<unsafe fn(*mut c_void) -> i32>,
}
pub use crate::types::stbi__uint32;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct stbi__result_info {
        pub bits_per_channel, num_channels, channel_order: i32;
    }
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct stbi__png {
        pub s: *mut stbi__context;
        pub idata, expanded, out: *mut stbi_uc;
        pub depth: i32;
    }
}
pub use crate::types::stbi__uint16;
pub const STBI__SCAN_load: C2RustUnnamed_1 = 0;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct stbi__pngchunk {
        pub length, type_0: stbi__uint32;
    }
}
pub const STBI__F_paeth_first: C2RustUnnamed_2 = 6;
pub const STBI__F_avg_first: C2RustUnnamed_2 = 5;
pub const STBI__F_paeth: C2RustUnnamed_2 = 4;
pub const STBI__F_avg: C2RustUnnamed_2 = 3;
pub const STBI__F_up: C2RustUnnamed_2 = 2;
pub const STBI__F_sub: C2RustUnnamed_2 = 1;
pub const STBI__F_none: C2RustUnnamed_2 = 0;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct stbi__zbuf {
        pub zbuffer, zbuffer_end: *mut stbi_uc;
        pub num_bits: i32;
        pub code_buffer: stbi__uint32;
        pub zout, zout_start, zout_end: *mut c_char;
        pub z_expandable: i32;
        pub z_length, z_distance: stbi__zhuffman;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__zhuffman {
    pub fast: [stbi__uint16; 512],
    pub firstcode: [stbi__uint16; 16],
    pub maxcode: [i32; 17],
    pub firstsymbol: [stbi__uint16; 16],
    pub size: [stbi_uc; 288],
    pub value: [stbi__uint16; 288],
}
pub const STBI__SCAN_header: C2RustUnnamed_1 = 2;
pub const STBI__SCAN_type: C2RustUnnamed_1 = 1;
pub const STBI_ORDER_RGB: C2RustUnnamed_0 = 0;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct RenderCtx {
        pub texture_unit, offset, num_vaos: u32;
        pub vaos: [u32; 51];
    }
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
        s: null_mut::<stbi__context>(),
        idata: null_mut::<stbi_uc>(),
        expanded: null_mut::<stbi_uc>(),
        out: null_mut::<stbi_uc>(),
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
        io_user_data: null_mut::<c_void>(),
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: null_mut::<stbi_uc>(),
        img_buffer_end: null_mut::<stbi_uc>(),
        img_buffer_original: null_mut::<stbi_uc>(),
        img_buffer_original_end: null_mut::<stbi_uc>(),
    };
}
impl stbi__zbuf {
    const EMPTY: Self = stbi__zbuf {
        zbuffer: null_mut::<stbi_uc>(),
        zbuffer_end: null_mut::<stbi_uc>(),
        num_bits: 0,
        code_buffer: 0,
        zout: null_mut::<c_char>(),
        zout_start: null_mut::<c_char>(),
        zout_end: null_mut::<c_char>(),
        z_expandable: 0,
        z_length: stbi__zhuffman::EMPTY,
        z_distance: stbi__zhuffman::EMPTY,
    };
}
pub type C2RustUnnamed_0 = u32;
pub const STBI_ORDER_BGR: C2RustUnnamed_0 = 1;
pub type C2RustUnnamed_1 = u32;
pub type C2RustUnnamed_2 = u32;
pub type C2RustUnnamed_3 = u32;
pub const INT_MAX: i32 = __INT_MAX__;
pub const UINT_MAX: u32 = (__INT_MAX__ as u32).wrapping_mul(2u32).wrapping_add(1);
pub const GL_TEXTURE_2D: i32 = gl::TEXTURE_2D as i32;
pub const GL_CLAMP: i32 = 0x2900;
pub const GL_FALSE: i32 = gl::FALSE as i32;
pub const GL_FLOAT: i32 = gl::FLOAT as i32;
pub const GL_LINEAR: i32 = gl::LINEAR as i32;
pub const GL_LINEAR_MIPMAP_LINEAR: i32 = gl::LINEAR_MIPMAP_LINEAR as i32;
pub const GL_LUMINANCE: i32 = 0x1909;
pub const GL_NEAREST: i32 = gl::NEAREST as i32;
pub const GL_NEAREST_MIPMAP_NEAREST: i32 = gl::NEAREST_MIPMAP_NEAREST as i32;
pub const GL_REPEAT: i32 = gl::REPEAT as i32;
pub const GL_RGB: i32 = gl::RGB as i32;
pub const GL_RGBA: i32 = gl::RGBA as i32;
pub const GL_TEXTURE_MAG_FILTER: i32 = gl::TEXTURE_MAG_FILTER as i32;
pub const GL_TEXTURE_MIN_FILTER: i32 = gl::TEXTURE_MIN_FILTER as i32;
pub const GL_TEXTURE_WRAP_S: i32 = gl::TEXTURE_WRAP_S as i32;
pub const GL_TEXTURE_WRAP_T: i32 = gl::TEXTURE_WRAP_T as i32;
pub const GL_TRIANGLE_STRIP: i32 = gl::TRIANGLE_STRIP as i32;
pub const GL_TRUE: i32 = gl::TRUE as i32;
pub const GL_UNSIGNED_BYTE: i32 = gl::UNSIGNED_BYTE as i32;
pub const GL_CLAMP_TO_EDGE: i32 = gl::CLAMP_TO_EDGE as i32;
pub const GL_UNSIGNED_INT_8_8_8_8: i32 = gl::UNSIGNED_INT_8_8_8_8 as i32;
pub const GL_CLAMP_TO_BORDER: i32 = gl::CLAMP_TO_BORDER as i32;
pub const GL_TEXTURE0: i32 = gl::TEXTURE0 as i32;
pub const GL_ARRAY_BUFFER: i32 = gl::ARRAY_BUFFER as i32;
pub const GL_STATIC_DRAW: i32 = gl::STATIC_DRAW as i32;
pub const GL_COMPILE_STATUS: i32 = gl::COMPILE_STATUS as i32;
pub const GL_FRAGMENT_SHADER: i32 = gl::FRAGMENT_SHADER as i32;
pub const GL_LINK_STATUS: i32 = gl::LINK_STATUS as i32;
pub const GL_VERTEX_SHADER: i32 = gl::VERTEX_SHADER as i32;
pub const GL_SRGB8_ALPHA8: i32 = gl::SRGB8_ALPHA8 as i32;
pub const GL_COLOR_ATTACHMENT0: i32 = gl::COLOR_ATTACHMENT0 as i32;
pub const GL_FRAMEBUFFER: i32 = gl::FRAMEBUFFER as i32;
pub const GL_FRAMEBUFFER_BINDING: i32 = gl::FRAMEBUFFER_BINDING as i32;
pub const GL_FRAMEBUFFER_SRGB: i32 = gl::FRAMEBUFFER_SRGB as i32;
pub const GL_RGBA32F: i32 = gl::RGBA32F as i32;
#[inline]
unsafe fn atoi(mut __nptr: *const c_char) -> i32 {
    strtol(__nptr, NULL as *mut *mut c_char, 10) as i32
}
#[inline]
unsafe fn atof(mut __nptr: *const c_char) -> c_double {
    strtod(__nptr, NULL as *mut *mut c_char)
}
pub const EOF: i32 = -1i32;
pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
unsafe fn stbi__start_callbacks(
    s: *mut stbi__context,
    c: *mut stbi_io_callbacks,
    user: *mut c_void,
) {
    (*s).io = *c;
    (*s).io_user_data = user;
    (*s).buflen = size_of::<[stbi_uc; 128]>() as i32;
    (*s).read_from_callbacks = 1;
    (*s).callback_already_read = 0;
    (*s).img_buffer_original = &raw mut (*s).buffer_start as *mut stbi_uc;
    (*s).img_buffer = (*s).img_buffer_original;
    stbi__refill_buffer(s);
    (*s).img_buffer_original_end = (*s).img_buffer_end;
}
unsafe fn stbi__stdio_read(user: *mut c_void, data: *mut c_char, size: i32) -> i32 {
    fread(
        data as *mut c_void,
        1 as size_t,
        size as size_t,
        user as *mut FILE,
    ) as i32
}
unsafe fn stbi__stdio_skip(user: *mut c_void, n: i32) {
    fseek(user as *mut FILE, n as c_long, SEEK_CUR);
    let mut ch: i32 = fgetc(user as *mut FILE);
    if ch != EOF {
        ungetc(ch, user as *mut FILE);
    }
}
unsafe fn stbi__stdio_eof(user: *mut c_void) -> i32 {
    (feof(user as *mut FILE) != 0 || ferror(user as *mut FILE) != 0) as i32
}
static mut stbi__stdio_callbacks: stbi_io_callbacks = unsafe {
    stbi_io_callbacks {
        read: Some(stbi__stdio_read as unsafe fn(*mut c_void, *mut c_char, i32) -> i32),
        skip: Some(stbi__stdio_skip as unsafe fn(*mut c_void, i32) -> ()),
        eof: Some(stbi__stdio_eof as unsafe fn(*mut c_void) -> i32),
    }
};
unsafe fn stbi__start_file(s: *mut stbi__context, f: *mut FILE) {
    stbi__start_callbacks(s, &raw mut stbi__stdio_callbacks, f as *mut c_void);
}
unsafe fn stbi__rewind(s: *mut stbi__context) {
    (*s).img_buffer = (*s).img_buffer_original;
    (*s).img_buffer_end = (*s).img_buffer_original_end;
}
static mut stbi__g_failure_reason: *const c_char = null::<c_char>();
unsafe fn stbi__malloc(size: size_t) -> *mut c_void {
    malloc(size)
}
unsafe fn stbi__addsizes_valid(a: i32, b: i32) -> i32 {
    if b < 0 {
        return 0;
    }
    (a <= INT_MAX - b) as i32
}
unsafe fn stbi__mul2sizes_valid(a: i32, b: i32) -> i32 {
    if a < 0 || b < 0 {
        return 0;
    }
    if b == 0 {
        return 1;
    }
    (a <= INT_MAX / b) as i32
}
unsafe fn stbi__mad2sizes_valid(a: i32, b: i32, add: i32) -> i32 {
    (stbi__mul2sizes_valid(a, b) != 0 && stbi__addsizes_valid(a * b, add) != 0) as i32
}
unsafe fn stbi__mad3sizes_valid(a: i32, b: i32, c: i32, add: i32) -> i32 {
    (stbi__mul2sizes_valid(a, b) != 0
        && stbi__mul2sizes_valid(a * b, c) != 0
        && stbi__addsizes_valid(a * b * c, add) != 0) as i32
}
unsafe fn stbi__malloc_mad2(a: i32, b: i32, add: i32) -> *mut c_void {
    if stbi__mad2sizes_valid(a, b, add) == 0 {
        return NULL_0;
    }
    stbi__malloc((a * b + add) as size_t)
}
unsafe fn stbi__malloc_mad3(a: i32, b: i32, c: i32, add: i32) -> *mut c_void {
    if stbi__mad3sizes_valid(a, b, c, add) == 0 {
        return NULL_0;
    }
    stbi__malloc((a * b * c + add) as size_t)
}
static mut stbi__vertically_flip_on_load_global: i32 = 0;
unsafe fn stbi__load_main(
    s: *mut stbi__context,
    x: *mut i32,
    y: *mut i32,
    comp: *mut i32,
    req_comp: i32,
    ri: *mut stbi__result_info,
    bpc: i32,
) -> *mut c_void {
    memset(
        ri as *mut c_void,
        0,
        size_of::<stbi__result_info>() as size_t,
    );
    (*ri).bits_per_channel = 8;
    (*ri).channel_order = STBI_ORDER_RGB as i32;
    (*ri).num_channels = 0;
    if stbi__png_test(s) != 0 {
        return stbi__png_load(s, x, y, comp, req_comp, ri);
    }
    ((if 0 != 0 { NULL_0 } else { NULL_0 }) as size_t as *mut c_uchar as *mut c_void)
}
unsafe fn stbi__convert_16_to_8(
    orig: *mut stbi__uint16,
    w: i32,
    h: i32,
    channels: i32,
) -> *mut stbi_uc {
    let img_len: i32 = w * h * channels;
    let mut reduced = stbi__malloc(img_len as size_t) as *mut stbi_uc;
    if reduced.is_null() {
        return (if 0 != 0 { NULL_0 } else { NULL_0 }) as size_t as *mut stbi_uc;
    }
    let mut i: i32 = 0;
    while i < img_len {
        *reduced.offset(i as isize) = (*orig.offset(i as isize) as i32 >> 8 & 0xff) as stbi_uc;
        i += 1;
    }
    free(orig as *mut c_void);
    reduced
}
unsafe fn stbi__vertical_flip(image: *mut c_void, w: i32, h: i32, bytes_per_pixel: i32) {
    let bytes_per_row: size_t = (w as size_t).wrapping_mul(bytes_per_pixel as size_t);
    let mut temp: [stbi_uc; 2048] = [0; 2048];
    let bytes = image as *mut stbi_uc;
    let mut row: i32 = 0;
    while row < h >> 1 {
        let mut row0 = bytes.offset((row as size_t).wrapping_mul(bytes_per_row) as isize);
        let mut row1 =
            bytes.offset(((h - row - 1i32) as size_t).wrapping_mul(bytes_per_row) as isize);
        let mut bytes_left: size_t = bytes_per_row;
        while bytes_left != 0 {
            let bytes_copy: size_t = if bytes_left < size_of::<[stbi_uc; 2048]>() as usize {
                bytes_left
            } else {
                size_of::<[stbi_uc; 2048]>() as size_t
            };
            memcpy(
                &raw mut temp as *mut stbi_uc as *mut c_void,
                row0 as *const c_void,
                bytes_copy,
            );
            memcpy(row0 as *mut c_void, row1 as *const c_void, bytes_copy);
            memcpy(
                row1 as *mut c_void,
                &raw mut temp as *mut stbi_uc as *const c_void,
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
    x: *mut i32,
    y: *mut i32,
    comp: *mut i32,
    req_comp: i32,
) -> *mut c_uchar {
    let mut ri: stbi__result_info = stbi__result_info {
        bits_per_channel: 0,
        num_channels: 0,
        channel_order: 0,
    };
    let mut result = stbi__load_main(s, x, y, comp, req_comp, &raw mut ri, 8);
    if result.is_null() {
        return null_mut::<c_uchar>();
    }
    crate::native_assertion! {
        '_c2rust_label: ri.bits_per_channel == 8 || ri.bits_per_channel == 16;
        b"ri.bits_per_channel == 8 || ri.bits_per_channel == 16\0",
        b"./third_party/stb/stb_image.h\0", 1249,
        b"unsigned char *stbi__load_and_postprocess_8bit(stbi__context *, int *, int *, int *, int)\0"
    };
    if ri.bits_per_channel != 8 {
        result = stbi__convert_16_to_8(
            result as *mut stbi__uint16,
            *x,
            *y,
            if req_comp == 0 { *comp } else { req_comp },
        ) as *mut c_void;
        ri.bits_per_channel = 8;
    }
    if stbi__vertically_flip_on_load_global != 0 {
        let channels: i32 = if req_comp != 0 { req_comp } else { *comp };
        stbi__vertical_flip(
            result,
            *x,
            *y,
            (channels as usize).wrapping_mul(size_of::<stbi_uc>() as usize) as i32,
        );
    }
    result as *mut c_uchar
}
unsafe fn stbi__fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    let mut f = fopen(filename, mode) as *mut FILE;
    f
}
pub unsafe fn stbi_load(
    filename: *const c_char,
    x: *mut i32,
    y: *mut i32,
    comp: *mut i32,
    req_comp: i32,
) -> *mut stbi_uc {
    let f = stbi__fopen(filename, b"rb\0" as *const u8 as *const c_char);
    if f.is_null() {
        return (if 0 != 0 { NULL_0 } else { NULL_0 }) as size_t as *mut stbi_uc;
    }
    let mut result = stbi_load_from_file(f, x, y, comp, req_comp) as *mut c_uchar;
    fclose(f);
    result as *mut stbi_uc
}
pub unsafe fn stbi_load_from_file(
    f: *mut FILE,
    x: *mut i32,
    y: *mut i32,
    comp: *mut i32,
    req_comp: i32,
) -> *mut stbi_uc {
    let mut result = null_mut::<c_uchar>();
    let mut s: stbi__context = stbi__context::EMPTY;
    stbi__start_file(&raw mut s, f);
    result = stbi__load_and_postprocess_8bit(&raw mut s, x, y, comp, req_comp);
    if !result.is_null() {
        fseek(
            f,
            -(s.img_buffer_end.offset_from(s.img_buffer) as c_long as i32) as c_long,
            SEEK_CUR,
        );
    }
    result as *mut stbi_uc
}
static mut stbi__l2h_scale: c_float = 1.0f32;
static mut stbi__l2h_gamma: c_float = 2.2f32;
static mut stbi__h2l_scale_i: c_float = 1.0f32;
static mut stbi__h2l_gamma_i: c_float = 1.0f32 / 2.2f32;
unsafe fn stbi__refill_buffer(s: *mut stbi__context) {
    let n: i32 = (*s).io.read.expect("non-null function pointer")(
        (*s).io_user_data,
        &raw mut (*s).buffer_start as *mut stbi_uc as *mut c_char,
        (*s).buflen,
    );
    (*s).callback_already_read +=
        (*s).img_buffer.offset_from((*s).img_buffer_original) as c_long as i32;
    if n == 0 {
        (*s).read_from_callbacks = 0;
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
        crate::update_place! { previous fresh17 = (*s).img_buffer => (*s).img_buffer.offset(1); }
        return *fresh17;
    }
    if (*s).read_from_callbacks != 0 {
        stbi__refill_buffer(s);
        crate::update_place! { previous fresh18 = (*s).img_buffer => (*s).img_buffer.offset(1); }
        return *fresh18;
    }
    0 as stbi_uc
}
unsafe fn stbi__skip(s: *mut stbi__context, n: i32) {
    if n == 0 {
        return;
    }
    if n < 0 {
        (*s).img_buffer = (*s).img_buffer_end;
        return;
    }
    if (*s).io.read.is_some() {
        let blen: i32 = (*s).img_buffer_end.offset_from((*s).img_buffer) as c_long as i32;
        if blen < n {
            (*s).img_buffer = (*s).img_buffer_end;
            (*s).io.skip.expect("non-null function pointer")((*s).io_user_data, n - blen);
            return;
        }
    }
    (*s).img_buffer = (*s).img_buffer.offset(n as isize);
}
unsafe fn stbi__getn(s: *mut stbi__context, buffer: *mut stbi_uc, n: i32) -> i32 {
    if (*s).io.read.is_some() {
        let blen: i32 = (*s).img_buffer_end.offset_from((*s).img_buffer) as c_long as i32;
        if blen < n {
            memcpy(
                buffer as *mut c_void,
                (*s).img_buffer as *const c_void,
                blen as size_t,
            );
            let mut count: i32 = (*s).io.read.expect("non-null function pointer")(
                (*s).io_user_data,
                (buffer as *mut c_char).offset(blen as isize),
                n - blen,
            );
            let mut res: i32 = (count == n - blen) as i32;
            (*s).img_buffer = (*s).img_buffer_end;
            return res;
        }
    }
    if (*s).img_buffer.offset(n as isize) <= (*s).img_buffer_end {
        memcpy(
            buffer as *mut c_void,
            (*s).img_buffer as *const c_void,
            n as size_t,
        );
        (*s).img_buffer = (*s).img_buffer.offset(n as isize);
        1
    } else {
        0
    }
}
unsafe fn stbi__get16be(s: *mut stbi__context) -> i32 {
    let z: i32 = stbi__get8(s) as i32;
    (z << 8) + stbi__get8(s) as i32
}
unsafe fn stbi__get32be(s: *mut stbi__context) -> stbi__uint32 {
    let z: stbi__uint32 = stbi__get16be(s) as stbi__uint32;
    (z << 16i32).wrapping_add(stbi__get16be(s) as stbi__uint32)
}
// Channel values use the same integer luminance weights at both widths.
trait ImageChannel: Copy {
    const OPAQUE: Self;
    fn luminance(r: Self, g: Self, b: Self) -> Self;
}
impl ImageChannel for u8 {
    const OPAQUE: Self = u8::MAX;
    fn luminance(r: Self, g: Self, b: Self) -> Self {
        ((r as i32 * 77 + g as i32 * 150 + 29 * b as i32) >> 8) as Self
    }
}
impl ImageChannel for u16 {
    const OPAQUE: Self = u16::MAX;
    fn luminance(r: Self, g: Self, b: Self) -> Self {
        ((r as i32 * 77 + g as i32 * 150 + 29 * b as i32) >> 8) as Self
    }
}

// Allocation and failure diagnostics remain specific to each public width.
unsafe fn convert_image_channels<T: ImageChannel>(
    data: *mut T,
    good: *mut T,
    img_n: i32,
    req_comp: i32,
    x: u32,
    y: u32,
) -> bool {
    let mut j = 0;
    while j < y as i32 {
        let mut src = data.offset((j as u32).wrapping_mul(x).wrapping_mul(img_n as u32) as isize);
        let mut dest =
            good.offset((j as u32).wrapping_mul(x).wrapping_mul(req_comp as u32) as isize);
        let conversion = img_n * 8 + req_comp;
        // The original checks the conversion even when this row has no pixels.
        if !matches!(
            conversion,
            10 | 11 | 12 | 17 | 19 | 20 | 25 | 26 | 28 | 33 | 34 | 35
        ) {
            return false;
        }
        let mut i = x.wrapping_sub(1) as i32;
        while i >= 0 {
            match conversion {
                10 => {
                    *dest = *src;
                    *dest.offset(1) = T::OPAQUE;
                }
                11 => {
                    crate::update_place! { reference fresh9 = *dest.offset(2) => *src; }
                    crate::update_place! { reference fresh10 = *dest.offset(1) => *fresh9; }
                    *dest = *fresh10;
                }
                12 => {
                    crate::update_place! { reference fresh11 = *dest.offset(2) => *src; }
                    crate::update_place! { reference fresh12 = *dest.offset(1) => *fresh11; }
                    *dest = *fresh12;
                    *dest.offset(3) = T::OPAQUE;
                }
                17 => *dest = *src,
                19 => {
                    crate::update_place! { reference fresh13 = *dest.offset(2) => *src; }
                    crate::update_place! { reference fresh14 = *dest.offset(1) => *fresh13; }
                    *dest = *fresh14;
                }
                20 => {
                    crate::update_place! { reference fresh15 = *dest.offset(2) => *src; }
                    crate::update_place! { reference fresh16 = *dest.offset(1) => *fresh15; }
                    *dest = *fresh16;
                    *dest.offset(3) = *src.offset(1);
                }
                28 => {
                    *dest = *src;
                    *dest.offset(1) = *src.offset(1);
                    *dest.offset(2) = *src.offset(2);
                    *dest.offset(3) = T::OPAQUE;
                }
                25 => *dest = T::luminance(*src, *src.offset(1), *src.offset(2)),
                26 => {
                    *dest = T::luminance(*src, *src.offset(1), *src.offset(2));
                    *dest.offset(1) = T::OPAQUE;
                }
                33 => *dest = T::luminance(*src, *src.offset(1), *src.offset(2)),
                34 => {
                    *dest = T::luminance(*src, *src.offset(1), *src.offset(2));
                    *dest.offset(1) = *src.offset(3);
                }
                35 => {
                    *dest = *src;
                    *dest.offset(1) = *src.offset(1);
                    *dest.offset(2) = *src.offset(2);
                }
                _ => unreachable!(),
            }
            i -= 1;
            src = src.offset((conversion >> 3) as isize);
            dest = dest.offset((conversion & 7) as isize);
        }
        j += 1;
    }
    true
}

unsafe fn stbi__convert_format(
    data: *mut c_uchar,
    img_n: i32,
    req_comp: i32,
    x: u32,
    y: u32,
) -> *mut c_uchar {
    if req_comp == img_n {
        return data;
    }
    crate::native_assertion! {
        '_c2rust_label: req_comp >= 1 && req_comp <= 4;
        b"req_comp >= 1 && req_comp <= 4\0", b"./third_party/stb/stb_image.h\0", 1741,
        b"unsigned char *stbi__convert_format(unsigned char *, int, int, unsigned int, unsigned int)\0"
    };
    let mut good = stbi__malloc_mad3(req_comp, x as i32, y as i32, 0) as *mut c_uchar;
    if good.is_null() {
        free(data as *mut c_void);
        return (if 0 != 0 { NULL_0 } else { NULL_0 }) as size_t as *mut c_uchar;
    }
    if !convert_image_channels(data, good, img_n, req_comp, x, y) {
        crate::native_assertion! {
            '_c2rust_label_0;
            b"0\0", b"./third_party/stb/stb_image.h\0", 1770,
            b"unsigned char *stbi__convert_format(unsigned char *, int, int, unsigned int, unsigned int)\0"
        };
        free(data as *mut c_void);
        free(good as *mut c_void);
        return (if 0 != 0 { NULL_0 } else { NULL_0 }) as size_t as *mut c_uchar;
    }
    free(data as *mut c_void);
    good
}
unsafe fn stbi__convert_format16(
    data: *mut stbi__uint16,
    img_n: i32,
    req_comp: i32,
    x: u32,
    y: u32,
) -> *mut stbi__uint16 {
    if req_comp == img_n {
        return data;
    }
    crate::native_assertion! {
        '_c2rust_label: req_comp >= 1 && req_comp <= 4;
        b"req_comp >= 1 && req_comp <= 4\0", b"./third_party/stb/stb_image.h\0", 1798,
        b"stbi__uint16 *stbi__convert_format16(stbi__uint16 *, int, int, unsigned int, unsigned int)\0"
    };
    let mut good = stbi__malloc(
        (req_comp as u32)
            .wrapping_mul(x)
            .wrapping_mul(y)
            .wrapping_mul(2) as size_t,
    ) as *mut stbi__uint16;
    if good.is_null() {
        free(data as *mut c_void);
        return (if 0 != 0 { NULL_0 } else { NULL_0 }) as size_t as *mut c_uchar
            as *mut stbi__uint16;
    }
    if !convert_image_channels(data, good, img_n, req_comp, x, y) {
        crate::native_assertion! {
            '_c2rust_label_0;
            b"0\0", b"./third_party/stb/stb_image.h\0", 1827,
            b"stbi__uint16 *stbi__convert_format16(stbi__uint16 *, int, int, unsigned int, unsigned int)\0"
        };
        free(data as *mut c_void);
        free(good as *mut c_void);
        return (if 0 != 0 { NULL_0 } else { NULL_0 }) as size_t as *mut c_uchar
            as *mut stbi__uint16;
    }
    free(data as *mut c_void);
    good
}
pub const STBI__ZFAST_BITS: i32 = 9;
pub const STBI__ZFAST_MASK: i32 = ((1i32) << STBI__ZFAST_BITS) - 1i32;
pub const STBI__ZNSYMS: i32 = 288;
unsafe fn stbi__bitreverse16(mut n: i32) -> i32 {
    n = (n & 0xaaaa) >> 1 | (n & 0x5555) << 1;
    n = (n & 0xcccc) >> 2 | (n & 0x3333) << 2;
    n = (n & 0xf0f0) >> 4 | (n & 0xf0f) << 4;
    n = (n & 0xff00) >> 8 | (n & 0xff) << 8;
    n
}
unsafe fn stbi__bit_reverse(v: i32, bits: i32) -> i32 {
    crate::native_assertion! {
        '_c2rust_label: bits <= 16;
        b"bits <= 16\0", b"./third_party/stb/stb_image.h\0", 4061,
        b"int stbi__bit_reverse(int, int)\0"
    };
    stbi__bitreverse16(v) >> 16 - bits
}
unsafe fn stbi__zbuild_huffman(z: *mut stbi__zhuffman, sizelist: *const stbi_uc, num: i32) -> i32 {
    let mut k: i32 = 0;
    let mut next_code: [i32; 16] = [0; 16];
    let mut sizes: [i32; 17] = [0; 17];
    memset(
        &raw mut sizes as *mut i32 as *mut c_void,
        0,
        size_of::<[i32; 17]>() as size_t,
    );
    memset(
        &raw mut (*z).fast as *mut stbi__uint16 as *mut c_void,
        0,
        size_of::<[stbi__uint16; 512]>() as size_t,
    );
    let mut i: i32 = 0;
    while i < num {
        sizes[*sizelist.offset(i as isize) as usize] += 1;
        i += 1;
    }
    sizes[0i32 as usize] = 0;
    i = 1;
    while i < 16 {
        if sizes[i as usize] > (1i32) << i {
            return 0;
        }
        i += 1;
    }
    let mut code: i32 = 0;
    i = 1;
    while i < 16 {
        next_code[i as usize] = code;
        (*z).firstcode[i as usize] = code as stbi__uint16;
        (*z).firstsymbol[i as usize] = k as stbi__uint16;
        code = code + sizes[i as usize];
        if sizes[i as usize] != 0 && code - 1i32 >= (1i32) << i {
            return 0;
        }
        (*z).maxcode[i as usize] = code << 16 - i;
        code <<= 1;
        k += sizes[i as usize];
        i += 1;
    }
    (*z).maxcode[16i32 as usize] = 0x10000;
    i = 0;
    while i < num {
        let s: i32 = *sizelist.offset(i as isize) as i32;
        if s != 0 {
            let c: i32 = next_code[s as usize] - (*z).firstcode[s as usize] as i32
                + (*z).firstsymbol[s as usize] as i32;
            let fastv: stbi__uint16 = (s << 9 | i) as stbi__uint16;
            (*z).size[c as usize] = s as stbi_uc;
            (*z).value[c as usize] = i as stbi__uint16;
            if s <= STBI__ZFAST_BITS {
                let mut j: i32 = stbi__bit_reverse(next_code[s as usize], s);
                while j < (1i32) << STBI__ZFAST_BITS {
                    (*z).fast[j as usize] = fastv;
                    j += (1i32) << s;
                }
            }
            next_code[s as usize] += 1;
        }
        i += 1;
    }
    1
}
unsafe fn stbi__zeof(z: *mut stbi__zbuf) -> i32 {
    ((*z).zbuffer >= (*z).zbuffer_end) as i32
}
unsafe fn stbi__zget8(z: *mut stbi__zbuf) -> stbi_uc {
    ((if stbi__zeof(z) != 0 {
        0
    } else {
        crate::update_place! { previous fresh49 = (*z).zbuffer => (*z).zbuffer.offset(1); }
        *fresh49 as i32
    }) as stbi_uc)
}
unsafe fn stbi__fill_bits(z: *mut stbi__zbuf) {
    loop {
        if (*z).code_buffer >= (1 as stbi__uint32) << (*z).num_bits {
            (*z).zbuffer = (*z).zbuffer_end;
            return;
        }
        (*z).code_buffer =
            ((*z).code_buffer | (stbi__zget8(z) as u32) << (*z).num_bits) as stbi__uint32;
        (*z).num_bits += 8;
        if ((*z).num_bits > 24) {
            break;
        }
    }
}
unsafe fn stbi__zreceive(z: *mut stbi__zbuf, n: i32) -> u32 {
    if (*z).num_bits < n {
        stbi__fill_bits(z);
    }
    let mut k: u32 = ((*z).code_buffer & (((1i32) << n) - 1i32) as stbi__uint32);
    (*z).code_buffer >>= n;
    (*z).num_bits -= n;
    k
}
unsafe fn stbi__zhuffman_decode_slowpath(a: *mut stbi__zbuf, z: *mut stbi__zhuffman) -> i32 {
    let mut k: i32 = stbi__bit_reverse((*a).code_buffer as i32, 16);
    let mut s: i32 = STBI__ZFAST_BITS + 1;
    while (k >= (*z).maxcode[s as usize]) {
        s += 1;
    }
    if s >= 16 {
        return -1i32;
    }
    let mut b: i32 =
        (k >> 16 - s) - (*z).firstcode[s as usize] as i32 + (*z).firstsymbol[s as usize] as i32;
    if b >= STBI__ZNSYMS {
        return -1i32;
    }
    if (*z).size[b as usize] as i32 != s {
        return -1i32;
    }
    (*a).code_buffer >>= s;
    (*a).num_bits -= s;
    (*z).value[b as usize] as i32
}
unsafe fn stbi__zhuffman_decode(a: *mut stbi__zbuf, z: *mut stbi__zhuffman) -> i32 {
    if (*a).num_bits < 16 {
        if stbi__zeof(a) != 0 {
            return -1i32;
        }
        stbi__fill_bits(a);
    }
    let mut b: i32 =
        (*z).fast[((*a).code_buffer & STBI__ZFAST_MASK as stbi__uint32) as usize] as i32;
    if b != 0 {
        let mut s: i32 = b >> 9;
        (*a).code_buffer >>= s;
        (*a).num_bits -= s;
        return b & 511;
    }
    stbi__zhuffman_decode_slowpath(a, z)
}
unsafe fn stbi__zexpand(z: *mut stbi__zbuf, zout: *mut c_char, n: i32) -> i32 {
    (*z).zout = zout;
    if (*z).z_expandable == 0 {
        return 0;
    }
    let mut cur: u32 = (*z).zout.offset_from((*z).zout_start) as c_long as u32;
    let mut old_limit: u32 = (*z).zout_end.offset_from((*z).zout_start) as c_long as u32;
    let mut limit: u32 = old_limit;
    if UINT_MAX.wrapping_sub(cur) < n as u32 {
        return 0;
    }
    while cur.wrapping_add(n as u32) > limit {
        if limit > UINT_MAX.wrapping_div(2) {
            return 0;
        }
        limit = limit.wrapping_mul(2);
    }
    let mut q = realloc((*z).zout_start as *mut c_void, limit as size_t) as *mut c_char;
    if q.is_null() {
        return 0;
    }
    (*z).zout_start = q;
    (*z).zout = q.offset(cur as isize);
    (*z).zout_end = q.offset(limit as isize);
    1
}
static mut stbi__zlength_base: [i32; 31] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258, 0, 0,
];
static mut stbi__zlength_extra: [i32; 31] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 0, 0,
];
static mut stbi__zdist_base: [i32; 32] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577, 0, 0,
];
static mut stbi__zdist_extra: [i32; 32] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13, 0, 0,
];
unsafe fn stbi__parse_huffman_block(a: *mut stbi__zbuf) -> i32 {
    let mut zout = (*a).zout;
    loop {
        let mut z: i32 = stbi__zhuffman_decode(a, &raw mut (*a).z_length);
        if z < 256 {
            if z < 0 {
                return 0;
            }
            if zout >= (*a).zout_end {
                if stbi__zexpand(a, zout, 1) == 0 {
                    return 0;
                }
                zout = (*a).zout;
            }
            crate::update_place! { previous fresh45 = zout => zout.offset(1); }
            *fresh45 = z as c_char;
        } else {
            if z == 256 {
                (*a).zout = zout;
                return 1;
            }
            z -= 257;
            let mut len: i32 = stbi__zlength_base[z as usize];
            if stbi__zlength_extra[z as usize] != 0 {
                len = (len as u32).wrapping_add(stbi__zreceive(a, stbi__zlength_extra[z as usize]))
                    as i32;
            }
            z = stbi__zhuffman_decode(a, &raw mut (*a).z_distance);
            if z < 0 {
                return 0;
            }
            let mut dist: i32 = stbi__zdist_base[z as usize];
            if stbi__zdist_extra[z as usize] != 0 {
                dist = (dist as u32).wrapping_add(stbi__zreceive(a, stbi__zdist_extra[z as usize]))
                    as i32;
            }
            if (zout.offset_from((*a).zout_start) as c_long) < dist as c_long {
                return 0;
            }
            if zout.offset(len as isize) > (*a).zout_end {
                if stbi__zexpand(a, zout, len) == 0 {
                    return 0;
                }
                zout = (*a).zout;
            }
            let mut p = zout.offset(-(dist as isize)) as *mut stbi_uc;
            if dist == 1 {
                let v: stbi_uc = *p;
                if len != 0 {
                    loop {
                        crate::update_place! { previous fresh46 = zout => zout.offset(1); }
                        *fresh46 = v as c_char;
                        len -= 1;
                        if (len == 0) {
                            break;
                        }
                    }
                }
            } else if len != 0 {
                loop {
                    crate::update_place! { previous fresh47 = p => p.offset(1); }
                    crate::update_place! { previous fresh48 = zout => zout.offset(1); }
                    *fresh48 = *fresh47 as c_char;
                    len -= 1;
                    if (len == 0) {
                        break;
                    }
                }
            }
        }
    }
}
unsafe fn stbi__compute_huffman_codes(a: *mut stbi__zbuf) -> i32 {
    static mut length_dezigzag: [stbi_uc; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let mut z_codelength: stbi__zhuffman = stbi__zhuffman::EMPTY;
    let mut lencodes: [stbi_uc; 455] = [0; 455];
    let mut codelength_sizes: [stbi_uc; 19] = [0; 19];
    let hlit: i32 = stbi__zreceive(a, 5i32).wrapping_add(257) as i32;
    let hdist: i32 = stbi__zreceive(a, 5i32).wrapping_add(1) as i32;
    let hclen: i32 = stbi__zreceive(a, 4i32).wrapping_add(4) as i32;
    let ntot: i32 = hlit + hdist;
    memset(
        &raw mut codelength_sizes as *mut stbi_uc as *mut c_void,
        0,
        size_of::<[stbi_uc; 19]>() as size_t,
    );
    let mut i: i32 = 0;
    while i < hclen {
        let s: i32 = stbi__zreceive(a, 3) as i32;
        codelength_sizes[length_dezigzag[i as usize] as usize] = s as stbi_uc;
        i += 1;
    }
    if stbi__zbuild_huffman(
        &raw mut z_codelength,
        &raw mut codelength_sizes as *mut stbi_uc,
        19,
    ) == 0
    {
        return 0;
    }
    let mut n: i32 = 0;
    while n < ntot {
        let mut c: i32 = stbi__zhuffman_decode(a, &raw mut z_codelength);
        if c < 0 || c >= 19 {
            return 0;
        }
        if c < 16 {
            crate::update_place! { previous fresh50 = n => n + 1; }
            lencodes[fresh50 as usize] = c as stbi_uc;
        } else {
            let mut fill: stbi_uc = 0 as stbi_uc;
            if c == 16 {
                c = stbi__zreceive(a, 2i32).wrapping_add(3) as i32;
                if n == 0 {
                    return 0;
                }
                fill = lencodes[(n - 1i32) as usize];
            } else if c == 17 {
                c = stbi__zreceive(a, 3i32).wrapping_add(3) as i32;
            } else if c == 18 {
                c = stbi__zreceive(a, 7i32).wrapping_add(11) as i32;
            } else {
                return 0;
            }
            if ntot - n < c {
                return 0;
            }
            memset(
                (&raw mut lencodes as *mut stbi_uc).offset(n as isize) as *mut c_void,
                fill as i32,
                c as size_t,
            );
            n += c;
        }
    }
    if n != ntot {
        return 0;
    }
    if stbi__zbuild_huffman(
        &raw mut (*a).z_length,
        &raw mut lencodes as *mut stbi_uc,
        hlit,
    ) == 0
    {
        return 0;
    }
    if stbi__zbuild_huffman(
        &raw mut (*a).z_distance,
        (&raw mut lencodes as *mut stbi_uc).offset(hlit as isize),
        hdist,
    ) == 0
    {
        return 0;
    }
    1
}
unsafe fn stbi__parse_uncompressed_block(a: *mut stbi__zbuf) -> i32 {
    let mut header: [stbi_uc; 4] = [0; 4];
    if (*a).num_bits & 7 != 0 {
        stbi__zreceive(a, (*a).num_bits & 7);
    }
    let mut k: i32 = 0;
    while (*a).num_bits > 0 {
        crate::update_place! { previous fresh51 = k => k + 1; }
        header[fresh51 as usize] = ((*a).code_buffer & 255 as stbi__uint32) as stbi_uc;
        (*a).code_buffer >>= 8;
        (*a).num_bits -= 8;
    }
    if (*a).num_bits < 0 {
        return 0;
    }
    while k < 4 {
        crate::update_place! { previous fresh52 = k => k + 1; }
        header[fresh52 as usize] = stbi__zget8(a);
    }
    let mut len: i32 = header[1i32 as usize] as i32 * 256 + header[0i32 as usize] as i32;
    let mut nlen: i32 = header[3i32 as usize] as i32 * 256 + header[2i32 as usize] as i32;
    if nlen != len ^ 0xffff {
        return 0;
    }
    if (*a).zbuffer.offset(len as isize) > (*a).zbuffer_end {
        return 0;
    }
    if (*a).zout.offset(len as isize) > (*a).zout_end && stbi__zexpand(a, (*a).zout, len) == 0 {
        return 0;
    }
    memcpy(
        (*a).zout as *mut c_void,
        (*a).zbuffer as *const c_void,
        len as size_t,
    );
    (*a).zbuffer = (*a).zbuffer.offset(len as isize);
    (*a).zout = (*a).zout.offset(len as isize);
    1
}
unsafe fn stbi__parse_zlib_header(a: *mut stbi__zbuf) -> i32 {
    let cmf: i32 = stbi__zget8(a) as i32;
    let cm: i32 = cmf & 15;
    let flg: i32 = stbi__zget8(a) as i32;
    if stbi__zeof(a) != 0 {
        return 0;
    }
    if (cmf * 256 + flg) % 31 != 0 {
        return 0;
    }
    if flg & 32 != 0 {
        return 0;
    }
    if cm != 8 {
        return 0;
    }
    1
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
unsafe fn stbi__parse_zlib(a: *mut stbi__zbuf, parse_header: i32) -> i32 {
    if parse_header != 0 && stbi__parse_zlib_header(a) == 0 {
        return 0;
    }
    (*a).num_bits = 0;
    (*a).code_buffer = 0 as stbi__uint32;
    loop {
        let mut final_0: i32 = stbi__zreceive(a, 1) as i32;
        let mut type_0: i32 = stbi__zreceive(a, 2) as i32;
        if type_0 == 0 {
            if stbi__parse_uncompressed_block(a) == 0 {
                return 0;
            }
        } else if type_0 == 3 {
            return 0;
        } else {
            if type_0 == 1 {
                if stbi__zbuild_huffman(
                    &raw mut (*a).z_length,
                    &raw const stbi__zdefault_length as *const stbi_uc,
                    STBI__ZNSYMS,
                ) == 0
                {
                    return 0;
                }
                if stbi__zbuild_huffman(
                    &raw mut (*a).z_distance,
                    &raw const stbi__zdefault_distance as *const stbi_uc,
                    32,
                ) == 0
                {
                    return 0;
                }
            } else if stbi__compute_huffman_codes(a) == 0 {
                return 0;
            }
            if stbi__parse_huffman_block(a) == 0 {
                return 0;
            }
        }
        if (final_0 != 0) {
            break;
        }
    }
    1
}
unsafe fn stbi__do_zlib(
    a: *mut stbi__zbuf,
    obuf: *mut c_char,
    olen: i32,
    exp: i32,
    parse_header: i32,
) -> i32 {
    (*a).zout_start = obuf;
    (*a).zout = obuf;
    (*a).zout_end = obuf.offset(olen as isize);
    (*a).z_expandable = exp;
    stbi__parse_zlib(a, parse_header)
}
pub unsafe fn stbi_zlib_decode_malloc_guesssize_headerflag(
    buffer: *const c_char,
    len: i32,
    initial_size: i32,
    outlen: *mut i32,
    parse_header: i32,
) -> *mut c_char {
    let mut a: stbi__zbuf = stbi__zbuf::EMPTY;
    let p = stbi__malloc(initial_size as size_t) as *mut c_char;
    if p.is_null() {
        return null_mut::<c_char>();
    }
    a.zbuffer = buffer as *mut stbi_uc;
    a.zbuffer_end = (buffer as *mut stbi_uc).offset(len as isize);
    if stbi__do_zlib(&raw mut a, p, initial_size, 1, parse_header) != 0 {
        if !outlen.is_null() {
            *outlen = a.zout.offset_from(a.zout_start) as c_long as i32;
        }
        a.zout_start
    } else {
        free(a.zout_start as *mut c_void);
        null_mut::<c_char>()
    }
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
unsafe fn stbi__check_png_header(s: *mut stbi__context) -> i32 {
    static mut png_sig: [stbi_uc; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    let mut i: i32 = 0;
    while i < 8 {
        if stbi__get8(s) as i32 != png_sig[i as usize] as i32 {
            return 0;
        }
        i += 1;
    }
    1
}
static mut first_row_filter: [stbi_uc; 5] = [
    STBI__F_none as i32 as stbi_uc,
    STBI__F_sub as i32 as stbi_uc,
    STBI__F_none as i32 as stbi_uc,
    STBI__F_avg_first as i32 as stbi_uc,
    STBI__F_paeth_first as i32 as stbi_uc,
];
unsafe fn stbi__paeth(a: i32, b: i32, c: i32) -> i32 {
    let p: i32 = a + b - c;
    let pa: i32 = abs(p - a);
    let pb: i32 = abs(p - b);
    let pc: i32 = abs(p - c);
    if pa <= pb && pa <= pc {
        return a;
    }
    if pb <= pc {
        return b;
    }
    c
}
static mut stbi__depth_scale_table: [stbi_uc; 9] = [0, 0xff, 0x55, 0, 0x11, 0, 0, 0, 0x1];
// Reconstruct one PNG byte, keeping raw/left/above reads in filter order.
unsafe fn png_filtered_byte(
    filter: i32,
    raw: *const u8,
    cur: *const u8,
    prior: *const u8,
    k: i32,
    previous_pixel_bytes: Option<i32>,
) -> u8 {
    let source = *raw.offset(k as isize);
    let prediction = match (filter, previous_pixel_bytes) {
        (0, _) | (1 | 5 | 6, None) => return source,
        (1, Some(bytes)) => *cur.offset((k - bytes) as isize) as i32,
        (2, _) => *prior.offset(k as isize) as i32,
        (3, None) => *prior.offset(k as isize) as i32 >> 1,
        (3, Some(bytes)) => {
            (*prior.offset(k as isize) as i32 + *cur.offset((k - bytes) as isize) as i32) >> 1
        }
        (4, None) => stbi__paeth(0, *prior.offset(k as isize) as i32, 0),
        (4, Some(bytes)) => stbi__paeth(
            *cur.offset((k - bytes) as isize) as i32,
            *prior.offset(k as isize) as i32,
            *prior.offset((k - bytes) as isize) as i32,
        ),
        (5, Some(bytes)) => *cur.offset((k - bytes) as isize) as i32 >> 1,
        (6, Some(bytes)) => stbi__paeth(*cur.offset((k - bytes) as isize) as i32, 0, 0),
        _ => return source,
    };
    (source as i32 + prediction & 255) as u8
}

// Packed input may overlap output: reload the source byte after each store.
unsafe fn unpack_png_low_depth(
    mut output: *mut u8,
    mut input: *mut u8,
    mut remaining: i32,
    depth: i32,
    scale: u8,
) -> (*mut u8, *mut u8, i32) {
    let pixels_per_byte = 8 / depth;
    let mask = (1 << depth) - 1;
    while remaining >= pixels_per_byte {
        for pixel in 0..pixels_per_byte {
            crate::update_place! { previous destination = output => output.offset(1); }
            *destination =
                (scale as i32 * (*input as i32 >> (8 - depth * (pixel + 1)) & mask)) as u8;
        }
        remaining -= pixels_per_byte;
        input = input.offset(1);
    }
    for pixel in 0..remaining {
        crate::update_place! { previous destination = output => output.offset(1); }
        *destination = (scale as i32 * (*input as i32 >> (8 - depth * (pixel + 1)) & mask)) as u8;
    }
    (output, input, remaining)
}

unsafe fn stbi__create_png_image_raw(
    a: *mut stbi__png,
    mut raw: *mut stbi_uc,
    raw_len: stbi__uint32,
    out_n: i32,
    x: stbi__uint32,
    y: stbi__uint32,
    depth: i32,
    color: i32,
) -> i32 {
    let bytes: i32 = if depth == 16 { 2 } else { 1 };
    let s = (*a).s;
    let mut i: stbi__uint32 = 0;
    let stride: stbi__uint32 = x
        .wrapping_mul(out_n as stbi__uint32)
        .wrapping_mul(bytes as stbi__uint32);
    let mut k: i32 = 0;
    let img_n: i32 = (*s).img_n;
    let output_bytes: i32 = out_n * bytes;
    let mut filter_bytes: i32 = img_n * bytes;
    let mut width: i32 = x as i32;
    crate::native_assertion! {
        '_c2rust_label: out_n == (*s).img_n || out_n == (*s).img_n + 1;
        b"out_n == s->img_n || out_n == s->img_n+1\0", b"./third_party/stb/stb_image.h\0", 4607,
        b"int stbi__create_png_image_raw(stbi__png *, stbi_uc *, stbi__uint32, int, stbi__uint32, stbi__uint32, int, int)\0"
    };
    (*a).out = stbi__malloc_mad3(x as i32, y as i32, output_bytes, 0) as *mut stbi_uc;
    if (*a).out.is_null() {
        return 0;
    }
    if stbi__mad3sizes_valid(img_n, x as i32, depth, 7) == 0 {
        return 0;
    }
    let mut img_width_bytes: stbi__uint32 = (img_n as stbi__uint32)
        .wrapping_mul(x)
        .wrapping_mul(depth as stbi__uint32)
        .wrapping_add(7 as stbi__uint32)
        >> 3;
    let mut img_len: stbi__uint32 = img_width_bytes
        .wrapping_add(1 as stbi__uint32)
        .wrapping_mul(y);
    if raw_len < img_len {
        return 0;
    }
    let mut j: stbi__uint32 = 0 as stbi__uint32;
    while j < y {
        let mut cur = (*a).out.offset(stride.wrapping_mul(j) as isize);
        crate::update_place! { previous fresh19 = raw => raw.offset(1); }
        let mut filter: i32 = *fresh19 as i32;
        if filter > 4 {
            return 0;
        }
        if depth < 8 {
            if img_width_bytes > x {
                return 0;
            }
            cur = cur.offset(
                x.wrapping_mul(out_n as stbi__uint32)
                    .wrapping_sub(img_width_bytes) as isize,
            );
            filter_bytes = 1;
            width = img_width_bytes as i32;
        }
        let mut prior = cur.offset(-(stride as isize));
        if j == 0 as stbi__uint32 {
            filter = first_row_filter[filter as usize] as i32;
        }
        k = 0;
        while k < filter_bytes {
            if (0..=6).contains(&filter) {
                *cur.offset(k as isize) = png_filtered_byte(filter, raw, cur, prior, k, None);
            }
            k += 1;
        }
        if depth == 8 {
            if img_n != out_n {
                *cur.offset(img_n as isize) = 255 as stbi_uc;
            }
            raw = raw.offset(img_n as isize);
            cur = cur.offset(out_n as isize);
            prior = prior.offset(out_n as isize);
        } else if depth == 16 {
            if img_n != out_n {
                *cur.offset(filter_bytes as isize) = 255 as stbi_uc;
                *cur.offset((filter_bytes + 1) as isize) = 255 as stbi_uc;
            }
            raw = raw.offset(filter_bytes as isize);
            cur = cur.offset(output_bytes as isize);
            prior = prior.offset(output_bytes as isize);
        } else {
            raw = raw.offset(1);
            cur = cur.offset(1);
            prior = prior.offset(1);
        }
        if depth < 8 || img_n == out_n {
            let nk: i32 = (width - 1i32) * filter_bytes;
            match filter {
                0 => {
                    memcpy(cur as *mut c_void, raw as *const c_void, nk as size_t);
                }
                1..=6 => {
                    k = 0;
                    while k < nk {
                        *cur.offset(k as isize) =
                            png_filtered_byte(filter, raw, cur, prior, k, Some(filter_bytes));
                        k += 1;
                    }
                }
                _ => {}
            }
            raw = raw.offset(nk as isize);
        } else {
            crate::native_assertion! {
                '_c2rust_label_0: img_n + 1 == out_n;
                b"img_n+1 == out_n\0", b"./third_party/stb/stb_image.h\0", 4691,
                b"int stbi__create_png_image_raw(stbi__png *, stbi_uc *, stbi__uint32, int, stbi__uint32, stbi__uint32, int, int)\0"
            };
            if (0..=6).contains(&filter) {
                i = x.wrapping_sub(1 as stbi__uint32);
                while i >= 1 as stbi__uint32 {
                    k = 0;
                    while k < filter_bytes {
                        *cur.offset(k as isize) =
                            png_filtered_byte(filter, raw, cur, prior, k, Some(output_bytes));
                        k += 1;
                    }
                    i = i.wrapping_sub(1);
                    *cur.offset(filter_bytes as isize) = 255 as stbi_uc;
                    raw = raw.offset(filter_bytes as isize);
                    cur = cur.offset(output_bytes as isize);
                    prior = prior.offset(output_bytes as isize);
                }
            }
            if depth == 16 {
                cur = (*a).out.offset(stride.wrapping_mul(j) as isize);
                i = 0 as stbi__uint32;
                while i < x {
                    *cur.offset((filter_bytes + 1) as isize) = 255 as stbi_uc;
                    i = i.wrapping_add(1);
                    cur = cur.offset(output_bytes as isize);
                }
            }
        }
        j = j.wrapping_add(1);
    }
    if depth < 8 {
        j = 0 as stbi__uint32;
        while j < y {
            let mut cur_0 = (*a).out.offset(stride.wrapping_mul(j) as isize);
            let mut in_0 = (*a)
                .out
                .offset(stride.wrapping_mul(j) as isize)
                .offset(x.wrapping_mul(out_n as stbi__uint32) as isize)
                .offset(-(img_width_bytes as isize));
            let scale: stbi_uc = (if color == 0 {
                stbi__depth_scale_table[depth as usize] as i32
            } else {
                1
            }) as stbi_uc;
            if depth == 4 || depth == 2 || depth == 1 {
                (cur_0, in_0, k) = unpack_png_low_depth(
                    cur_0,
                    in_0,
                    x.wrapping_mul(img_n as stbi__uint32) as i32,
                    depth,
                    scale,
                );
            }
            if img_n != out_n {
                let mut q: i32 = 0;
                cur_0 = (*a).out.offset(stride.wrapping_mul(j) as isize);
                if img_n == 1 {
                    q = x.wrapping_sub(1 as stbi__uint32) as i32;
                    while q >= 0 {
                        *cur_0.offset((q * 2 + 1) as isize) = 255 as stbi_uc;
                        *cur_0.offset((q * 2 + 0) as isize) = *cur_0.offset(q as isize);
                        q -= 1;
                    }
                } else {
                    crate::native_assertion! {
                        '_c2rust_label_1: img_n == 3;
                        b"img_n == 3\0", b"./third_party/stb/stb_image.h\0", 4780,
                        b"int stbi__create_png_image_raw(stbi__png *, stbi_uc *, stbi__uint32, int, stbi__uint32, stbi__uint32, int, int)\0"
                    };
                    q = x.wrapping_sub(1 as stbi__uint32) as i32;
                    while q >= 0 {
                        *cur_0.offset((q * 4 + 3) as isize) = 255 as stbi_uc;
                        *cur_0.offset((q * 4 + 2) as isize) = *cur_0.offset((q * 3 + 2) as isize);
                        *cur_0.offset((q * 4 + 1) as isize) = *cur_0.offset((q * 3 + 1) as isize);
                        *cur_0.offset((q * 4 + 0) as isize) = *cur_0.offset((q * 3 + 0) as isize);
                        q -= 1;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
    } else if depth == 16 {
        let mut cur_1 = (*a).out;
        let mut cur16 = cur_1 as *mut stbi__uint16;
        i = 0 as stbi__uint32;
        while i < x.wrapping_mul(y).wrapping_mul(out_n as stbi__uint32) {
            *cur16 = ((*cur_1 as i32) << 8 | *cur_1.offset(1) as i32) as stbi__uint16;
            i = i.wrapping_add(1);
            cur16 = cur16.offset(1);
            cur_1 = cur_1.offset(2);
        }
    }
    1
}
unsafe fn stbi__create_png_image(
    a: *mut stbi__png,
    mut image_data: *mut stbi_uc,
    mut image_data_len: stbi__uint32,
    out_n: i32,
    depth: i32,
    color: i32,
    interlaced: i32,
) -> i32 {
    let bytes: i32 = if depth == 16 { 2 } else { 1 };
    let out_bytes: i32 = out_n * bytes;
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
    let mut final_0 =
        stbi__malloc_mad3((*(*a).s).img_x as i32, (*(*a).s).img_y as i32, out_bytes, 0)
            as *mut stbi_uc;
    if final_0.is_null() {
        return 0;
    }
    let mut p: i32 = 0;
    while p < 7 {
        let xorig: [i32; 7] = [0, 4, 0, 2, 0, 1, 0];
        let yorig: [i32; 7] = [0, 0, 4, 0, 2, 0, 1];
        let xspc: [i32; 7] = [8, 8, 4, 4, 2, 2, 1];
        let yspc: [i32; 7] = [8, 8, 8, 4, 4, 2, 2];
        let mut x: i32 = (*(*a).s)
            .img_x
            .wrapping_sub(xorig[p as usize] as stbi__uint32)
            .wrapping_add(xspc[p as usize] as stbi__uint32)
            .wrapping_sub(1 as stbi__uint32)
            .wrapping_div(xspc[p as usize] as stbi__uint32) as i32;
        let mut y: i32 = (*(*a).s)
            .img_y
            .wrapping_sub(yorig[p as usize] as stbi__uint32)
            .wrapping_add(yspc[p as usize] as stbi__uint32)
            .wrapping_sub(1 as stbi__uint32)
            .wrapping_div(yspc[p as usize] as stbi__uint32) as i32;
        if x != 0 && y != 0 {
            let img_len: stbi__uint32 =
                ((((*(*a).s).img_n * x * depth + 7 >> 3) + 1) * y) as stbi__uint32;
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
                free(final_0 as *mut c_void);
                return 0;
            }
            let mut j: i32 = 0;
            while j < y {
                let mut i: i32 = 0;
                while i < x {
                    let out_y: i32 = j * yspc[p as usize] + yorig[p as usize];
                    let out_x: i32 = i * xspc[p as usize] + xorig[p as usize];
                    memcpy(
                        final_0
                            .offset(
                                (out_y as stbi__uint32)
                                    .wrapping_mul((*(*a).s).img_x)
                                    .wrapping_mul(out_bytes as stbi__uint32)
                                    as isize,
                            )
                            .offset((out_x * out_bytes) as isize)
                            as *mut c_void,
                        (*a).out.offset(((j * x + i) * out_bytes) as isize) as *const c_void,
                        out_bytes as size_t,
                    );
                    i += 1;
                }
                j += 1;
            }
            free((*a).out as *mut c_void);
            image_data = image_data.offset(img_len as isize);
            image_data_len = image_data_len.wrapping_sub(img_len);
        }
        p += 1;
    }
    (*a).out = final_0;
    1
}
unsafe fn stbi__compute_transparency(z: *mut stbi__png, tc: *mut stbi_uc, out_n: i32) -> i32 {
    let s = (*z).s;
    let mut i: stbi__uint32 = 0;
    let pixel_count: stbi__uint32 = (*s).img_x.wrapping_mul((*s).img_y);
    let mut p = (*z).out;
    crate::native_assertion! {
        '_c2rust_label: out_n == 2 || out_n == 4;
        b"out_n == 2 || out_n == 4\0", b"./third_party/stb/stb_image.h\0", 4859,
        b"int stbi__compute_transparency(stbi__png *, stbi_uc *, int)\0"
    };
    if out_n == 2 {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            *p.offset(1) = (if *p as i32 == *tc as i32 { 0 } else { 255 }) as stbi_uc;
            p = p.offset(2);
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            if *p as i32 == *tc as i32
                && *p.offset(1) as i32 == *tc.offset(1) as i32
                && *p.offset(2) as i32 == *tc.offset(2) as i32
            {
                *p.offset(3) = 0 as stbi_uc;
            }
            p = p.offset(4);
            i = i.wrapping_add(1);
        }
    }
    1
}
unsafe fn stbi__compute_transparency16(
    z: *mut stbi__png,
    tc: *mut stbi__uint16,
    out_n: i32,
) -> i32 {
    let s = (*z).s;
    let mut i: stbi__uint32 = 0;
    let pixel_count: stbi__uint32 = (*s).img_x.wrapping_mul((*s).img_y);
    let mut p = (*z).out as *mut stbi__uint16;
    crate::native_assertion! {
        '_c2rust_label: out_n == 2 || out_n == 4;
        b"out_n == 2 || out_n == 4\0", b"./third_party/stb/stb_image.h\0", 4884,
        b"int stbi__compute_transparency16(stbi__png *, stbi__uint16 *, int)\0"
    };
    if out_n == 2 {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            *p.offset(1) = (if *p as i32 == *tc as i32 { 0 } else { 65535 }) as stbi__uint16;
            p = p.offset(2);
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            if *p as i32 == *tc as i32
                && *p.offset(1) as i32 == *tc.offset(1) as i32
                && *p.offset(2) as i32 == *tc.offset(2) as i32
            {
                *p.offset(3) = 0 as stbi__uint16;
            }
            p = p.offset(4);
            i = i.wrapping_add(1);
        }
    }
    1
}
unsafe fn stbi__expand_png_palette(
    a: *mut stbi__png,
    palette: *mut stbi_uc,
    len: i32,
    pal_img_n: i32,
) -> i32 {
    let mut i: stbi__uint32 = 0;
    let pixel_count: stbi__uint32 = (*(*a).s).img_x.wrapping_mul((*(*a).s).img_y);
    let orig = (*a).out;
    let mut p = stbi__malloc_mad2(pixel_count as i32, pal_img_n, 0) as *mut stbi_uc;
    if p.is_null() {
        return 0;
    }
    let mut temp_out = p;
    if pal_img_n == 3 {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            let n: i32 = *orig.offset(i as isize) as i32 * 4;
            *p = *palette.offset(n as isize);
            *p.offset(1) = *palette.offset((n + 1) as isize);
            *p.offset(2) = *palette.offset((n + 2) as isize);
            p = p.offset(3);
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            let n_0: i32 = *orig.offset(i as isize) as i32 * 4;
            *p = *palette.offset(n_0 as isize);
            *p.offset(1) = *palette.offset((n_0 + 1) as isize);
            *p.offset(2) = *palette.offset((n_0 + 2) as isize);
            *p.offset(3) = *palette.offset((n_0 + 3) as isize);
            p = p.offset(4);
            i = i.wrapping_add(1);
        }
    }
    free((*a).out as *mut c_void);
    (*a).out = temp_out;
    1
}
static mut stbi__unpremultiply_on_load_global: i32 = 0;
static mut stbi__de_iphone_flag_global: i32 = 0;
unsafe fn stbi__de_iphone(z: *mut stbi__png) {
    let s = (*z).s;
    let mut i: stbi__uint32 = 0;
    let pixel_count: stbi__uint32 = (*s).img_x.wrapping_mul((*s).img_y);
    let mut p = (*z).out;
    if (*s).img_out_n == 3 {
        i = 0 as stbi__uint32;
        while i < pixel_count {
            let t: stbi_uc = *p;
            *p = *p.offset(2);
            *p.offset(2) = t;
            p = p.offset(3);
            i = i.wrapping_add(1);
        }
    } else {
        crate::native_assertion! {
            '_c2rust_label: (*s).img_out_n == 4;
            b"s->img_out_n == 4\0", b"./third_party/stb/stb_image.h\0", 4992,
            b"void stbi__de_iphone(stbi__png *)\0"
        };
        if stbi__unpremultiply_on_load_global != 0 {
            i = 0 as stbi__uint32;
            while i < pixel_count {
                let a: stbi_uc = *p.offset(3);
                let t_0: stbi_uc = *p;
                if a != 0 {
                    let half: stbi_uc = (a as i32 / 2) as stbi_uc;
                    *p = ((*p.offset(2) as i32 * 255 + half as i32) / a as i32) as stbi_uc;
                    *p.offset(1) =
                        ((*p.offset(1) as i32 * 255 + half as i32) / a as i32) as stbi_uc;
                    *p.offset(2) = ((t_0 as i32 * 255 + half as i32) / a as i32) as stbi_uc;
                } else {
                    *p = *p.offset(2);
                    *p.offset(2) = t_0;
                }
                p = p.offset(4);
                i = i.wrapping_add(1);
            }
        } else {
            i = 0 as stbi__uint32;
            while i < pixel_count {
                let t_1: stbi_uc = *p;
                *p = *p.offset(2);
                *p.offset(2) = t_1;
                p = p.offset(4);
                i = i.wrapping_add(1);
            }
        }
    };
}
unsafe fn stbi__parse_png_file(z: *mut stbi__png, scan: i32, req_comp: i32) -> i32 {
    let mut palette: [stbi_uc; 1024] = [0; 1024];
    let mut pal_img_n: stbi_uc = 0 as stbi_uc;
    let mut has_trans: stbi_uc = 0 as stbi_uc;
    let mut tc: [stbi_uc; 3] = [0i32 as stbi_uc, 0, 0];
    let mut tc16: [stbi__uint16; 3] = [0; 3];
    let mut ioff: stbi__uint32 = 0 as stbi__uint32;
    let mut idata_limit: stbi__uint32 = 0 as stbi__uint32;
    let mut i: stbi__uint32 = 0;
    let mut pal_len: stbi__uint32 = 0 as stbi__uint32;
    let mut first: i32 = 1;
    let mut k: i32 = 0;
    let mut interlace: i32 = 0;
    let mut color: i32 = 0;
    let mut is_iphone: i32 = 0;
    let s = (*z).s;
    (*z).expanded = null_mut::<stbi_uc>();
    (*z).idata = null_mut::<stbi_uc>();
    (*z).out = null_mut::<stbi_uc>();
    if stbi__check_png_header(s) == 0 {
        return 0;
    }
    if scan == STBI__SCAN_type as i32 {
        return 1;
    }
    loop {
        let c: stbi__pngchunk = stbi__get_chunk_header(s);
        match c.type_0 {
            1130840649 => {
                is_iphone = 1;
                stbi__skip(s, c.length as i32);
            }
            1229472850 => {
                if first == 0 {
                    return 0;
                }
                first = 0;
                if c.length != 13 as stbi__uint32 {
                    return 0;
                }
                (*s).img_x = stbi__get32be(s);
                (*s).img_y = stbi__get32be(s);
                if (*s).img_y > STBI_MAX_DIMENSIONS as stbi__uint32 {
                    return 0;
                }
                if (*s).img_x > STBI_MAX_DIMENSIONS as stbi__uint32 {
                    return 0;
                }
                (*z).depth = stbi__get8(s) as i32;
                if (*z).depth != 1
                    && (*z).depth != 2
                    && (*z).depth != 4
                    && (*z).depth != 8
                    && (*z).depth != 16
                {
                    return 0;
                }
                color = stbi__get8(s) as i32;
                if color > 6 {
                    return 0;
                }
                if color == 3 && (*z).depth == 16 {
                    return 0;
                }
                if color == 3 {
                    pal_img_n = 3 as stbi_uc;
                } else if color & 1 != 0 {
                    return 0;
                }
                let mut comp: i32 = stbi__get8(s) as i32;
                if comp != 0 {
                    return 0;
                }
                let mut filter: i32 = stbi__get8(s) as i32;
                if filter != 0 {
                    return 0;
                }
                interlace = stbi__get8(s) as i32;
                if interlace > 1 {
                    return 0;
                }
                if (*s).img_x == 0 || (*s).img_y == 0 {
                    return 0;
                }
                if pal_img_n == 0 {
                    (*s).img_n =
                        (if color & 2 != 0 { 3 } else { 1 }) + (if color & 4 != 0 { 1 } else { 0 });
                    if (((1i32) << 30i32) as stbi__uint32)
                        .wrapping_div((*s).img_x)
                        .wrapping_div((*s).img_n as stbi__uint32)
                        < (*s).img_y
                    {
                        return 0;
                    }
                    if scan == STBI__SCAN_header as i32 {
                        return 1;
                    }
                } else {
                    (*s).img_n = 1;
                    if (((1i32) << 30i32) as stbi__uint32)
                        .wrapping_div((*s).img_x)
                        .wrapping_div(4 as stbi__uint32)
                        < (*s).img_y
                    {
                        return 0;
                    }
                }
            }
            1347179589 => {
                if first != 0 {
                    return 0;
                }
                if c.length > (256 * 3) as stbi__uint32 {
                    return 0;
                }
                pal_len = c.length.wrapping_div(3 as stbi__uint32);
                if pal_len.wrapping_mul(3 as stbi__uint32) != c.length {
                    return 0;
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
                    return 0;
                }
                if !(*z).idata.is_null() {
                    return 0;
                }
                if pal_img_n != 0 {
                    if scan == STBI__SCAN_header as i32 {
                        (*s).img_n = 4;
                        return 1;
                    }
                    if pal_len == 0 as stbi__uint32 {
                        return 0;
                    }
                    if c.length > pal_len {
                        return 0;
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
                    if (*s).img_n & 1 == 0 {
                        return 0;
                    }
                    if c.length != ((*s).img_n as stbi__uint32).wrapping_mul(2 as stbi__uint32) {
                        return 0;
                    }
                    has_trans = 1 as stbi_uc;
                    if (*z).depth == 16 {
                        k = 0;
                        while k < (*s).img_n {
                            tc16[k as usize] = stbi__get16be(s) as stbi__uint16;
                            k += 1;
                        }
                    } else {
                        k = 0;
                        while k < (*s).img_n {
                            tc[k as usize] = ((stbi__get16be(s) & 255) as stbi_uc as i32
                                * stbi__depth_scale_table[(*z).depth as usize] as i32)
                                as stbi_uc;
                            k += 1;
                        }
                    }
                }
            }
            1229209940 => {
                if first != 0 {
                    return 0;
                }
                if pal_img_n as i32 != 0 && pal_len == 0 {
                    return 0;
                }
                if scan == STBI__SCAN_header as i32 {
                    (*s).img_n = pal_img_n as i32;
                    return 1;
                }
                if (ioff.wrapping_add(c.length) as i32) < ioff as i32 {
                    return 0;
                }
                if ioff.wrapping_add(c.length) > idata_limit {
                    let idata_limit_old: stbi__uint32 = idata_limit;
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
                    let mut p =
                        realloc((*z).idata as *mut c_void, idata_limit as size_t) as *mut stbi_uc;
                    if p.is_null() {
                        return 0;
                    }
                    (*z).idata = p;
                }
                if stbi__getn(s, (*z).idata.offset(ioff as isize), c.length as i32) == 0 {
                    return 0;
                }
                ioff = ioff.wrapping_add(c.length);
            }
            1229278788 => {
                if first != 0 {
                    return 0;
                }
                if scan != STBI__SCAN_load as i32 {
                    return 1;
                }
                if (*z).idata.is_null() {
                    return 0;
                }
                let mut bpl: stbi__uint32 = (*s)
                    .img_x
                    .wrapping_mul((*z).depth as stbi__uint32)
                    .wrapping_add(7 as stbi__uint32)
                    .wrapping_div(8 as stbi__uint32);
                let mut raw_len: stbi__uint32 = bpl
                    .wrapping_mul((*s).img_y)
                    .wrapping_mul((*s).img_n as stbi__uint32)
                    .wrapping_add((*s).img_y);
                (*z).expanded = stbi_zlib_decode_malloc_guesssize_headerflag(
                    (*z).idata as *mut c_char,
                    ioff as i32,
                    raw_len as i32,
                    &raw mut raw_len as *mut i32,
                    (is_iphone == 0) as i32,
                ) as *mut stbi_uc;
                if (*z).expanded.is_null() {
                    return 0;
                }
                free((*z).idata as *mut c_void);
                (*z).idata = null_mut::<stbi_uc>();
                if req_comp == (*s).img_n + 1 && req_comp != 3 && pal_img_n == 0
                    || has_trans as i32 != 0
                {
                    (*s).img_out_n = (*s).img_n + 1;
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
                    return 0;
                }
                if has_trans != 0 {
                    if (*z).depth == 16 {
                        if stbi__compute_transparency16(
                            z,
                            &raw mut tc16 as *mut stbi__uint16,
                            (*s).img_out_n,
                        ) == 0
                        {
                            return 0;
                        }
                    } else if stbi__compute_transparency(
                        z,
                        &raw mut tc as *mut stbi_uc,
                        (*s).img_out_n,
                    ) == 0
                    {
                        return 0;
                    }
                }
                if is_iphone != 0 && stbi__de_iphone_flag_global != 0 && (*s).img_out_n > 2 {
                    stbi__de_iphone(z);
                }
                if pal_img_n != 0 {
                    (*s).img_n = pal_img_n as i32;
                    (*s).img_out_n = pal_img_n as i32;
                    if req_comp >= 3 {
                        (*s).img_out_n = req_comp;
                    }
                    if stbi__expand_png_palette(
                        z,
                        &raw mut palette as *mut stbi_uc,
                        pal_len as i32,
                        (*s).img_out_n,
                    ) == 0
                    {
                        return 0;
                    }
                } else if has_trans != 0 {
                    (*s).img_n += 1;
                }
                free((*z).expanded as *mut c_void);
                (*z).expanded = null_mut::<stbi_uc>();
                stbi__get32be(s);
                return 1;
            }
            _ => {
                if first != 0 {
                    return 0;
                }
                if c.type_0 & ((1i32) << 29) as stbi__uint32 == 0 as stbi__uint32 {
                    return 0;
                }
                stbi__skip(s, c.length as i32);
            }
        }
        stbi__get32be(s);
    }
}
unsafe fn stbi__do_png(
    p: *mut stbi__png,
    x: *mut i32,
    y: *mut i32,
    n: *mut i32,
    req_comp: i32,
    ri: *mut stbi__result_info,
) -> *mut c_void {
    let mut result = NULL_0;
    if req_comp < 0 || req_comp > 4 {
        return (if 0 != 0 { NULL_0 } else { NULL_0 }) as size_t as *mut c_uchar as *mut c_void;
    }
    if stbi__parse_png_file(p, STBI__SCAN_load as i32, req_comp) != 0 {
        if (*p).depth <= 8 {
            (*ri).bits_per_channel = 8;
        } else if (*p).depth == 16 {
            (*ri).bits_per_channel = 16;
        } else {
            return (if 0 != 0 { NULL_0 } else { NULL_0 }) as size_t as *mut c_uchar as *mut c_void;
        }
        result = (*p).out as *mut c_void;
        (*p).out = null_mut::<stbi_uc>();
        if req_comp != 0 && req_comp != (*(*p).s).img_out_n {
            if (*ri).bits_per_channel == 8 {
                result = stbi__convert_format(
                    result as *mut c_uchar,
                    (*(*p).s).img_out_n,
                    req_comp,
                    (*(*p).s).img_x,
                    (*(*p).s).img_y,
                ) as *mut c_void;
            } else {
                result = stbi__convert_format16(
                    result as *mut stbi__uint16,
                    (*(*p).s).img_out_n,
                    req_comp,
                    (*(*p).s).img_x,
                    (*(*p).s).img_y,
                ) as *mut c_void;
            }
            (*(*p).s).img_out_n = req_comp;
            if result.is_null() {
                return result;
            }
        }
        *x = (*(*p).s).img_x as i32;
        *y = (*(*p).s).img_y as i32;
        if !n.is_null() {
            *n = (*(*p).s).img_n;
        }
    }
    free((*p).out as *mut c_void);
    (*p).out = null_mut::<stbi_uc>();
    free((*p).expanded as *mut c_void);
    (*p).expanded = null_mut::<stbi_uc>();
    free((*p).idata as *mut c_void);
    (*p).idata = null_mut::<stbi_uc>();
    result
}
unsafe fn stbi__png_load(
    s: *mut stbi__context,
    x: *mut i32,
    y: *mut i32,
    comp: *mut i32,
    req_comp: i32,
    ri: *mut stbi__result_info,
) -> *mut c_void {
    let mut p: stbi__png = stbi__png::EMPTY;
    p.s = s;
    stbi__do_png(&raw mut p, x, y, comp, req_comp, ri)
}
unsafe fn stbi__png_test(s: *mut stbi__context) -> i32 {
    let mut r: i32 = stbi__check_png_header(s);
    stbi__rewind(s);
    r
}
pub const STBI_MAX_DIMENSIONS: i32 = 4096;
pub const NULL: *mut c_void = null_mut::<c_void>();
pub const NULL_0: *mut c_void = null_mut::<c_void>();
unsafe fn ParseConfigKeyPass(
    gs: *mut GlslShader,
    mut key: *const c_char,
    mut match_0: *const c_char,
) -> *mut GlslPass {
    let mut endp = null_mut::<c_char>();
    while *match_0 != 0 {
        if *key as i32 != *match_0 as i32 {
            return null_mut::<GlslPass>();
        }
        key = key.offset(1);
        match_0 = match_0.offset(1);
    }
    if (*key as i32 - '0' as i32) as u8 as i32 >= 10 {
        return null_mut::<GlslPass>();
    }
    let pass: u32 = strtoul(key, &raw mut endp, 10) as u32;
    if pass >= (*gs).n_pass as u32 || *endp as i32 != 0 {
        return null_mut::<GlslPass>();
    }
    (*gs).pass.offset(pass as isize).offset(1)
}
unsafe fn ParseScaleType(s: *const c_char) -> u8 {
    ((if StringEqualsNoCase(s, b"source\0" as *const u8 as *const c_char) as i32 != 0 {
        GLSL_SOURCE as i32
    } else if StringEqualsNoCase(s, b"viewport\0" as *const u8 as *const c_char) as i32 != 0 {
        GLSL_VIEWPORT as i32
    } else if StringEqualsNoCase(s, b"absolute\0" as *const u8 as *const c_char) as i32 != 0 {
        GLSL_ABSOLUTE as i32
    } else {
        GLSL_NONE as i32
    }) as u8)
}
unsafe fn ParseWrapMode(s: *const c_char) -> u32 {
    ((if StringEqualsNoCase(s, b"repeat\0" as *const u8 as *const c_char) as i32 != 0 {
        GL_REPEAT
    } else if StringEqualsNoCase(s, b"clamp_to_edge\0" as *const u8 as *const c_char) as i32 != 0 {
        GL_CLAMP_TO_EDGE
    } else if StringEqualsNoCase(s, b"clamp\0" as *const u8 as *const c_char) as i32 != 0 {
        GL_CLAMP
    } else {
        GL_CLAMP_TO_BORDER
    }) as u32)
}
unsafe fn GlslPass_Initialize(pass: *mut GlslPass) {
    (*pass).scale_x = 1.0f32;
    (*pass).scale_y = 1.0f32;
    (*pass).wrap_mode = GL_CLAMP_TO_BORDER as u32;
}
unsafe fn ParseTextures(gs: *mut GlslShader, mut value: *mut c_char) {
    let mut id = null_mut::<c_char>();
    let mut nextp = &raw mut (*gs).first_texture;
    let mut num: i32 = 0;
    loop {
        id = NextDelim(&raw mut value, ';' as i32);
        if !(!id.is_null() && num < kGlslMaxTextures as i32) {
            break;
        }
        let t = calloc(size_of::<GlslTexture>() as size_t, 1 as size_t) as *mut GlslTexture;
        (*t).id = strdup(id);
        (*t).wrap_mode = GL_CLAMP_TO_BORDER as u32;
        (*t).filter = GL_NEAREST as u32;
        *nextp = t;
        nextp = &raw mut (*t).next;
        num += 1;
    }
}
unsafe fn ParseTextureKeyValue(
    gs: *mut GlslShader,
    key: *const c_char,
    value: *const c_char,
) -> bool {
    let mut t = (*gs).first_texture;
    while !t.is_null() {
        let key2 = SkipPrefix(key, (*t).id);
        if !key2.is_null() {
            if *key2 as i32 == 0 {
                StrSet(&raw mut (*t).filename, value);
                return true_0 != 0;
            } else if strcmp(key2, b"_wrap_mode\0" as *const u8 as *const c_char) == 0 {
                (*t).wrap_mode = ParseWrapMode(value);
                return true_0 != 0;
            } else if strcmp(key2, b"_mipmap\0" as *const u8 as *const c_char) == 0 {
                (*t).mipmap = ParseBool(value, null_mut::<bool>());
                return true_0 != 0;
            } else if strcmp(key2, b"_linear\0" as *const u8 as *const c_char) == 0 {
                (*t).filter = (if ParseBool(value, null_mut::<bool>()) as i32 != 0 {
                    GL_LINEAR
                } else {
                    GL_NEAREST
                }) as u32;
                return true_0 != 0;
            }
        }
        t = (*t).next;
    }
    false_0 != 0
}
unsafe fn GlslShader_GetParam(gs: *mut GlslShader, id: *const c_char) -> *mut GlslParam {
    let mut pp = &raw mut (*gs).first_param;
    while !(*pp).is_null() {
        if strcmp((**pp).id, id) == 0 {
            return *pp;
        }
        pp = &raw mut (**pp).next;
    }
    let p = calloc(1 as size_t, size_of::<GlslParam>() as size_t) as *mut GlslParam;
    *pp = p;
    (*p).id = strdup(id);
    p
}
unsafe fn ParseParameters(gs: *mut GlslShader, mut value: *mut c_char) {
    let mut id = null_mut::<c_char>();
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
    key: *const c_char,
    value: *const c_char,
) -> bool {
    let mut p = (*gs).first_param;
    while !p.is_null() {
        if strcmp((*p).id, key) == 0 {
            (*p).value = atof(value) as c_float;
            (*p).has_value = true_0 != 0;
            return true_0 != 0;
        }
        p = (*p).next;
    }
    false_0 != 0
}
unsafe fn GlslShader_InitializePasses(gs: *mut GlslShader, passes: i32) {
    (*gs).n_pass = passes;
    (*gs).pass = calloc(
        ((*gs).n_pass + 1) as size_t,
        size_of::<GlslPass>() as size_t,
    ) as *mut GlslPass;
    let mut i: i32 = 0;
    while i < (*gs).n_pass {
        GlslPass_Initialize((*gs).pass.offset(i as isize).offset(1));
        i += 1;
    }
}
unsafe fn GlslShader_ReadPresetFile(gs: *mut GlslShader, filename: *const c_char) -> bool {
    let mut data = ReadWholeFile(filename, null_mut::<size_t>()) as *mut c_char;
    let data_org = data;
    let mut line = null_mut::<c_char>();
    let mut pass = null_mut::<GlslPass>();
    if data.is_null() {
        return false_0 != 0;
    }
    let mut lineno: i32 = 1;
    loop {
        line = NextLineStripComments(&raw mut data);
        if line.is_null() {
            break;
        }
        let mut value = SplitKeyValue(line);
        if value.is_null() {
            if *line != 0 {
                fprintf(
                    stderr,
                    b"%s:%d: Expecting key=value\n\0" as *const u8 as *const c_char,
                    filename,
                    lineno,
                );
            }
        } else {
            if *value as i32 == '"' as i32 {
                value = value.offset(1);
                let mut t = value;
                while *t as i32 != 0 && *t as i32 != '"' as i32 {
                    t = t.offset(1);
                }
                if *t != 0 {
                    *t = 0 as c_char;
                }
            }
            if (*gs).n_pass == 0 {
                if strcmp(line, b"shaders\0" as *const u8 as *const c_char) != 0 {
                    fprintf(
                        stderr,
                        b"%s:%d: Expecting 'shaders'\n\0" as *const u8 as *const c_char,
                        filename,
                        lineno,
                    );
                    break;
                } else {
                    let passes: i32 = strtoul(value, null_mut::<*mut c_char>(), 10) as i32;
                    if passes < 1 || passes > kGlslMaxPasses as i32 {
                        break;
                    }
                    GlslShader_InitializePasses(gs, passes);
                }
            } else {
                pass =
                    ParseConfigKeyPass(gs, line, b"filter_linear\0" as *const u8 as *const c_char);
                if !pass.is_null() {
                    (*pass).filter = (if ParseBool(value, null_mut::<bool>()) as i32 != 0 {
                        GL_LINEAR
                    } else {
                        GL_NEAREST
                    }) as u32;
                } else {
                    pass =
                        ParseConfigKeyPass(gs, line, b"scale_type\0" as *const u8 as *const c_char);
                    if !pass.is_null() {
                        (*pass).scale_type_y = ParseScaleType(value);
                        (*pass).scale_type_x = (*pass).scale_type_y;
                    } else {
                        pass = ParseConfigKeyPass(
                            gs,
                            line,
                            b"scale_type_x\0" as *const u8 as *const c_char,
                        );
                        if !pass.is_null() {
                            (*pass).scale_type_x = ParseScaleType(value);
                        } else {
                            pass = ParseConfigKeyPass(
                                gs,
                                line,
                                b"scale_type_y\0" as *const u8 as *const c_char,
                            );
                            if !pass.is_null() {
                                (*pass).scale_type_y = ParseScaleType(value);
                            } else {
                                pass = ParseConfigKeyPass(
                                    gs,
                                    line,
                                    b"scale\0" as *const u8 as *const c_char,
                                );
                                if !pass.is_null() {
                                    (*pass).scale_y = atof(value) as c_float;
                                    (*pass).scale_x = (*pass).scale_y;
                                } else {
                                    pass = ParseConfigKeyPass(
                                        gs,
                                        line,
                                        b"scale_x\0" as *const u8 as *const c_char,
                                    );
                                    if !pass.is_null() {
                                        (*pass).scale_x = atof(value) as c_float;
                                    } else {
                                        pass = ParseConfigKeyPass(
                                            gs,
                                            line,
                                            b"scale_y\0" as *const u8 as *const c_char,
                                        );
                                        if !pass.is_null() {
                                            (*pass).scale_y = atof(value) as c_float;
                                        } else {
                                            pass = ParseConfigKeyPass(
                                                gs,
                                                line,
                                                b"shader\0" as *const u8 as *const c_char,
                                            );
                                            if !pass.is_null() {
                                                StrSet(&raw mut (*pass).filename, value);
                                            } else {
                                                pass = ParseConfigKeyPass(
                                                    gs,
                                                    line,
                                                    b"wrap_mode\0" as *const u8 as *const c_char,
                                                );
                                                if !pass.is_null() {
                                                    (*pass).wrap_mode = ParseWrapMode(value);
                                                } else {
                                                    pass = ParseConfigKeyPass(
                                                        gs,
                                                        line,
                                                        b"mipmap_input\0" as *const u8
                                                            as *const c_char,
                                                    );
                                                    if !pass.is_null() {
                                                        (*pass).mipmap_input =
                                                            ParseBool(value, null_mut::<bool>());
                                                    } else {
                                                        pass = ParseConfigKeyPass(
                                                            gs,
                                                            line,
                                                            b"frame_count_mod\0" as *const u8
                                                                as *const c_char,
                                                        );
                                                        if !pass.is_null() {
                                                            (*pass).frame_count_mod =
                                                                atoi(value) as u32;
                                                        } else {
                                                            pass = ParseConfigKeyPass(
                                                                gs,
                                                                line,
                                                                b"float_framebuffer\0" as *const u8
                                                                    as *const c_char,
                                                            );
                                                            if !pass.is_null() {
                                                                (*pass).float_framebuffer =
                                                                    ParseBool(
                                                                        value,
                                                                        null_mut::<bool>(),
                                                                    );
                                                            } else {
                                                                pass = ParseConfigKeyPass(
                                                                    gs,
                                                                    line,
                                                                    b"srgb_framebuffer\0"
                                                                        as *const u8
                                                                        as *const c_char,
                                                                );
                                                                if !pass.is_null() {
                                                                    (*pass).srgb_framebuffer =
                                                                        ParseBool(
                                                                            value,
                                                                            null_mut::<bool>(),
                                                                        );
                                                                } else {
                                                                    pass = ParseConfigKeyPass(
                                                                        gs,
                                                                        line,
                                                                        b"alias\0" as *const u8
                                                                            as *const c_char,
                                                                    );
                                                                    if pass.is_null() {
                                                                        if strcmp(
                                                                            line,
                                                                            b"textures\0" as *const u8 as *const c_char,
                                                                        ) == 0
                                                                            && (*gs).first_texture.is_null()
                                                                        {
                                                                            ParseTextures(gs, value);
                                                                        } else if strcmp(
                                                                            line,
                                                                            b"parameters\0" as *const u8 as *const c_char,
                                                                        ) == 0
                                                                        {
                                                                            ParseParameters(gs, value);
                                                                        } else if !ParseTextureKeyValue(gs, line, value)
                                                                            && !ParseParameterKeyValue(gs, line, value)
                                                                        {
                                                                            fprintf(
                                                                                stderr,
                                                                                b"%s:%d: Unknown key '%s'\n\0" as *const u8
                                                                                    as *const c_char,
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
    free(data_org as *mut c_void);
    (*gs).n_pass != 0
}
pub unsafe fn GlslShader_ReadShaderFile(
    gs: *mut GlslShader,
    filename: *const c_char,
    result: *mut ByteArray,
) {
    let mut data = ReadWholeFile(filename, null_mut::<size_t>()) as *mut c_char;
    let data_org = data;
    let mut line = null_mut::<c_char>();
    if data.is_null() {
        fprintf(
            stderr,
            b"Unable to read file '%s'\n\0" as *const u8 as *const c_char,
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
                line as *const c_void,
                b"#include\0" as *const u8 as *const c_char as *const c_void,
                8 as size_t,
            ) == 0
        {
            let mut tt = line.offset(8);
            let new_filename =
                ReplaceFilenameWithNewPath(filename, NextPossiblyQuotedString(&raw mut tt));
            GlslShader_ReadShaderFile(gs, new_filename, result);
            free(new_filename as *mut c_void);
        } else if linelen >= 17 as size_t
            && memcmp(
                line as *const c_void,
                b"#pragma parameter\0" as *const u8 as *const c_char as *const c_void,
                17 as size_t,
            ) == 0
        {
            let mut tt_0 = line.offset(17);
            let param = GlslShader_GetParam(gs, NextPossiblyQuotedString(&raw mut tt_0));
            NextPossiblyQuotedString(&raw mut tt_0);
            let value: c_float = atof(NextPossiblyQuotedString(&raw mut tt_0)) as c_float;
            if !(*param).has_value {
                (*param).value = value;
            }
            (*param).min = atof(NextPossiblyQuotedString(&raw mut tt_0)) as c_float;
            (*param).max = atof(NextPossiblyQuotedString(&raw mut tt_0)) as c_float;
        } else {
            *line.offset(linelen as isize) = '\n' as i32 as c_char;
            ByteArray_AppendData(result, line as *mut u8, linelen.wrapping_add(1 as size_t));
        }
    }
    free(data_org as *mut c_void);
}
unsafe fn GlslPass_Compile(p: *mut GlslPass, type_0: u32, data: *const u8, size: size_t) -> bool {
    static mut kVertexPrefix: [c_char; 42] = unsafe {
        transmute::<[u8; 42], [c_char; 42]>(*b"#define VERTEX\n#define PARAMETER_UNIFORM\n\0")
    };
    static mut kFragmentPrefix: [c_char; 44] = unsafe {
        transmute::<[u8; 44], [c_char; 44]>(*b"#define FRAGMENT\n#define PARAMETER_UNIFORM\n\0")
    };
    let mut strings: [*const GLchar; 3] = [null::<GLchar>(); 3];
    let mut lengths: [GLint; 3] = [0; 3];
    let mut buffer: [c_char; 256] = [0; 256];
    let mut compile_status: GLint = 0 as GLint;
    let mut skip: size_t = 0 as size_t;
    if size < 8 as size_t
        || memcmp(
            data as *const c_void,
            b"#version\0" as *const u8 as *const c_char as *const c_void,
            8 as size_t,
        ) != 0
    {
        strings[0i32 as usize] = b"#version 330\n\0" as *const u8 as *const c_char as *const GLchar;
        lengths[0i32 as usize] = (size_of::<[c_char; 14]>() as usize).wrapping_sub(1_) as GLint;
    } else {
        while skip < size && {
            crate::update_place! { previous fresh53 = skip => skip.wrapping_add(1); }
            *data.offset(fresh53 as isize) as i32 != '\n' as i32
        } {}
        strings[0i32 as usize] = data as *mut c_char;
        lengths[0i32 as usize] = skip as i32 as GLint;
    }
    strings[1i32 as usize] = (if type_0 == GL_VERTEX_SHADER as u32 {
        &raw const kVertexPrefix as *const c_char as *mut c_char as *const c_char
    } else {
        &raw const kFragmentPrefix as *const c_char
    }) as *const GLchar;
    lengths[1i32 as usize] = (if type_0 == GL_VERTEX_SHADER as u32 {
        (size_of::<[c_char; 42]>() as usize).wrapping_sub(1_)
    } else {
        (size_of::<[c_char; 44]>() as usize).wrapping_sub(1_)
    }) as GLint;
    strings[2i32 as usize] = (data as *mut GLchar).offset(skip as isize);
    lengths[2i32 as usize] = size.wrapping_sub(skip) as i32 as GLint;
    let shader: u32 = gl::CreateShader(type_0 as GLenum) as u32;
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
    buffer[0i32 as usize] = 0 as c_char;
    gl::GetShaderInfoLog(
        shader as GLuint,
        size_of::<[c_char; 256]>() as GLsizei,
        null_mut::<GLsizei>(),
        &raw mut buffer as *mut GLchar,
    );
    if compile_status != GL_TRUE || buffer[0i32 as usize] as i32 != 0 {
        fprintf(
            stderr,
            b"%s compiling %s shader in file '%s':\n%s\n\0" as *const u8 as *const c_char,
            if compile_status != GL_TRUE {
                b"Error\0" as *const u8 as *const c_char
            } else {
                b"While\0" as *const u8 as *const c_char
            },
            if type_0 == GL_VERTEX_SHADER as u32 {
                b"vertex\0" as *const u8 as *const c_char
            } else {
                b"fragment\0" as *const u8 as *const c_char
            },
            (*p).filename,
            &raw mut buffer as *mut c_char,
        );
    }
    if compile_status == GL_TRUE {
        gl::AttachShader((*p).gl_program as GLuint, shader as GLuint);
    }
    gl::DeleteShader(shader as GLuint);
    compile_status == GL_TRUE
}
unsafe fn GlslTextureUniform_Read(
    program: u32,
    prefix: *const c_char,
    i: i32,
    result: *mut GlslTextureUniform,
) {
    let mut buf: [c_char; 40] = [0; 40];
    let e = (&raw mut buf as *mut c_char).offset((snprintf
        as unsafe extern "C" fn(*mut c_char, size_t, *const c_char, ...) -> i32)(
        &raw mut buf as *mut c_char,
        size_of::<[c_char; 40]>() as size_t,
        if i >= 0 {
            b"%s%u\0" as *const u8 as *const c_char
        } else {
            b"%s\0" as *const u8 as *const c_char
        },
        prefix,
        i,
    ) as isize) as *mut c_char;
    memcpy(
        e as *mut c_void,
        b"Texture\0" as *const u8 as *const c_char as *const c_void,
        8 as size_t,
    );
    (*result).Texture =
        gl::GetUniformLocation(program as GLuint, &raw mut buf as *mut c_char) as i32;
    memcpy(
        e as *mut c_void,
        b"InputSize\0" as *const u8 as *const c_char as *const c_void,
        10 as size_t,
    );
    (*result).InputSize =
        gl::GetUniformLocation(program as GLuint, &raw mut buf as *mut c_char) as i32;
    memcpy(
        e as *mut c_void,
        b"TextureSize\0" as *const u8 as *const c_char as *const c_void,
        12 as size_t,
    );
    (*result).TextureSize =
        gl::GetUniformLocation(program as GLuint, &raw mut buf as *mut c_char) as i32;
    memcpy(
        e as *mut c_void,
        b"TexCoord\0" as *const u8 as *const c_char as *const c_void,
        9 as size_t,
    );
    (*result).TexCoord =
        gl::GetAttribLocation(program as GLuint, &raw mut buf as *mut c_char) as i32;
}
static mut kMvpMatrixOrtho: [c_float; 16] = [
    2.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 2.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, -1.0f32,
    0.0f32, -1.0f32, -1.0f32, 0.0f32, 1.0f32,
];
unsafe fn GlslShader_GetUniforms(gs: *mut GlslShader) {
    let mut pass_idx: i32 = 1;
    (*gs).max_prev_frame = 0;
    let mut p = (*gs).pass.offset(1);
    let p_end = p.offset((*gs).n_pass as isize);
    while p < p_end {
        let program: u32 = (*p).gl_program;
        gl::UseProgram(program as GLuint);
        let MVPMatrix: GLint = gl::GetUniformLocation(
            program as GLuint,
            b"MVPMatrix\0" as *const u8 as *const GLchar,
        );
        if MVPMatrix >= 0 {
            gl::UniformMatrix4fv(
                MVPMatrix,
                1 as GLsizei,
                GL_FALSE as GLboolean,
                &raw const kMvpMatrixOrtho as *const GLfloat,
            );
        }
        GlslTextureUniform_Read(
            program,
            b"\0" as *const u8 as *const c_char,
            -1i32,
            &raw mut (*p).unif.Top,
        );
        (*p).unif.OutputSize = gl::GetUniformLocation(
            program as GLuint,
            b"OutputSize\0" as *const u8 as *const GLchar,
        ) as i32;
        (*p).unif.FrameCount = gl::GetUniformLocation(
            program as GLuint,
            b"FrameCount\0" as *const u8 as *const GLchar,
        ) as i32;
        (*p).unif.FrameDirection = gl::GetUniformLocation(
            program as GLuint,
            b"FrameDirection\0" as *const u8 as *const GLchar,
        ) as i32;
        (*p).unif.LUTTexCoord = gl::GetAttribLocation(
            program as GLuint,
            b"LUTTexCoord\0" as *const u8 as *const GLchar,
        ) as i32;
        (*p).unif.VertexCoord = gl::GetAttribLocation(
            program as GLuint,
            b"VertexCoord\0" as *const u8 as *const GLchar,
        ) as i32;
        GlslTextureUniform_Read(
            program,
            b"Orig\0" as *const u8 as *const c_char,
            -1i32,
            &raw mut (*p).unif.Orig,
        );
        let mut j: i32 = 0;
        while j < 7 {
            GlslTextureUniform_Read(
                program,
                b"Prev\0" as *const u8 as *const c_char,
                if j != 0 { j } else { -1i32 },
                (&raw mut (*p).unif.Prev as *mut GlslTextureUniform).offset(j as isize),
            );
            if (*p).unif.Prev[j as usize].Texture >= 0 {
                (*gs).max_prev_frame = j + 1;
            }
            j += 1;
        }
        let mut j_0: i32 = 0;
        while j_0 < (*gs).n_pass {
            GlslTextureUniform_Read(
                program,
                b"Pass\0" as *const u8 as *const c_char,
                j_0,
                (&raw mut (*p).unif.Pass as *mut GlslTextureUniform).offset(j_0 as isize),
            );
            GlslTextureUniform_Read(
                program,
                b"PassPrev\0" as *const u8 as *const c_char,
                j_0,
                (&raw mut (*p).unif.PassPrev as *mut GlslTextureUniform).offset(j_0 as isize),
            );
            j_0 += 1;
        }
        let mut t = (*gs).first_texture;
        let mut j_1: i32 = 0;
        while !t.is_null() {
            (*p).unif.Texture[j_1 as usize] =
                gl::GetUniformLocation(program as GLuint, (*t).id) as i32;
            t = (*t).next;
            j_1 += 1;
        }
        let mut pa = (*gs).first_param;
        while !pa.is_null() {
            (*pa).uniform[pass_idx as usize] =
                gl::GetUniformLocation(program as GLuint, (*pa).id) as u32;
            pa = (*pa).next;
        }
        p = p.offset(1);
        pass_idx += 1;
    }
    gl::UseProgram(0 as GLuint);
}
unsafe fn IsGlslFilename(filename: *const c_char) -> bool {
    let len: size_t = strlen(filename);
    len >= 5 as size_t
        && memcmp(
            filename.offset(len as isize).offset(-(5i32 as isize)) as *const c_void,
            b".glsl\0" as *const u8 as *const c_char as *const c_void,
            5 as size_t,
        ) == 0
}
pub unsafe fn GlslShader_CreateFromFile(mut filename: *const c_char) -> *mut GlslShader {
    static mut kTexCoords: [GLfloat; 16] = [
        0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32, 1.0f32, 0.0f32, 1.0f32, 1.0f32,
        1.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32,
    ];
    let mut current_block: u64;
    let mut buffer: [c_char; 256] = [0; 256];
    let mut link_status: GLint = 0;
    let mut shader_code: ByteArray = ByteArray {
        data: null_mut::<u8>(),
        size: 0,
        capacity: 0,
    };
    let mut success: bool = false_0 != 0;
    let mut gs = calloc(size_of::<GlslShader>() as size_t, 1 as size_t) as *mut GlslShader;
    if gs.is_null() {
        return gs;
    }
    if IsGlslFilename(filename) {
        GlslShader_InitializePasses(gs, 1);
        crate::update_place! { reference fresh0 = (*(*gs).pass.offset(1)).filename => strdup(filename); }
        filename = b"\0" as *const u8 as *const c_char;
        current_block = 1394248824506584008;
    } else if !GlslShader_ReadPresetFile(gs, filename) {
        fprintf(
            stderr,
            b"Unable to read file '%s'\n\0" as *const u8 as *const c_char,
            filename,
        );
        current_block = 13851546192323845927;
    } else {
        current_block = 1394248824506584008;
    }
    if current_block == 1394248824506584008 {
        let mut i: i32 = 1;
        loop {
            if (i > (*gs).n_pass) {
                current_block = 7056779235015430508;
                break;
            }
            let p = (*gs).pass.offset(i as isize);
            shader_code.size = 0 as size_t;
            if (*p).filename.is_null() {
                fprintf(
                    stderr,
                    b"shader%d attribute missing\n\0" as *const u8 as *const c_char,
                    i - 1i32,
                );
                current_block = 13851546192323845927;
                break;
            } else {
                let new_filename = ReplaceFilenameWithNewPath(filename, (*p).filename);
                GlslShader_ReadShaderFile(gs, new_filename, &raw mut shader_code);
                free(new_filename as *mut c_void);
                if shader_code.size == 0 as size_t {
                    fprintf(
                        stderr,
                        b"Couldn't read shader in file '%s'\n\0" as *const u8 as *const c_char,
                        (*p).filename,
                    );
                    current_block = 13851546192323845927;
                    break;
                } else {
                    (*p).gl_program = gl::CreateProgram() as u32;
                    if !GlslPass_Compile(
                        p,
                        GL_VERTEX_SHADER as u32,
                        shader_code.data,
                        shader_code.size,
                    ) || !GlslPass_Compile(
                        p,
                        GL_FRAGMENT_SHADER as u32,
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
                    buffer[0i32 as usize] = 0 as c_char;
                    gl::GetProgramInfoLog(
                        (*p).gl_program as GLuint,
                        size_of::<[c_char; 256]>() as GLsizei,
                        null_mut::<GLsizei>(),
                        &raw mut buffer as *mut GLchar,
                    );
                    if link_status != GL_TRUE || buffer[0i32 as usize] as i32 != 0 {
                        fprintf(
                            stderr,
                            b"%s linking shader in file '%s':\n%s\n\0" as *const u8
                                as *const c_char,
                            if link_status != GL_TRUE {
                                b"Error\0" as *const u8 as *const c_char
                            } else {
                                b"While\0" as *const u8 as *const c_char
                            },
                            (*p).filename,
                            &raw mut buffer as *mut c_char,
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
        if current_block == 13851546192323845927 {
        } else {
            let mut t = (*gs).first_texture;
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
                    (if (*t).mipmap as i32 != 0 {
                        (if (*t).filter == GL_LINEAR as u32 {
                            GL_LINEAR_MIPMAP_LINEAR
                        } else {
                            GL_NEAREST_MIPMAP_NEAREST
                        }) as u32
                    } else {
                        (*t).filter
                    }) as GLint,
                );
                if !(*t).filename.is_null() {
                    let new_filename_0 = ReplaceFilenameWithNewPath(filename, (*t).filename);
                    let mut imw: i32 = 0;
                    let mut imh: i32 = 0;
                    let mut imn: i32 = 0;
                    let data =
                        stbi_load(new_filename_0, &raw mut imw, &raw mut imh, &raw mut imn, 0)
                            as *mut c_uchar;
                    if data.is_null() {
                        fprintf(
                            stderr,
                            b"Unable to read PNG '%s'\n\0" as *const u8 as *const c_char,
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
                            (if imn == 4 {
                                GL_RGBA
                            } else if imn == 3 {
                                GL_RGB
                            } else {
                                GL_LUMINANCE
                            }) as GLenum,
                            GL_UNSIGNED_BYTE as GLenum,
                            data as *const c_void,
                        );
                    }
                    free(data as *mut c_void);
                    free(new_filename_0 as *mut c_void);
                }
                if (*t).mipmap {
                    gl::GenerateMipmap(GL_TEXTURE_2D as GLenum);
                }
                t = (*t).next;
            }
            let mut p_0 = (*gs).first_param;
            while !p_0.is_null() {
                (*p_0).value = if (*p_0).value < (*p_0).min {
                    (*p_0).min
                } else if (*p_0).value > (*p_0).max {
                    (*p_0).max
                } else {
                    (*p_0).value
                };
                p_0 = (*p_0).next;
            }
            GlslShader_GetUniforms(gs);
            let mut i_0: i32 = 0;
            while i_0 < (*gs).max_prev_frame {
                gl::GenTextures(
                    1 as GLsizei,
                    &raw mut (*(&raw mut (*gs).prev_frame as *mut GlTextureWithSize)
                        .offset((-i_0 - 1i32 & 7i32) as isize))
                    .gl_texture,
                );
                i_0 += 1;
            }
            gl::GenBuffers(1 as GLsizei, &raw mut (*gs).gl_vbo);
            gl::BindBuffer(GL_ARRAY_BUFFER as GLenum, (*gs).gl_vbo as GLuint);
            gl::BufferData(
                GL_ARRAY_BUFFER as GLenum,
                size_of::<[GLfloat; 16]>() as GLsizeiptr,
                &raw const kTexCoords as *const GLfloat as *const c_void,
                GL_STATIC_DRAW as GLenum,
            );
            gl::BindBuffer(GL_ARRAY_BUFFER as GLenum, 0 as GLuint);
            success = true_0 != 0;
        }
    }
    if !success {
        GlslShader_Destroy(gs);
        gs = null_mut::<GlslShader>();
    }
    ByteArray_Destroy(&raw mut shader_code);
    gs
}
pub unsafe fn GlslShader_Destroy(gs: *mut GlslShader) {
    let mut p = (*gs).pass.offset(1);
    let p_end = p.offset((*gs).n_pass as isize);
    while p < p_end {
        gl::DeleteProgram((*p).gl_program as GLuint);
        gl::DeleteTextures(1 as GLsizei, &raw mut (*p).gl_texture);
        gl::DeleteFramebuffers(1 as GLsizei, &raw mut (*p).gl_fbo);
        free((*p).filename as *mut c_void);
        p = p.offset(1);
    }
    free((*gs).pass as *mut c_void);
    let mut t = null_mut::<GlslTexture>();
    loop {
        t = (*gs).first_texture;
        if t.is_null() {
            break;
        }
        (*gs).first_texture = (*t).next;
        gl::DeleteTextures(1 as GLsizei, &raw mut (*t).gl_texture);
        free((*t).id as *mut c_void);
        free((*t).filename as *mut c_void);
        free(t as *mut c_void);
    }
    loop {
        let mut pp = (*gs).first_param;
        if pp.is_null() {
            break;
        }
        (*gs).first_param = (*pp).next;
        free((*pp).id as *mut c_void);
        free(pp as *mut c_void);
    }
    let mut i: i32 = 0;
    while i < 8 {
        gl::DeleteTextures(
            1 as GLsizei,
            &raw mut (*(&raw mut (*gs).prev_frame as *mut GlTextureWithSize).offset(i as isize))
                .gl_texture,
        );
        i += 1;
    }
    gl::DeleteBuffers(1 as GLsizei, &raw mut (*gs).gl_vbo);
    free(gs as *mut c_void);
}
unsafe fn RenderCtx_SetTexture(ctx: *mut RenderCtx, textureu: i32, texture_id: u32) {
    if textureu >= 0 {
        gl::ActiveTexture((GL_TEXTURE0 as GLenum).wrapping_add((*ctx).texture_unit as GLenum));
        gl::BindTexture(GL_TEXTURE_2D as GLenum, texture_id as GLuint);
        crate::update_place! { previous fresh55 = (*ctx).texture_unit => (*ctx).texture_unit.wrapping_add(1); }
        gl::Uniform1i(textureu as GLint, fresh55 as GLint);
    }
}
unsafe fn RenderCtx_SetTexCoords(ctx: *mut RenderCtx, tex_coord: i32, offset: usize) {
    if tex_coord >= 0 {
        crate::native_assertion! {
            '_c2rust_label: (*ctx).num_vaos < kMaxVaosInRenderCtx as i32 as u32;
            b"ctx->num_vaos < kMaxVaosInRenderCtx\0", b"src/glsl_shader.c\0", 467,
            b"void RenderCtx_SetTexCoords(RenderCtx *, int, uintptr_t)\0"
        };
        crate::update_place! { previous fresh54 = (*ctx).num_vaos => (*ctx).num_vaos.wrapping_add(1); }
        (*ctx).vaos[fresh54 as usize] = tex_coord as u32;
        gl::VertexAttribPointer(
            tex_coord as GLuint,
            2 as GLint,
            GL_FLOAT as GLenum,
            GL_FALSE as GLboolean,
            0 as GLsizei,
            offset as *mut c_void,
        );
        gl::EnableVertexAttribArray(tex_coord as GLuint);
    }
}
unsafe fn RenderCtx_SetGlslTextureUniform(
    ctx: *mut RenderCtx,
    u: *mut GlslTextureUniform,
    width: i32,
    height: i32,
    texture: u32,
) {
    let mut size: [c_float; 2] = [width as c_float, height as c_float];
    RenderCtx_SetTexture(ctx, (*u).Texture, texture);
    if (*u).InputSize >= 0 {
        gl::Uniform2fv(
            (*u).InputSize as GLint,
            1 as GLsizei,
            &raw mut size as *mut c_float,
        );
    }
    if (*u).TextureSize >= 0 {
        gl::Uniform2fv(
            (*u).TextureSize as GLint,
            1 as GLsizei,
            &raw mut size as *mut c_float,
        );
    }
    RenderCtx_SetTexCoords(ctx, (*u).TexCoord, (*ctx).offset as usize);
}
unsafe fn GlslShader_SetShaderVars(gs: *mut GlslShader, ctx: *mut RenderCtx, pass: i32) {
    let p = (*gs).pass.offset(pass as isize);
    RenderCtx_SetGlslTextureUniform(
        ctx,
        &raw mut (*p).unif.Top,
        (*p.offset(-1i32 as isize)).width as i32,
        (*p.offset(-1i32 as isize)).height as i32,
        (*p.offset(-1i32 as isize)).gl_texture,
    );
    if (*p).unif.OutputSize >= 0 {
        let mut output_size: [c_float; 2] = [(*p).width as c_float, (*p).height as c_float];
        gl::Uniform2fv(
            (*p).unif.OutputSize as GLint,
            1 as GLsizei,
            &raw mut output_size as *mut c_float,
        );
    }
    if (*p).unif.FrameCount >= 0 {
        gl::Uniform1i(
            (*p).unif.FrameCount as GLint,
            (if (*p).frame_count_mod != 0 {
                (*gs).frame_count.wrapping_rem((*p).frame_count_mod)
            } else {
                (*gs).frame_count
            }) as GLint,
        );
    }
    if (*p).unif.FrameDirection >= 0 {
        gl::Uniform1i((*p).unif.FrameDirection as GLint, 1 as GLint);
    }
    RenderCtx_SetTexCoords(ctx, (*p).unif.LUTTexCoord, (*ctx).offset as usize);
    RenderCtx_SetTexCoords(ctx, (*p).unif.VertexCoord, 0 as usize);
    RenderCtx_SetGlslTextureUniform(
        ctx,
        &raw mut (*p).unif.Orig,
        (*(*gs).pass).width as i32,
        (*(*gs).pass).height as i32,
        (*(*gs).pass).gl_texture,
    );
    let mut i: i32 = 0;
    while i < (*gs).max_prev_frame {
        let t = (&raw mut (*gs).prev_frame as *mut GlTextureWithSize)
            .offset(((*gs).frame_count.wrapping_sub(1u32).wrapping_sub(i as u32) & 7) as isize);
        crate::native_assertion! {
            '_c2rust_label: (*t).gl_texture != 0;
            b"t->gl_texture != 0\0", b"src/glsl_shader.c\0", 504,
            b"void GlslShader_SetShaderVars(GlslShader *, RenderCtx *, int)\0"
        };
        if (*t).width != 0 {
            RenderCtx_SetGlslTextureUniform(
                ctx,
                (&raw mut (*p).unif.Prev as *mut GlslTextureUniform).offset(i as isize),
                (*t).width as i32,
                (*t).height as i32,
                (*t).gl_texture,
            );
        }
        i += 1;
    }
    let mut tctr: i32 = 0;
    let mut t_0 = (*gs).first_texture;
    while !t_0.is_null() {
        RenderCtx_SetTexture(ctx, (*p).unif.Texture[tctr as usize], (*t_0).gl_texture);
        t_0 = (*t_0).next;
        tctr += 1;
    }
    let mut i_0: i32 = 1;
    while i_0 < pass {
        RenderCtx_SetGlslTextureUniform(
            ctx,
            (&raw mut (*p).unif.Pass as *mut GlslTextureUniform).offset(i_0 as isize),
            (*(*gs).pass.offset(i_0 as isize)).width as i32,
            (*(*gs).pass.offset(i_0 as isize)).height as i32,
            (*(*gs).pass.offset(i_0 as isize)).gl_texture,
        );
        i_0 += 1;
    }
    let mut i_1: i32 = 1;
    while i_1 < pass {
        RenderCtx_SetGlslTextureUniform(
            ctx,
            (&raw mut (*p).unif.PassPrev as *mut GlslTextureUniform).offset((pass - i_1) as isize),
            (*(*gs).pass.offset(i_1 as isize)).width as i32,
            (*(*gs).pass.offset(i_1 as isize)).height as i32,
            (*(*gs).pass.offset(i_1 as isize)).gl_texture,
        );
        i_1 += 1;
    }
    let mut pa = (*gs).first_param;
    while !pa.is_null() {
        if (*pa).uniform[pass as usize] >= 0 {
            gl::Uniform1f(
                (*pa).uniform[pass as usize] as GLint,
                (*pa).value as GLfloat,
            );
        }
        pa = (*pa).next;
    }
    gl::ActiveTexture(GL_TEXTURE0 as GLenum);
}
pub unsafe fn GlslShader_Render(
    gs: *mut GlslShader,
    tex: *mut GlTextureWithSize,
    viewport_x: i32,
    viewport_y: i32,
    viewport_width: i32,
    viewport_height: i32,
) {
    (*(*gs).pass).gl_texture = (*tex).gl_texture;
    (*(*gs).pass).width = (*tex).width;
    (*(*gs).pass).height = (*tex).height;
    let mut previous_framebuffer: GLint = 0;
    gl::GetIntegerv(
        GL_FRAMEBUFFER_BINDING as GLenum,
        &raw mut previous_framebuffer,
    );
    gl::BindBuffer(GL_ARRAY_BUFFER as GLenum, (*gs).gl_vbo as GLuint);
    let mut pass: i32 = 1;
    while pass <= (*gs).n_pass {
        let last_pass: bool = pass == (*gs).n_pass;
        let p = (*gs).pass.offset(pass as isize);
        match (*p).scale_type_x as i32 {
            3 => (*p).width = (*p).scale_x as u16,
            1 => {
                (*p).width =
                    ((*p.offset(-1i32 as isize)).width as i32 as c_float * (*p).scale_x) as u16
            }
            2 => (*p).width = (viewport_width as c_float * (*p).scale_x) as u16,
            _ => {
                (*p).width = (if last_pass as i32 != 0 {
                    viewport_width as c_float
                } else {
                    (*p.offset(-1i32 as isize)).width as i32 as c_float * (*p).scale_x
                }) as u16
            }
        }
        match (*p).scale_type_y as i32 {
            3 => (*p).height = (*p).scale_y as u16,
            1 => {
                (*p).height =
                    ((*p.offset(-1i32 as isize)).height as i32 as c_float * (*p).scale_y) as u16
            }
            2 => (*p).height = (viewport_height as c_float * (*p).scale_y) as u16,
            _ => {
                (*p).height = (if last_pass as i32 != 0 {
                    viewport_height as c_float
                } else {
                    (*p.offset(-1i32 as isize)).height as i32 as c_float * (*p).scale_y
                }) as u16
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
                    null::<c_void>(),
                );
            } else {
                gl::TexImage2D(
                    GL_TEXTURE_2D as GLenum,
                    0 as GLint,
                    if (*p).float_framebuffer as i32 != 0 {
                        GL_RGBA32F
                    } else {
                        GL_RGBA
                    },
                    (*p).width as GLsizei,
                    (*p).height as GLsizei,
                    0 as GLint,
                    GL_RGBA as GLenum,
                    (if (*p).float_framebuffer as i32 != 0 {
                        GL_FLOAT
                    } else {
                        GL_UNSIGNED_INT_8_8_8_8
                    }) as GLenum,
                    null::<c_void>(),
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
            (*p.offset(-1i32 as isize)).gl_texture as GLuint,
        );
        let filter: u32 = if (*p).filter != 0 {
            (*p).filter
        } else {
            (if last_pass as i32 != 0 && g_config.linear_filtering as i32 != 0 {
                GL_LINEAR
            } else {
                GL_NEAREST
            }) as u32
        };
        gl::TexParameteri(
            GL_TEXTURE_2D as GLenum,
            GL_TEXTURE_MAG_FILTER as GLenum,
            filter as GLint,
        );
        gl::TexParameteri(
            GL_TEXTURE_2D as GLenum,
            GL_TEXTURE_MIN_FILTER as GLenum,
            (if (*p).mipmap_input as i32 != 0 {
                (if filter == GL_LINEAR as u32 {
                    GL_LINEAR_MIPMAP_LINEAR
                } else {
                    GL_NEAREST_MIPMAP_NEAREST
                }) as u32
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
        ctx.num_vaos = 0;
        ctx.texture_unit = ctx.num_vaos;
        ctx.offset = (if last_pass as i32 != 0 {
            (size_of::<c_float>() as usize).wrapping_mul(8_)
        } else {
            0_
        }) as u32;
        GlslShader_SetShaderVars(gs, &raw mut ctx, pass);
        gl::DrawArrays(GL_TRIANGLE_STRIP as GLenum, 0 as GLint, 4 as GLsizei);
        let mut i: i32 = 0;
        while i < ctx.num_vaos as i32 {
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
    if (*gs).max_prev_frame != 0 {
        let store_pos = (&raw mut (*gs).prev_frame as *mut GlTextureWithSize)
            .offset(((*gs).frame_count & 7) as isize);
        let load_pos = (&raw mut (*gs).prev_frame as *mut GlTextureWithSize)
            .offset(((*gs).frame_count.wrapping_sub((*gs).max_prev_frame as u32) & 7) as isize);
        crate::native_assertion! {
            '_c2rust_label: (*store_pos).gl_texture == 0;
            b"store_pos->gl_texture == 0\0", b"src/glsl_shader.c\0", 609,
            b"void GlslShader_Render(GlslShader *, GlTextureWithSize *, int, int, int, int)\0"
        };
        *store_pos = *tex;
        *tex = *load_pos;
        memset(
            load_pos as *mut c_void,
            0,
            size_of::<GlTextureWithSize>() as size_t,
        );
    }
    (*gs).frame_count = (*gs).frame_count.wrapping_add(1);
}
pub const __INT_MAX__: i32 = 2147483647;
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
