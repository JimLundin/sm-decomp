/* Exact definition-only bank rename redirects every C NextRandom call here. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern uint16_t C_NextRandom(void);
extern int32_t sm_next_random_v1(uint8_t *, size_t);
static unsigned long rng_c_hits, rng_rust_hits;
uint16_t NextRandom(void) {
  unsigned long sequence = rng_c_hits + rng_rust_hits + 1;
  uint16_t before = g_ram[0x5e5] | (g_ram[0x5e6] << 8);
  char path[96];
  if (sequence <= 16) {
    snprintf(path, sizeof(path), "fixtures/rng-%04lu-before.wram", sequence);
    FILE *f = fopen(path, "wb");
    if (!f || fwrite(g_ram, 1, sizeof(g_ram), f) != sizeof(g_ram) || fclose(f)) exit(2);
  }
  int32_t result;
  if (mixed) {
    result = sm_next_random_v1(g_ram, sizeof(g_ram));
    if (result < 0 || result > 65535) exit(4);
    rng_rust_hits++;
  } else { result = C_NextRandom(); rng_c_hits++; }
  if (sequence <= 16) {
    snprintf(path, sizeof(path), "fixtures/rng-%04lu-after.wram", sequence);
    FILE *f = fopen(path, "wb");
    if (!f || fwrite(g_ram, 1, sizeof(g_ram), f) != sizeof(g_ram) || fclose(f)) exit(2);
  }
  // The pure contract has one LE word store, logged as two ordered semantic byte effects.
  gzprintf(effects, "%d rng %lu %u %d writes 1509:%u 1510:%u\n", snes_frame_counter,
    sequence, before, result, g_ram[0x5e5], g_ram[0x5e6]);
  return result;
}
