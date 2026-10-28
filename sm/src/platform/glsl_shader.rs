use crate::c_library::strtod;
use crate::types::GlTextureWithSize;
use std::alloc::{Layout, alloc_zeroed, dealloc, handle_alloc_error};
use std::ffi::{CStr, CString, c_char, c_void};
use std::mem::{align_of, size_of};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};

pub enum GlslShader {}

impl Default for GlTextureWithSize {
    fn default() -> Self {
        Self {
            gl_texture: 0,
            width: 0,
            height: 0,
        }
    }
}

#[link(name = "sm_config", kind = "static", modifiers = "-bundle")]
unsafe extern "C-unwind" {
    fn sm_shader_create(filename: *const c_char, opengl_es: bool) -> *mut GlslShader;
    fn sm_shader_render(
        shader: *mut GlslShader,
        texture: *mut GlTextureWithSize,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    );
    fn sm_shader_destroy_gl(shader: *mut GlslShader);
    fn sm_shader_free_heap(shader: *mut GlslShader);
    fn sm_shader_load_body(
        shader: *mut GlslShader,
        filename: *const c_char,
        source: *mut c_void,
    ) -> bool;
    fn sm_shader_matches_source(index: u32, bytes: *const u8, length: usize) -> bool;
}

fn check_dependency() {
    static CHECKED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    CHECKED.get_or_init(|| {
        crate::config::check_dependency();
        for (index, source) in [
            include_bytes!("glsl_shader.rs").as_slice(),
            include_bytes!("../native_shader.c").as_slice(),
            include_bytes!("../native_shader.h").as_slice(),
            include_bytes!("texture_png.rs").as_slice(),
            include_bytes!("opengl.rs").as_slice(),
            include_bytes!("../types.rs").as_slice(),
            include_bytes!("../../../tools/install_native_config.py").as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                unsafe { sm_shader_matches_source(index as u32, source.as_ptr(), source.len()) },
                "stale native shader; rerun tools/install_native_config.py and rebuild"
            );
        }
    });
}

pub unsafe fn GlslShader_CreateFromFile(filename: *const c_char) -> *mut GlslShader {
    unsafe {
        check_dependency();
        sm_shader_create(filename, false)
    }
}

struct HeapOwner(*mut GlslShader);
impl Drop for HeapOwner {
    fn drop(&mut self) {
        unsafe {
            sm_shader_free_heap(self.0);
        }
    }
}

pub unsafe fn GlslShader_Destroy(shader: *mut GlslShader) {
    unsafe {
        let _heap = HeapOwner(shader);
        sm_shader_destroy_gl(shader);
    }
}

pub unsafe fn GlslShader_Render(
    shader: *mut GlslShader,
    texture: *mut GlTextureWithSize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    unsafe {
        check_dependency();
        sm_shader_render(shader, texture, x, y, width, height);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_destroy(shader: *mut GlslShader) {
    unsafe {
        GlslShader_Destroy(shader);
    }
}

// The root stays raw during creation. This local source owner drops before an
// ordinary failure destroys the root, and drops while the raw root leaks on panic.
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_load_resources(
    shader: *mut GlslShader,
    filename: *const c_char,
) -> bool {
    unsafe {
        let mut source = Vec::<u8>::new();
        sm_shader_load_body(shader, filename, (&raw mut source).cast())
    }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_source_clear(source: *mut c_void) {
    unsafe {
        (*source.cast::<Vec<u8>>()).clear();
    }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_source_data(source: *mut c_void) -> *const u8 {
    unsafe { (*source.cast::<Vec<u8>>()).as_ptr() }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_source_length(source: *mut c_void) -> usize {
    unsafe { (*source.cast::<Vec<u8>>()).len() }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_append_line(
    source: *mut c_void,
    line: *const u8,
    length: usize,
) {
    unsafe {
        let output = &mut *source.cast::<Vec<u8>>();
        if output.try_reserve(length + 1).is_err() {
            crate::host::Die(c"memory allocation failed".as_ptr());
        }
        output.extend_from_slice(std::slice::from_raw_parts(line, length));
        output.push(b'\n');
    }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_allocate(
    size: usize,
    alignment: usize,
    nullable: bool,
) -> *mut c_void {
    unsafe {
        let layout = Layout::from_size_align(size, alignment).unwrap();
        let allocation = alloc_zeroed(layout);
        if allocation.is_null() && !nullable {
            handle_alloc_error(layout);
        }
        allocation.cast()
    }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_deallocate(
    pointer: *mut c_void,
    size: usize,
    alignment: usize,
) {
    unsafe {
        if !pointer.is_null() {
            dealloc(
                pointer.cast(),
                Layout::from_size_align(size, alignment).unwrap(),
            );
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_clone_string(value: *const c_char) -> *mut c_char {
    unsafe { CStr::from_ptr(value).to_owned().into_raw() }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_free_string(value: *mut c_char) {
    unsafe {
        if !value.is_null() {
            drop(CString::from_raw(value));
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_set_string(target: *mut *mut c_char, value: *const c_char) {
    unsafe {
        let replacement = sm_shader_clone_string(value);
        sm_shader_free_string(std::ptr::replace(target, replacement));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_relative_path(
    base: *const c_char,
    name: *const c_char,
) -> *mut c_char {
    unsafe { crate::util::include_path(CStr::from_ptr(base), CStr::from_ptr(name)).into_raw() }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_upload_png(base: *const c_char, name: *const c_char) {
    unsafe {
        let path = CString::from_raw(sm_shader_relative_path(base, name));
        if let Some(image) = super::texture_png::load(path.as_ptr()) {
            let format = match image.channels {
                4 => gl::RGBA,
                3 => gl::RGB,
                _ => 0x1909,
            };
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                image.width as i32,
                image.height as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                image.pixels.as_ptr().cast(),
            );
        } else {
            crate::c_library::fprintf(
                crate::c_library::stderr,
                c"Unable to read PNG '%s'\n".as_ptr(),
                path.as_ptr(),
            );
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_number(value: *const c_char) -> f32 {
    unsafe { strtod(value, null_mut()) as f32 }
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_linear_filtering() -> bool {
    unsafe { crate::config::g_config.linear_filtering }
}

#[unsafe(no_mangle)]
extern "C-unwind" fn sm_shader_dimension(
    kind: u8,
    scale: f32,
    source: u16,
    viewport: i32,
    last: bool,
) -> u16 {
    match kind {
        3 => scale as u16,
        1 => (source as f32 * scale) as u16,
        2 => (viewport as f32 * scale) as u16,
        _ if last => viewport as f32 as u16,
        _ => (source as f32 * scale) as u16,
    }
}

#[unsafe(no_mangle)]
extern "C-unwind" fn sm_shader_check_index(index: usize, length: usize) {
    let _ = [0u8; 51][..length][index];
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_shader_assert(condition: bool, site: u32) {
    unsafe {
        const SITES: [(&CStr, u32, &CStr); 3] = [
            (
                c"ctx->num_vaos < kMaxVaosInRenderCtx",
                467,
                c"void RenderCtx_SetTexCoords(RenderCtx *, int, uintptr_t)",
            ),
            (
                c"t->gl_texture != 0",
                504,
                c"void GlslShader_SetShaderVars(GlslShader *, RenderCtx *, int)",
            ),
            (
                c"store_pos->gl_texture == 0",
                609,
                c"void GlslShader_Render(GlslShader *, GlTextureWithSize *, int, int, int, int)",
            ),
        ];
        if !condition {
            let (expression, line, function) = SITES[site as usize];
            crate::c_library::__assert_fail(
                expression.as_ptr(),
                c"src/glsl_shader.c".as_ptr(),
                line,
                function.as_ptr(),
            );
        }
    }
}

// Keep the native 42-pointer layout and the existing loader's lookup order.
// Native field reads and reloads require the same owning GL thread without
// concurrent access; atomic slot storage does not make native GL calls thread-safe.
#[unsafe(no_mangle)]
static sm_shader_gl: [AtomicPtr<c_void>; 42] =
    [const { AtomicPtr::new(missing_gl as *mut c_void) }; 42];
const _: () = assert!(size_of::<AtomicPtr<c_void>>() == size_of::<*const c_void>());
const _: () = assert!(align_of::<AtomicPtr<c_void>>() == align_of::<*const c_void>());

extern "system" fn missing_gl() -> ! {
    panic!("gl function was not loaded");
}

const PROCEDURES: [&[&str]; 42] = [
    &["glActiveTexture", "glActiveTextureARB"],
    &["glAttachShader", "glAttachObjectARB"],
    &["glBindBuffer", "glBindBufferARB"],
    &["glBindFramebuffer"],
    &["glBindTexture", "glBindTextureEXT"],
    &["glBufferData", "glBufferDataARB"],
    &["glCompileShader", "glCompileShaderARB"],
    &["glCreateProgram", "glCreateProgramObjectARB"],
    &["glCreateShader", "glCreateShaderObjectARB"],
    &["glDeleteBuffers", "glDeleteBuffersARB"],
    &["glDeleteFramebuffers", "glDeleteFramebuffersEXT"],
    &["glDeleteProgram"],
    &["glDeleteShader"],
    &["glDeleteTextures"],
    &["glDisable"],
    &[
        "glDisableVertexAttribArray",
        "glDisableVertexAttribArrayARB",
    ],
    &["glDrawArrays", "glDrawArraysEXT"],
    &["glEnable"],
    &["glEnableVertexAttribArray", "glEnableVertexAttribArrayARB"],
    &["glFramebufferTexture2D", "glFramebufferTexture2DEXT"],
    &["glGenBuffers", "glGenBuffersARB"],
    &["glGenFramebuffers", "glGenFramebuffersEXT"],
    &["glGenTextures"],
    &["glGenerateMipmap", "glGenerateMipmapEXT"],
    &["glGetAttribLocation", "glGetAttribLocationARB"],
    &["glGetIntegerv"],
    &["glGetProgramInfoLog"],
    &["glGetProgramiv"],
    &["glGetShaderInfoLog"],
    &["glGetShaderiv"],
    &["glGetUniformLocation", "glGetUniformLocationARB"],
    &["glLinkProgram", "glLinkProgramARB"],
    &["glShaderSource", "glShaderSourceARB"],
    &["glTexImage2D"],
    &["glTexParameteri"],
    &["glUniform1f", "glUniform1fARB"],
    &["glUniform1i", "glUniform1iARB"],
    &["glUniform2fv", "glUniform2fvARB"],
    &["glUniformMatrix4fv", "glUniformMatrix4fvARB"],
    &["glUseProgram", "glUseProgramObjectARB"],
    &["glVertexAttribPointer", "glVertexAttribPointerARB"],
    &["glViewport"],
];

// Associate each fallback with its unresolved canonical lookup. This also
// keeps another function's fallback queries from overwriting an accepted slot.
pub(super) unsafe fn load_with(mut load: impl FnMut(&str) -> *const c_void) {
    check_dependency();
    let mut pending = None::<(usize, usize)>;
    gl::load_with(|name| {
        let address = load(name);
        if let Some((slot, next)) = pending.take() {
            if PROCEDURES[slot].get(next) == Some(&name) {
                if !address.is_null() {
                    sm_shader_gl[slot].store(address.cast_mut(), Ordering::Relaxed);
                } else if next + 1 < PROCEDURES[slot].len() {
                    pending = Some((slot, next + 1));
                }
                return address;
            }
        }
        if let Some(slot) = PROCEDURES.iter().position(|names| names[0] == name) {
            sm_shader_gl[slot].store(
                if address.is_null() {
                    missing_gl as *mut c_void
                } else {
                    address.cast_mut()
                },
                Ordering::Relaxed,
            );
            if address.is_null() && PROCEDURES[slot].len() > 1 {
                pending = Some((slot, 1));
            }
        }
        address
    });
}
