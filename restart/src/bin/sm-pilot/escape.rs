//! Unchanged full Ceres route: traversal and ownership, without active-shot claims.
use sha2::{Digest, Sha256};
use sm_native_host::{Game, ProjectileCounters, State};

use sm_restart::route;

const STEPS: u64 = 16_991;
const INPUT_SHA256: &str = "39098a57ed3e51787b684da80c54561d81a46600b4931dc012d747c9e7c097f0";

pub(super) fn author_inputs(inputs: Vec<u16>) -> Result<Vec<u16>, String> {
    // Canonical saved-pad encoding authenticates the sequence independently of
    // whitespace accepted by the caller. Never truncate or add a firing edge.
    let mut digest = Sha256::new();
    for pad in &inputs {
        digest.update(format!("{pad:03x}\n"));
    }
    if inputs.len() as u64 != STEPS || format!("{:x}", digest.finalize()) != INPUT_SHA256 {
        return Err("escape requires the unchanged retained 16991-pad Ceres route".into());
    }
    Ok(inputs)
}

/// Call at host-step return after activation. C entries before activation are
/// retained separately; every later C call must match one attributed refusal.
pub(super) fn ownership_valid(counts: ProjectileCounters, prelude: [u64; 4]) -> bool {
    counts.other_refused == [0; 4]
        && counts.rust_refused[1] == 0
        && counts.ceres_high_bit_refused[1] == 0
        && (0..4).all(|i| {
            counts.rust_refused[i] == counts.ceres_high_bit_refused[i]
                && counts.rust_completed[i].checked_add(counts.rust_refused[i]) == Some(counts.rust_entries[i])
                && prelude[i].checked_add(counts.rust_refused[i]) == Some(counts.c_entries[i])
                && counts.ceres_high_bit_refused[i] <= counts.ceres_high_bit_entries[i]
                && counts.ceres_high_bit_completed[i] <= counts.rust_completed[i]
                && counts.ceres_high_bit_completed[i]
                    .checked_add(counts.ceres_high_bit_refused[i])
                    .is_some_and(|n| n <= counts.ceres_high_bit_entries[i])
                && counts.c_entries[i]
                    .checked_add(counts.rust_completed[i])
                    .is_some_and(|n| counts.ceres_high_bit_entries[i] <= n)
        })
}

pub(super) fn ownership_complete(counts: ProjectileCounters, prelude: [u64; 4]) -> bool {
    ownership_valid(counts, prelude)
        && counts.rust_completed[1..].iter().all(|n| *n > 0)
        && counts.ceres_high_bit_completed[1] > 0
}

#[derive(Default)]
pub(super) struct Material {
    route: route::Route,
    steps: u64,
    high_bit_steps: u64,
    first_high_bit_step: Option<u64>,
    last_high_bit_step: Option<u64>,
    escape_steps: [Option<u64>; 3],
    arrival_step: Option<u64>,
    active_projectile_steps: u64,
    final_state: State,
}

impl Material {
    pub fn observe(&mut self, step: u64, state: State, ram: &[u8], _counts: ProjectileCounters) -> Result<(), String> {
        if ram.len() != 0x20000 || step != self.steps + 1 || state.host_steps != step || step > STEPS {
            return Err("invalid full-route step sequence or WRAM extent".into());
        }
        self.steps = step;
        let before = self.route.escape_stage;
        self.route.observe(&state);
        if self.route.escape_stage > before {
            self.escape_steps[usize::from(self.route.escape_stage - 1)] = Some(step);
        }
        if self.route.arrived {
            self.arrival_step.get_or_insert(step);
        }
        if u16::from_le_bytes([ram[0x93f], ram[0x940]]) & 0x8000 != 0 {
            self.high_bit_steps += 1;
            self.first_high_bit_step.get_or_insert(step);
            self.last_high_bit_step = Some(step);
        }
        // This sample only reports slot presence; no active transformed-trail
        // or rendering coverage can be derived from it or from empty entries.
        self.active_projectile_steps += u64::from((0..10).any(|i| ram[0xc40 + 2 * i..0xc42 + 2 * i] != [0, 0]));
        self.final_state = state;
        Ok(())
    }

    fn reached(&self, counts: ProjectileCounters) -> bool {
        self.steps == STEPS
            && self.route.entered_ceres
            && self.route.controller_response
            && self.route.escape_stage == 3
            && self.route.arrived
            && self.high_bit_steps > 0
            && self.final_state.game_state == 8
            && self.final_state.area == 0
            && self.final_state.room == 0x91f8
            && self.final_state.cpu_fallback_calls == 0
            && self.final_state.cpu_instructions == 0
            && counts.ceres_high_bit_entries[1] > 0
            && (1..4).all(|i| counts.c_entries[i].checked_add(counts.rust_completed[i]).is_some_and(|n| n > 0))
    }

    pub fn complete(&self, game: &Game) -> bool {
        self.reached(game.projectile_counters())
    }

    pub fn report(&self, game: &Game) -> sm_binary_evidence::Value {
        let counts = game.projectile_counters();
        let mut report = sm_binary_evidence::Map::new();
        report.insert("material_complete".into(), self.reached(counts).into());
        report.insert("input_sha256".into(), INPUT_SHA256.into());
        for (key, value) in [
            ("steps", self.steps),
            ("high_bit_sampled_steps", self.high_bit_steps),
            ("active_projectile_sampled_steps", self.active_projectile_steps),
        ] {
            report.insert(key.into(), (value as i64).into());
        }
        for (key, value) in [
            ("first_high_bit_step", self.first_high_bit_step),
            ("last_high_bit_step", self.last_high_bit_step),
            ("escape20_step", self.escape_steps[0]),
            ("escape21_step", self.escape_steps[1]),
            ("escape22_step", self.escape_steps[2]),
            ("area0_arrival_step", self.arrival_step),
        ] {
            report.insert(key.into(), value.map_or(-1, |n| n as i64).into());
        }
        for (key, values) in [
            ("ceres_high_bit_entries", counts.ceres_high_bit_entries),
            ("ceres_high_bit_completed", counts.ceres_high_bit_completed),
            ("ceres_high_bit_refused", counts.ceres_high_bit_refused),
            ("other_refused", counts.other_refused),
        ] {
            report.insert(
                key.into(),
                values.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into(),
            );
        }
        report.insert("active_transformed_trail_claimed".into(), false.into());
        report.insert("active_escape_rendering_claimed".into(), false.into());
        report.into()
    }
}

fn integer(value: &sm_binary_evidence::Value, key: &str) -> Option<i64> {
    value.get(key)?.as_integer().filter(|n| *n >= 0)
}
fn integers(value: &sm_binary_evidence::Value, key: &str) -> Option<Vec<i64>> {
    value.get(key)?.as_array()?.iter().map(|n| n.as_integer().filter(|n| *n >= 0)).collect()
}
fn flag(value: &sm_binary_evidence::Value, key: &str) -> bool {
    value.get(key).and_then(sm_binary_evidence::Value::as_bool) == Some(true)
}
fn positive(value: &sm_binary_evidence::Value, key: &str) -> bool {
    integer(value, key).is_some_and(|n| n > 0)
}
fn balanced(value: &sm_binary_evidence::Value, requests: &str, completions: &str) -> bool {
    value.get(requests).is_some_and(|n| value.get(completions) == Some(n))
}
fn span(value: &sm_binary_evidence::Value, min: &str, max: &str, required: i64) -> bool {
    integer(value, min).zip(integer(value, max)).is_some_and(|(min, max)| max - min >= required)
}
fn roots(value: &sm_binary_evidence::Value, length: usize) -> Option<Vec<i64>> {
    let c = integers(value, "c_entries")?;
    let rust = integers(value, "rust_completed")?;
    if c.len() != length || rust.len() != length {
        return None;
    }
    c.into_iter().zip(rust).map(|(c, rust)| c.checked_add(rust)).collect()
}
fn routines(value: &sm_binary_evidence::Value, length: usize) -> Option<Vec<i64>> {
    let c = integers(value, "routine_c_completed")?;
    let rust = integers(value, "routine_rust_completed")?;
    if c.len() != length || rust.len() != length {
        return None;
    }
    c.into_iter().zip(rust).map(|(c, rust)| c.checked_add(rust)).collect()
}
fn both_flags(value: &sm_binary_evidence::Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(sm_binary_evidence::Value::as_array)
        .is_some_and(|flags| flags.len() == 2 && flags.iter().all(|flag| flag.as_bool() == Some(true)))
}

/// Consume the existing material reports. Their short-route final-room and
/// exactly-two-crossing booleans do not describe the full route. The original
/// two detailed rightward crossings remain required; extra leftward execution
/// is counted separately and is not mislabeled as their detailed motion proof.
pub(super) fn consumers_complete(
    scroll: &sm_binary_evidence::Value,
    doors: &sm_binary_evidence::Value,
    transitions: &sm_binary_evidence::Value,
    directional: &sm_binary_evidence::Value,
    camera: &sm_binary_evidence::Value,
) -> bool {
    let scroll_ok = roots(scroll, 3).is_some_and(|roots| {
        roots.iter().all(|n| *n > 0)
            && integers(scroll, "flush_requests")
                .is_some_and(|flushes| flushes.len() == 3 && roots[0].checked_mul(17) == Some(flushes[0]))
    }) && flag(scroll, "corridor_resumed")
        && span(scroll, "shaft_camera_y_min", "shaft_camera_y_max", 256)
        && span(scroll, "corridor_camera_x_min", "corridor_camera_x_max", 64)
        && span(scroll, "stairs_camera_y_min", "stairs_camera_y_max", 64)
        && ["corridor_columns", "stairs_rows"]
            .into_iter()
            .all(|key| integers(scroll, key).is_some_and(|counts| counts.len() == 2 && counts[0] > 0))
        && ["corridor_gameplay_dma_bytes", "stairs_gameplay_dma_bytes", "vram_dma_bytes"]
            .into_iter()
            .all(|key| positive(scroll, key))
        && [
            ("flush_requests", "flush_completions"),
            ("foreign_requests", "foreign_completions"),
            ("total_flush_requests", "total_flush_completions"),
        ]
        .into_iter()
        .all(|(a, b)| balanced(scroll, a, b));
    let doors_ok = roots(doors, 13).is_some_and(|roots| roots.iter().all(|n| *n > 0))
        && flag(doors, "parameter2_main_observed")
        && both_flags(doors, "exits")
        && both_flags(doors, "resumed_input")
        && integers(doors, "proximity_successes").is_some_and(|counts| counts.iter().filter(|n| **n > 0).count() >= 2)
        && positive(doors, "solid_setups")
        && balanced(doors, "foreign_requests", "foreign_completions");
    let transition_ok =
        transitions.get("crossings").and_then(sm_binary_evidence::Value::as_array).is_some_and(|rows| {
            rows.len() >= 2
                && [(0xdf45, 0xdf8d, 1), (0xdf8d, 0xdfd7, 0)].into_iter().enumerate().all(
                    |(i, (source, destination, palette))| {
                        flag(&rows[i], "material_complete")
                            && integer(&rows[i], "source") == Some(source)
                            && integer(&rows[i], "destination") == Some(destination)
                            && integer(&rows[i], "palette_branch") == Some(palette)
                    },
                )
        }) && routines(transitions, 27).is_some_and(|routines| routines[24] > 0)
            && integers(transitions, "root_pending").is_some_and(|counts| counts.len() == 3 && counts[2] > 0)
            && balanced(transitions, "root_pending", "root_resumed")
            && balanced(transitions, "service_requests", "service_completions");
    let directional_ok = integer(directional, "orphan_roots") == Some(0)
        && directional.get("crossings").and_then(sm_binary_evidence::Value::as_array).is_some_and(|rows| {
            rows.len() == 10
                && [0xdf8d, 0xdfd7]
                    .into_iter()
                    .enumerate()
                    .all(|(i, room)| integer(&rows[i], "room") == Some(room) && flag(&rows[i], "complete"))
        })
        && routines(directional, 13)
            .is_some_and(|routines| routines[3] == 5 && routines[4] == 5 && routines[9] == 320 && routines[10] == 320)
        && balanced(directional, "service_requests", "service_completions");
    scroll_ok && doors_ok && transition_ok && directional_ok && flag(camera, "material_complete")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts() -> ProjectileCounters {
        ProjectileCounters {
            c_entries: [0, 10, 14, 14],
            rust_entries: [0, 100, 104, 104],
            rust_completed: [0, 100, 100, 100],
            rust_refused: [0, 0, 4, 4],
            ceres_high_bit_entries: [0, 3, 4, 4],
            ceres_high_bit_completed: [0, 3, 0, 0],
            ceres_high_bit_refused: [0, 0, 4, 4],
            ..ProjectileCounters::default()
        }
    }

    #[test]
    fn zero_fire_and_explicit_draw_refusals_allow_strict_update_ownership() {
        let c = counts();
        assert!(ownership_complete(c, [0, 10, 10, 10]));
        let mut c = c;
        c.rust_entries[0] = 1;
        c.rust_refused[0] = 1;
        c.c_entries[0] = 1;
        c.ceres_high_bit_entries[0] = 1;
        c.ceres_high_bit_refused[0] = 1;
        assert!(ownership_complete(c, [0, 10, 10, 10]));
    }

    #[test]
    fn no_blanket_high_bit_allowance_or_unowned_update() {
        for kind in 0..4 {
            let mut c = counts();
            c.rust_entries[kind] += 1;
            c.rust_refused[kind] += 1;
            c.c_entries[kind] += 1;
            c.other_refused[kind] += 1;
            assert!(!ownership_valid(c, [0, 10, 10, 10]));
        }
        let mut c = counts();
        c.rust_entries[1] += 1;
        c.rust_refused[1] += 1;
        c.c_entries[1] += 1;
        c.ceres_high_bit_refused[1] += 1;
        assert!(!ownership_valid(c, [0, 10, 10, 10]));
        let mut c = counts();
        c.c_entries[1] += 1;
        assert!(!ownership_valid(c, [0, 10, 10, 10]));
        let mut c = counts();
        c.rust_entries[2] += 1;
        assert!(!ownership_valid(c, [0, 10, 10, 10]));
        let mut c = counts();
        c.ceres_high_bit_completed[1] = 0;
        assert!(ownership_valid(c, [0, 10, 10, 10]));
        assert!(!ownership_complete(c, [0, 10, 10, 10]));
    }

    fn full_route(ordered: bool, loading: u16, timer: u16) -> Material {
        let mut material = Material::default();
        let mut ram = vec![0; 0x20000];
        for step in 1..=STEPS {
            let mut state = State {
                host_steps: step,
                polls: step,
                room: 0xdf45,
                area: 6,
                game_state: 8,
                x: if step == 1 { 100 } else { 120 },
                joy1_current: 1,
                ..State::default()
            };
            match step {
                3 => state.game_state = if ordered { 0x20 } else { 0x21 },
                4 => state.game_state = if ordered { 0x21 } else { 0x20 },
                5 => {
                    state.game_state = 0x22;
                    state.loading_game_state = loading;
                    state.timer = timer;
                }
                6.. => {
                    state.area = 0;
                    state.room = 0x91f8;
                }
                _ => {}
            }
            ram[0x940] = if step == 3 { 0x80 } else { 0 };
            material.observe(step, state, &ram, counts()).unwrap();
        }
        material
    }

    #[test]
    fn full_extent_order_loading_and_timer_are_material() {
        let material = full_route(true, 0x22, 0);
        assert!(material.reached(counts()));
        assert_eq!(material.active_projectile_steps, 0);
        assert_eq!(material.escape_steps, [Some(3), Some(4), Some(5)]);
        assert!(!full_route(false, 0x22, 0).reached(counts()));
        assert!(!full_route(true, 0x21, 0).reached(counts()));
        assert!(!full_route(true, 0x22, 1).reached(counts()));
        let mut material = material;
        material.steps -= 1;
        assert!(!material.reached(counts()));
    }

    #[test]
    fn rejects_changed_pads_and_missing_or_duplicate_observation_steps() {
        assert!(author_inputs(vec![0; STEPS as usize]).is_err());
        assert!(author_inputs(vec![0; 9380]).is_err());
        let mut material = Material::default();
        let state = State { host_steps: 1, ..State::default() };
        assert!(material.observe(1, state, &[0; 2], counts()).is_err());
        let ram = vec![0; 0x20000];
        material.observe(1, state, &ram, counts()).unwrap();
        assert!(material.observe(1, state, &ram, counts()).is_err());
        assert!(material.observe(3, State { host_steps: 3, ..state }, &ram, counts()).is_err());
    }

    #[test]
    fn missing_reports_and_short_route_booleans_cannot_supply_full_material() {
        let report: sm_binary_evidence::Value = sm_binary_evidence::Value::Table(sm_binary_evidence::Map::from([(
            "material_complete".into(),
            true.into(),
        )]));
        assert!(!consumers_complete(&report, &report, &report, &report, &report));
    }

    fn consumer_reports() -> [sm_binary_evidence::Value; 5] {
        let mut reports: [sm_binary_evidence::Value; 5] = [
            sm_binary_evidence::Value::Table(sm_binary_evidence::Map::from([
                ("c_entries".into(), sm_binary_evidence::Value::Array(vec![2.into(), 645.into(), 5802.into()])),
                ("rust_completed".into(), sm_binary_evidence::Value::Array(vec![0.into(), 0.into(), 0.into()])),
                ("flush_requests".into(), sm_binary_evidence::Value::Array(vec![34.into(), 0.into(), 0.into()])),
                ("flush_completions".into(), sm_binary_evidence::Value::Array(vec![34.into(), 0.into(), 0.into()])),
                ("foreign_requests".into(), 1.into()),
                ("foreign_completions".into(), 1.into()),
                ("total_flush_requests".into(), 1.into()),
                ("total_flush_completions".into(), 1.into()),
                ("corridor_resumed".into(), true.into()),
                ("shaft_camera_y_min".into(), 0.into()),
                ("shaft_camera_y_max".into(), 256.into()),
                ("corridor_camera_x_min".into(), 0.into()),
                ("corridor_camera_x_max".into(), 64.into()),
                ("stairs_camera_y_min".into(), 0.into()),
                ("stairs_camera_y_max".into(), 64.into()),
                ("corridor_columns".into(), sm_binary_evidence::Value::Array(vec![1.into(), 0.into()])),
                ("stairs_rows".into(), sm_binary_evidence::Value::Array(vec![1.into(), 0.into()])),
                ("corridor_gameplay_dma_bytes".into(), 128.into()),
                ("stairs_gameplay_dma_bytes".into(), 128.into()),
                ("vram_dma_bytes".into(), 256.into()),
            ])),
            sm_binary_evidence::Value::Table(sm_binary_evidence::Map::from([
                ("parameter2_main_observed".into(), true.into()),
                ("exits".into(), sm_binary_evidence::Value::Array(vec![true.into(), true.into()])),
                ("resumed_input".into(), sm_binary_evidence::Value::Array(vec![true.into(), true.into()])),
                ("proximity_successes".into(), sm_binary_evidence::Value::Array(vec![1.into(), 1.into()])),
                ("solid_setups".into(), 1.into()),
                ("foreign_requests".into(), 10.into()),
                ("foreign_completions".into(), 10.into()),
            ])),
            sm_binary_evidence::Value::Table(sm_binary_evidence::Map::from([
                ("root_pending".into(), sm_binary_evidence::Value::Array(vec![0.into(), 0.into(), 630.into()])),
                ("root_resumed".into(), sm_binary_evidence::Value::Array(vec![0.into(), 0.into(), 630.into()])),
                ("service_requests".into(), sm_binary_evidence::Value::Array(vec![1.into()])),
                ("service_completions".into(), sm_binary_evidence::Value::Array(vec![1.into()])),
                (
                    "crossings".into(),
                    sm_binary_evidence::Value::Array(vec![
                        sm_binary_evidence::Value::Table(sm_binary_evidence::Map::from([
                            ("source".into(), 57157.into()),
                            ("destination".into(), 57229.into()),
                            ("palette_branch".into(), 1.into()),
                            ("material_complete".into(), true.into()),
                        ])),
                        sm_binary_evidence::Value::Table(sm_binary_evidence::Map::from([
                            ("source".into(), 57229.into()),
                            ("destination".into(), 57303.into()),
                            ("palette_branch".into(), 0.into()),
                            ("material_complete".into(), true.into()),
                        ])),
                    ]),
                ),
            ])),
            sm_binary_evidence::Value::Table(sm_binary_evidence::Map::from([
                ("orphan_roots".into(), 0.into()),
                (
                    "service_requests".into(),
                    sm_binary_evidence::Value::Array(vec![10.into(), 10.into(), 10.into(), 645.into()]),
                ),
                (
                    "service_completions".into(),
                    sm_binary_evidence::Value::Array(vec![10.into(), 10.into(), 10.into(), 645.into()]),
                ),
            ])),
            sm_binary_evidence::Value::Table(sm_binary_evidence::Map::from([(
                "material_complete".into(),
                true.into(),
            )])),
        ];
        let numbers = |values: Vec<i64>| {
            sm_binary_evidence::Value::Array(values.into_iter().map(sm_binary_evidence::Value::Integer).collect())
        };
        reports[1].as_table_mut().unwrap().insert("c_entries".into(), numbers(vec![1; 13]));
        reports[1].as_table_mut().unwrap().insert("rust_completed".into(), numbers(vec![0; 13]));
        for (report, length) in [(2, 27), (3, 13)] {
            reports[report].as_table_mut().unwrap().insert("routine_c_completed".into(), numbers(vec![0; length]));
            reports[report].as_table_mut().unwrap().insert("routine_rust_completed".into(), numbers(vec![0; length]));
        }
        reports[2]["routine_c_completed"][24] = 1.into();
        for (index, value) in [(3, 5), (4, 5), (9, 320), (10, 320)] {
            reports[3]["routine_c_completed"][index] = value.into();
        }
        let crossings = (0..10)
            .map(|i| {
                let room = if i == 0 { 0xdf8d } else { 0xdfd7 };
                sm_binary_evidence::Value::Table(sm_binary_evidence::Map::from([
                    ("room".into(), (room as i64).into()),
                    ("complete".into(), (i < 2).into()),
                ]))
            })
            .collect::<Vec<_>>()
            .into();
        reports[3].as_table_mut().unwrap().insert("crossings".into(), crossings);
        reports
    }

    #[test]
    fn full_route_preserves_original_material_without_short_endpoint_constraints() {
        let reports = consumer_reports();
        let complete = |r: &[sm_binary_evidence::Value; 5]| consumers_complete(&r[0], &r[1], &r[2], &r[3], &r[4]);
        assert!(complete(&reports));
        // Later leftward records are not rightward motion proofs. The two
        // original proofs and the reached leftward routine counts are separate.
        let mut missing_left = reports.clone();
        missing_left[3]["routine_c_completed"][10] = 0.into();
        assert!(!complete(&missing_left));
        let mut missing_crossing = reports.clone();
        missing_crossing[2]["crossings"][0]["material_complete"] = false.into();
        assert!(!complete(&missing_crossing));
        let mut missing_dma = reports.clone();
        missing_dma[0]["corridor_gameplay_dma_bytes"] = 0.into();
        assert!(!complete(&missing_dma));
        let mut pending = reports.clone();
        pending[2]["root_resumed"][2] = 629.into();
        assert!(!complete(&pending));
        let mut missing_command_a = reports;
        missing_command_a[2]["routine_c_completed"][24] = 0.into();
        assert!(!complete(&missing_command_a));
    }
}
