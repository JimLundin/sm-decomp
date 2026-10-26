"""Install the pinned configuration/input dependency with native SM compatibility."""
import argparse
import hashlib
import os
from pathlib import Path
import re
import subprocess
import urllib.request

REVISION = "45a149d8b4bb8098f53d1009f76e76e7d8b79bbf"
FILES = {
    "src/glsl_shader.c": "106245f15957009f1d9a7286844dcbc32d2d0d23f525c26b5a8b2a21d089e064",
    "src/glsl_shader.h": "5e07f863f10e567fe710a79174ff88281f0c47b841a5606080ec37886673744d",
    "src/config.c": "d7c4f90eb30050919c329b1aba21f07c122ffa5220edeb059c3d9e3e7364b111",
    "src/util.c": "60e519bd41049f1c09e158471021d4449cb418b560b6318e510b47cace1a049f",
    "src/config.h": "f428417ba93dd8df89fa0b1ecce7f15be1c14d50efaa9341fd6bac863b5dfc94",
    "src/types.h": "af3bbac047a9a6db0707f9241aa9e19767147ab3783b523cb22cfd55b0409e23",
    "LICENSE.txt": "78b243fa858f4bcb49f8ccca14fec262f62fcee4e44a1ce7565840bdb4ca2217",
}
NATIVE_INPUTS = ("sm/src/config.rs", "sm/src/native_config.c", "sm/src/native_config.h",
                 "sm/src/native_util.h", "sm/src/types.rs", "tools/install_native_config.py")
UTILITY_INPUTS = ("sm/src/util.rs", "sm/src/native_util.c", "sm/src/native_util.h",
                  "sm/src/types.rs", "tools/install_native_config.py")
SYMBOLS = {
    "g_config": "sm_config_state", "ParseConfigFile": "sm_config_parse_file",
    "FindCmdForSdlKey": "sm_config_find_key", "FindCmdForGamepadButton": "sm_config_find_button",
    "ParseBool": "sm_config_parse_bool", "ReadWholeFile": "sm_util_ReadWholeFile",
    "NextLineStripComments": "sm_util_NextLineStripComments", "NextDelim": "sm_util_NextDelim",
    "NextPossiblyQuotedString": "sm_util_NextPossiblyQuotedString",
    "ReplaceFilenameWithNewPath": "sm_util_ReplaceFilenameWithNewPath", "SkipPrefix": "sm_util_SkipPrefix",
    "SplitKeyValue": "sm_util_SplitKeyValue", "StringEqualsNoCase": "sm_util_StringEqualsNoCase",
    "StringStartsWithNoCase": "sm_util_StringStartsWithNoCase", "Die": "sm_util_die", "atoi": "sm_config_atoi",
}


UTILITY_NAMES = """NextDelim StringEqualsNoCase StringStartsWithNoCase ReadWholeFile
    NextLineStripComments NextPossiblyQuotedString ReplaceFilenameWithNewPath
    SplitKeyValue SkipPrefix StrSet""".split()


def utility_source(source):
    selected = []
    for name in ["ToLower"] + UTILITY_NAMES:
        match = re.search(r"^(?:static inline )?[^\n]+\b" + name +
                          r"\([^\n]*\) \{\n.*?^\}", source, re.M | re.S)
        if not match:
            raise ValueError(f"Missing pinned utility function: {name}")
        selected.append(match[0])
    source = "\n\n".join(selected) + "\n"
    for old, new in (
        ("fread(buffer, 1, size, f) != size",
         "(size_t)(uint32_t)fread(buffer, 1, size, f) != size"),
        ("memchr(p, '#', eol - p)", "memchr(p, '#', (size_t)(int32_t)(eol - p))"),
    ):
        if source.count(old) != 1:
            raise ValueError(f"Pinned utility patch no longer matches: {old}")
        source = source.replace(old, new)
    symbols = {name: "sm_util_" + name for name in UTILITY_NAMES}
    symbols["Die"] = "sm_util_die"
    return re.sub(r"\b(?:" + "|".join(symbols) + r")\b", lambda m: symbols[m[0]], source)


def rename_symbols(source):
    return re.sub(r"\b(?:" + "|".join(SYMBOLS) + r")\b", lambda m: SYMBOLS[m[0]], source)


def replace(source, old, new):
    if source.count(old) != 1:
        raise ValueError(f"Pinned configuration patch no longer matches: {old}")
    return source.replace(old, new)


def config_sources(source, header):
    source = replace(source, '#include "features.h"\n', '')
    source = replace(source, '#include "util.h"', '#include "native_config.h"')
    header = replace(header, '  const char *language;\n', '')
    header = replace(header, '  kOutputMethod_OpenGL_ES,\n', '')
    header = replace(header, '  kKeys_CheatWalkThroughWalls,\n', '')
    source = replace(source, '  _(SDLK_w), _(SDLK_o), S(SDLK_w), C(SDLK_e),',
                     '  _(SDLK_w), _(SDLK_q), S(SDLK_r),')
    source = replace(source, 'S(CheatLife), S(CheatKeys), S(CheatEquipment), S(CheatWalkThroughWalls),',
                     'S(CheatLife), S(CheatJump), S(ToggleWhichFrame),')
    for old, new in (("CheatKeys", "CheatJump"), ("CheatEquipment", "ToggleWhichFrame")):
        header = replace(header, "  kKeys_" + old + ",", "  kKeys_" + new + ",")
    reference_keys = "SDLK_1 SDLK_2 SDLK_3 SDLK_4 SDLK_5 SDLK_6 SDLK_7 SDLK_8 SDLK_9 SDLK_0 SDLK_MINUS SDLK_EQUALS SDLK_BACKSPACE".split()
    for title, modifier in (("Load Ref State", "_"), ("Replay Ref State", "C")):
        old = f"  // {title}\n  " + ", ".join(["N"] * 20) + ","
        new = f"  // {title}\n  " + ", ".join(f"{modifier}({key})" for key in reference_keys) + ", " + ", ".join(["N"] * 7) + ","
        source = replace(source, old, new)
    first = source.index("static bool ParseBoolBit(")
    last = source.index("static bool HandleIniConfig(", first)
    source = source[:first] + source[last:]
    source = replace(source, '                               StringEqualsNoCase(value, "OpenGL ES") ? kOutputMethod_OpenGL_ES :\n', '')
    source = replace(source, '    } else if (StringEqualsNoCase(key, "DimFlashes")) {\n      return ParseBoolBit(value, &g_config.features0, kFeatures0_DimFlashes);\n', '')
    first = source.index('    } else if (StringEqualsNoCase(key, "ExtendedAspectRatio")) {')
    last = source.index('    } else if (StringEqualsNoCase(key, "DisplayPerfInTitle")) {', first)
    source = source[:first] + source[last:]
    source = replace(source, '    } else if (StringEqualsNoCase(key, "Language")) {\n      g_config.language = value;\n', '')
    first = source.index('  } else if (section == 4) {')
    last = source.index('  }\n  return false;\n}', first)
    source = source[:first] + source[last:]
    source = replace(source, '"zelda3.user.ini"', '"sm.user.ini"')
    source = replace(source, '"zelda3.ini"', '"sm.ini"')
    # Input allocations and recursive temporary paths belong to the Rust owner.
    header = replace(header, "void ParseConfigFile(const char *filename);", "void ParseConfigFile(void *owner, const char *filename);")
    source = replace(source, 'static bool ParseOneConfigFile(const char *filename, int depth)',
                     'bool sm_config_parse_one_file(void *owner, const char *filename, int depth)')
    source = replace(source, '(char*)ReadWholeFile(filename, NULL)', 'sm_config_read_file(owner, filename)')
    source = replace(source, 'void ParseConfigFile(const char *filename)',
                     'void ParseConfigFile(void *owner, const char *filename)')
    source = replace(source, 'ParseOneConfigFile("sm.user.ini", 0)',
                     'sm_config_parse_one_file(owner, "sm.user.ini", 0)')
    source = replace(source, 'ParseOneConfigFile(filename, 0)',
                     'sm_config_parse_one_file(owner, filename, 0)')
    source = replace(source, '      char *new_filename = ReplaceFilenameWithNewPath(filename, NextPossiblyQuotedString(&tt));\n'
                            '      if (depth > 10 || !ParseOneConfigFile(new_filename, depth + 1))\n'
                            '        fprintf(stderr, "Warning: Unable to read %s\\n", new_filename);\n'
                            '      free(new_filename);',
                     '      sm_config_include(owner, filename, NextPossiblyQuotedString(&tt), depth);')
    # Use the same writable FILE pointer exposed by the native frontend.
    source = source.replace('fprintf(stderr,', 'fprintf(sm_config_stderr(),')
    return rename_symbols(source), rename_symbols(header)


def write_stamp(root, path, prefix, inputs):
    blobs = [(root / name).read_bytes() for name in inputs]
    declarations = [f"static const unsigned char {prefix}_source_{i}[] = {{" +
                    ",".join(str(byte) for byte in data) + "};" for i, data in enumerate(blobs)]
    declarations.append(f"static const unsigned char *const {prefix}_source_bytes[] = {{" +
                        ",".join(f"{prefix}_source_{i}" for i in range(len(blobs))) + "};")
    declarations.append(f"static const size_t {prefix}_source_lengths[] = {{" +
                        ",".join(str(len(data)) for data in blobs) + "};")
    path.write_text("\n".join(declarations) + "\n", encoding="ascii")


SHADER_INPUTS = ("sm/src/platform/glsl_shader.rs", "sm/src/native_shader.c",
                 "sm/src/native_shader.h", "sm/src/platform/texture_png.rs",
                 "sm/src/platform/opengl.rs", "sm/src/types.rs", "tools/install_native_config.py")


def shader_source(source):
    # The published parser/render algorithms stay external. These substitutions
    # retain native resource scopes, diagnostics, initialization and conversions.
    source = source[source.index("static GlslPass *ParseConfigKeyPass"):]
    source = replace(source, '  GlslTexture *t = calloc(sizeof(GlslTexture), 1);',
                     '  GlslTexture *t = sm_shader_allocate(sizeof(*t), _Alignof(GlslTexture), false);')
    source = replace(source, '  GlslParam *p = (GlslParam *)calloc(1, sizeof(GlslParam));',
                     '  GlslParam *p = sm_shader_allocate(sizeof(*p), _Alignof(GlslParam), false);')
    source = replace(source, '  gs->pass = (GlslPass *)calloc(gs->n_pass + 1, sizeof(GlslPass));',
                     '  gs->pass = sm_shader_allocate((gs->n_pass + 1) * sizeof(GlslPass), _Alignof(GlslPass), false);')
    # CString owners are distinct from the existing malloc-backed file buffers.
    source = source.replace('strdup(', 'sm_shader_clone_string(').replace('StrSet(', 'sm_shader_set_string(')
    source = source.replace('ReplaceFilenameWithNewPath(', 'sm_shader_relative_path(')
    source = source.replace('  char *data = (char *)ReadWholeFile(filename, NULL), *data_org = data, *line;',
                            '  char *data = (char *)ReadWholeFile(filename, NULL), *line;\n'
                            '  char *data_org __attribute__((cleanup(sm_shader_drop_file))) = data;')
    if source.count('__attribute__((cleanup(sm_shader_drop_file)))') != 2:
        raise ValueError('Pinned shader file scopes changed')
    source = source.replace('  free(data_org);\n', '')
    source = replace(source, '      char *new_filename = sm_shader_relative_path(filename, NextPossiblyQuotedString(&tt));',
                     '      char *new_filename __attribute__((cleanup(sm_shader_drop_string))) =\n'
                     '          sm_shader_relative_path(filename, NextPossiblyQuotedString(&tt));')
    source = replace(source, '      GlslShader_ReadShaderFile(gs, new_filename, result);\n      free(new_filename);',
                     '      GlslShader_ReadShaderFile(gs, new_filename, result);')
    source = replace(source, 'ByteArray *result)', 'void *result)')
    source = replace(source, "      line[linelen] = '\\n';\n      ByteArray_AppendData(result, (uint8 *)line, linelen + 1);",
                     '      sm_shader_append_line(result, (const uint8 *)line, linelen);')
    first = source.index('size_t LengthOfInitialComments(')
    last = source.index('static bool GlslPass_Compile(', first)
    source = source[:first] + source[last:]
    source = replace(source, 'GlslPass_Compile(GlslPass *p, uint type,',
                     'GlslPass_Compile(GlslPass *p, const char *filename, uint type,')
    source = replace(source, '  size_t commsize = LengthOfInitialComments(data, size);\n  data += commsize, size -= commsize;\n', '')
    source = replace(source, 'type == GL_VERTEX_SHADER ? "vertex" : "fragment", p->filename, buffer);',
                     'type == GL_VERTEX_SHADER ? "vertex" : "fragment", filename, buffer);')
    source = source.replace('  char buffer[256];', '  char buffer[256] = {0};')
    source = replace(source, '    for (GlslParam *pa = gs->first_param; pa != NULL; pa = pa->next)\n      pa->uniform[pass_idx] = glGetUniformLocation(program, pa->id);',
                     '    for (GlslParam *pa = gs->first_param; pa != NULL; pa = pa->next) {\n'
                     '      int location = glGetUniformLocation(program, pa->id);\n'
                     '      sm_shader_check_index(pass_idx, 20);\n'
                     '      pa->uniform[pass_idx] = location;\n    }')
    first = source.index('GlslShader *GlslShader_CreateFromFile(')
    last = source.index('\nvoid GlslShader_Destroy(', first)
    create = source[first:last]
    loop = create.index('  for (int i = 1; i <= gs->n_pass; i++) {')
    success = create.index('  success = true;')
    body = create[loop:success].replace('goto FAIL;', 'return false;').replace(', opengl_es)', ', false)')
    body = replace(body, '    shader_code.size = 0;\n', '')
    body = replace(body, '    char *new_filename = sm_shader_relative_path(filename, p->filename);\n'
                         '    GlslShader_ReadShaderFile(gs, new_filename, &shader_code);\n    free(new_filename);',
                   '    char *name __attribute__((cleanup(sm_shader_drop_string))) = sm_shader_clone_string(p->filename);\n'
                   '    sm_shader_source_clear(shader_code);\n    {\n'
                   '      char *path __attribute__((cleanup(sm_shader_drop_string))) = sm_shader_relative_path(filename, name);\n'
                   '      GlslShader_ReadShaderFile(gs, path, shader_code);\n    }')
    body = body.replace('shader_code.size', 'sm_shader_source_length(shader_code)')
    body = body.replace('shader_code.data', 'sm_shader_source_data(shader_code)')
    body = body.replace('GlslPass_Compile(p, GL_', 'GlslPass_Compile(p, name, GL_')
    body = body.replace('p->filename);', 'name);')
    # The clone input must remain the persistent filename, not the new local.
    body = replace(body, 'sm_shader_clone_string(name);', 'sm_shader_clone_string(p->filename);')
    body = replace(body, '    glLinkProgram(p->gl_program);',
                   '    glLinkProgram(p->gl_program);\n    GLint link_status = 0;\n    char buffer[256] = {0};')
    body = replace(body, 'link_status != GL_TRUE ? "Error" : "While", p->filename, buffer);',
                   'link_status != GL_TRUE ? "Error" : "While", name, buffer);')
    first_png = body.index('    if (t->filename) {')
    last_png = body.index('    if (t->mipmap)', first_png)
    body = body[:first_png] + '    if (t->filename)\n      sm_shader_upload_png(filename, t->filename);\n' + body[last_png:]
    body = 'bool sm_shader_load_body(GlslShader *gs, const char *filename, void *shader_code) {\n' + body + '  return true;\n}\n\n'
    create = create[:loop]
    create = replace(create, '  char buffer[256] = {0};\n  GLint link_status;\n  ByteArray shader_code = { 0 };\n  bool success = false;\n', '')
    create = replace(create, '  GlslShader *gs = (GlslShader *)calloc(sizeof(GlslShader), 1);',
                     '  GlslShader *gs = sm_shader_allocate(sizeof(*gs), _Alignof(GlslShader), true);')
    create += '  if (sm_shader_load_resources(gs, filename))\n    return gs;\nFAIL:\n  sm_shader_destroy(gs);\n  return NULL;\n}\n'
    source = source[:first] + body + create + source[last:]
    first = source.index('void GlslShader_Destroy(')
    last = source.index('\nenum {', first)
    destroy = source[first:last]
    destroy = destroy.replace('GlslShader_Destroy(', 'sm_shader_destroy_gl(')
    destroy = replace(destroy, '  for (GlslPass *p = gs->pass + 1, *p_end = p + gs->n_pass; p < p_end; p++) {',
                      '  for (int i = 1; i <= gs->n_pass; i++) {\n    GlslPass *p = gs->pass + i;')
    destroy = replace(destroy, '    free(p->filename);\n', '')
    destroy = replace(destroy, '  free(gs->pass);\n', '')
    first_texture = destroy.index('  GlslTexture *t;')
    last_texture = destroy.index('  for (int i = 0; i < 8; i++)', first_texture)
    destroy = destroy[:first_texture] + '  for (GlslTexture *t = gs->first_texture; t; t = t->next)\n    glDeleteTextures(1, &t->gl_texture);\n' + destroy[last_texture:]
    destroy = replace(destroy, '  free(gs);\n', '')
    source = source[:first] + destroy + source[last:]
    source = replace(source, '    assert(ctx->num_vaos < kMaxVaosInRenderCtx);',
                     '    sm_shader_assert(ctx->num_vaos < kMaxVaosInRenderCtx, 0);\n'
                     '    sm_shader_check_index(ctx->num_vaos, 51);')
    source = replace(source, '    assert(t->gl_texture != 0);', '    sm_shader_assert(t->gl_texture != 0, 1);')
    source = replace(source, '    assert(store_pos->gl_texture == 0);',
                     '    sm_shader_assert(store_pos->gl_texture == 0, 2);')
    source = replace(source, '  for (GlslParam *pa = gs->first_param; pa != NULL; pa = pa->next)\n'
                             '    if (pa->uniform[pass] >= 0)\n      glUniform1f(pa->uniform[pass], pa->value);',
                     '  for (GlslParam *pa = gs->first_param; pa != NULL; pa = pa->next) {\n'
                     '    sm_shader_check_index(pass, 20);\n'
                     '    glUniform1f(pa->uniform[pass], pa->value);\n  }')
    first = source.index('    switch (p->scale_type_x) {')
    last = source.index('    if (!last_pass) {', first)
    source = source[:first] + ('    p->width = sm_shader_dimension(p->scale_type_x, p->scale_x, p[-1].width, viewport_width, last_pass);\n'
                              '    p->height = sm_shader_dimension(p->scale_type_y, p->scale_y, p[-1].height, viewport_height, last_pass);\n\n') + source[last:]
    source = replace(source, '  GLint previous_framebuffer;', '  GLint previous_framebuffer = 0;')
    source = replace(source, '    RenderCtx ctx;', '    RenderCtx ctx = {0};')
    source = replace(source, '    GlslShader_SetShaderVars(gs, &ctx, pass);',
                     '    bool srgb = p->srgb_framebuffer;\n    GlslShader_SetShaderVars(gs, &ctx, pass);')
    source = replace(source, '    if (p->srgb_framebuffer)\n      glDisable', '    if (srgb)\n      glDisable')
    source = source.replace('g_config.linear_filtering', 'sm_shader_linear_filtering()')
    symbols = {name: 'sm_util_' + name for name in UTILITY_NAMES}
    symbols.update({'ParseBool': 'sm_config_parse_bool', 'atoi': 'sm_config_atoi', 'atof': 'sm_shader_number',
                    'GlslShader_CreateFromFile': 'sm_shader_create', 'GlslShader_Render': 'sm_shader_render'})
    source = re.sub(r'\b(?:' + '|'.join(symbols) + r')\b', lambda m: symbols[m[0]], source)
    source = re.sub(r'\bgl(?=[A-Z])', 'sm_shader_gl.', source)
    source = source.replace('fprintf(stderr,', 'fprintf(sm_config_stderr(),')
    return source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--zig", required=True, type=Path)
    parser.add_argument("--target", required=True, choices=("x86_64-windows-gnu", "x86_64-linux-gnu"))
    parser.add_argument("--sdl-include", required=True, type=Path)
    parser.add_argument("--prefix", type=Path)
    parser.add_argument("--library-directory", type=Path)
    parser.add_argument("--emit-only", action="store_true")
    args = parser.parse_args()
    if args.prefix is None:
        cache = Path(os.environ["LOCALAPPDATA"]) if os.name == "nt" else Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
        args.prefix = cache / "sm-decomp-tools/native-config-45a149d8"
    target_flags = ["-fPIC", "-D_DEFAULT_SOURCE"] if args.target == "x86_64-linux-gnu" else []
    root = Path(__file__).resolve().parents[1]
    source = args.prefix / "source"
    library = args.library_directory or args.prefix / "lib"
    library.mkdir(parents=True, exist_ok=True)
    for name, expected in FILES.items():
        path = source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.exists():
            urllib.request.urlretrieve(f"https://raw.githubusercontent.com/snesrev/zelda3/{REVISION}/{name}", path)
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError(f"Upstream source checksum mismatch: {name}")
    text, header = config_sources((source / "src/config.c").read_text(), (source / "src/config.h").read_text())
    (library / "native_config_selected.c").write_text(text, encoding="utf-8")
    (library / "config.h").write_text(header, encoding="utf-8")
    (library / "native_util_selected.c").write_text(utility_source((source / "src/util.c").read_text()), encoding="utf-8")
    write_stamp(root, library / "native_config_sources.h", "sm_config", NATIVE_INPUTS)
    write_stamp(root, library / "native_util_sources.h", "sm_util", UTILITY_INPUTS)
    (library / "glsl_shader.h").write_bytes((source / "src/glsl_shader.h").read_bytes())
    (library / "native_shader_selected.c").write_text(shader_source((source / "src/glsl_shader.c").read_text()), encoding="utf-8")
    write_stamp(root, library / "native_shader_sources.h", "sm_shader", SHADER_INPUTS)
    if args.emit_only:
        return
    objects = []
    for owner in ("config", "util", "shader"):
        obj = library / f"native_{owner}.o"
        subprocess.run([str(args.zig), "cc", "-target", args.target, "-O2", "-std=c11", *target_flags,
                        "-fexceptions", "-fwrapv", "-ffp-contract=off", "-fno-strict-aliasing", "-ffunction-sections", "-fdata-sections",
                        "-I", str(library), "-I", str(source / "src"), "-I", str(root / "sm/src"),
                        "-I", str(args.sdl_include), "-c", str(root / f"sm/src/native_{owner}.c"), "-o", str(obj)], check=True)
        objects.append(str(obj))
    subprocess.run([str(args.zig), "ar", "rcs", str(library / "libsm_config.a"), *objects], check=True)
    print(f"Installed native configuration library in {library}; add this directory to LIBRARY_PATH.")


if __name__ == "__main__":
    main()
