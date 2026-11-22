use super::{Event, EventKind, Fault};
use canonical::Canonical;
use data::dma::BusAddress;
use data::timing::{
    ChannelConfig, Interrupt, InterruptEnable, IrqPolicy, MasterTicks, TransferPort,
};

/// Retained beam position at a two-master-tick poll boundary.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BeamPosition {
    /// Scanline within the field.
    pub line: u16,
    /// Horizontal master ticks.
    pub horizontal: u16,
}

/// Beam and refresh owner, including delayed interrupt history.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Beam {
    /// Odd NTSC field.
    pub odd: bool,
    /// Current two-tick position.
    pub position: BeamPosition,
    /// Oldest to newest two-tick positions.
    pub history: [BeamPosition; 6],
    /// Unpolled single master tick.
    pub subtick: u8,
    /// Previous-line residue used by refresh and transfer alignment.
    pub residue: u8,
    /// Current scanline length.
    pub line_ticks: u16,
    /// This line has already paid refresh.
    pub refreshed: bool,
    /// Owned transfer recognition thresholds.
    pub thresholds: Thresholds,
}

/// Interrupt source's held physical transition state.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InterruptLine {
    /// Physical comparison, line and delayed hold.
    pub physical: PhysicalLine,
    /// Transition available to an exposure poll.
    pub transition: bool,
    /// Admitted request awaiting the completed exposure.
    pub pending: bool,
}

/// Native interrupt admission state.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Interrupts {
    /// Source enables.
    pub enable: InterruptEnable,
    /// Nine-bit horizontal target.
    pub horizontal_target: u16,
    /// Nine-bit vertical target.
    pub vertical_target: u16,
    /// NMI delayed state.
    pub nmi: InterruptLine,
    /// IRQ delayed state.
    pub irq: InterruptLine,
    /// Poll is locked until another charge.
    pub locked: bool,
    /// Current native IRQ policy.
    pub policy: IrqPolicy,
}
impl Interrupts {
    pub(super) const fn poll(&mut self, history: &[BeamPosition; 6]) {
        let [oldest, _, end_exclusion, _, delayed, _] = history;
        if self.nmi.physical.hold {
            self.nmi.physical.hold = false;
            if self.enable.nmi {
                self.nmi.transition = true;
            }
        }
        let valid = delayed.line >= 225;
        if !self.nmi.physical.valid && valid {
            self.nmi.physical.line = true;
            self.nmi.physical.hold = true;
        } else if self.nmi.physical.valid && !valid {
            self.nmi.physical.line = false;
        }
        self.nmi.physical.valid = valid;
        self.irq.physical.hold = false;
        let enabled = self.enable.irq.vertical() || self.enable.irq.horizontal();
        if self.irq.physical.line && enabled {
            self.irq.transition = true;
        }
        let valid = enabled
            && (!self.enable.irq.vertical() || oldest.line == self.vertical_target)
            && (!self.enable.irq.horizontal()
                || oldest.horizontal == self.horizontal_target.wrapping_add(1).wrapping_mul(4))
            && (self.vertical_target == 0 || end_exclusion.line != 0);
        if !self.irq.physical.valid && valid {
            self.irq.physical.line = true;
            self.irq.physical.hold = true;
        }
        self.irq.physical.valid = valid;
    }
    pub(super) fn sample(&mut self) {
        if self.locked {
            return;
        }
        if self.nmi.transition {
            self.nmi.transition = false;
            self.nmi.pending = true;
        }
        if self.irq.transition {
            self.irq.transition = false;
            if self.policy == IrqPolicy::Eligible {
                self.irq.pending = true;
            }
        }
    }
    pub(super) const fn pending(&self) -> Option<Interrupt> {
        if self.nmi.pending {
            Some(Interrupt::Nmi)
        } else if self.irq.pending {
            Some(Interrupt::Irq)
        } else {
            None
        }
    }
}

/// Retained multiplication unit, advanced only by arithmetic exposure edges.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Math {
    /// Stored dividend register; division initiation remains unsupported.
    pub dividend: u16,
    /// Stored first operand.
    pub multiplicand: u8,
    /// Stored second operand.
    pub multiplier: u8,
    /// Visible product result.
    pub product: u16,
    /// Combined shifting operand register.
    pub operand: u16,
    /// Addend for the next edge.
    pub shift: u32,
    /// Edges still required for this multiplication.
    pub remaining: u8,
}
impl Math {
    pub(in crate::production) fn start_multiply(&mut self, multiplier: u8) {
        self.multiplier = multiplier;
        self.product = 0;
        self.remaining = 8;
        self.operand = (u16::from(multiplier).wrapping_shl(8)) | u16::from(self.multiplicand);
        self.shift = u32::from(multiplier);
    }

    pub(in crate::production) const fn edge(&mut self) {
        if self.remaining == 0 {
            return;
        }
        self.remaining = self.remaining.wrapping_sub(1);
        if self.operand & 0x1 != 0 {
            let [low, high, _, _] = self.shift.to_le_bytes();
            self.product = self.product.wrapping_add(u16::from_le_bytes([low, high]));
        }
        self.operand >>= 1;
        self.shift = self.shift.wrapping_shl(1);
    }
}

/// First controller port's physical button word in serial order.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicalPad(pub u16);

/// Full physical button state presented at an absolute hardware time.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadChange {
    /// Time in the admitted hardware epoch.
    pub at: MasterTicks,
    /// Complete held state, including releases.
    pub pad: PhysicalPad,
}

/// Qualified original controller poll continuation and committed physical input.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Controller {
    /// Charge-end accumulator; at most one iteration is performed per charge end.
    pub accumulator: u64,
    /// Poll iteration since the field reset.
    pub iteration: u32,
    /// Latched polling enable.
    pub latched: bool,
    /// Physical automatic polling is active.
    pub active: bool,
    /// Next serial bit ordinal.
    pub serial: u8,
    /// Four accumulated result words.
    pub words: [u16; 4],
    /// Held first-port word for hardware serial polling.
    pub committed_pad: PhysicalPad,
    /// Ordered, unsampled host changes. These can precede the current hardware time.
    pub pending_pad: Vec<PadChange>,
}
impl Controller {
    fn commit_pad(&mut self, crossing: MasterTicks, events: &mut Vec<Event>) {
        let eligible = self
            .pending_pad
            .partition_point(|change| change.at <= crossing);
        if let Some(change) = self.pending_pad.drain(..eligible).next_back() {
            self.committed_pad = change.pad;
            events.push(Event {
                at: crossing,
                kind: EventKind::ControllerCommitted,
            });
        }
    }

    fn shift(&mut self) {
        let bit = if self.serial < 12 {
            (self.committed_pad.0 >> u32::from(15_u8.wrapping_sub(self.serial))) & 0x1
        } else {
            0
        };
        let [first, second, third, fourth] = &mut self.words;
        *first = first.wrapping_shl(1) | bit;
        *second = second.wrapping_shl(1);
        *third = third.wrapping_shl(1);
        *fourth = fourth.wrapping_shl(1);
        self.serial = self.serial.wrapping_add(1);
    }
    pub(super) fn charge_end(
        &mut self,
        ticks: u64,
        line: u16,
        enabled: bool,
        now: MasterTicks,
        events: &mut Vec<Event>,
    ) -> Result<(), Fault> {
        self.accumulator = self
            .accumulator
            .checked_add(ticks)
            .ok_or(Fault::TimeOverflow)?;
        if self.accumulator < 256 {
            return Ok(());
        }
        self.accumulator = self.accumulator.wrapping_sub(256);
        if line < 225 {
            return Ok(());
        }
        if self.iteration == 0 {
            self.latched = enabled;
        }
        self.active = self.iteration <= 15;
        if self.active && self.latched {
            if self.iteration == 0 {
                self.serial = 0;
                events.push(Event {
                    at: now,
                    kind: EventKind::ControllerStart,
                });
            }
            self.shift();
            if self.iteration == 15 {
                events.push(Event {
                    at: now,
                    kind: EventKind::ControllerEnd,
                });
            }
        }
        self.iteration = self.iteration.checked_add(1).ok_or(Fault::InvalidState)?;
        Ok(())
    }
}

/// Mutable configuration and progress of one hardware channel.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    /// Admitted source/destination format.
    pub config: ChannelConfig,
    /// Bulk source cursor.
    pub source_offset: u16,
    /// Bulk remaining register or indirect data pointer.
    pub pointer: u16,
    /// Descriptor table cursor.
    pub table: u16,
    /// Line countdown and repetition bit.
    pub lines: u8,
    /// Line-transfer enable and descriptor/payload state.
    pub line: LineStatus,
    /// Bulk channel enabled.
    pub bulk_enabled: bool,
}
impl Channel {
    pub(super) const fn table_address(&self) -> BusAddress {
        BusAddress {
            bank: self.config.source_bank,
            offset: self.table,
        }
    }
}

/// Hardware transfer recognized for the next arbitration.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineTransfer {
    /// Field-start table initialization.
    Initialize,
    /// Visible-line payload/update service.
    Visible,
}

/// One global staged transfer byte.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Payload {
    /// Destination device.
    pub target: TransferPort,
    /// Source byte.
    pub value: u8,
}

/// Owned transfer arbitration and global pipeline.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Transfers {
    /// Fixed hardware slot order.
    pub channels: [Channel; 8],
    /// Newly armed service must first permit one CPU exposure.
    pub active: bool,
    /// Recognized line service.
    pub pending_line: Option<LineTransfer>,
    /// Bulk request waiting for arbitration.
    pub pending_bulk: bool,
    /// Alignment/body ticks of the active episode, excluding refresh/completion.
    pub intrinsic: u64,
    /// Global delayed write shared by all channels.
    pub pipeline: Option<Payload>,
}

/// Complete platform hardware owner.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Hardware {
    /// Seed-relative elapsed master ticks.
    pub now: MasterTicks,
    /// Owned beam/history/refresh.
    pub beam: Beam,
    /// Charge debt retained until the current chunk ends.
    pub charge_debt: u64,
    /// Owned native interrupt polling state.
    pub interrupts: Interrupts,
    /// Owned arithmetic progress.
    pub math: Math,
    /// Owned physical controller progress.
    pub controller: Controller,
    /// Owned channel and pipeline progress.
    pub transfers: Transfers,
}
impl Hardware {
    pub(super) fn charge(
        &mut self,
        ticks: u64,
        complete: bool,
        events: &mut Vec<Event>,
    ) -> Result<(), Fault> {
        if ticks == 0 {
            return Err(Fault::InvalidCharge);
        }
        let previous_time = self.now;
        let previous_subtick = self.beam.subtick;
        self.interrupts.locked = false;
        self.now.0 = self.now.0.checked_add(ticks).ok_or(Fault::TimeOverflow)?;
        self.charge_debt = self
            .charge_debt
            .checked_add(ticks)
            .ok_or(Fault::TimeOverflow)?;
        let quantum = ticks
            .checked_add(u64::from(self.beam.subtick))
            .ok_or(Fault::TimeOverflow)?;
        self.beam.subtick = u8::try_from(quantum % 2).map_err(|_| Fault::InvalidState)?;
        for ordinal in 0..quantum / 2 {
            let crossing = ordinal
                .checked_add(1)
                .and_then(|steps| steps.checked_mul(2))
                .and_then(|ticks| ticks.checked_sub(u64::from(previous_subtick)))
                .and_then(|ticks| previous_time.0.checked_add(ticks))
                .ok_or(Fault::TimeOverflow)?;
            self.beam_step(MasterTicks(crossing), events);
        }
        if !complete {
            return Ok(());
        }
        let debt = std::mem::take(&mut self.charge_debt);
        self.controller.charge_end(
            debt,
            self.beam.position.line,
            self.interrupts.enable.controller,
            self.now,
            events,
        )?;
        let threshold = 538_u16.wrapping_sub(u16::from(self.beam.residue));
        if !self.beam.refreshed
            && self
                .beam
                .position
                .horizontal
                .wrapping_add(u16::from(self.beam.subtick))
                >= threshold
        {
            self.beam.refreshed = true;
            self.charge(40, true, events)?;
        }
        Ok(())
    }
    fn beam_step(&mut self, crossing: MasterTicks, events: &mut Vec<Event>) {
        let beam = &mut self.beam;
        beam.position.horizontal = beam.position.horizontal.wrapping_add(2);
        if beam.position.horizontal == beam.line_ticks {
            beam.position.horizontal = 0;
            beam.position.line = beam.position.line.wrapping_add(1);
            if beam.position.line == 262 {
                beam.position.line = 0;
                beam.odd = !beam.odd;
            }
            let [line_low, _] = beam.line_ticks.to_le_bytes();
            beam.residue = beam.residue.wrapping_add(line_low & 0x7) & 0x7;
            beam.line_ticks = if beam.odd && beam.position.line == 240 {
                1360
            } else {
                1364
            };
            beam.refreshed = false;
            if beam.position.line == 0 {
                beam.thresholds.initialize = false;
                beam.thresholds.initialize_at = 12_u16.wrapping_add(u16::from(beam.residue));
                self.controller.iteration = 0;
            }
            if beam.position.line <= 224 {
                beam.thresholds.visible = false;
            }
            if beam.position.line == 241 {
                self.controller.commit_pad(crossing, events);
            }
        }
        beam.history.rotate_left(1);
        if let Some(last) = beam.history.last_mut() {
            *last = beam.position;
        }
        if beam.position.horizontal & 0x2 != 0 {
            self.interrupts.poll(&beam.history);
        }
    }
    pub(super) fn dma_charge(&mut self, ticks: u64, events: &mut Vec<Event>) -> Result<(), Fault> {
        self.transfers.intrinsic = self
            .transfers
            .intrinsic
            .checked_add(ticks)
            .ok_or(Fault::TimeOverflow)?;
        self.charge(ticks, true, events)
    }
}

/// Owned threshold recognition for the two hardware line-transfer phases.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Thresholds {
    /// Field-start threshold retains the residue from line zero.
    pub initialize_at: u16,
    /// Field initialization threshold recognized.
    pub initialize: bool,
    /// Visible-line threshold recognized.
    pub visible: bool,
}
/// Physical interrupt comparator state before admission.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicalLine {
    /// Current comparison validity.
    pub valid: bool,
    /// Physical line remains asserted.
    pub line: bool,
    /// Hold until the next qualified hardware poll.
    pub hold: bool,
}
/// Owned line-transfer descriptor and payload state.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineStatus {
    /// Hardware channel enabled.
    pub enabled: bool,
    /// Zero descriptor completed this channel.
    pub completed: bool,
    /// Payload due at the next line service.
    pub payload: bool,
}
