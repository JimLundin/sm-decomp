//! Material observations for the two real scheduled Ceres room crossings.
use sm_native_host::{RoomTransitionCounters as Counts, State};

const PHASES: [u16; 15] = [
    0xe17d, 0xe19f, 0xe29e, 0xe2db, 0xe2f7, 0xe310, 0xe353, 0xe36e, 0xe38e, 0xe3c0, 0xe4a9, 0xe659, 0xe664, 0xe6a2,
    0xe737,
];
fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}
fn difference<const N: usize>(after: [u64; N], before: [u64; N]) -> [u64; N] {
    std::array::from_fn(|i| after[i] - before[i])
}
fn roots(counts: Counts) -> [u64; 3] {
    std::array::from_fn(|i| counts.c_entries[i] + counts.rust_entries[i])
}
fn routines(counts: Counts) -> [u64; 27] {
    std::array::from_fn(|i| counts.routine_c_entries[i] + counts.routine_rust_entries[i])
}
fn numbers(values: impl IntoIterator<Item = u64>) -> sm_binary_evidence::Value {
    values.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into()
}

#[derive(Default)]
struct StepEffects {
    background_copies: Vec<[u32; 3]>,
    background_decompressions: Vec<[u32; 2]>,
    background_commands: Vec<[u32; 3]>,
    completed_nmi_waits: u64,
}
struct ServiceCall {
    id: u32,
    first: u32,
    second: u32,
    third: Option<u32>,
    routine: Option<u32>,
}

// The existing effect stream is four little-endian u32 words per record.
// Count a background copy only after its actual named service returns while
// the private background-transfer routine is active. Other loading DMA does
// not substitute for the nonempty background command stream.
fn inspect_effects(bytes: &[u8]) -> Result<StepEffects, String> {
    if !bytes.len().is_multiple_of(16) {
        return Err("partial native effect record".into());
    }
    let mut routines = Vec::new();
    let mut service: Option<ServiceCall> = None;
    let mut result = StepEffects::default();
    for record in bytes.as_chunks::<16>().0 {
        let field = |at| u32::from_le_bytes(record[at..at + 4].try_into().unwrap());
        let [kind, a, b, c] = [field(0), field(4), field(8), field(12)];
        match kind {
            20 => routines.push((a, b)),
            21 => {
                let (id, operand) = routines.pop().ok_or("transition return without entry")?;
                if id != a {
                    return Err("unbalanced transition routine observation".into());
                }
                if id >= 19 {
                    result.background_commands.push([id, operand, b]);
                }
            }
            22 => {
                if service.is_some() {
                    return Err("overlapping transition services".into());
                }
                service = Some(ServiceCall {
                    id: a,
                    first: b,
                    second: c,
                    third: None,
                    routine: routines.last().map(|(id, _)| *id),
                });
            }
            23 => {
                let call = service.as_mut().ok_or("transition service arguments without entry")?;
                if call.id != a || call.third.is_some() {
                    return Err("wrong transition service arguments".into());
                }
                call.third = Some(b);
            }
            24 => {
                let ServiceCall { id, first, second, third, routine } =
                    service.take().ok_or("transition service return without entry")?;
                let third = third.ok_or("missing transition service arguments")?;
                if id != a {
                    return Err("wrong transition service return".into());
                }
                if id == 21 && routine == Some(20) {
                    result.background_copies.push([first, second, third]);
                }
                if id == 20 && routine == Some(21) {
                    result.background_decompressions.push([first, second]);
                }
                if id == 51 && b == 0 {
                    result.completed_nmi_waits += 1;
                }
            }
            _ => {}
        }
    }
    if !routines.is_empty() || service.is_some() {
        return Err("unfinished transition observation at host-step boundary".into());
    }
    Ok(result)
}

struct Crossing {
    source: u16,
    destination: u16,
    entry_step: u64,
    gameplay_step: Option<u64>,
    movement_step: Option<u64>,
    phases: [Option<u64>; 15],
    palette_branch: Option<u16>,
    first_pending: Option<u64>,
    resumed_after_irq: Option<u64>,
    stationary_resumes: u64,
    background_copies: Vec<[u32; 3]>,
    background_decompressions: Vec<[u32; 2]>,
    background_commands: Vec<[u32; 3]>,
    finish_state_cleared: bool,
    before: Counts,
    after: Counts,
}
impl Crossing {
    fn complete(&self) -> bool {
        let root = difference(roots(self.after), roots(self.before));
        let routine = difference(routines(self.after), routines(self.before));
        let services = difference(self.after.service_completions, self.before.service_completions);
        let phase_steps: Vec<_> =
            self.phases.iter().enumerate().filter(|(i, _)| *i != 1).filter_map(|(_, step)| *step).collect();
        self.gameplay_step.is_some()
            && self.movement_step.is_some()
            && root[0] == 1
            && root[1] == 0
            && root[2] > 0
            && routine[1] == 1
            && (0..=21).filter(|i| *i != 5).all(|i| routine[i] > 0)
            && self.phases.iter().enumerate().filter(|(i, _)| *i != 1).all(|(_, step)| step.is_some())
            && phase_steps.windows(2).all(|pair| pair[0] < pair[1])
            && (22..=38).all(|id| services[id] == 1)
            && routine[19] == 1
            && routine[20] == 2
            && routine[21] == 1
            && self.background_copies == [[0x4800, 0x7e4000, 0x800], [0x4c00, 0x7e4000, 0x800]]
            && self.background_decompressions == [[0xbac22a, 0x4000]]
            && self.background_commands
                == [[21, 0xe4a7, 0xe4ac], [20, 0xe4ae, 0xe4b5], [20, 0xe4b7, 0xe4be], [19, 0xe4c0, 0]]
            && self.first_pending.is_some()
            && self.resumed_after_irq.is_some()
            && self.stationary_resumes > 0
            && self.finish_state_cleared
    }
}

struct Previous {
    state: State,
    continuation: u8,
    flag: u16,
    counts: Counts,
}
#[derive(Default)]
pub(super) struct Material {
    crossings: Vec<Crossing>,
    active: Option<usize>,
    previous: Option<Previous>,
}
impl Material {
    pub fn observe(
        &mut self,
        step: u64,
        state: State,
        ram: &[u8],
        counts: Counts,
        effects: &[u8],
    ) -> Result<Option<String>, String> {
        let phase = word(ram, 0x99c);
        let continuation = ram[0x612];
        let flag = word(ram, 0x931);
        if let Some(previous) = &self.previous {
            for crossing in &mut self.crossings {
                if crossing.gameplay_step.is_some()
                    && crossing.movement_step.is_none()
                    && previous.state.game_state == 8
                    && state.game_state == 8
                    && previous.state.room == crossing.destination
                    && state.room == crossing.destination
                    && state.polls > previous.state.polls
                    && state.requested_buttons != 0
                    && (state.x, state.y) != (previous.state.x, previous.state.y)
                {
                    crossing.movement_step = Some(step);
                }
            }
            if self.active.is_none() && previous.state.game_state == 8 && state.game_state == 9 && state.area == 6 {
                let destination = match state.room {
                    0xdf45 => Some(0xdf8d),
                    0xdf8d => Some(0xdfd7),
                    _ => None,
                };
                if let Some(destination) = destination {
                    self.active = Some(self.crossings.len());
                    self.crossings.push(Crossing {
                        source: state.room,
                        destination,
                        entry_step: step,
                        gameplay_step: None,
                        movement_step: None,
                        phases: [None; 15],
                        palette_branch: None,
                        first_pending: None,
                        resumed_after_irq: None,
                        stationary_resumes: 0,
                        background_copies: Vec::new(),
                        background_decompressions: Vec::new(),
                        background_commands: Vec::new(),
                        finish_state_cleared: false,
                        before: counts,
                        after: counts,
                    });
                }
            }
        }
        let mut row = None;
        if let Some(index) = self.active {
            let observed = inspect_effects(effects)?;
            let crossing = &mut self.crossings[index];
            crossing.background_copies.extend(observed.background_copies);
            crossing.background_decompressions.extend(observed.background_decompressions);
            crossing.background_commands.extend(observed.background_commands);
            crossing.after = counts;
            if let Some(index) = PHASES.iter().position(|value| *value == phase) {
                crossing.phases[index].get_or_insert(step);
            }
            if routines(counts)[1] > routines(crossing.before)[1] {
                crossing.palette_branch.get_or_insert((word(ram, 0x7b1) | word(ram, 0x7b3)) & 1);
            }
            if counts.root_pending[2] > crossing.before.root_pending[2]
                && phase == 0xe4a9
                && continuation == 1
                && ram[0x611] == 0x15
            {
                crossing.first_pending.get_or_insert(step);
            }
            if let Some(previous) = &self.previous {
                if counts.root_resumed[2] > previous.counts.root_resumed[2]
                    && state.polls == previous.state.polls
                    && state.updates == previous.state.updates
                    && state.nmis > previous.state.nmis
                {
                    crossing.stationary_resumes += 1;
                }
                if previous.continuation == 1
                    && previous.flag & 0x8000 != 0
                    && counts.root_resumed[2] > previous.counts.root_resumed[2]
                    && observed.completed_nmi_waits > 0
                    && continuation == 0
                    && ram[0x782] == 0
                    && phase == 0xe659
                {
                    crossing.resumed_after_irq.get_or_insert(step);
                }
            }
            row = Some(format!(
                "{step},{},{:04x},{phase:04x},{continuation},{flag:04x},{},{}\n",
                state.game_state, state.room, counts.root_pending[2], counts.root_resumed[2]
            ));
            if state.game_state == 8 && state.room == crossing.destination {
                crossing.gameplay_step = Some(step);
                crossing.finish_state_cleared = ram[0x611..=0x613] == [0, 0, 0]
                    && word(ram, 0x795) == 0
                    && word(ram, 0x797) == 0
                    && word(ram, 0x5f5) == 0
                    && phase == 0xe737;
                self.active = None;
            }
        }
        self.previous = Some(Previous { state, continuation, flag, counts });
        Ok(row)
    }
    pub fn complete(&self) -> bool {
        self.active.is_none()
            && self.crossings.len() == 2
            && self.crossings.iter().all(Crossing::complete)
            && self.crossings[0].source == 0xdf45
            && self.crossings[1].source == 0xdf8d
            && self.crossings[0].palette_branch == Some(1)
            && self.crossings[1].palette_branch == Some(0)
    }
    pub fn report(&self, counts: Counts) -> sm_binary_evidence::Value {
        let mut report = sm_binary_evidence::Map::new();
        report.insert("material_complete".into(), self.complete().into());
        for (name, value) in [
            ("c_entries", counts.c_entries),
            ("rust_entries", counts.rust_entries),
            ("rust_completed", counts.rust_completed),
            ("rust_refused", counts.rust_refused),
            ("root_pending", counts.root_pending),
            ("root_resumed", counts.root_resumed),
        ] {
            report.insert(name.into(), numbers(value));
        }
        for (name, value) in [
            ("routine_c_entries", counts.routine_c_entries),
            ("routine_c_completed", counts.routine_c_completed),
            ("routine_rust_entries", counts.routine_rust_entries),
            ("routine_rust_completed", counts.routine_rust_completed),
        ] {
            report.insert(name.into(), numbers(value));
        }
        for (name, value) in [
            ("service_requests", counts.service_requests),
            ("service_completions", counts.service_completions),
            ("foreign_requests", counts.foreign_requests),
            ("foreign_completions", counts.foreign_completions),
            ("service_pending", counts.service_pending),
        ] {
            report.insert(name.into(), numbers(value));
        }
        for (name, value) in [("child_entries", counts.child_entries), ("child_completions", counts.child_completions)]
        {
            report.insert(name.into(), value.into_iter().map(numbers).collect::<Vec<_>>().into());
        }
        report.insert(
            "crossings".into(),
            self.crossings
                .iter()
                .map(|crossing| {
                    let mut value = sm_binary_evidence::Map::new();
                    for (name, n) in [
                        ("source", crossing.source as u64),
                        ("destination", crossing.destination as u64),
                        ("entry_step", crossing.entry_step),
                        ("stationary_resumes", crossing.stationary_resumes),
                    ] {
                        value.insert(name.into(), (n as i64).into());
                    }
                    for (name, n) in [
                        ("gameplay_step", crossing.gameplay_step),
                        ("movement_step", crossing.movement_step),
                        ("first_pending", crossing.first_pending),
                        ("resumed_after_irq", crossing.resumed_after_irq),
                    ] {
                        value.insert(name.into(), n.map_or(-1, |n| n as i64).into());
                    }
                    value.insert("palette_branch".into(), crossing.palette_branch.map_or(-1, i64::from).into());
                    value.insert("material_complete".into(), crossing.complete().into());
                    value.insert("finish_state_cleared".into(), crossing.finish_state_cleared.into());
                    value.insert(
                        "background_commands".into(),
                        crossing
                            .background_commands
                            .iter()
                            .map(|values| numbers(values.iter().map(|n| u64::from(*n))))
                            .collect::<Vec<_>>()
                            .into(),
                    );
                    value.insert(
                        "background_decompressions".into(),
                        crossing
                            .background_decompressions
                            .iter()
                            .map(|values| numbers(values.iter().map(|n| u64::from(*n))))
                            .collect::<Vec<_>>()
                            .into(),
                    );
                    value.insert(
                        "phase_steps".into(),
                        crossing
                            .phases
                            .iter()
                            .map(|n| sm_binary_evidence::Value::Integer(n.map_or(-1, |n| n as i64)))
                            .collect::<Vec<_>>()
                            .into(),
                    );
                    value.insert(
                        "routine_entries".into(),
                        numbers(difference(routines(crossing.after), routines(crossing.before))),
                    );
                    value.insert(
                        "service_completions".into(),
                        numbers(difference(crossing.after.service_completions, crossing.before.service_completions)),
                    );
                    value.insert(
                        "background_copies".into(),
                        crossing
                            .background_copies
                            .iter()
                            .map(|values| numbers(values.iter().map(|n| u64::from(*n))))
                            .collect::<Vec<_>>()
                            .into(),
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
        && counts.routine_c_entries == [0; 27]
        && counts.routine_rust_entries == counts.routine_rust_completed
        && counts.foreign_requests == counts.foreign_completions
        && counts.child_entries == counts.child_completions
        && (counts.rust_entries == [0; 3]
            || (0..3).all(|consumer| counts.child_completions.iter().any(|row| row[consumer] > 0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn stream(records: &[[u32; 4]]) -> Vec<u8> {
        records.iter().flat_map(|row| row.iter().flat_map(|n| n.to_le_bytes())).collect()
    }
    #[test]
    fn other_loading_copies_cannot_satisfy_background_material() {
        let records = [[20, 13, 0, 0], [22, 21, 0x4800, 0x7e4000], [23, 21, 0x800, 0], [24, 21, 0, 0], [21, 13, 0, 0]];
        assert!(inspect_effects(&stream(&records)).unwrap().background_copies.is_empty());
        let mut background = records;
        background[0][1] = 20;
        background[4][1] = 20;
        assert_eq!(inspect_effects(&stream(&background)).unwrap().background_copies, [[0x4800, 0x7e4000, 0x800]]);
    }
    #[test]
    fn incomplete_or_wrong_service_return_is_rejected() {
        assert!(inspect_effects(&stream(&[[22, 21, 0, 0], [23, 21, 8, 0]])).is_err());
        assert!(inspect_effects(&stream(&[[22, 21, 0, 0], [23, 21, 8, 0], [24, 20, 0, 0]])).is_err());
    }
}
