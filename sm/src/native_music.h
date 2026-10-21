#ifndef SM_NATIVE_MUSIC_H
#define SM_NATIVE_MUSIC_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

typedef uint8_t uint8;
typedef uint16_t uint16;
typedef uint32_t uint32;
typedef int8_t int8;
typedef int16_t int16;
typedef int32_t int32;
typedef struct Dsp Dsp;
typedef struct DspRegWriteHistory DspRegWriteHistory;

typedef struct Channel {
  uint16 pattern_order_ptr_for_chan;
  uint8 index, note_ticks_left, note_keyoff_ticks_left, subroutine_num_loops, volume_fade_ticks,
      pan_num_ticks, pitch_slide_length, pitch_slide_delay_left, vibrato_hold_count, vib_depth,
      tremolo_hold_count, tremolo_depth, vibrato_change_count, note_length, note_gate_off_fixedpt,
      channel_volume_master, instrument_id;
  uint16 instrument_pitch_base, saved_pattern_ptr, pattern_start_ptr;
  uint8 pitch_envelope_num_ticks, pitch_envelope_delay, pitch_envelope_direction,
      pitch_envelope_slide_value, vibrato_count, vibrato_rate, vibrato_delay_ticks,
      vibrato_fade_num_ticks, vibrato_fade_add_per_tick, vibrato_depth_target, tremolo_count,
      tremolo_rate, tremolo_delay_ticks, channel_transposition;
  uint16 channel_volume, volume_fade_addpertick;
  uint8 volume_fade_target, final_volume;
  uint16 pan_value, pan_add_per_tick;
  uint8 pan_target_value, pan_flag_with_phase_invert;
  uint16 pitch, pitch_add_per_tick;
  uint8 pitch_target, fine_tune, cutk;
} Channel;

typedef struct SfxChan {
  uint8 voice_bitset, instr_timer, subnote, repeat_pt, note, ptr_idx, voice_index,
      voice_index_mult8, release_timer, volume, target_note, phase_invert;
  uint16 ptr;
  uint8 repeat_ctr, enable_pitch_slide, release_flag, enable_pitch_slide_legato;
  uint16 adsr_settings;
  uint8 channel_mask, legato, disable, subnote_delta, update_adsr;
} SfxChan;

typedef struct SfxRegs {
  uint8 priority, chan_idx_x2;
  uint16 chan_voice_bitset_ptr;
  uint8 cur_sound;
  uint16 chan_voice_mask_ptr, chan_voice_index_ptr;
  uint8 some_value;
  uint16 cur_ptr;
  uint8 enabled_voices, voices_to_setup, chan_idx, voices_remaining, cur_sound_index, voice_id,
      init_flag, voice_index;
} SfxRegs;

typedef struct SpcPlayer {
  DspRegWriteHistory *reg_write_history;
  uint8 timer_cycles;
  Dsp *dsp;
  uint8 port_to_snes[4], last_value_from_snes[4];
  uint16 counter_sf0c, always_zero, temp_accum;
  uint8 ttt, did_affect_volumepitch_flag;
  uint16 addr0, addr1, lfsr_value;
  uint8 is_chan_on, fast_forward;
  uint16 word_20, music_ptr_toplevel;
  uint8 block_count, sfx_timer_accum, chn, key_ON, key_OFF, cur_chan_bit, reg_FLG, reg_NON, reg_EON,
      echo_stored_time, echo_parameter_EDL, reg_EFB, global_transposition, main_tempo_accum;
  uint16 tempo;
  uint8 tempo_fade_num_ticks, tempo_fade_final;
  uint16 tempo_fade_add, master_volume;
  uint8 master_volume_fade_ticks, master_volume_fade_target;
  uint16 master_volume_fade_add_per_tick;
  uint8 vol_dirty, percussion_base_id;
  uint16 echo_volume_left, echo_volume_right, echo_volume_fade_add_left, echo_volume_fade_add_right;
  uint8 echo_volume_fade_ticks, echo_volume_fade_target_left, echo_volume_fade_target_right;
  uint16 p_echoBuffer, memset_ptr;
  uint8 memset_ctr, disable_sfx2, byte_4B1;
  Channel channel[8];
  SfxRegs sfx1, sfx2, sfx3;
  SfxChan sfx_chans_1[4];
  SfxChan sfx_chans_2[2], sfx_chans_3[2];
  uint8 last_written_edl;
  uint8 input_ports[4];
  uint8 ram[65536];
} SpcPlayer;

void Dsp_Write(SpcPlayer *p, uint8 reg, uint8 value) __asm__("sm_music_Dsp_Write");
void Not_Implemented(void) __asm__("sm_music_Not_Implemented");
void WritePitchInner(SpcPlayer *p, Channel *c, uint16 pitch) __asm__("sm_music_WritePitchInner");
void Channel_SetInstrument_NoSetId(SpcPlayer *p, Channel *c, uint8 instrument)
    __asm__("sm_music_Channel_SetInstrument_NoSetId");
uint8 EffectByteLength(uint8 effect) __asm__("sm_music_EffectByteLength");
bool WantWriteKof(SpcPlayer *p, Channel *c) __asm__("sm_music_WantWriteKof");
void CalcFinalVolume(SpcPlayer *p, Channel *c, uint8 volume) __asm__("sm_music_CalcFinalVolume");

void sm_music_SetupEchoParameter_EDL(SpcPlayer *p, uint8 value);
void sm_music_WriteVolumeToDsp(SpcPlayer *p, Channel *c, uint16 volume);
void sm_music_HandlePanAndSweep(SpcPlayer *p, Channel *c);
void sm_music_Music_HandleCmdFromSnes(SpcPlayer *p);

#endif
