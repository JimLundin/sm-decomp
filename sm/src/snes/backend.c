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

// Layout anchors for the shared register prefix; no upstream renderer is called.
_Static_assert(offsetof(Ppu, vram) == 8, "Ppu.vram layout");
_Static_assert(offsetof(Ppu, bgLayer) == 67142, "Ppu.bgLayer layout");
_Static_assert(offsetof(Ppu, windowLayer) == 67244, "Ppu.windowLayer layout");
_Static_assert(offsetof(Ppu, pixelBuffer) == 67313, "Ppu.pixelbuffer_placeholder layout");

// DSP RAM may belong to either the native sequencer or the emulated APU.
#define APU_H
typedef struct Apu { uint8_t* ram; } Apu;
#include "dsp.c"

static int checked_dsp_cycle(Dsp* dsp) {
    bool unusual = dsp->firBufferIndex >= 8;
    for (int i = 0; i < 8; i++) unusual |= dsp->channel[i].adsrState > 4;
    if (unusual) {
        // Preserve Rust's partial state before an invalid envelope/FIR index fails.
        // Voice evaluation reads APU RAM but cannot write it; echo runs afterward.
        Dsp saved;
        memcpy(&saved, dsp, sizeof(saved));
        for (int i = 0; i < 8; i++) {
            DspChannel* ch = &dsp->channel[i];
            uint16_t pitch = ch->pitch;
            if (i > 0 && ch->pitchModulation) {
                pitch = (pitch * ((dsp->channel[i - 1].sampleOut >> 4) + 0x400)) >> 10;
                if (pitch > 0x3fff) pitch = 0x3fff;
            }
            int counter = ch->pitchCounter + pitch;
            bool releases = counter > 0xffff && ch->previousFlags == 1;
            if (ch->adsrState > 4 && !dsp->reset && !releases) {
                if (counter > 0xffff) dsp_decodeBrr(dsp, i);
                ch->pitchCounter = counter;
                return i + 1;
            }
            dsp_cycleChannel(dsp, i);
        }
        if (dsp->firBufferIndex >= 8) return 9;
        memcpy(dsp, &saved, sizeof(saved));
    }
    dsp_cycle(dsp);
    return 0;
}

int sm_backend_dsp_cycle(Dsp* dsp) {
    Apu view = {(uint8_t*)dsp->apu};
    Apu* saved = dsp->apu;
    dsp->apu = &view;
    int failure = checked_dsp_cycle(dsp);
    dsp->apu = saved;
    return failure;
}

_Static_assert(sizeof(DspChannel) == 86, "DspChannel layout");
_Static_assert(offsetof(Dsp, channel) == 136, "Dsp.channel layout");
_Static_assert(offsetof(Dsp, sampleBuffer) == 892, "Dsp.sampleBuffer layout");
_Static_assert(sizeof(Dsp) == 3032, "Dsp layout");
