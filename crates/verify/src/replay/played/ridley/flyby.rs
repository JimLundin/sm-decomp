//! Compare the admitted display consumer and the ordinary room's flyby output.

mod native;
use super::super::played;
use crate::replay::{compare::Observation, data_failure, result::Outcome};
use boss_ceres_ridley::{GetawayEffects, GetawayView, RidleyPhase};
use data::assets::ridley_flyby::Flyby;
use game::exact::ridley_flyby::{Plane, publish_tiles};
use oracle_record::FrameRecord;
use types::ceres_getaway::{FlybyBabyBeat, FlybyWingBeat, GetawayState};

pub(in crate::replay::played) use self::native::expected;

/// The previously published plane and art. Restore them once at an observed origin.
#[derive(Clone)]
pub(in crate::replay::played) struct Drawing {
    pub(in crate::replay::played) art: Flyby,
    plane: Plane,
    vram: Vec<u16>,
}

impl Drawing {
    /// First-room admission has no flyby display yet. Its first active turn
    /// publishes the plane and both sets of compared tiles.
    pub(in crate::replay::played) fn unpublished(art: Flyby) -> Self {
        Self {
            art,
            plane: Plane::default(),
            vram: vec![0; game::state::video::VRAM_WORDS],
        }
    }

    pub(in crate::replay::played) fn new(art: Flyby, seed: &FrameRecord) -> Self {
        use game::state::video::{Mode7, Mode7Select, Scroll};
        let [aa, bb, cc, dd, pivot_x, pivot_y] = seed.ppu.m7;
        let [scroll_x, scroll_y] = seed.ppu.bg_scroll.first().copied().unwrap_or_default();
        Self {
            art,
            plane: Plane {
                transform: Mode7 {
                    matrix: [aa, bb, cc, dd],
                    centre: [pivot_x, pivot_y],
                    select: Mode7Select(seed.ppu.m7sel),
                },
                scroll: Scroll {
                    x: scroll_x,
                    y: scroll_y,
                },
            },
            vram: seed.vram.to_vec(),
        }
    }

    #[cfg(test)]
    pub(in crate::replay::played) fn retained(&self) -> Result<(Plane, Vec<Observation>), Outcome> {
        Ok((self.plane, native::tile_fields(&self.vram)?))
    }

    pub(in crate::replay::played) fn actual(
        &mut self,
        room: &played::State,
        effects: GetawayEffects,
    ) -> Result<Vec<Observation>, Outcome> {
        let bad = || data_failure(None, "Ridley flyby publication");
        let RidleyPhase::Getaway(state) = room.station.ridley.ok_or_else(bad)?.phase else {
            return Err(bad());
        };
        let mut fields = native::owned(owned(state));
        let transform = effects.view.transform();
        let mut matrix = [0; 4];
        for (coefficient, bits) in matrix.iter_mut().zip(transform.matrix) {
            *coefficient = i16::try_from(bits).map_err(|_| bad())?;
        }
        let pivot = [transform.pivot.x, transform.pivot.y];
        let scroll = [transform.scroll.x, transform.scroll.y];
        let active = match effects.view {
            GetawayView::Flyby {
                zoom,
                baby_look,
                wing_look,
                ..
            } => {
                for (look, table) in [
                    (baby_look, self.art.baby.as_slice()),
                    (wing_look, self.art.wings.as_slice()),
                ] {
                    if let Some(look) = look {
                        let uploads = table.get(usize::from(look.index())).ok_or_else(bad)?;
                        publish_tiles(&mut self.vram, uploads).ok_or_else(bad)?;
                    }
                }
                let shades = room.palette.screen.get(81..96).ok_or_else(bad)?;
                fields.push(Observation::quantity(
                    "ridley.flyby.zoom",
                    i64::from(zoom.units()),
                ));
                for field in ["ridley.flyby.body.current", "ridley.flyby.body.cgram"] {
                    fields.push(native::words(
                        field,
                        shades.iter().map(|colour| colour.bits()),
                    ));
                }
                true
            }
            GetawayView::Room => false,
        };
        self.plane
            .publish(active.then_some((matrix, pivot, scroll)));
        fields.extend(native::geometry(
            if active { 7 } else { 9 },
            if active { 0x80 } else { 0 },
            scroll,
            packed(matrix, pivot),
        ));
        fields.extend(self.published()?);
        fields.push(Observation::quantity(
            "ridley.flyby.sound",
            i64::from(effects.sound.is_some()),
        ));
        fields.push(Observation::quantity(
            "ridley.flyby.push",
            i64::from(effects.push.is_some()),
        ));
        Ok(fields)
    }
    fn published(&self) -> Result<Vec<Observation>, Outcome> {
        let mut fields = Vec::from(native::physical(
            9,
            self.plane.transform.select.0,
            [self.plane.scroll.x, self.plane.scroll.y],
            packed(self.plane.transform.matrix, self.plane.transform.centre),
        ));
        fields.extend(native::tile_fields(&self.vram)?);
        Ok(fields)
    }

    pub(in crate::replay::played) fn origin(
        &self,
        room: &played::State,
    ) -> Result<Vec<Observation>, Outcome> {
        let mut fields = self.published()?;
        if let Some(RidleyPhase::Getaway(state)) = room.station.ridley.map(|ridley| ridley.phase) {
            fields.extend(native::owned(owned(state)));
        }
        Ok(fields)
    }
}

const fn packed([aa, bb, cc, dd]: [i16; 4], [pivot_x, pivot_y]: [i16; 2]) -> [i16; 6] {
    [aa, bb, cc, dd, pivot_x, pivot_y]
}

fn owned(state: GetawayState) -> native::State {
    let beats = |beats: types::ceres_getaway::FlybyBeats| {
        let baby = match beats.baby {
            FlybyBabyBeat::First => 0,
            FlybyBabyBeat::ToThird => 1,
            FlybyBabyBeat::Third => 2,
            FlybyBabyBeat::ToFirst => 3,
        };
        let wings = match beats.wings {
            FlybyWingBeat::First => 0,
            FlybyWingBeat::Second => 1,
        };
        (baby, wings)
    };
    match state {
        GetawayState::GetawayReady(state) => {
            let (baby, wings) = beats(state);
            native::State::Ready { baby, wings }
        }
        GetawayState::FlyingPast(state) => {
            let (baby, wings) = beats(state.beats);
            native::State::Flying {
                shown: u16::try_from(state.shown.get()).unwrap_or(u16::MAX),
                scroll: [state.scroll.x, state.scroll.y],
                turn: state.turn.0,
                baby,
                wings,
            }
        }
        GetawayState::WarningReady => native::State::Warning,
    }
}
