//! The chosen file's actor and controls across the opening and the playable intro room.

use super::super::intro::Fault;
use super::super::{flashback, intro_missiles as missiles, intro_rooms as rooms};
use super::intro_cannon::{Cannon, VIEW};
use super::{ActorsContent, Pressed};
use crate::intent::SpriteDraw;
use crate::production::playback::{self, Playback};
use enum_map::EnumMap;
use screens_intro::{ButtonSetting, Equipment, FightReply, FrameAwaitingReply};
use types::options::Settings;
use types::save_files::{
    Ammunition, Control, ControlBindings, Gear, PadButton, PlayerProfile, Supply,
};
use types::{Parity, Parts, RowColours};

/// The starting quantities handed to the intro by the file selection and options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorFile {
    /// Her chosen file's energy and capacity.
    pub energy: Supply,
    /// Her chosen file's reserve energy and capacity.
    pub reserves: Supply,
    /// All gear the file owns.
    pub owned: Gear,
    /// Gear in use when the file began.
    pub gear: Gear,
    /// The controls to restore when the intro returns its loan.
    pub buttons: ControlBindings,
    /// Ammunition before the loan.
    pub ammunition: EnumMap<Ammunition, Supply>,
}

impl From<&PlayerProfile> for ActorFile {
    fn from(profile: &PlayerProfile) -> Self {
        Self {
            energy: profile.energy,
            reserves: profile.reserves,
            owned: profile.owned,
            gear: profile.in_use,
            buttons: profile.controls,
            ammunition: profile.ammunition,
        }
    }
}

/// The physical controls used for the intro's recorded scenes.
#[must_use]
pub fn intro_buttons() -> ControlBindings {
    enum_map::enum_map! {
        Control::Shoot => PadButton::X,
        Control::Jump => PadButton::A,
        Control::Run => PadButton::B,
        Control::ItemCancel => PadButton::Y,
        Control::ItemSelect => PadButton::Select,
        Control::AimUp => PadButton::R,
        Control::AimDown => PadButton::L,
    }
}

/// The actor, her loan and the encounter currently selected by the scene.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actors {
    /// File and player preferences retained while the demo owns the controls.
    pub file: ActorFile,
    /// The options as the player left them on the way into the intro.
    pub settings: Settings,
    /// Her motion, health, ammunition and appearance once the intro creates her.
    pub rooms: Option<rooms::State>,
    /// The current missile capacity.
    pub missile_capacity: u16,
    /// The active control bindings.
    pub buttons: ControlBindings,
    /// The fight when it has begun.
    pub battle: Option<Battle>,
}

/// The two owners that take a turn on either side of Samus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Battle {
    /// The recorded controls.
    pub playback: Playback,
    cannon: Cannon,
    /// The encounter pictures after the two missile visits.
    pub encounter_draw: SpriteDraw,
    /// Both prepared missile and exhaust draws in their production order.
    pub weapon_draws: [SpriteDraw; 2],
    /// The encounter situated in the room's background.
    pub encounter: flashback::EncounterState,
}

impl Battle {
    fn encounter_turn(
        &mut self,
        frame: &FrameAwaitingReply<'_>,
        parity: Parity,
        room: &rooms::State,
        candidates: Vec<weapons_projectiles::MissileCandidate>,
    ) -> flashback::Effects {
        let play = flashback::RoomPlay {
            samus_x: room.actor.samus.movement.0.position.x.pixel(),
            shots: candidates
                .into_iter()
                .map(|candidate| flashback::ShotCandidate {
                    id: flashback::ShotId(missiles::slot_id(candidate.slot)),
                    kind: flashback::ShotKind::Missile,
                    x: core::num::Wrapping(candidate.whole_x),
                })
                .collect(),
        };
        flashback::turn(
            &mut self.encounter,
            flashback::Rules::REFERENCE,
            match parity {
                Parity::Even => Parity::Odd,
                Parity::Odd => Parity::Even,
            },
            if frame
                .requests()
                .any(|request| request == screens_intro::FlashbackRequest::HideFightSamusAndShots)
            {
                flashback::ReturnDissolve::ReachesZero
            } else {
                flashback::ReturnDissolve::Continuing
            },
            &play,
        )
    }

    /// The live ammunition, selected item and incoming-shot protection after room work.
    #[must_use]
    pub fn supplies(&self, actor: &rooms::Actor) -> missiles::Supplies {
        self.cannon.supplies(actor)
    }
}

impl Actors {
    pub(super) fn waiting(file: ActorFile, settings: Settings) -> Self {
        Self {
            file,
            settings,
            rooms: None,
            missile_capacity: file.ammunition[Ammunition::Missiles].maximum,
            buttons: file.buttons,
            battle: None,
        }
    }

    /// Energy and capacity. The room's health owner supplies the current value after setup.
    #[must_use]
    pub fn energy(&self) -> Supply {
        Supply {
            current: self
                .rooms
                .as_ref()
                .map_or(self.file.energy.current, |room| {
                    room.actor.health.0.energy.whole.into()
                }),
            maximum: self.file.energy.maximum,
        }
    }

    /// Ammunition and capacities with the intro's live missile supply while the loan is active.
    #[must_use]
    pub fn ammunition(&self, kind: Ammunition) -> Supply {
        let supply = self.file.ammunition[kind];
        if kind == Ammunition::Missiles {
            Supply {
                current: self
                    .rooms
                    .as_ref()
                    .map_or(supply.current, |room| room.actor.carry.0.missiles),
                maximum: self.missile_capacity,
            }
        } else {
            supply
        }
    }

    pub(super) fn equip(&mut self, equipment: Equipment) -> Result<(), Fault> {
        if let Some(room) = &mut self.rooms {
            room.actor.carry.0.missiles = equipment.missiles;
        } else {
            self.rooms = Some(rooms::State::new(
                rooms::Actor::enter(self.file.energy.current, equipment)
                    .map_err(Fault::IntroRoom)?,
                None,
            ));
        }
        self.buttons = match equipment.buttons {
            ButtonSetting::Default => intro_buttons(),
            ButtonSetting::PlayersOwn => self.file.buttons,
        };
        self.missile_capacity = equipment.missile_capacity;
        Ok(())
    }

    pub(super) fn start(&mut self, height: i16, content: &ActorsContent) -> Result<(), Fault> {
        let room = self.rooms.as_mut().ok_or(Fault::ActorUnstarted)?;
        room.start(
            rooms::Start::Fight,
            &content.room.rooms,
            &content.room.appearance,
        )
        .map_err(Fault::IntroRoom)?;
        let background = flashback::RoomBackgroundY(height);
        let mut encounter = flashback::EncounterState::before_encounter(background);
        flashback::start(&mut encounter, background);
        let mut playback = Playback::new(content.pad.clone());
        if playback.start(playback::SceneStart::Battle) != Some(playback::Handoff::StartBattle) {
            return Err(Fault::IntroControls);
        }
        self.battle = Some(Battle {
            playback,
            cannon: Cannon::default(),
            encounter_draw: SpriteDraw::default(),
            weapon_draws: Default::default(),
            encounter,
        });
        Ok(())
    }

    pub(super) fn turn(
        &mut self,
        frame: &FrameAwaitingReply<'_>,
        parity: Parity,
        movie: Pressed,
        waiting: samus_appearance::SoundsWaiting,
        content: &ActorsContent,
    ) -> Result<(FightReply, Option<RowColours>), Fault> {
        let Some(battle) = &mut self.battle else {
            return Ok((FightReply::Fighting, None));
        };
        let room = self.rooms.as_mut().ok_or(Fault::ActorUnstarted)?;
        let x = room.actor.samus.movement.0.position.x.pixel();
        let turn = battle.playback.turn(
            playback::WholeX(x.0),
            playback::CrossfadeCount(frame.state().dissolve),
        );
        let pad = turn
            .pad
            .filter(|_| turn.authority == playback::Authority::Controlled)
            .ok_or(Fault::IntroControls)?;
        let pad = rooms::DemoPad {
            held: playback::buttons(pad.current.held),
            pressed: playback::buttons(pad.current.pressed),
        };
        let motion = rooms::Motion {
            movement: &content.room.movement,
            appearance: &content.room.appearance,
            movement_rules: &content.rules.movement,
            appearance_rules: &content.rules.appearance,
            items: self.file.gear.items,
            sounds_waiting: waiting,
            charge: samus_appearance::Charge::new(0).ok_or(Fault::IntroControls)?,
        };
        if turn.handoff.is_some() {
            return Err(Fault::IntroControls);
        }
        let (input, eligible) = battle.cannon.inputs(
            &room.actor,
            pad,
            self.buttons[Control::Shoot],
            weapons_projectiles::UpdateMode::CannonAndShots,
            &content.missiles,
        )?;
        let motion = room
            .actor_turn(pad, parity, &content.room.rooms, &motion)
            .map_err(Fault::IntroRoom)?;
        let candidates =
            battle
                .cannon
                .update(&mut room.actor, &input, eligible, &content.missiles)?;
        if let Some(outcome) = &motion.movement {
            battle.cannon.moved(outcome);
        }
        let effects = battle.encounter_turn(frame, parity, room, candidates);
        if let Some(hit) = effects.missile_impact {
            let slot = missiles::slot_from_id(hit.0).ok_or(Fault::IntroRoomTurn)?;
            if !missiles::hit(&mut room.actor.shots.0, slot, &content.missiles) {
                return Err(Fault::IntroRoomTurn);
            }
        }
        room.actor.health.0.protection = room
            .actor
            .health
            .0
            .play_frame(
                samus_health::Impact::Absent,
                &samus_health::Rules::REFERENCE,
            )
            .state
            .protection;
        if let Some(ring) = effects.ring_knockback {
            room.actor.health.0.protection =
                samus_health::ProtectionFrames::new(ring.invincible_beats)
                    .ok_or(Fault::IntroRoomTurn)?;
            let push = &mut room.actor.samus.movement.0.knockback;
            push.side = ring.direction;
            push.time_left =
                types::PushFrames::new(ring.knockback_beats).ok_or(Fault::IntroRoomTurn)?;
        }
        battle.weapon_draws =
            missiles::draw_twice(&mut room.actor.shots.0, VIEW, &content.missiles)
                .map_err(Fault::IntroWeapon)?;
        battle.encounter_draw = flashback::draw(
            &battle.encounter,
            &content.encounter,
            types::Pair {
                x: core::num::Wrapping(VIEW.0.x),
                y: core::num::Wrapping(VIEW.0.y),
            },
        );
        battle.cannon.drawn(movie);
        Ok((
            if effects.intro == Some(flashback::IntroRequest::CuePageTwo) {
                FightReply::ReadyToReturn
            } else {
                FightReply::Fighting
            },
            effects
                .palette
                .map(|flash| flashback::flash_colours(&content.encounter, flash)),
        ))
    }
}
