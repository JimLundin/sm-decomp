extern "C" {
    fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    fn strncpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn SDL_GL_GetProcAddress(proc_0: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_void;
}
pub use crate::types::{
    __int64_t, int64_t, ptrdiff_t, size_t, GLbitfield, GLboolean, GLbyte, GLchar, GLdouble, GLenum,
    GLfloat, GLint, GLint64, GLintptr, GLshort, GLsizei, GLsizeiptr, GLubyte, GLuint, GLushort,
    GLDEBUGPROC,
};
pub const ogl_LOAD_FAILED: ogl_LoadStatus = 0;
pub use crate::types::ogl_LoadStatus;
pub const ogl_LOAD_SUCCEEDED: ogl_LoadStatus = 1;
pub type ogl_StrToExtMap = ogl_StrToExtMap_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ogl_StrToExtMap_s {
    pub extensionName: *mut ::core::ffi::c_char,
    pub extensionVariable: *mut ::core::ffi::c_int,
    pub LoadExtension: PFN_LOADFUNCPOINTERS,
}
pub use crate::types::PFN_LOADFUNCPOINTERS;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline]
unsafe fn atoi(mut __nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return strtol(
        __nptr,
        NULL as *mut *mut ::core::ffi::c_char,
        10 as ::core::ffi::c_int,
    ) as ::core::ffi::c_int;
}
pub const GL_EXTENSIONS: ::core::ffi::c_int = 0x1f03 as ::core::ffi::c_int;
pub const GL_VERSION: ::core::ffi::c_int = 0x1f02 as ::core::ffi::c_int;
pub const GL_NUM_EXTENSIONS: ::core::ffi::c_int = 0x821d as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_compressed_texture_pixel_storage: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_conservative_depth: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_ES2_compatibility: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_get_program_binary: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_explicit_uniform_location: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_internalformat_query: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_internalformat_query2: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_map_buffer_alignment: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_program_interface_query: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_separate_shader_objects: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_shading_language_420pack: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_shading_language_packing: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_texture_buffer_range: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_texture_storage: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_texture_view: ::core::ffi::c_int = ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_vertex_attrib_binding: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_viewport_array: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_arrays_of_arrays: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_clear_buffer_object: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_copy_image: ::core::ffi::c_int = ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_ES3_compatibility: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_fragment_layer_viewport: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_framebuffer_no_attachments: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_invalidate_subdata: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_robust_buffer_access_behavior: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_stencil_texturing: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_texture_query_levels: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_texture_storage_multisample: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_KHR_debug: ::core::ffi::c_int = ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_buffer_storage: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_clear_texture: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_enhanced_layouts: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_multi_bind: ::core::ffi::c_int = ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_query_buffer_object: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_texture_mirror_clamp_to_edge: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_texture_stencil8: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_vertex_type_10f_11f_11f_rev: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_seamless_cubemap_per_texture: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_clip_control: ::core::ffi::c_int = ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_conditional_render_inverted: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_cull_distance: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_derivative_control: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_direct_state_access: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_get_texture_sub_image: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_shader_texture_image_samples: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_ARB_texture_barrier: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_KHR_context_flush_control: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_KHR_robust_buffer_access_behavior: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_KHR_robustness: ::core::ffi::c_int = ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_EXT_texture_compression_s3tc: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_EXT_texture_sRGB: ::core::ffi::c_int = ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut ogl_ext_EXT_texture_filter_anisotropic: ::core::ffi::c_int =
    ogl_LOAD_FAILED as ::core::ffi::c_int;
pub static mut _ptrc_glClearDepthf: Option<unsafe extern "C" fn(GLfloat) -> ()> = None;
pub static mut _ptrc_glDepthRangef: Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()> = None;
pub static mut _ptrc_glGetShaderPrecisionFormat: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLint, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glReleaseShaderCompiler: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glShaderBinary: Option<
    unsafe extern "C" fn(GLsizei, *const GLuint, GLenum, *const ::core::ffi::c_void, GLsizei) -> (),
> = None;
unsafe fn Load_ARB_ES2_compatibility() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glClearDepthf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearDepthf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearDepthf.is_none() {
        numFailed += 1;
    }
    _ptrc_glDepthRangef = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDepthRangef\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDepthRangef.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetShaderPrecisionFormat = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetShaderPrecisionFormat\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetShaderPrecisionFormat.is_none() {
        numFailed += 1;
    }
    _ptrc_glReleaseShaderCompiler = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn() -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glReleaseShaderCompiler\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glReleaseShaderCompiler.is_none() {
        numFailed += 1;
    }
    _ptrc_glShaderBinary = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLsizei,
                *const GLuint,
                GLenum,
                *const ::core::ffi::c_void,
                GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glShaderBinary\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glShaderBinary.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glGetProgramBinary: Option<
    unsafe extern "C" fn(
        GLuint,
        GLsizei,
        *mut GLsizei,
        *mut GLenum,
        *mut ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glProgramBinary: Option<
    unsafe extern "C" fn(GLuint, GLenum, *const ::core::ffi::c_void, GLsizei) -> (),
> = None;
pub static mut _ptrc_glProgramParameteri: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLint) -> (),
> = None;
unsafe fn Load_ARB_get_program_binary() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glGetProgramBinary = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLsizei,
                *mut GLsizei,
                *mut GLenum,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramBinary\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramBinary.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramBinary = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *const ::core::ffi::c_void, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramBinary\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramBinary.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramParameteri = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramParameteri\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramParameteri.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glGetInternalformativ: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLsizei, *mut GLint) -> (),
> = None;
unsafe fn Load_ARB_internalformat_query() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glGetInternalformativ = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLsizei, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetInternalformativ\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetInternalformativ.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glGetInternalformati64v: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLsizei, *mut GLint64) -> (),
> = None;
unsafe fn Load_ARB_internalformat_query2() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glGetInternalformati64v = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLsizei, *mut GLint64) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetInternalformati64v\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetInternalformati64v.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glGetProgramInterfaceiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetProgramResourceIndex: Option<
    unsafe extern "C" fn(GLuint, GLenum, *const GLchar) -> GLuint,
> = None;
pub static mut _ptrc_glGetProgramResourceLocation: Option<
    unsafe extern "C" fn(GLuint, GLenum, *const GLchar) -> GLint,
> = None;
pub static mut _ptrc_glGetProgramResourceLocationIndex: Option<
    unsafe extern "C" fn(GLuint, GLenum, *const GLchar) -> GLint,
> = None;
pub static mut _ptrc_glGetProgramResourceName: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
> = None;
pub static mut _ptrc_glGetProgramResourceiv: Option<
    unsafe extern "C" fn(
        GLuint,
        GLenum,
        GLuint,
        GLsizei,
        *const GLenum,
        GLsizei,
        *mut GLsizei,
        *mut GLint,
    ) -> (),
> = None;
unsafe fn Load_ARB_program_interface_query() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glGetProgramInterfaceiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramInterfaceiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramInterfaceiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetProgramResourceIndex = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *const GLchar) -> GLuint>,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramResourceIndex\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramResourceIndex.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetProgramResourceLocation = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *const GLchar) -> GLint>,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramResourceLocation\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramResourceLocation.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetProgramResourceLocationIndex = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *const GLchar) -> GLint>,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramResourceLocationIndex\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramResourceLocationIndex.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetProgramResourceName = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLuint, GLenum, GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramResourceName\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramResourceName.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetProgramResourceiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLenum,
                GLuint,
                GLsizei,
                *const GLenum,
                GLsizei,
                *mut GLsizei,
                *mut GLint,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramResourceiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramResourceiv.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glActiveShaderProgram: Option<unsafe extern "C" fn(GLuint, GLuint) -> ()> =
    None;
pub static mut _ptrc_glBindProgramPipeline: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glCreateShaderProgramv: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *const *const GLchar) -> GLuint,
> = None;
pub static mut _ptrc_glDeleteProgramPipelines: Option<
    unsafe extern "C" fn(GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glGenProgramPipelines: Option<
    unsafe extern "C" fn(GLsizei, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glGetProgramPipelineInfoLog: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
> = None;
pub static mut _ptrc_glGetProgramPipelineiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glIsProgramPipeline: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glProgramUniform1d: Option<
    unsafe extern "C" fn(GLuint, GLint, GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniform1dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniform1f: Option<
    unsafe extern "C" fn(GLuint, GLint, GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniform1fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniform1i: Option<unsafe extern "C" fn(GLuint, GLint, GLint) -> ()> =
    None;
pub static mut _ptrc_glProgramUniform1iv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform1ui: Option<
    unsafe extern "C" fn(GLuint, GLint, GLuint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform1uiv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform2d: Option<
    unsafe extern "C" fn(GLuint, GLint, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniform2dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniform2f: Option<
    unsafe extern "C" fn(GLuint, GLint, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniform2fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniform2i: Option<
    unsafe extern "C" fn(GLuint, GLint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform2iv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform2ui: Option<
    unsafe extern "C" fn(GLuint, GLint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform2uiv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform3d: Option<
    unsafe extern "C" fn(GLuint, GLint, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniform3dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniform3f: Option<
    unsafe extern "C" fn(GLuint, GLint, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniform3fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniform3i: Option<
    unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform3iv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform3ui: Option<
    unsafe extern "C" fn(GLuint, GLint, GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform3uiv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform4d: Option<
    unsafe extern "C" fn(GLuint, GLint, GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniform4dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniform4f: Option<
    unsafe extern "C" fn(GLuint, GLint, GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniform4fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniform4i: Option<
    unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform4iv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform4ui: Option<
    unsafe extern "C" fn(GLuint, GLint, GLuint, GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glProgramUniform4uiv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix2dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix2fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix2x3dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix2x3fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix2x4dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix2x4fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix3dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix3fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix3x2dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix3x2fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix3x4dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix3x4fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix4dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix4fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix4x2dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix4x2fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix4x3dv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glProgramUniformMatrix4x3fv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUseProgramStages: Option<
    unsafe extern "C" fn(GLuint, GLbitfield, GLuint) -> (),
> = None;
pub static mut _ptrc_glValidateProgramPipeline: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
unsafe fn Load_ARB_separate_shader_objects() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glActiveShaderProgram = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glActiveShaderProgram\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glActiveShaderProgram.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindProgramPipeline = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindProgramPipeline\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindProgramPipeline.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateShaderProgramv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *const *const GLchar) -> GLuint>,
    >(SDL_GL_GetProcAddress(
        b"glCreateShaderProgramv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateShaderProgramv.is_none() {
        numFailed += 1;
    }
    _ptrc_glDeleteProgramPipelines = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDeleteProgramPipelines\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDeleteProgramPipelines.is_none() {
        numFailed += 1;
    }
    _ptrc_glGenProgramPipelines = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGenProgramPipelines\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGenProgramPipelines.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetProgramPipelineInfoLog = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramPipelineInfoLog\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramPipelineInfoLog.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetProgramPipelineiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramPipelineiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramPipelineiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsProgramPipeline = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsProgramPipeline\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsProgramPipeline.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform1d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform1d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform1d.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform1dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform1dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform1dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform1f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform1f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform1f.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform1fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform1fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform1fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform1i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform1i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform1i.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform1iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform1iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform1iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform1ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform1ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform1ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform1uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform1uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform1uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform2d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform2d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform2d.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform2i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform2i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform2i.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform2iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform2iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform2iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform2ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform2ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform2ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform2uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform2uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform2uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform3d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform3d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform3d.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform3ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform3ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform3ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform3uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform3uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform3uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform4d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLdouble, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform4d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform4d.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform4dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform4dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform4dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform4f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform4f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform4f.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform4i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform4i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform4i.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform4iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform4iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform4iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform4ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLuint, GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform4ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform4ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniform4uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniform4uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniform4uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix2x3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix2x3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix2x3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix2x3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix2x3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix2x3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix2x4dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix2x4dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix2x4dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix2x4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix2x4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix2x4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix3x2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix3x2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix3x2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix3x2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix3x2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix3x2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix3x4dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix3x4dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix3x4dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix3x4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix3x4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix3x4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix4dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix4dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix4dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix4x2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix4x2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix4x2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix4x2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix4x2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix4x2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix4x3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix4x3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix4x3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glProgramUniformMatrix4x3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glProgramUniformMatrix4x3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glProgramUniformMatrix4x3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUseProgramStages = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLbitfield, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUseProgramStages\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUseProgramStages.is_none() {
        numFailed += 1;
    }
    _ptrc_glValidateProgramPipeline = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glValidateProgramPipeline\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glValidateProgramPipeline.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glTexBufferRange: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLuint, GLintptr, GLsizeiptr) -> (),
> = None;
unsafe fn Load_ARB_texture_buffer_range() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glTexBufferRange = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLuint, GLintptr, GLsizeiptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexBufferRange\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexBufferRange.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glTexStorage1D: Option<
    unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei) -> (),
> = None;
pub static mut _ptrc_glTexStorage2D: Option<
    unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glTexStorage3D: Option<
    unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei, GLsizei, GLsizei) -> (),
> = None;
unsafe fn Load_ARB_texture_storage() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glTexStorage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexStorage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexStorage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexStorage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexStorage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexStorage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexStorage3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexStorage3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexStorage3D.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glTextureView: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLuint, GLenum, GLuint, GLuint, GLuint, GLuint) -> (),
> = None;
unsafe fn Load_ARB_texture_view() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glTextureView = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLenum,
                GLuint,
                GLenum,
                GLuint,
                GLuint,
                GLuint,
                GLuint,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTextureView\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureView.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glBindVertexBuffer: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLintptr, GLsizei) -> (),
> = None;
pub static mut _ptrc_glVertexAttribBinding: Option<unsafe extern "C" fn(GLuint, GLuint) -> ()> =
    None;
pub static mut _ptrc_glVertexAttribFormat: Option<
    unsafe extern "C" fn(GLuint, GLint, GLenum, GLboolean, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribIFormat: Option<
    unsafe extern "C" fn(GLuint, GLint, GLenum, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribLFormat: Option<
    unsafe extern "C" fn(GLuint, GLint, GLenum, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexBindingDivisor: Option<unsafe extern "C" fn(GLuint, GLuint) -> ()> =
    None;
unsafe fn Load_ARB_vertex_attrib_binding() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glBindVertexBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLintptr, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindVertexBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindVertexBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribBinding = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribBinding\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribBinding.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribFormat = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLenum, GLboolean, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribFormat\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribFormat.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribIFormat = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribIFormat\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribIFormat.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribLFormat = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribLFormat\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribLFormat.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexBindingDivisor = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexBindingDivisor\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexBindingDivisor.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glDepthRangeArrayv: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glDepthRangeIndexed: Option<
    unsafe extern "C" fn(GLuint, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glGetDoublei_v: Option<
    unsafe extern "C" fn(GLenum, GLuint, *mut GLdouble) -> (),
> = None;
pub static mut _ptrc_glGetFloati_v: Option<
    unsafe extern "C" fn(GLenum, GLuint, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glScissorArrayv: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLint) -> (),
> = None;
pub static mut _ptrc_glScissorIndexed: Option<
    unsafe extern "C" fn(GLuint, GLint, GLint, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glScissorIndexedv: Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glViewportArrayv: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glViewportIndexedf: Option<
    unsafe extern "C" fn(GLuint, GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glViewportIndexedfv: Option<
    unsafe extern "C" fn(GLuint, *const GLfloat) -> (),
> = None;
unsafe fn Load_ARB_viewport_array() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glDepthRangeArrayv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDepthRangeArrayv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDepthRangeArrayv.is_none() {
        numFailed += 1;
    }
    _ptrc_glDepthRangeIndexed = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDepthRangeIndexed\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDepthRangeIndexed.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetDoublei_v = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint, *mut GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetDoublei_v\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetDoublei_v.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetFloati_v = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetFloati_v\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetFloati_v.is_none() {
        numFailed += 1;
    }
    _ptrc_glScissorArrayv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glScissorArrayv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glScissorArrayv.is_none() {
        numFailed += 1;
    }
    _ptrc_glScissorIndexed = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLint, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glScissorIndexed\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glScissorIndexed.is_none() {
        numFailed += 1;
    }
    _ptrc_glScissorIndexedv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glScissorIndexedv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glScissorIndexedv.is_none() {
        numFailed += 1;
    }
    _ptrc_glViewportArrayv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glViewportArrayv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glViewportArrayv.is_none() {
        numFailed += 1;
    }
    _ptrc_glViewportIndexedf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glViewportIndexedf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glViewportIndexedf.is_none() {
        numFailed += 1;
    }
    _ptrc_glViewportIndexedfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glViewportIndexedfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glViewportIndexedfv.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glClearBufferData: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLenum, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glClearBufferSubData: Option<
    unsafe extern "C" fn(
        GLenum,
        GLenum,
        GLintptr,
        GLsizeiptr,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
unsafe fn Load_ARB_clear_buffer_object() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glClearBufferData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLenum, GLenum, GLenum, GLenum, *const ::core::ffi::c_void) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glClearBufferData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearBufferData.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearBufferSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLenum,
                GLintptr,
                GLsizeiptr,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glClearBufferSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearBufferSubData.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glCopyImageSubData: Option<
    unsafe extern "C" fn(
        GLuint,
        GLenum,
        GLint,
        GLint,
        GLint,
        GLint,
        GLuint,
        GLenum,
        GLint,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLsizei,
    ) -> (),
> = None;
unsafe fn Load_ARB_copy_image() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glCopyImageSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLenum,
                GLint,
                GLint,
                GLint,
                GLint,
                GLuint,
                GLenum,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCopyImageSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyImageSubData.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glFramebufferParameteri: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLint) -> (),
> = None;
pub static mut _ptrc_glGetFramebufferParameteriv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> (),
> = None;
unsafe fn Load_ARB_framebuffer_no_attachments() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glFramebufferParameteri = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFramebufferParameteri\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFramebufferParameteri.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetFramebufferParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetFramebufferParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetFramebufferParameteriv.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glInvalidateBufferData: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glInvalidateBufferSubData: Option<
    unsafe extern "C" fn(GLuint, GLintptr, GLsizeiptr) -> (),
> = None;
pub static mut _ptrc_glInvalidateFramebuffer: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *const GLenum) -> (),
> = None;
pub static mut _ptrc_glInvalidateSubFramebuffer: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *const GLenum, GLint, GLint, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glInvalidateTexImage: Option<unsafe extern "C" fn(GLuint, GLint) -> ()> = None;
pub static mut _ptrc_glInvalidateTexSubImage: Option<
    unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint, GLsizei, GLsizei, GLsizei) -> (),
> = None;
unsafe fn Load_ARB_invalidate_subdata() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glInvalidateBufferData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glInvalidateBufferData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glInvalidateBufferData.is_none() {
        numFailed += 1;
    }
    _ptrc_glInvalidateBufferSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLintptr, GLsizeiptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glInvalidateBufferSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glInvalidateBufferSubData.is_none() {
        numFailed += 1;
    }
    _ptrc_glInvalidateFramebuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *const GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glInvalidateFramebuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glInvalidateFramebuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glInvalidateSubFramebuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLsizei,
                *const GLenum,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glInvalidateSubFramebuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glInvalidateSubFramebuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glInvalidateTexImage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glInvalidateTexImage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glInvalidateTexImage.is_none() {
        numFailed += 1;
    }
    _ptrc_glInvalidateTexSubImage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glInvalidateTexSubImage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glInvalidateTexSubImage.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glTexStorage2DMultisample: Option<
    unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei, GLsizei, GLboolean) -> (),
> = None;
pub static mut _ptrc_glTexStorage3DMultisample: Option<
    unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei, GLsizei, GLsizei, GLboolean) -> (),
> = None;
unsafe fn Load_ARB_texture_storage_multisample() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glTexStorage2DMultisample = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei, GLsizei, GLboolean) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexStorage2DMultisample\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexStorage2DMultisample.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexStorage3DMultisample = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLsizei,
                GLenum,
                GLsizei,
                GLsizei,
                GLsizei,
                GLboolean,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTexStorage3DMultisample\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexStorage3DMultisample.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glDebugMessageCallback: Option<
    unsafe extern "C" fn(GLDEBUGPROC, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glDebugMessageControl: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLsizei, *const GLuint, GLboolean) -> (),
> = None;
pub static mut _ptrc_glDebugMessageInsert: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLuint, GLenum, GLsizei, *const GLchar) -> (),
> = None;
pub static mut _ptrc_glGetDebugMessageLog: Option<
    unsafe extern "C" fn(
        GLuint,
        GLsizei,
        *mut GLenum,
        *mut GLenum,
        *mut GLuint,
        *mut GLenum,
        *mut GLsizei,
        *mut GLchar,
    ) -> GLuint,
> = None;
pub static mut _ptrc_glGetObjectLabel: Option<
    unsafe extern "C" fn(GLenum, GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
> = None;
pub static mut _ptrc_glGetObjectPtrLabel: Option<
    unsafe extern "C" fn(*const ::core::ffi::c_void, GLsizei, *mut GLsizei, *mut GLchar) -> (),
> = None;
pub static mut _ptrc_glGetPointerv: Option<
    unsafe extern "C" fn(GLenum, *mut *mut ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glObjectLabel: Option<
    unsafe extern "C" fn(GLenum, GLuint, GLsizei, *const GLchar) -> (),
> = None;
pub static mut _ptrc_glObjectPtrLabel: Option<
    unsafe extern "C" fn(*const ::core::ffi::c_void, GLsizei, *const GLchar) -> (),
> = None;
pub static mut _ptrc_glPopDebugGroup: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glPushDebugGroup: Option<
    unsafe extern "C" fn(GLenum, GLuint, GLsizei, *const GLchar) -> (),
> = None;
unsafe fn Load_KHR_debug() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glDebugMessageCallback = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLDEBUGPROC, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDebugMessageCallback\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDebugMessageCallback.is_none() {
        numFailed += 1;
    }
    _ptrc_glDebugMessageControl = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLenum, GLenum, GLenum, GLsizei, *const GLuint, GLboolean) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glDebugMessageControl\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDebugMessageControl.is_none() {
        numFailed += 1;
    }
    _ptrc_glDebugMessageInsert = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLuint, GLenum, GLsizei, *const GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDebugMessageInsert\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDebugMessageInsert.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetDebugMessageLog = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLsizei,
                *mut GLenum,
                *mut GLenum,
                *mut GLuint,
                *mut GLenum,
                *mut GLsizei,
                *mut GLchar,
            ) -> GLuint,
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetDebugMessageLog\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetDebugMessageLog.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetObjectLabel = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetObjectLabel\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetObjectLabel.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetObjectPtrLabel = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_void,
                GLsizei,
                *mut GLsizei,
                *mut GLchar,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetObjectPtrLabel\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetObjectPtrLabel.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetPointerv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *mut *mut ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetPointerv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetPointerv.is_none() {
        numFailed += 1;
    }
    _ptrc_glObjectLabel = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint, GLsizei, *const GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glObjectLabel\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glObjectLabel.is_none() {
        numFailed += 1;
    }
    _ptrc_glObjectPtrLabel = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const ::core::ffi::c_void, GLsizei, *const GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glObjectPtrLabel\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glObjectPtrLabel.is_none() {
        numFailed += 1;
    }
    _ptrc_glPopDebugGroup =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glPopDebugGroup\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glPopDebugGroup.is_none() {
        numFailed += 1;
    }
    _ptrc_glPushDebugGroup = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint, GLsizei, *const GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPushDebugGroup\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPushDebugGroup.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glBufferStorage: Option<
    unsafe extern "C" fn(GLenum, GLsizeiptr, *const ::core::ffi::c_void, GLbitfield) -> (),
> = None;
unsafe fn Load_ARB_buffer_storage() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glBufferStorage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLenum, GLsizeiptr, *const ::core::ffi::c_void, GLbitfield) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glBufferStorage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBufferStorage.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glClearTexImage: Option<
    unsafe extern "C" fn(GLuint, GLint, GLenum, GLenum, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glClearTexSubImage: Option<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLsizei,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
unsafe fn Load_ARB_clear_texture() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glClearTexImage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLuint, GLint, GLenum, GLenum, *const ::core::ffi::c_void) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glClearTexImage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearTexImage.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearTexSubImage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLsizei,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glClearTexSubImage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearTexSubImage.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glBindBuffersBase: Option<
    unsafe extern "C" fn(GLenum, GLuint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glBindBuffersRange: Option<
    unsafe extern "C" fn(
        GLenum,
        GLuint,
        GLsizei,
        *const GLuint,
        *const GLintptr,
        *const GLsizeiptr,
    ) -> (),
> = None;
pub static mut _ptrc_glBindImageTextures: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glBindSamplers: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glBindTextures: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glBindVertexBuffers: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLuint, *const GLintptr, *const GLsizei) -> (),
> = None;
unsafe fn Load_ARB_multi_bind() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glBindBuffersBase = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindBuffersBase\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindBuffersBase.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindBuffersRange = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLuint,
                GLsizei,
                *const GLuint,
                *const GLintptr,
                *const GLsizeiptr,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glBindBuffersRange\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindBuffersRange.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindImageTextures = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindImageTextures\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindImageTextures.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindSamplers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindSamplers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindSamplers.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindTextures = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindTextures\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindTextures.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindVertexBuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLsizei,
                *const GLuint,
                *const GLintptr,
                *const GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glBindVertexBuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindVertexBuffers.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glClipControl: Option<unsafe extern "C" fn(GLenum, GLenum) -> ()> = None;
unsafe fn Load_ARB_clip_control() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glClipControl = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClipControl\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClipControl.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glBindTextureUnit: Option<unsafe extern "C" fn(GLuint, GLuint) -> ()> = None;
pub static mut _ptrc_glBlitNamedFramebuffer: Option<
    unsafe extern "C" fn(
        GLuint,
        GLuint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLbitfield,
        GLenum,
    ) -> (),
> = None;
pub static mut _ptrc_glCheckNamedFramebufferStatus: Option<
    unsafe extern "C" fn(GLuint, GLenum) -> GLenum,
> = None;
pub static mut _ptrc_glClearNamedBufferData: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLenum, GLenum, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glClearNamedBufferSubData: Option<
    unsafe extern "C" fn(
        GLuint,
        GLenum,
        GLintptr,
        GLsizeiptr,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glClearNamedFramebufferfi: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLint, GLfloat, GLint) -> (),
> = None;
pub static mut _ptrc_glClearNamedFramebufferfv: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLint, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glClearNamedFramebufferiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLint, *const GLint) -> (),
> = None;
pub static mut _ptrc_glClearNamedFramebufferuiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLint, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glCompressedTextureSubImage1D: Option<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLint,
        GLsizei,
        GLenum,
        GLsizei,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glCompressedTextureSubImage2D: Option<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLenum,
        GLsizei,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glCompressedTextureSubImage3D: Option<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLsizei,
        GLenum,
        GLsizei,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glCopyNamedBufferSubData: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLintptr, GLintptr, GLsizeiptr) -> (),
> = None;
pub static mut _ptrc_glCopyTextureSubImage1D: Option<
    unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint, GLsizei) -> (),
> = None;
pub static mut _ptrc_glCopyTextureSubImage2D: Option<
    unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint, GLint, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glCopyTextureSubImage3D: Option<
    unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint, GLint, GLint, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glCreateBuffers: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()> =
    None;
pub static mut _ptrc_glCreateFramebuffers: Option<
    unsafe extern "C" fn(GLsizei, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glCreateProgramPipelines: Option<
    unsafe extern "C" fn(GLsizei, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glCreateQueries: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glCreateRenderbuffers: Option<
    unsafe extern "C" fn(GLsizei, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glCreateSamplers: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()> =
    None;
pub static mut _ptrc_glCreateTextures: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glCreateTransformFeedbacks: Option<
    unsafe extern "C" fn(GLsizei, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glCreateVertexArrays: Option<
    unsafe extern "C" fn(GLsizei, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glDisableVertexArrayAttrib: Option<
    unsafe extern "C" fn(GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glEnableVertexArrayAttrib: Option<unsafe extern "C" fn(GLuint, GLuint) -> ()> =
    None;
pub static mut _ptrc_glFlushMappedNamedBufferRange: Option<
    unsafe extern "C" fn(GLuint, GLintptr, GLsizeiptr) -> (),
> = None;
pub static mut _ptrc_glGenerateTextureMipmap: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glGetCompressedTextureImage: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *mut ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glGetNamedBufferParameteri64v: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint64) -> (),
> = None;
pub static mut _ptrc_glGetNamedBufferParameteriv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetNamedBufferPointerv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut *mut ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glGetNamedBufferSubData: Option<
    unsafe extern "C" fn(GLuint, GLintptr, GLsizeiptr, *mut ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glGetNamedFramebufferAttachmentParameteriv: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetNamedFramebufferParameteriv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetNamedRenderbufferParameteriv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetQueryBufferObjecti64v: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLenum, GLintptr) -> (),
> = None;
pub static mut _ptrc_glGetQueryBufferObjectiv: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLenum, GLintptr) -> (),
> = None;
pub static mut _ptrc_glGetQueryBufferObjectui64v: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLenum, GLintptr) -> (),
> = None;
pub static mut _ptrc_glGetQueryBufferObjectuiv: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLenum, GLintptr) -> (),
> = None;
pub static mut _ptrc_glGetTextureImage: Option<
    unsafe extern "C" fn(GLuint, GLint, GLenum, GLenum, GLsizei, *mut ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glGetTextureLevelParameterfv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLenum, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetTextureLevelParameteriv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetTextureParameterIiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetTextureParameterIuiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glGetTextureParameterfv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetTextureParameteriv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetTransformFeedbacki64_v: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLuint, *mut GLint64) -> (),
> = None;
pub static mut _ptrc_glGetTransformFeedbacki_v: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLuint, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetTransformFeedbackiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetVertexArrayIndexed64iv: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLenum, *mut GLint64) -> (),
> = None;
pub static mut _ptrc_glGetVertexArrayIndexediv: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetVertexArrayiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glInvalidateNamedFramebufferData: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLenum) -> (),
> = None;
pub static mut _ptrc_glInvalidateNamedFramebufferSubData: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLenum, GLint, GLint, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glMapNamedBuffer: Option<
    unsafe extern "C" fn(GLuint, GLenum) -> *mut ::core::ffi::c_void,
> = None;
pub static mut _ptrc_glMapNamedBufferRange: Option<
    unsafe extern "C" fn(GLuint, GLintptr, GLsizeiptr, GLbitfield) -> *mut ::core::ffi::c_void,
> = None;
pub static mut _ptrc_glNamedBufferData: Option<
    unsafe extern "C" fn(GLuint, GLsizeiptr, *const ::core::ffi::c_void, GLenum) -> (),
> = None;
pub static mut _ptrc_glNamedBufferStorage: Option<
    unsafe extern "C" fn(GLuint, GLsizeiptr, *const ::core::ffi::c_void, GLbitfield) -> (),
> = None;
pub static mut _ptrc_glNamedBufferSubData: Option<
    unsafe extern "C" fn(GLuint, GLintptr, GLsizeiptr, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glNamedFramebufferDrawBuffer: Option<
    unsafe extern "C" fn(GLuint, GLenum) -> (),
> = None;
pub static mut _ptrc_glNamedFramebufferDrawBuffers: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLenum) -> (),
> = None;
pub static mut _ptrc_glNamedFramebufferParameteri: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLint) -> (),
> = None;
pub static mut _ptrc_glNamedFramebufferReadBuffer: Option<
    unsafe extern "C" fn(GLuint, GLenum) -> (),
> = None;
pub static mut _ptrc_glNamedFramebufferRenderbuffer: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLenum, GLuint) -> (),
> = None;
pub static mut _ptrc_glNamedFramebufferTexture: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLuint, GLint) -> (),
> = None;
pub static mut _ptrc_glNamedFramebufferTextureLayer: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLuint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glNamedRenderbufferStorage: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glNamedRenderbufferStorageMultisample: Option<
    unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glTextureBuffer: Option<unsafe extern "C" fn(GLuint, GLenum, GLuint) -> ()> =
    None;
pub static mut _ptrc_glTextureBufferRange: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLuint, GLintptr, GLsizeiptr) -> (),
> = None;
pub static mut _ptrc_glTextureParameterIiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *const GLint) -> (),
> = None;
pub static mut _ptrc_glTextureParameterIuiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glTextureParameterf: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLfloat) -> (),
> = None;
pub static mut _ptrc_glTextureParameterfv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glTextureParameteri: Option<
    unsafe extern "C" fn(GLuint, GLenum, GLint) -> (),
> = None;
pub static mut _ptrc_glTextureParameteriv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *const GLint) -> (),
> = None;
pub static mut _ptrc_glTextureStorage1D: Option<
    unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei) -> (),
> = None;
pub static mut _ptrc_glTextureStorage2D: Option<
    unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glTextureStorage2DMultisample: Option<
    unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei, GLsizei, GLboolean) -> (),
> = None;
pub static mut _ptrc_glTextureStorage3D: Option<
    unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glTextureStorage3DMultisample: Option<
    unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei, GLsizei, GLsizei, GLboolean) -> (),
> = None;
pub static mut _ptrc_glTextureSubImage1D: Option<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLint,
        GLsizei,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glTextureSubImage2D: Option<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glTextureSubImage3D: Option<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLsizei,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glTransformFeedbackBufferBase: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glTransformFeedbackBufferRange: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint, GLintptr, GLsizeiptr) -> (),
> = None;
pub static mut _ptrc_glUnmapNamedBuffer: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glVertexArrayAttribBinding: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexArrayAttribFormat: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLint, GLenum, GLboolean, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexArrayAttribIFormat: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLint, GLenum, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexArrayAttribLFormat: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLint, GLenum, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexArrayBindingDivisor: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexArrayElementBuffer: Option<
    unsafe extern "C" fn(GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexArrayVertexBuffer: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint, GLintptr, GLsizei) -> (),
> = None;
pub static mut _ptrc_glVertexArrayVertexBuffers: Option<
    unsafe extern "C" fn(
        GLuint,
        GLuint,
        GLsizei,
        *const GLuint,
        *const GLintptr,
        *const GLsizei,
    ) -> (),
> = None;
unsafe fn Load_ARB_direct_state_access() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glBindTextureUnit = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindTextureUnit\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindTextureUnit.is_none() {
        numFailed += 1;
    }
    _ptrc_glBlitNamedFramebuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLuint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLbitfield,
                GLenum,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glBlitNamedFramebuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBlitNamedFramebuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glCheckNamedFramebufferStatus = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum) -> GLenum>,
    >(SDL_GL_GetProcAddress(
        b"glCheckNamedFramebufferStatus\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCheckNamedFramebufferStatus.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearNamedBufferData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLuint, GLenum, GLenum, GLenum, *const ::core::ffi::c_void) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glClearNamedBufferData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearNamedBufferData.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearNamedBufferSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLenum,
                GLintptr,
                GLsizeiptr,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glClearNamedBufferSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearNamedBufferSubData.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearNamedFramebufferfi = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLint, GLfloat, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearNamedFramebufferfi\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearNamedFramebufferfi.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearNamedFramebufferfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLint, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearNamedFramebufferfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearNamedFramebufferfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearNamedFramebufferiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLint, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearNamedFramebufferiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearNamedFramebufferiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearNamedFramebufferuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLint, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearNamedFramebufferuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearNamedFramebufferuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glCompressedTextureSubImage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLsizei,
                GLenum,
                GLsizei,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCompressedTextureSubImage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCompressedTextureSubImage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCompressedTextureSubImage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLenum,
                GLsizei,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCompressedTextureSubImage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCompressedTextureSubImage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCompressedTextureSubImage3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLsizei,
                GLenum,
                GLsizei,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCompressedTextureSubImage3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCompressedTextureSubImage3D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyNamedBufferSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLintptr, GLintptr, GLsizeiptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCopyNamedBufferSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyNamedBufferSubData.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyTextureSubImage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCopyTextureSubImage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyTextureSubImage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyTextureSubImage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint, GLint, GLsizei, GLsizei) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCopyTextureSubImage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyTextureSubImage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyTextureSubImage3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCopyTextureSubImage3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyTextureSubImage3D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateBuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCreateBuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateBuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateFramebuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCreateFramebuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateFramebuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateProgramPipelines = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCreateProgramPipelines\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateProgramPipelines.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateQueries = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCreateQueries\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateQueries.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateRenderbuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCreateRenderbuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateRenderbuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateSamplers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCreateSamplers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateSamplers.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateTextures = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCreateTextures\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateTextures.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateTransformFeedbacks = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCreateTransformFeedbacks\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateTransformFeedbacks.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateVertexArrays = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCreateVertexArrays\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateVertexArrays.is_none() {
        numFailed += 1;
    }
    _ptrc_glDisableVertexArrayAttrib = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDisableVertexArrayAttrib\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDisableVertexArrayAttrib.is_none() {
        numFailed += 1;
    }
    _ptrc_glEnableVertexArrayAttrib = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEnableVertexArrayAttrib\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEnableVertexArrayAttrib.is_none() {
        numFailed += 1;
    }
    _ptrc_glFlushMappedNamedBufferRange = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLintptr, GLsizeiptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFlushMappedNamedBufferRange\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFlushMappedNamedBufferRange.is_none() {
        numFailed += 1;
    }
    _ptrc_glGenerateTextureMipmap = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGenerateTextureMipmap\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGenerateTextureMipmap.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetCompressedTextureImage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *mut ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetCompressedTextureImage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetCompressedTextureImage.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetNamedBufferParameteri64v = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint64) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetNamedBufferParameteri64v\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetNamedBufferParameteri64v.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetNamedBufferParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetNamedBufferParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetNamedBufferParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetNamedBufferPointerv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut *mut ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetNamedBufferPointerv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetNamedBufferPointerv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetNamedBufferSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLintptr, GLsizeiptr, *mut ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetNamedBufferSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetNamedBufferSubData.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetNamedFramebufferAttachmentParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetNamedFramebufferAttachmentParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetNamedFramebufferAttachmentParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetNamedFramebufferParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetNamedFramebufferParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetNamedFramebufferParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetNamedRenderbufferParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetNamedRenderbufferParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetNamedRenderbufferParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetQueryBufferObjecti64v = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLenum, GLintptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetQueryBufferObjecti64v\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetQueryBufferObjecti64v.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetQueryBufferObjectiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLenum, GLintptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetQueryBufferObjectiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetQueryBufferObjectiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetQueryBufferObjectui64v = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLenum, GLintptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetQueryBufferObjectui64v\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetQueryBufferObjectui64v.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetQueryBufferObjectuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLenum, GLintptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetQueryBufferObjectuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetQueryBufferObjectuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTextureImage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLenum,
                GLenum,
                GLsizei,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetTextureImage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTextureImage.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTextureLevelParameterfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTextureLevelParameterfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTextureLevelParameterfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTextureLevelParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTextureLevelParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTextureLevelParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTextureParameterIiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTextureParameterIiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTextureParameterIiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTextureParameterIuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTextureParameterIuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTextureParameterIuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTextureParameterfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTextureParameterfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTextureParameterfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTextureParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTextureParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTextureParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTransformFeedbacki64_v = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLuint, *mut GLint64) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTransformFeedbacki64_v\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTransformFeedbacki64_v.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTransformFeedbacki_v = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLuint, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTransformFeedbacki_v\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTransformFeedbacki_v.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTransformFeedbackiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTransformFeedbackiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTransformFeedbackiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetVertexArrayIndexed64iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLenum, *mut GLint64) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetVertexArrayIndexed64iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetVertexArrayIndexed64iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetVertexArrayIndexediv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetVertexArrayIndexediv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetVertexArrayIndexediv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetVertexArrayiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetVertexArrayiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetVertexArrayiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glInvalidateNamedFramebufferData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glInvalidateNamedFramebufferData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glInvalidateNamedFramebufferData.is_none() {
        numFailed += 1;
    }
    _ptrc_glInvalidateNamedFramebufferSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLsizei,
                *const GLenum,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glInvalidateNamedFramebufferSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glInvalidateNamedFramebufferSubData.is_none() {
        numFailed += 1;
    }
    _ptrc_glMapNamedBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum) -> *mut ::core::ffi::c_void>,
    >(SDL_GL_GetProcAddress(
        b"glMapNamedBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMapNamedBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glMapNamedBufferRange = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLintptr,
                GLsizeiptr,
                GLbitfield,
            ) -> *mut ::core::ffi::c_void,
        >,
    >(SDL_GL_GetProcAddress(
        b"glMapNamedBufferRange\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMapNamedBufferRange.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedBufferData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizeiptr, *const ::core::ffi::c_void, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNamedBufferData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedBufferData.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedBufferStorage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLuint, GLsizeiptr, *const ::core::ffi::c_void, GLbitfield) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glNamedBufferStorage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedBufferStorage.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedBufferSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLuint, GLintptr, GLsizeiptr, *const ::core::ffi::c_void) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glNamedBufferSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedBufferSubData.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedFramebufferDrawBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNamedFramebufferDrawBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedFramebufferDrawBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedFramebufferDrawBuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNamedFramebufferDrawBuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedFramebufferDrawBuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedFramebufferParameteri = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNamedFramebufferParameteri\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedFramebufferParameteri.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedFramebufferReadBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNamedFramebufferReadBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedFramebufferReadBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedFramebufferRenderbuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNamedFramebufferRenderbuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedFramebufferRenderbuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedFramebufferTexture = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLuint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNamedFramebufferTexture\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedFramebufferTexture.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedFramebufferTextureLayer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLuint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNamedFramebufferTextureLayer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedFramebufferTextureLayer.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedRenderbufferStorage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNamedRenderbufferStorage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedRenderbufferStorage.is_none() {
        numFailed += 1;
    }
    _ptrc_glNamedRenderbufferStorageMultisample = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNamedRenderbufferStorageMultisample\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNamedRenderbufferStorageMultisample.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureBufferRange = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLuint, GLintptr, GLsizeiptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureBufferRange\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureBufferRange.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureParameterIiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureParameterIiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureParameterIiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureParameterIuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureParameterIuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureParameterIuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureParameterf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureParameterf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureParameterf.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureParameterfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureParameterfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureParameterfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureParameteri = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureParameteri\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureParameteri.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureStorage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureStorage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureStorage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureStorage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureStorage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureStorage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureStorage2DMultisample = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei, GLsizei, GLboolean) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureStorage2DMultisample\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureStorage2DMultisample.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureStorage3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, GLenum, GLsizei, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTextureStorage3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureStorage3D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureStorage3DMultisample = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLsizei,
                GLenum,
                GLsizei,
                GLsizei,
                GLsizei,
                GLboolean,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTextureStorage3DMultisample\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureStorage3DMultisample.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureSubImage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLsizei,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTextureSubImage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureSubImage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureSubImage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTextureSubImage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureSubImage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTextureSubImage3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLsizei,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTextureSubImage3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTextureSubImage3D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTransformFeedbackBufferBase = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTransformFeedbackBufferBase\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTransformFeedbackBufferBase.is_none() {
        numFailed += 1;
    }
    _ptrc_glTransformFeedbackBufferRange = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint, GLintptr, GLsizeiptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTransformFeedbackBufferRange\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTransformFeedbackBufferRange.is_none() {
        numFailed += 1;
    }
    _ptrc_glUnmapNamedBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glUnmapNamedBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUnmapNamedBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexArrayAttribBinding = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexArrayAttribBinding\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexArrayAttribBinding.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexArrayAttribFormat = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLint, GLenum, GLboolean, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexArrayAttribFormat\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexArrayAttribFormat.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexArrayAttribIFormat = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLint, GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexArrayAttribIFormat\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexArrayAttribIFormat.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexArrayAttribLFormat = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLint, GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexArrayAttribLFormat\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexArrayAttribLFormat.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexArrayBindingDivisor = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexArrayBindingDivisor\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexArrayBindingDivisor.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexArrayElementBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexArrayElementBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexArrayElementBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexArrayVertexBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint, GLintptr, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexArrayVertexBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexArrayVertexBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexArrayVertexBuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLuint,
                GLsizei,
                *const GLuint,
                *const GLintptr,
                *const GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glVertexArrayVertexBuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexArrayVertexBuffers.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glGetCompressedTextureSubImage: Option<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLsizei,
        GLsizei,
        *mut ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glGetTextureSubImage: Option<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLsizei,
        GLenum,
        GLenum,
        GLsizei,
        *mut ::core::ffi::c_void,
    ) -> (),
> = None;
unsafe fn Load_ARB_get_texture_sub_image() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glGetCompressedTextureSubImage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLsizei,
                GLsizei,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetCompressedTextureSubImage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetCompressedTextureSubImage.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTextureSubImage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLsizei,
                GLenum,
                GLenum,
                GLsizei,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetTextureSubImage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTextureSubImage.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glTextureBarrier: Option<unsafe extern "C" fn() -> ()> = None;
unsafe fn Load_ARB_texture_barrier() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glTextureBarrier =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glTextureBarrier\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glTextureBarrier.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glGetGraphicsResetStatus: Option<unsafe extern "C" fn() -> GLenum> = None;
pub static mut _ptrc_glGetnUniformfv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetnUniformiv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetnUniformuiv: Option<
    unsafe extern "C" fn(GLuint, GLint, GLsizei, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glReadnPixels: Option<
    unsafe extern "C" fn(
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLenum,
        GLenum,
        GLsizei,
        *mut ::core::ffi::c_void,
    ) -> (),
> = None;
unsafe fn Load_KHR_robustness() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glGetGraphicsResetStatus = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn() -> GLenum>,
    >(SDL_GL_GetProcAddress(
        b"glGetGraphicsResetStatus\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetGraphicsResetStatus.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetnUniformfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetnUniformfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetnUniformfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetnUniformiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetnUniformiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetnUniformiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetnUniformuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetnUniformuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetnUniformuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glReadnPixels = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLenum,
                GLenum,
                GLsizei,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glReadnPixels\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glReadnPixels.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
pub static mut _ptrc_glAccum: Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glAlphaFunc: Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glBegin: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glBitmap: Option<
    unsafe extern "C" fn(
        GLsizei,
        GLsizei,
        GLfloat,
        GLfloat,
        GLfloat,
        GLfloat,
        *const GLubyte,
    ) -> (),
> = None;
pub static mut _ptrc_glBlendFunc: Option<unsafe extern "C" fn(GLenum, GLenum) -> ()> = None;
pub static mut _ptrc_glCallList: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glCallLists: Option<
    unsafe extern "C" fn(GLsizei, GLenum, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glClear: Option<unsafe extern "C" fn(GLbitfield) -> ()> = None;
pub static mut _ptrc_glClearAccum: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glClearColor: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glClearDepth: Option<unsafe extern "C" fn(GLdouble) -> ()> = None;
pub static mut _ptrc_glClearIndex: Option<unsafe extern "C" fn(GLfloat) -> ()> = None;
pub static mut _ptrc_glClearStencil: Option<unsafe extern "C" fn(GLint) -> ()> = None;
pub static mut _ptrc_glClipPlane: Option<unsafe extern "C" fn(GLenum, *const GLdouble) -> ()> =
    None;
pub static mut _ptrc_glColor3b: Option<unsafe extern "C" fn(GLbyte, GLbyte, GLbyte) -> ()> = None;
pub static mut _ptrc_glColor3bv: Option<unsafe extern "C" fn(*const GLbyte) -> ()> = None;
pub static mut _ptrc_glColor3d: Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()> =
    None;
pub static mut _ptrc_glColor3dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glColor3f: Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glColor3fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glColor3i: Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()> = None;
pub static mut _ptrc_glColor3iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glColor3s: Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()> =
    None;
pub static mut _ptrc_glColor3sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glColor3ub: Option<unsafe extern "C" fn(GLubyte, GLubyte, GLubyte) -> ()> =
    None;
pub static mut _ptrc_glColor3ubv: Option<unsafe extern "C" fn(*const GLubyte) -> ()> = None;
pub static mut _ptrc_glColor3ui: Option<unsafe extern "C" fn(GLuint, GLuint, GLuint) -> ()> = None;
pub static mut _ptrc_glColor3uiv: Option<unsafe extern "C" fn(*const GLuint) -> ()> = None;
pub static mut _ptrc_glColor3us: Option<unsafe extern "C" fn(GLushort, GLushort, GLushort) -> ()> =
    None;
pub static mut _ptrc_glColor3usv: Option<unsafe extern "C" fn(*const GLushort) -> ()> = None;
pub static mut _ptrc_glColor4b: Option<unsafe extern "C" fn(GLbyte, GLbyte, GLbyte, GLbyte) -> ()> =
    None;
pub static mut _ptrc_glColor4bv: Option<unsafe extern "C" fn(*const GLbyte) -> ()> = None;
pub static mut _ptrc_glColor4d: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glColor4dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glColor4f: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glColor4fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glColor4i: Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()> =
    None;
pub static mut _ptrc_glColor4iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glColor4s: Option<
    unsafe extern "C" fn(GLshort, GLshort, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glColor4sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glColor4ub: Option<
    unsafe extern "C" fn(GLubyte, GLubyte, GLubyte, GLubyte) -> (),
> = None;
pub static mut _ptrc_glColor4ubv: Option<unsafe extern "C" fn(*const GLubyte) -> ()> = None;
pub static mut _ptrc_glColor4ui: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glColor4uiv: Option<unsafe extern "C" fn(*const GLuint) -> ()> = None;
pub static mut _ptrc_glColor4us: Option<
    unsafe extern "C" fn(GLushort, GLushort, GLushort, GLushort) -> (),
> = None;
pub static mut _ptrc_glColor4usv: Option<unsafe extern "C" fn(*const GLushort) -> ()> = None;
pub static mut _ptrc_glColorMask: Option<
    unsafe extern "C" fn(GLboolean, GLboolean, GLboolean, GLboolean) -> (),
> = None;
pub static mut _ptrc_glColorMaterial: Option<unsafe extern "C" fn(GLenum, GLenum) -> ()> = None;
pub static mut _ptrc_glCopyPixels: Option<
    unsafe extern "C" fn(GLint, GLint, GLsizei, GLsizei, GLenum) -> (),
> = None;
pub static mut _ptrc_glCullFace: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glDeleteLists: Option<unsafe extern "C" fn(GLuint, GLsizei) -> ()> = None;
pub static mut _ptrc_glDepthFunc: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glDepthMask: Option<unsafe extern "C" fn(GLboolean) -> ()> = None;
pub static mut _ptrc_glDepthRange: Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()> = None;
pub static mut _ptrc_glDisable: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glDrawBuffer: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glDrawPixels: Option<
    unsafe extern "C" fn(GLsizei, GLsizei, GLenum, GLenum, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glEdgeFlag: Option<unsafe extern "C" fn(GLboolean) -> ()> = None;
pub static mut _ptrc_glEdgeFlagv: Option<unsafe extern "C" fn(*const GLboolean) -> ()> = None;
pub static mut _ptrc_glEnable: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glEnd: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glEndList: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glEvalCoord1d: Option<unsafe extern "C" fn(GLdouble) -> ()> = None;
pub static mut _ptrc_glEvalCoord1dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glEvalCoord1f: Option<unsafe extern "C" fn(GLfloat) -> ()> = None;
pub static mut _ptrc_glEvalCoord1fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glEvalCoord2d: Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()> = None;
pub static mut _ptrc_glEvalCoord2dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glEvalCoord2f: Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()> = None;
pub static mut _ptrc_glEvalCoord2fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glEvalMesh1: Option<unsafe extern "C" fn(GLenum, GLint, GLint) -> ()> = None;
pub static mut _ptrc_glEvalMesh2: Option<
    unsafe extern "C" fn(GLenum, GLint, GLint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glEvalPoint1: Option<unsafe extern "C" fn(GLint) -> ()> = None;
pub static mut _ptrc_glEvalPoint2: Option<unsafe extern "C" fn(GLint, GLint) -> ()> = None;
pub static mut _ptrc_glFeedbackBuffer: Option<
    unsafe extern "C" fn(GLsizei, GLenum, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glFinish: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glFlush: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glFogf: Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glFogfv: Option<unsafe extern "C" fn(GLenum, *const GLfloat) -> ()> = None;
pub static mut _ptrc_glFogi: Option<unsafe extern "C" fn(GLenum, GLint) -> ()> = None;
pub static mut _ptrc_glFogiv: Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()> = None;
pub static mut _ptrc_glFrontFace: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glFrustum: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glGenLists: Option<unsafe extern "C" fn(GLsizei) -> GLuint> = None;
pub static mut _ptrc_glGetBooleanv: Option<unsafe extern "C" fn(GLenum, *mut GLboolean) -> ()> =
    None;
pub static mut _ptrc_glGetClipPlane: Option<unsafe extern "C" fn(GLenum, *mut GLdouble) -> ()> =
    None;
pub static mut _ptrc_glGetDoublev: Option<unsafe extern "C" fn(GLenum, *mut GLdouble) -> ()> = None;
pub static mut _ptrc_glGetError: Option<unsafe extern "C" fn() -> GLenum> = None;
pub static mut _ptrc_glGetFloatv: Option<unsafe extern "C" fn(GLenum, *mut GLfloat) -> ()> = None;
pub static mut _ptrc_glGetIntegerv: Option<unsafe extern "C" fn(GLenum, *mut GLint) -> ()> = None;
pub static mut _ptrc_glGetLightfv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetLightiv: Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()> =
    None;
pub static mut _ptrc_glGetMapdv: Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLdouble) -> ()> =
    None;
pub static mut _ptrc_glGetMapfv: Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> ()> =
    None;
pub static mut _ptrc_glGetMapiv: Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()> =
    None;
pub static mut _ptrc_glGetMaterialfv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetMaterialiv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetPixelMapfv: Option<unsafe extern "C" fn(GLenum, *mut GLfloat) -> ()> =
    None;
pub static mut _ptrc_glGetPixelMapuiv: Option<unsafe extern "C" fn(GLenum, *mut GLuint) -> ()> =
    None;
pub static mut _ptrc_glGetPixelMapusv: Option<unsafe extern "C" fn(GLenum, *mut GLushort) -> ()> =
    None;
pub static mut _ptrc_glGetPolygonStipple: Option<unsafe extern "C" fn(*mut GLubyte) -> ()> = None;
pub static mut _ptrc_glGetString: Option<unsafe extern "C" fn(GLenum) -> *const GLubyte> = None;
pub static mut _ptrc_glGetTexEnvfv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetTexEnviv: Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()> =
    None;
pub static mut _ptrc_glGetTexGendv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLdouble) -> (),
> = None;
pub static mut _ptrc_glGetTexGenfv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetTexGeniv: Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()> =
    None;
pub static mut _ptrc_glGetTexImage: Option<
    unsafe extern "C" fn(GLenum, GLint, GLenum, GLenum, *mut ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glGetTexLevelParameterfv: Option<
    unsafe extern "C" fn(GLenum, GLint, GLenum, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetTexLevelParameteriv: Option<
    unsafe extern "C" fn(GLenum, GLint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetTexParameterfv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetTexParameteriv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glHint: Option<unsafe extern "C" fn(GLenum, GLenum) -> ()> = None;
pub static mut _ptrc_glIndexMask: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glIndexd: Option<unsafe extern "C" fn(GLdouble) -> ()> = None;
pub static mut _ptrc_glIndexdv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glIndexf: Option<unsafe extern "C" fn(GLfloat) -> ()> = None;
pub static mut _ptrc_glIndexfv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glIndexi: Option<unsafe extern "C" fn(GLint) -> ()> = None;
pub static mut _ptrc_glIndexiv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glIndexs: Option<unsafe extern "C" fn(GLshort) -> ()> = None;
pub static mut _ptrc_glIndexsv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glInitNames: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glIsEnabled: Option<unsafe extern "C" fn(GLenum) -> GLboolean> = None;
pub static mut _ptrc_glIsList: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glLightModelf: Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glLightModelfv: Option<unsafe extern "C" fn(GLenum, *const GLfloat) -> ()> =
    None;
pub static mut _ptrc_glLightModeli: Option<unsafe extern "C" fn(GLenum, GLint) -> ()> = None;
pub static mut _ptrc_glLightModeliv: Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glLightf: Option<unsafe extern "C" fn(GLenum, GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glLightfv: Option<unsafe extern "C" fn(GLenum, GLenum, *const GLfloat) -> ()> =
    None;
pub static mut _ptrc_glLighti: Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()> = None;
pub static mut _ptrc_glLightiv: Option<unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glLineStipple: Option<unsafe extern "C" fn(GLint, GLushort) -> ()> = None;
pub static mut _ptrc_glLineWidth: Option<unsafe extern "C" fn(GLfloat) -> ()> = None;
pub static mut _ptrc_glListBase: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glLoadIdentity: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glLoadMatrixd: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glLoadMatrixf: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glLoadName: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glLogicOp: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glMap1d: Option<
    unsafe extern "C" fn(GLenum, GLdouble, GLdouble, GLint, GLint, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glMap1f: Option<
    unsafe extern "C" fn(GLenum, GLfloat, GLfloat, GLint, GLint, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glMap2d: Option<
    unsafe extern "C" fn(
        GLenum,
        GLdouble,
        GLdouble,
        GLint,
        GLint,
        GLdouble,
        GLdouble,
        GLint,
        GLint,
        *const GLdouble,
    ) -> (),
> = None;
pub static mut _ptrc_glMap2f: Option<
    unsafe extern "C" fn(
        GLenum,
        GLfloat,
        GLfloat,
        GLint,
        GLint,
        GLfloat,
        GLfloat,
        GLint,
        GLint,
        *const GLfloat,
    ) -> (),
> = None;
pub static mut _ptrc_glMapGrid1d: Option<unsafe extern "C" fn(GLint, GLdouble, GLdouble) -> ()> =
    None;
pub static mut _ptrc_glMapGrid1f: Option<unsafe extern "C" fn(GLint, GLfloat, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glMapGrid2d: Option<
    unsafe extern "C" fn(GLint, GLdouble, GLdouble, GLint, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glMapGrid2f: Option<
    unsafe extern "C" fn(GLint, GLfloat, GLfloat, GLint, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glMaterialf: Option<unsafe extern "C" fn(GLenum, GLenum, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glMaterialfv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glMateriali: Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()> = None;
pub static mut _ptrc_glMaterialiv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> (),
> = None;
pub static mut _ptrc_glMatrixMode: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glMultMatrixd: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glMultMatrixf: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glNewList: Option<unsafe extern "C" fn(GLuint, GLenum) -> ()> = None;
pub static mut _ptrc_glNormal3b: Option<unsafe extern "C" fn(GLbyte, GLbyte, GLbyte) -> ()> = None;
pub static mut _ptrc_glNormal3bv: Option<unsafe extern "C" fn(*const GLbyte) -> ()> = None;
pub static mut _ptrc_glNormal3d: Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()> =
    None;
pub static mut _ptrc_glNormal3dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glNormal3f: Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glNormal3fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glNormal3i: Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()> = None;
pub static mut _ptrc_glNormal3iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glNormal3s: Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()> =
    None;
pub static mut _ptrc_glNormal3sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glOrtho: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glPassThrough: Option<unsafe extern "C" fn(GLfloat) -> ()> = None;
pub static mut _ptrc_glPixelMapfv: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glPixelMapuiv: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glPixelMapusv: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *const GLushort) -> (),
> = None;
pub static mut _ptrc_glPixelStoref: Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glPixelStorei: Option<unsafe extern "C" fn(GLenum, GLint) -> ()> = None;
pub static mut _ptrc_glPixelTransferf: Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glPixelTransferi: Option<unsafe extern "C" fn(GLenum, GLint) -> ()> = None;
pub static mut _ptrc_glPixelZoom: Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()> = None;
pub static mut _ptrc_glPointSize: Option<unsafe extern "C" fn(GLfloat) -> ()> = None;
pub static mut _ptrc_glPolygonMode: Option<unsafe extern "C" fn(GLenum, GLenum) -> ()> = None;
pub static mut _ptrc_glPolygonStipple: Option<unsafe extern "C" fn(*const GLubyte) -> ()> = None;
pub static mut _ptrc_glPopAttrib: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glPopMatrix: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glPopName: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glPushAttrib: Option<unsafe extern "C" fn(GLbitfield) -> ()> = None;
pub static mut _ptrc_glPushMatrix: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glPushName: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glRasterPos2d: Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()> = None;
pub static mut _ptrc_glRasterPos2dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glRasterPos2f: Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()> = None;
pub static mut _ptrc_glRasterPos2fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glRasterPos2i: Option<unsafe extern "C" fn(GLint, GLint) -> ()> = None;
pub static mut _ptrc_glRasterPos2iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glRasterPos2s: Option<unsafe extern "C" fn(GLshort, GLshort) -> ()> = None;
pub static mut _ptrc_glRasterPos2sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glRasterPos3d: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glRasterPos3dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glRasterPos3f: Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glRasterPos3fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glRasterPos3i: Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()> = None;
pub static mut _ptrc_glRasterPos3iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glRasterPos3s: Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()> =
    None;
pub static mut _ptrc_glRasterPos3sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glRasterPos4d: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glRasterPos4dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glRasterPos4f: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glRasterPos4fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glRasterPos4i: Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()> =
    None;
pub static mut _ptrc_glRasterPos4iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glRasterPos4s: Option<
    unsafe extern "C" fn(GLshort, GLshort, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glRasterPos4sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glReadBuffer: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glReadPixels: Option<
    unsafe extern "C" fn(
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLenum,
        GLenum,
        *mut ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glRectd: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glRectdv: Option<
    unsafe extern "C" fn(*const GLdouble, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glRectf: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glRectfv: Option<unsafe extern "C" fn(*const GLfloat, *const GLfloat) -> ()> =
    None;
pub static mut _ptrc_glRecti: Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()> = None;
pub static mut _ptrc_glRectiv: Option<unsafe extern "C" fn(*const GLint, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glRects: Option<
    unsafe extern "C" fn(GLshort, GLshort, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glRectsv: Option<unsafe extern "C" fn(*const GLshort, *const GLshort) -> ()> =
    None;
pub static mut _ptrc_glRenderMode: Option<unsafe extern "C" fn(GLenum) -> GLint> = None;
pub static mut _ptrc_glRotated: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glRotatef: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glScaled: Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()> =
    None;
pub static mut _ptrc_glScalef: Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()> = None;
pub static mut _ptrc_glScissor: Option<unsafe extern "C" fn(GLint, GLint, GLsizei, GLsizei) -> ()> =
    None;
pub static mut _ptrc_glSelectBuffer: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()> =
    None;
pub static mut _ptrc_glShadeModel: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glStencilFunc: Option<unsafe extern "C" fn(GLenum, GLint, GLuint) -> ()> =
    None;
pub static mut _ptrc_glStencilMask: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glStencilOp: Option<unsafe extern "C" fn(GLenum, GLenum, GLenum) -> ()> = None;
pub static mut _ptrc_glTexCoord1d: Option<unsafe extern "C" fn(GLdouble) -> ()> = None;
pub static mut _ptrc_glTexCoord1dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glTexCoord1f: Option<unsafe extern "C" fn(GLfloat) -> ()> = None;
pub static mut _ptrc_glTexCoord1fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glTexCoord1i: Option<unsafe extern "C" fn(GLint) -> ()> = None;
pub static mut _ptrc_glTexCoord1iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glTexCoord1s: Option<unsafe extern "C" fn(GLshort) -> ()> = None;
pub static mut _ptrc_glTexCoord1sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glTexCoord2d: Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()> = None;
pub static mut _ptrc_glTexCoord2dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glTexCoord2f: Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()> = None;
pub static mut _ptrc_glTexCoord2fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glTexCoord2i: Option<unsafe extern "C" fn(GLint, GLint) -> ()> = None;
pub static mut _ptrc_glTexCoord2iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glTexCoord2s: Option<unsafe extern "C" fn(GLshort, GLshort) -> ()> = None;
pub static mut _ptrc_glTexCoord2sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glTexCoord3d: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glTexCoord3dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glTexCoord3f: Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glTexCoord3fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glTexCoord3i: Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()> = None;
pub static mut _ptrc_glTexCoord3iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glTexCoord3s: Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()> =
    None;
pub static mut _ptrc_glTexCoord3sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glTexCoord4d: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glTexCoord4dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glTexCoord4f: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glTexCoord4fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glTexCoord4i: Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()> =
    None;
pub static mut _ptrc_glTexCoord4iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glTexCoord4s: Option<
    unsafe extern "C" fn(GLshort, GLshort, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glTexCoord4sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glTexEnvf: Option<unsafe extern "C" fn(GLenum, GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glTexEnvfv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glTexEnvi: Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()> = None;
pub static mut _ptrc_glTexEnviv: Option<unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glTexGend: Option<unsafe extern "C" fn(GLenum, GLenum, GLdouble) -> ()> = None;
pub static mut _ptrc_glTexGendv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glTexGenf: Option<unsafe extern "C" fn(GLenum, GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glTexGenfv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glTexGeni: Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()> = None;
pub static mut _ptrc_glTexGeniv: Option<unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glTexImage1D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLint,
        GLsizei,
        GLint,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glTexImage2D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLint,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glTexParameterf: Option<unsafe extern "C" fn(GLenum, GLenum, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glTexParameterfv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glTexParameteri: Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()> =
    None;
pub static mut _ptrc_glTexParameteriv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> (),
> = None;
pub static mut _ptrc_glTranslated: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glTranslatef: Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glVertex2d: Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()> = None;
pub static mut _ptrc_glVertex2dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glVertex2f: Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()> = None;
pub static mut _ptrc_glVertex2fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glVertex2i: Option<unsafe extern "C" fn(GLint, GLint) -> ()> = None;
pub static mut _ptrc_glVertex2iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glVertex2s: Option<unsafe extern "C" fn(GLshort, GLshort) -> ()> = None;
pub static mut _ptrc_glVertex2sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glVertex3d: Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()> =
    None;
pub static mut _ptrc_glVertex3dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glVertex3f: Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glVertex3fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glVertex3i: Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()> = None;
pub static mut _ptrc_glVertex3iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glVertex3s: Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()> =
    None;
pub static mut _ptrc_glVertex3sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glVertex4d: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glVertex4dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glVertex4f: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glVertex4fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glVertex4i: Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()> =
    None;
pub static mut _ptrc_glVertex4iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glVertex4s: Option<
    unsafe extern "C" fn(GLshort, GLshort, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glVertex4sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glViewport: Option<
    unsafe extern "C" fn(GLint, GLint, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glAreTexturesResident: Option<
    unsafe extern "C" fn(GLsizei, *const GLuint, *mut GLboolean) -> GLboolean,
> = None;
pub static mut _ptrc_glArrayElement: Option<unsafe extern "C" fn(GLint) -> ()> = None;
pub static mut _ptrc_glBindTexture: Option<unsafe extern "C" fn(GLenum, GLuint) -> ()> = None;
pub static mut _ptrc_glColorPointer: Option<
    unsafe extern "C" fn(GLint, GLenum, GLsizei, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glCopyTexImage1D: Option<
    unsafe extern "C" fn(GLenum, GLint, GLenum, GLint, GLint, GLsizei, GLint) -> (),
> = None;
pub static mut _ptrc_glCopyTexImage2D: Option<
    unsafe extern "C" fn(GLenum, GLint, GLenum, GLint, GLint, GLsizei, GLsizei, GLint) -> (),
> = None;
pub static mut _ptrc_glCopyTexSubImage1D: Option<
    unsafe extern "C" fn(GLenum, GLint, GLint, GLint, GLint, GLsizei) -> (),
> = None;
pub static mut _ptrc_glCopyTexSubImage2D: Option<
    unsafe extern "C" fn(GLenum, GLint, GLint, GLint, GLint, GLint, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glDeleteTextures: Option<unsafe extern "C" fn(GLsizei, *const GLuint) -> ()> =
    None;
pub static mut _ptrc_glDisableClientState: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glDrawArrays: Option<unsafe extern "C" fn(GLenum, GLint, GLsizei) -> ()> =
    None;
pub static mut _ptrc_glDrawElements: Option<
    unsafe extern "C" fn(GLenum, GLsizei, GLenum, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glEdgeFlagPointer: Option<
    unsafe extern "C" fn(GLsizei, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glEnableClientState: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glGenTextures: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()> = None;
pub static mut _ptrc_glIndexPointer: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glIndexub: Option<unsafe extern "C" fn(GLubyte) -> ()> = None;
pub static mut _ptrc_glIndexubv: Option<unsafe extern "C" fn(*const GLubyte) -> ()> = None;
pub static mut _ptrc_glInterleavedArrays: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glIsTexture: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glNormalPointer: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glPolygonOffset: Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()> = None;
pub static mut _ptrc_glPopClientAttrib: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glPrioritizeTextures: Option<
    unsafe extern "C" fn(GLsizei, *const GLuint, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glPushClientAttrib: Option<unsafe extern "C" fn(GLbitfield) -> ()> = None;
pub static mut _ptrc_glTexCoordPointer: Option<
    unsafe extern "C" fn(GLint, GLenum, GLsizei, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glTexSubImage1D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLint,
        GLsizei,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glTexSubImage2D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glVertexPointer: Option<
    unsafe extern "C" fn(GLint, GLenum, GLsizei, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glCopyTexSubImage3D: Option<
    unsafe extern "C" fn(GLenum, GLint, GLint, GLint, GLint, GLint, GLint, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glDrawRangeElements: Option<
    unsafe extern "C" fn(GLenum, GLuint, GLuint, GLsizei, GLenum, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glTexImage3D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLsizei,
        GLint,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glTexSubImage3D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLsizei,
        GLenum,
        GLenum,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glActiveTexture: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glClientActiveTexture: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glCompressedTexImage1D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLenum,
        GLsizei,
        GLint,
        GLsizei,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glCompressedTexImage2D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLenum,
        GLsizei,
        GLsizei,
        GLint,
        GLsizei,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glCompressedTexImage3D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLenum,
        GLsizei,
        GLsizei,
        GLsizei,
        GLint,
        GLsizei,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glCompressedTexSubImage1D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLint,
        GLsizei,
        GLenum,
        GLsizei,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glCompressedTexSubImage2D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLenum,
        GLsizei,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glCompressedTexSubImage3D: Option<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLint,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLsizei,
        GLenum,
        GLsizei,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glGetCompressedTexImage: Option<
    unsafe extern "C" fn(GLenum, GLint, *mut ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glLoadTransposeMatrixd: Option<unsafe extern "C" fn(*const GLdouble) -> ()> =
    None;
pub static mut _ptrc_glLoadTransposeMatrixf: Option<unsafe extern "C" fn(*const GLfloat) -> ()> =
    None;
pub static mut _ptrc_glMultTransposeMatrixd: Option<unsafe extern "C" fn(*const GLdouble) -> ()> =
    None;
pub static mut _ptrc_glMultTransposeMatrixf: Option<unsafe extern "C" fn(*const GLfloat) -> ()> =
    None;
pub static mut _ptrc_glMultiTexCoord1d: Option<unsafe extern "C" fn(GLenum, GLdouble) -> ()> = None;
pub static mut _ptrc_glMultiTexCoord1dv: Option<
    unsafe extern "C" fn(GLenum, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord1f: Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glMultiTexCoord1fv: Option<
    unsafe extern "C" fn(GLenum, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord1i: Option<unsafe extern "C" fn(GLenum, GLint) -> ()> = None;
pub static mut _ptrc_glMultiTexCoord1iv: Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glMultiTexCoord1s: Option<unsafe extern "C" fn(GLenum, GLshort) -> ()> = None;
pub static mut _ptrc_glMultiTexCoord1sv: Option<
    unsafe extern "C" fn(GLenum, *const GLshort) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord2d: Option<
    unsafe extern "C" fn(GLenum, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord2dv: Option<
    unsafe extern "C" fn(GLenum, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord2f: Option<
    unsafe extern "C" fn(GLenum, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord2fv: Option<
    unsafe extern "C" fn(GLenum, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord2i: Option<unsafe extern "C" fn(GLenum, GLint, GLint) -> ()> =
    None;
pub static mut _ptrc_glMultiTexCoord2iv: Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glMultiTexCoord2s: Option<
    unsafe extern "C" fn(GLenum, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord2sv: Option<
    unsafe extern "C" fn(GLenum, *const GLshort) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord3d: Option<
    unsafe extern "C" fn(GLenum, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord3dv: Option<
    unsafe extern "C" fn(GLenum, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord3f: Option<
    unsafe extern "C" fn(GLenum, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord3fv: Option<
    unsafe extern "C" fn(GLenum, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord3i: Option<
    unsafe extern "C" fn(GLenum, GLint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord3iv: Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glMultiTexCoord3s: Option<
    unsafe extern "C" fn(GLenum, GLshort, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord3sv: Option<
    unsafe extern "C" fn(GLenum, *const GLshort) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord4d: Option<
    unsafe extern "C" fn(GLenum, GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord4dv: Option<
    unsafe extern "C" fn(GLenum, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord4f: Option<
    unsafe extern "C" fn(GLenum, GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord4fv: Option<
    unsafe extern "C" fn(GLenum, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord4i: Option<
    unsafe extern "C" fn(GLenum, GLint, GLint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord4iv: Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glMultiTexCoord4s: Option<
    unsafe extern "C" fn(GLenum, GLshort, GLshort, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glMultiTexCoord4sv: Option<
    unsafe extern "C" fn(GLenum, *const GLshort) -> (),
> = None;
pub static mut _ptrc_glSampleCoverage: Option<unsafe extern "C" fn(GLfloat, GLboolean) -> ()> =
    None;
pub static mut _ptrc_glBlendColor: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glBlendEquation: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glBlendFuncSeparate: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLenum) -> (),
> = None;
pub static mut _ptrc_glFogCoordPointer: Option<
    unsafe extern "C" fn(GLenum, GLsizei, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glFogCoordd: Option<unsafe extern "C" fn(GLdouble) -> ()> = None;
pub static mut _ptrc_glFogCoorddv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glFogCoordf: Option<unsafe extern "C" fn(GLfloat) -> ()> = None;
pub static mut _ptrc_glFogCoordfv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glMultiDrawArrays: Option<
    unsafe extern "C" fn(GLenum, *const GLint, *const GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glMultiDrawElements: Option<
    unsafe extern "C" fn(
        GLenum,
        *const GLsizei,
        GLenum,
        *const *const ::core::ffi::c_void,
        GLsizei,
    ) -> (),
> = None;
pub static mut _ptrc_glPointParameterf: Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()> = None;
pub static mut _ptrc_glPointParameterfv: Option<
    unsafe extern "C" fn(GLenum, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glPointParameteri: Option<unsafe extern "C" fn(GLenum, GLint) -> ()> = None;
pub static mut _ptrc_glPointParameteriv: Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glSecondaryColor3b: Option<
    unsafe extern "C" fn(GLbyte, GLbyte, GLbyte) -> (),
> = None;
pub static mut _ptrc_glSecondaryColor3bv: Option<unsafe extern "C" fn(*const GLbyte) -> ()> = None;
pub static mut _ptrc_glSecondaryColor3d: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glSecondaryColor3dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> =
    None;
pub static mut _ptrc_glSecondaryColor3f: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glSecondaryColor3fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glSecondaryColor3i: Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()> =
    None;
pub static mut _ptrc_glSecondaryColor3iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glSecondaryColor3s: Option<
    unsafe extern "C" fn(GLshort, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glSecondaryColor3sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glSecondaryColor3ub: Option<
    unsafe extern "C" fn(GLubyte, GLubyte, GLubyte) -> (),
> = None;
pub static mut _ptrc_glSecondaryColor3ubv: Option<unsafe extern "C" fn(*const GLubyte) -> ()> =
    None;
pub static mut _ptrc_glSecondaryColor3ui: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glSecondaryColor3uiv: Option<unsafe extern "C" fn(*const GLuint) -> ()> = None;
pub static mut _ptrc_glSecondaryColor3us: Option<
    unsafe extern "C" fn(GLushort, GLushort, GLushort) -> (),
> = None;
pub static mut _ptrc_glSecondaryColor3usv: Option<unsafe extern "C" fn(*const GLushort) -> ()> =
    None;
pub static mut _ptrc_glSecondaryColorPointer: Option<
    unsafe extern "C" fn(GLint, GLenum, GLsizei, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glWindowPos2d: Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()> = None;
pub static mut _ptrc_glWindowPos2dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glWindowPos2f: Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()> = None;
pub static mut _ptrc_glWindowPos2fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glWindowPos2i: Option<unsafe extern "C" fn(GLint, GLint) -> ()> = None;
pub static mut _ptrc_glWindowPos2iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glWindowPos2s: Option<unsafe extern "C" fn(GLshort, GLshort) -> ()> = None;
pub static mut _ptrc_glWindowPos2sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glWindowPos3d: Option<
    unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glWindowPos3dv: Option<unsafe extern "C" fn(*const GLdouble) -> ()> = None;
pub static mut _ptrc_glWindowPos3f: Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glWindowPos3fv: Option<unsafe extern "C" fn(*const GLfloat) -> ()> = None;
pub static mut _ptrc_glWindowPos3i: Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()> = None;
pub static mut _ptrc_glWindowPos3iv: Option<unsafe extern "C" fn(*const GLint) -> ()> = None;
pub static mut _ptrc_glWindowPos3s: Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()> =
    None;
pub static mut _ptrc_glWindowPos3sv: Option<unsafe extern "C" fn(*const GLshort) -> ()> = None;
pub static mut _ptrc_glBeginQuery: Option<unsafe extern "C" fn(GLenum, GLuint) -> ()> = None;
pub static mut _ptrc_glBindBuffer: Option<unsafe extern "C" fn(GLenum, GLuint) -> ()> = None;
pub static mut _ptrc_glBufferData: Option<
    unsafe extern "C" fn(GLenum, GLsizeiptr, *const ::core::ffi::c_void, GLenum) -> (),
> = None;
pub static mut _ptrc_glBufferSubData: Option<
    unsafe extern "C" fn(GLenum, GLintptr, GLsizeiptr, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glDeleteBuffers: Option<unsafe extern "C" fn(GLsizei, *const GLuint) -> ()> =
    None;
pub static mut _ptrc_glDeleteQueries: Option<unsafe extern "C" fn(GLsizei, *const GLuint) -> ()> =
    None;
pub static mut _ptrc_glEndQuery: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glGenBuffers: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()> = None;
pub static mut _ptrc_glGenQueries: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()> = None;
pub static mut _ptrc_glGetBufferParameteriv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetBufferPointerv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut *mut ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glGetBufferSubData: Option<
    unsafe extern "C" fn(GLenum, GLintptr, GLsizeiptr, *mut ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glGetQueryObjectiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetQueryObjectuiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glGetQueryiv: Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()> =
    None;
pub static mut _ptrc_glIsBuffer: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glIsQuery: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glMapBuffer: Option<
    unsafe extern "C" fn(GLenum, GLenum) -> *mut ::core::ffi::c_void,
> = None;
pub static mut _ptrc_glUnmapBuffer: Option<unsafe extern "C" fn(GLenum) -> GLboolean> = None;
pub static mut _ptrc_glAttachShader: Option<unsafe extern "C" fn(GLuint, GLuint) -> ()> = None;
pub static mut _ptrc_glBindAttribLocation: Option<
    unsafe extern "C" fn(GLuint, GLuint, *const GLchar) -> (),
> = None;
pub static mut _ptrc_glBlendEquationSeparate: Option<unsafe extern "C" fn(GLenum, GLenum) -> ()> =
    None;
pub static mut _ptrc_glCompileShader: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glCreateProgram: Option<unsafe extern "C" fn() -> GLuint> = None;
pub static mut _ptrc_glCreateShader: Option<unsafe extern "C" fn(GLenum) -> GLuint> = None;
pub static mut _ptrc_glDeleteProgram: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glDeleteShader: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glDetachShader: Option<unsafe extern "C" fn(GLuint, GLuint) -> ()> = None;
pub static mut _ptrc_glDisableVertexAttribArray: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glDrawBuffers: Option<unsafe extern "C" fn(GLsizei, *const GLenum) -> ()> =
    None;
pub static mut _ptrc_glEnableVertexAttribArray: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glGetActiveAttrib: Option<
    unsafe extern "C" fn(
        GLuint,
        GLuint,
        GLsizei,
        *mut GLsizei,
        *mut GLint,
        *mut GLenum,
        *mut GLchar,
    ) -> (),
> = None;
pub static mut _ptrc_glGetActiveUniform: Option<
    unsafe extern "C" fn(
        GLuint,
        GLuint,
        GLsizei,
        *mut GLsizei,
        *mut GLint,
        *mut GLenum,
        *mut GLchar,
    ) -> (),
> = None;
pub static mut _ptrc_glGetAttachedShaders: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glGetAttribLocation: Option<
    unsafe extern "C" fn(GLuint, *const GLchar) -> GLint,
> = None;
pub static mut _ptrc_glGetProgramInfoLog: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
> = None;
pub static mut _ptrc_glGetProgramiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetShaderInfoLog: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
> = None;
pub static mut _ptrc_glGetShaderSource: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
> = None;
pub static mut _ptrc_glGetShaderiv: Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()> =
    None;
pub static mut _ptrc_glGetUniformLocation: Option<
    unsafe extern "C" fn(GLuint, *const GLchar) -> GLint,
> = None;
pub static mut _ptrc_glGetUniformfv: Option<
    unsafe extern "C" fn(GLuint, GLint, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetUniformiv: Option<unsafe extern "C" fn(GLuint, GLint, *mut GLint) -> ()> =
    None;
pub static mut _ptrc_glGetVertexAttribPointerv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut *mut ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glGetVertexAttribdv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLdouble) -> (),
> = None;
pub static mut _ptrc_glGetVertexAttribfv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLfloat) -> (),
> = None;
pub static mut _ptrc_glGetVertexAttribiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glIsProgram: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glIsShader: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glLinkProgram: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glShaderSource: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const *const GLchar, *const GLint) -> (),
> = None;
pub static mut _ptrc_glStencilFuncSeparate: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLint, GLuint) -> (),
> = None;
pub static mut _ptrc_glStencilMaskSeparate: Option<unsafe extern "C" fn(GLenum, GLuint) -> ()> =
    None;
pub static mut _ptrc_glStencilOpSeparate: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLenum) -> (),
> = None;
pub static mut _ptrc_glUniform1f: Option<unsafe extern "C" fn(GLint, GLfloat) -> ()> = None;
pub static mut _ptrc_glUniform1fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniform1i: Option<unsafe extern "C" fn(GLint, GLint) -> ()> = None;
pub static mut _ptrc_glUniform1iv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLint) -> (),
> = None;
pub static mut _ptrc_glUniform2f: Option<unsafe extern "C" fn(GLint, GLfloat, GLfloat) -> ()> =
    None;
pub static mut _ptrc_glUniform2fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniform2i: Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()> = None;
pub static mut _ptrc_glUniform2iv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLint) -> (),
> = None;
pub static mut _ptrc_glUniform3f: Option<
    unsafe extern "C" fn(GLint, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniform3fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniform3i: Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()> =
    None;
pub static mut _ptrc_glUniform3iv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLint) -> (),
> = None;
pub static mut _ptrc_glUniform4f: Option<
    unsafe extern "C" fn(GLint, GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniform4fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniform4i: Option<
    unsafe extern "C" fn(GLint, GLint, GLint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glUniform4iv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLint) -> (),
> = None;
pub static mut _ptrc_glUniformMatrix2fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniformMatrix3fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniformMatrix4fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUseProgram: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glValidateProgram: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glVertexAttrib1d: Option<unsafe extern "C" fn(GLuint, GLdouble) -> ()> = None;
pub static mut _ptrc_glVertexAttrib1dv: Option<
    unsafe extern "C" fn(GLuint, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib1f: Option<unsafe extern "C" fn(GLuint, GLfloat) -> ()> = None;
pub static mut _ptrc_glVertexAttrib1fv: Option<unsafe extern "C" fn(GLuint, *const GLfloat) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib1s: Option<unsafe extern "C" fn(GLuint, GLshort) -> ()> = None;
pub static mut _ptrc_glVertexAttrib1sv: Option<unsafe extern "C" fn(GLuint, *const GLshort) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib2d: Option<
    unsafe extern "C" fn(GLuint, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib2dv: Option<
    unsafe extern "C" fn(GLuint, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib2f: Option<
    unsafe extern "C" fn(GLuint, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib2fv: Option<unsafe extern "C" fn(GLuint, *const GLfloat) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib2s: Option<
    unsafe extern "C" fn(GLuint, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib2sv: Option<unsafe extern "C" fn(GLuint, *const GLshort) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib3d: Option<
    unsafe extern "C" fn(GLuint, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib3dv: Option<
    unsafe extern "C" fn(GLuint, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib3f: Option<
    unsafe extern "C" fn(GLuint, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib3fv: Option<unsafe extern "C" fn(GLuint, *const GLfloat) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib3s: Option<
    unsafe extern "C" fn(GLuint, GLshort, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib3sv: Option<unsafe extern "C" fn(GLuint, *const GLshort) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib4Nbv: Option<unsafe extern "C" fn(GLuint, *const GLbyte) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib4Niv: Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib4Nsv: Option<
    unsafe extern "C" fn(GLuint, *const GLshort) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib4Nub: Option<
    unsafe extern "C" fn(GLuint, GLubyte, GLubyte, GLubyte, GLubyte) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib4Nubv: Option<
    unsafe extern "C" fn(GLuint, *const GLubyte) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib4Nuiv: Option<
    unsafe extern "C" fn(GLuint, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib4Nusv: Option<
    unsafe extern "C" fn(GLuint, *const GLushort) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib4bv: Option<unsafe extern "C" fn(GLuint, *const GLbyte) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib4d: Option<
    unsafe extern "C" fn(GLuint, GLdouble, GLdouble, GLdouble, GLdouble) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib4dv: Option<
    unsafe extern "C" fn(GLuint, *const GLdouble) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib4f: Option<
    unsafe extern "C" fn(GLuint, GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib4fv: Option<unsafe extern "C" fn(GLuint, *const GLfloat) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib4iv: Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib4s: Option<
    unsafe extern "C" fn(GLuint, GLshort, GLshort, GLshort, GLshort) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib4sv: Option<unsafe extern "C" fn(GLuint, *const GLshort) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib4ubv: Option<
    unsafe extern "C" fn(GLuint, *const GLubyte) -> (),
> = None;
pub static mut _ptrc_glVertexAttrib4uiv: Option<unsafe extern "C" fn(GLuint, *const GLuint) -> ()> =
    None;
pub static mut _ptrc_glVertexAttrib4usv: Option<
    unsafe extern "C" fn(GLuint, *const GLushort) -> (),
> = None;
pub static mut _ptrc_glVertexAttribPointer: Option<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLenum,
        GLboolean,
        GLsizei,
        *const ::core::ffi::c_void,
    ) -> (),
> = None;
pub static mut _ptrc_glUniformMatrix2x3fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniformMatrix2x4fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniformMatrix3x2fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniformMatrix3x4fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniformMatrix4x2fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glUniformMatrix4x3fv: Option<
    unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glBeginConditionalRender: Option<unsafe extern "C" fn(GLuint, GLenum) -> ()> =
    None;
pub static mut _ptrc_glBeginTransformFeedback: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glBindBufferBase: Option<unsafe extern "C" fn(GLenum, GLuint, GLuint) -> ()> =
    None;
pub static mut _ptrc_glBindBufferRange: Option<
    unsafe extern "C" fn(GLenum, GLuint, GLuint, GLintptr, GLsizeiptr) -> (),
> = None;
pub static mut _ptrc_glBindFragDataLocation: Option<
    unsafe extern "C" fn(GLuint, GLuint, *const GLchar) -> (),
> = None;
pub static mut _ptrc_glBindFramebuffer: Option<unsafe extern "C" fn(GLenum, GLuint) -> ()> = None;
pub static mut _ptrc_glBindRenderbuffer: Option<unsafe extern "C" fn(GLenum, GLuint) -> ()> = None;
pub static mut _ptrc_glBindVertexArray: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glBlitFramebuffer: Option<
    unsafe extern "C" fn(
        GLint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLint,
        GLbitfield,
        GLenum,
    ) -> (),
> = None;
pub static mut _ptrc_glCheckFramebufferStatus: Option<unsafe extern "C" fn(GLenum) -> GLenum> =
    None;
pub static mut _ptrc_glClampColor: Option<unsafe extern "C" fn(GLenum, GLenum) -> ()> = None;
pub static mut _ptrc_glClearBufferfi: Option<
    unsafe extern "C" fn(GLenum, GLint, GLfloat, GLint) -> (),
> = None;
pub static mut _ptrc_glClearBufferfv: Option<
    unsafe extern "C" fn(GLenum, GLint, *const GLfloat) -> (),
> = None;
pub static mut _ptrc_glClearBufferiv: Option<
    unsafe extern "C" fn(GLenum, GLint, *const GLint) -> (),
> = None;
pub static mut _ptrc_glClearBufferuiv: Option<
    unsafe extern "C" fn(GLenum, GLint, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glColorMaski: Option<
    unsafe extern "C" fn(GLuint, GLboolean, GLboolean, GLboolean, GLboolean) -> (),
> = None;
pub static mut _ptrc_glDeleteFramebuffers: Option<
    unsafe extern "C" fn(GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glDeleteRenderbuffers: Option<
    unsafe extern "C" fn(GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glDeleteVertexArrays: Option<
    unsafe extern "C" fn(GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glDisablei: Option<unsafe extern "C" fn(GLenum, GLuint) -> ()> = None;
pub static mut _ptrc_glEnablei: Option<unsafe extern "C" fn(GLenum, GLuint) -> ()> = None;
pub static mut _ptrc_glEndConditionalRender: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glEndTransformFeedback: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut _ptrc_glFlushMappedBufferRange: Option<
    unsafe extern "C" fn(GLenum, GLintptr, GLsizeiptr) -> (),
> = None;
pub static mut _ptrc_glFramebufferRenderbuffer: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLuint) -> (),
> = None;
pub static mut _ptrc_glFramebufferTexture1D: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLuint, GLint) -> (),
> = None;
pub static mut _ptrc_glFramebufferTexture2D: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLuint, GLint) -> (),
> = None;
pub static mut _ptrc_glFramebufferTexture3D: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLuint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glFramebufferTextureLayer: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLuint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glGenFramebuffers: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()> =
    None;
pub static mut _ptrc_glGenRenderbuffers: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()> =
    None;
pub static mut _ptrc_glGenVertexArrays: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()> =
    None;
pub static mut _ptrc_glGenerateMipmap: Option<unsafe extern "C" fn(GLenum) -> ()> = None;
pub static mut _ptrc_glGetBooleani_v: Option<
    unsafe extern "C" fn(GLenum, GLuint, *mut GLboolean) -> (),
> = None;
pub static mut _ptrc_glGetFragDataLocation: Option<
    unsafe extern "C" fn(GLuint, *const GLchar) -> GLint,
> = None;
pub static mut _ptrc_glGetFramebufferAttachmentParameteriv: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetIntegeri_v: Option<
    unsafe extern "C" fn(GLenum, GLuint, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetRenderbufferParameteriv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetStringi: Option<unsafe extern "C" fn(GLenum, GLuint) -> *const GLubyte> =
    None;
pub static mut _ptrc_glGetTexParameterIiv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetTexParameterIuiv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glGetTransformFeedbackVarying: Option<
    unsafe extern "C" fn(
        GLuint,
        GLuint,
        GLsizei,
        *mut GLsizei,
        *mut GLsizei,
        *mut GLenum,
        *mut GLchar,
    ) -> (),
> = None;
pub static mut _ptrc_glGetUniformuiv: Option<
    unsafe extern "C" fn(GLuint, GLint, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glGetVertexAttribIiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetVertexAttribIuiv: Option<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glIsEnabledi: Option<unsafe extern "C" fn(GLenum, GLuint) -> GLboolean> = None;
pub static mut _ptrc_glIsFramebuffer: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glIsRenderbuffer: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glIsVertexArray: Option<unsafe extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut _ptrc_glMapBufferRange: Option<
    unsafe extern "C" fn(GLenum, GLintptr, GLsizeiptr, GLbitfield) -> *mut ::core::ffi::c_void,
> = None;
pub static mut _ptrc_glRenderbufferStorage: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glRenderbufferStorageMultisample: Option<
    unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glTexParameterIiv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> (),
> = None;
pub static mut _ptrc_glTexParameterIuiv: Option<
    unsafe extern "C" fn(GLenum, GLenum, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glTransformFeedbackVaryings: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const *const GLchar, GLenum) -> (),
> = None;
pub static mut _ptrc_glUniform1ui: Option<unsafe extern "C" fn(GLint, GLuint) -> ()> = None;
pub static mut _ptrc_glUniform1uiv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glUniform2ui: Option<unsafe extern "C" fn(GLint, GLuint, GLuint) -> ()> = None;
pub static mut _ptrc_glUniform2uiv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glUniform3ui: Option<
    unsafe extern "C" fn(GLint, GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glUniform3uiv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glUniform4ui: Option<
    unsafe extern "C" fn(GLint, GLuint, GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glUniform4uiv: Option<
    unsafe extern "C" fn(GLint, GLsizei, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI1i: Option<unsafe extern "C" fn(GLuint, GLint) -> ()> = None;
pub static mut _ptrc_glVertexAttribI1iv: Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glVertexAttribI1ui: Option<unsafe extern "C" fn(GLuint, GLuint) -> ()> = None;
pub static mut _ptrc_glVertexAttribI1uiv: Option<
    unsafe extern "C" fn(GLuint, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI2i: Option<unsafe extern "C" fn(GLuint, GLint, GLint) -> ()> =
    None;
pub static mut _ptrc_glVertexAttribI2iv: Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glVertexAttribI2ui: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI2uiv: Option<
    unsafe extern "C" fn(GLuint, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI3i: Option<
    unsafe extern "C" fn(GLuint, GLint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI3iv: Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glVertexAttribI3ui: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI3uiv: Option<
    unsafe extern "C" fn(GLuint, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI4bv: Option<unsafe extern "C" fn(GLuint, *const GLbyte) -> ()> =
    None;
pub static mut _ptrc_glVertexAttribI4i: Option<
    unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI4iv: Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()> =
    None;
pub static mut _ptrc_glVertexAttribI4sv: Option<
    unsafe extern "C" fn(GLuint, *const GLshort) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI4ubv: Option<
    unsafe extern "C" fn(GLuint, *const GLubyte) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI4ui: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint, GLuint, GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI4uiv: Option<
    unsafe extern "C" fn(GLuint, *const GLuint) -> (),
> = None;
pub static mut _ptrc_glVertexAttribI4usv: Option<
    unsafe extern "C" fn(GLuint, *const GLushort) -> (),
> = None;
pub static mut _ptrc_glVertexAttribIPointer: Option<
    unsafe extern "C" fn(GLuint, GLint, GLenum, GLsizei, *const ::core::ffi::c_void) -> (),
> = None;
pub static mut _ptrc_glCopyBufferSubData: Option<
    unsafe extern "C" fn(GLenum, GLenum, GLintptr, GLintptr, GLsizeiptr) -> (),
> = None;
pub static mut _ptrc_glDrawArraysInstanced: Option<
    unsafe extern "C" fn(GLenum, GLint, GLsizei, GLsizei) -> (),
> = None;
pub static mut _ptrc_glDrawElementsInstanced: Option<
    unsafe extern "C" fn(GLenum, GLsizei, GLenum, *const ::core::ffi::c_void, GLsizei) -> (),
> = None;
pub static mut _ptrc_glGetActiveUniformBlockName: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
> = None;
pub static mut _ptrc_glGetActiveUniformBlockiv: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetActiveUniformName: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
> = None;
pub static mut _ptrc_glGetActiveUniformsiv: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const GLuint, GLenum, *mut GLint) -> (),
> = None;
pub static mut _ptrc_glGetUniformBlockIndex: Option<
    unsafe extern "C" fn(GLuint, *const GLchar) -> GLuint,
> = None;
pub static mut _ptrc_glGetUniformIndices: Option<
    unsafe extern "C" fn(GLuint, GLsizei, *const *const GLchar, *mut GLuint) -> (),
> = None;
pub static mut _ptrc_glPrimitiveRestartIndex: Option<unsafe extern "C" fn(GLuint) -> ()> = None;
pub static mut _ptrc_glTexBuffer: Option<unsafe extern "C" fn(GLenum, GLenum, GLuint) -> ()> = None;
pub static mut _ptrc_glUniformBlockBinding: Option<
    unsafe extern "C" fn(GLuint, GLuint, GLuint) -> (),
> = None;
unsafe fn Load_Version_3_1() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    _ptrc_glAccum = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glAccum\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glAccum.is_none() {
        numFailed += 1;
    }
    _ptrc_glAlphaFunc = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glAlphaFunc\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glAlphaFunc.is_none() {
        numFailed += 1;
    }
    _ptrc_glBegin = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBegin\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBegin.is_none() {
        numFailed += 1;
    }
    _ptrc_glBitmap = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLsizei,
                GLsizei,
                GLfloat,
                GLfloat,
                GLfloat,
                GLfloat,
                *const GLubyte,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glBitmap\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBitmap.is_none() {
        numFailed += 1;
    }
    _ptrc_glBlendFunc = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBlendFunc\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBlendFunc.is_none() {
        numFailed += 1;
    }
    _ptrc_glCallList = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCallList\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCallList.is_none() {
        numFailed += 1;
    }
    _ptrc_glCallLists = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, GLenum, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCallLists\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCallLists.is_none() {
        numFailed += 1;
    }
    _ptrc_glClear = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLbitfield) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClear\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClear.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearAccum = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearAccum\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearAccum.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearColor = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearColor\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearColor.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearDepth = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearDepth\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearDepth.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearIndex = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearIndex\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearIndex.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearStencil = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearStencil\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearStencil.is_none() {
        numFailed += 1;
    }
    _ptrc_glClipPlane = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClipPlane\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClipPlane.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3b = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLbyte, GLbyte, GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3b\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3b.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3bv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3bv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3bv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3d.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3s.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3ub = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLubyte, GLubyte, GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3ub\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3ub.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3ubv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3ubv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3ubv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3us = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLushort, GLushort, GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3us\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3us.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor3usv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor3usv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor3usv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4b = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLbyte, GLbyte, GLbyte, GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4b\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4b.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4bv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4bv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4bv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4d.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4f.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4i.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4s.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4ub = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLubyte, GLubyte, GLubyte, GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4ub\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4ub.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4ubv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4ubv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4ubv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4us = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLushort, GLushort, GLushort, GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4us\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4us.is_none() {
        numFailed += 1;
    }
    _ptrc_glColor4usv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColor4usv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColor4usv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColorMask = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLboolean, GLboolean, GLboolean, GLboolean) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColorMask\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColorMask.is_none() {
        numFailed += 1;
    }
    _ptrc_glColorMaterial = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColorMaterial\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColorMaterial.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyPixels = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLsizei, GLsizei, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCopyPixels\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyPixels.is_none() {
        numFailed += 1;
    }
    _ptrc_glCullFace = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCullFace\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCullFace.is_none() {
        numFailed += 1;
    }
    _ptrc_glDeleteLists = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDeleteLists\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDeleteLists.is_none() {
        numFailed += 1;
    }
    _ptrc_glDepthFunc = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDepthFunc\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDepthFunc.is_none() {
        numFailed += 1;
    }
    _ptrc_glDepthMask = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLboolean) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDepthMask\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDepthMask.is_none() {
        numFailed += 1;
    }
    _ptrc_glDepthRange = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDepthRange\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDepthRange.is_none() {
        numFailed += 1;
    }
    _ptrc_glDisable = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDisable\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDisable.is_none() {
        numFailed += 1;
    }
    _ptrc_glDrawBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDrawBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDrawBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glDrawPixels = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLsizei,
                GLsizei,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glDrawPixels\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDrawPixels.is_none() {
        numFailed += 1;
    }
    _ptrc_glEdgeFlag = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLboolean) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEdgeFlag\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEdgeFlag.is_none() {
        numFailed += 1;
    }
    _ptrc_glEdgeFlagv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLboolean) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEdgeFlagv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEdgeFlagv.is_none() {
        numFailed += 1;
    }
    _ptrc_glEnable = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEnable\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEnable.is_none() {
        numFailed += 1;
    }
    _ptrc_glEnd =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glEnd\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glEnd.is_none() {
        numFailed += 1;
    }
    _ptrc_glEndList =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glEndList\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glEndList.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalCoord1d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalCoord1d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalCoord1d.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalCoord1dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalCoord1dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalCoord1dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalCoord1f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalCoord1f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalCoord1f.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalCoord1fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalCoord1fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalCoord1fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalCoord2d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalCoord2d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalCoord2d.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalCoord2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalCoord2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalCoord2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalCoord2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalCoord2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalCoord2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalCoord2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalCoord2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalCoord2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalMesh1 = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalMesh1\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalMesh1.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalMesh2 = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalMesh2\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalMesh2.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalPoint1 = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalPoint1\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalPoint1.is_none() {
        numFailed += 1;
    }
    _ptrc_glEvalPoint2 = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEvalPoint2\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEvalPoint2.is_none() {
        numFailed += 1;
    }
    _ptrc_glFeedbackBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFeedbackBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFeedbackBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glFinish =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glFinish\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glFinish.is_none() {
        numFailed += 1;
    }
    _ptrc_glFlush =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glFlush\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glFlush.is_none() {
        numFailed += 1;
    }
    _ptrc_glFogf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFogf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFogf.is_none() {
        numFailed += 1;
    }
    _ptrc_glFogfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFogfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFogfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glFogi = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFogi\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFogi.is_none() {
        numFailed += 1;
    }
    _ptrc_glFogiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFogiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFogiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glFrontFace = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFrontFace\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFrontFace.is_none() {
        numFailed += 1;
    }
    _ptrc_glFrustum = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble, GLdouble, GLdouble) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glFrustum\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFrustum.is_none() {
        numFailed += 1;
    }
    _ptrc_glGenLists = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei) -> GLuint>,
    >(SDL_GL_GetProcAddress(
        b"glGenLists\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGenLists.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetBooleanv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *mut GLboolean) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetBooleanv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetBooleanv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetClipPlane = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *mut GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetClipPlane\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetClipPlane.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetDoublev = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *mut GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetDoublev\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetDoublev.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetError = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn() -> GLenum>,
    >(SDL_GL_GetProcAddress(
        b"glGetError\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetError.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetFloatv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetFloatv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetFloatv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetIntegerv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetIntegerv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetIntegerv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetLightfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetLightfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetLightfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetLightiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetLightiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetLightiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetMapdv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetMapdv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetMapdv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetMapfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetMapfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetMapfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetMapiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetMapiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetMapiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetMaterialfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetMaterialfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetMaterialfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetMaterialiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetMaterialiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetMaterialiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetPixelMapfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetPixelMapfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetPixelMapfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetPixelMapuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetPixelMapuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetPixelMapuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetPixelMapusv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *mut GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetPixelMapusv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetPixelMapusv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetPolygonStipple = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*mut GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetPolygonStipple\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetPolygonStipple.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetString = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> *const GLubyte>,
    >(SDL_GL_GetProcAddress(
        b"glGetString\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetString.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexEnvfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexEnvfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexEnvfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexEnviv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexEnviv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexEnviv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexGendv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexGendv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexGendv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexGenfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexGenfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexGenfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexGeniv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexGeniv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexGeniv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexImage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLenum, GLenum, *mut ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexImage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexImage.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexLevelParameterfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexLevelParameterfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexLevelParameterfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexLevelParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexLevelParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexLevelParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexParameterfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexParameterfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexParameterfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glHint = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glHint\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glHint.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexMask = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexMask\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexMask.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexd = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexd\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexd.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexdv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexdv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexdv.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexf.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexi = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexi\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexi.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexs = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexs\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexs.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexsv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexsv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexsv.is_none() {
        numFailed += 1;
    }
    _ptrc_glInitNames =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glInitNames\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glInitNames.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsEnabled = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsEnabled\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsEnabled.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsList = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsList\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsList.is_none() {
        numFailed += 1;
    }
    _ptrc_glLightModelf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLightModelf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLightModelf.is_none() {
        numFailed += 1;
    }
    _ptrc_glLightModelfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLightModelfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLightModelfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glLightModeli = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLightModeli\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLightModeli.is_none() {
        numFailed += 1;
    }
    _ptrc_glLightModeliv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLightModeliv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLightModeliv.is_none() {
        numFailed += 1;
    }
    _ptrc_glLightf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLightf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLightf.is_none() {
        numFailed += 1;
    }
    _ptrc_glLightfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLightfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLightfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glLighti = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLighti\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLighti.is_none() {
        numFailed += 1;
    }
    _ptrc_glLightiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLightiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLightiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glLineStipple = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLineStipple\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLineStipple.is_none() {
        numFailed += 1;
    }
    _ptrc_glLineWidth = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLineWidth\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLineWidth.is_none() {
        numFailed += 1;
    }
    _ptrc_glListBase = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glListBase\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glListBase.is_none() {
        numFailed += 1;
    }
    _ptrc_glLoadIdentity =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glLoadIdentity\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glLoadIdentity.is_none() {
        numFailed += 1;
    }
    _ptrc_glLoadMatrixd = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLoadMatrixd\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLoadMatrixd.is_none() {
        numFailed += 1;
    }
    _ptrc_glLoadMatrixf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLoadMatrixf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLoadMatrixf.is_none() {
        numFailed += 1;
    }
    _ptrc_glLoadName = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLoadName\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLoadName.is_none() {
        numFailed += 1;
    }
    _ptrc_glLogicOp = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLogicOp\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLogicOp.is_none() {
        numFailed += 1;
    }
    _ptrc_glMap1d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLenum, GLdouble, GLdouble, GLint, GLint, *const GLdouble) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glMap1d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMap1d.is_none() {
        numFailed += 1;
    }
    _ptrc_glMap1f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat, GLfloat, GLint, GLint, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMap1f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMap1f.is_none() {
        numFailed += 1;
    }
    _ptrc_glMap2d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLdouble,
                GLdouble,
                GLint,
                GLint,
                GLdouble,
                GLdouble,
                GLint,
                GLint,
                *const GLdouble,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glMap2d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMap2d.is_none() {
        numFailed += 1;
    }
    _ptrc_glMap2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLfloat,
                GLfloat,
                GLint,
                GLint,
                GLfloat,
                GLfloat,
                GLint,
                GLint,
                *const GLfloat,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glMap2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMap2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glMapGrid1d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMapGrid1d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMapGrid1d.is_none() {
        numFailed += 1;
    }
    _ptrc_glMapGrid1f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMapGrid1f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMapGrid1f.is_none() {
        numFailed += 1;
    }
    _ptrc_glMapGrid2d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLdouble, GLdouble, GLint, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMapGrid2d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMapGrid2d.is_none() {
        numFailed += 1;
    }
    _ptrc_glMapGrid2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLfloat, GLfloat, GLint, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMapGrid2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMapGrid2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glMaterialf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMaterialf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMaterialf.is_none() {
        numFailed += 1;
    }
    _ptrc_glMaterialfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMaterialfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMaterialfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMateriali = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMateriali\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMateriali.is_none() {
        numFailed += 1;
    }
    _ptrc_glMaterialiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMaterialiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMaterialiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMatrixMode = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMatrixMode\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMatrixMode.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultMatrixd = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultMatrixd\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultMatrixd.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultMatrixf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultMatrixf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultMatrixf.is_none() {
        numFailed += 1;
    }
    _ptrc_glNewList = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNewList\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNewList.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormal3b = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLbyte, GLbyte, GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormal3b\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormal3b.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormal3bv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormal3bv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormal3bv.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormal3d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormal3d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormal3d.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormal3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormal3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormal3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormal3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormal3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormal3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormal3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormal3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormal3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormal3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormal3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormal3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormal3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormal3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormal3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormal3s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormal3s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormal3s.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormal3sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormal3sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormal3sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glOrtho = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble, GLdouble, GLdouble) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glOrtho\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glOrtho.is_none() {
        numFailed += 1;
    }
    _ptrc_glPassThrough = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPassThrough\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPassThrough.is_none() {
        numFailed += 1;
    }
    _ptrc_glPixelMapfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPixelMapfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPixelMapfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glPixelMapuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPixelMapuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPixelMapuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glPixelMapusv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *const GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPixelMapusv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPixelMapusv.is_none() {
        numFailed += 1;
    }
    _ptrc_glPixelStoref = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPixelStoref\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPixelStoref.is_none() {
        numFailed += 1;
    }
    _ptrc_glPixelStorei = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPixelStorei\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPixelStorei.is_none() {
        numFailed += 1;
    }
    _ptrc_glPixelTransferf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPixelTransferf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPixelTransferf.is_none() {
        numFailed += 1;
    }
    _ptrc_glPixelTransferi = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPixelTransferi\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPixelTransferi.is_none() {
        numFailed += 1;
    }
    _ptrc_glPixelZoom = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPixelZoom\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPixelZoom.is_none() {
        numFailed += 1;
    }
    _ptrc_glPointSize = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPointSize\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPointSize.is_none() {
        numFailed += 1;
    }
    _ptrc_glPolygonMode = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPolygonMode\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPolygonMode.is_none() {
        numFailed += 1;
    }
    _ptrc_glPolygonStipple = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPolygonStipple\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPolygonStipple.is_none() {
        numFailed += 1;
    }
    _ptrc_glPopAttrib =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glPopAttrib\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glPopAttrib.is_none() {
        numFailed += 1;
    }
    _ptrc_glPopMatrix =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glPopMatrix\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glPopMatrix.is_none() {
        numFailed += 1;
    }
    _ptrc_glPopName =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glPopName\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glPopName.is_none() {
        numFailed += 1;
    }
    _ptrc_glPushAttrib = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLbitfield) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPushAttrib\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPushAttrib.is_none() {
        numFailed += 1;
    }
    _ptrc_glPushMatrix =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, Option<unsafe extern "C" fn() -> ()>>(
            SDL_GL_GetProcAddress(b"glPushMatrix\0" as *const u8 as *const ::core::ffi::c_char),
        );
    if _ptrc_glPushMatrix.is_none() {
        numFailed += 1;
    }
    _ptrc_glPushName = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPushName\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPushName.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos2d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos2d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos2d.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos2i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos2i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos2i.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos2iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos2iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos2iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos2s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos2s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos2s.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos2sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos2sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos2sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos3d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos3d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos3d.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos3s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos3s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos3s.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos3sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos3sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos3sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos4d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos4d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos4d.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos4dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos4dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos4dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos4f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos4f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos4f.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos4i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos4i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos4i.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos4iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos4iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos4iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos4s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos4s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos4s.is_none() {
        numFailed += 1;
    }
    _ptrc_glRasterPos4sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRasterPos4sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRasterPos4sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glReadBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glReadBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glReadBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glReadPixels = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLenum,
                GLenum,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glReadPixels\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glReadPixels.is_none() {
        numFailed += 1;
    }
    _ptrc_glRectd = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRectd\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRectd.is_none() {
        numFailed += 1;
    }
    _ptrc_glRectdv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRectdv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRectdv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRectf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRectf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRectf.is_none() {
        numFailed += 1;
    }
    _ptrc_glRectfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRectfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRectfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRecti = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRecti\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRecti.is_none() {
        numFailed += 1;
    }
    _ptrc_glRectiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRectiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRectiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRects = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRects\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRects.is_none() {
        numFailed += 1;
    }
    _ptrc_glRectsv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRectsv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRectsv.is_none() {
        numFailed += 1;
    }
    _ptrc_glRenderMode = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> GLint>,
    >(SDL_GL_GetProcAddress(
        b"glRenderMode\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRenderMode.is_none() {
        numFailed += 1;
    }
    _ptrc_glRotated = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRotated\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRotated.is_none() {
        numFailed += 1;
    }
    _ptrc_glRotatef = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRotatef\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRotatef.is_none() {
        numFailed += 1;
    }
    _ptrc_glScaled = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glScaled\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glScaled.is_none() {
        numFailed += 1;
    }
    _ptrc_glScalef = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glScalef\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glScalef.is_none() {
        numFailed += 1;
    }
    _ptrc_glScissor = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glScissor\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glScissor.is_none() {
        numFailed += 1;
    }
    _ptrc_glSelectBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSelectBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSelectBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glShadeModel = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glShadeModel\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glShadeModel.is_none() {
        numFailed += 1;
    }
    _ptrc_glStencilFunc = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glStencilFunc\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glStencilFunc.is_none() {
        numFailed += 1;
    }
    _ptrc_glStencilMask = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glStencilMask\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glStencilMask.is_none() {
        numFailed += 1;
    }
    _ptrc_glStencilOp = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glStencilOp\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glStencilOp.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord1d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord1d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord1d.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord1dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord1dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord1dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord1f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord1f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord1f.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord1fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord1fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord1fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord1i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord1i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord1i.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord1iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord1iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord1iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord1s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord1s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord1s.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord1sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord1sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord1sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord2d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord2d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord2d.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord2i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord2i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord2i.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord2iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord2iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord2iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord2s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord2s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord2s.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord2sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord2sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord2sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord3d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord3d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord3d.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord3s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord3s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord3s.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord3sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord3sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord3sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord4d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord4d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord4d.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord4dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord4dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord4dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord4f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord4f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord4f.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord4i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord4i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord4i.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord4iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord4iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord4iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord4s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord4s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord4s.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoord4sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoord4sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoord4sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexEnvf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexEnvf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexEnvf.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexEnvfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexEnvfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexEnvfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexEnvi = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexEnvi\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexEnvi.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexEnviv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexEnviv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexEnviv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexGend = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexGend\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexGend.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexGendv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexGendv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexGendv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexGenf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexGenf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexGenf.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexGenfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexGenfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexGenfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexGeni = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexGeni\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexGeni.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexGeniv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexGeniv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexGeniv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexImage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLint,
                GLsizei,
                GLint,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTexImage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexImage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexImage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLint,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTexImage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexImage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexParameterf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexParameterf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexParameterf.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexParameterfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexParameterfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexParameterfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexParameteri = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexParameteri\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexParameteri.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTranslated = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTranslated\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTranslated.is_none() {
        numFailed += 1;
    }
    _ptrc_glTranslatef = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTranslatef\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTranslatef.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex2d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex2d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex2d.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex2i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex2i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex2i.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex2iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex2iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex2iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex2s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex2s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex2s.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex2sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex2sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex2sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex3d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex3d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex3d.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex3s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex3s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex3s.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex3sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex3sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex3sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex4d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex4d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex4d.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex4dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex4dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex4dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex4f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex4f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex4f.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex4i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex4i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex4i.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex4iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex4iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex4iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex4s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex4s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex4s.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertex4sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertex4sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertex4sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glViewport = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glViewport\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glViewport.is_none() {
        numFailed += 1;
    }
    _ptrc_glAreTexturesResident = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const GLuint, *mut GLboolean) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glAreTexturesResident\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glAreTexturesResident.is_none() {
        numFailed += 1;
    }
    _ptrc_glArrayElement = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glArrayElement\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glArrayElement.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindTexture = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindTexture\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindTexture.is_none() {
        numFailed += 1;
    }
    _ptrc_glColorPointer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLenum, GLsizei, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColorPointer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColorPointer.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyTexImage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLenum, GLint, GLint, GLsizei, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCopyTexImage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyTexImage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyTexImage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLenum,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLint,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCopyTexImage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyTexImage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyTexSubImage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLint, GLint, GLint, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCopyTexSubImage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyTexSubImage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyTexSubImage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLenum, GLint, GLint, GLint, GLint, GLint, GLsizei, GLsizei) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCopyTexSubImage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyTexSubImage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glDeleteTextures = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDeleteTextures\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDeleteTextures.is_none() {
        numFailed += 1;
    }
    _ptrc_glDisableClientState = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDisableClientState\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDisableClientState.is_none() {
        numFailed += 1;
    }
    _ptrc_glDrawArrays = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDrawArrays\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDrawArrays.is_none() {
        numFailed += 1;
    }
    _ptrc_glDrawElements = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, GLenum, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDrawElements\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDrawElements.is_none() {
        numFailed += 1;
    }
    _ptrc_glEdgeFlagPointer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEdgeFlagPointer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEdgeFlagPointer.is_none() {
        numFailed += 1;
    }
    _ptrc_glEnableClientState = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEnableClientState\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEnableClientState.is_none() {
        numFailed += 1;
    }
    _ptrc_glGenTextures = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGenTextures\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGenTextures.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetPointerv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *mut *mut ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetPointerv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetPointerv.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexPointer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexPointer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexPointer.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexub = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexub\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexub.is_none() {
        numFailed += 1;
    }
    _ptrc_glIndexubv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glIndexubv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIndexubv.is_none() {
        numFailed += 1;
    }
    _ptrc_glInterleavedArrays = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glInterleavedArrays\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glInterleavedArrays.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsTexture = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsTexture\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsTexture.is_none() {
        numFailed += 1;
    }
    _ptrc_glNormalPointer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glNormalPointer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glNormalPointer.is_none() {
        numFailed += 1;
    }
    _ptrc_glPolygonOffset = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPolygonOffset\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPolygonOffset.is_none() {
        numFailed += 1;
    }
    _ptrc_glPopClientAttrib = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn() -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPopClientAttrib\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPopClientAttrib.is_none() {
        numFailed += 1;
    }
    _ptrc_glPrioritizeTextures = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const GLuint, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPrioritizeTextures\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPrioritizeTextures.is_none() {
        numFailed += 1;
    }
    _ptrc_glPushClientAttrib = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLbitfield) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPushClientAttrib\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPushClientAttrib.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexCoordPointer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLenum, GLsizei, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexCoordPointer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexCoordPointer.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexSubImage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLint,
                GLsizei,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTexSubImage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexSubImage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexSubImage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTexSubImage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexSubImage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexPointer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLenum, GLsizei, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexPointer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexPointer.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyTexSubImage3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCopyTexSubImage3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyTexSubImage3D.is_none() {
        numFailed += 1;
    }
    _ptrc_glDrawRangeElements = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLuint,
                GLuint,
                GLsizei,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glDrawRangeElements\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDrawRangeElements.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexImage3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLsizei,
                GLint,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTexImage3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexImage3D.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexSubImage3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLsizei,
                GLenum,
                GLenum,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glTexSubImage3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexSubImage3D.is_none() {
        numFailed += 1;
    }
    _ptrc_glActiveTexture = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glActiveTexture\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glActiveTexture.is_none() {
        numFailed += 1;
    }
    _ptrc_glClientActiveTexture = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClientActiveTexture\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClientActiveTexture.is_none() {
        numFailed += 1;
    }
    _ptrc_glCompressedTexImage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLenum,
                GLsizei,
                GLint,
                GLsizei,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCompressedTexImage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCompressedTexImage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCompressedTexImage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLenum,
                GLsizei,
                GLsizei,
                GLint,
                GLsizei,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCompressedTexImage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCompressedTexImage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCompressedTexImage3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLenum,
                GLsizei,
                GLsizei,
                GLsizei,
                GLint,
                GLsizei,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCompressedTexImage3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCompressedTexImage3D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCompressedTexSubImage1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLint,
                GLsizei,
                GLenum,
                GLsizei,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCompressedTexSubImage1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCompressedTexSubImage1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCompressedTexSubImage2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLenum,
                GLsizei,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCompressedTexSubImage2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCompressedTexSubImage2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glCompressedTexSubImage3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLint,
                GLint,
                GLint,
                GLint,
                GLsizei,
                GLsizei,
                GLsizei,
                GLenum,
                GLsizei,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glCompressedTexSubImage3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCompressedTexSubImage3D.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetCompressedTexImage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, *mut ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetCompressedTexImage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetCompressedTexImage.is_none() {
        numFailed += 1;
    }
    _ptrc_glLoadTransposeMatrixd = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLoadTransposeMatrixd\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLoadTransposeMatrixd.is_none() {
        numFailed += 1;
    }
    _ptrc_glLoadTransposeMatrixf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLoadTransposeMatrixf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLoadTransposeMatrixf.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultTransposeMatrixd = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultTransposeMatrixd\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultTransposeMatrixd.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultTransposeMatrixf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultTransposeMatrixf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultTransposeMatrixf.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord1d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord1d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord1d.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord1dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord1dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord1dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord1f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord1f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord1f.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord1fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord1fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord1fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord1i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord1i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord1i.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord1iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord1iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord1iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord1s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord1s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord1s.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord1sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord1sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord1sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord2d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord2d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord2d.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord2i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord2i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord2i.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord2iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord2iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord2iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord2s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord2s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord2s.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord2sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord2sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord2sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord3d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord3d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord3d.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord3s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord3s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord3s.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord3sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord3sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord3sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord4d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLdouble, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord4d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord4d.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord4dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord4dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord4dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord4f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord4f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord4f.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord4i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord4i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord4i.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord4iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord4iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord4iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord4s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLshort, GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord4s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord4s.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiTexCoord4sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiTexCoord4sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiTexCoord4sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glSampleCoverage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLboolean) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSampleCoverage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSampleCoverage.is_none() {
        numFailed += 1;
    }
    _ptrc_glBlendColor = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBlendColor\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBlendColor.is_none() {
        numFailed += 1;
    }
    _ptrc_glBlendEquation = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBlendEquation\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBlendEquation.is_none() {
        numFailed += 1;
    }
    _ptrc_glBlendFuncSeparate = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBlendFuncSeparate\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBlendFuncSeparate.is_none() {
        numFailed += 1;
    }
    _ptrc_glFogCoordPointer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFogCoordPointer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFogCoordPointer.is_none() {
        numFailed += 1;
    }
    _ptrc_glFogCoordd = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFogCoordd\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFogCoordd.is_none() {
        numFailed += 1;
    }
    _ptrc_glFogCoorddv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFogCoorddv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFogCoorddv.is_none() {
        numFailed += 1;
    }
    _ptrc_glFogCoordf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFogCoordf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFogCoordf.is_none() {
        numFailed += 1;
    }
    _ptrc_glFogCoordfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFogCoordfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFogCoordfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiDrawArrays = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLint, *const GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glMultiDrawArrays\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiDrawArrays.is_none() {
        numFailed += 1;
    }
    _ptrc_glMultiDrawElements = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                *const GLsizei,
                GLenum,
                *const *const ::core::ffi::c_void,
                GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glMultiDrawElements\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMultiDrawElements.is_none() {
        numFailed += 1;
    }
    _ptrc_glPointParameterf = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPointParameterf\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPointParameterf.is_none() {
        numFailed += 1;
    }
    _ptrc_glPointParameterfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPointParameterfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPointParameterfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glPointParameteri = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPointParameteri\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPointParameteri.is_none() {
        numFailed += 1;
    }
    _ptrc_glPointParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPointParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPointParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3b = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLbyte, GLbyte, GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3b\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3b.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3bv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3bv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3bv.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3d.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3s.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3ub = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLubyte, GLubyte, GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3ub\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3ub.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3ubv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3ubv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3ubv.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3us = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLushort, GLushort, GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3us\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3us.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColor3usv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColor3usv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColor3usv.is_none() {
        numFailed += 1;
    }
    _ptrc_glSecondaryColorPointer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLenum, GLsizei, *const ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glSecondaryColorPointer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glSecondaryColorPointer.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos2d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos2d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos2d.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos2i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos2i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos2i.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos2iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos2iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos2iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos2s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos2s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos2s.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos2sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos2sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos2sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos3d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos3d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos3d.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos3s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos3s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos3s.is_none() {
        numFailed += 1;
    }
    _ptrc_glWindowPos3sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(*const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glWindowPos3sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glWindowPos3sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glBeginQuery = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBeginQuery\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBeginQuery.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glBufferData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizeiptr, *const ::core::ffi::c_void, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBufferData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBufferData.is_none() {
        numFailed += 1;
    }
    _ptrc_glBufferSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLenum, GLintptr, GLsizeiptr, *const ::core::ffi::c_void) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glBufferSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBufferSubData.is_none() {
        numFailed += 1;
    }
    _ptrc_glDeleteBuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDeleteBuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDeleteBuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glDeleteQueries = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDeleteQueries\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDeleteQueries.is_none() {
        numFailed += 1;
    }
    _ptrc_glEndQuery = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEndQuery\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEndQuery.is_none() {
        numFailed += 1;
    }
    _ptrc_glGenBuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGenBuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGenBuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glGenQueries = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGenQueries\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGenQueries.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetBufferParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetBufferParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetBufferParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetBufferPointerv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut *mut ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetBufferPointerv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetBufferPointerv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetBufferSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLintptr, GLsizeiptr, *mut ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetBufferSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetBufferSubData.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetQueryObjectiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetQueryObjectiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetQueryObjectiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetQueryObjectuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetQueryObjectuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetQueryObjectuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetQueryiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetQueryiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetQueryiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsQuery = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsQuery\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsQuery.is_none() {
        numFailed += 1;
    }
    _ptrc_glMapBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum) -> *mut ::core::ffi::c_void>,
    >(SDL_GL_GetProcAddress(
        b"glMapBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMapBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glUnmapBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glUnmapBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUnmapBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glAttachShader = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glAttachShader\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glAttachShader.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindAttribLocation = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, *const GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindAttribLocation\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindAttribLocation.is_none() {
        numFailed += 1;
    }
    _ptrc_glBlendEquationSeparate = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBlendEquationSeparate\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBlendEquationSeparate.is_none() {
        numFailed += 1;
    }
    _ptrc_glCompileShader = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCompileShader\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCompileShader.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateProgram = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn() -> GLuint>,
    >(SDL_GL_GetProcAddress(
        b"glCreateProgram\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateProgram.is_none() {
        numFailed += 1;
    }
    _ptrc_glCreateShader = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> GLuint>,
    >(SDL_GL_GetProcAddress(
        b"glCreateShader\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCreateShader.is_none() {
        numFailed += 1;
    }
    _ptrc_glDeleteProgram = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDeleteProgram\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDeleteProgram.is_none() {
        numFailed += 1;
    }
    _ptrc_glDeleteShader = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDeleteShader\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDeleteShader.is_none() {
        numFailed += 1;
    }
    _ptrc_glDetachShader = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDetachShader\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDetachShader.is_none() {
        numFailed += 1;
    }
    _ptrc_glDisableVertexAttribArray = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDisableVertexAttribArray\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDisableVertexAttribArray.is_none() {
        numFailed += 1;
    }
    _ptrc_glDrawBuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDrawBuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDrawBuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glEnableVertexAttribArray = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEnableVertexAttribArray\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEnableVertexAttribArray.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetActiveAttrib = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLuint,
                GLsizei,
                *mut GLsizei,
                *mut GLint,
                *mut GLenum,
                *mut GLchar,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetActiveAttrib\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetActiveAttrib.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetActiveUniform = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLuint,
                GLsizei,
                *mut GLsizei,
                *mut GLint,
                *mut GLenum,
                *mut GLchar,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetActiveUniform\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetActiveUniform.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetAttachedShaders = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetAttachedShaders\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetAttachedShaders.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetAttribLocation = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLchar) -> GLint>,
    >(SDL_GL_GetProcAddress(
        b"glGetAttribLocation\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetAttribLocation.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetProgramInfoLog = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramInfoLog\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramInfoLog.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetProgramiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetProgramiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetProgramiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetShaderInfoLog = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetShaderInfoLog\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetShaderInfoLog.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetShaderSource = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetShaderSource\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetShaderSource.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetShaderiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetShaderiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetShaderiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetUniformLocation = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLchar) -> GLint>,
    >(SDL_GL_GetProcAddress(
        b"glGetUniformLocation\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetUniformLocation.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetUniformfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetUniformfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetUniformfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetUniformiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetUniformiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetUniformiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetVertexAttribPointerv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut *mut ::core::ffi::c_void) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetVertexAttribPointerv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetVertexAttribPointerv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetVertexAttribdv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetVertexAttribdv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetVertexAttribdv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetVertexAttribfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetVertexAttribfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetVertexAttribfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetVertexAttribiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetVertexAttribiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetVertexAttribiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsProgram = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsProgram\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsProgram.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsShader = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsShader\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsShader.is_none() {
        numFailed += 1;
    }
    _ptrc_glLinkProgram = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glLinkProgram\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glLinkProgram.is_none() {
        numFailed += 1;
    }
    _ptrc_glShaderSource = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const *const GLchar, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glShaderSource\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glShaderSource.is_none() {
        numFailed += 1;
    }
    _ptrc_glStencilFuncSeparate = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glStencilFuncSeparate\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glStencilFuncSeparate.is_none() {
        numFailed += 1;
    }
    _ptrc_glStencilMaskSeparate = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glStencilMaskSeparate\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glStencilMaskSeparate.is_none() {
        numFailed += 1;
    }
    _ptrc_glStencilOpSeparate = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glStencilOpSeparate\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glStencilOpSeparate.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform1f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform1f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform1f.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform1fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform1fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform1fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform1i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform1i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform1i.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform1iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform1iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform1iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform2i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform2i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform2i.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform2iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform2iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform2iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform4f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform4f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform4f.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform4i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform4i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform4i.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform4iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform4iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform4iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniformMatrix2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniformMatrix2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniformMatrix2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniformMatrix3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniformMatrix3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniformMatrix3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniformMatrix4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniformMatrix4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniformMatrix4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUseProgram = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUseProgram\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUseProgram.is_none() {
        numFailed += 1;
    }
    _ptrc_glValidateProgram = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glValidateProgram\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glValidateProgram.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib1d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib1d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib1d.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib1dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib1dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib1dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib1f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib1f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib1f.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib1fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib1fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib1fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib1s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib1s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib1s.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib1sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib1sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib1sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib2d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib2d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib2d.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib2dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib2dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib2dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib2f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib2f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib2f.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib2s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib2s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib2s.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib2sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib2sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib2sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib3d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib3d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib3d.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib3dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib3dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib3dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib3f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib3f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib3f.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib3s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib3s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib3s.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib3sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib3sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib3sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4Nbv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4Nbv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4Nbv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4Niv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4Niv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4Niv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4Nsv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4Nsv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4Nsv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4Nub = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLubyte, GLubyte, GLubyte, GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4Nub\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4Nub.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4Nubv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4Nubv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4Nubv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4Nuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4Nuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4Nuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4Nusv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4Nusv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4Nusv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4bv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4bv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4bv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4d = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLdouble, GLdouble, GLdouble, GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4d\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4d.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4dv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLdouble) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4dv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4dv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4f = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLfloat, GLfloat, GLfloat, GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4f\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4f.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4s = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLshort, GLshort, GLshort, GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4s\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4s.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4ubv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4ubv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4ubv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttrib4usv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttrib4usv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttrib4usv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribPointer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLint,
                GLenum,
                GLboolean,
                GLsizei,
                *const ::core::ffi::c_void,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribPointer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribPointer.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniformMatrix2x3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniformMatrix2x3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniformMatrix2x3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniformMatrix2x4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniformMatrix2x4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniformMatrix2x4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniformMatrix3x2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniformMatrix3x2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniformMatrix3x2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniformMatrix3x4fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniformMatrix3x4fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniformMatrix3x4fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniformMatrix4x2fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniformMatrix4x2fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniformMatrix4x2fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniformMatrix4x3fv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniformMatrix4x3fv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniformMatrix4x3fv.is_none() {
        numFailed += 1;
    }
    _ptrc_glBeginConditionalRender = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBeginConditionalRender\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBeginConditionalRender.is_none() {
        numFailed += 1;
    }
    _ptrc_glBeginTransformFeedback = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBeginTransformFeedback\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBeginTransformFeedback.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindBufferBase = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindBufferBase\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindBufferBase.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindBufferRange = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint, GLuint, GLintptr, GLsizeiptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindBufferRange\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindBufferRange.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindFragDataLocation = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, *const GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindFragDataLocation\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindFragDataLocation.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindFramebuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindFramebuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindFramebuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindRenderbuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindRenderbuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindRenderbuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glBindVertexArray = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glBindVertexArray\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBindVertexArray.is_none() {
        numFailed += 1;
    }
    _ptrc_glBlitFramebuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLint,
                GLbitfield,
                GLenum,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glBlitFramebuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glBlitFramebuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glCheckFramebufferStatus = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> GLenum>,
    >(SDL_GL_GetProcAddress(
        b"glCheckFramebufferStatus\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCheckFramebufferStatus.is_none() {
        numFailed += 1;
    }
    _ptrc_glClampColor = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClampColor\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClampColor.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearBufferfi = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLfloat, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearBufferfi\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearBufferfi.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearBufferfv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, *const GLfloat) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearBufferfv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearBufferfv.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearBufferiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearBufferiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearBufferiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glClearBufferuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glClearBufferuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glClearBufferuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glColorMaski = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLboolean, GLboolean, GLboolean, GLboolean) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glColorMaski\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glColorMaski.is_none() {
        numFailed += 1;
    }
    _ptrc_glDeleteFramebuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDeleteFramebuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDeleteFramebuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glDeleteRenderbuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDeleteRenderbuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDeleteRenderbuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glDeleteVertexArrays = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDeleteVertexArrays\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDeleteVertexArrays.is_none() {
        numFailed += 1;
    }
    _ptrc_glDisablei = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDisablei\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDisablei.is_none() {
        numFailed += 1;
    }
    _ptrc_glEnablei = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEnablei\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEnablei.is_none() {
        numFailed += 1;
    }
    _ptrc_glEndConditionalRender = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn() -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEndConditionalRender\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEndConditionalRender.is_none() {
        numFailed += 1;
    }
    _ptrc_glEndTransformFeedback = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn() -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glEndTransformFeedback\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glEndTransformFeedback.is_none() {
        numFailed += 1;
    }
    _ptrc_glFlushMappedBufferRange = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLintptr, GLsizeiptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFlushMappedBufferRange\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFlushMappedBufferRange.is_none() {
        numFailed += 1;
    }
    _ptrc_glFramebufferRenderbuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFramebufferRenderbuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFramebufferRenderbuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glFramebufferTexture1D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLuint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFramebufferTexture1D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFramebufferTexture1D.is_none() {
        numFailed += 1;
    }
    _ptrc_glFramebufferTexture2D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLuint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFramebufferTexture2D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFramebufferTexture2D.is_none() {
        numFailed += 1;
    }
    _ptrc_glFramebufferTexture3D = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLuint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFramebufferTexture3D\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFramebufferTexture3D.is_none() {
        numFailed += 1;
    }
    _ptrc_glFramebufferTextureLayer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLuint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glFramebufferTextureLayer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glFramebufferTextureLayer.is_none() {
        numFailed += 1;
    }
    _ptrc_glGenFramebuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGenFramebuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGenFramebuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glGenRenderbuffers = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGenRenderbuffers\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGenRenderbuffers.is_none() {
        numFailed += 1;
    }
    _ptrc_glGenVertexArrays = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGenVertexArrays\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGenVertexArrays.is_none() {
        numFailed += 1;
    }
    _ptrc_glGenerateMipmap = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGenerateMipmap\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGenerateMipmap.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetBooleani_v = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint, *mut GLboolean) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetBooleani_v\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetBooleani_v.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetFragDataLocation = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLchar) -> GLint>,
    >(SDL_GL_GetProcAddress(
        b"glGetFragDataLocation\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetFragDataLocation.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetFramebufferAttachmentParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetFramebufferAttachmentParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetFramebufferAttachmentParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetIntegeri_v = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetIntegeri_v\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetIntegeri_v.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetRenderbufferParameteriv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetRenderbufferParameteriv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetRenderbufferParameteriv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetStringi = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint) -> *const GLubyte>,
    >(SDL_GL_GetProcAddress(
        b"glGetStringi\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetStringi.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexParameterIiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexParameterIiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexParameterIiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTexParameterIuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetTexParameterIuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTexParameterIuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetTransformFeedbackVarying = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLuint,
                GLuint,
                GLsizei,
                *mut GLsizei,
                *mut GLsizei,
                *mut GLenum,
                *mut GLchar,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glGetTransformFeedbackVarying\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetTransformFeedbackVarying.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetUniformuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetUniformuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetUniformuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetVertexAttribIiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetVertexAttribIiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetVertexAttribIiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetVertexAttribIuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetVertexAttribIuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetVertexAttribIuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsEnabledi = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsEnabledi\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsEnabledi.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsFramebuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsFramebuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsFramebuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsRenderbuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsRenderbuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsRenderbuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glIsVertexArray = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> GLboolean>,
    >(SDL_GL_GetProcAddress(
        b"glIsVertexArray\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glIsVertexArray.is_none() {
        numFailed += 1;
    }
    _ptrc_glMapBufferRange = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLintptr,
                GLsizeiptr,
                GLbitfield,
            ) -> *mut ::core::ffi::c_void,
        >,
    >(SDL_GL_GetProcAddress(
        b"glMapBufferRange\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glMapBufferRange.is_none() {
        numFailed += 1;
    }
    _ptrc_glRenderbufferStorage = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRenderbufferStorage\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRenderbufferStorage.is_none() {
        numFailed += 1;
    }
    _ptrc_glRenderbufferStorageMultisample = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLsizei, GLenum, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glRenderbufferStorageMultisample\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glRenderbufferStorageMultisample.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexParameterIiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexParameterIiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexParameterIiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexParameterIuiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexParameterIuiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexParameterIuiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glTransformFeedbackVaryings = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const *const GLchar, GLenum) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTransformFeedbackVaryings\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTransformFeedbackVaryings.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform1ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform1ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform1ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform1uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform1uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform1uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform2ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform2ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform2ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform2uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform2uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform2uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform3ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform3ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform3ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform3uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform3uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform3uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform4ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLuint, GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform4ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform4ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniform4uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLint, GLsizei, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniform4uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniform4uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI1i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI1i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI1i.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI1iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI1iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI1iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI1ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI1ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI1ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI1uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI1uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI1uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI2i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI2i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI2i.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI2iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI2iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI2iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI2ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI2ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI2ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI2uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI2uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI2uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI3i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI3i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI3i.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI3iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI3iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI3iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI3ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI3ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI3ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI3uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI3uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI3uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI4bv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLbyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI4bv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI4bv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI4i = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLint, GLint, GLint, GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI4i\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI4i.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI4iv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI4iv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI4iv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI4sv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLshort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI4sv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI4sv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI4ubv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLubyte) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI4ubv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI4ubv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI4ui = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI4ui\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI4ui.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI4uiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI4uiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI4uiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribI4usv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLushort) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribI4usv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribI4usv.is_none() {
        numFailed += 1;
    }
    _ptrc_glVertexAttribIPointer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(GLuint, GLint, GLenum, GLsizei, *const ::core::ffi::c_void) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glVertexAttribIPointer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glVertexAttribIPointer.is_none() {
        numFailed += 1;
    }
    _ptrc_glCopyBufferSubData = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLintptr, GLintptr, GLsizeiptr) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glCopyBufferSubData\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glCopyBufferSubData.is_none() {
        numFailed += 1;
    }
    _ptrc_glDrawArraysInstanced = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLint, GLsizei, GLsizei) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glDrawArraysInstanced\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDrawArraysInstanced.is_none() {
        numFailed += 1;
    }
    _ptrc_glDrawElementsInstanced = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<
            unsafe extern "C" fn(
                GLenum,
                GLsizei,
                GLenum,
                *const ::core::ffi::c_void,
                GLsizei,
            ) -> (),
        >,
    >(SDL_GL_GetProcAddress(
        b"glDrawElementsInstanced\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glDrawElementsInstanced.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetActiveUniformBlockName = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetActiveUniformBlockName\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetActiveUniformBlockName.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetActiveUniformBlockiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetActiveUniformBlockiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetActiveUniformBlockiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetActiveUniformName = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetActiveUniformName\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetActiveUniformName.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetActiveUniformsiv = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const GLuint, GLenum, *mut GLint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetActiveUniformsiv\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetActiveUniformsiv.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetUniformBlockIndex = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, *const GLchar) -> GLuint>,
    >(SDL_GL_GetProcAddress(
        b"glGetUniformBlockIndex\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetUniformBlockIndex.is_none() {
        numFailed += 1;
    }
    _ptrc_glGetUniformIndices = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const *const GLchar, *mut GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glGetUniformIndices\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetUniformIndices.is_none() {
        numFailed += 1;
    }
    _ptrc_glPrimitiveRestartIndex = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glPrimitiveRestartIndex\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glPrimitiveRestartIndex.is_none() {
        numFailed += 1;
    }
    _ptrc_glTexBuffer = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum, GLenum, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glTexBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glTexBuffer.is_none() {
        numFailed += 1;
    }
    _ptrc_glUniformBlockBinding = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLuint, GLuint, GLuint) -> ()>,
    >(SDL_GL_GetProcAddress(
        b"glUniformBlockBinding\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glUniformBlockBinding.is_none() {
        numFailed += 1;
    }
    return numFailed;
}
static mut ExtensionMap: [ogl_StrToExtMap; 52] = unsafe {
    [
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_compressed_texture_pixel_storage\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_compressed_texture_pixel_storage
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_conservative_depth\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_conservative_depth as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_ES2_compatibility\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_ES2_compatibility as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_ES2_compatibility as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_get_program_binary\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_get_program_binary as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_get_program_binary as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_explicit_uniform_location\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_explicit_uniform_location
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_internalformat_query\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_internalformat_query
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_internalformat_query as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_internalformat_query2\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_internalformat_query2
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(
                Load_ARB_internalformat_query2 as unsafe fn() -> ::core::ffi::c_int,
            ),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_map_buffer_alignment\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_map_buffer_alignment
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_program_interface_query\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_program_interface_query
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(
                Load_ARB_program_interface_query as unsafe fn() -> ::core::ffi::c_int,
            ),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_separate_shader_objects\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_separate_shader_objects
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(
                Load_ARB_separate_shader_objects as unsafe fn() -> ::core::ffi::c_int,
            ),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_shading_language_420pack\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_shading_language_420pack
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_shading_language_packing\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_shading_language_packing
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_texture_buffer_range\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_texture_buffer_range
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_texture_buffer_range as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_texture_storage\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_texture_storage as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_texture_storage as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_texture_view\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_texture_view as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_texture_view as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_vertex_attrib_binding\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_vertex_attrib_binding
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(
                Load_ARB_vertex_attrib_binding as unsafe fn() -> ::core::ffi::c_int,
            ),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_viewport_array\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_viewport_array as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_viewport_array as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_arrays_of_arrays\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_arrays_of_arrays as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_clear_buffer_object\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_clear_buffer_object
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_clear_buffer_object as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_copy_image\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_copy_image as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_copy_image as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_ES3_compatibility\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_ES3_compatibility as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_fragment_layer_viewport\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_fragment_layer_viewport
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_framebuffer_no_attachments\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_framebuffer_no_attachments
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(
                Load_ARB_framebuffer_no_attachments as unsafe fn() -> ::core::ffi::c_int,
            ),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_invalidate_subdata\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_invalidate_subdata as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_invalidate_subdata as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_robust_buffer_access_behavior\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_robust_buffer_access_behavior
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_stencil_texturing\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_stencil_texturing as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_texture_query_levels\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_texture_query_levels
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_texture_storage_multisample\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_texture_storage_multisample
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(
                Load_ARB_texture_storage_multisample as unsafe fn() -> ::core::ffi::c_int,
            ),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_KHR_debug\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_KHR_debug as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_KHR_debug as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_buffer_storage\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_buffer_storage as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_buffer_storage as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_clear_texture\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_clear_texture as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_clear_texture as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_enhanced_layouts\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_enhanced_layouts as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_multi_bind\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_multi_bind as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_multi_bind as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_query_buffer_object\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_query_buffer_object
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_texture_mirror_clamp_to_edge\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_texture_mirror_clamp_to_edge
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_texture_stencil8\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_texture_stencil8 as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_vertex_type_10f_11f_11f_rev\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_vertex_type_10f_11f_11f_rev
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_seamless_cubemap_per_texture\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_seamless_cubemap_per_texture
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_clip_control\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_clip_control as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_clip_control as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_conditional_render_inverted\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_conditional_render_inverted
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_cull_distance\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_cull_distance as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_derivative_control\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_derivative_control as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_direct_state_access\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_direct_state_access
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_direct_state_access as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_get_texture_sub_image\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_get_texture_sub_image
                as *mut ::core::ffi::c_int,
            LoadExtension: Some(
                Load_ARB_get_texture_sub_image as unsafe fn() -> ::core::ffi::c_int,
            ),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_shader_texture_image_samples\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_shader_texture_image_samples
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_ARB_texture_barrier\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_ARB_texture_barrier as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_ARB_texture_barrier as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_KHR_context_flush_control\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_KHR_context_flush_control
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_KHR_robust_buffer_access_behavior\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_KHR_robust_buffer_access_behavior
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_KHR_robustness\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_KHR_robustness as *mut ::core::ffi::c_int,
            LoadExtension: Some(Load_KHR_robustness as unsafe fn() -> ::core::ffi::c_int),
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_EXT_texture_compression_s3tc\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_EXT_texture_compression_s3tc
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_EXT_texture_sRGB\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_EXT_texture_sRGB as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
        ogl_StrToExtMap_s {
            extensionName: b"GL_EXT_texture_filter_anisotropic\0" as *const u8
                as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            extensionVariable: &raw const ogl_ext_EXT_texture_filter_anisotropic
                as *mut ::core::ffi::c_int,
            LoadExtension: None,
        },
    ]
};
static mut g_extensionMapSize: ::core::ffi::c_int = 52 as ::core::ffi::c_int;
unsafe fn FindExtEntry(mut extensionName: *const ::core::ffi::c_char) -> *mut ogl_StrToExtMap {
    let mut loop_0: ::core::ffi::c_int = 0;
    let mut currLoc: *mut ogl_StrToExtMap = &raw mut ExtensionMap as *mut ogl_StrToExtMap;
    loop_0 = 0 as ::core::ffi::c_int;
    while loop_0 < g_extensionMapSize {
        if strcmp(extensionName, (*currLoc).extensionName) == 0 as ::core::ffi::c_int {
            return currLoc;
        }
        loop_0 += 1;
        currLoc = currLoc.offset(1);
    }
    return ::core::ptr::null_mut::<ogl_StrToExtMap>();
}
unsafe fn ClearExtensionVars() {
    ogl_ext_ARB_compressed_texture_pixel_storage = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_conservative_depth = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_ES2_compatibility = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_get_program_binary = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_explicit_uniform_location = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_internalformat_query = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_internalformat_query2 = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_map_buffer_alignment = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_program_interface_query = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_separate_shader_objects = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_shading_language_420pack = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_shading_language_packing = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_texture_buffer_range = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_texture_storage = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_texture_view = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_vertex_attrib_binding = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_viewport_array = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_arrays_of_arrays = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_clear_buffer_object = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_copy_image = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_ES3_compatibility = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_fragment_layer_viewport = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_framebuffer_no_attachments = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_invalidate_subdata = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_robust_buffer_access_behavior = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_stencil_texturing = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_texture_query_levels = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_texture_storage_multisample = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_KHR_debug = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_buffer_storage = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_clear_texture = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_enhanced_layouts = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_multi_bind = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_query_buffer_object = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_texture_mirror_clamp_to_edge = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_texture_stencil8 = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_vertex_type_10f_11f_11f_rev = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_seamless_cubemap_per_texture = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_clip_control = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_conditional_render_inverted = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_cull_distance = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_derivative_control = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_direct_state_access = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_get_texture_sub_image = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_shader_texture_image_samples = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_ARB_texture_barrier = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_KHR_context_flush_control = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_KHR_robust_buffer_access_behavior = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_KHR_robustness = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_EXT_texture_compression_s3tc = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_EXT_texture_sRGB = ogl_LOAD_FAILED as ::core::ffi::c_int;
    ogl_ext_EXT_texture_filter_anisotropic = ogl_LOAD_FAILED as ::core::ffi::c_int;
}
unsafe fn LoadExtByName(mut extensionName: *const ::core::ffi::c_char) {
    let mut entry: *mut ogl_StrToExtMap = ::core::ptr::null_mut::<ogl_StrToExtMap>();
    entry = FindExtEntry(extensionName);
    if !entry.is_null() {
        if (*entry).LoadExtension.is_some() {
            let mut numFailed: ::core::ffi::c_int =
                (*entry).LoadExtension.expect("non-null function pointer")();
            if numFailed == 0 as ::core::ffi::c_int {
                *(*entry).extensionVariable = ogl_LOAD_SUCCEEDED as ::core::ffi::c_int;
            } else {
                *(*entry).extensionVariable = ogl_LOAD_SUCCEEDED as ::core::ffi::c_int + numFailed;
            }
        } else {
            *(*entry).extensionVariable = ogl_LOAD_SUCCEEDED as ::core::ffi::c_int;
        }
    }
}
unsafe fn ProcExtsFromExtString(mut strExtList: *const ::core::ffi::c_char) {
    let mut iExtListLen: size_t = strlen(strExtList);
    let mut strExtListEnd: *const ::core::ffi::c_char = strExtList.offset(iExtListLen as isize);
    let mut strCurrPos: *const ::core::ffi::c_char = strExtList;
    let mut strWorkBuff: [::core::ffi::c_char; 256] = [0; 256];
    while *strCurrPos != 0 {
        let mut iStrLen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut strEndStr: *const ::core::ffi::c_char = strchr(strCurrPos, ' ' as i32);
        let mut iStop: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if strEndStr.is_null() {
            strEndStr = strExtListEnd;
            iStop = 1 as ::core::ffi::c_int;
        }
        iStrLen = (strEndStr as ptrdiff_t - strCurrPos as ptrdiff_t) as ::core::ffi::c_int;
        if iStrLen > 255 as ::core::ffi::c_int {
            return;
        }
        strncpy(
            &raw mut strWorkBuff as *mut ::core::ffi::c_char,
            strCurrPos,
            iStrLen as size_t,
        );
        strWorkBuff[iStrLen as usize] = '\0' as i32 as ::core::ffi::c_char;
        LoadExtByName(&raw mut strWorkBuff as *mut ::core::ffi::c_char);
        strCurrPos = strEndStr.offset(1 as ::core::ffi::c_int as isize);
        if iStop != 0 {
            break;
        }
    }
}
unsafe fn ProcExtsFromExtList() {
    let mut iLoop: GLint = 0;
    let mut iNumExtensions: GLint = 0 as GLint;
    _ptrc_glGetIntegerv.expect("non-null function pointer")(
        GL_NUM_EXTENSIONS as GLenum,
        &raw mut iNumExtensions,
    );
    iLoop = 0 as ::core::ffi::c_int as GLint;
    while iLoop < iNumExtensions {
        let mut strExtensionName: *const ::core::ffi::c_char =
            _ptrc_glGetStringi.expect("non-null function pointer")(
                GL_EXTENSIONS as GLenum,
                iLoop as GLuint,
            ) as *const ::core::ffi::c_char;
        LoadExtByName(strExtensionName);
        iLoop += 1;
    }
}
pub unsafe fn ogl_LoadFunctions() -> ::core::ffi::c_int {
    let mut numFailed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    ClearExtensionVars();
    _ptrc_glGetString = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        Option<unsafe extern "C" fn(GLenum) -> *const GLubyte>,
    >(SDL_GL_GetProcAddress(
        b"glGetString\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    if _ptrc_glGetString.is_none() {
        return ogl_LOAD_FAILED as ::core::ffi::c_int;
    }
    if ogl_GetMajorVersion() < 3 as ::core::ffi::c_int {
        ProcExtsFromExtString(_ptrc_glGetString.expect("non-null function pointer")(
            GL_EXTENSIONS as GLenum,
        ) as *const ::core::ffi::c_char);
    } else {
        _ptrc_glGetIntegerv = ::core::mem::transmute::<
            *mut ::core::ffi::c_void,
            Option<unsafe extern "C" fn(GLenum, *mut GLint) -> ()>,
        >(SDL_GL_GetProcAddress(
            b"glGetIntegerv\0" as *const u8 as *const ::core::ffi::c_char,
        ));
        if _ptrc_glGetIntegerv.is_none() {
            return ogl_LOAD_FAILED as ::core::ffi::c_int;
        }
        _ptrc_glGetStringi = ::core::mem::transmute::<
            *mut ::core::ffi::c_void,
            Option<unsafe extern "C" fn(GLenum, GLuint) -> *const GLubyte>,
        >(SDL_GL_GetProcAddress(
            b"glGetStringi\0" as *const u8 as *const ::core::ffi::c_char,
        ));
        if _ptrc_glGetStringi.is_none() {
            return ogl_LOAD_FAILED as ::core::ffi::c_int;
        }
        ProcExtsFromExtList();
    }
    numFailed = Load_Version_3_1();
    if numFailed == 0 as ::core::ffi::c_int {
        return ogl_LOAD_SUCCEEDED as ::core::ffi::c_int;
    } else {
        return ogl_LOAD_SUCCEEDED as ::core::ffi::c_int + numFailed;
    };
}
static mut g_major_version: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut g_minor_version: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe fn ParseVersionFromString(
    mut pOutMajor: *mut ::core::ffi::c_int,
    mut pOutMinor: *mut ::core::ffi::c_int,
    mut strVersion: *const ::core::ffi::c_char,
) {
    let mut strDotPos: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    *pOutMinor = 0 as ::core::ffi::c_int;
    *pOutMajor = 0 as ::core::ffi::c_int;
    strDotPos = strchr(strVersion, '.' as i32);
    if strDotPos.is_null() {
        return;
    }
    *pOutMajor = atoi(strVersion);
    *pOutMinor = atoi(strDotPos.offset(1 as ::core::ffi::c_int as isize));
}
unsafe fn GetGLVersion() {
    ParseVersionFromString(
        &raw mut g_major_version,
        &raw mut g_minor_version,
        _ptrc_glGetString.expect("non-null function pointer")(GL_VERSION as GLenum)
            as *const ::core::ffi::c_char,
    );
}
pub unsafe fn ogl_GetMajorVersion() -> ::core::ffi::c_int {
    if g_major_version == 0 as ::core::ffi::c_int {
        GetGLVersion();
    }
    return g_major_version;
}
pub unsafe fn ogl_GetMinorVersion() -> ::core::ffi::c_int {
    if g_major_version == 0 as ::core::ffi::c_int {
        GetGLVersion();
    }
    return g_minor_version;
}
pub unsafe fn ogl_IsVersionGEQ(
    mut majorVersion: ::core::ffi::c_int,
    mut minorVersion: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if g_major_version == 0 as ::core::ffi::c_int {
        GetGLVersion();
    }
    if majorVersion < g_major_version {
        return 1 as ::core::ffi::c_int;
    }
    if majorVersion > g_major_version {
        return 0 as ::core::ffi::c_int;
    }
    if minorVersion <= g_minor_version {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
