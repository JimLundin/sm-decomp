// Preserve baseline host support; the scheduler, C banks and hardware are unchanged.
#define main baseline_main
#define __wrap_DecompressToMem baseline_decompression_observer
#include "headless.c"
#undef main
#undef __wrap_DecompressToMem
#include <stdint.h>
#include <unistd.h>
#include <zlib.h>
#include <sys/stat.h>
#include "snes/dma.h"
#include "snes/apu.h"

typedef struct RustReport { size_t written; uint32_t final_source, modes; } RustReport;
extern int sm_decompress_v1(const uint8_t *, size_t, uint8_t *, size_t, uint32_t, size_t, RustReport *);
static bool mixed, force_error;
static unsigned long rust_hits, c_hits;
static unsigned rust_modes;
static gzFile effects;
static int epoch;
#ifdef NEXT_RANDOM_JOB
#include "next_random_live.c"
#endif

void __real_ppu_write(Ppu *, uint8_t, uint8_t);
void __wrap_ppu_write(Ppu *ppu, uint8_t adr, uint8_t val) {
  gzprintf(effects, "%d ppu %d %02x %02x\n", snes_frame_counter, ppu == g_snes->my_ppu, adr, val);
  __real_ppu_write(ppu, adr, val);
}
void __real_RtlApuWrite(uint32 adr, uint8 val);
void __wrap_RtlApuWrite(uint32 adr, uint8 val) {
  gzprintf(effects, "%d apu %06x %02x\n", snes_frame_counter, adr, val);
  __real_RtlApuWrite(adr, val);
}
void __wrap_DecompressToMem(uint32 src, uint8 *dst) {
  uintptr_t address = (uintptr_t)dst, base = (uintptr_t)g_ram;
  if (address < base || address - base >= sizeof(g_ram) || g_snes->cart->romSize != 0x400000) exit(4);
  decompression_calls++;
  SaveRoutineState("before");
  if (mixed) {
    RustReport report = {0};
    int status = sm_decompress_v1(g_rom, g_snes->cart->romSize, g_ram, sizeof(g_ram),
        force_error ? 0 : src, address - base, &report);
    if (status) {
      fprintf(stderr, "ABI error status=%d rust_hits=%lu c_hits=%lu calls=%lu\n", status, rust_hits, c_hits, decompression_calls);
      exit(4); // No fallback or successful final state after a Rust error.
    }
    rust_hits++;
    rust_modes |= report.modes;
  } else {
    __real_DecompressToMem(src, dst);
    c_hits++;
  }
  SaveRoutineState("after");
  gzprintf(effects, "%d decompress %06x %lu\n", snes_frame_counter, src, (unsigned long)(address - base));
}
static void SaveBytes(void *ctx, void *data, size_t size) {
  if (fwrite(data, 1, size, ctx) != size) exit(2);
}
static void Snapshot(long frame) {
  char name[80];
  snprintf(name, sizeof(name), "frame-%06ld.bin", frame);
  FILE *file = fopen(name, "wb");
  if (!file) exit(2);
  uint8 version = g_snes->runningWhichVersion;
  snes_saveload(g_snes, SaveBytes, file);
  g_snes->runningWhichVersion = version;
  // Save both PPU implementations, including VRAM/OAM/CGRAM and registers.
  ppu_saveload(g_snes->my_ppu, SaveBytes, file);
  ppu_saveload(g_snes->snes_ppu, SaveBytes, file);
  SaveBytes(file, g_sram, 0x2000);
  SaveBytes(file, pixels, sizeof(pixels));
  if (fclose(file)) exit(2);
}
static int AbiControl(void) {
  // Exercise the real ABI on initialized allocations, including same-process reuse.
  uint8 *rom = calloc(1, 0x400000), *ram = calloc(1, sizeof(g_ram));
  if (!rom || !ram) return 2;
  const uint8 literal[] = {2, 0x12, 0x34, 0x56, 255};
  memcpy(rom, literal, sizeof(literal));
  RustReport report = {0};
  for (int repeat = 0; repeat < 2; repeat++) {
    memset(ram, 0, sizeof(g_ram));
    if (sm_decompress_v1(rom, 0x400000, ram, sizeof(g_ram), 0x808000, 0x1000, &report) ||
        memcmp(ram + 0x1000, literal + 1, 3) || report.written != 3 || report.modes != 1) return 3;
  }
  if (sm_decompress_v1(NULL, 0x400000, ram, sizeof(g_ram), 0x808000, 0x1000, &report) != 1 ||
      sm_decompress_v1(rom, 0x400000, ram, 1, 0x808000, 0x1000, &report) != 1 ||
      sm_decompress_v1(rom, 0x400000, rom, sizeof(g_ram), 0x808000, 0x1000, &report) != 1 ||
      sm_decompress_v1(rom, 0x400000, ram, sizeof(g_ram), 0, 0x1000, &report) != 2 ||
      sm_decompress_v1(rom, 0x400000, ram, sizeof(g_ram), 0x808000, sizeof(g_ram), &report) != 2) return 3;
  free(rom); free(ram);
  puts("abi-controls 7 2");
  return 0;
}
int main(int argc, char **argv) {
  if (argc == 2 && !strcmp(argv[1], "--abi-control")) return AbiControl();
  if (argc != 6) return 2;
  mixed = !strcmp(argv[1], "mixed") || !strcmp(argv[1], "error");
  force_error = !strcmp(argv[1], "error");
  long frames = strtol(argv[4], NULL, 10);
  if (frames < 1 || frames > 1000000 || !SnesInit(argv[2])) return 2;
  g_spc_player = SpcPlayer_Create();
  SpcPlayer_Initialize(g_spc_player);
  g_runmode = 1;
  for (epoch = 0; epoch < 2; epoch++) {
    char directory[16]; snprintf(directory, sizeof(directory), "epoch-%d", epoch);
    if (mkdir(directory, 0700) || chdir(directory) || mkdir("fixtures", 0700) || mkdir("saves", 0700)) return 2;
    effects = gzopen("hardware.log.gz", "wb1");
    FILE *inputs = fopen(argv[3], "r");
    if (!effects || !inputs) return 2;
    fallback_calls = fallback_instructions = decompression_calls = rust_hits = c_hits = rust_modes = 0;
    #ifdef NEXT_RANDOM_JOB
    rng_c_hits = rng_rust_hits = 0;
#endif
    memset(pixels, 0, sizeof(pixels));
    RtlReset(0);
    PpuBeginDrawing(g_snes->my_ppu, pixels, 512 * 4, 0);
    PpuBeginDrawing(g_snes->snes_ppu, pixels, 512 * 4, 0);
    for (long frame = 0; frame < frames; frame++) {
      unsigned pad, reversed = 0;
      if (fscanf(inputs, "%x", &pad) != 1 || pad > 65535) return 2;
      for (int bit = 0; bit < 16; bit++) reversed |= ((pad >> bit) & 1) << (15 - bit);
      RtlRunFrame(reversed);
      if (g_fail) return 3;
      if (frame == 0 || (frame + 1) % 1000 == 0 || frame + 1 == frames) Snapshot(frame + 1);
    }
    printf("epoch %d %ld %lu %lu %u %lu %lu\n",
        epoch, frames, rust_hits, c_hits, rust_modes, fallback_calls, fallback_instructions);
    #ifdef NEXT_RANDOM_JOB
    printf("rng-epoch %d %lu %lu\n", epoch, rng_c_hits, rng_rust_hits);
#endif
    if (gzclose(effects) || fclose(inputs) || chdir("..")) return 2;
  }
  return 0;
}
