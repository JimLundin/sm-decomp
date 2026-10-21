use crate::c_library::{__assert_fail, fprintf, free, stderr, strtod, strtol, strtoul};
use crate::config::{g_config, ParseBool};
use crate::types::GlTextureWithSize;
use crate::util::{
    NextDelim, NextLineStripComments, NextPossiblyQuotedString, ReadWholeFile,
    ReplaceFilenameWithNewPath, SplitKeyValue,
};
use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr::{null, null_mut};

const EMPTY_IMAGE: GlTextureWithSize = GlTextureWithSize {
    gl_texture: 0,
    width: 0,
    height: 0,
};
const TEX_COORDS: [f32; 16] = [
    0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0,
];
const MVP: [f32; 16] = [
    2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -1.0, -1.0, 0.0, 1.0,
];

impl Default for GlTextureWithSize {
    fn default() -> Self {
        EMPTY_IMAGE
    }
}

#[derive(Default)]
pub struct GlslShader {
    passes: Vec<ShaderPass>,
    parameters: Vec<Parameter>,
    textures: Vec<LookupTexture>,
    vbo: u32,
    frame_count: u32,
    previous_count: usize,
    previous: [GlTextureWithSize; 8],
}

struct Parameter {
    id: CString,
    has_value: bool,
    value: f32,
    min: f32,
    max: f32,
    // Preserve the existing one-based 20-slot parameter boundary.
    locations: [u32; 20],
}

struct LookupTexture {
    id: CString,
    filename: Option<CString>,
    filter: u32,
    texture: u32,
    wrap: u32,
    mipmap: bool,
}

#[derive(Default)]
struct ShaderPass {
    filename: Option<CString>,
    scale_type: [u8; 2],
    scale: [f32; 2],
    float_framebuffer: bool,
    srgb_framebuffer: bool,
    mipmap: bool,
    wrap: u32,
    frame_count_mod: u32,
    program: u32,
    fbo: u32,
    filter: u32,
    image: GlTextureWithSize,
    uniforms: Uniforms,
}

#[derive(Clone, Copy, Default)]
struct TextureUniforms {
    texture: i32,
    input_size: i32,
    texture_size: i32,
    coordinates: i32,
}

#[derive(Default)]
struct Uniforms {
    input: TextureUniforms,
    output_size: i32,
    frame_count: i32,
    frame_direction: i32,
    lut_coordinates: i32,
    vertex_coordinates: i32,
    original: TextureUniforms,
    previous: [TextureUniforms; 7],
    passes: [TextureUniforms; 20],
    pass_previous: [TextureUniforms; 20],
    textures: Vec<i32>,
}

// Keep the project's byte-oriented lexical and file policies, including its
// handling of comments, CRLF, numeric prefixes and relative path concatenation.
struct FileBuffer(*mut c_char);
impl Drop for FileBuffer {
    fn drop(&mut self) {
        unsafe {
            free(self.0.cast());
        }
    }
}

unsafe fn relative_path(base: &CStr, name: &CStr) -> CString {
    let allocated = FileBuffer(ReplaceFilenameWithNewPath(base.as_ptr(), name.as_ptr()));
    CStr::from_ptr(allocated.0).to_owned()
}

unsafe fn number(value: *const c_char) -> f32 {
    strtod(value, null_mut()) as f32
}

unsafe fn scale_type(value: *const c_char) -> u8 {
    let value = CStr::from_ptr(value).to_bytes();
    if value.eq_ignore_ascii_case(b"source") {
        1
    } else if value.eq_ignore_ascii_case(b"viewport") {
        2
    } else if value.eq_ignore_ascii_case(b"absolute") {
        3
    } else {
        0
    }
}

unsafe fn wrap_mode(value: *const c_char) -> u32 {
    let value = CStr::from_ptr(value).to_bytes();
    if value.eq_ignore_ascii_case(b"repeat") {
        gl::REPEAT
    } else if value.eq_ignore_ascii_case(b"clamp_to_edge") {
        gl::CLAMP_TO_EDGE
    } else if value.eq_ignore_ascii_case(b"clamp") {
        0x2900
    } else {
        gl::CLAMP_TO_BORDER
    }
}

fn minification_filter(filter: u32, mipmap: bool) -> i32 {
    if !mipmap {
        filter as i32
    } else if filter == gl::LINEAR {
        gl::LINEAR_MIPMAP_LINEAR as i32
    } else {
        gl::NEAREST_MIPMAP_NEAREST as i32
    }
}

impl GlslShader {
    fn initialize_passes(&mut self, count: usize) {
        self.passes.push(ShaderPass::default()); // original input image
        self.passes.extend((0..count).map(|_| ShaderPass {
            scale: [1.0; 2],
            wrap: gl::CLAMP_TO_BORDER,
            ..ShaderPass::default()
        }));
    }

    fn parameter(&mut self, id: &CStr) -> &mut Parameter {
        let index = self
            .parameters
            .iter()
            .position(|p| p.id.as_c_str() == id)
            .unwrap_or_else(|| {
                self.parameters.push(Parameter {
                    id: id.to_owned(),
                    has_value: false,
                    value: 0.0,
                    min: 0.0,
                    max: 0.0,
                    locations: [0; 20],
                });
                self.parameters.len() - 1
            });
        &mut self.parameters[index]
    }

    unsafe fn pass_key(&self, key: &CStr, prefix: &[u8]) -> Option<usize> {
        let suffix = key.to_bytes().strip_prefix(prefix)?;
        if !suffix.first()?.is_ascii_digit() {
            return None;
        }
        let mut end = null_mut();
        let index = strtoul(key.as_ptr().add(prefix.len()), &mut end, 10) as u32;
        (index < self.passes.len().saturating_sub(1) as u32 && *end == 0)
            .then_some(index as usize + 1)
    }

    unsafe fn read_setting(&mut self, key: &CStr, value: *mut c_char) -> bool {
        const KEYS: [&[u8]; 14] = [
            b"filter_linear",
            b"scale_type",
            b"scale_type_x",
            b"scale_type_y",
            b"scale",
            b"scale_x",
            b"scale_y",
            b"shader",
            b"wrap_mode",
            b"mipmap_input",
            b"frame_count_mod",
            b"float_framebuffer",
            b"srgb_framebuffer",
            b"alias",
        ];
        for (setting, prefix) in KEYS.iter().enumerate() {
            if let Some(index) = self.pass_key(key, prefix) {
                let pass = &mut self.passes[index];
                match setting {
                    0 => {
                        pass.filter = if ParseBool(value, null_mut()) {
                            gl::LINEAR
                        } else {
                            gl::NEAREST
                        }
                    }
                    1 => pass.scale_type = [scale_type(value); 2],
                    2 | 3 => pass.scale_type[setting - 2] = scale_type(value),
                    4 => pass.scale = [number(value); 2],
                    5 | 6 => pass.scale[setting - 5] = number(value),
                    7 => pass.filename = Some(CStr::from_ptr(value).to_owned()),
                    8 => pass.wrap = wrap_mode(value),
                    9 => pass.mipmap = ParseBool(value, null_mut()),
                    10 => pass.frame_count_mod = strtol(value, null_mut(), 10) as i32 as u32,
                    11 => pass.float_framebuffer = ParseBool(value, null_mut()),
                    12 => pass.srgb_framebuffer = ParseBool(value, null_mut()),
                    _ => {} // alias keys have always been accepted without effect
                }
                return true;
            }
        }
        if key.to_bytes() == b"textures" && self.textures.is_empty() {
            let mut cursor = value;
            loop {
                let id = NextDelim(&mut cursor, b';' as i32);
                if id.is_null() || self.textures.len() == 10 {
                    break;
                }
                self.textures.push(LookupTexture {
                    id: CStr::from_ptr(id).to_owned(),
                    filename: None,
                    filter: gl::NEAREST,
                    texture: 0,
                    wrap: gl::CLAMP_TO_BORDER,
                    mipmap: false,
                });
            }
            return true;
        }
        if key.to_bytes() == b"parameters" {
            let mut cursor = value;
            loop {
                let id = NextDelim(&mut cursor, b';' as i32);
                if id.is_null() {
                    break;
                }
                self.parameter(CStr::from_ptr(id));
            }
            return true;
        }
        for texture in &mut self.textures {
            if let Some(suffix) = key.to_bytes().strip_prefix(texture.id.to_bytes()) {
                match suffix {
                    b"" => texture.filename = Some(CStr::from_ptr(value).to_owned()),
                    b"_wrap_mode" => texture.wrap = wrap_mode(value),
                    b"_mipmap" => texture.mipmap = ParseBool(value, null_mut()),
                    b"_linear" => {
                        texture.filter = if ParseBool(value, null_mut()) {
                            gl::LINEAR
                        } else {
                            gl::NEAREST
                        }
                    }
                    _ => continue,
                }
                return true;
            }
        }
        if let Some(parameter) = self.parameters.iter_mut().find(|p| p.id.as_c_str() == key) {
            parameter.value = number(value);
            parameter.has_value = true;
            return true;
        }
        false
    }

    unsafe fn read_preset(&mut self, filename: &CStr) -> bool {
        let data = FileBuffer(ReadWholeFile(filename.as_ptr(), null_mut()).cast());
        if data.0.is_null() {
            return false;
        }
        let mut cursor = data.0;
        let mut line_number = 1;
        loop {
            let line = NextLineStripComments(&mut cursor);
            if line.is_null() {
                break;
            }
            let mut value = SplitKeyValue(line);
            if value.is_null() {
                if *line != 0 {
                    fprintf(
                        stderr,
                        c"%s:%d: Expecting key=value\n".as_ptr(),
                        filename.as_ptr(),
                        line_number,
                    );
                }
            } else {
                if *value == b'"' as c_char {
                    value = value.add(1);
                    let mut end = value;
                    while *end != 0 && *end != b'"' as c_char {
                        end = end.add(1);
                    }
                    *end = 0;
                }
                let key = CStr::from_ptr(line);
                if self.passes.is_empty() {
                    if key.to_bytes() != b"shaders" {
                        fprintf(
                            stderr,
                            c"%s:%d: Expecting 'shaders'\n".as_ptr(),
                            filename.as_ptr(),
                            line_number,
                        );
                        break;
                    }
                    let count = strtoul(value, null_mut(), 10) as i32;
                    if !(1..=20).contains(&count) {
                        break;
                    }
                    self.initialize_passes(count as usize);
                } else if !self.read_setting(key, value) {
                    fprintf(
                        stderr,
                        c"%s:%d: Unknown key '%s'\n".as_ptr(),
                        filename.as_ptr(),
                        line_number,
                        line,
                    );
                }
            }
            line_number += 1;
        }
        !self.passes.is_empty()
    }

    unsafe fn read_shader(&mut self, filename: &CStr, output: &mut Vec<u8>) {
        let data = FileBuffer(ReadWholeFile(filename.as_ptr(), null_mut()).cast());
        if data.0.is_null() {
            fprintf(
                stderr,
                c"Unable to read file '%s'\n".as_ptr(),
                filename.as_ptr(),
            );
            return;
        }
        let mut cursor = data.0;
        loop {
            let line = NextDelim(&mut cursor, b'\n' as i32);
            if line.is_null() {
                break;
            }
            let bytes = CStr::from_ptr(line).to_bytes();
            if bytes.starts_with(b"#include") {
                let mut argument = line.add(8);
                let name = NextPossiblyQuotedString(&mut argument);
                let path = relative_path(filename, CStr::from_ptr(name));
                self.read_shader(&path, output);
            } else if bytes.starts_with(b"#pragma parameter") {
                let mut arguments = line.add(17);
                let parameter =
                    self.parameter(CStr::from_ptr(NextPossiblyQuotedString(&mut arguments)));
                NextPossiblyQuotedString(&mut arguments);
                let value = number(NextPossiblyQuotedString(&mut arguments));
                if !parameter.has_value {
                    parameter.value = value;
                }
                parameter.min = number(NextPossiblyQuotedString(&mut arguments));
                parameter.max = number(NextPossiblyQuotedString(&mut arguments));
            } else {
                if output.try_reserve(bytes.len() + 1).is_err() {
                    crate::host::Die(c"memory allocation failed".as_ptr());
                }
                output.extend_from_slice(bytes);
                output.push(b'\n');
            }
        }
    }
}

fn scaled_dimension(kind: u8, scale: f32, source: u16, viewport: i32, last: bool) -> u16 {
    match kind {
        3 => scale as u16,
        1 => (source as f32 * scale) as u16,
        2 => (viewport as f32 * scale) as u16,
        _ if last => viewport as f32 as u16,
        _ => (source as f32 * scale) as u16,
    }
}

unsafe fn compile_stage(program: u32, filename: &CStr, stage: u32, source: &[u8]) -> bool {
    let skip = if source.starts_with(b"#version") {
        source
            .iter()
            .position(|&byte| byte == b'\n')
            .map_or(source.len(), |position| position + 1)
    } else {
        0
    };
    let version = if skip == 0 {
        b"#version 330\n".as_slice()
    } else {
        &source[..skip]
    };
    let prefix = if stage == gl::VERTEX_SHADER {
        b"#define VERTEX\n#define PARAMETER_UNIFORM\n".as_slice()
    } else {
        b"#define FRAGMENT\n#define PARAMETER_UNIFORM\n".as_slice()
    };
    let strings = [
        version.as_ptr().cast(),
        prefix.as_ptr().cast(),
        source[skip..].as_ptr().cast(),
    ];
    let lengths = [
        version.len() as i32,
        prefix.len() as i32,
        (source.len() - skip) as i32,
    ];
    let shader = gl::CreateShader(stage);
    gl::ShaderSource(shader, 3, strings.as_ptr(), lengths.as_ptr());
    gl::CompileShader(shader);
    let mut status = 0;
    let mut log = [0; 256];
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
    gl::GetShaderInfoLog(shader, 256, null_mut(), log.as_mut_ptr());
    if status != gl::TRUE as i32 || log[0] != 0 {
        fprintf(
            stderr,
            c"%s compiling %s shader in file '%s':\n%s\n".as_ptr(),
            if status != gl::TRUE as i32 {
                c"Error"
            } else {
                c"While"
            }
            .as_ptr(),
            if stage == gl::VERTEX_SHADER {
                c"vertex"
            } else {
                c"fragment"
            }
            .as_ptr(),
            filename.as_ptr(),
            log.as_ptr(),
        );
    }
    if status == gl::TRUE as i32 {
        gl::AttachShader(program, shader);
    }
    gl::DeleteShader(shader);
    status == gl::TRUE as i32
}

impl TextureUniforms {
    unsafe fn read(program: u32, prefix: &str) -> Self {
        let name = |suffix| CString::new(format!("{prefix}{suffix}")).unwrap();
        Self {
            texture: gl::GetUniformLocation(program, name("Texture").as_ptr()),
            input_size: gl::GetUniformLocation(program, name("InputSize").as_ptr()),
            texture_size: gl::GetUniformLocation(program, name("TextureSize").as_ptr()),
            coordinates: gl::GetAttribLocation(program, name("TexCoord").as_ptr()),
        }
    }
}

struct RenderContext {
    texture_unit: u32,
    offset: usize,
    attributes: [u32; 51],
    attribute_count: usize,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self {
            texture_unit: 0,
            offset: 0,
            attributes: [0; 51],
            attribute_count: 0,
        }
    }
}

impl RenderContext {
    unsafe fn texture(&mut self, location: i32, texture: u32) {
        if location >= 0 {
            gl::ActiveTexture(gl::TEXTURE0.wrapping_add(self.texture_unit));
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::Uniform1i(location, self.texture_unit as i32);
            self.texture_unit = self.texture_unit.wrapping_add(1);
        }
    }

    unsafe fn coordinates(&mut self, location: i32, offset: usize) {
        if location >= 0 {
            crate::native_assertion! {
                '_c2rust_label: self.attribute_count < 51;
                b"ctx->num_vaos < kMaxVaosInRenderCtx\0", b"src/glsl_shader.c\0", 467,
                b"void RenderCtx_SetTexCoords(RenderCtx *, int, uintptr_t)\0"
            };
            self.attributes[self.attribute_count] = location as u32;
            self.attribute_count += 1;
            gl::VertexAttribPointer(
                location as u32,
                2,
                gl::FLOAT,
                gl::FALSE,
                0,
                offset as *const c_void,
            );
            gl::EnableVertexAttribArray(location as u32);
        }
    }

    unsafe fn image(&mut self, uniforms: TextureUniforms, image: GlTextureWithSize) {
        let size = [image.width as f32, image.height as f32];
        self.texture(uniforms.texture, image.gl_texture);
        if uniforms.input_size >= 0 {
            gl::Uniform2fv(uniforms.input_size, 1, size.as_ptr());
        }
        if uniforms.texture_size >= 0 {
            gl::Uniform2fv(uniforms.texture_size, 1, size.as_ptr());
        }
        self.coordinates(uniforms.coordinates, self.offset);
    }
}

impl Drop for GlslShader {
    fn drop(&mut self) {
        unsafe {
            for pass in self.passes.iter().skip(1) {
                gl::DeleteProgram(pass.program);
                gl::DeleteTextures(1, &pass.image.gl_texture);
                gl::DeleteFramebuffers(1, &pass.fbo);
            }
            for texture in &self.textures {
                gl::DeleteTextures(1, &texture.texture);
            }
            for image in &self.previous {
                gl::DeleteTextures(1, &image.gl_texture);
            }
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}

pub unsafe fn GlslShader_CreateFromFile(filename: *const c_char) -> *mut GlslShader {
    let allocation =
        std::alloc::alloc(std::alloc::Layout::new::<GlslShader>()).cast::<GlslShader>();
    if allocation.is_null() {
        return null_mut();
    }
    allocation.write(GlslShader::default());
    let shader = &mut *allocation;
    let mut filename = CStr::from_ptr(filename);
    if filename.to_bytes().ends_with(b".glsl") {
        shader.initialize_passes(1);
        shader.passes[1].filename = Some(filename.to_owned());
        filename = c"";
    } else if !shader.read_preset(filename) {
        fprintf(
            stderr,
            c"Unable to read file '%s'\n".as_ptr(),
            filename.as_ptr(),
        );
        GlslShader_Destroy(allocation);
        return null_mut();
    }
    if shader.load_resources(filename) {
        allocation
    } else {
        GlslShader_Destroy(allocation);
        null_mut()
    }
}

pub unsafe fn GlslShader_Destroy(shader: *mut GlslShader) {
    drop(Box::from_raw(shader));
}

pub unsafe fn GlslShader_Render(
    shader: *mut GlslShader,
    texture: *mut GlTextureWithSize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    (*shader).render(&mut *texture, [x, y, width, height]);
}

impl GlslShader {
    unsafe fn load_resources(&mut self, filename: &CStr) -> bool {
        let mut source = Vec::new();
        for index in 1..self.passes.len() {
            let Some(name) = self.passes[index].filename.clone() else {
                fprintf(
                    stderr,
                    c"shader%d attribute missing\n".as_ptr(),
                    index as i32 - 1,
                );
                return false;
            };
            source.clear();
            self.read_shader(&relative_path(filename, &name), &mut source);
            if source.is_empty() {
                fprintf(
                    stderr,
                    c"Couldn't read shader in file '%s'\n".as_ptr(),
                    name.as_ptr(),
                );
                return false;
            }
            let pass = &mut self.passes[index];
            pass.program = gl::CreateProgram();
            if !compile_stage(pass.program, &name, gl::VERTEX_SHADER, &source)
                || !compile_stage(pass.program, &name, gl::FRAGMENT_SHADER, &source)
            {
                return false;
            }
            gl::LinkProgram(pass.program);
            let mut status = 0;
            let mut log = [0; 256];
            gl::GetProgramiv(pass.program, gl::LINK_STATUS, &mut status);
            gl::GetProgramInfoLog(pass.program, 256, null_mut(), log.as_mut_ptr());
            if status != gl::TRUE as i32 || log[0] != 0 {
                fprintf(
                    stderr,
                    c"%s linking shader in file '%s':\n%s\n".as_ptr(),
                    if status != gl::TRUE as i32 {
                        c"Error"
                    } else {
                        c"While"
                    }
                    .as_ptr(),
                    name.as_ptr(),
                    log.as_ptr(),
                );
            }
            if status != gl::TRUE as i32 {
                return false;
            }
            gl::GenFramebuffers(1, &mut pass.fbo);
            gl::GenTextures(1, &mut pass.image.gl_texture);
        }
        for texture in &mut self.textures {
            gl::GenTextures(1, &mut texture.texture);
            gl::BindTexture(gl::TEXTURE_2D, texture.texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, texture.wrap as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, texture.wrap as i32);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                texture.filter as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                minification_filter(texture.filter, texture.mipmap),
            );
            if let Some(name) = &texture.filename {
                let path = relative_path(filename, name);
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
                    fprintf(stderr, c"Unable to read PNG '%s'\n".as_ptr(), path.as_ptr());
                }
            }
            if texture.mipmap {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
        }
        for parameter in &mut self.parameters {
            parameter.value = if parameter.value < parameter.min {
                parameter.min
            } else if parameter.value > parameter.max {
                parameter.max
            } else {
                parameter.value
            };
        }
        self.find_uniforms();
        for index in 0..self.previous_count {
            gl::GenTextures(1, &mut self.previous[(7 - index) & 7].gl_texture);
        }
        gl::GenBuffers(1, &mut self.vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&TEX_COORDS) as isize,
            TEX_COORDS.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        true
    }

    unsafe fn find_uniforms(&mut self) {
        self.previous_count = 0;
        let count = self.passes.len() - 1;
        for (index, pass) in self.passes.iter_mut().enumerate().skip(1) {
            let program = pass.program;
            gl::UseProgram(program);
            let mvp = gl::GetUniformLocation(program, c"MVPMatrix".as_ptr());
            if mvp >= 0 {
                gl::UniformMatrix4fv(mvp, 1, gl::FALSE, MVP.as_ptr());
            }
            let uniforms = &mut pass.uniforms;
            uniforms.input = TextureUniforms::read(program, "");
            uniforms.output_size = gl::GetUniformLocation(program, c"OutputSize".as_ptr());
            uniforms.frame_count = gl::GetUniformLocation(program, c"FrameCount".as_ptr());
            uniforms.frame_direction = gl::GetUniformLocation(program, c"FrameDirection".as_ptr());
            uniforms.lut_coordinates = gl::GetAttribLocation(program, c"LUTTexCoord".as_ptr());
            uniforms.vertex_coordinates = gl::GetAttribLocation(program, c"VertexCoord".as_ptr());
            uniforms.original = TextureUniforms::read(program, "Orig");
            for (previous, uniform) in uniforms.previous.iter_mut().enumerate() {
                let name = if previous == 0 {
                    "Prev".to_owned()
                } else {
                    format!("Prev{previous}")
                };
                *uniform = TextureUniforms::read(program, &name);
                // The old loop overwrites this across passes; do not take max().
                if uniform.texture >= 0 {
                    self.previous_count = previous + 1;
                }
            }
            for previous in 0..count {
                uniforms.passes[previous] =
                    TextureUniforms::read(program, &format!("Pass{previous}"));
                uniforms.pass_previous[previous] =
                    TextureUniforms::read(program, &format!("PassPrev{previous}"));
            }
            uniforms.textures = self
                .textures
                .iter()
                .map(|texture| gl::GetUniformLocation(program, texture.id.as_ptr()))
                .collect();
            for parameter in &mut self.parameters {
                parameter.locations[index] =
                    gl::GetUniformLocation(program, parameter.id.as_ptr()) as u32;
            }
        }
        gl::UseProgram(0);
    }

    unsafe fn set_uniforms(&self, index: usize, context: &mut RenderContext) {
        let pass = &self.passes[index];
        let uniforms = &pass.uniforms;
        context.image(uniforms.input, self.passes[index - 1].image);
        if uniforms.output_size >= 0 {
            gl::Uniform2fv(
                uniforms.output_size,
                1,
                [pass.image.width as f32, pass.image.height as f32].as_ptr(),
            );
        }
        if uniforms.frame_count >= 0 {
            let count = if pass.frame_count_mod != 0 {
                self.frame_count % pass.frame_count_mod
            } else {
                self.frame_count
            };
            gl::Uniform1i(uniforms.frame_count, count as i32);
        }
        if uniforms.frame_direction >= 0 {
            gl::Uniform1i(uniforms.frame_direction, 1);
        }
        context.coordinates(uniforms.lut_coordinates, context.offset);
        context.coordinates(uniforms.vertex_coordinates, 0);
        context.image(uniforms.original, self.passes[0].image);
        for previous in 0..self.previous_count {
            let image = self.previous[(self
                .frame_count
                .wrapping_sub(1)
                .wrapping_sub(previous as u32)
                & 7) as usize];
            crate::native_assertion! {
                '_c2rust_label: image.gl_texture != 0;
                b"t->gl_texture != 0\0", b"src/glsl_shader.c\0", 504,
                b"void GlslShader_SetShaderVars(GlslShader *, RenderCtx *, int)\0"
            };
            if image.width != 0 {
                context.image(uniforms.previous[previous], image);
            }
        }
        for (texture, &location) in self.textures.iter().zip(&uniforms.textures) {
            context.texture(location, texture.texture);
        }
        for previous in 1..index {
            context.image(uniforms.passes[previous], self.passes[previous].image);
        }
        for previous in 1..index {
            context.image(
                uniforms.pass_previous[index - previous],
                self.passes[previous].image,
            );
        }
        for parameter in &self.parameters {
            // The legacy unsigned >= 0 test calls this even for location -1.
            gl::Uniform1f(parameter.locations[index] as i32, parameter.value);
        }
        gl::ActiveTexture(gl::TEXTURE0);
    }

    unsafe fn render(&mut self, input: &mut GlTextureWithSize, viewport: [i32; 4]) {
        self.passes[0].image = *input;
        let mut framebuffer = 0;
        gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut framebuffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        for index in 1..self.passes.len() {
            let last = index == self.passes.len() - 1;
            let previous = self.passes[index - 1].image;
            let pass = &mut self.passes[index];
            pass.image.width = scaled_dimension(
                pass.scale_type[0],
                pass.scale[0],
                previous.width,
                viewport[2],
                last,
            );
            pass.image.height = scaled_dimension(
                pass.scale_type[1],
                pass.scale[1],
                previous.height,
                viewport[3],
                last,
            );
            if !last {
                gl::BindTexture(gl::TEXTURE_2D, pass.image.gl_texture);
                let (format, data_type) = if pass.srgb_framebuffer {
                    gl::Enable(gl::FRAMEBUFFER_SRGB);
                    (gl::SRGB8_ALPHA8, gl::UNSIGNED_INT_8_8_8_8)
                } else if pass.float_framebuffer {
                    (gl::RGBA32F, gl::FLOAT)
                } else {
                    (gl::RGBA, gl::UNSIGNED_INT_8_8_8_8)
                };
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    format as i32,
                    pass.image.width as i32,
                    pass.image.height as i32,
                    0,
                    gl::RGBA,
                    data_type,
                    null(),
                );
                gl::Viewport(0, 0, pass.image.width as i32, pass.image.height as i32);
                gl::BindFramebuffer(gl::FRAMEBUFFER, pass.fbo);
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    pass.image.gl_texture,
                    0,
                );
            } else {
                gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer as u32);
                gl::Viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
            }
            gl::BindTexture(gl::TEXTURE_2D, previous.gl_texture);
            let filter = if pass.filter != 0 {
                pass.filter
            } else if last && g_config.linear_filtering {
                gl::LINEAR
            } else {
                gl::NEAREST
            };
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                minification_filter(filter, pass.mipmap),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, pass.wrap as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, pass.wrap as i32);
            if pass.mipmap {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
            gl::UseProgram(pass.program);
            let srgb = pass.srgb_framebuffer;
            let mut context = RenderContext {
                offset: if last {
                    8 * std::mem::size_of::<f32>()
                } else {
                    0
                },
                ..Default::default()
            };
            self.set_uniforms(index, &mut context);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            for &attribute in &context.attributes[..context.attribute_count] {
                gl::DisableVertexAttribArray(attribute);
            }
            if srgb {
                gl::Disable(gl::FRAMEBUFFER_SRGB);
            }
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer as u32);
        gl::UseProgram(0);
        gl::BindTexture(gl::TEXTURE_2D, 0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        if self.previous_count != 0 {
            let store = (self.frame_count & 7) as usize;
            let load = (self.frame_count.wrapping_sub(self.previous_count as u32) & 7) as usize;
            crate::native_assertion! {
                '_c2rust_label: self.previous[store].gl_texture == 0;
                b"store_pos->gl_texture == 0\0", b"src/glsl_shader.c\0", 609,
                b"void GlslShader_Render(GlslShader *, GlTextureWithSize *, int, int, int, int)\0"
            };
            self.previous[store] = *input;
            *input = self.previous[load];
            self.previous[load] = EMPTY_IMAGE;
        }
        self.frame_count = self.frame_count.wrapping_add(1);
    }
}
