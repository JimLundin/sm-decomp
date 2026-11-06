//! Samus's palette (bank $91 log, handle Samus palette `$91:D6F7`).
//!
//! <https://patrickjohnston.org/bank/91>. The normal new state handler (bank $90 log,
//! `$90:E725`) runs it after the pose update; riding the elevator (`$90:E8EC`) doesn't.
//!
//! Slice 0 has none of what takes priority over the visor: no beam charge or charged-shot
//! glow, no grapple, no hyper beam, no special palette (screw attack, speed boosting,
//! shinesparking, crystal flash, x-ray) and no hurt flash. With all of those off the handler
//! reaches the visor (`$91:D83F`) through the no-charge branch of the beam charge palettes
//! (`$91:D7B0`), and the special palette handler for normal play (`$91:D9B2`) keeps the suit
//! palette as it is.

use data::palette::VisorColours;

use crate::state::State;
use crate::state::room::{ElevatorStatus, LayerBlending};
use crate::state::samus::{Visor, VisorTimer};
use crate::state::video::Colour;

/// The visor's colour: sprite palette 4 (Samus's), colour 4.
const VISOR_COLOUR: usize = 0xc4;

/// Frames each visor colour shows (`$91:D864`).
const VISOR_FRAMES: VisorTimer = VisorTimer(5);

/// Handle Samus's palette for the frame (`$91:D6F7`).
// open: the beam charge and charged-shot glow palettes, the grapple's, the hyper beam's, the
// special palettes (`$91:D72D`) and the hurt flash and super special palettes (`$91:D8A5`)
// join with their lanes; slice 0 reaches none of them. The new state handler is the original's
// `$0A44` pointer; until the Samus commands that set it exist (bank $90, `$90:F084`), riding is
// read from the ride's status, which the elevator ends on the frame it sets the normal handler.
pub fn handle(state: &mut State, colours: &VisorColours) {
    if !matches!(state.elevator.status, ElevatorStatus::Inactive) {
        return;
    }
    let colour = visor(&mut state.samus.visor, state.room.layer_blending, colours);
    if let (Some(colour), Some(slot)) = (colour, state.video.batch.cgram.get_mut(VISOR_COLOUR)) {
        *slot = colour;
    }
}

/// The visor palette (`$91:D83F`): where colour math adds the backdrop, a new colour from
/// the cycle every five frames; elsewhere the cycle restarts at its first colour.
///
/// Returns the colour it writes to the visor, if it writes one.
pub fn visor(visor: &mut Visor, blending: LayerBlending, colours: &VisorColours) -> Option<Colour> {
    if !blending.adds_backdrop() {
        // `$91:D89D`: the next frame in such a room shows the first colour.
        *visor = Visor::default();
        return None;
    }
    // A timer of 0 never stays: the count stores 5 when it reaches 0, and the restart 1.
    visor.timer = VisorTimer(visor.timer.0.wrapping_sub(1));
    if visor.timer.0 != 0 {
        return None;
    }
    visor.timer = VISOR_FRAMES;
    let colour = colours.0.get(visor.shade.index()).copied().map(Colour);
    visor.shade = visor.shade.next();
    colour
}

#[cfg(test)]
mod tests {
    use data::palette::VisorColours;

    use super::{VISOR_COLOUR, handle, visor};
    use crate::state::State;
    use crate::state::room::{ElevatorStatus, LayerBlending};
    use crate::state::samus::{Visor, VisorShade, VisorTimer};
    use crate::state::video::Colour;

    const COLOURS: VisorColours = VisorColours([0x1111, 0x2222, 0x3333]);
    const BACKDROP: LayerBlending = LayerBlending(0x2a);

    #[test]
    fn the_visor_waits_out_its_timer() {
        let mut cycle = Visor {
            shade: VisorShade::Dim,
            timer: VisorTimer(3),
        };
        assert_eq!(visor(&mut cycle, BACKDROP, &COLOURS), None);
        assert_eq!(
            cycle,
            Visor {
                shade: VisorShade::Dim,
                timer: VisorTimer(2),
            }
        );
    }

    #[test]
    fn the_visor_steps_through_three_colours_five_frames_apart() {
        let mut cycle = Visor::default();
        let mut shown = Vec::new();
        for _ in 0..11 {
            shown.push(visor(&mut cycle, LayerBlending(0x28), &COLOURS));
        }
        let colour = |c| Some(Colour(c));
        assert_eq!(
            shown,
            [
                colour(0x1111),
                None,
                None,
                None,
                None,
                colour(0x2222),
                None,
                None,
                None,
                None,
                colour(0x3333),
            ]
        );
        assert_eq!(visor(&mut cycle, BACKDROP, &COLOURS), None);
        assert_eq!(cycle.shade, VisorShade::Bright);
    }

    #[test]
    fn elsewhere_the_visor_restarts_without_a_write() {
        let mut cycle = Visor {
            shade: VisorShade::Dark,
            timer: VisorTimer(1),
        };
        assert_eq!(visor(&mut cycle, LayerBlending(0), &COLOURS), None);
        assert_eq!(cycle, Visor::default());
    }

    #[test]
    fn the_handler_writes_the_visor_colour_but_not_on_the_ride() {
        let mut state = State::default();
        state.room.layer_blending = BACKDROP;
        handle(&mut state, &COLOURS);
        assert_eq!(state.video.batch.cgram[VISOR_COLOUR], Colour(0x1111));
        assert_eq!(state.samus.visor.shade, VisorShade::Dim);

        let mut riding = State::default();
        riding.room.layer_blending = BACKDROP;
        riding.elevator.status = ElevatorStatus::EnteringRoom;
        handle(&mut riding, &COLOURS);
        assert_eq!(riding.video.batch.cgram[VISOR_COLOUR], Colour::default());
        assert_eq!(riding.samus.visor, Visor::default());
    }
}
