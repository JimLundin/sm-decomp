//! Beam and status consumers over the actual owned delivery authority.
#![forbid(unsafe_code)]

use super::{
    irq,
    settle::SettlementHost,
    state::{DisplayError, DisplayPhase, DisplayState},
};

/// Actual G capabilities at the original beam points. Each callback uses the
/// supplied current owner, commits its original prefix and returns before the
/// next field query. No copied channels, delayed operations or default effects.
/// Raster rows must update real pixel/status inputs and retain immutable history
/// before the subsequent HDMA writes. Room transition is the actual IRQ effect.
pub trait Effects: irq::OwnedIrqEffects {
    fn initialize_hdma(&mut self, state: &mut DisplayState) -> Result<(), DisplayError>;
    fn step_hdma(&mut self, state: &mut DisplayState) -> Result<(), DisplayError>;
    /// Native row0 only changes frame status; visible raster callbacks start1.
    fn raster_row(&mut self, state: &mut DisplayState, row: u16) -> Result<(), DisplayError>;
}

pub struct OwnedSettlement<'a, E> {
    pub state: &'a mut DisplayState,
    pub effects: &'a mut E,
}

/// Arms failure poisoning for the one owner without catching unwinding or
/// rolling back any committed effect. Disarm only after the operation succeeds.
pub struct FaultGuard<'a> {
    state: &'a mut DisplayState,
    armed: bool,
}

impl<'a> FaultGuard<'a> {
    pub fn new(state: &'a mut DisplayState) -> Self {
        Self { state, armed: true }
    }
    pub fn state(&mut self) -> &mut DisplayState {
        self.state
    }
    pub fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for FaultGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.state.phase = DisplayPhase::Failed;
        }
    }
}

/// Actual owned consumer; refresh has finished and the owner entered Settling.
/// Failure preserves delivery/effect prefixes and prevents later Scene sealing.
pub fn settle_owned(
    state: &mut DisplayState,
    effects: &mut impl Effects,
) -> Result<(), DisplayError> {
    if state.phase == DisplayPhase::Failed {
        return Err(DisplayError::Poisoned);
    }
    if state.phase != DisplayPhase::Settling {
        return Err(DisplayError::Phase);
    }
    let mut guard = FaultGuard::new(state);
    let result = super::settle::settle(&mut OwnedSettlement {
        state: guard.state(),
        effects,
    });
    if result.is_ok() {
        guard.disarm();
    }
    result
}

impl<E: Effects> SettlementHost for OwnedSettlement<'_, E> {
    type Error = DisplayError;

    fn reset_beam(&mut self) {
        self.state.delivery.vertical_position = 0;
        self.state.delivery.horizontal_position = self.state.delivery.vertical_position;
    }
    fn nmi_wanted(&self) -> bool {
        self.state.delivery.nmi_wanted
    }
    fn advance_at(&mut self, position: u16) -> Result<(), DisplayError> {
        self.state.delivery.horizontal_position = position;
        advance_beam(self.state, self.effects)
    }
    fn vertical_irq_enabled(&self) -> bool {
        self.state.delivery.vertical_irq_enabled
    }
    fn vertical_position(&self) -> u16 {
        self.state.delivery.vertical_position
    }
    fn vertical_timer(&self) -> u16 {
        self.state.interrupt.vertical
    }
    fn interrupt(&mut self) -> Result<(), DisplayError> {
        irq::owned_advance(self.state, self.effects)
    }
    fn clear_nmi(&mut self) {
        self.state.delivery.nmi_wanted = false;
    }
}

/// Mirrors advance_beam, including its immediate effects and final +2/wrap.
/// The four-position settlement caller supplies actual event positions; no
/// CPU/APU/input cycles, normalization or synthetic NMI eligibility are added.
pub fn advance_beam(
    state: &mut DisplayState,
    effects: &mut impl Effects,
) -> Result<(), DisplayError> {
    if state.delivery.horizontal_position == 0 {
        let mut starting_vblank = false;
        if state.delivery.vertical_position == 0 {
            state.delivery.in_vblank = false;
            state.delivery.in_nmi = false;
            effects.initialize_hdma(state)?;
        } else if state.delivery.vertical_position == 225 {
            state.raster.frame_overscan = state.raster.overscan;
            starting_vblank = !state.raster.frame_overscan;
        } else if state.delivery.vertical_position == 240 && !state.delivery.in_vblank {
            starting_vblank = true;
        }
        if starting_vblank {
            if !state.effective.forced_blank {
                state.latches.object_address = state.latches.object_written_address;
                state.latches.object_high_table = state.latches.object_written_high_table;
                state.latches.object_second = false;
            }
            state.raster.frame_interlaced = state.raster.interlaced;
            state.delivery.in_vblank = true;
            state.delivery.in_nmi = true;
            // The selected native CPU is marked unconditionally at this point.
            state.delivery.nmi_wanted = true;
            if state.delivery.automatic_input_read {
                state.delivery.automatic_input_timer = 0;
            }
        }
    } else if state.delivery.horizontal_position == 512 {
        if !state.delivery.in_vblank && !state.delivery.raster_suppressed {
            let row = state.delivery.vertical_position;
            if row == 0 {
                state.raster.mosaic_origin = 1;
                state.raster.range_overflow = false;
                state.raster.time_overflow = false;
                state.raster.even_frame = !state.raster.even_frame;
            } else {
                effects.raster_row(state, row)?;
            }
        }
    } else if state.delivery.horizontal_position == 1024 && !state.delivery.in_vblank {
        effects.step_hdma(state)?;
    }
    state.delivery.horizontal_position = state.delivery.horizontal_position.wrapping_add(2);
    if state.delivery.horizontal_position == 1364 {
        state.delivery.horizontal_position = 0;
        state.delivery.vertical_position = state.delivery.vertical_position.wrapping_add(1);
        if state.delivery.vertical_position == 262 {
            state.delivery.vertical_position = 0;
            state.delivery.beam_frames = state.delivery.beam_frames.wrapping_add(1);
        }
    }
    Ok(())
}

/// Actual interrupt-enable publication; staged mirror bytes remain independent.
pub fn interrupt_enable(state: &mut DisplayState, value: u8) {
    state.delivery.automatic_input_read = value & 1 != 0;
    if !state.delivery.automatic_input_read {
        state.delivery.automatic_input_timer = 0;
    }
    state.delivery.horizontal_irq_enabled = value & 0x10 != 0;
    state.delivery.vertical_irq_enabled = value & 0x20 != 0;
    state.delivery.nmi_enabled = value & 0x80 != 0;
    if !state.delivery.horizontal_irq_enabled && !state.delivery.vertical_irq_enabled {
        state.delivery.in_irq = false;
        state.delivery.irq_wanted = false;
    }
}
