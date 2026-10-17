//! Material normal-camera work at its actual scheduled body boundary.
use sm_native_host::CameraCounters as Counts;
use std::collections::BTreeMap;

const ROUTINES: [&str; 13] = [
    "MainScrolling",
    "CallFinishedHook",
    "SporeSpawnFinish",
    "HandleX",
    "HandleY",
    "DistanceX",
    "DistanceY",
    "AutoX",
    "TriggerRight",
    "TriggerLeft",
    "AutoY",
    "TriggerDown",
    "TriggerUp",
];

fn numbers(values: impl IntoIterator<Item = u64>) -> sm_binary_evidence::Value {
    values.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into()
}

// Snapshot pairs put the integer in the low word and fraction in the high
// word. Pair equality is useful here; raw packed values are not fixed point.
#[derive(Default)]
struct Snapshot {
    layer: [u32; 2],
    actor: [u32; 2],
    previous: [u32; 2],
}

#[derive(Default)]
struct Frame {
    state: u16,
    room: u16,
    slow: u16,
    before: Snapshot,
    after: Snapshot,
    routines: [u64; 13],
}
impl Frame {
    fn actor_unchanged(&self) -> bool {
        self.before.actor == self.after.actor
    }
    fn previous_published(&self) -> bool {
        self.after.previous == self.after.actor
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    BeforePrimary,
    BeforeSecondary,
    Routines,
    AfterPrimary,
    AfterSecondary,
    End,
}

struct RoutineCall {
    id: u32,
    children: Vec<u32>,
}
struct Active {
    frame: Frame,
    stage: Stage,
    stack: Vec<RoutineCall>,
    root_seen: bool,
}
impl Active {
    fn new(state: u16, room: u16, slow: u16) -> Self {
        Self {
            frame: Frame { state, room, slow, ..Frame::default() },
            stage: Stage::BeforePrimary,
            stack: Vec::new(),
            root_seen: false,
        }
    }
    fn enter(&mut self, id: u32, operand: u32, reserved: u32) -> Result<(), String> {
        if self.stage != Stage::Routines
            || id >= 13
            || reserved != 0
            || operand != if id == 1 { 0x9589 } else { 0 }
            || self.stack.iter().any(|call| call.id == id)
        {
            return Err("invalid camera routine entry".into());
        }
        if let Some(parent) = self.stack.last_mut() {
            let allowed = match parent.id {
                0 => matches!(id, 1 | 3..=7 | 10),
                1 => id == 2,
                3 => (7..=9).contains(&id),
                4 => (10..=12).contains(&id),
                _ => false,
            };
            if !allowed {
                return Err("invalid camera routine parent".into());
            }
            parent.children.push(id);
        } else {
            if id != 0 || self.root_seen {
                return Err("missing or repeated camera main routine".into());
            }
            self.root_seen = true;
        }
        self.frame.routines[id as usize] += 1;
        self.stack.push(RoutineCall { id, children: Vec::new() });
        Ok(())
    }
    fn leave(&mut self, id: u32, b: u32, c: u32) -> Result<(), String> {
        if self.stage != Stage::Routines || b != 0 || c != 0 {
            return Err("invalid camera routine return".into());
        }
        let call = self.stack.pop().ok_or("camera routine return without entry")?;
        if call.id != id {
            return Err("unbalanced camera routine return".into());
        }
        let children = call.children.as_slice();
        let valid = match id {
            0 => {
                let axes: &[u32] = if self.frame.slow == 0 { &[5, 3, 6, 4] } else { &[7, 10] };
                children == axes
                    || (children.len() == axes.len() + 1 && children.starts_with(axes) && children.last() == Some(&1))
            }
            1 => children == [2],
            3 | 4 => {
                let axis = usize::from(id == 4);
                let stationary = self.frame.before.previous[axis] as u16 == self.frame.before.actor[axis] as u16;
                let auto = if axis == 0 { 7 } else { 10 };
                if stationary {
                    children == [auto]
                } else {
                    children.is_empty() || children == [auto + 1] || children == [auto + 2]
                }
            }
            _ => children.is_empty(),
        };
        if !valid {
            return Err("incorrect camera axis or private-call order".into());
        }
        if id == 0 {
            self.stage = Stage::AfterPrimary;
        }
        Ok(())
    }
}

// The pure root emits exactly39,43,45, its balanced private routines,44,46,40.
// Other native effects are allowed between camera roots, never inside one.
fn inspect_effects(bytes: &[u8]) -> Result<Vec<Frame>, String> {
    if !bytes.len().is_multiple_of(16) {
        return Err("partial camera effect record".into());
    }
    let mut frames = Vec::new();
    let mut current: Option<Active> = None;
    for record in bytes.as_chunks::<16>().0 {
        let field = |at| u32::from_le_bytes(record[at..at + 4].try_into().unwrap());
        let [kind, a, b, c] = [field(0), field(4), field(8), field(12)];
        if !(39..=46).contains(&kind) {
            if current.is_some() {
                return Err("foreign effect inside pure camera root".into());
            }
            continue;
        }
        if kind == 39 {
            if current.is_some() || [a, b, c].iter().any(|value| *value > u32::from(u16::MAX)) {
                return Err("invalid or overlapping camera root".into());
            }
            current = Some(Active::new(a as u16, b as u16, c as u16));
            continue;
        }
        let active = current.as_mut().ok_or("camera event outside root")?;
        match kind {
            40 => {
                if active.stage != Stage::End
                    || !active.stack.is_empty()
                    || [a, b, c]
                        != [u32::from(active.frame.state), u32::from(active.frame.room), u32::from(active.frame.slow)]
                {
                    return Err("unbalanced camera root or changed root context".into());
                }
                frames.push(current.take().unwrap().frame);
            }
            41 => active.enter(a, b, c)?,
            42 => active.leave(a, b, c)?,
            43 if active.stage == Stage::BeforePrimary => {
                active.frame.before.layer = [a, b];
                active.frame.before.actor[0] = c;
                active.stage = Stage::BeforeSecondary;
            }
            45 if active.stage == Stage::BeforeSecondary => {
                active.frame.before.actor[1] = a;
                active.frame.before.previous = [b, c];
                active.stage = Stage::Routines;
            }
            44 if active.stage == Stage::AfterPrimary => {
                active.frame.after.layer = [a, b];
                active.frame.after.actor[0] = c;
                active.stage = Stage::AfterSecondary;
            }
            46 if active.stage == Stage::AfterSecondary => {
                active.frame.after.actor[1] = a;
                active.frame.after.previous = [b, c];
                active.stage = Stage::End;
            }
            _ => return Err("missing, duplicate or misplaced camera snapshot".into()),
        }
    }
    if current.is_some() {
        return Err("unfinished camera root at host-step boundary".into());
    }
    Ok(frames)
}

#[derive(Default)]
struct RoomMotion {
    calls: u64,
    integer_changes: [u64; 2],
    fraction_changes: [u64; 2],
}

#[derive(Default)]
pub(super) struct Material {
    calls: u64,
    normal_calls: u64,
    actor_unchanged_calls: u64,
    previous_published_calls: u64,
    nonvacuous_previous_calls: u64,
    previous_changes: [u64; 2],
    previous_fraction_changes: [u64; 2],
    routine_entries: [u64; 13],
    tail_calls: Vec<(u64, u16)>,
    rooms: BTreeMap<u16, RoomMotion>,
}
impl Material {
    pub fn observe(&mut self, step: u64, effects: &[u8]) -> Result<String, String> {
        let mut rows = String::new();
        for frame in inspect_effects(effects)? {
            let published = frame.previous_published();
            self.calls += 1;
            self.normal_calls += u64::from(frame.slow == 0);
            self.actor_unchanged_calls += u64::from(frame.actor_unchanged());
            self.previous_published_calls += u64::from(published);
            self.nonvacuous_previous_calls += u64::from(published && frame.before.previous != frame.after.previous);
            for (total, count) in self.routine_entries.iter_mut().zip(frame.routines) {
                *total += count;
            }
            if frame.state == 9 {
                self.tail_calls.push((step, frame.room));
            }
            let room = self.rooms.entry(frame.room).or_default();
            room.calls += 1;
            for axis in 0..2 {
                room.integer_changes[axis] +=
                    u64::from(frame.before.layer[axis] as u16 != frame.after.layer[axis] as u16);
                room.fraction_changes[axis] +=
                    u64::from(frame.before.layer[axis] >> 16 != frame.after.layer[axis] >> 16);
                self.previous_changes[axis] += u64::from(frame.before.previous[axis] != frame.after.previous[axis]);
                self.previous_fraction_changes[axis] +=
                    u64::from(frame.before.previous[axis] >> 16 != frame.after.previous[axis] >> 16);
            }
            rows.push_str(&format!(
                "{step},{},{:04x},{},{},{},{},{published}\n",
                frame.state,
                frame.room,
                frame.before.layer[0] as u16,
                frame.before.layer[1] as u16,
                frame.after.layer[0] as u16,
                frame.after.layer[1] as u16,
            ));
        }
        Ok(rows)
    }
    pub fn complete(&self, counts: Counts) -> bool {
        let motion = |room, axis| self.rooms.get(&room).is_some_and(|row| row.integer_changes[axis] > 0);
        self.calls > 0
            && self.normal_calls > 0
            && self.actor_unchanged_calls == self.calls
            && self.previous_published_calls == self.calls
            && self.nonvacuous_previous_calls > 0
            && motion(0xdf8d, 0)
            && motion(0xdf45, 1)
            && motion(0xdfd7, 1)
            && [0xdf45, 0xdf8d].into_iter().all(|room| self.tail_calls.iter().any(|(_, at)| *at == room))
            // Rust attempts that refused are already represented by their C fallback.
            && counts.c_entries.checked_add(counts.rust_completed) == Some(self.calls)
            && (0..13).all(|i| {
                counts.routine_c_entries[i].checked_add(counts.routine_rust_entries[i]) == Some(self.routine_entries[i])
                    && counts.routine_c_completed[i].checked_add(counts.routine_rust_completed[i])
                        == Some(self.routine_entries[i])
            })
    }
    pub fn report(&self, counts: Counts) -> sm_binary_evidence::Value {
        let mut report = sm_binary_evidence::Map::new();
        report.insert("material_complete".into(), self.complete(counts).into());
        for (name, value) in [
            ("c_entries", counts.c_entries),
            ("rust_entries", counts.rust_entries),
            ("rust_completed", counts.rust_completed),
            ("rust_refused", counts.rust_refused),
            ("observed_calls", self.calls),
            ("normal_calls", self.normal_calls),
            ("slow_calls", self.calls - self.normal_calls),
            ("actor_unchanged_calls", self.actor_unchanged_calls),
            ("previous_published_calls", self.previous_published_calls),
            ("nonvacuous_previous_calls", self.nonvacuous_previous_calls),
        ] {
            report.insert(name.into(), (value as i64).into());
        }
        for (name, values) in [
            ("routine_c_entries", counts.routine_c_entries),
            ("routine_c_completed", counts.routine_c_completed),
            ("routine_rust_entries", counts.routine_rust_entries),
            ("routine_rust_completed", counts.routine_rust_completed),
            ("observed_routine_entries", self.routine_entries),
        ] {
            report.insert(name.into(), numbers(values));
        }
        report.insert(
            "routine_names".into(),
            ROUTINES.into_iter().map(|name| sm_binary_evidence::Value::String(name.into())).collect::<Vec<_>>().into(),
        );
        report.insert("previous_changes_xy".into(), numbers(self.previous_changes));
        report.insert("previous_fraction_changes_xy".into(), numbers(self.previous_fraction_changes));
        report.insert("state9_tail_steps".into(), numbers(self.tail_calls.iter().map(|(step, _)| *step)));
        report.insert("state9_tail_rooms".into(), numbers(self.tail_calls.iter().map(|(_, room)| u64::from(*room))));
        report.insert(
            "rooms".into(),
            self.rooms
                .iter()
                .map(|(&room, motion)| {
                    let mut row = sm_binary_evidence::Map::new();
                    row.insert("room".into(), i64::from(room).into());
                    row.insert("calls".into(), (motion.calls as i64).into());
                    row.insert("integer_changes_xy".into(), numbers(motion.integer_changes));
                    row.insert("fraction_changes_xy".into(), numbers(motion.fraction_changes));
                    sm_binary_evidence::Value::Table(row)
                })
                .collect::<Vec<_>>()
                .into(),
        );
        report.into()
    }
}

pub(super) fn ownership_complete(counts: Counts) -> bool {
    counts.c_entries == 0
        && counts.rust_entries == counts.rust_completed
        && counts.rust_refused == 0
        && counts.routine_c_entries == [0; 13]
        && counts.routine_c_completed == [0; 13]
        && counts.routine_rust_entries == counts.routine_rust_completed
        && counts.routine_rust_entries[0] == counts.rust_completed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(records: &[[u32; 4]]) -> Vec<u8> {
        records.iter().flatten().flat_map(|word| word.to_le_bytes()).collect()
    }
    fn pair(integer: u16, fraction: u16) -> u32 {
        u32::from(integer) | (u32::from(fraction) << 16)
    }
    fn leaf(records: &mut Vec<[u32; 4]>, id: u32) {
        records.extend([[41, id, 0, 0], [42, id, 0, 0]]);
    }
    fn frame(state: u32, room: u32, slow: u32, before: [u32; 2], after: [u32; 2]) -> Vec<[u32; 4]> {
        let x = pair(400, 0x1234);
        let y = pair(500, 0x5678);
        let mut records = vec![
            [39, state, room, slow],
            [43, before[0], before[1], x],
            [45, y, pair(400, 0x100), pair(500, 0)],
            [41, 0, 0, 0],
        ];
        if slow == 0 {
            leaf(&mut records, 5);
            records.push([41, 3, 0, 0]);
            leaf(&mut records, 7);
            records.push([42, 3, 0, 0]);
            leaf(&mut records, 6);
            records.push([41, 4, 0, 0]);
            leaf(&mut records, 10);
            records.push([42, 4, 0, 0]);
        } else {
            leaf(&mut records, 7);
            leaf(&mut records, 10);
        }
        records.extend([[42, 0, 0, 0], [44, after[0], after[1], x], [46, y, x, y], [40, state, room, slow]]);
        records
    }
    fn reference_counts(material: &Material) -> Counts {
        Counts {
            c_entries: material.calls,
            routine_c_entries: material.routine_entries,
            routine_c_completed: material.routine_entries,
            ..Counts::default()
        }
    }

    #[test]
    fn fractions_publish_and_integer_stationary_axes_take_autoscrollers() {
        let records = frame(9, 0xdf45, 0, [0, 100], [0, 103]);
        let frames = inspect_effects(&bytes(&records)).unwrap();
        assert!(frames[0].actor_unchanged());
        assert!(frames[0].previous_published());
        assert_eq!(frames[0].routines, [1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0]);
        let mut material = Material::default();
        assert_eq!(material.observe(8726, &bytes(&records)).unwrap(), "8726,9,df45,0,100,0,103,true\n");
        assert_eq!(material.previous_fraction_changes, [1, 1]);
    }

    #[test]
    fn material_uses_root_local_motion_and_measured_counts() {
        let mut records = Vec::new();
        for (state, room, after) in
            [(8, 0xdf45, [0, 3]), (8, 0xdf8d, [3, 0]), (8, 0xdfd7, [0, 3]), (9, 0xdf45, [0, 0]), (9, 0xdf8d, [0, 0])]
        {
            records.extend(frame(state, room, 0, [0, 0], after));
        }
        let mut material = Material::default();
        // Multiple complete calls may share a host step; their event order matters.
        material.observe(100, &bytes(&records)).unwrap();
        let counts = reference_counts(&material);
        assert!(material.complete(counts));
        let mut wrong_counts = counts;
        wrong_counts.c_entries = 936;
        assert!(!material.complete(wrong_counts));

        // A stationary corridor camera cannot borrow movement from other rooms.
        let mut no_corridor_motion = records.clone();
        let start = no_corridor_motion.iter().position(|row| row[0] == 39 && row[2] == 0xdf8d).unwrap();
        let after = no_corridor_motion[start..].iter().position(|row| row[0] == 44).unwrap() + start;
        no_corridor_motion[after][1] = 0;
        let mut material = Material::default();
        material.observe(100, &bytes(&no_corridor_motion)).unwrap();
        assert!(!material.complete(reference_counts(&material)));
    }

    #[test]
    fn slow_mode_and_optional_finish_hook_have_their_own_order() {
        let mut records = frame(8, 0xdf45, 1, [0, 100], [3, pair(464, 0xbeef)]);
        let at = records.len() - 4;
        records.splice(at..at, [[41, 1, 0x9589, 0], [41, 2, 0, 0], [42, 2, 0, 0], [42, 1, 0, 0]]);
        let frames = inspect_effects(&bytes(&records)).unwrap();
        assert_eq!(frames[0].routines, [1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0]);
        records[at][2] = 0x958a;
        assert!(inspect_effects(&bytes(&records)).is_err());
    }

    #[test]
    fn material_keeps_fraction_publication_and_actor_mutation_visible() {
        let records = frame(8, 0xdf8d, 0, [0, 0], [1, 0]);
        let mut unpublished = records.clone();
        let at = unpublished.len() - 2;
        unpublished[at][2] ^= 0x10000;
        let frames = inspect_effects(&bytes(&unpublished)).unwrap();
        assert!(frames[0].actor_unchanged());
        assert!(!frames[0].previous_published());
        let mut changed_actor = records;
        let at = changed_actor.len() - 3;
        changed_actor[at][3] ^= 0x10000;
        assert!(!inspect_effects(&bytes(&changed_actor)).unwrap()[0].actor_unchanged());
    }

    #[test]
    fn rejects_wrong_axis_order_recursion_and_private_parent() {
        let records = frame(8, 0xdf45, 0, [0, 0], [0, 0]);
        let mut reordered = records.clone();
        for row in &mut reordered {
            if matches!(row[0], 41 | 42) {
                row[1] = match row[1] {
                    5 => 6,
                    6 => 5,
                    other => other,
                };
            }
        }
        assert!(inspect_effects(&bytes(&reordered)).is_err());
        let mut recursive = records.clone();
        recursive.insert(4, [41, 0, 0, 0]);
        assert!(inspect_effects(&bytes(&recursive)).is_err());
        let mut wrong_parent = records;
        wrong_parent.insert(5, [41, 7, 0, 0]);
        assert!(inspect_effects(&bytes(&wrong_parent)).is_err());
    }

    #[test]
    fn rejects_partial_unbalanced_reordered_or_foreign_records() {
        assert!(inspect_effects(&[0]).is_err());
        assert!(inspect_effects(&bytes(&[[42, 0, 0, 0]])).is_err());
        let records = frame(8, 0xdf45, 0, [0, 0], [0, 0]);
        assert!(inspect_effects(&bytes(&records[..records.len() - 1])).is_err());
        let mut misplaced = records.clone();
        misplaced.swap(1, 2);
        assert!(inspect_effects(&bytes(&misplaced)).is_err());
        let mut duplicate = records.clone();
        duplicate.insert(2, duplicate[1]);
        assert!(inspect_effects(&bytes(&duplicate)).is_err());
        let mut foreign = records.clone();
        foreign.insert(4, [22, 1, 0, 0]);
        assert!(inspect_effects(&bytes(&foreign)).is_err());
        let mut context = records.clone();
        context.last_mut().unwrap()[2] = 0xdf8d;
        assert!(inspect_effects(&bytes(&context)).is_err());
        let mut outside = vec![[22, 1, 0, 0]];
        outside.extend(records);
        outside.push([24, 1, 0, 0]);
        assert_eq!(inspect_effects(&bytes(&outside)).unwrap().len(), 1);
    }
}
