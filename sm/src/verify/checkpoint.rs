//! Local verifier checkpoints, taken between completed frames and audio steps.
//! Unlike frontend save states, these preserve both PPUs and live audio state.
//! Pointers and padding are never serialized; initialized allocations stay owned
//! by the new host. These files are disposable, ROM-derived, and must stay local.
use sm::recording::RecordingBytes;
use sm::{apu_queue::ApuQueue, game_state::Game, rtl::StateRecorder, types::*};
use std::path::Path;

const MAGIC: &[u8; 8] = b"SMVERIFY";

// A cache fingerprint, not an authentication mechanism. The full payload is
// checked before any state is loaded, including lengths and metadata.
fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn compatibility() -> u64 {
    // Gameplay edits can reuse checkpoints. Changes to the state layout,
    // oracle, audio or hardware require regeneration from boot.
    let mut source = concat!(
        include_str!("checkpoint.rs"),
        include_str!("../types.rs"),
        include_str!("../initial_state.rs"),
        include_str!("../game/enemy_layouts.rs"),
        include_str!("../verify.rs"),
        include_str!("../verify_dll.rs"),
        include_str!("../main.rs"),
        include_str!("../native_audio.rs"),
        include_str!("../cpu_infra.rs"),
        include_str!("../rtl.rs"),
        include_str!("../recording.rs"),
        include_str!("../save_ram.rs"),
        include_str!("../snapshot_io.rs"),
        include_str!("../apu_ram.rs"),
        include_str!("../apu_queue.rs"),
        include_str!("../audio_upload.rs"),
        include_str!("../audio_snapshot.rs"),
        include_str!("../spc_player.rs"),
        include_str!("../native_player_ram.rs"),
        include_str!("../snes/snes.rs"),
        include_str!("../snes/cpu.rs"),
        include_str!("../snes/ppu.rs"),
        include_str!("../snes/registers.rs"),
        include_str!("../snes/apu.rs"),
        include_str!("../snes/spc.rs"),
        include_str!("../snes/dsp.rs"),
        include_str!("../snes/dma.rs"),
        include_str!("../snes/cart.rs"),
        include_str!("../snes/input.rs"),
        include_str!("../snes/snes_other.rs"),
    )
    .replace("\r\n", "\n");
    // Accessor-only gameplay cleanups do not alter checkpoint compatibility.
    let game_source = include_str!("../game_state.rs").replace("\r\n", "\n");
    source.push_str(game_source.split_once("\n}\n").expect("Game declaration").0);
    fingerprint(source.as_bytes())
}

enum Archive<'a> {
    Save(Vec<u8>),
    Load(&'a [u8]),
}

impl Archive<'_> {
    fn bytes(&mut self, value: &mut [u8]) -> Result<(), String> {
        match self {
            Self::Save(output) => output.extend_from_slice(value),
            Self::Load(input) => {
                let (head, tail) = input
                    .split_at_checked(value.len())
                    .ok_or("truncated checkpoint")?;
                value.copy_from_slice(head);
                *input = tail;
            }
        }
        Ok(())
    }
}

trait State {
    fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String>;
}

macro_rules! scalar {
    ($($ty:ty),* $(,)?) => {$ (
        impl State for $ty {
            fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String> {
                let mut bytes = self.to_le_bytes();
                archive.bytes(&mut bytes)?;
                if matches!(archive, Archive::Load(_)) {
                    *self = Self::from_le_bytes(bytes);
                }
                Ok(())
            }
        }
    )*};
}
scalar!(u8, i8, u16, i16, u32, i32, u64, f64);

impl State for bool {
    fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String> {
        let mut byte = u8::from(*self);
        byte.state(archive)?;
        if byte > 1 {
            return Err("invalid checkpoint boolean".into());
        }
        *self = byte != 0;
        Ok(())
    }
}

impl<T: State, const N: usize> State for [T; N] {
    fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String> {
        for value in self {
            value.state(archive)?;
        }
        Ok(())
    }
}

// Exhaustive patterns make newly added hardware fields a compile error until
// their checkpoint behavior is specified. Allocation pointers and padding are omitted.
macro_rules! fields {
    ($ty:ident { $($field:ident),* $(,)? } skip { $($skip:ident),* $(,)? }) => {
        impl State for $ty {
            fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String> {
                let Self { $($field,)* $($skip: _,)* } = self;
                $($field.state(archive)?;)*
                Ok(())
            }
        }
    };
}

fields!(Apu { ram, registers } skip { spc, dsp });
fields!(ApuRegisters {
    romReadable, dspAdr, cycles, inPorts, outPorts, timer, cpuCyclesLeft, history
} skip { pad, snapshot_padding0 });
impl State for sm::apu_ram::ApuRam {
    fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String> {
        for offset in 0..65536 {
            let mut byte = self.read(offset as u16);
            byte.state(archive)?;
            self.write(offset as u16, byte);
        }
        Ok(())
    }
}

fields!(BgLayer {
    hScroll, vScroll, tilemapWider, tilemapHigher, tilemapAdr, tileAdr, bigTiles,
    mosaicEnabled
} skip {  });

fields!(Channel {
    pattern_order_ptr_for_chan, index, note_ticks_left, note_keyoff_ticks_left,
    subroutine_num_loops, volume_fade_ticks, pan_num_ticks, pitch_slide_length,
    pitch_slide_delay_left, vibrato_hold_count, vib_depth, tremolo_hold_count,
    tremolo_depth, vibrato_change_count, note_length, note_gate_off_fixedpt,
    channel_volume_master, instrument_id, instrument_pitch_base, saved_pattern_ptr,
    pattern_start_ptr, pitch_envelope_num_ticks, pitch_envelope_delay,
    pitch_envelope_direction, pitch_envelope_slide_value, vibrato_count, vibrato_rate,
    vibrato_delay_ticks, vibrato_fade_num_ticks, vibrato_fade_add_per_tick,
    vibrato_depth_target, tremolo_count, tremolo_rate, tremolo_delay_ticks,
    channel_transposition, channel_volume, volume_fade_addpertick, volume_fade_target,
    final_volume, pan_value, pan_add_per_tick, pan_target_value,
    pan_flag_with_phase_invert, pitch, pitch_add_per_tick, pitch_target, fine_tune, cutk
} skip {  });

fields!(Cpu {
    memType, a, x, y, sp, pc, dp, k, db, c, z, v, n, i, d, xf, mf, e, irqWanted,
    nmiWanted, waiting, stopped, cyclesUsed, spBreakpoint, in_emu
} skip { mem });

fields!(Dma {
    channel, hdmaTimer, dmaTimer, dmaBusy
} skip { snes, pad, snapshot_padding0 });

fields!(DmaChannel {
    bAdr, aAdr, aBank, size, indBank, tableAdr, repCount, unusedByte, dmaActive,
    hdmaActive, mode, fixed, decrement, indirect, fromB, unusedBit, doTransfer,
    terminated, offIndex
} skip { snapshot_padding0, snapshot_padding1, snapshot_padding2, snapshot_padding3 });

fields!(Dsp {
    ram, channel, dirPage, evenCycle, mute, reset, masterVolumeL, masterVolumeR,
    noiseSample, noiseRate, noiseCounter, echoWrites, echoVolumeL, echoVolumeR,
    feedbackVolume, echoBufferAdr, echoDelay, echoRemain, echoBufferIndex,
    firBufferIndex, firValues, firBufferL, firBufferR, sampleBuffer, sampleOffset
} skip { apu_ram, snapshot_padding0, snapshot_padding1, snapshot_padding2 });

fields!(DspChannel {
    pitch, pitchCounter, pitchModulation, decodeBuffer, srcn, decodeOffset,
    previousFlags, old, older, useNoise, adsrRates, rateCounter, adsrState,
    sustainLevel, useGain, gainMode, directGain, gainValue, gain, keyOn, keyOff,
    sampleOut, volumeL, volumeR, echoEnable
} skip { snapshot_padding0, snapshot_padding1, snapshot_padding2, snapshot_padding3, snapshot_padding4, snapshot_padding5, snapshot_padding6 });

fields!(DspRegWriteHistory {
    count, addr, val
} skip {  });

fields!(Input {
    type_0, currentState
} skip { snes });

fields!(Layer {
    mainScreenEnabled, subScreenEnabled, mainScreenWindowed, subScreenWindowed
} skip {  });

fields!(Ppu {
    vram, vramPointer, vramIncrementOnHigh, vramIncrement, vramRemapMode,
    vramReadBuffer, cgram, cgramPointer, cgramSecondWrite, cgramBuffer, oam, highOam,
    oamAdr, oamAdrWritten, oamInHigh, oamInHighWritten, oamSecondWrite, oamBuffer,
    objPriority, objTileAdr1, objTileAdr2, objSize, objPixelBufferXX,
    objPriorityBufferXX, timeOver, rangeOver, objInterlace, bgLayer, scrollPrev,
    scrollPrev2, mosaicSize, mosaicStartLine, layer, m7matrix, m7prev, m7largeField,
    m7charFill, m7xFlip, m7yFlip, m7extBg, m7startX, m7startY, windowLayer, window1left,
    window1right, window2left, window2right, clipMode, preventMathMode, addSubscreen,
    subtractColor, halfColor, mathEnabled, fixedColorR, fixedColorG, fixedColorB,
    forcedBlank, brightness, mode, bg3priority, evenFrame, pseudoHires, overscan,
    frameOverscan, interlace, frameInterlace, directColor, hCount, vCount, hCountSecond,
    vCountSecond, countersLatched, ppu1openBus, ppu2openBus, pixelbuffer_placeholder,
    windowsel, extraLeftCur, extraRightCur, extraLeftRight, screenEnabled,
    screenWindowed, mosaicEnabled, lastBrightnessMult, lineHasSprites, bgBuffers,
    objBuffer, renderPitch, brightnessMult, brightnessMultHalf, mosaicModulo
} skip { snes, renderBuffer, snapshot_padding0, snapshot_padding1, snapshot_padding2, snapshot_padding3, snapshot_padding4 });

fields!(PpuPixelPrioBufs {
    data
} skip {  });

fields!(SfxChan {
    voice_bitset, instr_timer, subnote, repeat_pt, note, ptr_idx, voice_index,
    voice_index_mult8, release_timer, volume, target_note, phase_invert, ptr,
    repeat_ctr, enable_pitch_slide, release_flag, enable_pitch_slide_legato,
    adsr_settings, channel_mask, legato, disable, subnote_delta, update_adsr
} skip {  });

fields!(SfxRegs {
    priority, chan_idx_x2, chan_voice_bitset_ptr, cur_sound, chan_voice_mask_ptr,
    chan_voice_index_ptr, some_value, cur_ptr, enabled_voices, voices_to_setup,
    chan_idx, voices_remaining, cur_sound_index, voice_id, init_flag, voice_index
} skip {  });

fields!(Snes {
    debug_cycles, debug_apu_cycles, disableRender, runningWhichVersion, ramAdr,
    hPos, vPos, frames, cpuCyclesLeft, cpuMemOps, apuCatchupCycles, hIrqEnabled,
    vIrqEnabled, nmiEnabled, hTimer, vTimer, inNmi, inIrq, inVblank, portAutoReadX,
    autoJoyRead, autoJoyTimer, ppuLatch, multiplyA, multiplyResult, divideA,
    divideResult, fastMem, openBus
} skip { cpu, apu, ppu, snes_ppu, my_ppu, dma, cart, input1, input2, ram, padx, padpad, snapshot_padding0, snapshot_padding1, snapshot_padding2 });

fields!(Spc {
    a, x, y, sp, pc, c, z, v, n, i, h, p, b, stopped, cyclesUsed
} skip { apu });

fields!(NativePlayerState {
    timer_cycles, port_to_snes, last_value_from_snes, counter_sf0c, always_zero,
    temp_accum, ttt, did_affect_volumepitch_flag, addr0, addr1, lfsr_value, is_chan_on,
    fast_forward, word_20, music_ptr_toplevel, block_count, sfx_timer_accum, chn,
    key_ON, key_OFF, cur_chan_bit, reg_FLG, reg_NON, reg_EON, echo_stored_time,
    echo_parameter_EDL, reg_EFB, global_transposition, main_tempo_accum, tempo,
    tempo_fade_num_ticks, tempo_fade_final, tempo_fade_add, master_volume,
    master_volume_fade_ticks, master_volume_fade_target,
    master_volume_fade_add_per_tick, vol_dirty, percussion_base_id, echo_volume_left,
    echo_volume_right, echo_volume_fade_add_left, echo_volume_fade_add_right,
    echo_volume_fade_ticks, echo_volume_fade_target_left, echo_volume_fade_target_right,
    p_echoBuffer, memset_ptr, memset_ctr, disable_sfx2, byte_4B1, channel, sfx1, sfx2,
    sfx3, sfx_chans_1, sfx_chans_2, sfx_chans_3, last_written_edl, input_ports
} skip { reg_write_history, dsp, ram });

fields!(Timer {
    cycles, divider, target, counter, enabled
} skip {  });

fields!(WindowLayer {
    window1enabled, window2enabled, window1inversed, window2inversed, maskLogic
} skip {  });

fields!(Point16U {
    x, y
} skip {  });

fields!(Rect16U {
    x, y, w, h
} skip {  });

fields!(ApuQueue {
    pending, entries, time_since_empty, position, size
} skip { });

fields!(StateRecorder {
    last_inputs, frames_since_last, total_frames, replay_pos, replay_pos_last_complete,
    replay_frame_counter, replay_next_cmd_at, snapshot_flags, replay_cmd, replay_mode,
    log, base_snapshot, has_base_audio_queue, base_audio_queue
} skip {});

impl State for RecordingBytes {
    fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String> {
        let mut length = self.size as u64;
        length.state(archive)?;
        if let Archive::Load(input) = archive {
            let length = usize::try_from(length).map_err(|_| "checkpoint log length overflow")?;
            if length > input.len() {
                return Err("truncated checkpoint log".into());
            }
            unsafe {
                sm::rtl::recording_allocation(self.resize(length));
            }
            self.as_mut_slice().fill(0);
        }
        if self.size != 0 {
            archive.bytes(self.as_mut_slice())?;
        }
        Ok(())
    }
}

unsafe fn machine(game: &mut Game, archive: &mut Archive<'_>) -> Result<(), String> {
    // Game's allocation/ROM view and callbacks belong to SnesInit. Oracle
    // scratch snapshots are overwritten before use on the next paired frame.
    game.g_ram.state(archive)?;
    game.decompress_src.state(archive)?;
    game.eproj_spawn_rect.state(archive)?;
    game.eproj_spawn_r22.state(archive)?;
    game.eproj_spawn_pt.state(archive)?;
    game.eproj_spawn_varE24.state(archive)?;
    game.message_box_das0l_value.state(archive)?;
    game.k_out.state(archive)?;
    game.apu_queue.state(archive)?;
    game.g_runmode.state(archive)?;
    game.g_calling_asm_from_c.state(archive)?;
    game.g_calling_asm_from_c_ret.state(archive)?;
    game.g_fail.state(archive)?;
    game.g_use_my_apu_code.state(archive)?;
    game.hookmode.state(archive)?;
    game.hookcnt.state(archive)?;
    game.hookadr.state(archive)?;
    game.hook_orgbyte.state(archive)?;
    game.hook_fixbug_orgbyte.state(archive)?;
    game.kPatchedCarrysOrg.state(archive)?;
    game.VERIFY_ACTIVE.state(archive)?;
    game.VERIFY_FAILURES.state(archive)?;
    game.g_got_mismatch_count.state(archive)?;
    game.g_other_image.state(archive)?;
    game.g_debug_flag.state(archive)?;
    game.g_new_ppu.state(archive)?;
    game.snes_frame_counter.state(archive)?;
    game.pc_hist.state(archive)?;
    game.pc_hist_ctr.state(archive)?;
    game.pc_bp.state(archive)?;
    game.g_debug_apu_cycles.state(archive)?;
    game.state_recorder.state(archive)?;
    let snes = game.g_snes;
    (*snes).state(archive)?;
    (*(*snes).cpu).state(archive)?;
    (*(*snes).dma).state(archive)?;
    (*(*snes).input1).state(archive)?;
    (*(*snes).input2).state(archive)?;
    (*(*snes).snes_ppu).state(archive)?;
    (*(*snes).my_ppu).state(archive)?;
    (*(*snes).apu).state(archive)?;
    (*(*(*snes).apu).spc).state(archive)?;
    (*(*(*snes).apu).dsp).state(archive)?;
    (*game.g_spc_player).state.state(archive)?;
    (*game.g_spc_player).ram.state(archive)?;
    (*(*game.g_spc_player).state.dsp).state(archive)?;
    let mut save_bytes = game.g_sram.bytes(0..8192);
    archive.bytes(&mut save_bytes)?;
    game.g_sram.write_bytes(0, &save_bytes);
    Ok(())
}

fn input_fingerprint(inputs: &[u16]) -> u64 {
    fingerprint(
        &inputs
            .iter()
            .flat_map(|pad| pad.to_le_bytes())
            .collect::<Vec<_>>(),
    )
}

/// Capture/read exact continuation bytes for local round-trip checks as well
/// as disk checkpoints. The caller owns an initialized, idle verifier Game.
pub(super) unsafe fn capture(game: &mut Game) -> Result<Vec<u8>, String> {
    if !game.VERIFY_ACTIVE
        || game.g_runmode != sm::cpu_infra::RM_BOTH as u8
        || (*game.g_snes).disableRender
        || game.g_other_image
        || game.VERIFY_FAILURES != 0
        || game.g_fail
        || game.g_calling_asm_from_c
        || game.state_recorder.replay_mode
        || !(*game.g_spc_player).state.reg_write_history.is_null()
        || (*(*game.g_snes).cart).ramSize != 8192
    {
        return Err("checkpoint requires a successful, idle verifier frame".into());
    }
    let mut archive = Archive::Save(Vec::new());
    machine(game, &mut archive)?;
    let Archive::Save(bytes) = archive else {
        unreachable!()
    };
    Ok(bytes)
}

pub(super) unsafe fn save(
    path: &Path,
    game: &mut Game,
    rom: &[u8],
    inputs: &[u16],
    completed: usize,
) -> Result<(), String> {
    let payload = capture(game)?;
    let mut bytes = Vec::from(MAGIC.as_slice());
    for word in [
        compatibility(),
        fingerprint(rom),
        input_fingerprint(&inputs[..completed]),
        completed as u64,
        u64::from(game.g_use_my_apu_code),
        fingerprint(&payload),
    ] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes.extend_from_slice(&payload);
    // create_new prevents silently replacing a checkpoint under another run.
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(&bytes)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(())
}

pub(super) unsafe fn load(
    path: &Path,
    game: &mut Game,
    rom: &[u8],
    inputs: &[u16],
) -> Result<usize, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() < 56 || &bytes[..8] != MAGIC {
        return Err("not a verifier checkpoint".into());
    }
    let word = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
    let completed = usize::try_from(word(32)).map_err(|_| "checkpoint frame overflow")?;
    if word(8) != compatibility() {
        return Err("checkpoint runtime changed; regenerate from boot".into());
    }
    if word(16) != fingerprint(rom) {
        return Err("checkpoint ROM differs".into());
    }
    if completed > inputs.len() || word(24) != input_fingerprint(&inputs[..completed]) {
        return Err("checkpoint input prefix differs".into());
    }
    if word(40) != u64::from(game.g_use_my_apu_code) {
        return Err("checkpoint audio backend differs".into());
    }
    if word(48) != fingerprint(&bytes[56..]) {
        return Err("checkpoint payload is corrupt".into());
    }
    let mut archive = Archive::Load(&bytes[56..]);
    machine(game, &mut archive)?;
    if !matches!(archive, Archive::Load([])) {
        return Err("checkpoint has extra state bytes".into());
    }
    sm::cpu_infra::RtlUpdateSnesPatchForBugfix(game);
    // Verify that every serialized field survives reconstruction. Pointers
    // remain those established by this process's normal initialization.
    if capture(game)? != bytes[56..] {
        return Err("checkpoint state did not round-trip".into());
    }
    Ok(completed)
}
