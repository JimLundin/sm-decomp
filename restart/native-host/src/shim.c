// Minimal process-lifetime bridge to the pinned recovered C. All application,
// build, scenario and verification logic lives in Rust.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "spc_player.h"
#include "snes/cart.h"
#include "snes/input.h"
#include <stdlib.h>
#include <string.h>

bool g_debug_flag, g_is_turbo, g_other_image;
bool g_new_ppu = true;
int g_got_mismatch_count;
SpcPlayer *g_spc_player;
extern uint8 g_runmode;
extern bool g_calling_asm_from_c;

// Events: actual poll entry, completed scheduler update, NMI entry, yielded
// explicit wait, native scheduler entry, C-to-CPU call, fallback instruction.
static uint64_t counts[7];
static uint32_t pixels[256 * 240];
static uint64_t steps;
static uint16_t requested_buttons;
static int booted;

void sm_native_observe(unsigned event) {
  if (event < 7) counts[event]++;
  if (event < 4) sm_native_effect(9, event, (uint32_t)counts[event], (uint32_t)(counts[event] >> 32));
}
void sm_native_cpu_instruction(void) {
  if (g_calling_asm_from_c) counts[6]++;
}
void RtlApuLock(void) {}
void RtlApuUnlock(void) {}
void Die(const char *error) { fprintf(stderr, "native fatal: %s\n", error); exit(2); }
void Warning(const char *error) { fprintf(stderr, "native warning: %s\n", error); exit(2); }

int sm_native_boot(const char *rom) {
  if (booted) return -1;
  booted = 1;
  if (!SnesInit(rom)) return -2;
  g_spc_player = SpcPlayer_Create();
  if (!g_spc_player) return -3;
  SpcPlayer_Initialize(g_spc_player);
  g_runmode = 1; // RM_MINE, no comparison-state restoration.
  RtlReset(0);   // Ordinary cold reset, zero SRAM.
  // snes_init mallocs this field without initializing it; upstream's GUI sets
  // it every frame. Headless and interactive callers must both render.
  g_snes->disableRender = false;
  PpuBeginDrawing(g_snes->my_ppu, (uint8 *)pixels, 256 * 4, 0);
  PpuBeginDrawing(g_snes->snes_ppu, (uint8 *)pixels, 256 * 4, 0);
  return 0;
}

int sm_native_step(uint16_t buttons, int16_t *audio) {
  if (!booted || !g_snes) return -1;
  requested_buttons = buttons;
  sm_native_begin_step();
  // Explicit compatibility fix from the upstream comparison driver: a native
  // soft-reset request must resume RESET before ffff indexes the state table.
  if (game_state == 0xffff) {
    g_snes->cpu->k = 0x80;
    g_snes->cpu->pc = 0x8462;
    coroutine_state_0 = 3;
  }
  RtlRunFrame(buttons);
  steps++;
  if (g_fail) return -2;
  // The real player generates 534 samples per step. One drain per step avoids
  // GUI/headless differences in the mutable APU queue and player state.
  RtlRenderAudio(audio, 534, 2);
  sm_native_capture(pixels, audio);
  return g_fail ? -3 : 0;
}

void sm_native_counts(uint64_t *out) {
  out[0] = steps;
  memcpy(out + 1, counts, sizeof(counts));
}
uint16_t sm_native_buttons(void) { return g_snes->input1->currentState; }
uint16_t sm_native_requested_buttons(void) { return requested_buttons; }
const uint32_t *sm_native_pixels(void) { return pixels; }
const uint8_t *sm_native_wram(void) { return g_ram; }
const uint8_t *sm_native_rom(void) { return g_rom; }
size_t sm_native_rom_size(void) { return g_snes->cart->romSize; }
