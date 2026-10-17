//! Material directional motion through the existing two Ceres crossings.
use sm_native_host::{DoorScrollingCounters as Counts, State};

fn numbers(values: impl IntoIterator<Item = u64>) -> sm_binary_evidence::Value {
    values.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into()
}
fn pair(value: u32) -> [u16; 2] {
    [value as u16, (value >> 16) as u16]
}

#[derive(Clone, Default)]
struct Snapshot {
    layers: Option<[u32; 3]>, // layer1 XY, layer2 XY, BG1 HV
    rest: Option<[u32; 3]>,   // BG2 HV, destination XY, flags
}
#[derive(Default)]
struct Call {
    id: u32,
    counter: u16,
    after: u16,
    routine: Option<u32>,
    children: Vec<(u32, u16, u16)>,
}
#[derive(Default)]
struct Motion {
    entry: u32,
    direction: u16,
    before_counter: u16,
    after_counter: u16,
    before_flag: u16,
    after_flag: u16,
    before: Snapshot,
    after: Snapshot,
    movements: Vec<(u32, u16, u8, u16)>,
    calls: Vec<Call>,
}
impl Motion {
    fn right_complete(&self, counter: u16) -> bool {
        let Some(before) = self.before.layers else { return false };
        let Some(after) = self.after.layers else { return false };
        let Some(before_rest) = self.before.rest else { return false };
        let Some(after_rest) = self.after.rest else { return false };
        let terminal = counter == 63;
        let expected_children: &[u16] = if terminal { &[63, 64] } else { &[counter] };
        let scrolling: Vec<_> = self.calls.iter().filter(|call| call.id == 3).collect();
        let calls_ok = scrolling.len() == expected_children.len()
            && scrolling.iter().zip(expected_children).all(|(call, &at)| {
                call.routine == Some(9) && call.counter == at && call.after == at && call.children == [(1, at, at)]
            });
        let expected_services: &[u32] = if self.entry == 1 {
            &[0, 1, 2, 3]
        } else if terminal {
            &[3, 3]
        } else {
            &[3]
        };
        let before_l1 = pair(before[0]);
        let after_l1 = pair(after[0]);
        let before_l2 = pair(before[1]);
        let after_l2 = pair(after[1]);
        let destination = pair(after_rest[1]);
        let coordinates_ok = if self.entry == 1 {
            // Setup rebases to destination-256 and performs its first +4 move.
            after_l1 == [destination[0].wrapping_sub(252), destination[1]] && self.before_flag == self.after_flag
        } else {
            after_l2 == [before_l2[0].wrapping_add(4), before_l2[1]]
                && after_l1 == if terminal { destination } else { [before_l1[0].wrapping_add(4), before_l1[1]] }
                && self.before_flag & 0x8000 == 0
                && self.after_flag == self.before_flag | if terminal { 0x8000 } else { 0 }
        };
        self.direction & 3 == 0
            && self.before_counter == counter
            && self.after_counter == counter + 1
            && self.movements == [(9, counter, u8::from(terminal), counter + 1)]
            && self.calls.iter().map(|call| call.id).eq(expected_services.iter().copied())
            && calls_ok
            && coordinates_ok
            && before_rest[1] == after_rest[1]
            && before_rest[2] == u32::from(self.before_flag)
            && after_rest[2] == u32::from(self.after_flag)
    }
}

// New effects27..38 are closed native observations; unrelated existing effect
// kinds pass through the same trace comparator. Parse only balanced completed
// operations, so counters cannot substitute for missing terminal child order.
fn inspect_effects(bytes: &[u8]) -> Result<Vec<Motion>, String> {
    if !bytes.len().is_multiple_of(16) {
        return Err("partial directional effect record".into());
    }
    let mut result = Vec::new();
    let mut current: Option<Motion> = None;
    let mut routines: Vec<(u32, u16)> = Vec::new();
    let mut call: Option<Call> = None;
    let mut child: Option<(u32, u16)> = None;
    for record in bytes.as_chunks::<16>().0 {
        let field = |at| u32::from_le_bytes(record[at..at + 4].try_into().unwrap());
        let [kind, a, b, c] = [field(0), field(4), field(8), field(12)];
        if !(27..=38).contains(&kind) {
            continue;
        }
        if kind == 27 {
            if current.is_some() || a > 2 || b > u32::from(u16::MAX) {
                return Err("invalid/overlapping directional root".into());
            }
            current = Some(Motion {
                entry: a,
                direction: b as u16,
                before_counter: c as u16,
                before_flag: (c >> 16) as u16,
                ..Motion::default()
            });
            continue;
        }
        let motion = current.as_mut().ok_or("directional event outside root")?;
        match kind {
            28 => {
                if motion.entry != a || u32::from(motion.direction) != b || !routines.is_empty() || call.is_some() {
                    return Err("unbalanced directional root return".into());
                }
                motion.after_counter = c as u16;
                motion.after_flag = (c >> 16) as u16;
                result.push(current.take().unwrap());
            }
            29 => {
                if a >= 13 || b != 0 || call.is_some() || routines.iter().any(|(id, _)| *id == a) {
                    return Err("invalid directional routine enter".into());
                }
                routines.push((a, c as u16));
            }
            30 => {
                let (id, counter) = routines.pop().ok_or("directional routine return without entry")?;
                if id != a || call.is_some() || b > 255 {
                    return Err("unbalanced directional routine return".into());
                }
                if id >= 9 {
                    motion.movements.push((id, counter, b as u8, c as u16));
                }
            }
            31 => {
                if a > 3 || b != 0 || call.is_some() || routines.is_empty() {
                    return Err("invalid/overlapping directional service".into());
                }
                call = Some(Call {
                    id: a,
                    counter: c as u16,
                    routine: routines.last().map(|(id, _)| *id),
                    ..Call::default()
                });
            }
            32 => {
                let mut finished = call.take().ok_or("directional service return without entry")?;
                if finished.id != a || child.is_some() || b > 255 || (a >= 2 && b != 0) {
                    return Err("unbalanced directional service return".into());
                }
                finished.after = c as u16;
                motion.calls.push(finished);
            }
            33 => {
                if a != 3 || b != 1 || call.as_ref().map(|call| call.id) != Some(a) || child.is_some() {
                    return Err("invalid directional scrolling child".into());
                }
                child = Some((b, c as u16));
            }
            34 => {
                let (entry, counter) = child.take().ok_or("directional child return without entry")?;
                let call = call.as_mut().ok_or("directional child outside service")?;
                if a != call.id || b != entry {
                    return Err("unbalanced directional child return".into());
                }
                call.children.push((entry, counter, c as u16));
            }
            35 | 36 => {
                let snapshot = if kind == 35 { &mut motion.before } else { &mut motion.after };
                if snapshot.layers.replace([a, b, c]).is_some() {
                    return Err("duplicate directional layer snapshot".into());
                }
            }
            37 | 38 => {
                let snapshot = if kind == 37 { &mut motion.before } else { &mut motion.after };
                if snapshot.rest.replace([a, b, c]).is_some() {
                    return Err("duplicate directional destination snapshot".into());
                }
            }
            _ => unreachable!(),
        }
    }
    if current.is_some() || !routines.is_empty() || call.is_some() || child.is_some() {
        return Err("unfinished directional operation at host-step boundary".into());
    }
    Ok(result)
}

struct Crossing {
    room: u16,
    steps: Vec<u64>,
    motions: Vec<Motion>,
}
impl Crossing {
    fn complete(&self) -> bool {
        self.motions.len() == 64
            && self.motions.iter().enumerate().all(|(counter, motion)| {
                motion.entry == if counter == 0 { 1 } else { 2 } && motion.right_complete(counter as u16)
            })
            && self.steps.windows(2).all(|steps| steps[0] <= steps[1])
    }
}

#[derive(Default)]
pub(super) struct Material {
    crossings: Vec<Crossing>,
    orphan_roots: u64,
}
impl Material {
    pub fn observe(&mut self, step: u64, state: State, _ram: &[u8], effects: &[u8]) -> Result<String, String> {
        let mut rows = String::new();
        for motion in inspect_effects(effects)? {
            rows.push_str(&format!(
                "{step},{},{},{},{},{:04x},{}\n",
                motion.entry,
                motion.direction,
                motion.before_counter,
                motion.after_counter,
                motion.after_flag,
                motion.calls.iter().map(|call| call.children.len()).sum::<usize>()
            ));
            if motion.entry == 1 {
                self.crossings.push(Crossing { room: state.room, steps: Vec::new(), motions: Vec::new() });
            }
            if let Some(crossing) = self.crossings.last_mut() {
                crossing.steps.push(step);
                crossing.motions.push(motion);
            } else {
                self.orphan_roots += 1;
            }
        }
        Ok(rows)
    }
    pub fn complete(&self, counts: Counts) -> bool {
        let roots: [u64; 3] = std::array::from_fn(|i| counts.c_entries[i] + counts.rust_entries[i]);
        let routines: [u64; 13] = std::array::from_fn(|i| counts.routine_c_entries[i] + counts.routine_rust_entries[i]);
        self.orphan_roots == 0
            && self.crossings.len() == 2
            && self.crossings[0].room == 0xdf8d
            && self.crossings[1].room == 0xdfd7
            && self.crossings.iter().all(Crossing::complete)
            && roots == [0, 2, 126]
            && routines == [0, 2, 126, 2, 0, 0, 0, 2, 2, 128, 0, 0, 0]
            && counts.coordinator_entries == [0, 2, 0]
            && counts.standalone_entries == [0, 0, 126]
            && counts.service_requests == [2, 2, 2, 130]
            && counts.service_completions == counts.service_requests
    }
    pub fn report(&self, counts: Counts) -> sm_binary_evidence::Value {
        let mut report = sm_binary_evidence::Map::new();
        report.insert("material_complete".into(), self.complete(counts).into());
        report.insert("orphan_roots".into(), (self.orphan_roots as i64).into());
        for (name, values) in [
            ("c_entries", counts.c_entries),
            ("rust_entries", counts.rust_entries),
            ("rust_completed", counts.rust_completed),
            ("rust_refused", counts.rust_refused),
            ("coordinator_entries", counts.coordinator_entries),
            ("standalone_entries", counts.standalone_entries),
        ] {
            report.insert(name.into(), numbers(values));
        }
        for (name, values) in [
            ("routine_c_entries", counts.routine_c_entries),
            ("routine_c_completed", counts.routine_c_completed),
            ("routine_rust_entries", counts.routine_rust_entries),
            ("routine_rust_completed", counts.routine_rust_completed),
        ] {
            report.insert(name.into(), numbers(values));
        }
        for (name, values) in [
            ("service_requests", counts.service_requests),
            ("service_completions", counts.service_completions),
            ("foreign_requests", counts.foreign_requests),
            ("foreign_completions", counts.foreign_completions),
            ("child_entries", counts.child_entries),
            ("child_completions", counts.child_completions),
        ] {
            report.insert(name.into(), numbers(values));
        }
        report.insert(
            "crossings".into(),
            self.crossings
                .iter()
                .map(|crossing| {
                    let mut value = sm_binary_evidence::Map::new();
                    value.insert("room".into(), i64::from(crossing.room).into());
                    value.insert("complete".into(), crossing.complete().into());
                    value.insert("root_steps".into(), numbers(crossing.steps.iter().copied()));
                    value.insert(
                        "scroll_child_counters".into(),
                        numbers(crossing.motions.iter().flat_map(|motion| {
                            motion.calls.iter().filter(|call| call.id == 3).map(|call| u64::from(call.counter))
                        })),
                    );
                    sm_binary_evidence::Value::Table(value)
                })
                .collect::<Vec<_>>()
                .into(),
        );
        report.into()
    }
}

pub(super) fn ownership_complete(counts: Counts) -> bool {
    counts.c_entries == [0; 3]
        && counts.rust_entries == counts.rust_completed
        && counts.rust_refused == [0; 3]
        && counts.routine_c_entries == [0; 13]
        && counts.routine_rust_entries == counts.routine_rust_completed
        && counts.foreign_requests == counts.foreign_completions
        && counts.child_entries == counts.child_completions
        && (counts.rust_entries == [0; 3] || counts.child_completions[3] > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bytes(records: &[[u32; 4]]) -> Vec<u8> {
        records.iter().flatten().flat_map(|word| word.to_le_bytes()).collect()
    }
    #[test]
    fn terminal_right_requires_distinct_children_before_movement_returns() {
        let records = [
            [27, 2, 0, 63],
            [35, 0xfffc, 100, 0],
            [37, 0, 0, 0],
            [29, 2, 0, 63],
            [29, 9, 0, 63],
            [31, 3, 0, 63],
            [33, 3, 1, 63],
            [34, 3, 1, 63],
            [32, 3, 0, 63],
            [31, 3, 0, 64],
            [33, 3, 1, 64],
            [34, 3, 1, 64],
            [32, 3, 0, 64],
            [30, 9, 1, 64],
            [30, 2, 0, 64],
            [36, 0, 104, 0],
            [38, 0, 0, 0x8000],
            [28, 2, 0, 0x80000040],
        ];
        let parsed = inspect_effects(&bytes(&records)).unwrap();
        assert!(parsed[0].right_complete(63));
        let mut wrong = records;
        wrong[9][3] = 63;
        assert!(!inspect_effects(&bytes(&wrong)).unwrap()[0].right_complete(63));
        let mut wrong = records;
        wrong.swap(9, 13);
        assert!(inspect_effects(&bytes(&wrong)).is_err());
    }
    #[test]
    fn rejects_incomplete_or_unbalanced_directional_operations() {
        assert!(inspect_effects(&[0]).is_err());
        assert!(inspect_effects(&bytes(&[[27, 2, 0, 1]])).is_err());
        assert!(inspect_effects(&bytes(&[[34, 3, 1, 0]])).is_err());
        assert!(inspect_effects(&bytes(&[[1, 2, 3, 4]])).unwrap().is_empty());
    }
}
