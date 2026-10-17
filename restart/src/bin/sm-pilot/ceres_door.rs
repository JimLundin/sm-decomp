//! Observed Ceres actor transitions, paired with passage into the next room.
use sm_native_host::{CeresDoorCounters, Game, State};
use std::collections::BTreeMap;

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_flags_without_observed_proximity_do_not_complete_an_actor() {
        let mut material = Material::default();
        let mut ram = vec![0; 0x20000];
        let set = |ram: &mut [u8], at: usize, value: u16| ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        let state = State { game_state: 8, area: 6, room: 0xdf45, ..State::default() };
        set(&mut ram, 0xf78, 0xe23f);
        set(&mut ram, 0xfb4, 1);
        set(&mut ram, 0xf92, 0xf5f2);
        material.observe(1, state, &ram, CeresDoorCounters::default());
        set(&mut ram, 0xf92, 0xf5ce);
        set(&mut ram, 0xf86, 0x500);
        set(&mut ram, 0xf8e, 0xfac7);
        material.observe(2, state, &ram, CeresDoorCounters::default());
        let actor = &material.actors[&(0xdf45, 0)];
        assert_eq!(actor.closed_step, Some(1));
        assert_eq!(actor.trigger_step, None);
        assert_eq!(actor.open_step, None);
    }
}

#[derive(Default)]
struct Actor {
    parameter: u16,
    x: u16,
    y: u16,
    closed_step: Option<u64>,
    trigger_step: Option<u64>,
    animation_step: Option<u64>,
    open_step: Option<u64>,
    closed_sprite: u16,
    previous: Option<(u16, u16, u16)>,
}

#[derive(Default)]
pub(super) struct Material {
    actors: BTreeMap<(u16, usize), Actor>,
    proximity: [u64; 32],
    previous_state: Option<State>,
    exits: [bool; 2],
    resumed: [bool; 2],
    solid_blocks: bool,
    animated_actor_seen: bool,
}

impl Material {
    pub fn observe(&mut self, step: u64, state: State, ram: &[u8], counts: CeresDoorCounters) -> Vec<String> {
        let mut rows = Vec::new();
        if let Some(previous) = self.previous_state {
            for (i, (source, destination)) in [(0xdf45, 0xdf8d), (0xdf8d, 0xdfd7)].into_iter().enumerate() {
                if previous.game_state == 8
                    && previous.area == 6
                    && previous.room == source
                    && state.game_state == 9
                    && self
                        .actors
                        .iter()
                        .any(|((room, _), actor)| *room == source && actor.parameter == 1 && actor.open_step.is_some())
                {
                    self.exits[i] = true;
                }
                if self.exits[i]
                    && previous.game_state == 8
                    && state.game_state == 8
                    && previous.room == destination
                    && state.room == destination
                    && state.area == 6
                    && state.polls > previous.polls
                    && state.requested_buttons != 0
                    && (state.x, state.y) != (previous.x, previous.y)
                {
                    self.resumed[i] = true;
                }
            }
        }
        self.previous_state = Some(state);
        if state.game_state == 8 && state.area == 6 {
            if state.room == 0xdf45
                && counts.solid_setups > 0
                && [0x104e0, 0x10500, 0x10520, 0x10540].iter().all(|at| word(ram, *at) & 0xf000 == 0x8000)
            {
                self.solid_blocks = true;
            }
            for slot in 0..32 {
                let base = 0xf78 + 64 * slot;
                if word(ram, base) != 0xe23f {
                    continue;
                }
                let parameter = word(ram, base + 0x3c);
                let instruction = word(ram, base + 0x1a);
                let properties = word(ram, base + 0x0e);
                let sprite = word(ram, base + 0x16);
                let x = word(ram, base + 2);
                let y = word(ram, base + 6);
                self.animated_actor_seen |= parameter == 2 && word(ram, base + 0x30) == 0xf7bd;
                let actor = self.actors.entry((state.room, slot)).or_default();
                actor.parameter = parameter;
                actor.x = x;
                actor.y = y;
                let trigger = counts.proximity_successes[slot] > self.proximity[slot];
                let pair = match parameter {
                    0 => Some((0xf5a0, 0xf57c)),
                    1 => Some((0xf5f2, 0xf5ce)),
                    _ => None,
                };
                if let Some((closed, open)) = pair {
                    if instruction == closed && properties & 0x500 == 0 {
                        actor.closed_step.get_or_insert(step);
                        actor.closed_sprite = sprite;
                    }
                    if trigger && actor.closed_step.is_some() {
                        actor.trigger_step.get_or_insert(step);
                    }
                    if actor.trigger_step.is_some()
                        && actor.open_step.is_none()
                        && instruction != closed
                        && sprite != actor.closed_sprite
                    {
                        actor.animation_step.get_or_insert(step);
                    }
                    if actor.trigger_step.is_some()
                        && actor.animation_step.is_some()
                        && instruction == open
                        && properties & 0x500 == 0x500
                    {
                        actor.open_step.get_or_insert(step);
                    }
                }
                let now = (instruction, properties, sprite);
                if actor.previous != Some(now) || trigger {
                    rows.push(format!(
                        "{step},{:04x},{},{parameter},{x},{y},{instruction:04x},{properties:04x},{sprite:04x},{},{},{},{}\n",
                        state.room,
                        slot * 64,
                        u8::from(trigger),
                        actor.closed_step.unwrap_or(0),
                        actor.animation_step.unwrap_or(0),
                        actor.open_step.unwrap_or(0),
                    ));
                    actor.previous = Some(now);
                }
            }
        }
        self.proximity = counts.proximity_successes;
        rows
    }

    pub fn complete(&self, game: &Game) -> bool {
        let counts = game.ceres_door_counters();
        // BA48 is installed by the boss-event DF71 room setup. This pre-escape
        // route selects DF57 instead; report its coverage without requiring it.
        self.animated_actor_seen
            && self.exits == [true; 2]
            && self.resumed == [true; 2]
            && counts.proximity_successes.iter().sum::<u64>() >= 2
            && game.state().game_state == 8
            && game.state().area == 6
            && game.state().room == 0xdfd7
    }

    pub fn report(&self, game: &Game) -> sm_binary_evidence::Value {
        let counts = game.ceres_door_counters();
        let mut report = sm_binary_evidence::Map::new();
        for (name, values) in [
            ("c_entries", counts.c_entries),
            ("rust_entries", counts.rust_entries),
            ("rust_completed", counts.rust_completed),
            ("rust_refused", counts.rust_refused),
        ] {
            report.insert(
                name.into(),
                values.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into(),
            );
        }
        for (name, value) in [
            ("solid_setups", counts.solid_setups),
            ("foreign_requests", counts.foreign_requests),
            ("foreign_completions", counts.foreign_completions),
        ] {
            report.insert(name.into(), (value as i64).into());
        }
        report.insert("four_solid_blocks_observed".into(), self.solid_blocks.into());
        report.insert("parameter2_main_observed".into(), self.animated_actor_seen.into());
        report.insert(
            "exits".into(),
            self.exits.into_iter().map(sm_binary_evidence::Value::Boolean).collect::<Vec<_>>().into(),
        );
        report.insert(
            "resumed_input".into(),
            self.resumed.into_iter().map(sm_binary_evidence::Value::Boolean).collect::<Vec<_>>().into(),
        );
        report.insert(
            "proximity_successes".into(),
            self.proximity.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into(),
        );
        let actors: Vec<_> = self
            .actors
            .iter()
            .map(|((room, slot), actor)| {
                let mut table = sm_binary_evidence::Map::new();
                for (name, value) in [
                    ("room", *room as i64),
                    ("index", (slot * 64) as i64),
                    ("parameter", actor.parameter as i64),
                    ("x", actor.x as i64),
                    ("y", actor.y as i64),
                ] {
                    table.insert(name.into(), value.into());
                }
                for (name, value) in [
                    ("closed_step", actor.closed_step),
                    ("trigger_step", actor.trigger_step),
                    ("animation_step", actor.animation_step),
                    ("open_step", actor.open_step),
                ] {
                    table.insert(name.into(), value.map_or(-1, |n| n as i64).into());
                }
                sm_binary_evidence::Value::Table(table)
            })
            .collect();
        report.insert("actors".into(), actors.into());
        sm_binary_evidence::Value::Table(report)
    }
}
