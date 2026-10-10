/* Minimal I/O and RomPtr shim. DecompressToMem is linked from the unchanged upstream bank. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
static uint8_t *rom;
static size_t rom_size;
extern void DecompressToMem(uint32_t src, uint8_t *dst);
uint8_t *RomPtr(uint32_t address) {
  uint32_t offset = (((address >> 16) << 15) | (address & 0x7fff)) & 0x3fffff;
  if (address > 0xffffff || !(address & 0x8000) || offset >= rom_size) {
    puts("{\"status\":\"error\",\"error\":\"invalid ROM read\"}"); exit(1);
  }
  return rom + offset;
}
static uint8_t *load(const char *path, size_t *size) {
  FILE *f = fopen(path, "rb");
  if (!f || fseek(f, 0, SEEK_END) || (*size = ftell(f)) > 0x400000 || fseek(f, 0, SEEK_SET)) exit(2);
  uint8_t *data = malloc(*size);
  if (!data || fread(data, 1, *size, f) != *size) exit(2);
  fclose(f); return data;
}
int main(int argc, char **argv) {
  if (argc != 6) return 2;
  rom = load(argv[1], &rom_size);
  size_t size; uint8_t *wram = load(argv[2], &size);
  unsigned long src = strtoul(argv[3], NULL, 10), dst = strtoul(argv[4], NULL, 10);
  if (!rom_size || size != 0x20000 || src > 0xffffff || dst >= size) return 2;
  /* Only valid, bounded fixtures enter this unsafe upstream routine. */
  DecompressToMem((uint32_t)src, wram + dst);
  FILE *f = fopen(argv[5], "wb");
  if (!f || fwrite(wram, 1, size, f) != size || fclose(f)) return 2;
  puts("{\"status\":\"ok\"}");
  free(rom); free(wram); return 0;
}
