//! The NMI (D6 step 4, D8): the batch commit and the pad latch when the main strand requested
//! it and the NMI arrived on time, the lag bookkeeping otherwise, and the counters.
//!
//! Bank `$80` log, NMI `$80:9583`: if the request flag is set, update OAM and CGRAM (`$933A`),
//! transfer Samus's tiles (`$9376`), the animated tiles' VRAM transfers (`$9416`), update the IO
//! registers (`$91EE`), the HDMA queue, the mode 7 transfers when BG mode 7 is on (`$8BBA`), the
//! VRAM write table (`$8C83`) and read table (`$8EA2`), enable HDMA, read the controller
//! (`$9459`), then clear the request and the lag counter and increment the 8- and 16-bit frame
//! counters. Otherwise increment the 8-bit lag counter and raise the maximum lag to it. Either
//! way, increment the NMI counter.

use crate::pose::Content;
use crate::session::Rules;
use crate::state::slots::Frames;
use crate::state::{Buttons, Controller, Counters, FrameCount, NmiTiming, State};

/// One step of the NMI's commit, in the original's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NmiCommand {
    /// OAM and CGRAM DMA.
    OamAndCgram,
    /// Samus's tiles to VRAM.
    SamusTiles,
    /// The animated tiles' VRAM transfers.
    AnimatedTiles,
    /// The PPU register mirrors.
    IoRegisters,
    /// The HDMA channel bindings.
    HdmaQueue,
    /// The mode 7 transfers, when BG mode 7 is on.
    Mode7,
    /// The VRAM write table and the scrolling DMAs.
    VramWrites,
    /// The VRAM read table into its targets.
    VramReads,
    /// Enable the HDMA channels.
    HdmaEnable,
    /// Latch the pad into the controller words.
    ReadController,
}

/// The commit's steps, in order.
pub const COMMIT: [NmiCommand; 10] = [
    NmiCommand::OamAndCgram,
    NmiCommand::SamusTiles,
    NmiCommand::AnimatedTiles,
    NmiCommand::IoRegisters,
    NmiCommand::HdmaQueue,
    NmiCommand::Mode7,
    NmiCommand::VramWrites,
    NmiCommand::VramReads,
    NmiCommand::HdmaEnable,
    NmiCommand::ReadController,
];

/// Which path a vblank's NMI took.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NmiKind {
    /// The main strand had requested it: the batch committed and the pad latched.
    Committed,
    /// A lag NMI: counters only.
    Lag,
    /// NMIs were disabled: nothing ran.
    Disabled,
}

/// The NMI at the end of a Tick, arriving as `timing` says with the pad as auto-read.
pub fn nmi(
    state: &mut State,
    rules: Rules,
    content: &Content,
    timing: NmiTiming,
    pad: Buttons,
) -> NmiKind {
    state.timing.last = timing;
    // The engine serialises the strands, so a strand that overran has already set its
    // request by the time its NMI runs; on the console that NMI found the flag clear.
    let kind = match timing {
        NmiTiming::Disabled => return NmiKind::Disabled,
        NmiTiming::OnTime if state.video.nmi_requested => {
            for command in COMMIT {
                run(command, state, rules, content, pad);
            }
            state.video.nmi_requested = false;
            commit_counters(&mut state.clock.counters);
            NmiKind::Committed
        }
        NmiTiming::OnTime | NmiTiming::Overrun => {
            lag_counters(&mut state.clock.counters);
            NmiKind::Lag
        }
    };
    state.clock.counters.nmi = FrameCount(state.clock.counters.nmi.0.wrapping_add(1));
    // The console auto-reads the pad every NMI; direct reads in the next Tick see it.
    state.controller.auto_read = pad;
    kind
}

/// One commit step.
fn run(command: NmiCommand, state: &mut State, rules: Rules, content: &Content, pad: Buttons) {
    match command {
        NmiCommand::OamAndCgram => state.video.upload_oam_and_cgram(),
        NmiCommand::SamusTiles => state
            .video
            .transfer_samus_tiles(&content.samus_tiles, &content.graphics),
        NmiCommand::IoRegisters => state.video.commit_registers(),
        NmiCommand::VramWrites => state.video.process_vram_writes(&content.graphics),
        // open: the animated tiles join with their content (B3, C lanes); mode 7 and the
        // read table have no slice 0 user; HDMA is B6's.
        NmiCommand::AnimatedTiles
        | NmiCommand::HdmaQueue
        | NmiCommand::Mode7
        | NmiCommand::VramReads
        | NmiCommand::HdmaEnable => {}
        NmiCommand::ReadController => latch(&mut state.controller, rules, pad),
    }
}

/// The requested NMI's counters: the lag run ends and the frame counter counts.
const fn commit_counters(counters: &mut Counters) {
    counters.lag = FrameCount(0);
    counters.completed = FrameCount(counters.completed.0.wrapping_add(1));
    counters.requested_byte = counters.requested_byte.wrapping_add(1);
}

/// A lag NMI's counters. The lag counter and its maximum are bytes in the original, so the
/// run wraps at 256 and the maximum keeps its old value when it does.
fn lag_counters(counters: &mut Counters) {
    let [run, _] = counters.lag.0.to_le_bytes();
    let lag = FrameCount(u16::from(run.wrapping_add(1)));
    counters.lag = lag;
    if lag.0 >= counters.max_lag.0 {
        counters.max_lag = lag;
    }
}

/// Bank `$80` log, read controller input `$80:9459`: the pad becomes the held word, the newly
/// pressed word is what wasn't held last time, and the auto-repeat word starts as the newly
/// pressed one. While the same non-empty pad stays held the repeat timer counts down and, at
/// zero, repeats the held buttons and reloads with the subsequent delay; any other pad reloads
/// it with the initial delay. Then the held word becomes the previous one.
fn latch(controller: &mut Controller, rules: Rules, pad: Buttons) {
    controller.held = pad;
    controller.pressed = pad.difference(&controller.previous);
    controller.repeat = controller.pressed;
    if !pad.is_empty() && pad == controller.previous {
        controller.repeat_delay = Frames(controller.repeat_delay.0.wrapping_sub(1));
        if controller.repeat_delay == Frames(0) {
            controller.repeat = pad;
            controller.repeat_delay = rules.auto_repeat.subsequent;
        }
    } else {
        controller.repeat_delay = rules.auto_repeat.initial;
    }
    controller.previous = pad;
}
