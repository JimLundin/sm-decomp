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
static bool desktop_initialized;
#include "native_desktop_selected.c"
#include "native_desktop_sources.h"

_Static_assert(sizeof(Config) == 72, "desktop Config layout");
_Static_assert(sizeof(RendererFuncs) == 32, "desktop renderer callback layout");
_Static_assert(sizeof(long) == 4, "desktop Windows pointer-difference conversion");

bool sm_desktop_matches_source(uint32_t index, const uint8_t *bytes, size_t length) {
  return index < 7 && length == sm_desktop_source_lengths[index] &&
         memcmp(bytes, sm_desktop_source_bytes[index], length) == 0;
}

// Called on normal return and Rust unwind, before the Game owner is released.
void sm_desktop_close(void *audio) {
  if (!desktop_initialized)
    return;
  desktop_initialized = false;
  if (g_audio_device) {
    SDL_PauseAudioDevice(g_audio_device, 1);
    SDL_CloseAudioDevice(g_audio_device);
    g_audio_device = 0;
  }
  SDL_DestroyMutex(sm_desktop_audio_mutex);
  sm_desktop_audio_mutex = NULL;
  sm_host_release_audio(audio);
  if (g_window) {
    // Audio has joined. Do not reacquire a Game gate poisoned by the unwind.
    g_renderer_funcs.Destroy();
    SDL_DestroyWindow(g_window);
    g_window = NULL;
  }
  SDL_Quit();
}

// Existing recursive SDL lock; no Game borrow enters this native device edge.
int sm_desktop_begin_audio(void) {
  if (SDL_LockMutex(sm_desktop_audio_mutex)) sm_host_die("Mutex lock failed!");
  return g_sdl_audio_mixer_volume;
}
void sm_desktop_end_audio(void) {
  SDL_UnlockMutex(sm_desktop_audio_mutex);
}
