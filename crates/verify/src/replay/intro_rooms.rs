//! Independent one-origin checks for the intro's interrupted setup and completed first turn.

use super::compare::Observation;
use super::index::{
    Case, CaseKind, ContentProvider, ExpectedProvider, ObservationId, Responsibility,
};
use super::played::platform::{Controller, finished, polled};
use super::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, record,
};
use super::result::Outcome;
use super::{Sources, appearance, data_failure, health_tail, movement, production_failure};
use crate::cache::Cache;
use crate::lock::{Lock, RowId};
use crate::source::{self, WramWord};
use canonical::model::Model;
use core::num::Wrapping;
use data::assets::samus::{
    key::{Dress, SpinSprite, Suit},
    map,
};
use data::pose::PoseId;
use data::rom::Rom;
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
    ProductionSeed, played::intro_rooms as rooms,
};
use game::state::owned::OwnedState;
use interface_hud::{Ammo, SelectableItem, Selection};
use types::{Pair, Pose, Position};
use weapons_projectiles as beam;

const FIGHT_ROW: &str = "any-intro-0568";
const ROOMS_ROW: &str = "any-intro-2219";
const FIGHT_ORIGIN: u64 = 1650;
const DISCOVERY_ORIGIN: u64 = 1297;

pub(super) const GRID_FIELDS: usize = 2 + rooms::BACKED_CELLS * 2;
pub(super) const ACTOR_FIELDS: usize = 18;

fn recipe(case: &Case) -> Option<(u64, rooms::Start, u64)> {
    use super::index::Start::CacheCut;
    match (case.row.as_str(), &case.start, case.end.cut_inclusive) {
        (
            FIGHT_ROW,
            CacheCut {
                frame: FIGHT_ORIGIN,
            },
            1652,
        ) => Some((FIGHT_ORIGIN, rooms::Start::Fight, 0)),
        (
            ROOMS_ROW,
            CacheCut {
                frame: DISCOVERY_ORIGIN,
            },
            1299,
        ) => Some((DISCOVERY_ORIGIN, rooms::Start::Discovery, 1298)),
        _ => None,
    }
}

pub(super) fn valid(case: &Case) -> bool {
    use ObservationId as O;
    recipe(case).is_some()
        && case.kind == CaseKind::Scoped
        && case.responsibilities == [Responsibility::IntroRooms]
        && case.rules == [rooms::RULE]
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.fields == [O::IntroRoomGrid, O::IntroRoomActor, O::IntroRoomClock]
        && case.expectations.outputs.is_empty()
        && case.checkpoints
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
}

pub(super) struct Inputs {
    first: u64,
    setup: rooms::Opportunity,
    completion: rooms::Opportunity,
}

impl Inputs {
    pub(super) fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        let input = if cut == self.first {
            self.setup
        } else {
            self.completion
        };
        production
            .advance(AdvanceInput::IntroRooms(input))
            .map_err(|fault| production_failure(cut, fault))
    }
}

pub(super) struct Expected {
    cache: Cache,
    first: u64,
    first_record: u64,
}

impl Expected {
    pub(super) fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        let number = self
            .first_record
            .saturating_add(cut.saturating_sub(self.first));
        let record = record(&self.cache, number)?;
        let wram = &record.wram;
        let mut fields = grid(&mirror(wram)?);
        fields.extend(ActorView::read(wram)?.observations());
        fields.push(clock(word(wram, source::NMI_FRAME_COUNTER_WORD)?));
        Ok(ExpectedCut {
            semantics: fields.clone(),
            semantic_count: fields.len(),
            fields,
            output_count: 0,
            replies: 0,
        })
    }
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let (origin, start, first_record) =
        recipe(case).ok_or_else(|| data_failure(None, "intro room join recipe"))?;
    let lock = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "record lock file"))?;
    let lock = Lock::parse(&lock).map_err(|_| data_failure(None, "record lock schema"))?;
    let cache = |id: &str| {
        let row = lock
            .row(&RowId(id.into()))
            .ok_or_else(|| data_failure(None, "intro room lock row"))?;
        Cache::open(&sources.cache, row).map_err(|_| data_failure(None, "intro room cache pins"))
    };
    let origin_cache = cache(&case.row)?;
    let seed = record(&origin_cache, origin)?;
    let input_cache = if case.row == ROOMS_ROW {
        origin_cache
    } else {
        cache(ROOMS_ROW)?
    };
    let pending = record(&input_cache, first_record)?;
    let done = record(&input_cache, first_record.saturating_add(1))?;
    if [pending.video_frame, done.video_frame]
        != [
            seed.video_frame.wrapping_add(1),
            seed.video_frame.wrapping_add(2),
        ]
    {
        return Err(data_failure(None, "intro room consecutive video cuts"));
    }
    let previous = &seed.wram;
    if start == rooms::Start::Fight {
        let latch = Controller::seeded(previous)?;
        if !polled(&seed) || latch.held != seed.pad.0 || latch.pressed == 0 {
            return Err(data_failure(Some(origin), "intro fight latched press"));
        }
    }
    let clock = word(previous, source::NMI_FRAME_COUNTER_WORD)?;
    let bytes = std::fs::read(&sources.rom).map_err(|_| data_failure(None, "player ROM file"))?;
    let rom = Rom::new(bytes).map_err(|_| data_failure(None, "player ROM pin"))?;
    let (content, appearance) = room_content(&sources.rom, &rom)?;
    let actor = actor(previous, &appearance)?;
    let production_content = ProductionContent::IntroRooms(Box::new(content));
    let rules = ProductionRules::IntroRooms(Box::new(rooms::ScopedRules {
        movement: samus_movement::Rules::default(),
        appearance: samus_appearance::Rules::VANILLA,
    }));
    let state = rooms::Scoped {
        rooms: rooms::State::new(actor, Some(mirror(previous)?)),
        finished_frames: clock,
    };
    let seeded = ProductionSeed::IntroRooms(Box::new(state));
    let production = Production::admit(seeded, production_content.clone(), rules.clone())
        .map_err(|_| data_failure(Some(origin), "intro room production admission"))?;
    let waiting = byte(previous, 0x0644)?.wrapping_sub(byte(previous, 0x0645)?) & 15;
    let setup = rooms::Opportunity {
        start: Some(start),
        finished: finished(&pending),
        pad: rooms::DemoPad::default(),
        items: movement::items(word(previous, source::EQUIPPED_ITEMS)?),
        sounds_waiting: samus_appearance::SoundsWaiting::new(waiting)
            .ok_or_else(|| data_failure(Some(origin), "appearance sound queue count"))?,
        charge: appearance::charge(previous)?,
    };
    let completion = rooms::Opportunity {
        start: None,
        finished: finished(&done),
        ..setup
    };
    let first = origin.saturating_add(1);
    Ok(AdmittedCase {
        production,
        content: production_content,
        rules,
        actual: ActualReader::IntroRooms,
        schedule: vec![first, first.saturating_add(1)],
        inputs: InputReader::IntroRooms(Inputs {
            first,
            setup,
            completion,
        }),
        expected: ExpectedReader::IntroRooms(Box::new(Expected {
            cache: cache(ROOMS_ROW)?,
            first,
            first_record,
        })),
    })
}

/// The playable rectangles and shared movement/appearance tables from the cartridge.
pub(super) fn room_content(
    path: &std::path::Path,
    rom: &Rom,
) -> Result<(rooms::ScopedContent, appearance::Content), Outcome> {
    let raw = import::intro::room_cells(rom.lorom())
        .map_err(|_| data_failure(None, "intro room cell import"))?;
    let slopes = data::level::slope::Slopes::load(rom.lorom())
        .map_err(|_| data_failure(None, "slope content"))?;
    let rooms = rooms::Content::new(raw, &slopes)
        .map_err(|_| data_failure(None, "intro room collision content"))?;
    let movement = movement::Content::load(path)?;
    let appearance = appearance::Content::load(path)?;
    Ok((
        rooms::ScopedContent {
            rooms,
            movement: movement.movement,
            appearance: appearance.appearance.clone(),
        },
        appearance,
    ))
}

pub(super) fn actor(wram: &[u8], content: &appearance::Content) -> Result<rooms::Actor, Outcome> {
    let observed = ActorView::read(wram)?;
    let samus = super::played::samus::seed(wram, content)?;
    let mut shots = beam::State::reset(enum_map::EnumMap::from_fn(|_| beam::Axes {
        x: beam::PixelFraction(0),
        y: beam::PixelFraction(0),
    }));
    shots.cooldown = beam::FrameWord(observed.cooldown);
    shots.count = beam::ShotCount(observed.shots);
    for (index, slot) in shots.slots.values_mut().enumerate() {
        let w = |at: usize| {
            raw_word(
                wram,
                at.saturating_add(index.saturating_mul(2)),
                "intro room prior shot slot",
            )
        };
        if w(0x0c40)? != 0 {
            return Err(Outcome::Unsupported {
                cut: None,
                dependency: "intro room active prior shot".into(),
            });
        }
        *slot = beam::ShotState::Free {
            trail: beam::FrameWord(w(0x0c90)?),
            retained_fraction: beam::Axes {
                x: beam::PixelFraction(w(0x0b8c)?),
                y: beam::PixelFraction(w(0x0ba0)?),
            },
        };
    }
    Ok(rooms::Actor {
        samus,
        health: Model(health_tail::decode(wram, None)?),
        shots: Model(Box::new(shots)),
        carry: Model(rooms::ActorCarry {
            missiles: observed.missiles,
            selection: match observed.selected {
                0 => Selection::None,
                1 => Selection::Item(SelectableItem::Ammo(Ammo::Missiles)),
                _ => return Err(data_failure(None, "intro room prior item selection")),
            },
            previous: observed.previous,
            prior_pose: pose(observed.prior_pose)?,
            older_pose: pose(observed.older_pose)?,
            radius: observed.radius,
            demo: observed.demo,
        }),
    })
}

fn word(wram: &[u8], at: WramWord) -> Result<u16, Outcome> {
    at.read(wram).map_err(|_| data_failure(None, at.name))
}

fn raw_word(wram: &[u8], at: usize, name: &str) -> Result<u16, Outcome> {
    wram.get(at..)
        .and_then(<[u8]>::first_chunk::<2>)
        .copied()
        .map(u16::from_le_bytes)
        .ok_or_else(|| data_failure(None, name))
}

fn byte(wram: &[u8], at: usize) -> Result<u8, Outcome> {
    wram.get(at)
        .copied()
        .ok_or_else(|| data_failure(None, "intro room record byte"))
}

fn pose(id: u16) -> Result<Pose, Outcome> {
    map::key_of(PoseId(id)).ok_or_else(|| data_failure(None, "intro room prior pose"))
}

fn pose_id(pose: Pose) -> Result<u16, Outcome> {
    map::id_of(
        pose,
        Dress {
            spin: SpinSprite::Plain,
            suit: Suit::First,
        },
    )
    .map(|id| id.0)
    .ok_or_else(|| data_failure(None, "intro room played pose"))
}

fn mirror(wram: &[u8]) -> Result<rooms::Mirror, Outcome> {
    let dimensions = [
        word(wram, source::ROOM_WIDTH_IN_BLOCKS)?,
        word(wram, source::ROOM_HEIGHT_IN_BLOCKS)?,
    ];
    let raw = source::LEVEL_DATA
        .read(wram)
        .map_err(|_| data_failure(None, "level data mirror"))?;
    let mut cells = Box::new([0; rooms::BACKED_CELLS]);
    for (cell, [lo, hi]) in cells.iter_mut().zip(raw.as_chunks::<2>().0) {
        *cell = u16::from_le_bytes([*lo, *hi]);
    }
    let marks = source::BTS
        .read(wram)
        .ok()
        .and_then(<[u8]>::first_chunk::<{ rooms::BACKED_CELLS }>)
        .ok_or_else(|| data_failure(None, "cell mark mirror"))?;
    Ok(rooms::Mirror {
        dimensions,
        cells,
        marks: Box::new(*marks),
    })
}

fn grid(mirror: &rooms::Mirror) -> Vec<Observation> {
    let [width, height] = mirror.dimensions;
    let mut fields = vec![
        Observation::quantity("intro-room.width", i64::from(width)),
        Observation::quantity("intro-room.height", i64::from(height)),
    ];
    for (index, (&cell, &mark)) in mirror.cells.iter().zip(mirror.marks.iter()).enumerate() {
        fields.push(Observation::quantity(
            format!("intro-room.cell.{index}"),
            i64::from(cell),
        ));
        fields.push(Observation::quantity(
            format!("intro-room.mark.{index}"),
            i64::from(mark),
        ));
    }
    fields
}

fn clock(count: u16) -> Observation {
    Observation::quantity("intro-room.finished-frames", i64::from(count))
}

struct ActorView {
    centre: Position,
    previous: Position,
    pose: u16,
    prior_pose: u16,
    older_pose: u16,
    radius: types::Radius,
    step: u16,
    hold: u16,
    selected: u16,
    missiles: u16,
    energy: u16,
    demo: rooms::DemoPad,
    cooldown: u16,
    shots: u16,
}

impl ActorView {
    fn read(wram: &[u8]) -> Result<Self, Outcome> {
        let w = |at| raw_word(wram, at, "intro room actor");
        let position = |at: usize| {
            Ok(Position {
                x: types::fixed(Wrapping(w(at)?), w(at.saturating_add(2))?),
                y: types::fixed(Wrapping(w(at.saturating_add(4))?), w(at.saturating_add(6))?),
            })
        };
        Ok(Self {
            centre: position(0x0af6)?,
            previous: position(0x0b10)?,
            pose: w(0x0a1c)?,
            prior_pose: w(0x0a20)?,
            older_pose: w(0x0a24)?,
            radius: Pair {
                x: Wrapping(w(0x0afe)?),
                y: Wrapping(w(0x0b00)?),
            },
            step: w(0x0a96)?,
            hold: w(0x0a94)?,
            selected: w(0x09d2)?,
            missiles: w(0x09c6)?,
            energy: w(0x09c2)?,
            demo: rooms::DemoPad {
                held: movement::pad(w(0x0a84)?),
                pressed: movement::pad(w(0x0a86)?),
            },
            cooldown: w(0x0ccc)?,
            shots: w(0x0cce)?,
        })
    }

    fn observations(&self) -> [Observation; ACTOR_FIELDS] {
        [
            ("centre.x", i64::from(self.centre.x.0)),
            ("centre.y", i64::from(self.centre.y.0)),
            ("previous.x", i64::from(self.previous.x.0)),
            ("previous.y", i64::from(self.previous.y.0)),
            ("pose", i64::from(self.pose)),
            ("prior-pose", i64::from(self.prior_pose)),
            ("older-pose", i64::from(self.older_pose)),
            ("radius.x", i64::from(self.radius.x.0)),
            ("radius.y", i64::from(self.radius.y.0)),
            ("clip-step", i64::from(self.step)),
            ("clip-hold", i64::from(self.hold)),
            ("selected", i64::from(self.selected)),
            ("missiles", i64::from(self.missiles)),
            ("energy", i64::from(self.energy)),
            ("demo-held", i64::from(self.demo.held.0)),
            ("demo-pressed", i64::from(self.demo.pressed.0)),
            ("shot-cooldown", i64::from(self.cooldown)),
            ("shot-count", i64::from(self.shots)),
        ]
        .map(|(field, value)| Observation::quantity(format!("intro-room.actor.{field}"), value))
    }
}

pub(super) fn actual(production: &Production) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::IntroRooms(state) = production.state() else {
        return Err(data_failure(None, "intro room owned scope"));
    };
    let mirror = state
        .rooms
        .mirror
        .as_ref()
        .ok_or_else(|| data_failure(None, "intro room mirror scope"))?;
    let mut fields = grid(mirror);
    fields.extend(actor_fields(&state.rooms.actor)?);
    fields.push(clock(state.finished_frames));
    Ok(fields)
}

#[cfg(test)]
pub(super) fn discovery_actor(wram: &[u8]) -> Result<(Position, rooms::DemoPad), Outcome> {
    let recorded = ActorView::read(wram)?;
    Ok((recorded.centre, recorded.demo))
}

fn actor_fields(actor: &rooms::Actor) -> Result<[Observation; ACTOR_FIELDS], Outcome> {
    let carry = &actor.carry.0;
    let playhead = actor.samus.appearance.0.animation.playhead;
    let selected = match carry.selection {
        Selection::None => 0,
        Selection::Item(SelectableItem::Ammo(Ammo::Missiles)) => 1,
        Selection::Item(_) | Selection::OneShot(_) => {
            return Err(data_failure(None, "intro room played item selection"));
        }
    };
    Ok(ActorView {
        centre: actor.samus.movement.0.position,
        previous: carry.previous,
        pose: pose_id(actor.samus.movement.0.pose)?,
        prior_pose: pose_id(carry.prior_pose)?,
        older_pose: pose_id(carry.older_pose)?,
        radius: carry.radius,
        step: u16::from(playhead.step.0),
        hold: u16::from(playhead.time_left.0),
        selected,
        missiles: carry.missiles,
        energy: actor.health.0.energy.whole.get(),
        demo: carry.demo,
        cooldown: actor.shots.0.cooldown.0,
        shots: actor.shots.0.count.0,
    }
    .observations())
}

/// Independent actor and loan observations from the power-on run.
pub(super) fn opening_fields(
    actors: &game::production::played::front_end::Actors,
    wram: &[u8],
) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
    use types::save_files::Ammunition;

    let raw = |address| raw_word(wram, address, "intro actor entry");
    let quantity = Observation::quantity;
    let started = raw(0x0a44)? != 0;
    let mut expected = vec![quantity("intro-actor.started", i64::from(started))];
    let mut actual = vec![quantity(
        "intro-actor.started",
        i64::from(actors.rooms.is_some()),
    )];
    if started {
        expected.push(quantity("intro-actor.protection", i64::from(raw(0x18a8)?)));
        expected.push(quantity("intro-actor.push-hold", i64::from(raw(0x18aa)?)));
        expected.extend(ActorView::read(wram)?.observations());
        expected.push(quantity(
            "intro-actor.cannon-delay",
            2_i64.saturating_sub(i64::from(raw(0x0aaa)?)),
        ));
        expected.push(recorded_controls(
            wram,
            "intro-actor.saved-buttons",
            0x0d82,
        )?);
    }
    if let Some(room) = &actors.rooms {
        actual.push(quantity(
            "intro-actor.protection",
            i64::from(room.actor.health.0.protection.get()),
        ));
        actual.push(quantity(
            "intro-actor.push-hold",
            i64::from(room.actor.samus.movement.0.knockback.time_left.frames()),
        ));
        actual.extend(actor_fields(&room.actor)?);
        actual.push(quantity(
            "intro-actor.cannon-delay",
            i64::from(room.actor.samus.appearance.0.cannon.remaining()),
        ));
        actual.push(played_controls(
            "intro-actor.saved-buttons",
            &actors.file.buttons,
        ));
    }
    let health = actors.energy();
    let missiles = actors.ammunition(Ammunition::Missiles);
    let super_missiles = actors.ammunition(Ammunition::SuperMissiles);
    let power_bombs = actors.ammunition(Ammunition::PowerBombs);
    let reserves = actors.file.reserves;
    let [owned_items, owned_beams] = game::exact::save_files::gear_words(&actors.file.owned);
    let [equipped_items, equipped_beams] = game::exact::save_files::gear_words(&actors.file.gear);
    for (field, address, value) in [
        ("intro-actor.current-energy", 0x09c2, health.current),
        ("intro-actor.energy-capacity", 0x09c4, health.maximum),
        ("intro-actor.current-missiles", 0x09c6, missiles.current),
        ("intro-actor.missile-capacity", 0x09c8, missiles.maximum),
        ("intro-actor.super-missiles", 0x09ca, super_missiles.current),
        (
            "intro-actor.super-missile-capacity",
            0x09cc,
            super_missiles.maximum,
        ),
        ("intro-actor.power-bombs", 0x09ce, power_bombs.current),
        (
            "intro-actor.power-bomb-capacity",
            0x09d0,
            power_bombs.maximum,
        ),
        ("intro-actor.reserve-energy", 0x09d6, reserves.current),
        ("intro-actor.reserve-capacity", 0x09d4, reserves.maximum),
        ("intro-actor.owned-items", 0x09a4, owned_items),
        ("intro-actor.equipped-items", 0x09a2, equipped_items),
        ("intro-actor.owned-beams", 0x09a8, owned_beams),
        ("intro-actor.equipped-beams", 0x09a6, equipped_beams),
    ] {
        expected.push(quantity(field, i64::from(raw(address)?)));
        actual.push(quantity(field, i64::from(value)));
    }
    expected.push(quantity(
        "intro-actor.moonwalk",
        i64::from(raw(0x09e4)? != 0),
    ));
    actual.push(quantity(
        "intro-actor.moonwalk",
        i64::from(actors.settings.moonwalk),
    ));
    expected.push(recorded_controls(wram, "intro-actor.buttons", 0x09b2)?);
    actual.push(played_controls("intro-actor.buttons", &actors.buttons));
    Ok((expected, actual))
}

fn recorded_controls(wram: &[u8], name: &str, address: usize) -> Result<Observation, Outcome> {
    let words = wram
        .get(address..address.saturating_add(14))
        .ok_or_else(|| data_failure(None, "intro actor control words"))?;
    Ok(Observation {
        field: name.to_owned(),
        bytes: words.to_vec(),
    })
}

fn played_controls(name: &str, controls: &types::save_files::ControlBindings) -> Observation {
    Observation {
        field: name.to_owned(),
        bytes: game::exact::save_files::control_words(controls)
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect(),
    }
}

/// Samus's published visibility during the opening and first fight turn.
pub(super) fn fight_shown(wram: &[u8]) -> Result<bool, Outcome> {
    Ok(raw_word(wram, 0x1a57, "intro fight visibility")? != 0)
}

/// Compare the installed rectangle, omitting the inherited rows.
pub(super) fn populated(
    room: Option<rooms::Start>,
    content: &rooms::Content,
    wram: &[u8],
    shown: bool,
    discovery_selected: bool,
) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
    let quantity = Observation::quantity;
    let mut expected = vec![quantity("intro-actor.shown", i64::from(shown))];
    let mut actual = vec![quantity("intro-actor.shown", i64::from(room.is_some()))];
    if shown {
        let width = word(wram, source::ROOM_WIDTH_IN_BLOCKS)?;
        let height = word(wram, source::ROOM_HEIGHT_IN_BLOCKS)?;
        let populated_rows = if discovery_selected { 12 } else { 14 };
        let length = usize::from(width)
            .saturating_mul(populated_rows)
            .saturating_mul(2);
        let bytes = wram
            .get(0x10002..0x10002usize.saturating_add(length))
            .ok_or_else(|| data_failure(None, "intro room installed rectangle"))?;
        expected.push(quantity("intro-room.width", i64::from(width)));
        expected.push(quantity("intro-room.height", i64::from(height)));
        expected.push(Observation {
            field: "intro-room.populated".to_owned(),
            bytes: bytes.to_vec(),
        });
    }
    if let Some(room) = room {
        let [width, height] = content.dimensions(room);
        actual.push(quantity("intro-room.width", i64::from(width)));
        actual.push(quantity("intro-room.height", i64::from(height)));
        actual.push(Observation {
            field: "intro-room.populated".to_owned(),
            bytes: content
                .populated(room)
                .iter()
                .flat_map(|cell| cell.word().to_le_bytes())
                .collect(),
        });
    }
    Ok((expected, actual))
}
