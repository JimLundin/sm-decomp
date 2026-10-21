// Scope cleanup is enabled for foreign unwinds with -fexceptions.
#include "native_shader.h"
#include <stdlib.h>
#include <string.h>

static void sm_shader_drop_file(char **value) {
  free(*value);
}

static void sm_shader_drop_string(char **value) {
  sm_shader_free_string(*value);
}

#include "native_shader_selected.c"
#include "native_shader_sources.h"

_Static_assert(sizeof(ShaderGl) == 42 * sizeof(void *), "native GL table size");
_Static_assert(_Alignof(ShaderGl) == _Alignof(void *), "native GL table alignment");
_Static_assert(sizeof(GlTextureWithSize) == 8, "native image size");
_Static_assert(offsetof(GlTextureWithSize, width) == 4, "native image width");
_Static_assert(offsetof(GlTextureWithSize, height) == 6, "native image height");

// All GL deletion precedes owned-field release, matching Rust drop glue.
void sm_shader_free_heap(GlslShader *shader) {
  for (int i = 0; i <= shader->n_pass && shader->pass; i++)
    sm_shader_free_string(shader->pass[i].filename);
  sm_shader_deallocate(shader->pass, (shader->n_pass + 1) * sizeof(GlslPass), _Alignof(GlslPass));
  for (GlslParam *p = shader->first_param, *next; p; p = next) {
    next = p->next;
    sm_shader_free_string(p->id);
    sm_shader_deallocate(p, sizeof(*p), _Alignof(GlslParam));
  }
  for (GlslTexture *t = shader->first_texture, *next; t; t = next) {
    next = t->next;
    sm_shader_free_string(t->id);
    sm_shader_free_string(t->filename);
    sm_shader_deallocate(t, sizeof(*t), _Alignof(GlslTexture));
  }
  sm_shader_deallocate(shader, sizeof(*shader), _Alignof(GlslShader));
}

bool sm_shader_matches_source(uint32_t index, const uint8_t *bytes, size_t length) {
  return index < 7 && length == sm_shader_source_lengths[index] &&
         memcmp(bytes, sm_shader_source_bytes[index], length) == 0;
}
