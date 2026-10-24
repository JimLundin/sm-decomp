//! Safe actor-owner fixture access; choreography lives at verification root.
#![forbid(unsafe_code)]
use crate::{game::session_runtime::Runtime, game_state::Game};

pub(crate) fn prepare_solid_actor(g: &mut Game, reversed: bool) -> u16 {
    let mut h = Runtime::new(g);
    let slot = h
        .session()
        .actors
        .slots
        .iter()
        .position(|a| a.parameters[0] == 0xe1)
        .unwrap();
    assert_eq!(slot, usize::from(reversed));
    h.session_mut().actors.slots[slot].properties |= 0x8000;
    assert_eq!(h.session().actors.schedule.interactive_write_position, 6);
    slot as u16 * 64
}
