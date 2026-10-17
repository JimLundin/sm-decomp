// Narrow scheduled projectile boundary and passive observation of the actual
// recovered-C runtime. No candidate implementation is compiled into this file.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include "spc_player.h"
#include "snes/cart.h"
#include "snes/dma.h"
#include "snes/input.h"
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

extern SpcPlayer *g_spc_player;
extern int32 *cur_coll_amt32;
extern bool g_calling_asm_from_c;
extern uint8 g_runmode;
extern bool g_use_my_apu_code;
extern uint16 currently_installed_bug_fix_counter;

typedef int (*ProjectileHook)(void *, unsigned);
static ProjectileHook projectile_hook;
static void *projectile_context;
// Four arrays in order: C entries, Rust entries, completed, refused; then
// explicit Rust-to-C service requests. Ownership is separate from equivalence.
static uint64_t projectile_counts[37];
static unsigned active_entry;
static uint16_t entry_oam;
static bool entry_has_ordinary_shot;
static uint64_t rom_generation;
static uint8_t *effects;
static size_t effects_size;
static size_t effects_capacity;
static uint64_t effect_kind_counts[52];
enum { kEffectReportBytes = 4 * 1024 * 1024, kEffectLimitBytes = 64 * 1024 * 1024 };
static uint8_t observation[1024 * 1024];
static size_t observation_size;

void sm_native_set_projectile_hook(ProjectileHook hook, void *context) {
  if (active_entry || sm_native_scroll_active() || sm_native_ceres_door_active() || sm_native_room_transition_active() || sm_native_door_scrolling_active() || sm_native_camera_active() || sm_native_item_instruction_active())
    Die("changing projectile hook during an invocation");
  projectile_hook = hook;
  projectile_context = context;
}
unsigned sm_native_projectile_active(void) { return active_entry; }
void sm_native_projectile_counts(uint64_t *out) {
  memcpy(out, projectile_counts, sizeof(projectile_counts));
}
void sm_native_rom_changed(void) { rom_generation++; }
uint64_t sm_native_rom_generation(void) { return rom_generation; }
void sm_native_commit_wram(const uint8_t *ram) { memcpy(g_ram, ram, sizeof(g_ram)); }

static void Put32(uint8_t *p, uint32_t v) {
  for (unsigned i = 0; i < 4; i++) p[i] = (uint8_t)(v >> (i * 8));
}
void sm_native_effect(unsigned kind, unsigned a, unsigned b, unsigned c) {
  if (effects_size + 16 > effects_capacity) {
    if (effects_capacity == kEffectLimitBytes) {
      fprintf(stderr, "native effect limit: step=%d bytes=%zu kind=%u a=%x b=%x\n",
              snes_frame_counter + 1, effects_size, kind, a, b);
      Die("native effect trace exceeded 64 MiB hard limit; no events truncated");
    }
    size_t capacity = effects_capacity ? effects_capacity * 2 : 64 * 1024;
    uint8_t *grown = realloc(effects, capacity);
    if (!grown) Die("native effect trace allocation failed; no events truncated");
    effects = grown;
    effects_capacity = capacity;
    if (capacity > kEffectReportBytes)
      fprintf(stderr, "native effect growth: step=%d capacity=%zu kind=%u a=%x b=%x\n",
              snes_frame_counter + 1, capacity, kind, a, b);
  }
  Put32(effects + effects_size, kind);
  Put32(effects + effects_size + 4, a);
  Put32(effects + effects_size + 8, b);
  Put32(effects + effects_size + 12, c);
  effects_size += 16;
  if (kind < 52) effect_kind_counts[kind]++;
}
void sm_native_begin_step(void) {
  effects_size = 0;
  memset(effect_kind_counts, 0, sizeof(effect_kind_counts));
}
const uint8_t *sm_native_effects(void) {
  static const uint8_t empty;
  return effects ? effects : &empty;
}
size_t sm_native_effects_size(void) { return effects_size; }

// Observe the entry context without changing admission or the compared effects.
// This recognizes the retained Fire/draw exclusion only when no other guard
// can explain refusal. Update refusals always belong to the other bucket.
static bool OnlyCeresExclusion(unsigned entry) {
  if (entry == 1 || !(ceres_status & 0x8000) || equipped_beams || hyper_beam_flag ||
      *(uint16_t *)(g_ram + 0xac4) || *(uint16_t *)(g_ram + 0xac6))
    return false;
  for (unsigned i = 0; i < 10; i++) {
    if (!projectile_bomb_instruction_ptr[i]) continue;
    if (i >= 5) return false;
    if (!((projectile_type[i] == 0x8000 && projectile_bomb_pre_instructions[i] == 0xaef3) ||
          (projectile_type[i] == 0x8700 && projectile_bomb_pre_instructions[i] == 0xb169)))
      return false;
  }
  return true;
}

int sm_native_dispatch(unsigned entry) {
  if (entry >= 4 || active_entry || sm_native_scroll_active() || sm_native_ceres_door_active() || sm_native_door_scrolling_active() || sm_native_camera_active() || sm_native_item_instruction_active())
    Die("invalid or nested projectile entry");
  sm_native_room_transition_child_begin(0, entry);
  active_entry = entry + 1;
  entry_oam = oam_next_ptr;
  entry_has_ordinary_shot = false;
  for (unsigned i = 0; i < 5; i++) {
    if (projectile_type[i] == 0x8000 || projectile_type[i] == 0x8700)
      entry_has_ordinary_shot = true;
  }
  sm_native_effect(5, entry, 0, 0);
  bool high_bit = (ceres_status & 0x8000) != 0;
  bool only_ceres_exclusion = OnlyCeresExclusion(entry);
  if (high_bit) projectile_counts[21 + entry]++;
  if (!projectile_hook) {
    projectile_counts[entry]++;
    return 0;
  }
  projectile_counts[4 + entry]++;
  sm_native_room_transition_child_rust(0, 0);
  int result = projectile_hook(projectile_context, entry);
  if (result == 1) {
    projectile_counts[8 + entry]++;
    if (high_bit) projectile_counts[25 + entry]++;
    sm_native_room_transition_child_rust(0, 1);
    return 1;
  }
  if (result == 0) {
    projectile_counts[12 + entry]++;
    projectile_counts[(only_ceres_exclusion ? 29 : 33) + entry]++;
    projectile_counts[entry]++;
    return 0;
  }
  Die("Rust projectile owned failure; C retry prohibited");
  return 1;
}
void sm_native_entry_done(unsigned entry) {
  if (active_entry != entry + 1) Die("projectile entry completion mismatch");
  if (entry >= 2 && entry_has_ordinary_shot && oam_next_ptr > entry_oam)
    projectile_counts[17 + entry] += (oam_next_ptr - entry_oam) / 4;
  sm_native_effect(6, entry, 0, 0);
  active_entry = 0;
  sm_native_room_transition_child_done(0, entry);
}

int sm_native_spawn_plm(uint8_t *ram, uint16_t header) {
  projectile_counts[16]++; // Count requests before entering foreign gameplay.
  if (!active_entry || !projectile_hook || header < 0x8000 || cur_coll_amt32)
    return -1;
  // The quicksand setup can dereference the Samus collision invocation's
  // external pointer. Projectile invocations have no such context.
  const uint8_t *definition = RomPtr(0x840000 | header);
  uint16_t setup = definition[0] | definition[1] << 8;
  if (setup == 0xb4c4 && (ram[(uint8_t *)&samus_collision_direction - g_ram] & 2))
    return -2;
  memcpy(g_ram, ram, sizeof(g_ram));
  uint8_t result = SpawnPLM(header);
  memcpy(ram, g_ram, sizeof(g_ram));
  return result;
}

static void Append(const void *p, size_t n) {
  if (observation_size + n > sizeof(observation)) Die("native observation capacity exceeded");
  memcpy(observation + observation_size, p, n);
  observation_size += n;
}
static void Number(uint64_t v) {
  uint8_t bytes[8];
  for (unsigned i = 0; i < 8; i++) bytes[i] = (uint8_t)(v >> (i * 8));
  Append(bytes, sizeof(bytes));
}
#define N(x) Number((uint64_t)(x))
#define A(x) Append((x), sizeof(x))

static void CapturePpu(Ppu *p) {
  A(p->vram); A(p->cgram); A(p->oam); A(p->highOam);
  A(p->objPixelBufferXX); A(p->objPriorityBufferXX); A(p->m7matrix);
  A(p->mathEnabled); A(p->screenEnabled); A(p->screenWindowed);
  A(p->brightnessMult); A(p->brightnessMultHalf); A(p->mosaicModulo);
  A(p->bgBuffers[0].data); A(p->bgBuffers[1].data); A(p->objBuffer.data);
  N(p->vramPointer); N(p->vramIncrementOnHigh); N(p->vramIncrement); N(p->vramRemapMode); N(p->vramReadBuffer); N(p->cgramPointer); N(p->cgramSecondWrite); N(p->cgramBuffer); N(p->oamAdr); N(p->oamAdrWritten); N(p->oamInHigh); N(p->oamInHighWritten); N(p->oamSecondWrite); N(p->oamBuffer); N(p->objPriority); N(p->objTileAdr1); N(p->objTileAdr2); N(p->objSize); N(p->timeOver); N(p->rangeOver); N(p->objInterlace); N(p->scrollPrev); N(p->scrollPrev2); N(p->mosaicSize); N(p->mosaicStartLine); N(p->m7prev); N(p->m7largeField); N(p->m7charFill); N(p->m7xFlip); N(p->m7yFlip); N(p->m7extBg); N(p->m7startX); N(p->m7startY); N(p->window1left); N(p->window1right); N(p->window2left); N(p->window2right); N(p->clipMode); N(p->preventMathMode); N(p->addSubscreen); N(p->subtractColor); N(p->halfColor); N(p->fixedColorR); N(p->fixedColorG); N(p->fixedColorB); N(p->forcedBlank); N(p->brightness); N(p->mode); N(p->bg3priority); N(p->evenFrame); N(p->pseudoHires); N(p->overscan); N(p->frameOverscan); N(p->interlace); N(p->frameInterlace); N(p->directColor); N(p->hCount); N(p->vCount); N(p->hCountSecond); N(p->vCountSecond); N(p->countersLatched); N(p->ppu1openBus); N(p->ppu2openBus); N(p->windowsel); N(p->extraLeftCur); N(p->extraRightCur); N(p->extraLeftRight); N(p->mosaicEnabled); N(p->lastBrightnessMult); N(p->lineHasSprites); N(p->renderPitch);
  for (unsigned i = 0; i < 4; i++) {
    BgLayer *b = &p->bgLayer[i];
    N(b->hScroll); N(b->vScroll); N(b->tilemapAdr); N(b->tileAdr);
    N(b->bigTiles); N(b->mosaicEnabled); N(b->tilemapWider); N(b->tilemapHigher);
  }
  for (unsigned i = 0; i < 5; i++) {
    N(p->layer[i].mainScreenEnabled); N(p->layer[i].subScreenEnabled);
    N(p->layer[i].mainScreenWindowed); N(p->layer[i].subScreenWindowed);
  }
  for (unsigned i = 0; i < 6; i++) {
    WindowLayer *w = &p->windowLayer[i];
    N(w->window1enabled); N(w->window2enabled);
    N(w->window1inversed); N(w->window2inversed); N(w->maskLogic);
  }
}
static void CaptureDsp(Dsp *p) {
  // DSP allocation/reset leaves padding unspecified: serialize fields, never
  // raw structs or process pointers.
  A(p->ram);
  for (unsigned i = 0; i < 8; i++) {
    DspChannel *c = &p->channel[i];
    N(c->pitch); N(c->pitchCounter); N(c->pitchModulation); A(c->decodeBuffer);
    N(c->srcn); N(c->decodeOffset); N(c->previousFlags); N(c->old); N(c->older);
    N(c->useNoise); A(c->adsrRates); N(c->rateCounter); N(c->adsrState);
    N(c->sustainLevel); N(c->useGain); N(c->gainMode); N(c->directGain);
    N(c->gainValue); N(c->gain); N(c->keyOn); N(c->keyOff); N(c->sampleOut);
    N(c->volumeL); N(c->volumeR); N(c->echoEnable);
  }
  N(p->dirPage); N(p->evenCycle); N(p->mute); N(p->reset);
  N(p->masterVolumeL); N(p->masterVolumeR); N(p->noiseSample); N(p->noiseRate);
  N(p->noiseCounter); N(p->echoWrites); N(p->echoVolumeL); N(p->echoVolumeR);
  N(p->feedbackVolume); N(p->echoBufferAdr); N(p->echoDelay); N(p->echoRemain);
  N(p->echoBufferIndex); N(p->firBufferIndex); A(p->firValues);
  A(p->firBufferL); A(p->firBufferR); A(p->sampleBuffer); N(p->sampleOffset);
}

static void CaptureSfxRegs(SfxRegs *p) {
  N(p->priority); N(p->chan_idx_x2); N(p->chan_voice_bitset_ptr); N(p->cur_sound); N(p->chan_voice_mask_ptr); N(p->chan_voice_index_ptr); N(p->some_value); N(p->cur_ptr); N(p->enabled_voices); N(p->voices_to_setup); N(p->chan_idx); N(p->voices_remaining); N(p->cur_sound_index); N(p->voice_id); N(p->init_flag); N(p->voice_index);
}
static void CaptureSfxChannel(SfxChan *p) {
  N(p->voice_bitset); N(p->instr_timer); N(p->subnote); N(p->repeat_pt); N(p->note); N(p->ptr_idx); N(p->voice_index); N(p->voice_index_mult8); N(p->release_timer); N(p->volume); N(p->target_note); N(p->phase_invert); N(p->ptr); N(p->repeat_ctr); N(p->enable_pitch_slide); N(p->release_flag); N(p->enable_pitch_slide_legato); N(p->adsr_settings); N(p->channel_mask); N(p->legato); N(p->disable); N(p->subnote_delta); N(p->update_adsr);
}
static void CaptureSpcPlayer(SpcPlayer *p) {
  N(p->timer_cycles); N(p->reg_write_history != NULL);
  A(p->port_to_snes); A(p->last_value_from_snes); A(p->input_ports); A(p->ram);
  N(p->counter_sf0c); N(p->always_zero); N(p->temp_accum); N(p->ttt); N(p->did_affect_volumepitch_flag); N(p->addr0); N(p->addr1); N(p->lfsr_value); N(p->is_chan_on); N(p->fast_forward); N(p->word_20); N(p->music_ptr_toplevel); N(p->block_count); N(p->sfx_timer_accum); N(p->chn); N(p->key_ON); N(p->key_OFF); N(p->cur_chan_bit); N(p->reg_FLG); N(p->reg_NON); N(p->reg_EON); N(p->echo_stored_time); N(p->echo_parameter_EDL); N(p->reg_EFB); N(p->global_transposition); N(p->main_tempo_accum); N(p->tempo); N(p->tempo_fade_num_ticks); N(p->tempo_fade_final); N(p->tempo_fade_add); N(p->master_volume); N(p->master_volume_fade_ticks); N(p->master_volume_fade_target); N(p->master_volume_fade_add_per_tick); N(p->vol_dirty); N(p->percussion_base_id); N(p->echo_volume_left); N(p->echo_volume_right); N(p->echo_volume_fade_add_left); N(p->echo_volume_fade_add_right); N(p->echo_volume_fade_ticks); N(p->echo_volume_fade_target_left); N(p->echo_volume_fade_target_right); N(p->p_echoBuffer); N(p->memset_ptr); N(p->memset_ctr); N(p->disable_sfx2); N(p->byte_4B1); N(p->last_written_edl);
  for (unsigned i = 0; i < 8; i++) {
    Channel *p = &g_spc_player->channel[i];
    N(p->pattern_order_ptr_for_chan); N(p->index); N(p->note_ticks_left); N(p->note_keyoff_ticks_left); N(p->subroutine_num_loops); N(p->volume_fade_ticks); N(p->pan_num_ticks); N(p->pitch_slide_length); N(p->pitch_slide_delay_left); N(p->vibrato_hold_count); N(p->vib_depth); N(p->tremolo_hold_count); N(p->tremolo_depth); N(p->vibrato_change_count); N(p->note_length); N(p->note_gate_off_fixedpt); N(p->channel_volume_master); N(p->instrument_id); N(p->instrument_pitch_base); N(p->saved_pattern_ptr); N(p->pattern_start_ptr); N(p->pitch_envelope_num_ticks); N(p->pitch_envelope_delay); N(p->pitch_envelope_direction); N(p->pitch_envelope_slide_value); N(p->vibrato_count); N(p->vibrato_rate); N(p->vibrato_delay_ticks); N(p->vibrato_fade_num_ticks); N(p->vibrato_fade_add_per_tick); N(p->vibrato_depth_target); N(p->tremolo_count); N(p->tremolo_rate); N(p->tremolo_delay_ticks); N(p->channel_transposition); N(p->channel_volume); N(p->volume_fade_addpertick); N(p->volume_fade_target); N(p->final_volume); N(p->pan_value); N(p->pan_add_per_tick); N(p->pan_target_value); N(p->pan_flag_with_phase_invert); N(p->pitch); N(p->pitch_add_per_tick); N(p->pitch_target); N(p->fine_tune); N(p->cutk);
  }
  CaptureSfxRegs(&p->sfx1); CaptureSfxRegs(&p->sfx2); CaptureSfxRegs(&p->sfx3);
  for (unsigned i = 0; i < 4; i++) CaptureSfxChannel(&p->sfx_chans_1[i]);
  for (unsigned i = 0; i < 2; i++) CaptureSfxChannel(&p->sfx_chans_2[i]);
  for (unsigned i = 0; i < 2; i++) CaptureSfxChannel(&p->sfx_chans_3[i]);
  CaptureDsp(p->dsp);
}

void sm_native_capture(const uint32_t *pixels, const int16_t *audio) {
  if (effects_size > kEffectReportBytes) {
    fprintf(stderr, "native effect volume: step=%d bytes=%zu", snes_frame_counter, effects_size);
    for (unsigned kind = 1; kind < 52; kind++)
      fprintf(stderr, " kind%u=%llu", kind, (unsigned long long)effect_kind_counts[kind]);
    fputc('\n', stderr);
  }
  observation_size = 0;
  Append("SMNOBS02", 8);
  uint64_t semantic_counts[8];
  sm_native_counts(semantic_counts);
  for (unsigned i = 0; i < 5; i++) N(semantic_counts[i]);
  N(sm_native_requested_buttons());
  N(snes_frame_counter); N(currently_installed_bug_fix_counter);
  N(cur_coll_amt32 != NULL);
  if (cur_coll_amt32) Die("projectile host-step ended inside collision context");
  N(g_calling_asm_from_c); N(g_runmode); N(g_use_my_apu_code);
  N(eproj_spawn_pt.x); N(eproj_spawn_pt.y);
  Append(g_sram, 0x2000);
  Snes *s = g_snes;
  N(s->ppu == s->my_ppu); N(s->disableRender); N(s->runningWhichVersion);
  N(s->ramAdr); N(s->hPos); N(s->vPos); N(s->frames); N(s->cpuCyclesLeft);
  N(s->cpuMemOps); Append(&s->apuCatchupCycles, sizeof(s->apuCatchupCycles));
  N(s->hIrqEnabled); N(s->vIrqEnabled); N(s->nmiEnabled); N(s->hTimer);
  N(s->vTimer); N(s->inNmi); N(s->inIrq); N(s->inVblank); A(s->portAutoReadX);
  N(s->autoJoyRead); N(s->autoJoyTimer); N(s->ppuLatch); N(s->multiplyA);
  N(s->multiplyResult); N(s->divideA); N(s->divideResult); N(s->fastMem); N(s->openBus);
  N(s->input1->currentState); N(s->input2->currentState);
  Cpu *c = s->cpu;
  N(c->a); N(c->x); N(c->y); N(c->sp); N(c->pc); N(c->dp); N(c->k); N(c->db);
  N(c->c); N(c->z); N(c->v); N(c->n); N(c->i); N(c->d); N(c->xf); N(c->mf);
  N(c->e); N(c->irqWanted); N(c->nmiWanted); N(c->waiting); N(c->stopped);
  N(c->cyclesUsed); N(c->spBreakpoint); N(c->in_emu);
  CapturePpu(s->my_ppu); CapturePpu(s->snes_ppu);
  Dma *d = s->dma;
  for (unsigned i = 0; i < 8; i++) {
    DmaChannel *p = &d->channel[i];
    N(p->bAdr); N(p->aAdr); N(p->aBank); N(p->size); N(p->indBank);
    N(p->tableAdr); N(p->repCount); N(p->unusedByte); N(p->dmaActive);
    N(p->hdmaActive); N(p->mode); N(p->fixed); N(p->decrement); N(p->indirect);
    N(p->fromB); N(p->unusedBit); N(p->doTransfer); N(p->terminated); N(p->offIndex);
  }
  N(d->hdmaTimer); N(d->dmaTimer); N(d->dmaBusy);
  SpcPlayer *p = g_spc_player;
  CaptureSpcPlayer(p);
  Apu *a = s->apu;
  A(a->ram); N(a->romReadable); N(a->dspAdr); N(a->cycles);
  A(a->inPorts); A(a->outPorts); N(a->cpuCyclesLeft);
  for (unsigned i = 0; i < 3; i++) {
    N(a->timer[i].cycles); N(a->timer[i].divider); N(a->timer[i].target);
    N(a->timer[i].counter); N(a->timer[i].enabled);
  }
  Spc *sp = a->spc;
  N(sp->a); N(sp->x); N(sp->y); N(sp->sp); N(sp->pc); N(sp->c); N(sp->z);
  N(sp->v); N(sp->n); N(sp->i); N(sp->h); N(sp->p); N(sp->b);
  N(sp->stopped); N(sp->cyclesUsed);
  CaptureDsp(a->dsp);
  uint8_t queue[71];
  sm_native_audio_queue(queue);
  A(queue);
  Append(pixels, 256 * 240 * sizeof(uint32_t));
  Append(audio, 534 * 2 * sizeof(int16_t));
}
const uint8_t *sm_native_observation(void) { return observation; }
size_t sm_native_observation_size(void) { return observation_size; }
