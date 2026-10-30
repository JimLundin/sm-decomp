use crate::config::g_config;
use crate::platform::glsl_shader::{GlslShader, GlslShader_CreateFromFile, GlslShader_Render};
use sdl2_sys::SDL_GLattr::{
    SDL_GL_CONTEXT_MAJOR_VERSION, SDL_GL_CONTEXT_MINOR_VERSION, SDL_GL_CONTEXT_PROFILE_MASK,
};
use sdl2_sys::{
    SDL_GL_CreateContext, SDL_GL_GetDrawableSize, SDL_GL_SetAttribute, SDL_GL_SetSwapInterval,
    SDL_GL_SwapWindow,
};
use sm::c_library::printf;
use sm::host::Die;
use sm::types::{GlTextureWithSize, RendererFuncs, SDL_GLContext, SDL_Window};
use std::cell::RefCell;
use std::ffi::{CStr, c_void};
use std::ptr::{null, null_mut};

struct Presenter {
    window: *mut SDL_Window,
    pixels: Vec<u8>,
    width: i32,
    height: i32,
    program: u32,
    vao: u32,
    texture: GlTextureWithSize,
    shader: *mut GlslShader,
}

impl Presenter {
    fn pixels(&mut self) -> *mut u8 {
        if self.pixels.is_empty() {
            null_mut()
        } else {
            self.pixels.as_mut_ptr()
        }
    }
}

thread_local! {
    static PRESENTER: RefCell<Presenter> = RefCell::new(Presenter {
        window: null_mut(),
        pixels: Vec::new(),
        width: 0,
        height: 0,
        program: 0,
        vao: 0,
        texture: GlTextureWithSize {
            gl_texture: 0,
            width: 0,
            height: 0,
        },
        shader: null_mut(),
    });
}

unsafe fn compile_display_shader(stage: u32, source: &CStr) -> u32 {
    unsafe {
        let shader = gl::CreateShader(stage);
        gl::ShaderSource(shader, 1, &source.as_ptr(), null());
        gl::CompileShader(shader);
        let mut status = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status == 0 {
            let mut log = [0; 512];
            gl::GetShaderInfoLog(shader, 512, null_mut(), log.as_mut_ptr());
            printf(c"%s\n".as_ptr(), log.as_ptr());
        }
        shader
    }
}

unsafe extern "C-unwind" fn OpenGLRenderer_Init(window: *mut SDL_Window) -> bool {
    PRESENTER.with(|presenter| {
        let mut state = presenter.borrow_mut();
        unsafe {
            state.window = window;
            let _context: SDL_GLContext = SDL_GL_CreateContext(window);
            SDL_GL_SetSwapInterval(1);
            crate::platform::glsl_shader::load_with(|name| {
                sdl2_sys::SDL_GL_GetProcAddress(
                    std::ffi::CString::new(name).expect("GL symbol").as_ptr(),
                ) as *const c_void
            });
            let mut gl_major = 0;
            let mut gl_minor = 0;
            gl::GetIntegerv(gl::MAJOR_VERSION, &raw mut gl_major);
            gl::GetIntegerv(gl::MINOR_VERSION, &raw mut gl_minor);
            if (gl_major, gl_minor) < (3, 3) {
                Die(c"You need OpenGL 3.3".as_ptr());
            }
            gl::GenTextures(1, &mut state.texture.gl_texture);
            const VERTICES: [f32; 20] = [
                -1.0, 1.0, 0.0, 0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0,
                -1.0, 0.0, 1.0, 1.0,
            ];
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::GenVertexArrays(1, &mut state.vao);
            gl::BindVertexArray(state.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(&VERTICES) as isize,
                VERTICES.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                5 * std::mem::size_of::<f32>() as i32,
                null(),
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                5 * std::mem::size_of::<f32>() as i32,
                (3 * std::mem::size_of::<f32>()) as *const c_void,
            );
            gl::EnableVertexAttribArray(1);
            let vertex = compile_display_shader(gl::VERTEX_SHADER,
            c"#version 330 core\nlayout(location = 0) in vec3 aPos; layout(location = 1) in vec2 aTexCoord; out vec2 TexCoord; void main(void) { gl_Position = vec4(aPos, 1.0); TexCoord = vec2(aTexCoord.x, aTexCoord.y); }");
            let fragment = compile_display_shader(gl::FRAGMENT_SHADER,
            c"#version 330 core\nout vec4 FragColor; in vec2 TexCoord; uniform sampler2D texture1; void main(void) { FragColor = texture(texture1, TexCoord); }");
            state.program = gl::CreateProgram();
            gl::AttachShader(state.program, vertex);
            gl::AttachShader(state.program, fragment);
            gl::LinkProgram(state.program);
            let mut status = 0;
            gl::GetProgramiv(state.program, gl::LINK_STATUS, &mut status);
            if status == 0 {
                let mut log = [0; 512];
                gl::GetProgramInfoLog(state.program, 512, null_mut(), log.as_mut_ptr());
                printf(c"%s\n".as_ptr(), log.as_ptr());
            }
            if !g_config.shader.is_null() {
                state.shader = GlslShader_CreateFromFile(g_config.shader);
            }
            true
        }
    })
}

extern "C-unwind" fn OpenGLRenderer_Destroy() {
    PRESENTER.with(|presenter| presenter.borrow_mut().pixels = Vec::new());
}

unsafe extern "C-unwind" fn OpenGLRenderer_BeginDraw(
    width: i32,
    height: i32,
    pixels: *mut *mut u8,
    pitch: *mut i32,
) {
    PRESENTER.with(|presenter| {
        let mut state = presenter.borrow_mut();
        unsafe {
            let size = width * height;
            if size as usize > state.pixels.len() / 4 {
                state.pixels = Vec::new();
                state.pixels.resize((size * 4) as usize, 0);
            }
            // The frame writer fills 256 columns; clear untouched row tails on reuse too.
            state.pixels[..(size * 4) as usize].fill(0);
            state.width = width;
            state.height = height;
            *pixels = state.pixels();
            *pitch = width * 4;
        }
    })
}

unsafe extern "C-unwind" fn OpenGLRenderer_EndDraw() {
    PRESENTER.with(|presenter| {
        let mut state = presenter.borrow_mut();
        unsafe {
            let mut drawable_width = 0;
            let mut drawable_height = 0;
            SDL_GL_GetDrawableSize(state.window, &mut drawable_width, &mut drawable_height);
            let mut width = drawable_width;
            let mut height = drawable_height;
            if !g_config.ignore_aspect_ratio {
                if width * state.height < height * state.width {
                    height = width * state.height / state.width;
                } else {
                    width = height * state.width / state.height;
                }
            }
            let x = (drawable_width - width) >> 1;
            // Preserve the current top-aligned viewport, including its zero-Y formula.
            let y = (height - height) >> 1;
            gl::BindTexture(gl::TEXTURE_2D, state.texture.gl_texture);
            if state.width == state.texture.width as i32
                && state.height == state.texture.height as i32
            {
                gl::TexSubImage2D(
                    gl::TEXTURE_2D,
                    0,
                    0,
                    0,
                    state.width,
                    state.height,
                    gl::BGRA,
                    gl::UNSIGNED_INT_8_8_8_8_REV,
                    state.pixels().cast(),
                );
            } else {
                state.texture.width = state.width as u16;
                state.texture.height = state.height as u16;
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    state.width,
                    state.height,
                    0,
                    gl::BGRA,
                    gl::UNSIGNED_INT_8_8_8_8_REV,
                    state.pixels().cast(),
                );
            }
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            if state.shader.is_null() {
                gl::Viewport(x, y, width, height);
                gl::UseProgram(state.program);
                let filter = if g_config.linear_filtering {
                    gl::LINEAR
                } else {
                    gl::NEAREST
                };
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
                gl::BindVertexArray(state.vao);
                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            } else {
                GlslShader_Render(state.shader, &mut state.texture, x, y, width, height);
            }
            SDL_GL_SwapWindow(state.window);
        }
    })
}

#[unsafe(export_name = "sm_host_OpenGLRenderer_Create")]
pub unsafe extern "C-unwind" fn OpenGLRenderer_Create(funcs: *mut RendererFuncs) {
    unsafe {
        SDL_GL_SetAttribute(
            SDL_GL_CONTEXT_PROFILE_MASK,
            sdl2_sys::SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_CORE as i32,
        );
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
        *funcs = RendererFuncs {
            Initialize: Some(OpenGLRenderer_Init),
            Destroy: Some(OpenGLRenderer_Destroy),
            BeginDraw: Some(OpenGLRenderer_BeginDraw),
            EndDraw: Some(OpenGLRenderer_EndDraw),
        };
    }
}
