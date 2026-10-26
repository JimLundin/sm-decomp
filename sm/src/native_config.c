// Includes only the independently published, checked and adapted config source.
#include "native_config.h"
#include "native_config_selected.c"
#include "native_config_sources.h"

_Static_assert(sizeof(Config) == 72, "native Config size");
_Static_assert(sizeof(KeyMapHashEnt) == 6, "native keyboard entry size");
_Static_assert(sizeof(GamepadMapEnt) == 8, "native gamepad entry size");
_Static_assert(kKeys_Total == 130, "native key command count");
_Static_assert(kGamepadBtn_Count == 17, "native gamepad button count");

_Static_assert(offsetof(Config, window_width) == 0, "native window_width");
_Static_assert(offsetof(Config, window_height) == 4, "native window_height");
_Static_assert(offsetof(Config, enhanced_mode7) == 8, "native enhanced_mode7");
_Static_assert(offsetof(Config, new_renderer) == 9, "native new_renderer");
_Static_assert(offsetof(Config, ignore_aspect_ratio) == 10, "native ignore_aspect_ratio");
_Static_assert(offsetof(Config, fullscreen) == 11, "native fullscreen");
_Static_assert(offsetof(Config, window_scale) == 12, "native window_scale");
_Static_assert(offsetof(Config, enable_audio) == 13, "native enable_audio");
_Static_assert(offsetof(Config, linear_filtering) == 14, "native linear_filtering");
_Static_assert(offsetof(Config, output_method) == 15, "native output_method");
_Static_assert(offsetof(Config, audio_freq) == 16, "native audio_freq");
_Static_assert(offsetof(Config, audio_channels) == 18, "native audio_channels");
_Static_assert(offsetof(Config, audio_samples) == 20, "native audio_samples");
_Static_assert(offsetof(Config, autosave) == 22, "native autosave");
_Static_assert(offsetof(Config, extended_aspect_ratio) == 23, "native extended_aspect_ratio");
_Static_assert(offsetof(Config, extend_y) == 24, "native extend_y");
_Static_assert(offsetof(Config, no_sprite_limits) == 25, "native no_sprite_limits");
_Static_assert(offsetof(Config, display_perf_title) == 26, "native display_perf_title");
_Static_assert(offsetof(Config, enable_msu) == 27, "native enable_msu");
_Static_assert(offsetof(Config, resume_msu) == 28, "native resume_msu");
_Static_assert(offsetof(Config, disable_frame_delay) == 29, "native disable_frame_delay");
_Static_assert(offsetof(Config, msuvolume) == 30, "native msuvolume");
_Static_assert(offsetof(Config, features0) == 32, "native features0");
_Static_assert(offsetof(Config, link_graphics) == 40, "native link_graphics");
_Static_assert(offsetof(Config, memory_buffer) == 48, "native memory_buffer");
_Static_assert(offsetof(Config, shader) == 56, "native shader");
_Static_assert(offsetof(Config, msu_path) == 64, "native msu_path");

bool sm_config_matches_source(uint32_t index, const uint8_t *bytes, size_t length) {
  return index < 6 && length == sm_config_source_lengths[index] &&
         memcmp(bytes, sm_config_source_bytes[index], length) == 0;
}
