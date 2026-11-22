//! One-way import of the qualified completed-return hardware origin.
//!
//! Context integrity and origin provenance belong to admission. This edge imports
//! no content, expected observations, future snapshots or semantic work profiles.

use data::timing::{
    ChannelConfig, InterruptEnable, IrqMode, IrqPolicy, MasterTicks, TransferPattern, TransferPort,
};
use game::production::timing::{
    Beam, BeamPosition, Channel, Controller, Hardware, InterruptLine, Interrupts, LineStatus,
    LineTransfer, Math, PhysicalLine, PhysicalPad, State, Thresholds, Transfers,
};

/// Invalid raw ABI or an origin outside the qualified mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedFault {
    /// Context is not the complete native ABI.
    Length,
    /// A native boolean is not encoded as zero or one.
    Boolean,
    /// A numeric field cannot be represented by its typed owner.
    Range,
    /// Source mechanism or origin continuation is not qualified.
    Unsupported,
    /// Complete production state validation failed.
    InvalidState,
}

struct Context<'a>(&'a [u8]);
impl Context<'_> {
    fn byte(&self, at: usize) -> u8 {
        self.0.get(at).copied().unwrap_or_default()
    }
    fn flag(&self, at: usize) -> Result<bool, SeedFault> {
        match self.byte(at) {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SeedFault::Boolean),
        }
    }
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([
            self.0.get(at).copied().unwrap_or_default(),
            self.byte(at.wrapping_add(1)),
        ])
    }
    fn integer(&self, at: usize) -> u32 {
        u32::from_le_bytes([
            self.0.get(at).copied().unwrap_or_default(),
            self.byte(at.wrapping_add(1)),
            self.byte(at.wrapping_add(2)),
            self.byte(at.wrapping_add(3)),
        ])
    }
    fn narrow(&self, at: usize) -> Result<u16, SeedFault> {
        u16::try_from(self.integer(at)).map_err(|_| SeedFault::Range)
    }
    fn line(&self, at: usize) -> Result<InterruptLine, SeedFault> {
        Ok(InterruptLine {
            physical: PhysicalLine {
                valid: self.flag(at)?,
                line: self.flag(at.wrapping_add(1))?,
                hold: self.flag(at.wrapping_add(4))?,
            },
            transition: self.flag(at.wrapping_add(2))?,
            pending: self.flag(at.wrapping_add(3))?,
        })
    }
    fn controller(&self, current: PhysicalPad) -> Result<Controller, SeedFault> {
        Ok(Controller {
            accumulator: u64::from(self.integer(402)),
            iteration: self.integer(398),
            latched: self.flag(394)?,
            active: self.flag(393)?,
            serial: 0,
            words: [
                self.word(454),
                self.word(456),
                self.word(458),
                self.word(460),
            ],
            committed_pad: current,
            pending_pad: Vec::new(),
        })
    }
    fn math(&self) -> Math {
        Math {
            dividend: self.word(434),
            multiplicand: self.byte(431),
            multiplier: self.byte(432),
            product: self.word(452),
            operand: self.word(450),
            shift: self.integer(470),
            remaining: 0,
        }
    }
    fn interrupts(&self, policy: IrqPolicy) -> Result<Interrupts, SeedFault> {
        let irq = match (self.flag(427)?, self.flag(428)?) {
            (false, false) => IrqMode::Disabled,
            (true, false) => IrqMode::Horizontal,
            (false, true) => IrqMode::Vertical,
            (true, true) => IrqMode::Both,
        };
        Ok(Interrupts {
            enable: InterruptEnable {
                nmi: self.flag(426)?,
                irq,
                controller: self.flag(429)?,
            },
            horizontal_target: self.narrow(438)?,
            vertical_target: self.narrow(442)?,
            nmi: self.line(367)?,
            irq: self.line(372)?,
            locked: self.flag(342)?,
            policy,
        })
    }
    fn channel(&self, slot: usize) -> Result<Channel, SeedFault> {
        let at = 90_usize.wrapping_add(28_usize.wrapping_mul(slot));
        if self.flag(at.wrapping_add(2))? {
            return Err(SeedFault::Unsupported);
        }
        let pattern = match self.integer(at.wrapping_add(8)) {
            0 => TransferPattern::Single,
            1 => TransferPattern::Pair,
            2 => TransferPattern::Double,
            3 => TransferPattern::DoublePair,
            4 => TransferPattern::Quad,
            5 => TransferPattern::PairPair,
            6 => TransferPattern::DoubleAgain,
            7 => TransferPattern::DoublePairAgain,
            _ => return Err(SeedFault::Range),
        };
        Ok(Channel {
            config: ChannelConfig {
                source_bank: self.byte(at.wrapping_add(16)),
                indirect_bank: self.byte(at.wrapping_add(20)),
                fixed: self.flag(at.wrapping_add(6))?,
                reverse: self.flag(at.wrapping_add(5))?,
                indirect: self.flag(at.wrapping_add(3))?,
                target: TransferPort(self.byte(at.wrapping_add(12))),
                pattern,
            },
            source_offset: self.word(at.wrapping_add(14)),
            pointer: self.word(at.wrapping_add(18)),
            table: self.word(at.wrapping_add(22)),
            lines: self.byte(at.wrapping_add(24)),
            line: LineStatus {
                enabled: self.flag(at.wrapping_add(1))?,
                completed: self.flag(at.wrapping_add(26))?,
                payload: self.flag(at.wrapping_add(27))?,
            },
            bulk_enabled: self.flag(at)?,
        })
    }
}

/// Decode independent hexadecimal current/preview physical action rows.
///
/// Row identity and commitment timing must be independently admitted by the caller.
///
/// # Errors
/// Returns [`SeedFault::Range`] for a malformed or overflowing row.
pub fn pads(current: &str, preview: &str) -> Result<(PhysicalPad, PhysicalPad), SeedFault> {
    Ok((pad(current)?, pad(preview)?))
}

/// Decode one authenticated physical action row.
///
/// # Errors
/// Returns [`SeedFault::Range`] for a malformed or overflowing row.
pub fn pad(text: &str) -> Result<PhysicalPad, SeedFault> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(SeedFault::Range);
    }
    u16::from_str_radix(text, 16)
        .map(PhysicalPad)
        .map_err(|_| SeedFault::Range)
}

/// Import the complete hardware owner at the independently qualified idle origin.
///
/// The caller supplies source-qualified IRQ policy and physical actions. The
/// origin has completed its charge; the future automatic latch resets serial
/// readiness before its first consumer. No content resolver is required here.
///
/// # Errors
/// Rejects truncated, malformed, unsupported or invalid complete hardware state.
pub fn decode(bytes: &[u8], policy: IrqPolicy, current: PhysicalPad) -> Result<State, SeedFault> {
    if bytes.len() != 494 {
        return Err(SeedFault::Length);
    }
    let c = Context(bytes);
    if c.byte(82) != 0
        || c.byte(326) != 2
        || c.flag(314)?
        || c.integer(466) != 0
        || c.integer(462) != 0
        || c.flag(330)?
        || c.flag(377)?
        || c.flag(378)?
        || c.flag(390)?
    {
        return Err(SeedFault::Unsupported);
    }
    let position = BeamPosition {
        line: c.word(84),
        horizontal: c.word(86),
    };
    if position.horizontal < 10 {
        return Err(SeedFault::Unsupported);
    }
    let mut history = [position; 6];
    for (place, back) in history.iter_mut().zip([10_u16, 8, 6, 4, 2, 0]) {
        place.horizontal = position.horizontal.wrapping_sub(back);
    }
    let residue = u8::try_from(c.integer(382)).map_err(|_| SeedFault::Range)?;
    if c.integer(346)
        != 538_u32
            .checked_sub(u32::from(residue))
            .ok_or(SeedFault::Range)?
        || c.integer(362) != 1104
    {
        return Err(SeedFault::Unsupported);
    }
    let pending_line = if c.flag(391)? {
        Some(if c.flag(392)? {
            LineTransfer::Visible
        } else {
            LineTransfer::Initialize
        })
    } else {
        None
    };
    let mut channels = std::array::from_fn(|_| Channel::default());
    for (slot, channel) in channels.iter_mut().enumerate() {
        *channel = c.channel(slot)?;
    }
    let state = State {
        hardware: Hardware {
            now: MasterTicks(0),
            charge_debt: 0,
            beam: Beam {
                odd: c.flag(83)?,
                position,
                history,
                subtick: 0,
                residue,
                line_ticks: c.narrow(338)?,
                refreshed: c.flag(350)?,
                thresholds: Thresholds {
                    initialize_at: c.narrow(354)?,
                    initialize: c.flag(358)?,
                    visible: c.flag(366)?,
                },
            },
            interrupts: c.interrupts(policy)?,
            math: c.math(),
            controller: c.controller(current)?,
            transfers: Transfers {
                channels,
                active: false,
                pending_line,
                pending_bulk: false,
                intrinsic: u64::from(c.integer(386)),
                pipeline: None,
            },
        },
        work: Vec::new(),
        interrupt_boundary: None,
        pending_handler: None,
        interrupted_wait: None,
    };
    state.validate().map_err(|_| SeedFault::InvalidState)?;
    Ok(state)
}
