use crate::config::g_config;
use crate::host::Die;
use crate::platform::gl_core::{
    _ptrc_glAttachShader, _ptrc_glBindBuffer, _ptrc_glBindTexture, _ptrc_glBindVertexArray,
    _ptrc_glBufferData, _ptrc_glClear, _ptrc_glClearColor, _ptrc_glCompileShader,
    _ptrc_glCreateProgram, _ptrc_glCreateShader, _ptrc_glDebugMessageCallback, _ptrc_glDrawArrays,
    _ptrc_glEnable, _ptrc_glEnableVertexAttribArray, _ptrc_glGenBuffers, _ptrc_glGenTextures,
    _ptrc_glGenVertexArrays, _ptrc_glGetProgramInfoLog, _ptrc_glGetProgramiv,
    _ptrc_glGetShaderInfoLog, _ptrc_glGetShaderiv, _ptrc_glLinkProgram, _ptrc_glShaderSource,
    _ptrc_glTexImage2D, _ptrc_glTexParameteri, _ptrc_glTexSubImage2D, _ptrc_glUseProgram,
    _ptrc_glVertexAttribPointer, _ptrc_glViewport, ogl_IsVersionGEQ, ogl_LoadFunctions,
};
use crate::platform::glsl_shader::{GlslShader_CreateFromFile, GlslShader_Render};
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data, SDL_Window};
extern "C" {

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

}
pub use crate::types::{
    _IO_lock_t, __off64_t, __off_t, __uint16_t, __uint32_t, __uint8_t, ptrdiff_t, size_t, uint,
    uint16_t, uint32_t, uint8_t, GLbitfield, GLboolean, GLchar, GLenum, GLfloat, GLint, GLsizei,
    GLsizeiptr, GLuint, SDL_GLContext, SDL_GLattr, _IO_FILE, FILE, GLDEBUGPROC,
};
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
pub use crate::types::{
    uint16, uint32, uint8, Config, GlTextureWithSize, GlslParam, GlslPass, GlslShader, GlslTexture,
    GlslTextureUniform, GlslUniforms, RendererFuncs,
};
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
unsafe fn OpenGLRenderer_Init(mut window: *mut SDL_Window) -> bool {
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
unsafe fn OpenGLRenderer_Destroy() {}
unsafe fn OpenGLRenderer_BeginDraw(
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
unsafe fn OpenGLRenderer_EndDraw() {
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
    Initialize: Some(OpenGLRenderer_Init as unsafe fn(*mut SDL_Window) -> bool),
    Destroy: Some(OpenGLRenderer_Destroy as unsafe fn() -> ()),
    BeginDraw: Some(
        OpenGLRenderer_BeginDraw
            as unsafe fn(
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                *mut *mut uint8,
                *mut ::core::ffi::c_int,
            ) -> (),
    ),
    EndDraw: Some(OpenGLRenderer_EndDraw as unsafe fn() -> ()),
};
pub unsafe fn OpenGLRenderer_Create(mut funcs: *mut RendererFuncs) {
    SDL_GL_SetAttribute(
        SDL_GL_CONTEXT_PROFILE_MASK,
        SDL_GL_CONTEXT_PROFILE_CORE as ::core::ffi::c_int,
    );
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3 as ::core::ffi::c_int);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3 as ::core::ffi::c_int);
    *funcs = kOpenGLRendererFuncs;
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
