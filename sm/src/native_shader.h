// Narrow ABI for the published shader and pointers captured from the Rust loader.
#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include "glsl_shader.h"
#include "native_config.h"

typedef char GLchar;
typedef int GLint;
typedef float GLfloat;

enum {
  GL_ARRAY_BUFFER = 0x8892,
  GL_CLAMP = 0x2900,
  GL_CLAMP_TO_BORDER = 0x812D,
  GL_CLAMP_TO_EDGE = 0x812F,
  GL_COLOR_ATTACHMENT0 = 0x8CE0,
  GL_COMPILE_STATUS = 0x8B81,
  GL_FALSE = 0,
  GL_FLOAT = 0x1406,
  GL_FRAGMENT_SHADER = 0x8B30,
  GL_FRAMEBUFFER = 0x8D40,
  GL_FRAMEBUFFER_BINDING = 0x8CA6,
  GL_FRAMEBUFFER_SRGB = 0x8DB9,
  GL_LINEAR = 0x2601,
  GL_LINEAR_MIPMAP_LINEAR = 0x2703,
  GL_LINK_STATUS = 0x8B82,
  GL_LUMINANCE = 0x1909,
  GL_NEAREST = 0x2600,
  GL_NEAREST_MIPMAP_NEAREST = 0x2700,
  GL_REPEAT = 0x2901,
  GL_RGB = 0x1907,
  GL_RGBA = 0x1908,
  GL_RGBA32F = 0x8814,
  GL_SRGB8_ALPHA8 = 0x8C43,
  GL_STATIC_DRAW = 0x88E4,
  GL_TEXTURE0 = 0x84C0,
  GL_TEXTURE_2D = 0x0DE1,
  GL_TEXTURE_MAG_FILTER = 0x2800,
  GL_TEXTURE_MIN_FILTER = 0x2801,
  GL_TEXTURE_WRAP_S = 0x2802,
  GL_TEXTURE_WRAP_T = 0x2803,
  GL_TRIANGLE_STRIP = 0x0005,
  GL_TRUE = 1,
  GL_UNSIGNED_BYTE = 0x1401,
  GL_UNSIGNED_INT_8_8_8_8 = 0x8035,
  GL_VERTEX_SHADER = 0x8B31,
};

typedef struct ShaderGl {
  void (*ActiveTexture)(uint32_t texture);
  void (*AttachShader)(uint32_t program, uint32_t shader);
  void (*BindBuffer)(uint32_t target, uint32_t buffer);
  void (*BindFramebuffer)(uint32_t target, uint32_t framebuffer);
  void (*BindTexture)(uint32_t target, uint32_t texture);
  void (*BufferData)(uint32_t target, intptr_t size, void const * data, uint32_t usage);
  void (*CompileShader)(uint32_t shader);
  uint32_t (*CreateProgram)(void);
  uint32_t (*CreateShader)(uint32_t type_);
  void (*DeleteBuffers)(int32_t n, uint32_t const * buffers);
  void (*DeleteFramebuffers)(int32_t n, uint32_t const * framebuffers);
  void (*DeleteProgram)(uint32_t program);
  void (*DeleteShader)(uint32_t shader);
  void (*DeleteTextures)(int32_t n, uint32_t const * textures);
  void (*Disable)(uint32_t cap);
  void (*DisableVertexAttribArray)(uint32_t index);
  void (*DrawArrays)(uint32_t mode, int32_t first, int32_t count);
  void (*Enable)(uint32_t cap);
  void (*EnableVertexAttribArray)(uint32_t index);
  void (*FramebufferTexture2D)(uint32_t target, uint32_t attachment, uint32_t textarget, uint32_t texture, int32_t level);
  void (*GenBuffers)(int32_t n, uint32_t * buffers);
  void (*GenFramebuffers)(int32_t n, uint32_t * framebuffers);
  void (*GenTextures)(int32_t n, uint32_t * textures);
  void (*GenerateMipmap)(uint32_t target);
  int32_t (*GetAttribLocation)(uint32_t program, char const * name);
  void (*GetIntegerv)(uint32_t pname, int32_t * data);
  void (*GetProgramInfoLog)(uint32_t program, int32_t bufSize, int32_t * length, char * infoLog);
  void (*GetProgramiv)(uint32_t program, uint32_t pname, int32_t * params);
  void (*GetShaderInfoLog)(uint32_t shader, int32_t bufSize, int32_t * length, char * infoLog);
  void (*GetShaderiv)(uint32_t shader, uint32_t pname, int32_t * params);
  int32_t (*GetUniformLocation)(uint32_t program, char const * name);
  void (*LinkProgram)(uint32_t program);
  void (*ShaderSource)(uint32_t shader, int32_t count, char const * const * string, int32_t const * length);
  void (*TexImage2D)(uint32_t target, int32_t level, int32_t internalformat, int32_t width, int32_t height, int32_t border, uint32_t format, uint32_t type_, void const * pixels);
  void (*TexParameteri)(uint32_t target, uint32_t pname, int32_t param);
  void (*Uniform1f)(int32_t location, float v0);
  void (*Uniform1i)(int32_t location, int32_t v0);
  void (*Uniform2fv)(int32_t location, int32_t count, float const * value);
  void (*UniformMatrix4fv)(int32_t location, int32_t count, uint8_t transpose, float const * value);
  void (*UseProgram)(uint32_t program);
  void (*VertexAttribPointer)(uint32_t index, int32_t size, uint32_t type_, uint8_t normalized, int32_t stride, void const * pointer);
  void (*Viewport)(int32_t x, int32_t y, int32_t width, int32_t height);
} ShaderGl;
extern ShaderGl sm_shader_gl;

bool sm_config_parse_bool(const char *value, bool *result);
void *sm_shader_allocate(size_t size, size_t alignment, bool nullable);
void sm_shader_deallocate(void *pointer, size_t size, size_t alignment);
char *sm_shader_clone_string(const char *value);
void sm_shader_free_string(char *value);
void sm_shader_set_string(char **target, const char *value);
char *sm_shader_relative_path(const char *base, const char *name);
void sm_shader_upload_png(const char *base, const char *name);
float sm_shader_number(const char *value);
bool sm_shader_linear_filtering(void);
uint16_t sm_shader_dimension(uint8_t kind, float scale, uint16_t source, int viewport, bool last);
void sm_shader_check_index(size_t index, size_t length);
void sm_shader_assert(bool condition, uint32_t site);
void sm_shader_destroy(GlslShader *shader);
bool sm_shader_load_resources(GlslShader *shader, const char *filename);
void sm_shader_source_clear(void *source);
const uint8_t *sm_shader_source_data(void *source);
size_t sm_shader_source_length(void *source);
void sm_shader_append_line(void *source, const uint8_t *line, size_t length);
