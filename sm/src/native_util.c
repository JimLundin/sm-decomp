// Only the selected published utility functions are included in this object.
#include "native_util.h"
#include "types.h"
#include <limits.h>
#include <stdio.h>
#include <string.h>
#include "native_util_selected.c"
#include "native_util_sources.h"

// Windows long widths are explicit int32_t/uint32_t casts in the selected source.
_Static_assert(CHAR_MIN < 0, "native Windows char signedness");

bool sm_util_matches_source(uint32_t index, const uint8_t *bytes, size_t length) {
  return index < 5 && length == sm_util_source_lengths[index] &&
         memcmp(bytes, sm_util_source_bytes[index], length) == 0;
}
