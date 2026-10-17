//! Adapted from main's camera_scroll.rs and the bounded Samus distance helper.
use crate::{Observation, Routine, constraints, state::*};

pub(crate) struct Camera {
    pub state: State,
    pub events: Vec<Observation>,
}

impl Camera {
    pub fn call(&mut self, routine: Routine) -> Result<(), &'static str> {
        let operand = if routine == Routine::CallFinishedHook { self.state.word(FINISHED_HOOK) } else { 0 };
        self.events.push(Observation::Enter { routine, operand });
        match routine {
            Routine::MainScrolling => self.main_scrolling()?,
            Routine::CallFinishedHook => self.call(Routine::SporeSpawnFinish)?,
            Routine::SporeSpawnFinish => {
                if self.state.word(LAYER_Y) <= 0x1d0 {
                    self.state.set_word(LAYER_Y, 464);
                }
            }
            Routine::HandleX => self.handle_x()?,
            Routine::HandleY => self.handle_y()?,
            Routine::DistanceX => self.distance_moved(true),
            Routine::DistanceY => self.distance_moved(false),
            Routine::AutoX => constraints::automatic_x(&mut self.state)?,
            Routine::TriggerRight => constraints::horizontal(&mut self.state, true)?,
            Routine::TriggerLeft => constraints::horizontal(&mut self.state, false)?,
            Routine::AutoY => constraints::automatic_y(&mut self.state)?,
            Routine::TriggerDown => constraints::vertical(&mut self.state, true)?,
            Routine::TriggerUp => constraints::vertical(&mut self.state, false)?,
        }
        self.events.push(Observation::Return { routine });
        Ok(())
    }

    fn add_distance(&mut self, x: bool, negative: bool) {
        let (position, fraction, distance, distance_fraction) = if x {
            (LAYER_X, LAYER_X_FRACTION, DISTANCE_X, DISTANCE_X_FRACTION)
        } else {
            (LAYER_Y, LAYER_Y_FRACTION, DISTANCE_Y, DISTANCE_Y_FRACTION)
        };
        let ram = &mut self.state;
        let old = (u32::from(ram.word(position)) << 16) | u32::from(ram.word(fraction));
        let amount = (u32::from(ram.word(distance)) << 16) | u32::from(ram.word(distance_fraction));
        let next = if negative { old.wrapping_sub(amount) } else { old.wrapping_add(amount) };
        ram.set_word(position, (next >> 16) as u16);
        ram.set_word(fraction, next as u16);
    }

    fn distance_moved(&mut self, horizontal: bool) {
        let (hi, lo, prev_hi, prev_lo, out_hi, out_lo) = if horizontal {
            (X, X_FRACTION, PREVIOUS_X, PREVIOUS_X_FRACTION, DISTANCE_X, DISTANCE_X_FRACTION)
        } else {
            (Y, Y_FRACTION, PREVIOUS_Y, PREVIOUS_Y_FRACTION, DISTANCE_Y, DISTANCE_Y_FRACTION)
        };
        let ram = &mut self.state;
        let current = (u32::from(ram.word(hi)) << 16) | u32::from(ram.word(lo));
        let previous = (u32::from(ram.word(prev_hi)) << 16) | u32::from(ram.word(prev_lo));
        let difference = if ram.word(hi).wrapping_sub(ram.word(prev_hi)) & 0x8000 == 0 {
            current.wrapping_sub(previous)
        } else {
            previous.wrapping_sub(current)
        };
        let moved = difference.wrapping_add(0x10000);
        ram.set_word(out_hi, (moved >> 16) as u16);
        ram.set_word(out_lo, moved as u16);
    }

    fn handle_x(&mut self) -> Result<(), &'static str> {
        let ram = &mut self.state;
        if ram.word(PREVIOUS_X) == ram.word(X) {
            return self.call(Routine::AutoX);
        }
        const LEFT: [u16; 4] = [0xa0, 0x50, 0x20, 0xe0];
        const RIGHT: [u16; 4] = [0x60, 0x40, 0x20, 0xe0];
        let right = (ram.word(KNOCKBACK_DIRECTION) != 0
            || ram.word(POSE_AND_MOVEMENT) >> 8 == 16
            || ram.word(X_ACCELERATION_MODE) == 1)
            ^ (ram.word(POSE_AND_MOVEMENT) as u8 != 4);
        let distance = (if right { RIGHT } else { LEFT })
            .get(usize::from(ram.word(CAMERA_DISTANCE_INDEX) >> 1))
            .copied()
            .ok_or("camera distance index outside four native entries")?;
        let ideal = ram.word(X).wrapping_sub(distance);
        ram.set_word(IDEAL_X, ideal);
        if ideal != ram.word(LAYER_X) {
            let negative = ideal.wrapping_sub(ram.word(LAYER_X)) & 0x8000 != 0;
            self.add_distance(true, negative);
            self.call(if negative { Routine::TriggerLeft } else { Routine::TriggerRight })?;
        }
        Ok(())
    }

    fn handle_y(&mut self) -> Result<(), &'static str> {
        let ram = &mut self.state;
        if ram.word(PREVIOUS_Y) == ram.word(Y) {
            return self.call(Routine::AutoY);
        }
        let scroller = if ram.word(Y_DIRECTION) == 1 { DOWN_SCROLLER } else { UP_SCROLLER };
        let ideal = ram.word(Y).wrapping_sub(ram.word(scroller));
        ram.set_word(IDEAL_Y, ideal);
        if ideal != ram.word(LAYER_Y) {
            let negative = ideal.wrapping_sub(ram.word(LAYER_Y)) & 0x8000 != 0;
            self.add_distance(false, negative);
            self.call(if negative { Routine::TriggerUp } else { Routine::TriggerDown })?;
        }
        Ok(())
    }

    fn main_scrolling(&mut self) -> Result<(), &'static str> {
        let ram = &mut self.state;
        if ram.word(SLOW_GRAPPLE) != 0 {
            // The signed-high X branch skips both manual axes in the source.
            if ram.word(X) & 0x8000 == 0 {
                let relative = ram.word(X).wrapping_sub(ram.word(LAYER_X));
                if ram.word(X) < ram.word(LAYER_X) || relative < 0x60 {
                    ram.set_word(LAYER_X, ram.word(LAYER_X).wrapping_sub(3));
                } else if relative >= 0xa0 {
                    ram.set_word(LAYER_X, ram.word(LAYER_X).wrapping_add(3));
                }
                if ram.word(Y) & 0x8000 == 0 {
                    let relative = ram.word(Y).wrapping_sub(ram.word(LAYER_Y));
                    if ram.word(Y) < ram.word(LAYER_Y) || relative < 0x70 {
                        ram.set_word(LAYER_Y, ram.word(LAYER_Y).wrapping_sub(3));
                    } else if relative >= 0x90 {
                        ram.set_word(LAYER_Y, ram.word(LAYER_Y).wrapping_add(3));
                    }
                }
            }
            self.call(Routine::AutoX)?;
            self.call(Routine::AutoY)?;
        } else {
            self.call(Routine::DistanceX)?;
            self.call(Routine::HandleX)?;
            self.call(Routine::DistanceY)?;
            self.call(Routine::HandleY)?;
        }
        if self.state.word(FINISHED_HOOK) != 0 {
            self.call(Routine::CallFinishedHook)?;
        }
        for (previous, current) in
            [(PREVIOUS_X, X), (PREVIOUS_X_FRACTION, X_FRACTION), (PREVIOUS_Y, Y), (PREVIOUS_Y_FRACTION, Y_FRACTION)]
        {
            self.state.set_word(previous, self.state.word(current));
        }
        Ok(())
    }
}
