use crate::config::g_config;
use crate::game_state::Game;
use crate::host::Die;
use crate::platform::glsl_shader::{GlslShader_CreateFromFile, GlslShader_Render};
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data, SDL_Window};
pub use sdl2_sys::{
    SDL_GL_CreateContext, SDL_GL_GetDrawableSize, SDL_GL_SetAttribute, SDL_GL_SetSwapInterval,
    SDL_GL_SwapWindow, SDL_GLattr, SDL_GameController, SDL_Renderer, SDL_SysWMmsg, SDL_Texture,
    SDL_mutex,
};
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

}
pub use crate::types::{
    _IO_lock_t, __off64_t, __off_t, __uint16_t, __uint32_t, __uint8_t, ptrdiff_t, size_t, uint,
    uint16_t, uint32_t, uint8_t, GLbitfield, GLboolean, GLchar, GLenum, GLfloat, GLint, GLsizei,
    GLsizeiptr, GLuint, SDL_GLContext, _IO_FILE, FILE, GLDEBUGPROC,
};
pub const SDL_GL_FLOATBUFFERS: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_FLOATBUFFERS;
pub const SDL_GL_CONTEXT_NO_ERROR: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_NO_ERROR;
pub const SDL_GL_CONTEXT_RESET_NOTIFICATION: SDL_GLattr =
    sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_RESET_NOTIFICATION;
pub const SDL_GL_CONTEXT_RELEASE_BEHAVIOR: SDL_GLattr =
    sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_RELEASE_BEHAVIOR;
pub const SDL_GL_FRAMEBUFFER_SRGB_CAPABLE: SDL_GLattr =
    sdl2_sys::SDL_GLattr::SDL_GL_FRAMEBUFFER_SRGB_CAPABLE;
pub const SDL_GL_SHARE_WITH_CURRENT_CONTEXT: SDL_GLattr =
    sdl2_sys::SDL_GLattr::SDL_GL_SHARE_WITH_CURRENT_CONTEXT;
pub const SDL_GL_CONTEXT_PROFILE_MASK: SDL_GLattr =
    sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_PROFILE_MASK;
pub const SDL_GL_CONTEXT_FLAGS: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_FLAGS;
pub const SDL_GL_CONTEXT_EGL: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_EGL;
pub const SDL_GL_CONTEXT_MINOR_VERSION: SDL_GLattr =
    sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_MINOR_VERSION;
pub const SDL_GL_CONTEXT_MAJOR_VERSION: SDL_GLattr =
    sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_MAJOR_VERSION;
pub const SDL_GL_RETAINED_BACKING: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_RETAINED_BACKING;
pub const SDL_GL_ACCELERATED_VISUAL: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_ACCELERATED_VISUAL;
pub const SDL_GL_MULTISAMPLESAMPLES: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_MULTISAMPLESAMPLES;
pub const SDL_GL_MULTISAMPLEBUFFERS: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_MULTISAMPLEBUFFERS;
pub const SDL_GL_STEREO: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_STEREO;
pub const SDL_GL_ACCUM_ALPHA_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_ACCUM_ALPHA_SIZE;
pub const SDL_GL_ACCUM_BLUE_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_ACCUM_BLUE_SIZE;
pub const SDL_GL_ACCUM_GREEN_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_ACCUM_GREEN_SIZE;
pub const SDL_GL_ACCUM_RED_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_ACCUM_RED_SIZE;
pub const SDL_GL_STENCIL_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_STENCIL_SIZE;
pub const SDL_GL_DEPTH_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_DEPTH_SIZE;
pub const SDL_GL_DOUBLEBUFFER: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_DOUBLEBUFFER;
pub const SDL_GL_BUFFER_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_BUFFER_SIZE;
pub const SDL_GL_ALPHA_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_ALPHA_SIZE;
pub const SDL_GL_BLUE_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_BLUE_SIZE;
pub const SDL_GL_GREEN_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_GREEN_SIZE;
pub const SDL_GL_RED_SIZE: SDL_GLattr = sdl2_sys::SDL_GLattr::SDL_GL_RED_SIZE;
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const SDL_GL_CONTEXT_PROFILE_ES: C2RustUnnamed =
    sdl2_sys::SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_ES as C2RustUnnamed;
pub const SDL_GL_CONTEXT_PROFILE_COMPATIBILITY: C2RustUnnamed =
    sdl2_sys::SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_COMPATIBILITY as C2RustUnnamed;
pub const SDL_GL_CONTEXT_PROFILE_CORE: C2RustUnnamed =
    sdl2_sys::SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_CORE as C2RustUnnamed;
pub use crate::types::{
    uint16, uint32, uint8, Config, GlTextureWithSize, GlslParam, GlslPass, GlslShader, GlslTexture,
    GlslTextureUniform, GlslUniforms, RendererFuncs,
};
pub const true_0: ::core::ffi::c_int = 1;
pub const GL_TEXTURE_2D: ::core::ffi::c_int = gl::TEXTURE_2D as ::core::ffi::c_int;
pub const GL_DEBUG_OUTPUT: ::core::ffi::c_int = gl::DEBUG_OUTPUT as ::core::ffi::c_int;
pub const GL_DEBUG_OUTPUT_SYNCHRONOUS: ::core::ffi::c_int =
    gl::DEBUG_OUTPUT_SYNCHRONOUS as ::core::ffi::c_int;
pub const GL_DEBUG_TYPE_ERROR: ::core::ffi::c_int = gl::DEBUG_TYPE_ERROR as ::core::ffi::c_int;
pub const GL_DEBUG_TYPE_OTHER: ::core::ffi::c_int = gl::DEBUG_TYPE_OTHER as ::core::ffi::c_int;
pub const GL_COLOR_BUFFER_BIT: ::core::ffi::c_int = gl::COLOR_BUFFER_BIT as ::core::ffi::c_int;
pub const GL_FALSE: ::core::ffi::c_int = gl::FALSE as ::core::ffi::c_int;
pub const GL_FLOAT: ::core::ffi::c_int = gl::FLOAT as ::core::ffi::c_int;
pub const GL_LINEAR: ::core::ffi::c_int = gl::LINEAR as ::core::ffi::c_int;
pub const GL_NEAREST: ::core::ffi::c_int = gl::NEAREST as ::core::ffi::c_int;
pub const GL_RGBA: ::core::ffi::c_int = gl::RGBA as ::core::ffi::c_int;
pub const GL_TEXTURE_MAG_FILTER: ::core::ffi::c_int = gl::TEXTURE_MAG_FILTER as ::core::ffi::c_int;
pub const GL_TEXTURE_MIN_FILTER: ::core::ffi::c_int = gl::TEXTURE_MIN_FILTER as ::core::ffi::c_int;
pub const GL_TRIANGLE_STRIP: ::core::ffi::c_int = gl::TRIANGLE_STRIP as ::core::ffi::c_int;
pub const GL_BGRA: ::core::ffi::c_int = gl::BGRA as ::core::ffi::c_int;
pub const GL_UNSIGNED_INT_8_8_8_8_REV: ::core::ffi::c_int =
    gl::UNSIGNED_INT_8_8_8_8_REV as ::core::ffi::c_int;
pub const GL_ARRAY_BUFFER: ::core::ffi::c_int = gl::ARRAY_BUFFER as ::core::ffi::c_int;
pub const GL_STATIC_DRAW: ::core::ffi::c_int = gl::STATIC_DRAW as ::core::ffi::c_int;
pub const GL_COMPILE_STATUS: ::core::ffi::c_int = gl::COMPILE_STATUS as ::core::ffi::c_int;
pub const GL_FRAGMENT_SHADER: ::core::ffi::c_int = gl::FRAGMENT_SHADER as ::core::ffi::c_int;
pub const GL_LINK_STATUS: ::core::ffi::c_int = gl::LINK_STATUS as ::core::ffi::c_int;
pub const GL_VERTEX_SHADER: ::core::ffi::c_int = gl::VERTEX_SHADER as ::core::ffi::c_int;
pub const kDebugFlag: ::core::ffi::c_int = 0;
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
unsafe fn OpenGLRenderer_Init(mut window: *mut SDL_Window, game: &mut Game) -> bool {
    g_window = window;
    let mut context: SDL_GLContext = SDL_GL_CreateContext(window as *mut SDL_Window);
    SDL_GL_SetSwapInterval(1 as ::core::ffi::c_int);
    gl::load_with(|name| {
        sdl2_sys::SDL_GL_GetProcAddress(std::ffi::CString::new(name).expect("GL symbol").as_ptr())
            as *const ::core::ffi::c_void
    });
    let mut gl_major = 0;
    let mut gl_minor = 0;
    gl::GetIntegerv(gl::MAJOR_VERSION, &raw mut gl_major);
    gl::GetIntegerv(gl::MINOR_VERSION, &raw mut gl_minor);
    if (gl_major, gl_minor) < (3, 3) {
        Die(b"You need OpenGL 3.3\0" as *const u8 as *const ::core::ffi::c_char);
    }
    gl::GenTextures(1 as GLsizei, &raw mut g_texture.gl_texture);
    static mut kVertices: [::core::ffi::c_float; 20] = [
        -1.0f32, 1.0f32, 0.0f32, 0.0f32, 0.0f32, -1.0f32, -1.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32,
        1.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, -1.0f32, 0.0f32, 1.0f32, 1.0f32,
    ];
    let mut vbo: ::core::ffi::c_uint = 0;
    gl::GenBuffers(1 as GLsizei, &raw mut vbo);
    gl::GenVertexArrays(1 as GLsizei, &raw mut g_VAO);
    gl::BindVertexArray(g_VAO as GLuint);
    gl::BindBuffer(GL_ARRAY_BUFFER as GLenum, vbo as GLuint);
    gl::BufferData(
        GL_ARRAY_BUFFER as GLenum,
        ::core::mem::size_of::<[::core::ffi::c_float; 20]>() as GLsizeiptr,
        &raw const kVertices as *const ::core::ffi::c_float as *const ::core::ffi::c_void,
        GL_STATIC_DRAW as GLenum,
    );
    gl::VertexAttribPointer(
        0 as GLuint,
        3 as GLint,
        GL_FLOAT as GLenum,
        GL_FALSE as GLboolean,
        (5 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_float>() as usize)
            as GLsizei,
        ::core::ptr::null::<::core::ffi::c_void>(),
    );
    gl::EnableVertexAttribArray(0 as GLuint);
    gl::VertexAttribPointer(
        1 as GLuint,
        2 as GLint,
        GL_FLOAT as GLenum,
        GL_FALSE as GLboolean,
        (5 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_float>() as usize)
            as GLsizei,
        (3 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_float>() as usize)
            as *mut ::core::ffi::c_void,
    );
    gl::EnableVertexAttribArray(1 as GLuint);
    let mut vs_code: *const GLchar = b"#version 330 core\nlayout(location = 0) in vec3 aPos; layout(location = 1) in vec2 aTexCoord; out vec2 TexCoord; void main(void) { gl_Position = vec4(aPos, 1.0); TexCoord = vec2(aTexCoord.x, aTexCoord.y); }\0"
        as *const u8 as *const GLchar;
    let mut vs: ::core::ffi::c_uint =
        gl::CreateShader(GL_VERTEX_SHADER as GLenum) as ::core::ffi::c_uint;
    gl::ShaderSource(
        vs as GLuint,
        1 as GLsizei,
        &raw mut vs_code,
        ::core::ptr::null::<GLint>(),
    );
    gl::CompileShader(vs as GLuint);
    let mut success: ::core::ffi::c_int = 0;
    let mut infolog: [::core::ffi::c_char; 512] = [0; 512];
    gl::GetShaderiv(vs as GLuint, GL_COMPILE_STATUS as GLenum, &raw mut success);
    if success == 0 {
        gl::GetShaderInfoLog(
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
        gl::CreateShader(GL_FRAGMENT_SHADER as GLenum) as ::core::ffi::c_uint;
    gl::ShaderSource(
        fs as GLuint,
        1 as GLsizei,
        &raw mut fs_code,
        ::core::ptr::null::<GLint>(),
    );
    gl::CompileShader(fs as GLuint);
    gl::GetShaderiv(fs as GLuint, GL_COMPILE_STATUS as GLenum, &raw mut success);
    if success == 0 {
        gl::GetShaderInfoLog(
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
    g_program = gl::CreateProgram() as ::core::ffi::c_uint;
    let mut program: ::core::ffi::c_int = g_program as ::core::ffi::c_int;
    gl::AttachShader(program as GLuint, vs as GLuint);
    gl::AttachShader(program as GLuint, fs as GLuint);
    gl::LinkProgram(program as GLuint);
    gl::GetProgramiv(
        program as GLuint,
        GL_LINK_STATUS as GLenum,
        &raw mut success,
    );
    if success == 0 {
        gl::GetProgramInfoLog(
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
    true_0 != 0
}
unsafe fn OpenGLRenderer_Destroy(game: &mut Game) {}
unsafe fn OpenGLRenderer_BeginDraw(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
    mut pixels: *mut *mut uint8,
    mut pitch: *mut ::core::ffi::c_int,
    game: &mut Game,
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
unsafe fn OpenGLRenderer_EndDraw(game: &mut Game) {
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
    gl::BindTexture(GL_TEXTURE_2D as GLenum, g_texture.gl_texture as GLuint);
    if g_draw_width == g_texture.width as ::core::ffi::c_int
        && g_draw_height == g_texture.height as ::core::ffi::c_int
    {
        gl::TexSubImage2D(
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
        gl::TexImage2D(
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
    gl::ClearColor(0.0f32, 0.0f32, 0.0f32, 1.0f32);
    gl::Clear(GL_COLOR_BUFFER_BIT as GLbitfield);
    if g_glsl_shader.is_null() {
        gl::Viewport(
            viewport_x as GLint,
            viewport_y as GLint,
            viewport_width as GLsizei,
            viewport_height as GLsizei,
        );
        gl::UseProgram(g_program as GLuint);
        let mut filter: ::core::ffi::c_int = if g_config.linear_filtering as ::core::ffi::c_int != 0
        {
            GL_LINEAR
        } else {
            GL_NEAREST
        };
        gl::TexParameteri(
            GL_TEXTURE_2D as GLenum,
            GL_TEXTURE_MIN_FILTER as GLenum,
            filter as GLint,
        );
        gl::TexParameteri(
            GL_TEXTURE_2D as GLenum,
            GL_TEXTURE_MAG_FILTER as GLenum,
            filter as GLint,
        );
        gl::BindVertexArray(g_VAO as GLuint);
        gl::DrawArrays(GL_TRIANGLE_STRIP as GLenum, 0 as GLint, 4 as GLsizei);
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
    Initialize: Some(OpenGLRenderer_Init as unsafe fn(*mut SDL_Window, &mut Game) -> bool),
    Destroy: Some(OpenGLRenderer_Destroy as unsafe fn(&mut Game) -> ()),
    BeginDraw: Some(
        OpenGLRenderer_BeginDraw
            as unsafe fn(
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                *mut *mut uint8,
                *mut ::core::ffi::c_int,
                &mut Game,
            ) -> (),
    ),
    EndDraw: Some(OpenGLRenderer_EndDraw as unsafe fn(&mut Game) -> ()),
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
