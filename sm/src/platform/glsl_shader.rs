use crate::c_library::{
    __assert_fail, calloc, fprintf, free, memcmp, memcpy, memset, snprintf, stderr, strcmp, strdup,
    strlen, strtod, strtol, strtoul,
};
use crate::config::{g_config, ParseBool};
use crate::set_fields;
pub use crate::types::{
    size_t, GLboolean, GLchar, GLenum, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint,
};
use crate::util::{
    ByteArray_AppendData, ByteArray_Destroy, NextDelim, NextLineStripComments,
    NextPossiblyQuotedString, ReadWholeFile, ReplaceFilenameWithNewPath, SkipPrefix, SplitKeyValue,
    StrSet, StringEqualsNoCase,
};
use ::core::ffi::{c_char, c_double, c_float, c_void};
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
    ByteArray, GlTextureWithSize, GlslParam, GlslPass, GlslShader, GlslTexture, GlslTextureUniform,
    GlslUniforms,
};
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct RenderCtx {
        pub texture_unit, offset, num_vaos: u32;
        pub vaos: [u32; 51];
    }
}
pub const kMaxVaosInRenderCtx: C2RustUnnamed_3 = 51;
pub use crate::types::Config;
pub type C2RustUnnamed_3 = u32;
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
pub const NULL: *mut c_void = null_mut::<c_void>();
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
                    let image = super::texture_png::load(new_filename_0);
                    if image.is_none() {
                        fprintf(
                            stderr,
                            b"Unable to read PNG '%s'\n\0" as *const u8 as *const c_char,
                            new_filename_0,
                        );
                    } else {
                        let image = image.unwrap();
                        gl::TexImage2D(
                            GL_TEXTURE_2D as GLenum,
                            0 as GLint,
                            GL_RGBA,
                            image.width as GLsizei,
                            image.height as GLsizei,
                            0 as GLint,
                            (if image.channels == 4 {
                                GL_RGBA
                            } else if image.channels == 3 {
                                GL_RGB
                            } else {
                                GL_LUMINANCE
                            }) as GLenum,
                            GL_UNSIGNED_BYTE as GLenum,
                            image.pixels.as_ptr().cast(),
                        );
                    }
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
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
