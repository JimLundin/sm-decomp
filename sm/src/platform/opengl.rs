extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type SDL_Window;
    static mut _ptrc_glDebugMessageCallback:
        Option<unsafe extern "C" fn(GLDEBUGPROC, *const ::core::ffi::c_void) -> ()>;
    static mut _ptrc_glClear: Option<unsafe extern "C" fn(GLbitfield) -> ()>;
    static mut _ptrc_glClearColor:
        Option<unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> ()>;
    static mut _ptrc_glEnable: Option<unsafe extern "C" fn(GLenum) -> ()>;
    static mut _ptrc_glTexImage2D: Option<
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
    >;
    static mut _ptrc_glTexParameteri: Option<unsafe extern "C" fn(GLenum, GLenum, GLint) -> ()>;
    static mut _ptrc_glViewport: Option<unsafe extern "C" fn(GLint, GLint, GLsizei, GLsizei) -> ()>;
    static mut _ptrc_glBindTexture: Option<unsafe extern "C" fn(GLenum, GLuint) -> ()>;
    static mut _ptrc_glDrawArrays: Option<unsafe extern "C" fn(GLenum, GLint, GLsizei) -> ()>;
    static mut _ptrc_glGenTextures: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>;
    static mut _ptrc_glTexSubImage2D: Option<
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
    >;
    static mut _ptrc_glBindBuffer: Option<unsafe extern "C" fn(GLenum, GLuint) -> ()>;
    static mut _ptrc_glBufferData:
        Option<unsafe extern "C" fn(GLenum, GLsizeiptr, *const ::core::ffi::c_void, GLenum) -> ()>;
    static mut _ptrc_glGenBuffers: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>;
    static mut _ptrc_glAttachShader: Option<unsafe extern "C" fn(GLuint, GLuint) -> ()>;
    static mut _ptrc_glCompileShader: Option<unsafe extern "C" fn(GLuint) -> ()>;
    static mut _ptrc_glCreateProgram: Option<unsafe extern "C" fn() -> GLuint>;
    static mut _ptrc_glCreateShader: Option<unsafe extern "C" fn(GLenum) -> GLuint>;
    static mut _ptrc_glEnableVertexAttribArray: Option<unsafe extern "C" fn(GLuint) -> ()>;
    static mut _ptrc_glGetProgramInfoLog:
        Option<unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> ()>;
    static mut _ptrc_glGetProgramiv: Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>;
    static mut _ptrc_glGetShaderInfoLog:
        Option<unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> ()>;
    static mut _ptrc_glGetShaderiv: Option<unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> ()>;
    static mut _ptrc_glLinkProgram: Option<unsafe extern "C" fn(GLuint) -> ()>;
    static mut _ptrc_glShaderSource:
        Option<unsafe extern "C" fn(GLuint, GLsizei, *const *const GLchar, *const GLint) -> ()>;
    static mut _ptrc_glUseProgram: Option<unsafe extern "C" fn(GLuint) -> ()>;
    static mut _ptrc_glVertexAttribPointer: Option<
        unsafe extern "C" fn(
            GLuint,
            GLint,
            GLenum,
            GLboolean,
            GLsizei,
            *const ::core::ffi::c_void,
        ) -> (),
    >;
    static mut _ptrc_glBindVertexArray: Option<unsafe extern "C" fn(GLuint) -> ()>;
    static mut _ptrc_glGenVertexArrays: Option<unsafe extern "C" fn(GLsizei, *mut GLuint) -> ()>;
    fn ogl_LoadFunctions() -> ::core::ffi::c_int;
    fn ogl_IsVersionGEQ(
        majorVersion: ::core::ffi::c_int,
        minorVersion: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    static mut stderr: *mut FILE;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn SDL_GL_SetAttribute(attr: SDL_GLattr, value: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn SDL_GL_CreateContext(window: *mut SDL_Window) -> SDL_GLContext;
    fn SDL_GL_GetDrawableSize(
        window: *mut SDL_Window,
        w: *mut ::core::ffi::c_int,
        h: *mut ::core::ffi::c_int,
    );
    fn SDL_GL_SetSwapInterval(interval: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn SDL_GL_SwapWindow(window: *mut SDL_Window);
    fn Die(error: *const ::core::ffi::c_char);
    fn GlslShader_CreateFromFile(filename: *const ::core::ffi::c_char) -> *mut GlslShader;
    fn GlslShader_Render(
        gs: *mut GlslShader,
        tex: *mut GlTextureWithSize,
        viewport_x: ::core::ffi::c_int,
        viewport_y: ::core::ffi::c_int,
        viewport_width: ::core::ffi::c_int,
        viewport_height: ::core::ffi::c_int,
    );
    static mut g_config: Config;
}
pub use crate::types::{
    __off64_t, __off_t, __uint16_t, __uint32_t, __uint8_t, ptrdiff_t, size_t, uint, uint16_t,
    uint32_t, uint8_t, GLbitfield, GLboolean, GLchar, GLenum, GLfloat, GLint, GLsizei, GLsizeiptr,
    GLuint, GLDEBUGPROC,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub __pad5: size_t,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub use crate::types::_IO_lock_t;
pub type FILE = _IO_FILE;
pub use crate::types::{SDL_GLContext, SDL_GLattr};
pub const SDL_GL_FLOATBUFFERS: SDL_GLattr = 27;
pub const SDL_GL_CONTEXT_NO_ERROR: SDL_GLattr = 26;
pub const SDL_GL_CONTEXT_RESET_NOTIFICATION: SDL_GLattr = 25;
pub const SDL_GL_CONTEXT_RELEASE_BEHAVIOR: SDL_GLattr = 24;
pub const SDL_GL_FRAMEBUFFER_SRGB_CAPABLE: SDL_GLattr = 23;
pub const SDL_GL_SHARE_WITH_CURRENT_CONTEXT: SDL_GLattr = 22;
pub const SDL_GL_CONTEXT_PROFILE_MASK: SDL_GLattr = 21;
pub const SDL_GL_CONTEXT_FLAGS: SDL_GLattr = 20;
pub const SDL_GL_CONTEXT_EGL: SDL_GLattr = 19;
pub const SDL_GL_CONTEXT_MINOR_VERSION: SDL_GLattr = 18;
pub const SDL_GL_CONTEXT_MAJOR_VERSION: SDL_GLattr = 17;
pub const SDL_GL_RETAINED_BACKING: SDL_GLattr = 16;
pub const SDL_GL_ACCELERATED_VISUAL: SDL_GLattr = 15;
pub const SDL_GL_MULTISAMPLESAMPLES: SDL_GLattr = 14;
pub const SDL_GL_MULTISAMPLEBUFFERS: SDL_GLattr = 13;
pub const SDL_GL_STEREO: SDL_GLattr = 12;
pub const SDL_GL_ACCUM_ALPHA_SIZE: SDL_GLattr = 11;
pub const SDL_GL_ACCUM_BLUE_SIZE: SDL_GLattr = 10;
pub const SDL_GL_ACCUM_GREEN_SIZE: SDL_GLattr = 9;
pub const SDL_GL_ACCUM_RED_SIZE: SDL_GLattr = 8;
pub const SDL_GL_STENCIL_SIZE: SDL_GLattr = 7;
pub const SDL_GL_DEPTH_SIZE: SDL_GLattr = 6;
pub const SDL_GL_DOUBLEBUFFER: SDL_GLattr = 5;
pub const SDL_GL_BUFFER_SIZE: SDL_GLattr = 4;
pub const SDL_GL_ALPHA_SIZE: SDL_GLattr = 3;
pub const SDL_GL_BLUE_SIZE: SDL_GLattr = 2;
pub const SDL_GL_GREEN_SIZE: SDL_GLattr = 1;
pub const SDL_GL_RED_SIZE: SDL_GLattr = 0;
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const SDL_GL_CONTEXT_PROFILE_ES: C2RustUnnamed = 4;
pub const SDL_GL_CONTEXT_PROFILE_COMPATIBILITY: C2RustUnnamed = 2;
pub const SDL_GL_CONTEXT_PROFILE_CORE: C2RustUnnamed = 1;
pub use crate::types::{uint16, uint32, uint8};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RendererFuncs {
    pub Initialize: Option<unsafe extern "C" fn(*mut SDL_Window) -> bool>,
    pub Destroy: Option<unsafe extern "C" fn() -> ()>,
    pub BeginDraw: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut *mut uint8,
            *mut ::core::ffi::c_int,
        ) -> (),
    >,
    pub EndDraw: Option<unsafe extern "C" fn() -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslTextureUniform {
    pub Texture: ::core::ffi::c_int,
    pub InputSize: ::core::ffi::c_int,
    pub TextureSize: ::core::ffi::c_int,
    pub TexCoord: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslUniforms {
    pub Top: GlslTextureUniform,
    pub OutputSize: ::core::ffi::c_int,
    pub FrameCount: ::core::ffi::c_int,
    pub FrameDirection: ::core::ffi::c_int,
    pub LUTTexCoord: ::core::ffi::c_int,
    pub VertexCoord: ::core::ffi::c_int,
    pub Orig: GlslTextureUniform,
    pub Prev: [GlslTextureUniform; 7],
    pub Pass: [GlslTextureUniform; 20],
    pub PassPrev: [GlslTextureUniform; 20],
    pub Texture: [::core::ffi::c_int; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslPass {
    pub filename: *mut ::core::ffi::c_char,
    pub scale_type_x: uint8,
    pub scale_type_y: uint8,
    pub float_framebuffer: bool,
    pub srgb_framebuffer: bool,
    pub mipmap_input: bool,
    pub scale_x: ::core::ffi::c_float,
    pub scale_y: ::core::ffi::c_float,
    pub wrap_mode: uint,
    pub frame_count_mod: uint,
    pub frame_count: uint,
    pub gl_program: uint,
    pub gl_fbo: uint,
    pub filter: uint,
    pub gl_texture: uint,
    pub width: uint16,
    pub height: uint16,
    pub unif: GlslUniforms,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlTextureWithSize {
    pub gl_texture: uint,
    pub width: uint16,
    pub height: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslTexture {
    pub next: *mut GlslTexture,
    pub id: *mut ::core::ffi::c_char,
    pub filename: *mut ::core::ffi::c_char,
    pub filter: uint,
    pub gl_texture: uint,
    pub wrap_mode: uint,
    pub mipmap: bool,
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslParam {
    pub next: *mut GlslParam,
    pub id: *mut ::core::ffi::c_char,
    pub has_value: bool,
    pub value: ::core::ffi::c_float,
    pub min: ::core::ffi::c_float,
    pub max: ::core::ffi::c_float,
    pub uniform: [uint; 20],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GlslShader {
    pub n_pass: ::core::ffi::c_int,
    pub pass: *mut GlslPass,
    pub first_param: *mut GlslParam,
    pub first_texture: *mut GlslTexture,
    pub gl_vao: *mut uint,
    pub gl_vbo: uint,
    pub frame_count: uint,
    pub max_prev_frame: ::core::ffi::c_int,
    pub prev_frame: [GlTextureWithSize; 8],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Config {
    pub window_width: ::core::ffi::c_int,
    pub window_height: ::core::ffi::c_int,
    pub enhanced_mode7: bool,
    pub new_renderer: bool,
    pub ignore_aspect_ratio: bool,
    pub fullscreen: uint8,
    pub window_scale: uint8,
    pub enable_audio: bool,
    pub linear_filtering: bool,
    pub output_method: uint8,
    pub audio_freq: uint16,
    pub audio_channels: uint8,
    pub audio_samples: uint16,
    pub autosave: bool,
    pub extended_aspect_ratio: uint8,
    pub extend_y: bool,
    pub no_sprite_limits: bool,
    pub display_perf_title: bool,
    pub enable_msu: uint8,
    pub resume_msu: bool,
    pub disable_frame_delay: bool,
    pub msuvolume: uint8,
    pub features0: uint32,
    pub link_graphics: *const ::core::ffi::c_char,
    pub memory_buffer: *mut ::core::ffi::c_char,
    pub shader: *const ::core::ffi::c_char,
    pub msu_path: *const ::core::ffi::c_char,
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const GL_TEXTURE_2D: ::core::ffi::c_int = 0xde1 as ::core::ffi::c_int;
pub const GL_DEBUG_OUTPUT: ::core::ffi::c_int = 0x92e0 as ::core::ffi::c_int;
pub const GL_DEBUG_OUTPUT_SYNCHRONOUS: ::core::ffi::c_int = 0x8242 as ::core::ffi::c_int;
pub const GL_DEBUG_TYPE_ERROR: ::core::ffi::c_int = 0x824c as ::core::ffi::c_int;
pub const GL_DEBUG_TYPE_OTHER: ::core::ffi::c_int = 0x8251 as ::core::ffi::c_int;
pub const GL_COLOR_BUFFER_BIT: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const GL_FALSE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const GL_FLOAT: ::core::ffi::c_int = 0x1406 as ::core::ffi::c_int;
pub const GL_LINEAR: ::core::ffi::c_int = 0x2601 as ::core::ffi::c_int;
pub const GL_NEAREST: ::core::ffi::c_int = 0x2600 as ::core::ffi::c_int;
pub const GL_RGBA: ::core::ffi::c_int = 0x1908 as ::core::ffi::c_int;
pub const GL_TEXTURE_MAG_FILTER: ::core::ffi::c_int = 0x2800 as ::core::ffi::c_int;
pub const GL_TEXTURE_MIN_FILTER: ::core::ffi::c_int = 0x2801 as ::core::ffi::c_int;
pub const GL_TRIANGLE_STRIP: ::core::ffi::c_int = 0x5 as ::core::ffi::c_int;
pub const GL_BGRA: ::core::ffi::c_int = 0x80e1 as ::core::ffi::c_int;
pub const GL_UNSIGNED_INT_8_8_8_8_REV: ::core::ffi::c_int = 0x8367 as ::core::ffi::c_int;
pub const GL_ARRAY_BUFFER: ::core::ffi::c_int = 0x8892 as ::core::ffi::c_int;
pub const GL_STATIC_DRAW: ::core::ffi::c_int = 0x88e4 as ::core::ffi::c_int;
pub const GL_COMPILE_STATUS: ::core::ffi::c_int = 0x8b81 as ::core::ffi::c_int;
pub const GL_FRAGMENT_SHADER: ::core::ffi::c_int = 0x8b30 as ::core::ffi::c_int;
pub const GL_LINK_STATUS: ::core::ffi::c_int = 0x8b82 as ::core::ffi::c_int;
pub const GL_VERTEX_SHADER: ::core::ffi::c_int = 0x8b31 as ::core::ffi::c_int;
pub const kDebugFlag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut g_window: *mut SDL_Window = ::core::ptr::null::<SDL_Window>() as *mut SDL_Window;
static mut g_screen_buffer: *mut uint8 = ::core::ptr::null::<uint8>() as *mut uint8;
static mut g_screen_buffer_size: size_t = 0;
static mut g_draw_width: ::core::ffi::c_int = 0;
static mut g_draw_height: ::core::ffi::c_int = 0;
static mut g_program: ::core::ffi::c_uint = 0;
static mut g_VAO: ::core::ffi::c_uint = 0;
static mut g_texture: GlTextureWithSize = GlTextureWithSize {
    gl_texture: 0,
    width: 0,
    height: 0,
};
static mut g_glsl_shader: *mut GlslShader = ::core::ptr::null::<GlslShader>() as *mut GlslShader;
unsafe extern "C" fn MessageCallback(
    mut source: GLenum,
    mut type_0: GLenum,
    mut id: GLuint,
    mut severity: GLenum,
    mut length: GLsizei,
    mut message: *const GLchar,
    mut userParam: *const ::core::ffi::c_void,
) {
    if type_0 == GL_DEBUG_TYPE_OTHER as GLenum {
        return;
    }
    fprintf(
        stderr,
        b"GL CALLBACK: %s type = 0x%x, severity = 0x%x, message = %s\n\0" as *const u8
            as *const ::core::ffi::c_char,
        if type_0 == GL_DEBUG_TYPE_ERROR as GLenum {
            b"** GL ERROR **\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        type_0,
        severity,
        message,
    );
    if type_0 == GL_DEBUG_TYPE_ERROR as GLenum {
        Die(b"OpenGL error!\n\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
unsafe extern "C" fn OpenGLRenderer_Init(mut window: *mut SDL_Window) -> bool {
    g_window = window;
    let mut context: SDL_GLContext = SDL_GL_CreateContext(window as *mut SDL_Window);
    SDL_GL_SetSwapInterval(1 as ::core::ffi::c_int);
    ogl_LoadFunctions();
    if ogl_IsVersionGEQ(3 as ::core::ffi::c_int, 3 as ::core::ffi::c_int) == 0 {
        Die(b"You need OpenGL 3.3\0" as *const u8 as *const ::core::ffi::c_char);
    }
    _ptrc_glGenTextures.expect("non-null function pointer")(
        1 as GLsizei,
        &raw mut g_texture.gl_texture,
    );
    static mut kVertices: [::core::ffi::c_float; 20] = [
        -1.0f32, 1.0f32, 0.0f32, 0.0f32, 0.0f32, -1.0f32, -1.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32,
        1.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, -1.0f32, 0.0f32, 1.0f32, 1.0f32,
    ];
    let mut vbo: ::core::ffi::c_uint = 0;
    _ptrc_glGenBuffers.expect("non-null function pointer")(1 as GLsizei, &raw mut vbo);
    _ptrc_glGenVertexArrays.expect("non-null function pointer")(1 as GLsizei, &raw mut g_VAO);
    _ptrc_glBindVertexArray.expect("non-null function pointer")(g_VAO as GLuint);
    _ptrc_glBindBuffer.expect("non-null function pointer")(
        GL_ARRAY_BUFFER as GLenum,
        vbo as GLuint,
    );
    _ptrc_glBufferData.expect("non-null function pointer")(
        GL_ARRAY_BUFFER as GLenum,
        ::core::mem::size_of::<[::core::ffi::c_float; 20]>() as GLsizeiptr,
        &raw const kVertices as *const ::core::ffi::c_float as *const ::core::ffi::c_void,
        GL_STATIC_DRAW as GLenum,
    );
    _ptrc_glVertexAttribPointer.expect("non-null function pointer")(
        0 as GLuint,
        3 as GLint,
        GL_FLOAT as GLenum,
        GL_FALSE as GLboolean,
        (5 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_float>() as usize)
            as GLsizei,
        ::core::ptr::null::<::core::ffi::c_void>(),
    );
    _ptrc_glEnableVertexAttribArray.expect("non-null function pointer")(0 as GLuint);
    _ptrc_glVertexAttribPointer.expect("non-null function pointer")(
        1 as GLuint,
        2 as GLint,
        GL_FLOAT as GLenum,
        GL_FALSE as GLboolean,
        (5 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_float>() as usize)
            as GLsizei,
        (3 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_float>() as usize)
            as *mut ::core::ffi::c_void,
    );
    _ptrc_glEnableVertexAttribArray.expect("non-null function pointer")(1 as GLuint);
    let mut vs_code: *const GLchar = b"#version 330 core\nlayout(location = 0) in vec3 aPos; layout(location = 1) in vec2 aTexCoord; out vec2 TexCoord; void main(void) { gl_Position = vec4(aPos, 1.0); TexCoord = vec2(aTexCoord.x, aTexCoord.y); }\0"
        as *const u8 as *const GLchar;
    let mut vs: ::core::ffi::c_uint =
        _ptrc_glCreateShader.expect("non-null function pointer")(GL_VERTEX_SHADER as GLenum)
            as ::core::ffi::c_uint;
    _ptrc_glShaderSource.expect("non-null function pointer")(
        vs as GLuint,
        1 as GLsizei,
        &raw mut vs_code,
        ::core::ptr::null::<GLint>(),
    );
    _ptrc_glCompileShader.expect("non-null function pointer")(vs as GLuint);
    let mut success: ::core::ffi::c_int = 0;
    let mut infolog: [::core::ffi::c_char; 512] = [0; 512];
    _ptrc_glGetShaderiv.expect("non-null function pointer")(
        vs as GLuint,
        GL_COMPILE_STATUS as GLenum,
        &raw mut success,
    );
    if success == 0 {
        _ptrc_glGetShaderInfoLog.expect("non-null function pointer")(
            vs as GLuint,
            512 as GLsizei,
            ::core::ptr::null_mut::<GLsizei>(),
            &raw mut infolog as *mut GLchar,
        );
        printf(
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut infolog as *mut ::core::ffi::c_char,
        );
    }
    let mut fs_code: *const GLchar = b"#version 330 core\nout vec4 FragColor; in vec2 TexCoord; uniform sampler2D texture1; void main(void) { FragColor = texture(texture1, TexCoord); }\0"
        as *const u8 as *const GLchar;
    let mut fs: ::core::ffi::c_uint =
        _ptrc_glCreateShader.expect("non-null function pointer")(GL_FRAGMENT_SHADER as GLenum)
            as ::core::ffi::c_uint;
    _ptrc_glShaderSource.expect("non-null function pointer")(
        fs as GLuint,
        1 as GLsizei,
        &raw mut fs_code,
        ::core::ptr::null::<GLint>(),
    );
    _ptrc_glCompileShader.expect("non-null function pointer")(fs as GLuint);
    _ptrc_glGetShaderiv.expect("non-null function pointer")(
        fs as GLuint,
        GL_COMPILE_STATUS as GLenum,
        &raw mut success,
    );
    if success == 0 {
        _ptrc_glGetShaderInfoLog.expect("non-null function pointer")(
            fs as GLuint,
            512 as GLsizei,
            ::core::ptr::null_mut::<GLsizei>(),
            &raw mut infolog as *mut GLchar,
        );
        printf(
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut infolog as *mut ::core::ffi::c_char,
        );
    }
    g_program = _ptrc_glCreateProgram.expect("non-null function pointer")() as ::core::ffi::c_uint;
    let mut program: ::core::ffi::c_int = g_program as ::core::ffi::c_int;
    _ptrc_glAttachShader.expect("non-null function pointer")(program as GLuint, vs as GLuint);
    _ptrc_glAttachShader.expect("non-null function pointer")(program as GLuint, fs as GLuint);
    _ptrc_glLinkProgram.expect("non-null function pointer")(program as GLuint);
    _ptrc_glGetProgramiv.expect("non-null function pointer")(
        program as GLuint,
        GL_LINK_STATUS as GLenum,
        &raw mut success,
    );
    if success == 0 {
        _ptrc_glGetProgramInfoLog.expect("non-null function pointer")(
            program as GLuint,
            512 as GLsizei,
            ::core::ptr::null_mut::<GLsizei>(),
            &raw mut infolog as *mut GLchar,
        );
        printf(
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut infolog as *mut ::core::ffi::c_char,
        );
    }
    if !g_config.shader.is_null() {
        g_glsl_shader = GlslShader_CreateFromFile(g_config.shader);
    }
    return true_0 != 0;
}
unsafe extern "C" fn OpenGLRenderer_Destroy() {}
unsafe extern "C" fn OpenGLRenderer_BeginDraw(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
    mut pixels: *mut *mut uint8,
    mut pitch: *mut ::core::ffi::c_int,
) {
    let mut size: ::core::ffi::c_int = width * height;
    if size as size_t > g_screen_buffer_size {
        g_screen_buffer_size = size as size_t;
        free(g_screen_buffer as *mut ::core::ffi::c_void);
        g_screen_buffer = malloc((size * 4 as ::core::ffi::c_int) as size_t) as *mut uint8;
    }
    g_draw_width = width;
    g_draw_height = height;
    *pixels = g_screen_buffer;
    *pitch = width * 4 as ::core::ffi::c_int;
}
unsafe extern "C" fn OpenGLRenderer_EndDraw() {
    let mut drawable_width: ::core::ffi::c_int = 0;
    let mut drawable_height: ::core::ffi::c_int = 0;
    SDL_GL_GetDrawableSize(
        g_window as *mut SDL_Window,
        &raw mut drawable_width,
        &raw mut drawable_height,
    );
    let mut viewport_width: ::core::ffi::c_int = drawable_width;
    let mut viewport_height: ::core::ffi::c_int = drawable_height;
    if !g_config.ignore_aspect_ratio {
        if viewport_width * g_draw_height < viewport_height * g_draw_width {
            viewport_height = viewport_width * g_draw_height / g_draw_width;
        } else {
            viewport_width = viewport_height * g_draw_width / g_draw_height;
        }
    }
    let mut viewport_x: ::core::ffi::c_int =
        drawable_width - viewport_width >> 1 as ::core::ffi::c_int;
    let mut viewport_y: ::core::ffi::c_int =
        viewport_height - viewport_height >> 1 as ::core::ffi::c_int;
    _ptrc_glBindTexture.expect("non-null function pointer")(
        GL_TEXTURE_2D as GLenum,
        g_texture.gl_texture as GLuint,
    );
    if g_draw_width == g_texture.width as ::core::ffi::c_int
        && g_draw_height == g_texture.height as ::core::ffi::c_int
    {
        _ptrc_glTexSubImage2D.expect("non-null function pointer")(
            GL_TEXTURE_2D as GLenum,
            0 as GLint,
            0 as GLint,
            0 as GLint,
            g_draw_width as GLsizei,
            g_draw_height as GLsizei,
            GL_BGRA as GLenum,
            GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
            g_screen_buffer as *const ::core::ffi::c_void,
        );
    } else {
        g_texture.width = g_draw_width as uint16;
        g_texture.height = g_draw_height as uint16;
        _ptrc_glTexImage2D.expect("non-null function pointer")(
            GL_TEXTURE_2D as GLenum,
            0 as GLint,
            GL_RGBA,
            g_draw_width as GLsizei,
            g_draw_height as GLsizei,
            0 as GLint,
            GL_BGRA as GLenum,
            GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
            g_screen_buffer as *const ::core::ffi::c_void,
        );
    }
    _ptrc_glClearColor.expect("non-null function pointer")(0.0f32, 0.0f32, 0.0f32, 1.0f32);
    _ptrc_glClear.expect("non-null function pointer")(GL_COLOR_BUFFER_BIT as GLbitfield);
    if g_glsl_shader.is_null() {
        _ptrc_glViewport.expect("non-null function pointer")(
            viewport_x as GLint,
            viewport_y as GLint,
            viewport_width as GLsizei,
            viewport_height as GLsizei,
        );
        _ptrc_glUseProgram.expect("non-null function pointer")(g_program as GLuint);
        let mut filter: ::core::ffi::c_int = if g_config.linear_filtering as ::core::ffi::c_int != 0
        {
            GL_LINEAR
        } else {
            GL_NEAREST
        };
        _ptrc_glTexParameteri.expect("non-null function pointer")(
            GL_TEXTURE_2D as GLenum,
            GL_TEXTURE_MIN_FILTER as GLenum,
            filter as GLint,
        );
        _ptrc_glTexParameteri.expect("non-null function pointer")(
            GL_TEXTURE_2D as GLenum,
            GL_TEXTURE_MAG_FILTER as GLenum,
            filter as GLint,
        );
        _ptrc_glBindVertexArray.expect("non-null function pointer")(g_VAO as GLuint);
        _ptrc_glDrawArrays.expect("non-null function pointer")(
            GL_TRIANGLE_STRIP as GLenum,
            0 as GLint,
            4 as GLsizei,
        );
    } else {
        GlslShader_Render(
            g_glsl_shader,
            &raw mut g_texture,
            viewport_x,
            viewport_y,
            viewport_width,
            viewport_height,
        );
    }
    SDL_GL_SwapWindow(g_window as *mut SDL_Window);
}
static mut kOpenGLRendererFuncs: RendererFuncs = RendererFuncs {
    Initialize: Some(OpenGLRenderer_Init as unsafe extern "C" fn(*mut SDL_Window) -> bool),
    Destroy: Some(OpenGLRenderer_Destroy as unsafe extern "C" fn() -> ()),
    BeginDraw: Some(
        OpenGLRenderer_BeginDraw
            as unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                *mut *mut uint8,
                *mut ::core::ffi::c_int,
            ) -> (),
    ),
    EndDraw: Some(OpenGLRenderer_EndDraw as unsafe extern "C" fn() -> ()),
};
#[no_mangle]
pub unsafe extern "C" fn OpenGLRenderer_Create(mut funcs: *mut RendererFuncs) {
    SDL_GL_SetAttribute(
        SDL_GL_CONTEXT_PROFILE_MASK,
        SDL_GL_CONTEXT_PROFILE_CORE as ::core::ffi::c_int,
    );
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3 as ::core::ffi::c_int);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3 as ::core::ffi::c_int);
    *funcs = kOpenGLRendererFuncs;
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
