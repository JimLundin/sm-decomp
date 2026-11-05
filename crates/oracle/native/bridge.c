// The oracle crate's bridge to upstream LakeSnes. Upstream sources are compiled
// unmodified; this translation unit includes snes.c so that it can observe
// the machine through renamed calls (ppu_write, ppu_runLine, ppu_handleVblank),
// the CPU's read/write callbacks and dma.c's B-bus calls (renamed by build.rs).
// It also owns the step loop, so Rust makes one call per video frame.
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#define ppu_write oh_hook_ppu_write
#define ppu_runLine oh_hook_ppu_runLine
#define ppu_handleVblank oh_hook_ppu_handleVblank
#include "snes.c"
#undef ppu_write
#undef ppu_runLine
#undef ppu_handleVblank

void ppu_write(Ppu* ppu, uint8_t adr, uint8_t val);
void ppu_runLine(Ppu* ppu, int line);
void ppu_handleVblank(Ppu* ppu);

#include "spc.h"
#include "dsp.h"

enum { OH_LINES = 239, OH_WIDTH = 256, OH_MAX_NMIS = 8 };

// The kinds of APU port access, as the Rust side names them.
enum { OH_APU_CPU_WRITE = 0, OH_APU_CPU_READ = 1, OH_APU_DMA_WRITE = 2, OH_APU_DMA_READ = 3 };

typedef struct OhApuAccess {
  uint8_t kind;
  uint8_t port;
  uint8_t value;
  uint8_t reserved;
} OhApuAccess;

typedef struct OhNmi {
  uint32_t interrupted_pc;
  uint8_t polled;
  uint8_t returned;
  uint16_t stack;
} OhNmi;

// Facts of one step, read by Rust after oh_step returns.
typedef struct OhFacts {
  uint32_t video_frame;
  uint32_t nmi_count;
  OhNmi nmis[OH_MAX_NMIS];
  uint32_t polls;
  uint16_t pad_latched;
  uint8_t ended_at_rti;
  uint8_t overscan;
  uint32_t lines_rendered;
  // The beam position when the CPU first reached a wait site in the interval
  // (0xffff when it did not).
  uint16_t wait_v;
  uint16_t wait_h;
} OhFacts;

typedef struct OhMachine {
  Snes* snes;
  // APU port log since the last step began.
  OhApuAccess* apu_log;
  uint32_t apu_len;
  uint32_t apu_cap;
  bool record_apu;
  // NMI tracking.
  bool in_nmi;
  uint16_t nmi_stack;
  uint32_t nmi_interrupted;
  bool nmi_polled;
  // The last value written to each B-bus PPU register $2100-$213F and each
  // CPU I/O register $4200-$421F.
  uint8_t ppu_written[0x40];
  uint8_t io_written[0x20];
  // The picture scanned out during this step, in PPU-domain BGR555, and
  // each line's brightness and forced blank.
  bool record_picture;
  uint16_t picture[OH_LINES * OH_WIDTH];
  uint8_t brightness[OH_LINES];
  uint8_t forced_blank[OH_LINES];
  OhFacts facts;
  // The game's wait-for-NMI loop addresses (oh_set_wait_sites).
  uint32_t wait_sites[8];
  uint32_t wait_site_count;
} OhMachine;

static _Thread_local OhMachine* oh_current;

static void oh_log_apu(OhMachine* m, uint8_t kind, uint8_t port, uint8_t value) {
  if(!m->record_apu) return;
  if(m->apu_len == m->apu_cap) {
    uint32_t cap = m->apu_cap ? m->apu_cap * 2 : 256;
    OhApuAccess* grown = realloc(m->apu_log, cap * sizeof(OhApuAccess));
    if(grown == NULL) return;
    m->apu_log = grown;
    m->apu_cap = cap;
  }
  m->apu_log[m->apu_len++] = (OhApuAccess){kind, port, value, 0};
}

// Whether a CPU address reaches the system area ($00-$3F and $80-$BF banks, $0000-$7FFF).
static bool oh_system(uint32_t adr) {
  uint8_t bank = adr >> 16;
  return (adr & 0xffff) < 0x8000 && (bank < 0x40 || (bank >= 0x80 && bank < 0xc0));
}

static uint8_t oh_cpu_read(void* mem, uint32_t adr) {
  OhMachine* m = mem;
  uint8_t value = snes_cpuRead(m->snes, adr);
  if(adr == 0xffea && !m->in_nmi) {
    // The native-mode NMI vector: the CPU has pushed K, PC and P.
    Cpu* cpu = m->snes->cpu;
    uint8_t* ram = m->snes->ram;
    uint16_t sp = cpu->sp;
    m->in_nmi = true;
    m->nmi_stack = sp;
    m->nmi_polled = false;
    m->nmi_interrupted = ((uint32_t) ram[(uint16_t) (sp + 4)] << 16)
      | ((uint32_t) ram[(uint16_t) (sp + 3)] << 8) | ram[(uint16_t) (sp + 2)];
  } else if(oh_system(adr)) {
    uint16_t low = adr & 0xffff;
    if(low >= 0x2140 && low < 0x2180) {
      oh_log_apu(m, OH_APU_CPU_READ, low & 3, value);
    } else if(low >= 0x4218 && low < 0x4220) {
      m->facts.polls++;
      if(m->in_nmi) m->nmi_polled = true;
    }
  }
  return value;
}

static void oh_cpu_write(void* mem, uint32_t adr, uint8_t value) {
  OhMachine* m = mem;
  if(oh_system(adr)) {
    uint16_t low = adr & 0xffff;
    if(low >= 0x2140 && low < 0x2180) {
      oh_log_apu(m, OH_APU_CPU_WRITE, low & 3, value);
    } else if(low >= 0x4200 && low < 0x4220) {
      m->io_written[low - 0x4200] = value;
    }
  }
  snes_cpuWrite(m->snes, adr, value);
}

static void oh_cpu_idle(void* mem, bool waiting) {
  OhMachine* m = mem;
  snes_cpuIdle(m->snes, waiting);
}

void oh_hook_ppu_write(Ppu* ppu, uint8_t adr, uint8_t val) {
  OhMachine* m = oh_current;
  if(m != NULL && adr < 0x40) m->ppu_written[adr] = val;
  ppu_write(ppu, adr, val);
}

void oh_hook_ppu_runLine(Ppu* ppu, int line) {
  OhMachine* m = oh_current;
  if(m == NULL || !m->record_picture) {
    ppu_runLine(ppu, line);
    return;
  }
  int index = line - 1;
  // Render at full brightness so LakeSnes's 5-to-8-bit expansion is exact,
  // then keep the real brightness beside the line.
  uint8_t brightness = ppu->brightness;
  m->brightness[index] = brightness;
  m->forced_blank[index] = ppu->forcedBlank;
  ppu->brightness = 15;
  ppu_runLine(ppu, line);
  ppu->brightness = brightness;
  int row = index + (ppu->evenFrame ? 0 : 239);
  const uint8_t* source = &ppu->pixelBuffer[row * 2048 + 4 + ppu->pixelOutputFormat];
  uint16_t* target = &m->picture[index * OH_WIDTH];
  for(int x = 0; x < OH_WIDTH; x++) {
    const uint8_t* pixel = source + x * 8;
    target[x] = (uint16_t) ((pixel[2] >> 3) | ((pixel[1] >> 3) << 5) | ((pixel[0] >> 3) << 10));
  }
  m->facts.lines_rendered++;
}

void oh_hook_ppu_handleVblank(Ppu* ppu) {
  ppu_handleVblank(ppu);
}

// dma.c's B-bus calls, renamed by build.rs, so DMA's APU port accesses are logged.
uint8_t oh_dma_readBBus(Snes* snes, uint8_t adr) {
  uint8_t value = snes_readBBus(snes, adr);
  OhMachine* m = oh_current;
  if(m != NULL && adr >= 0x40 && adr < 0x80) oh_log_apu(m, OH_APU_DMA_READ, adr & 3, value);
  return value;
}

void oh_dma_writeBBus(Snes* snes, uint8_t adr, uint8_t val) {
  OhMachine* m = oh_current;
  if(m != NULL && adr >= 0x40 && adr < 0x80) oh_log_apu(m, OH_APU_DMA_WRITE, adr & 3, val);
  snes_writeBBus(snes, adr, val);
}

OhMachine* oh_open(const uint8_t* rom, int length) {
  OhMachine* m = calloc(1, sizeof(OhMachine));
  if(m == NULL) return NULL;
  m->snes = snes_init();
  m->snes->cpu->mem = m;
  m->snes->cpu->read = oh_cpu_read;
  m->snes->cpu->write = oh_cpu_write;
  m->snes->cpu->idle = oh_cpu_idle;
  m->record_apu = true;
  m->record_picture = true;
  oh_current = m;
  if(!snes_loadRom(m->snes, rom, length)) {
    oh_current = NULL;
    snes_free(m->snes);
    free(m);
    return NULL;
  }
  oh_current = NULL;
  return m;
}

void oh_close(OhMachine* m) {
  if(m == NULL) return;
  snes_free(m->snes);
  free(m->apu_log);
  free(m);
}

void oh_set_recording(OhMachine* m, bool picture, bool apu) {
  m->record_picture = picture;
  m->record_apu = apu;
}

// LakeSnes's controller shifts out bit 0 first; the pad word is in JOY1 order (B = bit 15).
static uint16_t oh_serial(uint16_t pad) {
  uint16_t serial = 0;
  for(int bit = 0; bit < 16; bit++) {
    if(pad & (0x8000 >> bit)) serial |= 1 << bit;
  }
  return serial;
}

// Run one instruction (or interrupt entry); note an NMI handler's return.
static void oh_run_opcode(OhMachine* m) {
  Cpu* cpu = m->snes->cpu;
  cpu_runOpcode(cpu);
  if(m->facts.wait_v == 0xffff) {
    uint32_t pc = ((uint32_t) cpu->k << 16) | cpu->pc;
    for(uint32_t i = 0; i < m->wait_site_count; i++) {
      if(pc == m->wait_sites[i]) {
        m->facts.wait_v = m->snes->vPos;
        m->facts.wait_h = m->snes->hPos;
        break;
      }
    }
  }
  if(m->in_nmi && cpu->sp == (uint16_t) (m->nmi_stack + 4)
    && (((uint32_t) cpu->k << 16) | cpu->pc) == m->nmi_interrupted) {
    m->in_nmi = false;
    if(m->facts.nmi_count < OH_MAX_NMIS) {
      m->facts.nmis[m->facts.nmi_count] = (OhNmi){m->nmi_interrupted, m->nmi_polled, 1, m->nmi_stack};
    }
    m->facts.nmi_count++;
  }
}

// Run one video frame with `pad` on controller 1: to the next vblank, where
// auto-joypad latches the pad, then to the RTI that ends the NMI serviced in
// that vblank. Without an NMI the frame ends at vblank start; an NMI handler
// still running when vblank ends returns 1.
int oh_step(OhMachine* m, uint16_t pad) {
  Snes* snes = m->snes;
  Cpu* cpu = snes->cpu;
  oh_current = m;
  memset(&m->facts, 0, sizeof(m->facts));
  m->apu_len = 0;
  m->facts.wait_v = 0xffff;
  snes->input1->currentState = oh_serial(pad);
  int status = 0;
  while(snes->inVblank) oh_run_opcode(m);
  uint32_t frame = snes->frames;
  while(!snes->inVblank && frame == snes->frames) oh_run_opcode(m);
  m->facts.video_frame = snes->frames;
  m->facts.overscan = snes->ppu->frameOverscan;
  m->facts.pad_latched = snes->portAutoRead[0];
  uint32_t returned = m->facts.nmi_count;
  if(cpu->nmiWanted || m->in_nmi) {
    while(m->facts.nmi_count == returned) {
      if(!snes->inVblank) {
        status = 1;
        break;
      }
      oh_run_opcode(m);
    }
    m->facts.ended_at_rti = m->facts.nmi_count != returned;
  }
  snes_catchupApu(snes);
  oh_current = NULL;
  return status;
}

// Raw memory and state, valid until the next step.
uint8_t* oh_wram(OhMachine* m) { return m->snes->ram; }
uint16_t* oh_vram(OhMachine* m) { return m->snes->ppu->vram; }
uint16_t* oh_cgram(OhMachine* m) { return m->snes->ppu->cgram; }
uint16_t* oh_oam(OhMachine* m) { return m->snes->ppu->oam; }
uint8_t* oh_high_oam(OhMachine* m) { return m->snes->ppu->highOam; }
uint8_t* oh_sram(OhMachine* m, int* size) { *size = m->snes->cart->ramSize; return m->snes->cart->ram; }
uint8_t* oh_apu_ram(OhMachine* m) { return m->snes->apu->ram; }
uint8_t* oh_dsp_registers(OhMachine* m) { return m->snes->apu->dsp->ram; }
const OhFacts* oh_facts(OhMachine* m) { return &m->facts; }
const OhApuAccess* oh_apu_log(OhMachine* m, uint32_t* length) { *length = m->apu_len; return m->apu_log; }
const uint16_t* oh_picture(OhMachine* m) { return m->picture; }
const uint8_t* oh_line_brightness(OhMachine* m) { return m->brightness; }
const uint8_t* oh_line_forced_blank(OhMachine* m) { return m->forced_blank; }
const uint8_t* oh_ppu_written(OhMachine* m) { return m->ppu_written; }
const uint8_t* oh_io_written(OhMachine* m) { return m->io_written; }

// The CPU registers, in a fixed layout.
typedef struct OhCpu {
  uint16_t a, x, y, sp, pc, dp;
  uint8_t k, db, p, e;
} OhCpu;

static uint8_t oh_cpu_flags(const Cpu* cpu) {
  return (cpu->n << 7) | (cpu->v << 6) | (cpu->mf << 5) | (cpu->xf << 4) | (cpu->d << 3)
    | (cpu->i << 2) | (cpu->z << 1) | cpu->c;
}

void oh_cpu(OhMachine* m, OhCpu* out) {
  Cpu* cpu = m->snes->cpu;
  *out = (OhCpu){cpu->a, cpu->x, cpu->y, cpu->sp, cpu->pc, cpu->dp, cpu->k, cpu->db,
    oh_cpu_flags(cpu), cpu->e};
}

// The SPC700 registers, in a fixed layout.
typedef struct OhSpc {
  uint8_t a, x, y, sp;
  uint16_t pc;
  uint8_t psw, stopped;
  uint8_t in_ports[4];
  uint8_t out_ports[4];
} OhSpc;

void oh_spc(OhMachine* m, OhSpc* out) {
  Spc* spc = m->snes->apu->spc;
  uint8_t psw = (spc->n << 7) | (spc->v << 6) | (spc->p << 5) | (spc->b << 4)
    | (spc->h << 3) | (spc->i << 2) | (spc->z << 1) | spc->c;
  *out = (OhSpc){spc->a, spc->x, spc->y, spc->sp, spc->pc, psw, spc->stopped, {0}, {0}};
  memcpy(out->in_ports, m->snes->apu->inPorts, 4);
  memcpy(out->out_ports, m->snes->apu->outPorts, 4);
}

// The PPU's decoded state, flattened in a fixed layout of plain integers.
typedef struct OhBg {
  uint16_t h_scroll, v_scroll, tilemap_adr, tile_adr;
  uint8_t tilemap_wider, tilemap_higher, big_tiles, mosaic_enabled;
} OhBg;

typedef struct OhPpu {
  OhBg bg[4];
  int16_t m7matrix[8];
  uint16_t vram_pointer, vram_increment, vram_read_buffer;
  uint16_t obj_tile_adr1, obj_tile_adr2;
  uint16_t h_count, v_count;
  uint8_t layer_main[5], layer_sub[5], layer_main_windowed[5], layer_sub_windowed[5];
  uint8_t window_enabled1[6], window_enabled2[6], window_inversed1[6], window_inversed2[6], mask_logic[6];
  uint8_t math_enabled[6];
  uint8_t vram_increment_on_high, vram_remap_mode;
  uint8_t cgram_pointer, cgram_second_write, cgram_buffer;
  uint8_t oam_adr, oam_adr_written, oam_in_high, oam_in_high_written, oam_second_write, oam_buffer;
  uint8_t obj_priority, obj_size, time_over, range_over, obj_interlace;
  uint8_t scroll_prev, scroll_prev2, mosaic_size, mosaic_start_line, m7prev;
  uint8_t m7large_field, m7char_fill, m7x_flip, m7y_flip, m7ext_bg;
  uint8_t window1left, window1right, window2left, window2right;
  uint8_t clip_mode, prevent_math_mode, add_subscreen, subtract_color, half_color;
  uint8_t fixed_r, fixed_g, fixed_b;
  uint8_t forced_blank, brightness, mode, bg3priority, even_frame, pseudo_hires;
  uint8_t overscan, frame_overscan, interlace, frame_interlace, direct_color;
  uint8_t h_count_second, v_count_second, counters_latched, ppu1open_bus, ppu2open_bus;
} OhPpu;

void oh_ppu(OhMachine* m, OhPpu* out) {
  Ppu* p = m->snes->ppu;
  memset(out, 0, sizeof(*out));
  for(int i = 0; i < 4; i++) {
    BgLayer* bg = &p->bgLayer[i];
    out->bg[i] = (OhBg){bg->hScroll, bg->vScroll, bg->tilemapAdr, bg->tileAdr,
      bg->tilemapWider, bg->tilemapHigher, bg->bigTiles, bg->mosaicEnabled};
  }
  memcpy(out->m7matrix, p->m7matrix, sizeof(out->m7matrix));
  out->vram_pointer = p->vramPointer;
  out->vram_increment = p->vramIncrement;
  out->vram_read_buffer = p->vramReadBuffer;
  out->obj_tile_adr1 = p->objTileAdr1;
  out->obj_tile_adr2 = p->objTileAdr2;
  out->h_count = p->hCount;
  out->v_count = p->vCount;
  for(int i = 0; i < 5; i++) {
    out->layer_main[i] = p->layer[i].mainScreenEnabled;
    out->layer_sub[i] = p->layer[i].subScreenEnabled;
    out->layer_main_windowed[i] = p->layer[i].mainScreenWindowed;
    out->layer_sub_windowed[i] = p->layer[i].subScreenWindowed;
  }
  for(int i = 0; i < 6; i++) {
    out->window_enabled1[i] = p->windowLayer[i].window1enabled;
    out->window_enabled2[i] = p->windowLayer[i].window2enabled;
    out->window_inversed1[i] = p->windowLayer[i].window1inversed;
    out->window_inversed2[i] = p->windowLayer[i].window2inversed;
    out->mask_logic[i] = p->windowLayer[i].maskLogic;
    out->math_enabled[i] = p->mathEnabled[i];
  }
  out->vram_increment_on_high = p->vramIncrementOnHigh;
  out->vram_remap_mode = p->vramRemapMode;
  out->cgram_pointer = p->cgramPointer;
  out->cgram_second_write = p->cgramSecondWrite;
  out->cgram_buffer = p->cgramBuffer;
  out->oam_adr = p->oamAdr;
  out->oam_adr_written = p->oamAdrWritten;
  out->oam_in_high = p->oamInHigh;
  out->oam_in_high_written = p->oamInHighWritten;
  out->oam_second_write = p->oamSecondWrite;
  out->oam_buffer = p->oamBuffer;
  out->obj_priority = p->objPriority;
  out->obj_size = p->objSize;
  out->time_over = p->timeOver;
  out->range_over = p->rangeOver;
  out->obj_interlace = p->objInterlace;
  out->scroll_prev = p->scrollPrev;
  out->scroll_prev2 = p->scrollPrev2;
  out->mosaic_size = p->mosaicSize;
  out->mosaic_start_line = p->mosaicStartLine;
  out->m7prev = p->m7prev;
  out->m7large_field = p->m7largeField;
  out->m7char_fill = p->m7charFill;
  out->m7x_flip = p->m7xFlip;
  out->m7y_flip = p->m7yFlip;
  out->m7ext_bg = p->m7extBg;
  out->window1left = p->window1left;
  out->window1right = p->window1right;
  out->window2left = p->window2left;
  out->window2right = p->window2right;
  out->clip_mode = p->clipMode;
  out->prevent_math_mode = p->preventMathMode;
  out->add_subscreen = p->addSubscreen;
  out->subtract_color = p->subtractColor;
  out->half_color = p->halfColor;
  out->fixed_r = p->fixedColorR;
  out->fixed_g = p->fixedColorG;
  out->fixed_b = p->fixedColorB;
  out->forced_blank = p->forcedBlank;
  out->brightness = p->brightness;
  out->mode = p->mode;
  out->bg3priority = p->bg3priority;
  out->even_frame = p->evenFrame;
  out->pseudo_hires = p->pseudoHires;
  out->overscan = p->overscan;
  out->frame_overscan = p->frameOverscan;
  out->interlace = p->interlace;
  out->frame_interlace = p->frameInterlace;
  out->direct_color = p->directColor;
  out->h_count_second = p->hCountSecond;
  out->v_count_second = p->vCountSecond;
  out->counters_latched = p->countersLatched;
  out->ppu1open_bus = p->ppu1openBus;
  out->ppu2open_bus = p->ppu2openBus;
}

// The DMA channels' registers $43x0-$43xB, as the CPU would read them back.
void oh_dma_registers(OhMachine* m, uint8_t out[8][12]) {
  for(int i = 0; i < 8; i++) {
    DmaChannel* c = &m->snes->dma->channel[i];
    out[i][0] = (c->fromB << 7) | (c->indirect << 6) | (c->unusedBit << 5) | (c->decrement << 4)
      | (c->fixed << 3) | c->mode;
    out[i][1] = c->bAdr;
    out[i][2] = c->aAdr & 0xff;
    out[i][3] = c->aAdr >> 8;
    out[i][4] = c->aBank;
    out[i][5] = c->size & 0xff;
    out[i][6] = c->size >> 8;
    out[i][7] = c->indBank;
    out[i][8] = c->tableAdr & 0xff;
    out[i][9] = c->tableAdr >> 8;
    out[i][10] = c->repCount;
    out[i][11] = c->unusedByte;
  }
}

// The CPU I/O state the PPU contract names.
typedef struct OhIo {
  uint16_t h_timer, v_timer;
  uint8_t nmi_enabled, h_irq_enabled, v_irq_enabled, auto_joy_read, fast_mem, hdma_enabled;
  uint16_t h_pos, v_pos;
} OhIo;

void oh_io(OhMachine* m, OhIo* out) {
  Snes* s = m->snes;
  uint8_t hdma = 0;
  for(int i = 0; i < 8; i++) hdma |= s->dma->channel[i].hdmaActive << i;
  *out = (OhIo){s->hTimer, s->vTimer, s->nmiEnabled, s->hIrqEnabled, s->vIrqEnabled,
    s->autoJoyRead, s->fastMem, hdma, s->hPos, s->vPos};
}

// Save and load the emulator's own state blob.
int oh_save_state(OhMachine* m, uint8_t* data) { return snes_saveState(m->snes, data); }
bool oh_load_state(OhMachine* m, uint8_t* data, int size) { return snes_loadState(m->snes, data, size); }

// Power on (hard reset) or press reset.
void oh_reset(OhMachine* m, bool hard) {
  oh_current = m;
  snes_reset(m->snes, hard);
  m->in_nmi = false;
  oh_current = NULL;
}

// The interrupted address of an NMI handler still running, or UINT32_MAX.
uint32_t oh_running_nmi(OhMachine* m) {
  return m->in_nmi ? m->nmi_interrupted : UINT32_MAX;
}

// Set the CPU registers, for loading a migration seed.
void oh_set_cpu(OhMachine* m, const OhCpu* in) {
  Cpu* cpu = m->snes->cpu;
  cpu->a = in->a;
  cpu->x = in->x;
  cpu->y = in->y;
  cpu->sp = in->sp;
  cpu->pc = in->pc;
  cpu->dp = in->dp;
  cpu->k = in->k;
  cpu->db = in->db;
  cpu->e = in->e;
  uint8_t p = in->p;
  cpu->n = p & 0x80;
  cpu->v = p & 0x40;
  cpu->mf = p & 0x20;
  cpu->xf = p & 0x10;
  cpu->d = p & 0x08;
  cpu->i = p & 0x04;
  cpu->z = p & 0x02;
  cpu->c = p & 0x01;
  cpu->waiting = false;
  cpu->stopped = false;
  cpu->intWanted = false;
  cpu->resetWanted = false;
  m->in_nmi = false;
}

// Write one B-bus or CPU register as the CPU would, for loading a migration seed.
void oh_poke(OhMachine* m, uint32_t adr, uint8_t value) {
  oh_current = m;
  snes_write(m->snes, adr, value);
  oh_current = NULL;
}

// The addresses at which the main thread waits for NMI (at most 8).
void oh_set_wait_sites(OhMachine* m, const uint32_t* pcs, uint32_t count) {
  m->wait_site_count = count < 8 ? count : 8;
  memcpy(m->wait_sites, pcs, m->wait_site_count * sizeof(uint32_t));
}
