//! The Ceres shaft: the elevator's pad and the cover below it, and the overlay's lights.
//!
//! In the shaft as she arrives, the elevator's pad holds her: while it does, her moves and her
//! animation don't run, and the pad sets her height after her frame. The shaft's overlay, run
//! with the room's enemies, steps the lights, whose beacon sets screen colours.

use super::{Content, Fault, Rules, station};
use crate::production::shaft;
use boss_ceres_ridley::{Lights, PadAndCover};
use world_doors::Palette;

impl station::State {
    /// The pad's run, which sets `samus`'s height while it holds her.
    pub(super) fn pad(&self, samus: &mut samus_movement::Samus, rules: &Rules) -> PadAndCover {
        shaft::run_pad(self.pad_and_cover, samus, &rules.ceres)
    }
}

impl super::State {
    /// The shaft's lights, on a frame the room's enemies run: with the overlay in the room, the
    /// beacon's step sets the colours it names.
    pub(super) fn lights(
        &self,
        content: &Content,
        rules: &Rules,
        palette: &mut Palette,
    ) -> Result<Option<Lights>, Fault> {
        if !self.station.shaft_overlay {
            return Ok(None);
        }
        let lights = shaft::lights(
            self.frame.finished_frames,
            self.passage.doors,
            self.station.stage,
            &rules.ceres,
        )
        .map_err(Fault::Lights)?;
        shaft::show(lights, &content.rooms.actors.beacon, palette).map_err(Fault::Lights)?;
        Ok(Some(lights))
    }
}

#[cfg(test)]
mod tests {
    use crate::production::played::Fault;
    use crate::production::played::tests::{content, play, play_with, state};
    use crate::production::tests::fixtures;
    use boss_ceres_ridley::{PadAndCover, Stage};
    use core::num::Wrapping;
    use types::{Pair, Parts};
    use world_doors::{Doors, FadeTime, Passage};

    /// The shaft's pad, under her with its wait `wait`, and the cover below.
    fn on_the_pad(wait: u16) -> PadAndCover {
        let her = fixtures::samus().position;
        let mut placed = PadAndCover::arrival(
            Pair {
                x: her.x.pixel(),
                y: her.y.pixel(),
            },
            &boss_ceres_ridley::Rules::default(),
        )
        .pad_and_cover;
        if let Some(pad) = placed.pad.as_mut() {
            pad.wait = boss_ceres_ridley::PadWait::new(wait).unwrap();
        }
        placed
    }

    #[test]
    fn while_the_pad_holds_her_her_moves_and_animation_dont_run_and_it_sets_her_height() {
        let mut held = state(Vec::new());
        held.station.pad_and_cover = on_the_pad(1);
        // A pixel above the floor, so the pad's run lowers her to it.
        held.samus.movement.0.position.y = types::fixed(Wrapping(71), 0);
        let before = held.clone();
        let outputs = play(&mut held, types::button::RIGHT, Stage::Calm).unwrap();
        assert_eq!(outputs.movement, None);
        assert_eq!(
            held.samus.movement.0.position.x,
            before.samus.movement.0.position.x
        );
        assert_eq!(
            held.samus.movement.0.position.y,
            types::fixed(Wrapping(72), 0)
        );
        assert_eq!(held.samus.appearance, before.samus.appearance);
        assert!(
            held.station.pad_and_cover.holds_her(),
            "she reaches the floor held"
        );
        // The next run lets her go after her frame, still held; the frame after, she moves.
        play(&mut held, types::button::RIGHT, Stage::Calm).unwrap();
        assert!(!held.station.pad_and_cover.holds_her());
        assert_eq!(
            held.samus.movement.0.position.x,
            before.samus.movement.0.position.x
        );
        play(&mut held, types::button::RIGHT, Stage::Calm).unwrap();
        assert!(held.samus.movement.0.position.x.0 > before.samus.movement.0.position.x.0);
    }

    #[test]
    fn with_the_shafts_overlay_the_beacon_sets_its_colours_from_the_count_of_finished_frames() {
        let mut content = content();
        let mut beacon = data::assets::actors::BeaconColours::default();
        for (step, set) in beacon.sets.iter_mut().enumerate() {
            *set = [data::assets::art::Colour(u16::try_from(step).unwrap() + 1); 6];
        }
        content.rooms.actors.beacon = beacon;
        let mut shaft = state(Vec::new());
        shaft.station.shaft_overlay = true;
        // Step (24 / 8) mod 8 = 3 of the cycle.
        shaft.frame.finished_frames = 24;
        let outputs =
            play_with(&mut shaft, &content, types::Buttons::default(), Stage::Calm).unwrap();
        assert_eq!(
            outputs
                .lights
                .and_then(|lights| lights.beacon)
                .map(boss_ceres_ridley::BeaconStep::place),
            Some(3)
        );
        let first = data::assets::actors::BeaconColours::FIRST;
        assert_eq!(shaft.palette.screen[first - 1], types::Colour::BLACK);
        assert!(
            shaft.palette.screen[first..first + 6]
                .iter()
                .all(|colour| colour.bits() == 4)
        );
        assert_eq!(shaft.palette.screen[first + 6], types::Colour::BLACK);
        assert_eq!(shaft.frame.finished_frames, 25);
        // In the escape a later piece owns the overlay.
        assert_eq!(
            play_with(
                &mut shaft,
                &content,
                types::Buttons::default(),
                Stage::Escape
            ),
            Err(Fault::Escape)
        );
    }

    /// As the colours come up, the lights run before the fade's step, so on its first step the
    /// beacon still shows; as they go dark the fade's step comes first, so the beacon shows none.
    #[test]
    fn the_beacon_shows_before_the_fade_in_takes_its_step_and_not_once_the_fade_out_has() {
        let mut content = content();
        content.rooms.actors.beacon = data::assets::actors::BeaconColours {
            sets: [[data::assets::art::Colour(7); 6]; 8],
        };
        let fading = |phase| {
            let mut shaft = state(Vec::new());
            shaft.station.shaft_overlay = true;
            shaft.passage.doors = Doors::Passing(Passage {
                phase,
                arrival: None,
                slide_frames: 0,
            });
            let outputs =
                play_with(&mut shaft, &content, types::Buttons::default(), Stage::Calm).unwrap();
            (outputs.lights.and_then(|lights| lights.beacon), shaft)
        };
        let first = data::assets::actors::BeaconColours::FIRST;
        let (shown, brightening) = fading(world_doors::Phase::Brightening(FadeTime::START));
        assert!(shown.is_some());
        assert_eq!(brightening.palette.screen[first].bits(), 7);
        let (shown, darkening) = fading(world_doors::Phase::Darkening(FadeTime::START));
        assert_eq!(shown, None);
        assert_eq!(darkening.palette.screen[first], types::Colour::BLACK);
    }
}
