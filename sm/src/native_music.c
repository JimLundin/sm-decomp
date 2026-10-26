// Music routines selected from the pinned Zelda dependency by install_native_music.py.
#include "native_music.h"
#include "src/types.h"
#include "snes/dsp_regs.h"

static void CalcTremolo(SpcPlayer *p, Channel *c, uint8 phase) {
  uint8 wave = (phase * 2) ^ ((phase & 0x80) ? 255 : 0);
  wave = wave * c->tremolo_depth >> 8;
  CalcFinalVolume(p, c, wave ^ 255);
}

static void HandleTremolo(SpcPlayer *p, Channel *c) {
  p->did_affect_volumepitch_flag |= 0x80;
  CalcTremolo(p, c, c->tremolo_rate * p->main_tempo_accum >> 8);
}

#include "native_music_selected.h"

void sm_music_SetupEchoParameter_EDL(SpcPlayer *p, uint8 value) {
  SetupEchoParameter_EDL(p, value);
}

void sm_music_WriteVolumeToDsp(SpcPlayer *p, Channel *c, uint16 volume) {
  WriteVolumeToDsp(p, c, volume);
}

void sm_music_HandlePanAndSweep(SpcPlayer *p, Channel *c) {
  HandlePanAndSweep(p, c);
}

void sm_music_Music_HandleCmdFromSnes(SpcPlayer *p) {
  Port0_HandleMusic(p);
}

const char *sm_music_source_hash(void) {
  return SM_MUSIC_SOURCE_HASH;
}
