// Selected desktop algorithms from the pinned Zelda3 dependency.
#include "native_desktop.h"
#include "types.h"
#include "config.h"
#include <string.h>
#ifdef _WIN32
#include <direct.h>
#else
#include <sys/stat.h>
#include <unistd.h>
#endif
#include "native_desktop_selected.c"
#include "native_desktop_sources.h"

_Static_assert(sizeof(Config) == 72, "desktop Config layout");
_Static_assert(sizeof(RendererFuncs) == 32, "desktop renderer callback layout");
_Static_assert(sizeof(long) == 4, "desktop Windows pointer-difference conversion");

bool sm_desktop_matches_source(uint32_t index, const uint8_t *bytes, size_t length) {
  return index < 7 && length == sm_desktop_source_lengths[index] &&
         memcmp(bytes, sm_desktop_source_bytes[index], length) == 0;
}
