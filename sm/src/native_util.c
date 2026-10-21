// Only the selected published utility functions are included in this object.
#include "native_util.h"
#include "types.h"
#include <limits.h>
#include <stdio.h>
#include <string.h>
#include "native_util_selected.c"
#include "native_util_sources.h"

_Static_assert(sizeof(ByteArray) == 24, "native ByteArray size");
_Static_assert(offsetof(ByteArray, size) == 8, "native ByteArray size offset");
_Static_assert(offsetof(ByteArray, capacity) == 16, "native ByteArray capacity offset");
_Static_assert(sizeof(long) == 4 && CHAR_MIN < 0, "native Windows long and char widths");

bool sm_util_matches_source(uint32_t index, const uint8_t *bytes, size_t length) {
  return index < 5 && length == sm_util_source_lengths[index] &&
         memcmp(bytes, sm_util_source_bytes[index], length) == 0;
}
