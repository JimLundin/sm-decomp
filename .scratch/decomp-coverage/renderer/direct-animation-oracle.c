// Exact pinned-C three-word animation interface used by a Rust verification task.
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

extern uint8_t g_ram[0x20000];
extern const uint8_t *g_rom;
bool g_fail;
extern void DrawPauseScreenSpriteAnim(uint16_t animation, uint16_t x, uint16_t y);

int main(int argc, char **argv) {
  if (argc != 7) return 2;
  FILE *file = fopen(argv[6], "rb");
  if (!file || fseek(file, 0, SEEK_END)) return 2;
  long length = ftell(file);
  if (length <= 0 || length > 0x400000 || fseek(file, 0, SEEK_SET)) return 2;
  uint8_t *rom = malloc((size_t)length);
  if (!rom || fread(rom, 1, (size_t)length, file) != (size_t)length || fclose(file)) return 2;
  g_rom = rom;
  file = fopen(argv[4], "rb");
  if (!file || fread(g_ram, 1, sizeof(g_ram), file) != sizeof(g_ram) || fgetc(file) != EOF || fclose(file)) return 2;
  DrawPauseScreenSpriteAnim((uint16_t)strtoul(argv[1], 0, 10), (uint16_t)strtoul(argv[2], 0, 10), (uint16_t)strtoul(argv[3], 0, 10));
  if (g_fail) return 3;
  file = fopen(argv[5], "wb");
  if (!file || fwrite(g_ram, 1, sizeof(g_ram), file) != sizeof(g_ram) || fclose(file)) return 2;
  puts("0");
  return 0;
}
