// Build against unmodified LakeSnes 28b8fbc15ad15afff003165ff4f2a8adb1c93d18.
#include <stdint.h>
#include <stddef.h>
#define SNES_H
typedef struct Snes {
    uint16_t hPos, vPos;
    uint8_t openBus;
} Snes;
#include "ppu.c"

const char* sm_backend_source_hash(void) {
    return SM_BACKEND_SOURCE_HASH;
}

uint8_t sm_backend_ppu_read(Ppu* ppu, uint8_t adr, uint16_t h, uint16_t v, uint8_t bus) {
    Snes view = {h, v, bus};
    Snes* saved = ppu->snes;
    ppu->snes = &view;
    uint8_t result = ppu_read(ppu, adr);
    ppu->snes = saved;
    return result;
}

void sm_backend_ppu_write(Ppu* ppu, uint8_t adr, uint8_t value) {
    // Current Rust restarts mosaic at line zero, independently of beam position.
    Snes view = {0};
    Snes* saved = ppu->snes;
    ppu->snes = &view;
    ppu_write(ppu, adr, value);
    ppu->snes = saved;
}

// Layout anchors for the state shared by register and legacy pixel operations.
_Static_assert(offsetof(Ppu, vram) == 8, "Ppu.vram layout");
_Static_assert(offsetof(Ppu, bgLayer) == 67142, "Ppu.bgLayer layout");
_Static_assert(offsetof(Ppu, windowLayer) == 67244, "Ppu.windowLayer layout");
_Static_assert(offsetof(Ppu, pixelBuffer) == 67313, "Ppu.pixelbuffer_placeholder layout");

int sm_backend_ppu_pixel(Ppu* ppu, int x, int y, bool sub, uint16_t sprite, int* color) {
    // Adapt one packed sprite without changing the obsolete snapshot buffers.
    uint8_t pixel = ppu->objPixelBuffer[x];
    uint8_t priority = ppu->objPriorityBuffer[x];
    ppu->objPixelBuffer[x] = sprite & 0xff;
    int tag = sprite >> 12;
    ppu->objPriorityBuffer[x] = (tag & 3) == 2 ? tag >> 2 : 0xff;
    int layer = ppu_getPixel(ppu, x, y, sub, &color[0], &color[1], &color[2]);
    ppu->objPixelBuffer[x] = pixel;
    ppu->objPriorityBuffer[x] = priority;
    return layer;
}
