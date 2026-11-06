//! The main and interrupt strands from a cut: the wait site, the return chain on the stack,
//! and the interrupt command (D10, D21).
//!
//! The wait sites are the oracle's (`oracle::game::WAIT_SITES`); each instruction, push and
//! call below is quoted from the bank logs (<https://patrickjohnston.org/bank/>). JSR pushes
//! the address of its last byte, two bytes in the routine's bank; JSL pushes three, with the
//! bank. So a return address here is the instruction after the call, one past the pushed
//! value.

use game::state::slots::{Frames, SlotIndex};
use game::state::strand::{
    Frame, GameplayAt, IrqCommand, MAIN_STRAND_DEPTH, MainLoopAt, MainStrand, MessageBoxAt, Wait,
};

use crate::decode::DecodeError;
use crate::source::WramWord;

/// How a routine returns to its caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Return {
    /// RTS: a two-byte return address in the routine's own bank.
    Rts,
    /// RTL: a three-byte return address.
    Rtl,
    /// Nothing calls it: the bottom of the main thread.
    Bottom,
}

/// A loop in which the main thread waits for an NMI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaitSite {
    /// The routine, as the bank logs name it.
    pub routine: &'static str,
    /// The loop's instructions, as 24-bit addresses: an NMI interrupts one of them.
    pub loop_pcs: &'static [u32],
    /// What the main strand waits for there.
    pub wait: Wait,
    /// The frame the waiting routine is itself, when it is one.
    pub frame: Option<Frame>,
    /// The bytes the routine has pushed above its return address.
    pub pushed: u16,
    /// How it returns.
    pub returns: Return,
}

/// What a caller resumes as once the routine it called returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resume {
    /// This frame.
    Frame(Frame),
    /// The PLM pass, at the slot the PLM index variable holds.
    Plms,
    /// The message box growing open, its frames left from the animation radius.
    MessageBoxOpening,
    /// The message box shrinking closed, its frames left from the animation radius.
    MessageBoxClosing,
    /// No frame of its own: a step inside a frame further out or further in.
    Within,
}

/// A call whose return address can be on the main thread's stack at a wait.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallSite {
    /// The caller and the call, as the bank logs name them.
    pub routine: &'static str,
    /// The return address: the instruction after the call.
    pub returns_to: u32,
    /// What the caller resumes as.
    pub resume: Resume,
    /// The bytes the caller has pushed above its own return address at the call.
    pub pushed: u16,
    /// How the caller returns.
    pub returns: Return,
}

/// The original's wait sites: the oracle's three.
pub const WAIT_SITES: &[WaitSite] = &[
    // Bank $80 log, $80:8338 Wait for NMI: PHP, PHB, ..., $8343 LDA $05B4, $8346 BNE $8343,
    // ..., RTL.
    WaitSite {
        routine: "$80:8338 Wait for NMI",
        loop_pcs: &[0x80_8343, 0x80_8346],
        wait: Wait::Nmi,
        frame: None,
        pushed: 2,
        returns: Return::Rtl,
    },
    // Bank $85 log, $85:8136 Wait for lag frame: PHP, SEP #$20, LDA $05B8, $813C CMP $05B8,
    // $813F BEQ $813C, PLP, RTS.
    WaitSite {
        routine: "$85:8136 Wait for lag frame",
        loop_pcs: &[0x85_813C, 0x85_813F],
        wait: Wait::LagFrame,
        frame: None,
        pushed: 1,
        returns: Return::Rts,
    },
    // Bank $85 log, $85:846D Handle message box interaction, LOOP_INPUT $84A3-$84B2: spins on
    // $4212 and $4218/$4219 until a button, then RTS at $84B4; $846D-$8492 push nothing.
    WaitSite {
        routine: "$85:846D Handle message box interaction, LOOP_INPUT",
        loop_pcs: &[
            0x85_84A3, 0x85_84A6, 0x85_84A8, 0x85_84AA, 0x85_84AD, 0x85_84AF, 0x85_84B2,
        ],
        wait: Wait::LagFrame,
        frame: Some(Frame::MessageBox(MessageBoxAt::Shown)),
        pushed: 0,
        returns: Return::Rts,
    },
];

/// The calls on the return chains slice 0's rows wait in.
pub const CALL_SITES: &[CallSite] = &[
    // Bank $82 log, main game loop: LOOP $8948 PHP, ..., $896B JSR ($8981,x), $896E ...,
    // $897A JSL $808338, $897E PLP, BRA LOOP. Entered by a jump, so it is the bottom.
    CallSite {
        routine: "$82:897A main game loop, JSL Wait for NMI",
        returns_to: 0x82_897E,
        resume: Resume::Frame(Frame::MainLoop(MainLoopAt::Start)),
        pushed: 1,
        returns: Return::Bottom,
    },
    CallSite {
        routine: "$82:896B main game loop, JSR game state",
        returns_to: 0x82_896E,
        resume: Resume::Frame(Frame::MainLoop(MainLoopAt::AfterGameState)),
        pushed: 1,
        returns: Return::Bottom,
    },
    // Bank $82 log, game state 8 main gameplay: $8B44 PHP, ..., $8B71 JSL $8485B4 PLM handler,
    // $8B75 ..., $8BB8 RTS.
    CallSite {
        routine: "$82:8B71 main gameplay, JSL PLM handler",
        returns_to: 0x82_8B75,
        resume: Resume::Frame(Frame::Gameplay(GameplayAt::AfterPlms)),
        pushed: 1,
        returns: Return::Rts,
    },
    // Bank $84 log, PLM handler: $85B4 PHP, PHB, PHK, PLB, ..., $85C5 STX $1C27, ...,
    // $85CD JSR $85DA, $85D0 LDX $1C27, ..., RTL.
    CallSite {
        routine: "$84:85CD PLM handler, JSR process PLM",
        returns_to: 0x84_85D0,
        resume: Resume::Plms,
        pushed: 2,
        returns: Return::Rtl,
    },
    // Bank $84 log, process PLM: $85F7 PEA $85ED, $85FA JMP ($0012): the instruction's RTS
    // returns to $85EE, and process PLM has pushed nothing else.
    CallSite {
        routine: "$84:85F7 process PLM, PEA $85ED and JMP to the instruction",
        returns_to: 0x84_85EE,
        resume: Resume::Within,
        pushed: 0,
        returns: Return::Rts,
    },
    // Bank $84 log, pick up equipment [[Y]] and display message box: $88F3 pushes nothing
    // before $8912 JSL $858080. The other item instructions join with their rows.
    CallSite {
        routine: "$84:8912 pick up equipment instruction, JSL display message box",
        returns_to: 0x84_8916,
        resume: Resume::Within,
        pushed: 0,
        returns: Return::Rts,
    },
    // Bank $85 log, display message box: $8080 PHP, PHB, PHX, PHY (16-bit X and Y, from the
    // PLM handler's REP #$30): six bytes. $8099 JSR $844C open, $80A4 JSR $846D interaction,
    // $80A7 JSR $8589 close, ..., RTL.
    CallSite {
        routine: "$85:8099 display message box, JSR open",
        returns_to: 0x85_809C,
        resume: Resume::Within,
        pushed: 6,
        returns: Return::Rtl,
    },
    CallSite {
        routine: "$85:80A4 display message box, JSR interaction",
        returns_to: 0x85_80A7,
        resume: Resume::Within,
        pushed: 6,
        returns: Return::Rtl,
    },
    CallSite {
        routine: "$85:80A7 display message box, JSR close",
        returns_to: 0x85_80AA,
        resume: Resume::Within,
        pushed: 6,
        returns: Return::Rtl,
    },
    // Bank $85 log, open message box: $844C REP #$30, STZ $05A2, $8451 JSR $859B, ...,
    // $8469 JSR $859B, $846C RTS.
    CallSite {
        routine: "$85:8451 open message box, JSR write HDMA table",
        returns_to: 0x85_8454,
        resume: Resume::MessageBoxOpening,
        pushed: 0,
        returns: Return::Rts,
    },
    CallSite {
        routine: "$85:8469 open message box, last JSR write HDMA table",
        returns_to: 0x85_846C,
        resume: Resume::MessageBoxOpening,
        pushed: 0,
        returns: Return::Rts,
    },
    // Bank $85 log, close message box: $8589 REP #$30, $858B JSR $859B, ..., BPL, $859A RTS.
    CallSite {
        routine: "$85:858B close message box, JSR write HDMA table",
        returns_to: 0x85_858E,
        resume: Resume::MessageBoxClosing,
        pushed: 0,
        returns: Return::Rts,
    },
    // Bank $85 log, write message box BG3 Y scroll HDMA table: $859B PHP, $859C JSR $8136.
    CallSite {
        routine: "$85:859C write HDMA table, JSR wait for lag frame",
        returns_to: 0x85_859F,
        resume: Resume::Within,
        pushed: 1,
        returns: Return::Rts,
    },
];

/// Bank $85 log: open and close move the animation radius by $200 a frame, between 0 and
/// $1800.
const RADIUS_STEP: u16 = 0x0200;
const RADIUS_OPEN: u16 = 0x1800;

/// The sources and tables the strand decode reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrandRules<'t> {
    /// The wait sites.
    pub sites: &'t [WaitSite],
    /// The calls on the return chains.
    pub calls: &'t [CallSite],
    /// The PLM pass's slot, times two.
    pub plm_index: WramWord,
    /// The message box animation's radius.
    pub radius: WramWord,
}

/// The bank of a 24-bit address.
const fn bank(address: u32) -> u32 {
    address & 0xFF_0000
}

/// The stack byte at `address` in bank $00, which mirrors $7E:0000-$1FFF.
fn stack_byte(wram: &[u8], address: u16) -> Result<u8, DecodeError> {
    wram.get(usize::from(address))
        .copied()
        .ok_or(DecodeError::WramTooShort { source: "stack" })
}

impl StrandRules<'_> {
    /// The main strand waiting at `pc`, its return chain read from the stack at `s`.
    ///
    /// # Errors
    ///
    /// [`DecodeError::UnknownYieldSite`] when `pc`, or a return address on the chain, is in
    /// none of the tables, or the chain is deeper than `MAIN_STRAND_DEPTH`.
    pub fn main(&self, pc: u32, s: u16, wram: &[u8]) -> Result<MainStrand, DecodeError> {
        let site = self
            .sites
            .iter()
            .find(|site| site.loop_pcs.contains(&pc))
            .ok_or(DecodeError::UnknownYieldSite { pc })?;
        // Innermost first while walking; the strand keeps them innermost last.
        let mut frames: Vec<Frame> = site.frame.into_iter().collect();
        let mut cursor = s.wrapping_add(1).wrapping_add(site.pushed);
        let mut returns = site.returns;
        let mut routine_bank = bank(pc);
        let mut steps = 0_usize;
        loop {
            let returns_to = match returns {
                Return::Bottom => break,
                Return::Rts => {
                    let low = stack_byte(wram, cursor)?;
                    let high = stack_byte(wram, cursor.wrapping_add(1))?;
                    cursor = cursor.wrapping_add(2);
                    routine_bank | u32::from(u16::from_le_bytes([low, high]).wrapping_add(1))
                }
                Return::Rtl => {
                    let low = stack_byte(wram, cursor)?;
                    let high = stack_byte(wram, cursor.wrapping_add(1))?;
                    let pushed_bank = stack_byte(wram, cursor.wrapping_add(2))?;
                    cursor = cursor.wrapping_add(3);
                    (u32::from(pushed_bank) << 16)
                        | u32::from(u16::from_le_bytes([low, high]).wrapping_add(1))
                }
            };
            steps = steps.wrapping_add(1);
            let call = self
                .calls
                .iter()
                .find(|call| call.returns_to == returns_to)
                // A chain longer than the table repeats a call: no stack the original builds.
                .filter(|_| steps <= self.calls.len())
                .ok_or(DecodeError::UnknownYieldSite { pc: returns_to })?;
            if let Some(frame) = self.resume(call.resume, wram)? {
                frames.push(frame);
            }
            if frames.len() > MAIN_STRAND_DEPTH {
                return Err(DecodeError::UnknownYieldSite { pc: returns_to });
            }
            cursor = cursor.wrapping_add(call.pushed);
            returns = call.returns;
            routine_bank = bank(returns_to);
        }
        frames.reverse();
        Ok(MainStrand {
            frames,
            waiting: Some(site.wait),
        })
    }

    /// The frame a caller resumes as.
    fn resume(&self, resume: Resume, wram: &[u8]) -> Result<Option<Frame>, DecodeError> {
        match resume {
            Resume::Frame(frame) => Ok(Some(frame)),
            Resume::Within => Ok(None),
            Resume::Plms => {
                let index = self.plm_index.read(wram)?;
                let slot = u8::try_from(index / 2)
                    .ok()
                    .filter(|_| index % 2 == 0)
                    .ok_or(DecodeError::Unrepresentable {
                        source: self.plm_index.name,
                        value: index,
                    })?;
                Ok(Some(Frame::Plms {
                    slot: SlotIndex(slot, std::marker::PhantomData),
                }))
            }
            Resume::MessageBoxOpening => {
                let left = self.radius_steps(wram, |radius| RADIUS_OPEN.checked_sub(radius))?;
                Ok(Some(Frame::MessageBox(MessageBoxAt::Opening { left })))
            }
            Resume::MessageBoxClosing => {
                let left = self.radius_steps(wram, Some)?;
                Ok(Some(Frame::MessageBox(MessageBoxAt::Closing { left })))
            }
        }
    }

    /// The animation frames still to come: `distance` from the radius over the step.
    ///
    /// A radius off the step or past the open size is no radius the animation passes.
    fn radius_steps(
        &self,
        wram: &[u8],
        distance: impl Fn(u16) -> Option<u16>,
    ) -> Result<Frames, DecodeError> {
        let radius = self.radius.read(wram)?;
        distance(radius)
            .filter(|_| radius <= RADIUS_OPEN && radius % RADIUS_STEP == 0)
            .map(|distance| Frames(distance / RADIUS_STEP))
            .ok_or(DecodeError::Unrepresentable {
                source: self.radius.name,
                value: radius,
            })
    }
}

/// Bank $80 log, the interrupt commands ($80:9616): 0 nothing, 4 main gameplay begin HUD
/// drawing, 6 main gameplay end HUD drawing. The rest (2 disable, the door and Draygon
/// commands) aren't decoded yet.
#[must_use]
pub const fn irq_command(command: u16) -> Option<IrqCommand> {
    match command {
        0 => Some(IrqCommand::Off),
        4 => Some(IrqCommand::GameplayHud),
        6 => Some(IrqCommand::GameplayPlayfield),
        _ => None,
    }
}
