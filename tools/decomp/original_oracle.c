/* Isolated instruction runner: unchanged original ROM, actual upstream 65816 CPU.
 * No gameplay hooks, MMIO, ROM patches, interrupts or C decompressor calls. */
#include "snes/cpu.h"
#include "snes/snes.h"
static uint8_t *rom, ram[0x20000];
static size_t rom_size;
static unsigned stack_min = 0x200, stack_max = 0;
Snes *g_snes;
int CpuOpcodeHook(uint32_t address) { (void)address; exit(6); }
void Die(const char *message) { fputs(message, stderr); exit(6); }
bool HookedFunctionRts(int is_long) { (void)is_long; return false; }
static int ram_offset(uint32_t address) {
  unsigned bank = address >> 16, low = address & 0xffff;
  if (bank == 0x7e || bank == 0x7f) return address - 0x7e0000;
  if ((bank & 0x7f) < 0x40 && low < 0x2000) return low;
  return -1;
}
uint8_t snes_cpuRead(Snes *snes, uint32_t address) {
  (void)snes;
  int offset = ram_offset(address);
  if (offset >= 0) return ram[offset];
  unsigned physical = (((address >> 16) << 15) | (address & 0x7fff)) & 0x3fffff;
  if (!(address & 0x8000) || address > 0xffffff || physical >= rom_size) exit(3);
  return rom[physical];
}
void snes_cpuWrite(Snes *snes, uint32_t address, uint8_t value) {
  (void)snes;
  int offset = ram_offset(address);
  if (offset < 0) exit(3);
  ram[offset] = value;
  if (offset >= 0x100 && offset <= 0x1ff) {
    if ((unsigned)offset < stack_min) stack_min = offset;
    if ((unsigned)offset > stack_max) stack_max = offset;
  }
}
static void load(const char *path, uint8_t *data, size_t size) {
  FILE *f = fopen(path, "rb");
  if (!f || fread(data, 1, size, f) != size || fgetc(f) != EOF) exit(2);
  fclose(f);
}
int main(int argc, char **argv) {
  if (argc != 6) return 2;
  FILE *f = fopen(argv[1], "rb");
  if (!f || fseek(f, 0, SEEK_END)) return 2;
  rom_size = ftell(f); fclose(f);
  if (rom_size != 0x300000) return 2;
  rom = malloc(rom_size); if (!rom) return 2;
  load(argv[1], rom, rom_size); load(argv[2], ram, sizeof(ram));
  unsigned src = strtoul(argv[3], NULL, 10), dst = strtoul(argv[4], NULL, 10);
  if (src > 0xffffff || dst >= sizeof(ram)) return 2;
  for (unsigned i = 0; i < 3; i++) {
    ram[0x47 + i] = src >> (8 * i);
    ram[0x4c + i] = (0x7e0000 + dst) >> (8 * i);
  }
  /* Synthetic RTL frame only, sentinel $7E:2000, stack=$01FC. */
  ram[0x1fd] = 0xff; ram[0x1fe] = 0x1f; ram[0x1ff] = 0x7e;
  Cpu *cpu = cpu_init(NULL, 0);
  cpu->pc = 0xb119; cpu->k = 0x80; cpu->dp = 0; cpu->db = 0;
  cpu->sp = 0x1fc; cpu->e = false; cpu->mf = false; cpu->xf = false;
  cpu->i = true; cpu->d = false;
  unsigned instructions = 0;
  while (!(cpu->k == 0x7e && cpu->pc == 0x2000)) {
    if (++instructions > 2000000 || cpu->waiting || cpu->stopped) return 4;
    cpu_runOpcode(cpu);
  }
  if (cpu->sp != 0x1ff || cpu->dp || cpu->db || cpu->e || cpu->mf || cpu->xf) return 5;
  f = fopen(argv[5], "wb");
  if (!f || fwrite(ram, 1, sizeof(ram), f) != sizeof(ram) || fclose(f)) return 2;
  printf("status = \"ok\"\ninstructions = %u\nstack_written_min = %u\nstack_written_max = %u\nreturn_address = 8265728\n", instructions, stack_min, stack_max);
  cpu_free(cpu); free(rom); return 0;
}
