//! Install collision and Samus's entry state before the intro's first room turn.

use super::samus::{self, Between, TurnContent, TurnInputs};
use crate::production::health_tail::HealthPin;
use crate::production::room_loading;
use crate::production::world::{Boundary, MovementWorld};
use crate::state::owned::PowerPin;
use canonical::Canonical;
use canonical::model::{Model, Pinned};
use core::cell::Cell as Flag;
use data::assets::intro::RoomCells;
use data::level::slope::Slopes;
use interface_hud::{Ammo, SelectableItem, Selection};
use samus_appearance::{Figure, Instruction, Playhead, Step};
use types::{Buttons, Pair, Parity, Pose, Position, Px, World};

/// The room installer's fixed catalogue rule.
pub const RULE: &str = "world-intro-rooms/ShowIntroRoom";

/// The intro's causal request to open a flashback room.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// Samus's fight on Zebes.
    Fight,
    /// Samus finding the baby.
    Discovery,
}

impl Start {
    const fn layout(self) -> (u16, usize, [u16; 2]) {
        match self {
            Self::Fight => (16, 14, [155, 115]),
            Self::Discovery => (32, 12, [376, 147]),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Room {
    width: u16,
    cells: Vec<data::level::Block>,
    world: MovementWorld,
}

impl Room {
    fn new(start: Start, cells: Vec<data::level::Block>, slopes: &Slopes) -> Result<Self, Fault> {
        let (width, rows, _) = start.layout();
        if usize::from(width).saturating_mul(rows) != cells.len() {
            return Err(Fault::Layout);
        }
        let kinds: Vec<_> = cells.iter().map(|block| block.kind).collect();
        let (blocks, lets_through) = room_loading::blocks(&kinds, width, slopes, |_| false)
            .map_err(|fault| Fault::Samus(super::Fault::Load(fault)))?;
        let world = MovementWorld::new(width, blocks, lets_through, Vec::new(), Boundary::Solid)
            .map_err(|_| Fault::Layout)?;
        Ok(Self {
            width,
            cells,
            world,
        })
    }
}

/// Checked cartridge content for both starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    fight: Room,
    discovery: Room,
}

impl Content {
    /// Convert the imported blocks using the ordinary room's collision types.
    ///
    /// # Errors
    /// Missing cells or invalid collision content.
    pub fn new(cells: RoomCells, slopes: &Slopes) -> Result<Self, Fault> {
        Ok(Self {
            fight: Room::new(Start::Fight, cells.fight, slopes)?,
            discovery: Room::new(Start::Discovery, cells.discovery, slopes)?,
        })
    }

    const fn room(&self, start: Start) -> &Room {
        match start {
            Start::Fight => &self.fight,
            Start::Discovery => &self.discovery,
        }
    }

    /// Logical grid dimensions, including the inherited area.
    #[must_use]
    pub const fn dimensions(&self, start: Start) -> [u16; 2] {
        [self.room(start).width, 16]
    }

    /// Source cells installed from the top left of the room.
    #[must_use]
    pub fn populated(&self, start: Start) -> &[data::level::Block] {
        &self.room(start).cells
    }
}

/// Number of backed cells in the intro's platform mirror.
pub const BACKED_CELLS: usize = 512;

/// The platform's full backed room image at the intro join.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Mirror {
    /// Logical grid dimensions.
    pub dimensions: [u16; 2],
    /// Cell words, including the retained tail.
    pub cells: Box<[u16; BACKED_CELLS]>,
    /// Per-cell marks independent of the loaded words.
    pub marks: Box<[u8; BACKED_CELLS]>,
}

impl Mirror {
    fn install(&mut self, start: Start, content: &Content) {
        self.dimensions = content.dimensions(start);
        for (word, cell) in self.cells.iter_mut().zip(content.populated(start)) {
            *word = cell.word();
        }
        if start == Start::Fight {
            self.marks.fill(0);
        }
    }
}

const STANDING_LEFT: Pose = Pose {
    body: types::Body::Upright,
    action: types::Action::Still,
    facing: types::Facing::Left,
    aim: types::Aim::Forward,
    gun: types::Gun::Stowed,
    legs: types::Legs::Together,
};

/// The existing actor owners and the platform mirrors read at room entry.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    /// Movement, figure and appearance.
    pub samus: samus::State,
    /// Health and protection.
    pub health: Model<HealthPin>,
    /// The existing shot slots.
    pub shots: Model<PowerPin>,
    /// Carried ammunition, selection, previous pose and centre, and the demo pad.
    pub carry: Model<CarryPin>,
}

impl Actor {
    pub(super) fn sample_body(&mut self, content: &super::MovementContent) -> Result<(), Fault> {
        let pose = self.samus.movement.0.pose;
        self.carry.0.radius = content
            .radius(pose)
            .ok_or(Fault::Samus(super::Fault::Body(pose)))?;
        Ok(())
    }

    pub(super) fn stand_left(
        &mut self,
        appearance: &super::AppearanceContent,
    ) -> Result<(), Fault> {
        let figure = Figure {
            pose: STANDING_LEFT,
            sprite: types::Sprite::Plain,
        };
        if self.carry.0.prior_pose != STANDING_LEFT {
            let Some(Instruction::Show(hold)) =
                appearance.get(figure).and_then(|clip| clip.at(Step::FIRST))
            else {
                return Err(Fault::Samus(super::Fault::Clip(figure)));
            };
            self.samus.appearance.0.animation.playhead = Playhead {
                step: Step::FIRST,
                time_left: hold.frames(),
            };
        }
        self.carry.0.older_pose = self.carry.0.prior_pose;
        self.carry.0.prior_pose = figure.pose;
        let body = &mut self.samus.movement.0;
        body.pose = figure.pose;
        body.choices.before = body.choices.latest;
        body.choices.latest = samus_movement::Chosen::Other;
        self.samus.figure = Model(figure);
        Ok(())
    }

    /// Her opening setup while hidden, with the chosen file's energy and the intro's loan.
    ///
    /// # Errors
    /// The fresh pose or the file's health cannot enter play.
    pub fn enter(energy: u16, equipment: screens_intro::Equipment) -> Result<Self, Fault> {
        let samus = super::start::fresh_at((0, 0)).map_err(Fault::Samus)?;
        let pose = samus.movement.0.pose;
        let centre = samus.movement.0.position;
        let fractions = enum_map::EnumMap::from_fn(|_| weapons_projectiles::Axes {
            x: weapons_projectiles::PixelFraction(0),
            y: weapons_projectiles::PixelFraction(0),
        });
        Ok(Self {
            samus,
            health: Model(super::start::new_file_health(energy).map_err(Fault::Samus)?),
            shots: Model(Box::new(weapons_projectiles::State::reset(fractions))),
            carry: Model(ActorCarry {
                missiles: equipment.missiles,
                selection: Selection::None,
                previous: centre,
                prior_pose: pose,
                older_pose: pose,
                radius: Pair {
                    x: core::num::Wrapping(0),
                    y: core::num::Wrapping(0),
                },
                demo: DemoPad::default(),
            }),
        })
    }
}

/// The saved representation of the actor's platform carry.
pub struct CarryPin;

impl Pinned for CarryPin {
    type State = ActorCarry;
    const NAME: &'static str = "intro-room-actor-carry";
    const SHAPE: u64 = 0xc1a8_39d8_bdaa_0d37;
}

/// Actor facts carried across the room join.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ActorCarry {
    /// Current missile supply.
    pub missiles: u16,
    /// Selected item.
    pub selection: Selection,
    /// The platform's previous centre.
    pub previous: Position,
    /// The platform's prior pose mirror.
    pub prior_pose: Pose,
    /// The platform's older pose mirror.
    pub older_pose: Pose,
    /// The platform's cached radii, updated on her first turn.
    pub radius: Pair<Px>,
    /// Held and new demo buttons.
    pub demo: DemoPad,
}

/// Held and newly pressed demo actions from the demo owner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DemoPad {
    /// Held actions.
    pub held: Buttons,
    /// New presses.
    pub pressed: Buttons,
}

/// The intro's installed room and the independently owned state it carries to its turn.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// State from the opening or from the previous room.
    pub actor: Actor,
    /// If provided by the platform, the complete backed grid.
    pub mirror: Option<Mirror>,
    phase: Option<Phase>,
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Pending(Start),
    Played(Start),
}

/// A handoff that cannot be played by the admitted production owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Room content failed to form its populated rectangle.
    Layout,
    /// The start cannot run while a turn is pending, or the first turn is already complete.
    Order,
    /// A first turn reached a cell outside the populated rectangle.
    OutsideRoom,
    /// This turn requires demo or encounter work beyond the quiet first turn.
    ActiveEncounter,
    /// The ordinary movement or appearance turn refused the inputs or content.
    Samus(super::Fault),
}

impl State {
    /// Carry the actor owners into the intro's room scheduler.
    #[must_use]
    pub const fn new(actor: Actor, mirror: Option<Mirror>) -> Self {
        Self {
            actor,
            mirror,
            phase: None,
        }
    }

    /// The room already installed, including while its first turn is pending.
    #[must_use]
    pub const fn active(&self) -> Option<Start> {
        match self.phase {
            Some(Phase::Pending(room) | Phase::Played(room)) => Some(room),
            None => None,
        }
    }

    /// Whether the room is installed and has yet to take its first turn.
    #[must_use]
    pub const fn pending(&self) -> bool {
        matches!(self.phase, Some(Phase::Pending(_)))
    }

    /// Install the requested room before the demo and movement run.
    ///
    /// # Errors
    /// An unconsumed start or a missing standing clip.
    pub fn start(
        &mut self,
        start: Start,
        content: &Content,
        appearance: &super::AppearanceContent,
    ) -> Result<(), Fault> {
        if self.pending() || self.active() == Some(start) {
            return Err(Fault::Order);
        }

        let actor = &mut self.actor;
        actor.stand_left(appearance)?;
        let (_, _, [x, y]) = start.layout();
        write_centre(&mut actor.samus.movement.0.position, x, y);
        write_centre(&mut actor.carry.0.previous, x, y);
        actor.carry.0.demo = DemoPad::default();
        actor.carry.0.selection = match start {
            Start::Fight => {
                actor.health.0.protection = samus_health::ProtectionFrames::ZERO;
                let movement = &mut actor.samus.movement.0;
                movement.knockback = samus_movement::Knockback::spent(movement.knockback.side);
                *actor.shots.0 = reset_shots(&actor.shots.0);
                Selection::Item(SelectableItem::Ammo(Ammo::Missiles))
            }
            Start::Discovery => Selection::None,
        };
        if let Some(mirror) = &mut self.mirror {
            mirror.install(start, content);
        }
        self.phase = Some(Phase::Pending(start));
        Ok(())
    }

    /// Play the single admitted quiet turn after setup. The caller supplies the live frame
    /// parity and demo pad and orders this before encounter contact.
    ///
    /// # Errors
    /// Setup is missing or already consumed, the demo or shots are active, or the ordinary
    /// turn cannot be played in the populated room.
    pub fn first_turn(
        &mut self,
        pad: DemoPad,
        parity: Parity,
        content: &Content,
        motion: &Motion<'_>,
    ) -> Result<FirstTurn, Fault> {
        if !self.pending() {
            return Err(Fault::Order);
        }
        if pad != DemoPad::default()
            || self.actor.shots.0.count.0 != 0
            || self.actor.shots.0.cooldown.0 != 0
            || self
                .actor
                .shots
                .0
                .slots
                .values()
                .any(|slot| slot.body().is_some())
        {
            return Err(Fault::ActiveEncounter);
        }
        self.actor.sample_body(motion.movement)?;
        self.actor_turn(pad, parity, content, motion, true)
    }

    /// Step motion and appearance after the demo has issued its controls.
    ///
    /// # Errors
    /// The room is absent, the selection is unsupported, or motion fails.
    pub(super) fn actor_turn(
        &mut self,
        pad: DemoPad,
        parity: Parity,
        content: &Content,
        motion: &Motion<'_>,
        controlled: bool,
    ) -> Result<FirstTurn, Fault> {
        let active = self.active().ok_or(Fault::Order)?;
        let actor = &mut self.actor;
        let world = Populated {
            world: &content.room(active).world,
            outside: Flag::new(false),
        };
        let mut activities = types::Set::default();
        activities[samus_appearance::Activity::Cinematic] = true;
        let energy = actor.health.0.energy.whole.get();
        let selected = match actor.carry.0.selection.item() {
            Some(SelectableItem::Ammo(Ammo::Missiles)) => samus_appearance::SelectedItem::Missiles,
            None => samus_appearance::SelectedItem::None,
            Some(_) => return Err(Fault::ActiveEncounter),
        };
        let (her, ()) = actor
            .samus
            .turn(
                TurnInputs {
                    held: pad.held,
                    pressed: pad.pressed,
                    parity,
                    activities,
                    items: motion.items,
                    energy,
                    liquid: samus_appearance::Liquid::NONE,
                    sounds_waiting: motion.sounds_waiting,
                    charge: motion.charge,
                    item_controls: if controlled {
                        samus_appearance::ItemControls::Ran(samus_appearance::ItemControlAnswer {
                            before: selected,
                            after: selected,
                            eligibility: samus_appearance::ControlEligibility::Eligible,
                        })
                    } else {
                        samus_appearance::ItemControls::Skipped
                    },
                },
                &world,
                TurnContent {
                    movement: motion.movement,
                    appearance: motion.appearance,
                    movement_rules: motion.movement_rules,
                    appearance_rules: motion.appearance_rules,
                },
                |_| {
                    Ok(Between {
                        played: (),
                        hit: None,
                        energy,
                    })
                },
            )
            .map_err(Fault::Samus)?;
        if world.outside.get() {
            return Err(Fault::OutsideRoom);
        }
        if her
            .outcome
            .as_ref()
            .is_some_and(|outcome| outcome.recorded_choice.is_some())
        {
            actor.carry.0.older_pose = actor.carry.0.prior_pose;
            actor.carry.0.prior_pose = her.samus.pose;
        }
        actor
            .samus
            .played(her.samus, &her, samus::jump_button(pad.held));
        actor.carry.0.demo = pad;
        self.phase = Some(Phase::Played(active));
        Ok(FirstTurn {
            movement: her.outcome,
            colours: her.colours,
            sounds: her.sounds,
        })
    }
}

/// Imported Samus content, tuning and external inputs at the join.
pub struct Motion<'a> {
    /// Pose bodies and cannon headings.
    pub movement: &'a super::MovementContent,
    /// Clips by figure and run pace.
    pub appearance: &'a super::AppearanceContent,
    /// Movement tuning.
    pub movement_rules: &'a samus_movement::Rules,
    /// Appearance tuning.
    pub appearance_rules: &'a samus_appearance::Rules,
    /// Equipped items from the intro loan.
    pub items: types::Set<types::Item>,
    /// Current sound occupancy read by appearance.
    pub sounds_waiting: samus_appearance::SoundsWaiting,
    /// Current charge read by appearance.
    pub charge: samus_appearance::Charge,
}

/// Outputs from the ordinary movement and appearance owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FirstTurn {
    /// What her movement produced for readers after it.
    pub movement: Option<samus_movement::Outcome>,
    /// The row her appearance asks the palette owner to set.
    pub colours: Option<types::RowColours>,
    /// The sound opportunities her appearance produced.
    pub sounds: Vec<samus_appearance::SoundOffer>,
}

fn write_centre(centre: &mut Position, x: u16, y: u16) {
    centre.x.0 = (u32::from(x) << 16) | (centre.x.0 & 0xffff);
    centre.y.0 = (u32::from(y) << 16) | (centre.y.0 & 0xffff);
}

fn reset_shots(shots: &weapons_projectiles::State) -> weapons_projectiles::State {
    use weapons_projectiles::{Axes, PixelFraction, ShotState};
    weapons_projectiles::State::reset(enum_map::EnumMap::from_fn(|slot| {
        match &shots.slots[slot] {
            ShotState::Free {
                retained_fraction, ..
            } => *retained_fraction,
            ShotState::Flying { body, .. } | ShotState::Impact { body } => {
                let fraction = |position: weapons_projectiles::FixedPosition| {
                    let [lo, hi, ..] = position.0.to_le_bytes();
                    PixelFraction(u16::from_le_bytes([lo, hi]))
                };
                Axes {
                    x: fraction(body.position.x),
                    y: fraction(body.position.y),
                }
            }
        }
    }))
}

/// Record any query that would need the unpopulated part of the room.
struct Populated<'a> {
    world: &'a MovementWorld,
    outside: Flag<bool>,
}

impl Populated<'_> {
    fn visit(&self, at: types::Cell) {
        if !self.world.contains(at) {
            self.outside.set(true);
        }
    }
}

impl World for Populated<'_> {
    fn block(&self, at: types::Cell) -> types::Block {
        self.visit(at);
        self.world.block(at)
    }

    fn lets_through(&self, at: types::Cell, way: types::Way) -> bool {
        self.visit(at);
        self.world.lets_through(at, way)
    }

    fn solid_enemies(&self) -> &[types::Hitbox] {
        self.world.solid_enemies()
    }
}

/// A room join with the live finished-work count carried across both cuts.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Scoped {
    /// The room and her actor owners.
    pub rooms: State,
    /// Finished-work count from the frame owner.
    pub finished_frames: u16,
}

/// Imported immutable content for a scoped pair of room cuts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedContent {
    /// Playable room geometry.
    pub rooms: Content,
    /// Imported movement bodies and cannon headings.
    pub movement: super::MovementContent,
    /// Imported appearance clips and paces.
    pub appearance: super::AppearanceContent,
}

/// Tuning used during the room's ordinary actor turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedRules {
    /// Movement rules.
    pub movement: samus_movement::Rules,
    /// Appearance rules.
    pub appearance: samus_appearance::Rules,
}

/// Scoped inputs from the intro, demo, audio and frame owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Opportunity {
    /// Start requested during this interval, if one occurs.
    pub start: Option<Start>,
    /// Whether the frame's work finished before this cut.
    pub finished: bool,
    /// Causally admitted demo input.
    pub pad: DemoPad,
    /// Equipped items.
    pub items: types::Set<types::Item>,
    /// Sound queue occupancy.
    pub sounds_waiting: samus_appearance::SoundsWaiting,
    /// Current weapon charge.
    pub charge: samus_appearance::Charge,
}

pub(crate) fn validate_pins() -> Result<(), canonical::model::Unpinned> {
    use crate::state::owned::{AppearancePin, MovementPin};
    use canonical::model::pin_holds;
    pin_holds::<CarryPin>()
        .and_then(|()| pin_holds::<HealthPin>())
        .and_then(|()| pin_holds::<PowerPin>())
        .and_then(|()| pin_holds::<MovementPin>())
        .and_then(|()| pin_holds::<AppearancePin>())
        .and_then(|()| pin_holds::<samus::FigurePin>())
        .and_then(|()| pin_holds::<samus::JumpButtonPin>())
}

impl Scoped {
    pub(crate) fn advance(
        &mut self,
        input: Opportunity,
        content: &ScopedContent,
        rules: &ScopedRules,
    ) -> Result<Option<FirstTurn>, Fault> {
        if let Some(start) = input.start {
            self.rooms
                .start(start, &content.rooms, &content.appearance)?;
        }
        if !input.finished {
            return Ok(None);
        }
        let parity = if self.finished_frames & 1 == 0 {
            Parity::Even
        } else {
            Parity::Odd
        };
        let turn = self.rooms.first_turn(
            input.pad,
            parity,
            &content.rooms,
            &Motion {
                movement: &content.movement,
                appearance: &content.appearance,
                movement_rules: &rules.movement,
                appearance_rules: &rules.appearance,
                items: input.items,
                sounds_waiting: input.sounds_waiting,
                charge: input.charge,
            },
        )?;
        self.finished_frames =
            super::frame::finished(self.finished_frames, super::frame::finishing(true));
        Ok(Some(turn))
    }
}

#[cfg(test)]
mod tests {
    use super::{BACKED_CELLS, Content, Mirror, RoomCells, Slopes, Start};
    use data::level::Block;

    #[test]
    fn each_room_keeps_the_tail_and_only_the_fight_clears_marks() {
        let content = Content::new(
            RoomCells {
                fight: vec![Block::new(1, 0); 16 * 14],
                discovery: vec![Block::new(2, 0); 32 * 12],
            },
            &Slopes::default(),
        )
        .unwrap();

        for (start, height, word, mark) in [(Start::Fight, 14, 1, 0), (Start::Discovery, 12, 2, 7)]
        {
            let mut mirror = Mirror {
                dimensions: [0; 2],
                cells: Box::new([9; BACKED_CELLS]),
                marks: Box::new([7; BACKED_CELLS]),
            };
            mirror.install(start, &content);
            let [width, logical_rows] = mirror.dimensions;
            assert_eq!(logical_rows, 16);
            let (populated, tail) = mirror.cells.split_at(usize::from(width) * height);
            assert_eq!(populated.len(), content.populated(start).len());
            assert!(populated.iter().all(|&cell| cell == word));
            assert!(tail.iter().all(|&cell| cell == 9));
            assert!(mirror.marks.iter().all(|&actual| actual == mark));
        }
    }

    #[test]
    fn intro_room_state_pins_hold() {
        assert_eq!(super::validate_pins(), Ok(()));
    }
}
