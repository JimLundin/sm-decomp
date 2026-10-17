//! Complete pure-WRAM normal-camera frame from the pinned recovered C.
mod camera;
mod constraints;
mod state;

pub const WRAM_SIZE: usize = 0x20000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Handled,
    Refused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Routine {
    MainScrolling = 0,
    CallFinishedHook = 1,
    SporeSpawnFinish = 2,
    HandleX = 3,
    HandleY = 4,
    DistanceX = 5,
    DistanceY = 6,
    AutoX = 7,
    TriggerRight = 8,
    TriggerLeft = 9,
    AutoY = 10,
    TriggerDown = 11,
    TriggerUp = 12,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    Enter { routine: Routine, operand: u16 },
    Return { routine: Routine },
}

/// Passive routine observations only; no WRAM synchronization or gameplay calls.
pub trait Observer {
    fn observe(&mut self, observation: Observation) -> Result<(), String>;
}

/// Calculate the complete frame once in bounded private state. Unsupported
/// inputs refuse before observations or caller mutation, including late lookup
/// failures. Emit admitted observations, then commit the fourteen output words
/// once. An observer failure is owned and prohibits C fallback.
pub fn run(ram: &mut [u8], observer: &mut impl Observer) -> Result<Outcome, String> {
    if ram.len() != WRAM_SIZE {
        return Err("camera requires complete canonical WRAM".into());
    }
    let state = state::State::read(ram);
    if !state.admitted() {
        return Ok(Outcome::Refused);
    }
    let mut camera = camera::Camera { state, events: Vec::with_capacity(18) };
    if camera.call(Routine::MainScrolling).is_err() {
        return Ok(Outcome::Refused);
    }
    for event in camera.events {
        observer.observe(event)?;
    }
    camera.state.commit(ram);
    Ok(Outcome::Handled)
}

#[cfg(test)]
mod tests;
