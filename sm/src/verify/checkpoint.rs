//! Local verifier checkpoints, taken between completed frames and audio steps.
//! Unlike frontend save states, these preserve both PPUs and live audio state.
//! Pointers and padding are never serialized; initialized allocations stay owned
//! by the new host. These files are disposable, ROM-derived, and must stay local.
//! Unsafe entries require the caller's initialized, idle Game to retain valid
//! backend/player/cart allocations and exclusive access through adoption. The
//! codec itself owns complete machine and actor preflight/adoption; no caller
//! callback can report an incomplete machine restore as successful.
use crate::recording::RecordingBytes;
use crate::{
    apu_queue::ApuQueue,
    game_state::{Game, WorkRam},
    rtl::StateRecorder,
    types::*,
};
use std::path::Path;

const MAGIC: &[u8; 8] = b"SMVERIFY";
const APU_RAM_BYTES: usize = 65536;

// A cache fingerprint, not an authentication mechanism. The full payload is
// checked before any state is loaded, including lengths and metadata.
pub fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

/// Reset an initialized, idle machine while retaining its selected world binding.
///
/// # Safety
/// The Game must retain live backend, CPU, cartridge and player allocations,
/// initialized host hooks, and exclusive access throughout the native reset.
/// Native and audio users must be stopped while reset mutates their state.
pub unsafe fn reset_retaining_prepared_world(game: &mut Game) -> Result<(), String> {
    let world = game.world().clone();
    unsafe { crate::rtl::RtlReset(2, game) };
    if !std::sync::Arc::ptr_eq(game.world(), &world) {
        return Err("prepared reset changed the selected binding".into());
    }
    Ok(())
}

// Bind seed-backed checkpoints to their actual original observation, not its path.
// Like the other checkpoint fingerprints this detects stale caches, not hostile input.
pub fn origin(rom: &[u8], seed: Option<&Path>) -> Result<u64, String> {
    let Some(path) = seed else {
        return Ok(fingerprint(rom));
    };
    let prefix = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix("-wram.bin"))
        .ok_or("seed path must name a -wram.bin file")?;
    let mut identity = Vec::from(b"original-emulator-seed\0".as_slice());
    identity.extend_from_slice(&fingerprint(rom).to_le_bytes());
    for region in [
        "wram.bin",
        "sram.bin",
        "vram.bin",
        "cgram.bin",
        "oam.bin",
        "apuram.bin",
        "registers.csv",
    ] {
        let path = path.with_file_name(format!("{prefix}-{region}"));
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        identity.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        identity.extend_from_slice(&fingerprint(&bytes).to_le_bytes());
    }
    Ok(fingerprint(&identity))
}

/// Ordered serialized field/type/count metadata, independent of execution bodies.
#[forbid(unsafe_code)]
pub fn serialized_layout() -> String {
    let mut layout = String::from("SMVERIFY/serialized-state-v1;payload{machine-length:");
    <u64>::layout(&mut layout);
    machine_layout(&mut layout);
    layout.push_str("actors:");
    layout.push_str(&sm_game::actors::snapshot::ActorCheckpointPayload::serialized_layout());
    layout.push('}');
    layout
}

fn compatibility() -> u64 {
    // Layout identity and ROM/origin/input/audio/full-payload/loaded-source/native guards
    // serve separate contracts. Keep the historical runtime-changed error for callers.
    static LAYOUT: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *LAYOUT.get_or_init(|| fingerprint(serialized_layout().as_bytes()))
}

enum Archive<'a> {
    Save(Vec<u8>),
    Load(&'a [u8]),
    Validate(&'a [u8], u8, u8),
}

impl Archive<'_> {
    fn check_field(&self, field: &str) -> Result<(), String> {
        let Self::Validate(input, mode, audio) = self else {
            return Ok(());
        };
        let expected = match field {
            "g_runmode" => Some(*mode),
            "g_use_my_apu_code" => Some(*audio),
            "VERIFY_ACTIVE" => Some(1),
            "g_calling_asm_from_c"
            | "g_fail"
            | "g_other_image"
            | "replay_mode"
            | "disableRender" => Some(0),
            _ => None,
        };
        if let Some(expected) = expected {
            if input.first() != Some(&expected) {
                return Err(format!("checkpoint is not committed: {field}"));
            }
        }
        if field == "VERIFY_FAILURES" && input.get(..8) != Some(&[0; 8]) {
            return Err("checkpoint contains verifier failures".into());
        }
        if field == "snes_frame_counter" && u32::from(*mode) == crate::cpu_infra::RM_MINE {
            let bytes = input.get(..4).ok_or("truncated checkpoint frame counter")?;
            if i32::from_le_bytes(bytes.try_into().unwrap()) <= 0 {
                return Err("prepared checkpoint has no completed frame".into());
            }
        }
        Ok(())
    }
    fn work_ram(&mut self, ram: &WorkRam) -> Result<(), String> {
        for at in 0..ram.len() {
            match self {
                Self::Save(output) => output.push(ram.read_byte(at)),
                Self::Load(input) => {
                    let (head, tail) = input.split_at_checked(1).ok_or("truncated checkpoint")?;
                    ram.write_byte(at, head[0]);
                    *input = tail;
                }
                Self::Validate(input, _, _) => {
                    let (_, tail) = input.split_at_checked(1).ok_or("truncated checkpoint")?;
                    *input = tail;
                }
            }
        }
        Ok(())
    }

    fn bytes(&mut self, value: &mut [u8]) -> Result<(), String> {
        match self {
            Self::Save(output) => output.extend_from_slice(value),
            Self::Load(input) | Self::Validate(input, _, _) => {
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
    fn layout(layout: &mut String);
    fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String>;
}

// The projection is never invoked; its signature infers the actual declared field type.
#[forbid(unsafe_code)]
fn field_layout<S, T: State>(layout: &mut String, name: &str, _field: fn(&S) -> &T) {
    layout.push_str(name);
    layout.push(':');
    T::layout(layout);
}

macro_rules! scalar {
    ($($ty:ty),* $(,)?) => {$ (
        impl State for $ty {
            fn layout(layout: &mut String) {
                layout.push_str(concat!(stringify!($ty), "-le;"));
            }
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
    fn layout(layout: &mut String) {
        layout.push_str("bool-u8-0-or-1;");
    }
    fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String> {
        let mut byte = [u8::from(*self)];
        archive.bytes(&mut byte)?;
        if byte[0] > 1 {
            return Err("invalid checkpoint boolean".into());
        }
        if matches!(archive, Archive::Load(_)) {
            *self = byte[0] != 0;
        }
        Ok(())
    }
}

impl<T: State, const N: usize> State for [T; N] {
    fn layout(layout: &mut String) {
        layout.push_str(&format!("array[{N}]{{"));
        T::layout(layout);
        layout.push('}');
    }
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
            fn layout(layout: &mut String) {
                layout.push_str(concat!(stringify!($ty), "{"));
                $(fields!(@layout $ty, layout, $field);)*
                layout.push('}');
            }
            fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String> {
                let Self { $($field,)* $($skip: _,)* } = self;
                $(archive.check_field(stringify!($field))?; $field.state(archive)?;)*
                Ok(())
            }
        }
    };
    (@layout StateRecorder, $layout:ident, log) => {
        field_layout::<Self, _>($layout, "log", |state| &state.log);
        crate::recording::log_format::serialized_layout($layout);
    };
    (@layout StateRecorder, $layout:ident, base_snapshot) => {
        field_layout::<Self, _>($layout, "base_snapshot", |state| &state.base_snapshot);
        $layout.push_str("base-snapshot{length0:reset;length-sram:");
        <[u8; crate::recording::BASE_SRAM_SIZE]>::layout($layout);
        $layout.push_str("length-greater-than-sram:");
        crate::snes::snes::snapshot_layout($layout);
        $layout.push('}');
    };
    (@layout StateRecorder, $layout:ident, base_audio_queue) => {
        field_layout::<Self, _>($layout, "base_audio_queue", |state| &state.base_audio_queue);
        ApuQueue::serialized_layout($layout);
    };
    (@layout $ty:ident, $layout:ident, $field:ident) => {
        field_layout::<Self, _>($layout, stringify!($field), |state| &state.$field);
    };
}

fields!(Apu { ram, registers } skip { spc, dsp });
fields!(ApuRegisters {
    romReadable, dspAdr, cycles, inPorts, outPorts, timer, cpuCyclesLeft, history
} skip { pad, snapshot_padding0 });
impl State for crate::apu_ram::ApuRam {
    fn layout(layout: &mut String) {
        <[u8; APU_RAM_BYTES]>::layout(layout);
    }
    fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String> {
        for offset in 0..APU_RAM_BYTES {
            let mut byte = self.read(offset as u16);
            byte.state(archive)?;
            if matches!(archive, Archive::Load(_)) {
                self.write(offset as u16, byte);
            }
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
} skip {  });

fields!(Dma {
    channel, hdmaTimer, dmaTimer, dmaBusy
} skip { pad, snapshot_padding0 });

fields!(DmaChannel {
    bAdr, aAdr, aBank, size, indBank, tableAdr, repCount, unusedByte, dmaActive,
    hdmaActive, mode, fixed, decrement, indirect, fromB, unusedBit, doTransfer,
    terminated, offIndex
} skip { snapshot_padding0, snapshot_padding1, snapshot_padding2, snapshot_padding3 });

fields!(DspState {
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
} skip {  });

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
} skip { cpu, apu, ppu, snes_ppu, my_ppu, cart, padx, padpad, snapshot_padding0, snapshot_padding1, snapshot_padding2 });

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
    fn layout(layout: &mut String) {
        layout.push_str("RecordingBytes{length:u64-le;bytes:length*u8;}");
    }
    fn state(&mut self, archive: &mut Archive<'_>) -> Result<(), String> {
        let mut length = (self.size as u64).to_le_bytes();
        archive.bytes(&mut length)?;
        let length = usize::try_from(u64::from_le_bytes(length))
            .map_err(|_| "checkpoint log length overflow")?;
        match archive {
            Archive::Validate(input, _, _) => {
                let (_, tail) = input
                    .split_at_checked(length)
                    .ok_or("truncated checkpoint log")?;
                *input = tail;
                return Ok(());
            }
            Archive::Load(input) => {
                if length > input.len() {
                    return Err("truncated checkpoint log".into());
                }
                unsafe {
                    crate::rtl::recording_allocation(self.resize(length));
                }
                self.as_mut_slice().fill(0);
            }
            Archive::Save(_) => {}
        }
        if self.size != 0 {
            archive.bytes(self.as_mut_slice())?;
        }
        Ok(())
    }
}

trait ByteArray: State {
    const LEN: usize;
}
impl<const N: usize> ByteArray for [u8; N] {
    const LEN: usize = N;
}

// One ordered declaration drives the machine bytes and their static field schema.
// Typed receivers make each schema's type the compiler-checked archive receiver.
macro_rules! machine_state {
    ($game:ident, $archive:ident, $snes:ident;
        game { $($field:ident : $check:ident),* $(,)? }
        devices { $($name:ident : $ty:ty = $value:expr => $kind:ident),* $(,)? }
    ) => {
        fn machine_layout(layout: &mut String) {
            layout.push_str("machine{work_ram:");
            <[u8; WorkRam::LEN]>::layout(layout);
            $(field_layout::<Game, _>(layout, stringify!($field), |game| &game.$field);)*
            $(layout.push_str(concat!(stringify!($name), ":")); <$ty>::layout(layout);)*
            layout.push('}');
        }
        unsafe fn machine($game: &mut Game, $archive: &mut Archive<'_>) -> Result<(), String> {
            unsafe {
                $archive.work_ram($game.ram())?;
                $(machine_state!(@check $archive, $field, $check); $game.$field.state($archive)?;)*
                let $snes = $game.g_snes;
                $(machine_state!(@device $game, $archive, $ty, $value, $kind);)*
                Ok(())
            }
        }
    };
    (@check $archive:ident, $field:ident, check) => { $archive.check_field(stringify!($field))?; };
    (@check $archive:ident, $field:ident, raw) => {};
    (@device $game:ident, $archive:ident, $ty:ty, $value:expr, state) => {
        let value: &mut $ty = &mut $value;
        value.state($archive)?;
    };
    (@device $game:ident, $archive:ident, $ty:ty, $value:expr, borrowed) => {
        let value: &mut $ty = $value;
        value.state($archive)?;
    };
    (@device $game:ident, $archive:ident, $ty:ty, $value:expr, dma) => {
        let mut dma: $ty = $value;
        let dma_result = dma.state($archive);
        if matches!($archive, Archive::Load(_)) { $game.set_dma_state(dma); }
        dma_result?;
    };
    (@device $game:ident, $archive:ident, $ty:ty, $value:expr, sram) => {
        let mut save_bytes: $ty = $value.bytes(0..<$ty as ByteArray>::LEN).try_into().unwrap();
        $archive.bytes(&mut save_bytes)?;
        if !matches!($archive, Archive::Validate(..)) { $game.g_sram.write_bytes(0, &save_bytes); }
    };
}

machine_state! { game, archive, snes;
    game {
        decompress_src: raw,
        eproj_spawn_rect: raw,
        eproj_spawn_r22: raw,
        eproj_spawn_pt: raw,
        eproj_spawn_varE24: raw,
        message_box_das0l_value: raw,
        k_out: raw,
        apu_queue: raw,
        g_runmode: check,
        g_calling_asm_from_c: check,
        g_calling_asm_from_c_ret: raw,
        g_fail: check,
        g_use_my_apu_code: check,
        hookmode: raw,
        hookcnt: raw,
        hookadr: raw,
        hook_orgbyte: raw,
        hook_fixbug_orgbyte: raw,
        kPatchedCarrysOrg: raw,
        VERIFY_ACTIVE: check,
        VERIFY_FAILURES: check,
        g_got_mismatch_count: raw,
        g_other_image: check,
        g_debug_flag: raw,
        g_new_ppu: raw,
        snes_frame_counter: check,
        pc_hist: raw,
        pc_hist_ctr: raw,
        pc_bp: raw,
        g_debug_apu_cycles: raw,
        state_recorder: raw
    }
    devices {
        snes: Snes = *snes => state,
        cpu: Cpu = *(*snes).cpu => state,
        dma: Dma = game.dma_state() => dma,
        input1: Input = game.input1_mut() => borrowed,
        input2: Input = game.input2_mut() => borrowed,
        snes_ppu: Ppu = *(*snes).snes_ppu => state,
        my_ppu: Ppu = *(*snes).my_ppu => state,
        apu: Apu = *(*snes).apu => state,
        spc: Spc = *(*(*snes).apu).spc => state,
        apu_dsp: DspState = (*(*(*snes).apu).dsp).state => state,
        player: NativePlayerState = (*game.g_spc_player).state => state,
        player_ram: crate::apu_ram::ApuRam = (*game.g_spc_player).ram => state,
        player_dsp: DspState = (*(*game.g_spc_player).state.dsp).state => state,
        sram: [u8; crate::recording::BASE_SRAM_SIZE] = game.g_sram => sram
    }
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
pub unsafe fn capture(game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe { capture_with_sources(game, &[]) }
}

pub(crate) unsafe fn capture_with_sources(
    game: &mut Game,
    sources: &[(sm_game::room_entry::world::ResourceId, crate::rom::RomImage)],
) -> Result<Vec<u8>, String> {
    unsafe {
        crate::cpu_infra::session_native_adoption::require_legacy_binary(game)?;
        let required_mode = if game.supports_legacy_persistence() {
            crate::cpu_infra::RM_BOTH
        } else {
            crate::cpu_infra::RM_MINE
        };
        if game.actor_identity_is_with_session() {
            return Err("checkpoint requires a Game-owned committed interval".into());
        }
        if game.g_snes.is_null() || game.g_spc_player.is_null() {
            return Err("full checkpoint requires the initialized native verifier backend".into());
        }
        if !game.VERIFY_ACTIVE
            || u32::from(game.g_runmode) != required_mode
            || (*game.g_snes).disableRender
            || game.g_other_image
            || game.VERIFY_FAILURES != 0
            || game.g_fail
            || game.g_calling_asm_from_c
            || (!game.supports_legacy_persistence() && game.snes_frame_counter <= 0)
            || game.state_recorder.replay_mode
            || !(*game.g_spc_player).state.reg_write_history.is_null()
            || (*(*game.g_snes).cart).ramSize != crate::recording::BASE_SRAM_SIZE as u32
        {
            return Err("checkpoint requires a successful, idle committed verifier frame".into());
        }

        let mut archive = Archive::Save(Vec::new());
        machine(game, &mut archive)?;
        let Archive::Save(machine) = archive else {
            unreachable!()
        };
        let actors = sm_game::actors::snapshot::ActorCheckpointPayload::capture(
            game.actor_identity(),
            game.world(),
        )?
        .encode(
            game.loaded_rom.as_ref(),
            game.capture_native_rom_binding().as_ref(),
            sources,
            &|at| machine[at],
        )?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(machine.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&machine);
        bytes.extend_from_slice(&actors);
        Ok(bytes)
    }
}

pub unsafe fn save(
    path: &Path,
    game: &mut Game,
    origin: u64,
    inputs: &[u16],
    completed: usize,
) -> Result<(), String> {
    unsafe { save_with_sources(path, game, origin, inputs, completed, &[]) }
}

pub unsafe fn save_with_sources(
    path: &Path,
    game: &mut Game,
    origin: u64,
    inputs: &[u16],
    completed: usize,
    sources: &[(sm_game::room_entry::world::ResourceId, crate::rom::RomImage)],
) -> Result<(), String> {
    unsafe {
        if completed > inputs.len() {
            return Err("checkpoint input frame is unavailable".into());
        }
        if completed == 0 && !game.supports_legacy_persistence() {
            return Err("prepared checkpoint requires completed input progress".into());
        }
        let payload = capture_with_sources(game, sources)?;
        let mut bytes = Vec::from(MAGIC.as_slice());
        for word in [
            compatibility(),
            origin,
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
}

pub unsafe fn load(
    path: &Path,
    game: &mut Game,
    origin: u64,
    inputs: &[u16],
) -> Result<usize, String> {
    unsafe { load_with_sources(path, game, origin, inputs, &[]) }
}

pub unsafe fn load_with_sources(
    path: &Path,
    game: &mut Game,
    origin: u64,
    inputs: &[u16],
    sources: &[(sm_game::room_entry::world::ResourceId, crate::rom::RomImage)],
) -> Result<usize, String> {
    unsafe {
        if game.actor_identity_is_with_session() {
            return Err("checkpoint requires a Game-owned committed interval".into());
        }
        if game.g_snes.is_null() || game.g_spc_player.is_null() {
            return Err("full checkpoint requires the initialized native verifier backend".into());
        }
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        if bytes.len() < 56 || &bytes[..8] != MAGIC {
            return Err("not a verifier checkpoint".into());
        }
        let word = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        let completed = usize::try_from(word(32)).map_err(|_| "checkpoint frame overflow")?;
        if completed == 0 && !game.supports_legacy_persistence() {
            return Err("prepared checkpoint requires completed input progress".into());
        }
        if word(8) != compatibility() {
            return Err("checkpoint runtime changed; regenerate from boot".into());
        }
        if word(16) != origin {
            return Err("checkpoint ROM or seed differs".into());
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
        restore_capture_with_sources(&bytes[56..], game, sources)?;
        Ok(completed)
    }
}

/// Restore an exact locally captured payload through the disk loader's same preflight.
/// File origin/source/input guards remain in load_with_sources.
pub(crate) unsafe fn restore_capture_with_sources(
    payload: &[u8],
    game: &mut Game,
    sources: &[(sm_game::room_entry::world::ResourceId, crate::rom::RomImage)],
) -> Result<(), String> {
    unsafe {
        crate::cpu_infra::session_native_adoption::require_legacy_binary(game)?;
        if game.actor_identity_is_with_session() {
            return Err("checkpoint requires a Game-owned committed interval".into());
        }
        if game.g_snes.is_null() || game.g_spc_player.is_null() {
            return Err("full checkpoint requires the initialized native verifier backend".into());
        }
        let size = payload
            .get(..8)
            .ok_or("truncated checkpoint machine length")?;
        let size = usize::try_from(u64::from_le_bytes(size.try_into().unwrap()))
            .map_err(|_| "checkpoint machine length overflow")?;
        let (machine_bytes, actor_bytes) = payload[8..]
            .split_at_checked(size)
            .ok_or("truncated checkpoint machine")?;
        let mode = if game.supports_legacy_persistence() {
            crate::cpu_infra::RM_BOTH
        } else {
            crate::cpu_infra::RM_MINE
        };
        let mut validate =
            Archive::Validate(machine_bytes, mode as u8, u8::from(game.g_use_my_apu_code));
        machine(game, &mut validate)?;
        if !matches!(validate, Archive::Validate([], _, _)) {
            return Err("checkpoint has extra machine bytes".into());
        }
        let ram: &[u8; 131072] = machine_bytes
            .get(..131072)
            .ok_or("truncated checkpoint RAM")?
            .try_into()
            .unwrap();
        // Compare the bytes the existing post-load patch will publish, without
        // changing live allocations before the complete machine/actor preflight.
        let loaded_after_patch = game.loaded_rom.as_ref().and_then(|loaded| {
            // Short images retain the original post-machine patch failure prefix.
            // $85:9650's14 bytes are the updater's highest addressed range.
            if loaded.len() < crate::rom::lorom_offset(0x859650) as usize + 14 {
                return None;
            }
            let projected = crate::rom::RomImage::zeroed(loaded.len());
            for at in 0..loaded.len() {
                projected.write(at, loaded.read(at));
            }
            crate::rom_patches::RomPatches::new(projected.clone())
                .update_bugfix(u16::from_le_bytes([ram[0xc2], ram[0xc3]]));
            Some(projected)
        });
        let restore = sm_game::actors::snapshot::ActorCheckpointPayload::prepare(
            actor_bytes,
            game.actor_identity(),
            game.world(),
            game.loaded_rom.as_ref(),
            game.capture_native_rom_binding().as_ref(),
            sources,
            &|at| ram[at],
            loaded_after_patch.as_ref(),
        )?;
        let failure = game
            .actor_identity()
            .prepare_canonical_transition()
            .map_err(|error| format!("{error:?}"))?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut archive = Archive::Load(machine_bytes);
            machine(game, &mut archive)?;
            crate::cpu_infra::RtlUpdateSnesPatchForBugfix(game);
            let mut roundtrip = Archive::Save(Vec::new());
            machine(game, &mut roundtrip)?;
            if !matches!(roundtrip, Archive::Save(ref bytes) if bytes == machine_bytes) {
                return Err::<(), String>("checkpoint state did not round-trip".into());
            }
            Ok(())
        }));
        game.actor_snapshots = Default::default();
        match result {
            Ok(Ok(())) => restore.commit(game.actor_identity_mut()),
            failed => {
                if game.supports_legacy_persistence() {
                    game.commit_actor_canonical(failure);
                } else {
                    game.actor_identity_mut().poison(failure);
                    game.g_fail = true;
                }
                match failed {
                    Ok(Err(error)) => return Err(error),
                    Err(panic) => std::panic::resume_unwind(panic),
                    Ok(Ok(())) => unreachable!(),
                }
            }
        }
        game.audio_output.new_epoch();
        Ok(())
    }
}

// Movies cannot supply a populated Prepared palette receipt with a corrupted catalog tag.
#[cfg(sm_verify_dll)]
pub(crate) unsafe fn check_prepared_palette_checkpoint(game: &mut Game) {
    use crate::game::room_entry::{
        content::{AssetRef, AssetSource},
        effects::{EntryEffects, InstallBuffer},
    };
    use std::sync::Arc;
    unsafe {
        let world = game.world().clone();
        let assets = world.prepared_room_assets().unwrap();
        EntryEffects::decompress(
            &mut crate::game::session_runtime::Runtime::new(game),
            AssetSource::Bound(AssetRef::Prepared(crate::room_world_fixture::PALETTE)),
            InstallBuffer::Palette,
        );
        crate::rtl::RtlRunFrame(0, game);
        crate::rtl::RtlRenderAudio(&mut [[0; 2]; 534], game);
        let first = crate::verification_checkpoint::capture(game).unwrap();
        assert!(
            first.ends_with(&assets.fingerprint()),
            "room.assets.populated-prepared-palette-tag"
        );
        let mut bad = first.clone();
        *bad.last_mut().unwrap() ^= 1;
        assert_eq!(
            crate::verification_checkpoint::restore_capture_with_sources(&bad, game, &[])
                .unwrap_err(),
            "prepared checkpoint palette catalog differs"
        );
        assert_eq!(
            crate::verification_checkpoint::capture(game).unwrap(),
            first,
            "room.assets.palette-preflight-no-adoption"
        );
        let mut audio = [[[0; 2]; 534]; 2];
        crate::rtl::RtlRunFrame(0, game);
        crate::rtl::RtlRenderAudio(&mut audio[0], game);
        let continued = crate::verification_checkpoint::capture(game).unwrap();
        crate::verification_checkpoint::restore_capture_with_sources(&first, game, &[]).unwrap();
        assert_eq!(
            crate::verification_checkpoint::capture(game).unwrap(),
            first
        );
        crate::rtl::RtlRunFrame(0, game);
        crate::rtl::RtlRenderAudio(&mut audio[1], game);
        assert_eq!(audio[0], audio[1]);
        assert_eq!(
            crate::verification_checkpoint::capture(game).unwrap(),
            continued
        );
        assert!(Arc::ptr_eq(game.world(), &world));
    }
}
