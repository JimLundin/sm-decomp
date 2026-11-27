//! The scoped ordinary power insertion observes ordered contact and exact sprite bytes.

use super::index::FixtureFile;
use super::{compare::Observation, data_failure, result::Outcome, weapons};
use enemy_interactions::{
    Centre, ContactBox, ContactConditions, DoorContact, ProtectionFrames, Radii,
};
use game::{
    production::{
        AdvanceInput,
        weapons::{FlyingDraw, Opportunity, ShotDrawingOutputs},
    },
    state::video::{NmiBatch, SpriteCount},
};
use serde::Deserialize;
use std::path::Path;
use weapons_projectiles as beam;

pub(super) const ROW: &str = "chain-any-0003-DFD7";
pub(super) const START: u64 = 156;
pub(super) const END: u64 = 161;
const FIRST: u64 = START + 1;

#[derive(Deserialize)]
struct Tape<T> {
    row: String,
    admission_cut: u64,
    #[serde(alias = "opportunity")]
    cut: Vec<T>,
}

fn load<T: serde::de::DeserializeOwned>(
    root: &Path,
    file: &FixtureFile,
    number: impl Fn(&T) -> u64,
) -> Result<Vec<T>, Outcome> {
    let tape: Tape<T> = file.read(root)?;
    if tape.row != ROW
        || tape.admission_cut != START
        || !tape.cut.iter().map(number).eq(FIRST..=END)
    {
        return Err(data_failure(None, "scoped shot fixture"));
    }
    Ok(tape.cut)
}

fn at<T>(tape: &[T], cut: u64) -> Result<&T, Outcome> {
    cut.checked_sub(FIRST)
        .and_then(|offset| usize::try_from(offset).ok())
        .and_then(|offset| tape.get(offset))
        .ok_or_else(|| data_failure(Some(cut), "scoped shot fixture cut"))
}

#[derive(Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independently watched control and drawing facts"
)]
pub(super) struct InputCut {
    cut: u64,
    shoot_held: bool,
    shoot_new: bool,
    shoot_previous_new: bool,
    draw_camera: [u16; 2],
    draw_flicker_odd: bool,
    scoped_insertion_cursor_bytes: u16,
    actor: Vec<Actor>,
}

#[derive(Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent contact permissions"
)]
struct Actor {
    slot: u8,
    centre: [u16; 2],
    radius: [u16; 2],
    normal_box: bool,
    box_live: bool,
    tangible: bool,
    protection_remaining: u16,
    updates_enabled: bool,
    interaction_enabled: bool,
    samus_stopped: bool,
    enemies_stopped: bool,
}

impl Actor {
    fn aligned(&self, actual: &game::production::weapons::DoorOpportunity) -> bool {
        let [x, y] = self.centre;
        let [rx, ry] = self.radius;
        let door = if !self.tangible {
            DoorContact::Passable
        } else if !self.box_live {
            DoorContact::WithoutFigure
        } else {
            DoorContact::Box(ContactBox {
                centre: Centre { x, y },
                radii: Radii { x: rx, y: ry },
            })
        };
        self.normal_box
            && actual.actor.0 == self.slot
            && actual.door == door
            && actual.conditions
                == ContactConditions {
                    actor_updates_enabled: self.updates_enabled,
                    interactions_enabled: self.interaction_enabled,
                    samus_time_stopped: self.samus_stopped,
                    enemies_stopped_in_time: self.enemies_stopped,
                    processing_membership: true,
                    protection_remaining: ProtectionFrames(self.protection_remaining),
                }
    }
}

impl InputCut {
    pub(super) fn load(root: &Path, file: &FixtureFile) -> Result<Vec<Self>, Outcome> {
        load(root, file, |input: &Self| input.cut)
    }

    pub(super) fn advance(
        tape: &[Self],
        wram: &[u8],
        content: &weapons::Content,
        cut: u64,
        production: &mut game::production::Production,
    ) -> Result<game::production::ProductionOutputs, Outcome> {
        let observed = at(tape, cut)?;
        let input = weapons::ordinary_input(wram, content)?;
        if (
            input.update.cannon.shoot,
            input.update.cannon.new_press,
            input.update.cannon.previous_new_press,
        ) != (
            observed.shoot_held,
            observed.shoot_new,
            observed.shoot_previous_new,
        ) || !observed.scoped_insertion_cursor_bytes.is_multiple_of(4)
            || input.opportunities.len() != observed.actor.len()
            || !input
                .opportunities
                .iter()
                .zip(&observed.actor)
                .all(|(actual, seen)| seen.aligned(actual))
        {
            return Err(data_failure(Some(cut), "scoped shot input alignment"));
        }
        let [x, y] = observed.draw_camera;
        let batch = NmiBatch {
            sprites: SpriteCount(observed.scoped_insertion_cursor_bytes / 4),
            ..NmiBatch::default()
        };
        production
            .advance(AdvanceInput::WeaponsContactDrawing {
                shot: Opportunity {
                    input: &input.update,
                    world: &input.world,
                    contacts: &input.opportunities,
                },
                drawing: FlyingDraw {
                    phase: if observed.draw_flicker_odd {
                        beam::FlickerPhase::Odd
                    } else {
                        beam::FlickerPhase::Even
                    },
                    view: &beam::DrawView::Normal(beam::ScreenOrigin(beam::Axes { x, y })),
                    batch: &batch,
                },
            })
            .map_err(|fault| super::production_failure(cut, fault))
    }
}

#[derive(Deserialize)]
pub(super) struct ExpectedCut {
    pub(super) number: u64,
    events: Vec<Event>,
    hit_actor_and_shot: Vec<[u8; 2]>,
    sprite: Option<Sprite>,
}

#[derive(Deserialize, PartialEq, Eq)]
enum Event {
    PowerBeamSpawn,
    Trail,
    Consume,
    Remove,
}

#[derive(Deserialize)]
struct Sprite {
    shot_slot: u8,
    picture: u16,
    screen_centre: [u16; 2],
    oam_index: u16,
    low: [u8; 4],
    high_pair: u8,
}

fn field(name: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: name.to_owned(),
        bytes,
    }
}

fn picture(slot: u8, id: &beam::PictureId, [x, y]: [u16; 2]) -> Vec<u8> {
    let mut bytes = vec![slot];
    bytes.extend(x.to_le_bytes());
    bytes.extend(y.to_le_bytes());
    bytes.extend(id.0.as_bytes());
    bytes.push(0);
    bytes
}

fn entry(index: u16, low: [u8; 4], high_pair: u8) -> Vec<u8> {
    let mut bytes = Vec::from(index.to_le_bytes());
    bytes.extend(low);
    bytes.push(high_pair);
    bytes
}

impl ExpectedCut {
    pub(super) fn load(root: &Path, file: &FixtureFile) -> Result<Vec<Self>, Outcome> {
        load(root, file, |record: &Self| record.number)
    }

    pub(super) fn read(
        tape: &[Self],
        wram: &[u8],
        content: &weapons::Content,
        cut: u64,
    ) -> Result<Vec<Observation>, Outcome> {
        let mut fields = weapons::expected(wram, content)?;
        fields.extend(at(tape, cut)?.outputs(content)?);
        Ok(fields)
    }

    pub(super) fn outputs(&self, content: &weapons::Content) -> Result<Vec<Observation>, Outcome> {
        let (intent, sprite) = if let Some(sprite) = &self.sprite {
            (
                picture(
                    sprite.shot_slot,
                    &content.picture(sprite.picture)?,
                    sprite.screen_centre,
                ),
                entry(sprite.oam_index, sprite.low, sprite.high_pair),
            )
        } else {
            (Vec::new(), Vec::new())
        };
        Ok(vec![
            field(
                "weapons.update-events",
                self.events
                    .iter()
                    .filter_map(|event| match event {
                        Event::PowerBeamSpawn => Some(0),
                        Event::Trail => Some(1),
                        Event::Consume | Event::Remove => None,
                    })
                    .collect(),
            ),
            field(
                "contact.receivers",
                self.hit_actor_and_shot.iter().flatten().copied().collect(),
            ),
            field("weapons.flying-pictures", intent),
            field("weapons.oam-insertion", sprite),
        ])
    }
}

const fn slot(slot: beam::ShotSlot) -> u8 {
    match slot {
        beam::ShotSlot::Slot0 => 0,
        beam::ShotSlot::Slot1 => 1,
        beam::ShotSlot::Slot2 => 2,
        beam::ShotSlot::Slot3 => 3,
        beam::ShotSlot::Slot4 => 4,
    }
}

pub(super) fn held(state: &beam::State) -> Vec<Observation> {
    let mut observations = weapons::initial(state);
    observations.extend(
        [
            "weapons.update-events",
            "contact.receivers",
            "weapons.flying-pictures",
            "weapons.oam-insertion",
        ]
        .map(|name| field(name, Vec::new())),
    );
    observations
}

pub(super) fn actual(
    state: &beam::State,
    output: &ShotDrawingOutputs,
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    let mut fields = weapons::actual(state, &output.weapons);
    let mut sprites = Vec::new();
    let first = usize::from(output.first_sprite);
    let end = usize::from(output.batch.sprites.0);
    if end < first {
        return Err(data_failure(Some(cut), "scoped sprite insertion wraps"));
    }
    for index in first..end {
        let outside = || data_failure(Some(cut), "scoped sprite insertion outside OAM");
        let parts = output.batch.oam.low.get(index).ok_or_else(outside)?;
        let high = output.batch.oam.high.get(index / 4).ok_or_else(outside)?;
        let high_pair = (high >> (index % 4).wrapping_mul(2)) & 3;
        sprites.extend(entry(
            u16::try_from(index).map_err(|_| data_failure(Some(cut), "sprite index"))?,
            [parts.x, parts.y, parts.tile, parts.attributes],
            high_pair,
        ));
    }
    fields.extend([
        field(
            "weapons.update-events",
            output
                .weapons
                .events
                .iter()
                .filter_map(|event| match event {
                    beam::Event::FireSound(_) => Some(0),
                    beam::Event::Trail { .. } => Some(1),
                    beam::Event::ImpactSound(_) => Some(2),
                    beam::Event::MissileDebit => Some(3),
                    beam::Event::ClearSelection => Some(4),
                    beam::Event::PuffRequest { .. } => Some(5),
                    beam::Event::Protection(_) | beam::Event::PendingAimConsumed => None,
                })
                .collect(),
        ),
        field(
            "contact.receivers",
            output
                .weapons
                .contacts
                .iter()
                .filter_map(|contact| Some([contact.opportunity.actor.0, slot(contact.consumed?)]))
                .flatten()
                .collect(),
        ),
        field(
            "weapons.flying-pictures",
            output
                .pictures
                .iter()
                .flat_map(|intent| {
                    picture(
                        slot(intent.slot),
                        &intent.picture,
                        [intent.screen.0.x, intent.screen.0.y],
                    )
                })
                .collect(),
        ),
        field("weapons.oam-insertion", sprites),
    ]);
    Ok(fields)
}
