//! The touched doorway origin and independent first-entry sound markers.

use super::{
    Sources, camera, data_failure, gameplay_sounds as seen,
    index::{Case, InputProvider},
    production_failure,
    readers::{ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader},
    result::Outcome,
};
use crate::lock::Sha256;
use audio_transport::{
    CpuMasterTicks, CueId, QueueAllowance, Request, SenderProgress, SenderState, SoundGroup,
};
use data::{
    assets::world::Doorway,
    rom::{Rom, RomAddress},
};
use game::production::sender::Sounds;
use game::{
    production::{
        AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
        ProductionSeed, Senders,
        frame::sender_contract::{QueueWindow, Queues},
        gameplay_sounds::GameplaySounds,
        passage_audio::{Advance, Content, Output, ShootHeld, State},
    },
    state::owned::OwnedState,
};
use serde::Deserialize;
use std::{collections::VecDeque, num::Wrapping, path::Path};
use types::{Action, Colour, Pair, fixed};
use world_doors::{Arrival, ColourGroup, Palette, Rules, Scene};

pub(super) const PACKET: &str = "passage-offers-v1";
pub(super) const START_PIN: &str =
    "5d420eca8c3be05f7cb6ba7ac1411e1f8fb04336084a1a040a7232f537368426";
pub(super) const EVENT_PIN: &str =
    "4a734ec6ec507ce15f2319de787c2b1e405c118e1e022535bed4a0685b8ae369";
const SNAPSHOT_PIN: &str = "f3da06a246b2fbaf68810fc98726fbf0b5df6cfa5ac630bcc2e45f79151ce865";
const PADS_PIN: &str = "8acf117e6bca14e60b84c7e90b7504b179f8767e878e3955c82453d5b17ad349";
const FIRST: u64 = 183;

fn bad() -> Outcome {
    data_failure(None, "scoped passage sound fixture")
}

fn member(root: &Path, path: &str, pin: &str) -> Result<Vec<u8>, Outcome> {
    let bytes = std::fs::read(root.join(path)).map_err(|_| bad())?;
    if Sha256::try_from(pin).ok() != Some(Sha256::of(&bytes)) {
        return Err(bad());
    }
    Ok(bytes)
}

fn record(root: &Path, path: &str, pin: &str) -> Result<oracle_record::FrameRecord, Outcome> {
    let mut record = oracle_record::lsnes::blank();
    oracle_record::lsnes::decode(&member(root, path, pin)?, &mut record).map_err(|_| bad())?;
    Ok(record)
}

pub(super) struct Input {
    shoot: ShootHeld,
}

impl Input {
    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        production
            .advance(AdvanceInput::PassageAudio(Advance::ThroughHold(self.shoot)))
            .map_err(|fault| production_failure(cut, fault))
    }

    pub fn restorations(
        &self,
        start: &Production,
        content: &ProductionContent,
        rules: &ProductionRules,
    ) -> Result<usize, Outcome> {
        let failed = || Outcome::CheckpointFailure { cut: FIRST };
        let mut direct = start.clone();
        let next = || AdvanceInput::PassageAudio(Advance::Next(self.shoot));
        let mut saves = Vec::new();
        let mut outputs = Vec::new();
        for _ in 0..3 {
            outputs.push(direct.advance(next()).map_err(|_| failed())?);
            saves.push(direct.save().map_err(|_| failed())?);
        }
        let combined = self.advance(&mut start.clone(), FIRST)?;
        let ProductionOutputs::PassageAudio(combined) = combined else {
            return Err(failed());
        };
        if outputs
            .iter()
            .zip(&combined)
            .any(|(output, one)| output != &ProductionOutputs::PassageAudio(vec![one.clone()]))
            || combined.len() != outputs.len()
        {
            return Err(failed());
        }
        let boundary = direct.advance(next());
        for (index, saved) in saves.iter().enumerate().take(2) {
            let mut restored =
                Production::restore(saved, content.clone(), rules.clone()).map_err(|_| failed())?;
            for expected in outputs.iter().skip(index.saturating_add(1)) {
                if restored.advance(next()).as_ref() != Ok(expected) {
                    return Err(failed());
                }
            }
            if restored.advance(next()) != boundary || restored != direct {
                return Err(failed());
            }
        }
        if !matches!(
            boundary,
            Err(game::production::ProductionFault::Unsupported(
                game::production::Unsupported::SenderService
            ))
        ) {
            return Err(failed());
        }
        Ok(2)
    }
}

#[derive(Clone, Copy)]
struct Window<'a>(&'a [u8]);

impl Window<'_> {
    fn byte(self, at: usize) -> Result<u8, Outcome> {
        self.0.get(at).copied().ok_or_else(bad)
    }

    fn positions(self, index: usize) -> Result<[u8; 2], Outcome> {
        let head = self.byte(index)?;
        let tail = self.byte(3_usize.saturating_add(index))?;
        QueueWindow::new(head, tail).map_err(|_| bad())?;
        Ok([head, tail])
    }

    fn queue(self, index: usize) -> Result<VecDeque<CueId>, Outcome> {
        let [head, tail] = self.positions(index)?;
        let length = tail.wrapping_sub(head) % 16;
        let base = 19_usize.saturating_add(index.saturating_mul(16));
        (0..length)
            .map(|offset| {
                let cue =
                    self.byte(base.saturating_add(usize::from(head.wrapping_add(offset) % 16)))?;
                native_cue(index, cue)
            })
            .collect()
    }

    fn current(self) -> Result<game::production::sender::OfferCurrent, Outcome> {
        Ok(seen::current(
            seen::word(self.0, 68)? != 0,
            seen::word(self.0, 70)? >= 0x28,
            seen::word(self.0, 76)? & 0x8000 != 0,
        ))
    }

    fn ready(self, index: usize) -> Result<SenderProgress, Outcome> {
        if self.byte(6_usize.saturating_add(index))? != 0 {
            return Err(bad());
        }
        Ok(SenderProgress::Ready)
    }

    fn prefix(self, bytes: &mut Vec<u8>, held_shoot_action: Option<Self>) -> Result<(), Outcome> {
        for index in 0..seen::GROUPS.len() {
            let window = if index == 0 {
                held_shoot_action.unwrap_or(self)
            } else {
                self
            };
            seen::prefix(
                bytes,
                window.queue(index)?.into_iter(),
                Some(window.positions(index)?),
            );
        }
        Ok(())
    }

    fn fifo(self, held_shoot_action: Option<Self>) -> Result<super::compare::Observation, Outcome> {
        let mut bytes = Vec::new();
        for index in 0..seen::GROUPS.len() {
            let window = if index == 0 {
                held_shoot_action.unwrap_or(self)
            } else {
                self
            };
            seen::append_fifo(
                &mut bytes,
                &window.queue(index)?,
                &window.ready(index)?,
                Some(window.positions(index)?),
            );
        }
        Ok(seen::fifo(bytes))
    }

    fn seed(self) -> Result<GameplaySounds, Outcome> {
        let parse = |index| -> Result<_, Outcome> {
            let [head, tail] = self.positions(index)?;
            let window = QueueWindow::new(head, tail).map_err(|_| bad())?;
            let state = SenderState {
                queue: self.queue(index)?,
                progress: self.ready(index)?,
                // Reply values are unused until physical service, outside this start.
                reply: Request::Absent,
                now: CpuMasterTicks(0),
                pending: None,
            };
            Ok((window, state))
        };
        let (action, action_state) = parse(0)?;
        let (ambient, ambient_state) = parse(1)?;
        let (contact, contact_state) = parse(2)?;
        let senders =
            Senders::new(action_state, ambient_state, contact_state).map_err(|_| bad())?;
        let queues = Queues::new(action, ambient, contact, &senders).map_err(|_| bad())?;
        GameplaySounds::new(senders, queues, self.current()?).map_err(|_| bad())
    }
}

fn native_cue(group: usize, cue: u8) -> Result<CueId, Outcome> {
    match (group, cue) {
        (0, 0x02) => Ok(CueId::SamusSilence),
        (1, 0x71) => Ok(CueId::BackgroundSilence),
        _ => Err(bad()),
    }
}

#[derive(Deserialize)]
struct Markers {
    event: Vec<Event>,
}

#[derive(Deserialize)]
struct Event {
    cut: u64,
    name: String,
    after_initial_cut_master: u64,
    context: String,
    operand: Option<u16>,
}

impl Event {
    fn context(&self) -> Result<Vec<u8>, Outcome> {
        let (pairs, tail) = self.context.as_bytes().as_chunks::<2>();
        if !tail.is_empty() || pairs.len() != 128 {
            return Err(bad());
        }
        pairs
            .iter()
            .map(|part| {
                let digits = std::str::from_utf8(part).map_err(|_| bad())?;
                u8::from_str_radix(digits, 16).map_err(|_| bad())
            })
            .collect()
    }
}

impl Markers {
    fn marker(&self, cut: u64, name: &str) -> Result<&Event, Outcome> {
        let mut events = self
            .event
            .iter()
            .filter(|event| event.cut == cut && event.name == name);
        let first = events.next().ok_or_else(bad)?;
        if events.next().is_some() {
            return Err(bad());
        }
        Ok(first)
    }

    fn expectation(&self, shoot: bool) -> Result<ExpectedCut, Outcome> {
        let action = self.marker(FIRST, "action_ingress")?.context()?;
        let earlier = shoot.then_some(Window(&action));
        let mut offers = seen::Offers::default();
        for (index, group, label) in [
            (0, SoundGroup::Action, "action_ingress"),
            (1, SoundGroup::Ambient, "ambient_ingress"),
        ] {
            if index == 0 && shoot {
                continue;
            }
            let event = self.marker(FIRST, label)?;
            let window_bytes = event.context()?;
            let window = Window(&window_bytes);
            let [allowance, cue] = event.operand.ok_or_else(bad)?.to_le_bytes();
            if allowance != 15 {
                return Err(bad());
            }
            let cue = native_cue(index, cue)?;
            let permission = window.current()?.transport_permission();
            offers.ingress(group, cue, QueueAllowance::Fifteen, permission, true);
            window.prefix(&mut offers.prefixes, earlier)?;
        }
        let before = self.marker(FIRST, "hold_before")?.context()?;
        let after = self.marker(FIRST, "hold_after")?.context()?;
        let (before, after) = (Window(&before), Window(&after));
        if before.fifo(None)? != after.fifo(None)?
            || before.current()?.sounds == after.current()?.sounds
        {
            return Err(bad());
        }
        offers.order.push(seen::HOLD);
        let mut output = offers.observations();
        output.push(seen::hold(after.current()?.sounds == Sounds::Disabled));
        Ok(seen::expected_cut(vec![after.fifo(earlier)?], output))
    }

    fn validate_native_suffix(&self, root: &Path) -> Result<(), Outcome> {
        let cut = record(
            root,
            "expected/cut-184.record",
            "1011525696a372bdfb45f3f5aa56cf62994ec4027cb9f0235869aa733959b631",
        )?;
        for name in ["read_action", "read_ambient", "read_contact"] {
            let bytes = self.marker(184, name)?.context()?;
            let window = Window(&bytes);
            for (group, phase) in [1, 1, 0].into_iter().enumerate() {
                if !window.queue(group)?.is_empty()
                    || window.byte(6_usize.saturating_add(group))? != phase
                {
                    return Err(bad());
                }
            }
        }
        if seen::word(&cut.wram, 0x099c)? != 0xe2db {
            return Err(bad());
        }
        Ok(())
    }

    fn validate_order(&self) -> Result<(), Outcome> {
        let ordered = [
            "touch_to_setup",
            "setup",
            "actors",
            "draw_scene",
            "draw_inanimate",
            "destination",
            "cre",
            "action_ingress",
            "ambient_ingress",
            "hold_before",
            "hold_after",
        ];
        let times = ordered
            .into_iter()
            .map(|name| {
                self.marker(FIRST, name)
                    .map(|event| event.after_initial_cut_master)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if times.windows(2).all(|pair| pair.first() < pair.last()) {
            Ok(())
        } else {
            Err(bad())
        }
    }
}

fn colour_bank(wram: &[u8], offset: usize) -> Result<[Colour; world_doors::COLOURS], Outcome> {
    let mut colours = [Colour::BLACK; world_doors::COLOURS];
    for (index, colour) in colours.iter_mut().enumerate() {
        let raw = seen::word(wram, offset.saturating_add(index.saturating_mul(2)))?;
        *colour = Colour::new(raw).ok_or_else(bad)?;
    }
    Ok(colours)
}

fn scene(wram: &[u8]) -> Result<Scene, Outcome> {
    let word = |offset| seen::word(wram, offset);
    let camera = camera::seed(wram, &camera::Content)?;
    Ok(Scene {
        her: Pair {
            x: fixed(Wrapping(word(0x0af6)?), word(0x0af8)?),
            y: fixed(Wrapping(word(0x0afa)?), word(0x0afc)?),
        },
        seen: Pair {
            x: Wrapping(word(0x0b10)?),
            y: Wrapping(word(0x0b14)?),
        },
        camera: camera.position(),
        palette: Palette {
            screen: colour_bank(wram, 0xc000)?,
            goal: colour_bank(wram, 0xc200)?,
        },
    })
}

fn doorway(wram: &[u8], path: &Path) -> Result<(Doorway, bool), Outcome> {
    use data::{
        assets::world::{Door, ItemLighting},
        room::RoomKey,
    };
    let room = import::world::id(RoomKey(RomAddress::new(0x8f, seen::word(wram, 0x079b)?)))
        .ok_or_else(bad)?;
    let door =
        import::world::doorway(RomAddress::new(0x83, seen::word(wram, 0x078d)?)).ok_or_else(bad)?;
    let image = Rom::new(std::fs::read(path).map_err(|_| bad())?).map_err(|_| bad())?;
    let world = import::world::import(image.lorom(), &[room]).map_err(|_| bad())?;
    let current = world
        .room(room)
        .filter(|room| room.doors.contains(&Door::To(door)))
        .ok_or_else(bad)?;
    let doorway = *world.doorway(door).ok_or_else(bad)?;
    let destination = import::world::import(image.lorom(), &[doorway.room]).map_err(|_| bad())?;
    let dark = [current, destination.room(doorway.room).ok_or_else(bad)?]
        .iter()
        .any(|room| room.item_lighting == ItemLighting::Dark);
    Ok((doorway, dark))
}

fn groups() -> [ColourGroup; world_doors::COLOURS] {
    std::array::from_fn(|index| match index {
        9 | 10 | 13 | 14 | 17..=19 | 29 => ColourGroup::Lit,
        20..=23 | 28 => ColourGroup::Icon,
        209 | 210 | 212 | 221 => ColourGroup::Timer,
        196 => ColourGroup::Visor,
        _ => ColourGroup::Other,
    })
}

fn movement(wram: &[u8]) -> Result<Action, Outcome> {
    let pose = data::assets::samus::map::key_of(data::pose::PoseId(seen::word(wram, 0x0a1c)?))
        .ok_or_else(bad)?;
    if (pose.action == Action::Spinning) != (wram.get(0x0a1f) == Some(&3))
        || (pose.action == Action::WallJumping) != (wram.get(0x0a1f) == Some(&20))
    {
        return Err(bad());
    }
    Ok(pose.action)
}

fn inputs(
    root: &Path,
    wram: &[u8],
    rom: &Path,
) -> Result<(Content, Input, GameplaySounds), Outcome> {
    let word = |offset| seen::word(wram, offset);
    if word(0x0998)? != 9 || word(0x099c)? != 0xe17d {
        return Err(bad());
    }
    let snapshot = member(root, "inputs/sender-and-music-182.bin", SNAPSHOT_PIN)?;
    if snapshot.get(..68) != wram.get(0x0643..0x0687) {
        return Err(bad());
    }
    let audio = Window(&snapshot).seed()?;
    if audio.current() != seen::wram_current(wram)? {
        return Err(bad());
    }
    let pads = member(root, "inputs/steps-183-to-185.pads", PADS_PIN)?;
    let mut pads = std::str::from_utf8(&pads).map_err(|_| bad())?.lines();
    let pad =
        super::frame_origin::hardware_seed::pad(pads.next().ok_or_else(bad)?).map_err(|_| bad())?;
    let shoot = word(0x09b2)?;
    let shoot_held = pad.0 & shoot != 0;
    if shoot_held != (word(0x008b)? & shoot != 0) {
        return Err(bad());
    }
    let (doorway, items_dark) = doorway(wram, rom)?;
    let arrival = doorway.arrival;
    let content = Content {
        scene: scene(wram)?,
        doorway,
        movement: movement(wram)?,
        charge: samus_appearance::Charge::try_from(word(0x0cd0)?).map_err(|_| bad())?,
        inputs: world_doors::Inputs {
            touched: false,
            work_done: true,
            sounds_sent: false,
            music_ready: false,
            timer_running: word(0x0943)? != 0,
            items_dark,
            doorway: Some(Arrival {
                way: arrival.way,
                view: Pair {
                    x: arrival.view.x.pixels(),
                    y: arrival.view.y.pixels(),
                },
                speed: arrival.speed,
            }),
            groups: groups(),
        },
    };
    Ok((
        content,
        Input {
            shoot: ShootHeld(shoot_held),
        },
        audio,
    ))
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let root = sources.native.join(PACKET);
    let initial = record(&root, "inputs/cut-182.record", START_PIN)?;
    let (content, mut input, audio) = inputs(&root, &initial.wram, &sources.rom)?;
    let content = ProductionContent::PassageAudio(Box::new(content));
    let rules = ProductionRules::PassageAudio(Rules::default());
    let production = Production::admit(
        ProductionSeed::PassageAudio(Box::new(State::touched(audio))),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| bad())?;
    let events = member(&root, "expected/events.toml", EVENT_PIN)?;
    let markers: Markers =
        toml::from_str(std::str::from_utf8(&events).map_err(|_| bad())?).map_err(|_| bad())?;
    markers.validate_order()?;
    markers.validate_native_suffix(&root)?;
    let held_shoot = case.inputs == InputProvider::PassageSoundShootHeld;
    if held_shoot {
        input.shoot = ShootHeld(true);
    }
    let expected = markers.expectation(held_shoot)?;
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::PassageAudio(input),
        expected: ExpectedReader::PassageAudio(expected),
        actual: ActualReader::PassageAudio,
        schedule: vec![FIRST],
    })
}

pub(super) fn actual(
    production: &Production,
    output: &[Output],
) -> Result<Vec<super::compare::Observation>, Outcome> {
    let mut offers = seen::Offers::default();
    let mut held = false;
    for item in output {
        match item {
            Output::Action(Some(offer)) | Output::Ambient(offer) => {
                offers.observe(offer, &seen::GROUPS, true);
            }
            Output::Action(None) => {}
            Output::Held => {
                offers.order.push(seen::HOLD);
                held = true;
            }
        }
    }
    let mut results = offers.observations();
    let OwnedState::PassageAudio(state) = production.state() else {
        return Err(bad());
    };
    results.push(seen::hold(
        held && state.audio().current().sounds == Sounds::Disabled,
    ));
    results.extend(initial(production)?);
    Ok(results)
}

pub(super) fn initial(
    production: &Production,
) -> Result<Vec<super::compare::Observation>, Outcome> {
    let OwnedState::PassageAudio(state) = production.state() else {
        return Err(bad());
    };
    Ok(vec![seen::queues(state.audio(), &seen::GROUPS, true)])
}
