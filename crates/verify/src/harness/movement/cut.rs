//! Source-defined decompressor cuts inside the enemy pass, before Samus POST.
//!
//! At $80:B1DD/B1DF PHP and PHB are saved. Kraid graphics returns through the mouth
//! collision, main AI, enemy pass and gameplay. The $82:8B69 continuation precedes
//! Samus POST; validating the complete six-return grammar establishes its absence.

use crate::decode::DecodeError;
use crate::strand::{StackReturn, stack_return};
use oracle_record::{Cut, FrameRecord, NmiKind};

struct Caller {
    saved: u16,
    width: StackReturn,
    resumes: &'static [u32],
}

const BEFORE_POST: &[Caller] = &[
    Caller {
        saved: 2,
        width: StackReturn::Far,
        resumes: &[0xA7_AAEA, 0xA7_AB0C],
    },
    Caller {
        saved: 0,
        width: StackReturn::Near,
        resumes: &[0xA7_B0A4],
    },
    Caller {
        saved: 0,
        width: StackReturn::Near,
        resumes: &[0xA7_AC24],
    },
    Caller {
        saved: 0,
        width: StackReturn::Far,
        resumes: &[0xA0_9093],
    },
    Caller {
        saved: 2,
        width: StackReturn::Far,
        resumes: &[0x82_8B69],
    },
    Caller {
        saved: 1,
        width: StackReturn::Near,
        resumes: &[0x82_896E],
    },
];

/// Whether this exact cut proves that PRE ran but POST has not started.
pub(super) fn before_post(record: &FrameRecord) -> Result<bool, DecodeError> {
    let Cut::NmiReturn(nmi) = record.cut else {
        return Ok(false);
    };
    if nmi.kind != NmiKind::Overrun || ![0x80_B1DD, 0x80_B1DF].contains(&nmi.interrupted_pc) {
        return Ok(false);
    }
    let mut cursor = record.cpu.s.wrapping_add(1);
    let mut bank = nmi.interrupted_pc & 0xFF_0000;
    for caller in BEFORE_POST {
        cursor = cursor.wrapping_add(caller.saved);
        let returned = stack_return(&record.wram, cursor, bank, caller.width)?;
        if !caller.resumes.contains(&returned) {
            return Ok(false);
        }
        cursor = cursor.wrapping_add(caller.width.bytes());
        bank = returned & 0xFF_0000;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::before_post;
    use oracle_record::{Cut, FrameRecord, NmiKind};

    fn decompression_cut(pc: u32, graphics_return: u32, enemy_return: u32) -> FrameRecord {
        let mut record = super::super::tests::record(1, &[(crate::source::GAME_STATE, 8)]);
        record.cpu.s = 0x01C0;
        let Cut::NmiReturn(ref mut nmi) = record.cut else {
            panic!("NMI fixture");
        };
        nmi.kind = NmiKind::Overrun;
        nmi.interrupted_pc = pc;
        let mut cursor = usize::from(record.cpu.s) + 1;
        // Independent source fixture: saved PHP/PHB, graphics/mouth/AI/enemy/gameplay calls.
        for (saved, bytes, returned) in [
            (2, 3, graphics_return),
            (0, 2, 0xA7_B0A4),
            (0, 2, 0xA7_AC24),
            (0, 3, 0xA0_9093),
            (2, 3, enemy_return),
            (1, 2, 0x82_896E),
        ] {
            cursor = cursor
                .checked_add(saved)
                .expect("saved registers fit the fixture stack");
            let pushed = returned.wrapping_sub(1).to_le_bytes();
            let end = cursor
                .checked_add(bytes)
                .expect("return fits the fixture stack");
            record
                .wram
                .get_mut(cursor..end)
                .expect("fixture stack is in work RAM")
                .copy_from_slice(
                    pushed
                        .get(..bytes)
                        .expect("return width fits the pushed address"),
                );
            cursor = end;
        }
        record
    }

    #[test]
    fn both_fixed_instruction_leaves_and_graphics_callers_prove_pre_post() {
        for pc in [0x80_B1DD, 0x80_B1DF] {
            for graphics in [0xA7_AAEA, 0xA7_AB0C] {
                assert_eq!(
                    before_post(&decompression_cut(pc, graphics, 0x82_8B69)),
                    Ok(true)
                );
            }
        }
    }

    #[test]
    fn operands_wrong_banks_and_post_post_callers_cannot_mask_publication() {
        for (pc, graphics, enemy) in [
            (0x80_B1DE, 0xA7_AAEA, 0x82_8B69),
            (0x81_B1DF, 0xA7_AAEA, 0x82_8B69),
            (0x80_B1DF, 0xA6_AAEA, 0x82_8B69),
            (0x80_B1DF, 0xA7_AAEA, 0x83_8B69),
            (0x80_B1DF, 0xA7_AAEA, 0x82_8B6D),
        ] {
            assert_eq!(
                before_post(&decompression_cut(pc, graphics, enemy)),
                Ok(false)
            );
        }
    }

    #[test]
    fn a_missing_caller_or_unreadable_chain_never_proves_pre_post() {
        let mut record = decompression_cut(0x80_B1DF, 0xA7_AAEA, 0x82_8B69);
        let mouth_return = usize::from(record.cpu.s) + 1 + 2 + 3;
        record.wram[mouth_return..mouth_return + 2].fill(0);
        assert_eq!(before_post(&record), Ok(false));
        record.wram = Box::default();
        assert!(before_post(&record).is_err());
    }

    #[test]
    fn a_validated_pre_post_cut_defers_pending_without_hiding_motion() {
        use super::super::{
            Quantity,
            tests::{masked, record},
        };
        let from = record(0, &[]);
        let before = decompression_cut(0x80_B1DF, 0xA7_AAEA, 0x82_8B69);
        assert_eq!(
            masked(&from, &before),
            Quantity::Placed.set().union(Quantity::PendingAim.set())
        );
        let after = decompression_cut(0x80_B1DF, 0xA7_AAEA, 0x82_8B6D);
        assert_eq!(masked(&from, &after), Quantity::Placed.set());
    }

    #[test]
    fn a_completed_cut_does_not_claim_an_unexecuted_post() {
        let mut record = decompression_cut(0x80_B1DF, 0xA7_AAEA, 0x82_8B69);
        let Cut::NmiReturn(ref mut nmi) = record.cut else {
            panic!("NMI fixture");
        };
        nmi.kind = NmiKind::Requested;
        assert_eq!(before_post(&record), Ok(false));
    }
}
