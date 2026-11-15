//! Camera measurement count from independently observed cut execution phases.
//!
//! The observer's fixed stack contract follows source-defined calls after ordinary following
//! to the main-loop wait. It reads instruction identities and return metadata, never camera
//! output. Unrecognised overruns are defects until their execution phase is established.

use crate::decode::DecodeError;
use crate::source::{WramByte, WramWord};
use oracle_record::{Cut, FrameRecord, NmiKind};

#[path = "execution_data.rs"]
mod data;

#[cfg(test)]
#[path = "door_data.rs"]
mod door_data;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Address(u32);

#[derive(Clone, Copy)]
struct StackBytes(u16);

#[derive(Clone, Copy)]
enum Return {
    Near,
    Far,
}

#[derive(Clone, Copy)]
struct Layout {
    saved: StackBytes,
    returning: Return,
}

struct Leaf {
    instructions: &'static [Address],
    layout: Layout,
}

/// Why the cut metadata cannot establish the camera's measurement count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// Source metadata is unreadable.
    Unreadable(DecodeError),
    /// The interrupted instruction has no established execution phase.
    Instruction(u32),
    /// A fixed source return chain does not name an established caller.
    Caller(u32),
    /// The established source chain did not reach the main-loop tail.
    ChainDepth,
    /// The next endpoint is not the documented requested main-loop wait.
    Endpoint,
    /// A dynamic callback lacks the required source write footprint.
    Capability(&'static str),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unreadable(error) => write!(f, "unreadable cut metadata: {error:?}"),
            Self::Instruction(pc) => write!(f, "unproven interrupted phase at {pc:06X}"),
            Self::Caller(pc) => write!(f, "unproven source caller at {pc:06X}"),
            Self::ChainDepth => {
                f.write_str("source return chain did not reach its established tail")
            }
            Self::Endpoint => f.write_str("cut does not end at the established main-loop wait"),
            Self::Capability(name) => write!(f, "unproven camera source capability: {name}"),
        }
    }
}

impl From<DecodeError> for Error {
    fn from(error: DecodeError) -> Self {
        Self::Unreadable(error)
    }
}

fn return_at(
    record: &FrameRecord,
    cursor: u16,
    bank: u32,
    returning: Return,
) -> Result<Address, Error> {
    let low = WramWord {
        name: "CAMERA_CUT_RETURN",
        offset: cursor,
    }
    .read(&record.wram)?
    .wrapping_add(1);
    let bank = match returning {
        Return::Near => bank,
        Return::Far => {
            u32::from(
                WramByte {
                    name: "CAMERA_CUT_RETURN_BANK",
                    offset: cursor.wrapping_add(2),
                }
                .read(&record.wram)?,
            ) << 16
        }
    };
    Ok(Address(bank | u32::from(low)))
}

pub(super) fn main_wait(record: &FrameRecord) -> Result<bool, Error> {
    let Cut::NmiReturn(nmi) = record.cut else {
        return Ok(false);
    };
    if nmi.kind != NmiKind::Requested || ![0x80_8343, 0x80_8346].contains(&nmi.interrupted_pc) {
        return Ok(false);
    }
    Ok(return_at(record, record.cpu.s.wrapping_add(3), 0, Return::Far)? == Address(0x82_897E))
}

fn expected_return(
    record: &FrameRecord,
    offset: u16,
    returning: Return,
    expected: &[Address],
) -> Result<(), Error> {
    let actual = return_at(
        record,
        record.cpu.s.wrapping_add(offset),
        0x85_0000,
        returning,
    )?;
    if expected.contains(&actual) {
        Ok(())
    } else {
        Err(Error::Caller(actual.0))
    }
}

/// The final message-box wait resumes the selected gameplay's pre-follow PLM tail.
fn nested_before_follow(record: &FrameRecord) -> Result<bool, Error> {
    let Cut::NmiReturn(nmi) = record.cut else {
        return Ok(false);
    };
    if nmi.kind != NmiKind::Requested || ![0x85_813C, 0x85_813F].contains(&nmi.interrupted_pc) {
        return Ok(false);
    }
    if record.cpu.x != 1 {
        return Err(Error::Capability("message-box final lag iteration"));
    }
    expected_return(record, 2, Return::Near, &[Address(0x85_857B)])?;
    expected_return(record, 4, Return::Near, &[Address(0x85_80B7)])?;
    expected_return(
        record,
        12,
        Return::Far,
        &[Address(0x84_8916), Address(0x84_89CF)],
    )?;
    for (offset, expected) in [(15, 0x84_85EE), (17, 0x84_85D0), (25, 0x82_896E)] {
        let actual = return_at(
            record,
            record.cpu.s.wrapping_add(offset),
            expected & 0xFF_0000,
            Return::Near,
        )?;
        if actual != Address(expected) {
            return Err(Error::Caller(actual.0));
        }
    }
    expected_return(record, 21, Return::Far, &[Address(0x82_8B75)])?;
    Ok(true)
}

/// These callbacks cannot write physical centre or the selector after Remembering.
pub(super) fn centre_tail(from: &FrameRecord, next: &FrameRecord) -> Result<(), Error> {
    let room = WramWord {
        name: "ROOM_MAIN_ASM",
        offset: 0x07DF,
    };
    let before = room.read(&from.wram)?;
    let after = room.read(&next.wram)?;
    if before != after || ![0, 0xC116, 0xE51F, 0xE524, 0xE525, 0xE571].contains(&after) {
        return Err(Error::Capability("room main ASM"));
    }
    for record in [from, next] {
        if (WramWord {
            name: "SCROLLING_FINISHED_HOOK",
            offset: 0x07E9,
        })
        .read(&record.wram)?
            != 0
        {
            return Err(Error::Capability("scrolling finished hook"));
        }
    }
    let graphics = u32::from(
        (WramWord {
            name: "ENEMY_GRAPHICS_DRAWN_HOOK",
            offset: 0x178C,
        })
        .read(&next.wram)?,
    ) | (u32::from(
        (WramByte {
            name: "ENEMY_GRAPHICS_DRAWN_HOOK_BANK",
            offset: 0x178E,
        })
        .read(&next.wram)?,
    ) << 16);
    if ![0xA0_804C, 0xA6_A2F2, 0xA8_804C, 0xA8_E86E].contains(&graphics) {
        return Err(Error::Capability("enemy graphics drawn hook"));
    }
    Ok(())
}

/// Validate a cut inside the acquired no-measurement door call closure.
#[cfg(test)]
pub(super) fn door_endpoint(record: &FrameRecord) -> Result<(), Error> {
    if main_wait(record)? {
        return Ok(());
    }
    let Cut::NmiReturn(nmi) = record.cut else {
        return Err(Error::Endpoint);
    };
    let pc = Address(nmi.interrupted_pc);
    let mut layout = match nmi.kind {
        NmiKind::Requested if pc == Address(0x80_8346) => {
            expected_return(record, 3, Return::Far, &[Address(0x82_E544)])?;
            for (offset, expected) in [(6, 0x82_E292), (10, 0x82_896E)] {
                let actual = return_at(
                    record,
                    record.cpu.s.wrapping_add(offset),
                    0x82_0000,
                    Return::Near,
                )?;
                if actual != Address(expected) {
                    return Err(Error::Caller(actual.0));
                }
            }
            return Ok(());
        }
        NmiKind::Requested => return Err(Error::Instruction(pc.0)),
        NmiKind::Overrun => {
            door_data::LEAVES
                .iter()
                .find(|leaf| leaf.instructions.contains(&pc))
                .ok_or(Error::Instruction(pc.0))?
                .layout
        }
    };
    let mut cursor = record.cpu.s.wrapping_add(1);
    let mut bank = pc.0 & 0xFF_0000;
    let mut reached_wrapper = false;
    for _ in 0..12 {
        cursor = cursor.wrapping_add(layout.saved.0);
        let returned = return_at(record, cursor, bank, layout.returning)?;
        cursor = cursor.wrapping_add(match layout.returning {
            Return::Near => 2,
            Return::Far => 3,
        });
        bank = returned.0 & 0xFF_0000;
        if returned == Address(0x82_E292) {
            reached_wrapper = true;
        }
        if returned == Address(0x82_896E) {
            return if reached_wrapper {
                Ok(())
            } else {
                Err(Error::Endpoint)
            };
        }
        layout = door_data::CALLERS
            .iter()
            .find(|(caller, _)| *caller == returned)
            .ok_or(Error::Caller(returned.0))?
            .1;
    }
    Err(Error::ChainDepth)
}

/// Source timing of the ordinary measurement and remembered-centre write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    BeforeMeasurement,
    AfterMeasurementBeforeRemembering,
    AfterRemembering,
}

/// Establish the interval from an independently decoded execution cut to the main-loop wait.
/// A marker has meaning only after every caller and the complete tail have been validated.
///
/// # Errors
/// An interrupted instruction, return chain or endpoint outside the observer's source contract.
pub(super) fn phase_between(from: &FrameRecord, next: &FrameRecord) -> Result<Phase, Error> {
    let Cut::NmiReturn(nmi) = from.cut else {
        return Err(Error::Endpoint);
    };
    match nmi.kind {
        NmiKind::Requested => {
            return if main_wait(next)? && (main_wait(from)? || nested_before_follow(from)?) {
                Ok(Phase::BeforeMeasurement)
            } else {
                Err(Error::Endpoint)
            };
        }
        NmiKind::Overrun => {}
    }
    if !main_wait(next)? {
        return Err(Error::Endpoint);
    }
    let pc = Address(nmi.interrupted_pc);
    if [Address(0x82_8972), Address(0x82_897A)].contains(&pc) {
        return Ok(Phase::AfterRemembering);
    }
    let mut layout = data::LEAVES
        .iter()
        .find(|leaf| leaf.instructions.contains(&pc))
        .ok_or(Error::Instruction(pc.0))?
        .layout;
    let mut cursor = from.cpu.s.wrapping_add(1);
    let mut bank = pc.0 & 0xFF_0000;
    let mut phase = Phase::AfterRemembering;
    // Sixteen bounds the finite, source-defined return grammar, not a stack search.
    for _ in 0..16 {
        cursor = cursor.wrapping_add(layout.saved.0);
        let returned = return_at(from, cursor, bank, layout.returning)?;
        cursor = cursor.wrapping_add(match layout.returning {
            Return::Near => 2,
            Return::Far => 3,
        });
        bank = returned.0 & 0xFF_0000;
        if [
            Address(0x82_8B5C),
            Address(0x82_8B69),
            Address(0x82_8B6D),
            Address(0x82_8B82),
            Address(0x82_8B8A),
        ]
        .contains(&returned)
        {
            phase = Phase::BeforeMeasurement;
        }
        if returned == Address(0x90_96A5) {
            phase = Phase::AfterMeasurementBeforeRemembering;
        }
        if [
            Address(0x82_896E),
            Address(0x82_8972),
            Address(0x82_8976),
            Address(0x82_897E),
        ]
        .contains(&returned)
        {
            return Ok(phase);
        }
        layout = data::CALLERS
            .iter()
            .find(|(caller, _)| *caller == returned)
            .ok_or(Error::Caller(returned.0))?
            .1;
    }
    Err(Error::ChainDepth)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cut(pc: u32, kind: NmiKind) -> FrameRecord {
        let mut record = crate::harness::tests::record(0, 8, [0; 4]);
        record.cpu.s = 0x1F00;
        let Cut::NmiReturn(nmi) = &mut record.cut else {
            panic!("test record is an NMI cut")
        };
        nmi.kind = kind;
        nmi.interrupted_pc = pc;
        record
    }

    fn stack(record: &mut FrameRecord, offset: u16, bytes: &[u8]) {
        let start = usize::from(record.cpu.s.wrapping_add(offset));
        record.wram[start..start.checked_add(bytes.len()).unwrap()].copy_from_slice(bytes);
    }

    fn post_follow_pair() -> (FrameRecord, FrameRecord) {
        let mut from = cut(0xA0_8853, NmiKind::Overrun);
        // Enemy draw's PHB, its far return to the post-follow gameplay tail,
        // then gameplay's PHP and its near return to the main loop.
        stack(&mut from, 2, &[0x9B, 0x8B, 0x82]);
        stack(&mut from, 6, &[0x6D, 0x89]);
        let mut next = cut(0x80_8343, NmiKind::Requested);
        stack(&mut next, 3, &[0x7D, 0x89, 0x82]);
        (from, next)
    }

    #[test]
    fn both_fixed_return_chains_are_required_for_retention() {
        let (mut from, mut next) = post_follow_pair();
        assert_eq!(phase_between(&from, &next), Ok(Phase::AfterRemembering));
        stack(&mut next, 3, &[0x7D, 0x89, 0x90]);
        assert_eq!(phase_between(&from, &next), Err(Error::Endpoint));
        stack(&mut next, 3, &[0x7D, 0x89, 0x82]);
        stack(&mut from, 6, &[0x00, 0x80]);
        assert!(matches!(phase_between(&from, &next), Err(Error::Caller(_))));
    }

    #[test]
    fn the_nested_message_wait_requires_its_complete_final_iteration_chain() {
        let (_, next) = post_follow_pair();
        let mut from = cut(0x85_813C, NmiKind::Requested);
        from.cpu.x = 1;
        for (offset, bytes) in [
            (2, &[0x7A, 0x85][..]),
            (4, &[0xB6, 0x80][..]),
            (12, &[0x15, 0x89, 0x84][..]),
            (15, &[0xED, 0x85][..]),
            (17, &[0xCF, 0x85][..]),
            (21, &[0x74, 0x8B, 0x82][..]),
            (25, &[0x6D, 0x89][..]),
        ] {
            stack(&mut from, offset, bytes);
        }
        assert_eq!(phase_between(&from, &next), Ok(Phase::BeforeMeasurement));
        from.cpu.x = 2;
        assert!(matches!(
            phase_between(&from, &next),
            Err(Error::Capability(_))
        ));
        from.cpu.x = 1;
        stack(&mut from, 25, &[0x00, 0x80]);
        assert!(matches!(phase_between(&from, &next), Err(Error::Caller(_))));
    }

    #[test]
    fn next_centre_requires_the_callback_write_closure() {
        let from = cut(0x80_8343, NmiKind::Requested);
        let mut next = from.clone();
        next.wram[0x178C..0x178F].copy_from_slice(&[0x4C, 0x80, 0xA0]);
        assert_eq!(centre_tail(&from, &next), Ok(()));
        next.wram[0x07E9] = 1;
        assert!(matches!(
            centre_tail(&from, &next),
            Err(Error::Capability(_))
        ));
        next.wram[0x07E9] = 0;
        next.wram[0x178C] = 0x4D;
        assert!(matches!(
            centre_tail(&from, &next),
            Err(Error::Capability(_))
        ));
    }

    #[test]
    fn the_nested_door_wait_requires_the_wrapper_bottom() {
        let mut record = cut(0x80_8346, NmiKind::Requested);
        stack(&mut record, 3, &[0x43, 0xE5, 0x82]);
        stack(&mut record, 6, &[0x91, 0xE2]);
        stack(&mut record, 10, &[0x6D, 0x89]);
        assert_eq!(door_endpoint(&record), Ok(()));
        stack(&mut record, 10, &[0x00, 0x80]);
        assert!(matches!(door_endpoint(&record), Err(Error::Caller(_))));
    }

    #[test]
    fn requested_kind_requires_both_main_wait_endpoints() {
        let (_, next) = post_follow_pair();
        let mut from = next.clone();
        assert_eq!(phase_between(&from, &next), Ok(Phase::BeforeMeasurement));
        stack(&mut from, 3, &[0x00, 0x80, 0x85]);
        assert_eq!(phase_between(&from, &next), Err(Error::Endpoint));
    }

    #[test]
    fn an_operand_address_is_not_an_execution_phase() {
        let (mut from, next) = post_follow_pair();
        let Cut::NmiReturn(nmi) = &mut from.cut else {
            panic!("test record is an NMI cut")
        };
        nmi.interrupted_pc = 0xA0_8854;
        assert!(matches!(
            phase_between(&from, &next),
            Err(Error::Instruction(_))
        ));
    }
}
