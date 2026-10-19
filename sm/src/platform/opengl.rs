use crate::config::g_config;
use crate::game_state::Game;
use crate::host::Die;
use crate::platform::glsl_shader::{GlslShader_CreateFromFile, GlslShader_Render};
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data, SDL_Window};
use ::core::ffi::{c_char, c_float, c_void};
use ::core::mem::size_of;
use ::core::ptr::{null, null_mut};
pub use sdl2_sys::{
    SDL_GL_CreateContext, SDL_GL_GetDrawableSize, SDL_GL_SetAttribute, SDL_GL_SetSwapInterval,
    SDL_GL_SwapWindow, SDL_GLattr, SDL_GameController, SDL_Renderer, SDL_SysWMmsg, SDL_Texture,
    SDL_mutex,
};
extern "C" {

    static mut stderr: *mut FILE;
    fn fprintf(__stream: *mut FILE, __format: *const c_char, ...) -> i32;
    fn printf(__format: *const c_char, ...) -> i32;
    fn malloc(__size: size_t) -> *mut c_void;
    fn free(__ptr: *mut c_void);

}
pub use crate::types::{
    _IO_lock_t, __off64_t, __off_t, ptrdiff_t, size_t, GLbitfield, GLboolean, GLchar, GLenum,
    GLfloat, GLint, GLsizei, GLsizeiptr, GLuint, SDL_GLContext, _IO_FILE, FILE, GLDEBUGPROC,
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
pub type C2RustUnnamed = u32;
pub const SDL_GL_CONTEXT_PROFILE_ES: C2RustUnnamed =
    sdl2_sys::SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_ES as C2RustUnnamed;
pub const SDL_GL_CONTEXT_PROFILE_COMPATIBILITY: C2RustUnnamed =
    sdl2_sys::SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_COMPATIBILITY as C2RustUnnamed;
pub const SDL_GL_CONTEXT_PROFILE_CORE: C2RustUnnamed =
    sdl2_sys::SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_CORE as C2RustUnnamed;
pub use crate::types::{
    Config, GlTextureWithSize, GlslParam, GlslPass, GlslShader, GlslTexture, GlslTextureUniform,
    GlslUniforms, RendererFuncs,
};
pub const true_0: i32 = 1;
pub const GL_TEXTURE_2D: i32 = gl::TEXTURE_2D as i32;
pub const GL_DEBUG_OUTPUT: i32 = gl::DEBUG_OUTPUT as i32;
pub const GL_DEBUG_OUTPUT_SYNCHRONOUS: i32 = gl::DEBUG_OUTPUT_SYNCHRONOUS as i32;
pub const GL_DEBUG_TYPE_ERROR: i32 = gl::DEBUG_TYPE_ERROR as i32;
pub const GL_DEBUG_TYPE_OTHER: i32 = gl::DEBUG_TYPE_OTHER as i32;
pub const GL_COLOR_BUFFER_BIT: i32 = gl::COLOR_BUFFER_BIT as i32;
pub const GL_FALSE: i32 = gl::FALSE as i32;
pub const GL_FLOAT: i32 = gl::FLOAT as i32;
pub const GL_LINEAR: i32 = gl::LINEAR as i32;
pub const GL_NEAREST: i32 = gl::NEAREST as i32;
pub const GL_RGBA: i32 = gl::RGBA as i32;
pub const GL_TEXTURE_MAG_FILTER: i32 = gl::TEXTURE_MAG_FILTER as i32;
pub const GL_TEXTURE_MIN_FILTER: i32 = gl::TEXTURE_MIN_FILTER as i32;
pub const GL_TRIANGLE_STRIP: i32 = gl::TRIANGLE_STRIP as i32;
pub const GL_BGRA: i32 = gl::BGRA as i32;
pub const GL_UNSIGNED_INT_8_8_8_8_REV: i32 = gl::UNSIGNED_INT_8_8_8_8_REV as i32;
pub const GL_ARRAY_BUFFER: i32 = gl::ARRAY_BUFFER as i32;
pub const GL_STATIC_DRAW: i32 = gl::STATIC_DRAW as i32;
pub const GL_COMPILE_STATUS: i32 = gl::COMPILE_STATUS as i32;
pub const GL_FRAGMENT_SHADER: i32 = gl::FRAGMENT_SHADER as i32;
pub const GL_LINK_STATUS: i32 = gl::LINK_STATUS as i32;
pub const GL_VERTEX_SHADER: i32 = gl::VERTEX_SHADER as i32;
pub const kDebugFlag: i32 = 0;
static mut g_window: *mut SDL_Window = null::<SDL_Window>() as *mut SDL_Window;
static mut g_screen_buffer: *mut u8 = null::<u8>() as *mut u8;
static mut g_screen_buffer_size: size_t = 0;
static mut g_draw_width: i32 = 0;
static mut g_draw_height: i32 = 0;
static mut g_program: u32 = 0;
static mut g_VAO: u32 = 0;
static mut g_texture: GlTextureWithSize = GlTextureWithSize {
    gl_texture: 0,
    width: 0,
    height: 0,
};
static mut g_glsl_shader: *mut GlslShader = null::<GlslShader>() as *mut GlslShader;
unsafe extern "C" fn MessageCallback(
    source: GLenum,
    type_0: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    userParam: *const c_void,
) {
    if type_0 == GL_DEBUG_TYPE_OTHER as GLenum {
        return;
    }
    fprintf(
        stderr,
        b"GL CALLBACK: %s type = 0x%x, severity = 0x%x, message = %s\n\0" as *const u8
            as *const c_char,
        if type_0 == GL_DEBUG_TYPE_ERROR as GLenum {
            b"** GL ERROR **\0" as *const u8 as *const c_char
        } else {
            b"\0" as *const u8 as *const c_char
        },
        type_0,
        severity,
        message,
    );
    if type_0 == GL_DEBUG_TYPE_ERROR as GLenum {
        Die(b"OpenGL error!\n\0" as *const u8 as *const c_char);
    }
}
unsafe fn OpenGLRenderer_Init(window: *mut SDL_Window, game: &mut Game) -> bool {
    g_window = window;
    let context: SDL_GLContext = SDL_GL_CreateContext(window as *mut SDL_Window);
    SDL_GL_SetSwapInterval(1i32);
    gl::load_with(|name| {
        sdl2_sys::SDL_GL_GetProcAddress(std::ffi::CString::new(name).expect("GL symbol").as_ptr())
            as *const c_void
    });
    let mut gl_major = 0;
    let mut gl_minor = 0;
    gl::GetIntegerv(gl::MAJOR_VERSION, &raw mut gl_major);
    gl::GetIntegerv(gl::MINOR_VERSION, &raw mut gl_minor);
    if (gl_major, gl_minor) < (3, 3) {
        Die(b"You need OpenGL 3.3\0" as *const u8 as *const c_char);
    }
    gl::GenTextures(1 as GLsizei, &raw mut g_texture.gl_texture);
    static mut kVertices: [c_float; 20] = [
        -1.0f32, 1.0f32, 0.0f32, 0.0f32, 0.0f32, -1.0f32, -1.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32,
        1.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, -1.0f32, 0.0f32, 1.0f32, 1.0f32,
    ];
    let mut vbo: u32 = 0;
    gl::GenBuffers(1 as GLsizei, &raw mut vbo);
    gl::GenVertexArrays(1 as GLsizei, &raw mut g_VAO);
    gl::BindVertexArray(g_VAO as GLuint);
    gl::BindBuffer(GL_ARRAY_BUFFER as GLenum, vbo as GLuint);
    gl::BufferData(
        GL_ARRAY_BUFFER as GLenum,
        size_of::<[c_float; 20]>() as GLsizeiptr,
        &raw const kVertices as *const c_float as *const c_void,
        GL_STATIC_DRAW as GLenum,
    );
    gl::VertexAttribPointer(
        0 as GLuint,
        3 as GLint,
        GL_FLOAT as GLenum,
        GL_FALSE as GLboolean,
        5_usize.wrapping_mul(size_of::<c_float>() as usize) as GLsizei,
        null::<c_void>(),
    );
    gl::EnableVertexAttribArray(0 as GLuint);
    gl::VertexAttribPointer(
        1 as GLuint,
        2 as GLint,
        GL_FLOAT as GLenum,
        GL_FALSE as GLboolean,
        5_usize.wrapping_mul(size_of::<c_float>() as usize) as GLsizei,
        3_usize.wrapping_mul(size_of::<c_float>() as usize) as *mut c_void,
    );
    gl::EnableVertexAttribArray(1 as GLuint);
    let mut vs_code: *const GLchar = b"#version 330 core\nlayout(location = 0) in vec3 aPos; layout(location = 1) in vec2 aTexCoord; out vec2 TexCoord; void main(void) { gl_Position = vec4(aPos, 1.0); TexCoord = vec2(aTexCoord.x, aTexCoord.y); }\0"
        as *const u8 as *const GLchar;
    let vs: u32 = gl::CreateShader(GL_VERTEX_SHADER as GLenum) as u32;
    gl::ShaderSource(
        vs as GLuint,
        1 as GLsizei,
        &raw mut vs_code,
        null::<GLint>(),
    );
    gl::CompileShader(vs as GLuint);
    let mut success: i32 = 0;
    let mut infolog: [c_char; 512] = [0; 512];
    gl::GetShaderiv(vs as GLuint, GL_COMPILE_STATUS as GLenum, &raw mut success);
    if success == 0 {
        gl::GetShaderInfoLog(
            vs as GLuint,
            512 as GLsizei,
            null_mut::<GLsizei>(),
            &raw mut infolog as *mut GLchar,
        );
        printf(
            b"%s\n\0" as *const u8 as *const c_char,
            &raw mut infolog as *mut c_char,
        );
    }
    let mut fs_code: *const GLchar = b"#version 330 core\nout vec4 FragColor; in vec2 TexCoord; uniform sampler2D texture1; void main(void) { FragColor = texture(texture1, TexCoord); }\0"
        as *const u8 as *const GLchar;
    let fs: u32 = gl::CreateShader(GL_FRAGMENT_SHADER as GLenum) as u32;
    gl::ShaderSource(
        fs as GLuint,
        1 as GLsizei,
        &raw mut fs_code,
        null::<GLint>(),
    );
    gl::CompileShader(fs as GLuint);
    gl::GetShaderiv(fs as GLuint, GL_COMPILE_STATUS as GLenum, &raw mut success);
    if success == 0 {
        gl::GetShaderInfoLog(
            fs as GLuint,
            512 as GLsizei,
            null_mut::<GLsizei>(),
            &raw mut infolog as *mut GLchar,
        );
        printf(
            b"%s\n\0" as *const u8 as *const c_char,
            &raw mut infolog as *mut c_char,
        );
    }
    g_program = gl::CreateProgram() as u32;
    let program: i32 = g_program as i32;
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
            null_mut::<GLsizei>(),
            &raw mut infolog as *mut GLchar,
        );
        printf(
            b"%s\n\0" as *const u8 as *const c_char,
            &raw mut infolog as *mut c_char,
        );
    }
    if !g_config.shader.is_null() {
        g_glsl_shader = GlslShader_CreateFromFile(g_config.shader);
    }
    true_0 != 0
}
unsafe fn OpenGLRenderer_Destroy(game: &mut Game) {}
unsafe fn OpenGLRenderer_BeginDraw(
    width: i32,
    height: i32,
    pixels: *mut *mut u8,
    pitch: *mut i32,
    game: &mut Game,
) {
    let size: i32 = width * height;
    if size as size_t > g_screen_buffer_size {
        g_screen_buffer_size = size as size_t;
        free(g_screen_buffer as *mut c_void);
        g_screen_buffer = malloc((size * 4i32) as size_t) as *mut u8;
    }
    g_draw_width = width;
    g_draw_height = height;
    *pixels = g_screen_buffer;
    *pitch = width * 4i32;
}
unsafe fn OpenGLRenderer_EndDraw(game: &mut Game) {
    let mut drawable_width: i32 = 0;
    let mut drawable_height: i32 = 0;
    SDL_GL_GetDrawableSize(
        g_window as *mut SDL_Window,
        &raw mut drawable_width,
        &raw mut drawable_height,
    );
    let mut viewport_width: i32 = drawable_width;
    let mut viewport_height: i32 = drawable_height;
    if !g_config.ignore_aspect_ratio {
        if viewport_width * g_draw_height < viewport_height * g_draw_width {
            viewport_height = viewport_width * g_draw_height / g_draw_width;
        } else {
            viewport_width = viewport_height * g_draw_width / g_draw_height;
        }
    }
    let viewport_x: i32 = drawable_width - viewport_width >> 1i32;
    let viewport_y: i32 = viewport_height - viewport_height >> 1i32;
    gl::BindTexture(GL_TEXTURE_2D as GLenum, g_texture.gl_texture as GLuint);
    if g_draw_width == g_texture.width as i32 && g_draw_height == g_texture.height as i32 {
        gl::TexSubImage2D(
            GL_TEXTURE_2D as GLenum,
            0 as GLint,
            0 as GLint,
            0 as GLint,
            g_draw_width as GLsizei,
            g_draw_height as GLsizei,
            GL_BGRA as GLenum,
            GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
            g_screen_buffer as *const c_void,
        );
    } else {
        g_texture.width = g_draw_width as u16;
        g_texture.height = g_draw_height as u16;
        gl::TexImage2D(
            GL_TEXTURE_2D as GLenum,
            0 as GLint,
            GL_RGBA,
            g_draw_width as GLsizei,
            g_draw_height as GLsizei,
            0 as GLint,
            GL_BGRA as GLenum,
            GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
            g_screen_buffer as *const c_void,
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
        let filter: i32 = if g_config.linear_filtering as i32 != 0 {
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
        OpenGLRenderer_BeginDraw as unsafe fn(i32, i32, *mut *mut u8, *mut i32, &mut Game) -> (),
    ),
    EndDraw: Some(OpenGLRenderer_EndDraw as unsafe fn(&mut Game) -> ()),
};
pub unsafe fn OpenGLRenderer_Create(funcs: *mut RendererFuncs) {
    SDL_GL_SetAttribute(
        SDL_GL_CONTEXT_PROFILE_MASK,
        SDL_GL_CONTEXT_PROFILE_CORE as i32,
    );
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3i32);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3i32);
    *funcs = kOpenGLRendererFuncs;
}
pub const NULL: *mut c_void = null_mut::<c_void>();
