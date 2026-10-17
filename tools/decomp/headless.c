// Single-threaded host for the pinned MIT snesrev/sm implementation.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "ida_types.h"
#include "variables.h"
#include "spc_player.h"
#include <stdlib.h>
#include <string.h>
#include "snes/cart.h"
#include "headless_frame.h"

bool g_debug_flag, g_is_turbo, g_other_image;
bool g_new_ppu = true;
int g_got_mismatch_count;
SpcPlayer *g_spc_player;
extern uint8 g_runmode;
static unsigned long fallback_calls;
static unsigned long fallback_instructions;
static unsigned long decompression_calls;
static uint8 pixels[512 * 480 * 4];

static void SaveRoutineState(const char *phase) {
  char name[96];
  snprintf(name, sizeof(name), "fixtures/decompress-%04lu-%s.wram", decompression_calls, phase);
  FILE *file = fopen(name, "wb");
  if (!file || fwrite(g_ram, 1, sizeof(g_ram), file) != sizeof(g_ram) || fclose(file)) {
    fprintf(stderr, "unable to preserve routine fixture %s\n", name);
    exit(2);
  }
}

// Linker wrapping observes C-to-CPU fallback without modifying the oracle.
void __real_Call(uint32 pc);
void __wrap_Call(uint32 pc) {
  fallback_calls++;
  fprintf(stderr, "fallback frame=%d address=%06x\n", snes_frame_counter, pc);
  __real_Call(pc);
}
extern bool g_calling_asm_from_c;
void __real_cpu_runOpcode(Cpu *cpu);
void __wrap_cpu_runOpcode(Cpu *cpu) {
  if (g_calling_asm_from_c) fallback_instructions++;
  __real_cpu_runOpcode(cpu);
}
void __real_DecompressToMem(uint32 src, uint8 *dst);
void __wrap_DecompressToMem(uint32 src, uint8 *dst) {
  decompression_calls++;
  fprintf(stderr, "decompression frame=%d source=%06x destination=%ld\n",
          snes_frame_counter, src, (long)(dst - g_ram));
  char rom_name[96];
  snprintf(rom_name, sizeof(rom_name), "fixtures/decompress-%04lu.rom", decompression_calls);
  FILE *rom = fopen(rom_name, "wb");
  if (!rom || fwrite(g_rom, 1, g_snes->cart->romSize, rom) != g_snes->cart->romSize || fclose(rom)) exit(2);
  SaveRoutineState("before");
  __real_DecompressToMem(src, dst);
  SaveRoutineState("after");
  FILE *manifest = fopen("fixtures/calls.bin", "ab+");
  if (manifest && fseek(manifest, 0, SEEK_END) != 0) exit(2);
  if (!manifest) exit(2);
  if (ftell(manifest) == 0 && fwrite("SMDC2", 1, 5, manifest) != 5) exit(2);
  const uint64_t fields[] = {decompression_calls, snes_frame_counter, src, (uint64_t)(dst - g_ram)};
  for (unsigned field = 0; field < 4; field++) {
    uint8_t encoded[8];
    for (unsigned byte = 0; byte < 8; byte++) encoded[byte] = (uint8_t)(fields[field] >> (byte * 8));
    if (fwrite(encoded, 1, 8, manifest) != 8) exit(2);
  }
  if (fclose(manifest)) exit(2);
}

// The host has no audio thread; preserve the real player and hardware routines.
void RtlApuLock(void) {}
void RtlApuUnlock(void) {}
void Die(const char *error) { fprintf(stderr, "%s\n", error); exit(2); }
void Warning(const char *error) { fprintf(stderr, "%s\n", error); exit(2); }

int main(int argc, char **argv) {
  bool control = argc == 4 && !strcmp(argv[1], "--observation-control");
  if (argc != 5 && !control) {
    fprintf(stderr, "usage: headless ROM INPUTS FRAMES STATE_OUTPUT\n");
    return 2;
  }
  if (control) {
    if (!SnesInit(argv[2])) return 2;
    g_spc_player = SpcPlayer_Create();
    SpcPlayer_Initialize(g_spc_player);
    g_runmode = 1;
    RtlReset(0);
    // Isolated observation control, never part of candidate replay: execute
    // an actual C-to-CPU fallback and an actual cross-unit decompression call.
    uint8 *code = (uint8 *)RomPtr(0xff8000);
    const uint8 program[] = {0xa9, 0x42, 0x00, 0x8f, 0x00, 0x10, 0x7e, 0x6b}; // LDA #$0042; STA $7e1000; RTL
    memcpy(code, program, sizeof(program));
    g_cpu->sp = 0x1ff;
    Call(0xff8000);
    if (g_ram[0x1000] != 0x42) return 3;
    uint8 *stream = (uint8 *)RomPtr(0xff8100);
    const uint8 literal[] = {0x02, 0x12, 0x34, 0x56, 0xff};
    memcpy(stream, literal, sizeof(literal));
    DecompressToMem(0xff8100, g_ram + 0x12000);
    if (memcmp(g_ram + 0x12000, literal + 1, 3)) return 3;
    printf("control %lu %lu %lu\n", fallback_calls, fallback_instructions, decompression_calls);
    return 0;
  }
  char *end;
  long frames = strtol(argv[3], &end, 10);
  if (*end || frames < 1 || frames > 1000000) return 2;
  FILE *inputs = fopen(argv[2], "r");
  if (!inputs || !SnesInit(argv[1])) return 2;
  FILE *rom_view = fopen("rom-view.bin", "wb");
  if (!rom_view || fwrite(g_rom, 1, g_snes->cart->romSize, rom_view) != g_snes->cart->romSize ||
      fclose(rom_view)) return 2;
  g_spc_player = SpcPlayer_Create();
  SpcPlayer_Initialize(g_spc_player);
  g_runmode = 1; // RM_MINE: candidate runs independently; no comparison restoration.
  RtlReset(0);
  PpuBeginDrawing(g_snes->my_ppu, pixels, 512 * 4, 0);
  PpuBeginDrawing(g_snes->snes_ppu, pixels, 512 * 4, 0);
  for (long frame = 0; frame < frames; frame++) {
    unsigned pad;
    if (fscanf(inputs, "%x", &pad) != 1 || pad > 0xffff) return 2;
    // Imported .inputs uses SNES serial order; upstream host uses bit-reversed order.
    unsigned reversed = 0;
    for (int bit = 0; bit < 16; bit++) reversed |= ((pad >> bit) & 1) << (15 - bit);
    HeadlessPrepareFrame();
    RtlRunFrame(reversed);
    if (g_fail) { fprintf(stderr, "candidate error frame=%ld\n", frame); return 3; }
    if (frame == 0 || (frame + 1) % 1000 == 0 || frame + 1 == frames) {
      printf("progress %ld %u %u %u %u %u %lu %lu %lu\n",
             frame + 1, game_state, room_ptr, samus_x_pos, samus_y_pos, random_number,
             fallback_calls, fallback_instructions, decompression_calls);
      fflush(stdout);
    }
  }
  FILE *state = fopen(argv[4], "wb");
  if (!state || fwrite(g_ram, 1, sizeof(g_ram), state) != sizeof(g_ram) ||
      fwrite(g_sram, 1, 0x2000, state) != 0x2000 || fclose(state)) return 2;
  fclose(inputs);
  return 0;
}
