/* Actual NextRandom is retained from the pinned whole bank, never transcribed. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
uint8_t g_ram[0x20000];
extern uint16_t NextRandom(void);
int main(int argc, char **argv) {
  if (argc == 2 && !strcmp(argv[1], "exhaustive")) {
    for (size_t i = 0; i < sizeof(g_ram); i++) g_ram[i] = i * 37;
    for (unsigned seed = 0; seed < 65536; seed++) {
      g_ram[0x5e5] = seed; g_ram[0x5e6] = seed >> 8;
      uint16_t result = NextRandom();
      for (size_t i = 0; i < sizeof(g_ram); i++)
        if (i != 0x5e5 && i != 0x5e6 && g_ram[i] != (uint8_t)(i * 37)) return 3;
      if (g_ram[0x5e5] != (uint8_t)result || g_ram[0x5e6] != result >> 8) return 3;
      if (fwrite(g_ram + 0x5e5, 1, 2, stdout) != 2) return 2;
    }
    return 0;
  }
  if (argc != 3) return 2;
  FILE *f = fopen(argv[1], "rb");
  if (!f || fread(g_ram, 1, sizeof(g_ram), f) != sizeof(g_ram) || fgetc(f) != EOF || fclose(f)) return 2;
  unsigned result = NextRandom();
  f = fopen(argv[2], "wb");
  if (!f || fwrite(g_ram, 1, sizeof(g_ram), f) != sizeof(g_ram) || fclose(f)) return 2;
  printf("%u\n", result);
  return 0;
}
