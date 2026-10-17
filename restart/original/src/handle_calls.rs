//! Fixed original HandleProjectile calls, each seeded independently at entry.
//! Capture authentication belongs to the batch caller, once before selection.
use crate::{Result, Table};
use std::fs;
use std::path::Path;

const WRAM: usize = 0x20000;
const HEADER: &str = "frame,event,kind,address,value,call,game_state,room,x,y,held,pressed,projectile_count,slot,type,instruction,interrupts";
const ENTRY: u64 = 0x90aece;
const EXIT: u64 = 0x90aef2;
const RETURN: u64 = 0x90dd03;
// Frozen before execution, from capture04's complete call inventory. Exit is
// an RTS witness, not a separate output snapshot. No result-based selection.
const EVENTS: [[u64; 3]; 34] = [
    [73926, 73931, 73932],
    [76140, 76145, 76146],
    [78355, 78360, 78361],
    [80569, 80574, 80575],
    [82782, 82787, 82788],
    [84996, 85001, 85002],
    [87211, 87216, 87217],
    [89425, 89430, 89431],
    [91638, 91643, 91644],
    [93851, 93856, 93857],
    [96064, 96069, 96070],
    [98277, 98282, 98283],
    [100490, 100495, 100496],
    [102703, 102708, 102709],
    [104916, 104921, 104922],
    [107129, 107136, 107137],
    [109345, 109346, 109347],
    [111556, 111557, 111558],
    [113766, 113767, 113768],
    [115975, 115976, 115977],
    [118185, 118186, 118187],
    [120396, 120397, 120398],
    [122606, 122607, 122608],
    [124815, 124816, 124817],
    [127024, 127025, 127026],
    [129233, 129234, 129235],
    [131442, 131443, 131444],
    [133651, 133652, 133653],
    [135860, 135861, 135862],
    [138069, 138070, 138071],
    [140278, 140279, 140280],
    [142487, 142488, 142489],
    [144696, 144697, 144698],
    [146905, 146906, 146907],
];

pub struct Call {
    pub id: u64,
    pub frame: u64,
    pub entry_event: u64,
    pub exit_event: u64,
    pub return_event: u64,
    pub entry_cpu: Vec<u8>,
    pub return_cpu: Vec<u8>,
    pub input: Vec<u8>,
    pub original: Vec<u8>,
    pub ledger: Vec<u8>,
    pub slot: usize,
    pub hardware_events: u64,
    pub sound_requests: u64,
}

fn require(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}

#[derive(Clone)]
struct Event {
    kind: String,
    numbers: [u64; 17],
    slot: i64,
}
impl Event {
    fn parse(line: &str) -> Result<Self> {
        let fields: Vec<_> = line.split(',').collect();
        require(fields.len() == 17, "original Handle ledger extent differs")?;
        let mut numbers = [0; 17];
        for i in (0..17).filter(|i| ![2, 13].contains(i)) {
            numbers[i] = fields[i].parse().map_err(|_| format!("invalid original ledger field {i}"))?;
        }
        let slot = fields[13].parse().map_err(|_| "invalid original ledger slot")?;
        require((-1..10).contains(&slot), "original ledger slot outside observer domain")?;
        Ok(Self { kind: fields[2].into(), numbers, slot })
    }
    fn n(&self, index: usize) -> u64 {
        self.numbers[index]
    }
}

struct Span {
    events: Vec<Event>,
    ledger: Vec<u8>,
}

fn spans(ledger: &str) -> Result<Vec<Span>> {
    let mut lines = ledger.lines();
    require(lines.next() == Some(HEADER), "original Handle ledger header differs")?;
    let mut spans = Vec::new();
    let mut active: Option<Span> = None;
    let mut last_frame = 0;
    for (index, line) in lines.enumerate() {
        let event = Event::parse(line)?;
        require(
            event.n(1) == index as u64 + 1 && index < 500_000 && event.n(0) >= last_frame && event.n(0) <= 8850,
            "original Handle ledger event/frame order differs",
        )?;
        last_frame = event.n(0);
        if event.kind == "handle-entry" {
            require(active.is_none() && spans.len() < EVENTS.len(), "duplicate or nested original Handle entry")?;
            active = Some(Span { events: Vec::new(), ledger: format!("{HEADER}\n").into_bytes() });
        }
        if let Some(span) = active.as_mut() {
            span.ledger.extend_from_slice(line.as_bytes());
            span.ledger.push(b'\n');
            span.events.push(event.clone());
            require(span.events.len() <= 9, "original Handle span exceeds fixed bound")?;
            if event.kind == "handle-return" {
                let finished = active.take().unwrap();
                validate_span(&finished, spans.len())?;
                spans.push(finished);
            }
        } else if event.kind.starts_with("handle-") {
            return Err("original Handle witness outside call".into());
        }
    }
    require(active.is_none() && spans.len() == EVENTS.len(), "original Handle inventory is not all 34 complete pairs")?;
    Ok(spans)
}

fn validate_span(span: &Span, index: usize) -> Result<()> {
    let frame = 8661 + index as u64;
    let id = 2 + 3 * index as u64;
    let events = &span.events;
    let flight = index <= 15;
    let impact = index == 15;
    let expected_length = 3 + if flight { 4 } else { 0 } + if impact { 2 } else { 0 };
    require(events.len() == expected_length, "original Handle effect/witness count differs")?;
    for event in events {
        require(
            event.n(0) == frame && event.n(5) == id && event.n(16) == 0 && event.slot == events[0].slot,
            "original Handle frame/call/slot/interrupt identity differs",
        )?;
    }
    for (event, kind, address, identity) in [
        (&events[0], "handle-entry", ENTRY, EVENTS[index][0]),
        (&events[events.len() - 2], "handle-exit", EXIT, EVENTS[index][1]),
        (&events[events.len() - 1], "handle-return", RETURN, EVENTS[index][2]),
    ] {
        require(
            event.kind == kind && event.n(3) == address && event.n(4) == 0 && event.n(1) == identity,
            "original Handle entry/RTS/continuation identity differs",
        )?;
    }
    if flight {
        for (event, (kind, address)) in events[1..5].iter().zip([
            ("hardware-write", 0x944202),
            ("hardware-write", 0x944203),
            ("hardware-read", 0x944216),
            ("hardware-read", 0x944217),
        ]) {
            require(
                event.kind == kind && event.n(3) == address && event.n(4) <= 255,
                "original Handle effect is outside bounded multiplication MMIO",
            )?;
        }
    }
    if impact {
        require(
            events[5].kind == "sfx2-max6-entry"
                && events[5].n(3) == 0x8090cb
                && events[5].n(4) == 12
                && events[6].kind == "semantic-status"
                && events[6].n(3) == 0x8090cb
                && events[6].n(4) <= 255
                && events[6].n(4) & 0x30 == 0,
            "original Handle impact sound or semantic CPU width differs",
        )?;
    }
    // This exhaustive grammar rejects PLM, interrupts, additional sounds,
    // nested roots and hardware outside the predeclared multiplication span.
    Ok(())
}

fn read_state(capture: &Path, event: u64, extension: &str) -> Result<Vec<u8>> {
    let path = capture.join(format!("states/state-{event}.{extension}"));
    require(
        fs::symlink_metadata(capture.join("states")).map_err(|e| e.to_string())?.is_dir(),
        "original state directory is not a regular directory",
    )?;
    require(
        fs::symlink_metadata(&path).map_err(|e| e.to_string())?.is_file(),
        "original Handle snapshot is not a regular file",
    )?;
    let length = fs::metadata(&path).map_err(|e| e.to_string())?.len();
    require(
        if extension == "wram" { length == WRAM as u64 } else { length <= 4096 },
        "original Handle snapshot extent differs",
    )?;
    fs::read(path).map_err(|e| e.to_string())
}

fn cpu(bytes: &[u8], event: &Event, phase: &str) -> Result<Table> {
    let table: Table =
        toml::from_str(std::str::from_utf8(bytes).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let number = |key| number(&table, key);
    require(
        table.get("schema").and_then(toml::Value::as_str) == Some("original-cpu-context-v1")
            && table.get("phase").and_then(toml::Value::as_str) == Some(phase)
            && number("boundary_pc")? == event.n(3) as i64
            && number("call")? == event.n(5) as i64
            && number("pb")? == 0x90
            && number("pc")? == ((event.n(3) + 1) & 0xffff) as i64
            && number("e")? == 0
            && number("d")? == 0
            && number("db")? == 0x90
            && number("p")? & 0x30 == 0
            && number("interrupts")? == 0
            && number("saved_status")? == -1,
        "original Handle CPU mode, widths, bank, call or phase differs",
    )?;
    for key in ["a", "x", "y", "s", "d", "pc", "entry_s", "expected_return_s"] {
        require((0..=65535).contains(&number(key)?), "original Handle CPU register outside word")?;
    }
    require((0..=255).contains(&number("p")?), "original Handle status outside byte")?;
    Ok(table)
}
fn number(table: &Table, key: &str) -> Result<i64> {
    table.get(key).and_then(toml::Value::as_integer).ok_or_else(|| format!("missing original Handle CPU field {key}"))
}

fn cpu_pair(entry: &Table, returned: &Table, input: &[u8], original: &[u8]) -> Result<()> {
    let s = usize::try_from(number(entry, "s")?).map_err(|_| "negative original Handle stack")?;
    require((0x100..0x1ffe).contains(&s), "original Handle stack outside WRAM mirror")?;
    let continuation = 0x900000 | u64::from(word(input, s + 1).wrapping_add(1));
    require(
        continuation == RETURN
            && word(original, s + 1) == word(input, s + 1)
            && number(entry, "entry_s")? == s as i64
            && number(entry, "expected_return_s")? == (s + 2) as i64
            && number(entry, "continuation_pc")? == RETURN as i64
            && number(entry, "exit_pc")? == 0
            && number(returned, "exit_pc")? == EXIT as i64
            && number(returned, "s")? == (s + 2) as i64,
        "original Handle ordinary JSR/RTS linkage differs",
    )?;
    for key in ["entry_s", "expected_return_s", "continuation_pc", "saved_status"] {
        require(number(entry, key)? == number(returned, key)?, "original Handle CPU linkage changed")?;
    }
    Ok(())
}

fn snapshot_context(event: &Event, ram: &[u8]) -> Result<()> {
    require((0..5).contains(&event.slot), "original Handle selected slot outside ordinary pool")?;
    let slot = event.slot as usize;
    for (field, address) in [
        (6, 0x998),
        (7, 0x79b),
        (8, 0xaf6),
        (9, 0xafa),
        (10, 0x8b),
        (11, 0x8f),
        (12, 0xcce),
        (14, 0xc18 + 2 * slot),
        (15, 0xc40 + 2 * slot),
    ] {
        require(event.n(field) == u64::from(word(ram, address)), "original Handle ledger/WRAM identity differs")?;
    }
    require(
        word(ram, 0x998) == 8
            && word(ram, 0x79f) == 6
            && word(ram, 0x79b) == 0xdf45
            && word(ram, 0x93f) & 0x8000 == 0
            && [0x9a6, 0xa76, 0xac4, 0xac6, 0xcd2].into_iter().all(|at| word(ram, at) == 0),
        "original Handle is outside ordinary preescape inventory",
    )?;
    for other in 0..10 {
        if other != slot {
            require(
                word(ram, 0xc40 + 2 * other) == 0 && word(ram, 0xc2c + 2 * other) == 0,
                "original Handle has another projectile or bomb slot",
            )?;
        }
    }
    Ok(())
}

fn eligibility(index: usize, entry: &Event, returned: &Event, input: &[u8], original: &[u8]) -> Result<usize> {
    require(input.len() == WRAM && original.len() == WRAM, "original Handle requires full WRAM")?;
    snapshot_context(entry, input)?;
    snapshot_context(returned, original)?;
    let slot = entry.slot as usize;
    require(entry.slot == returned.slot, "original Handle output slot changed")?;
    let expected_entry = if index <= 15 { (0x8000, 0xaef3) } else { (0x8700, 0xb169) };
    let active = |ram: &[u8], expected: (u16, u16)| {
        word(ram, 0xcce) == 1
            && word(ram, 0xc40 + 2 * slot) >= 0x8000
            && word(ram, 0xc2c + 2 * slot) != 0
            && (word(ram, 0xc18 + 2 * slot), word(ram, 0xc68 + 2 * slot)) == expected
    };
    require(active(input, expected_entry), "original Handle entry lacks the declared beam/explosion")?;
    if index == EVENTS.len() - 1 {
        require(
            word(original, 0xcce) == 0
                && word(original, 0xc40 + 2 * slot) == 0
                && word(original, 0xc18 + 2 * slot) == 0
                && word(original, 0xc2c + 2 * slot) == 0,
            "last original Handle did not remove its projectile",
        )?;
    } else {
        require(
            active(original, if index < 15 { (0x8000, 0xaef3) } else { (0x8700, 0xb169) }),
            "original Handle output left the declared beam/explosion",
        )?;
    }
    Ok(slot)
}

pub fn select(capture: &Path) -> Result<Vec<Call>> {
    let path = capture.join("events.csv");
    let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    require(
        metadata.is_file() && metadata.len() <= 64 * 1024 * 1024,
        "original Handle ledger is not bounded regular data",
    )?;
    let ledger = fs::read_to_string(path).map_err(|e| e.to_string())?;
    spans(&ledger)?
        .into_iter()
        .enumerate()
        .map(|(index, span)| {
            let entry = &span.events[0];
            let returned = span.events.last().unwrap();
            let input = read_state(capture, entry.n(1), "wram")?;
            let original = read_state(capture, returned.n(1), "wram")?;
            let entry_cpu = read_state(capture, entry.n(1), "cpu.toml")?;
            let return_cpu = read_state(capture, returned.n(1), "cpu.toml")?;
            let e = cpu(&entry_cpu, entry, "entry")?;
            let r = cpu(&return_cpu, returned, "return")?;
            cpu_pair(&e, &r, &input, &original)?;
            let slot = eligibility(index, entry, returned, &input, &original)?;
            Ok(Call {
                id: entry.n(5),
                frame: entry.n(0),
                entry_event: entry.n(1),
                exit_event: span.events[span.events.len() - 2].n(1),
                return_event: returned.n(1),
                entry_cpu,
                return_cpu,
                input,
                original,
                ledger: span.ledger,
                slot,
                hardware_events: u64::from(index <= 15) * 4,
                sound_requests: u64::from(index == 15),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(index: usize) -> Span {
        let mut events = Vec::new();
        let mut push = |kind: &str, address, value| {
            let mut numbers = [0; 17];
            numbers[0] = 8661 + index as u64;
            numbers[1] = EVENTS[index][0] + events.len() as u64;
            numbers[3] = address;
            numbers[4] = value;
            numbers[5] = 2 + 3 * index as u64;
            events.push(Event { kind: kind.into(), numbers, slot: 0 });
        };
        push("handle-entry", ENTRY, 0);
        if index <= 15 {
            for (kind, address, value) in [
                ("hardware-write", 0x944202, 4),
                ("hardware-write", 0x944203, 16),
                ("hardware-read", 0x944216, 64),
                ("hardware-read", 0x944217, 0),
            ] {
                push(kind, address, value);
            }
        }
        if index == 15 {
            push("sfx2-max6-entry", 0x8090cb, 12);
            push("semantic-status", 0x8090cb, 0);
        }
        push("handle-exit", EXIT, 0);
        push("handle-return", RETURN, 0);
        Span { events, ledger: Vec::new() }
    }

    #[test]
    fn all_fixed_pairs_include_flight_impact_and_removal_without_drawing() {
        for index in 0..34 {
            validate_span(&span(index), index).unwrap();
        }
        let mut wrong_root = span(0);
        wrong_root.events[0].kind = "draw-player-entry".into();
        assert!(validate_span(&wrong_root, 0).is_err());
        let mut wrong_id = span(0);
        wrong_id.events[1].numbers[5] += 1;
        assert!(validate_span(&wrong_id, 0).is_err());
        let mut wrong_frame = span(0);
        wrong_frame.events.last_mut().unwrap().numbers[0] += 1;
        assert!(validate_span(&wrong_frame, 0).is_err());
        let mut missing_exit = span(0);
        missing_exit.events.remove(5);
        assert!(validate_span(&missing_exit, 0).is_err());
    }

    #[test]
    fn interrupts_plm_unexpected_mmio_and_bad_sound_status_are_rejected() {
        for kind in ["nmi-entry", "irq-entry", "plm-entry", "sfx1-max15-entry", "handle-entry"] {
            let mut bad = span(0);
            bad.events[1].kind = kind.into();
            assert!(validate_span(&bad, 0).is_err(), "{kind}");
        }
        let mut bad = span(0);
        bad.events[1].numbers[3] = 0x002132;
        assert!(validate_span(&bad, 0).is_err());
        let mut bad = span(15);
        bad.events[6].numbers[4] = 0x20;
        assert!(validate_span(&bad, 15).is_err());
        let mut bad = span(15);
        bad.events[5].numbers[4] = 11;
        assert!(validate_span(&bad, 15).is_err());
        let mut bad = span(16);
        bad.events[2].numbers[16] = 1;
        assert!(validate_span(&bad, 16).is_err());
    }

    fn line(event: &Event) -> String {
        (0..17)
            .map(|i| match i {
                2 => event.kind.clone(),
                13 => event.slot.to_string(),
                _ => event.n(i).to_string(),
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn selector_never_reduces_missing_duplicate_or_incomplete_inventory() {
        let mut ledger = format!("{HEADER}\n");
        let calls: Vec<_> = (0..34).map(span).collect();
        let selected: std::collections::BTreeMap<_, _> =
            calls.iter().flat_map(|s| &s.events).map(|e| (e.n(1), e)).collect();
        let mut frame = 1;
        for id in 1..=EVENTS[33][2] {
            let text = if let Some(event) = selected.get(&id) {
                frame = event.n(0);
                line(event)
            } else {
                format!("{frame},{id},other,0,0,0,0,0,0,0,0,0,0,-1,0,0,0")
            };
            ledger.push_str(&text);
            ledger.push('\n');
        }
        assert_eq!(spans(&ledger).unwrap().len(), 34);
        let without_return = ledger[..ledger.trim_end().rfind('\n').unwrap() + 1].to_owned();
        assert!(spans(&without_return).is_err());
        let missing = ledger.replace(
            &line(&calls[33].events[0]),
            &line(&Event { kind: "other".into(), ..calls[33].events[0].clone() }),
        );
        assert!(spans(&missing).is_err());
        let duplicate = ledger.replace(&line(&calls[1].events[0]), &line(&calls[0].events[0]));
        assert!(spans(&duplicate).is_err());
    }

    fn cpu_bytes(event: &Event, phase: &str) -> Vec<u8> {
        format!("schema = \"original-cpu-context-v1\"\nphase = \"{phase}\"\na = 0\nx = 0\ny = 0\ns = {}\nd = 0\ndb = 144\npb = 144\npc = {}\np = 2\ne = 0\nboundary_pc = {}\ncall = {}\ncontinuation_pc = {RETURN}\nentry_s = 8177\nexpected_return_s = 8179\nexit_pc = {}\ninterrupts = 0\nsaved_status = -1\n",
            if phase == "entry" {8177} else {8179}, (event.n(3)+1)&65535, event.n(3), event.n(5),
            if phase == "entry" {0} else {EXIT}).into_bytes()
    }
    fn set(ram: &mut [u8], at: usize, value: u16) {
        ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn ordinary_jsr_requires_two_stack_bytes_and_no_inherited_php() {
        let span = span(0);
        let entry = &span.events[0];
        let returned = span.events.last().unwrap();
        let e = cpu(&cpu_bytes(entry, "entry"), entry, "entry").unwrap();
        let r = cpu(&cpu_bytes(returned, "return"), returned, "return").unwrap();
        let mut ram = vec![0; WRAM];
        set(&mut ram, 0x1ff2, 0xdd02);
        cpu_pair(&e, &r, &ram, &ram).unwrap();
        let mut php = e.clone();
        php.insert("expected_return_s".into(), 8180.into());
        php.insert("saved_status".into(), 2.into());
        assert!(cpu_pair(&php, &r, &ram, &ram).is_err());
        assert!(cpu(toml::to_string(&php).unwrap().as_bytes(), entry, "entry").is_err());
        for (key, value) in [("d", 1), ("e", 1), ("p", 0x20), ("db", 0xa0), ("interrupts", 1), ("pc", 0xaece)] {
            let mut bad = e.clone();
            bad.insert(key.into(), value.into());
            assert!(cpu(toml::to_string(&bad).unwrap().as_bytes(), entry, "entry").is_err(), "{key}");
        }
        set(&mut ram, 0x1ff2, 0xdd03);
        assert!(cpu_pair(&e, &r, &ram, &ram).is_err());
    }

    fn input_and_event() -> (Vec<u8>, Event) {
        let mut ram = vec![0; WRAM];
        for (at, value) in [
            (0x998, 8),
            (0x79f, 6),
            (0x79b, 0xdf45),
            (0xcce, 1),
            (0xc18, 0x8000),
            (0xc40, 0x86f3),
            (0xc2c, 1),
            (0xc68, 0xaef3),
        ] {
            set(&mut ram, at, value);
        }
        let mut event = span(0).events.remove(0);
        for (field, at) in [(6, 0x998), (7, 0x79b), (12, 0xcce), (14, 0xc18), (15, 0xc40)] {
            event.numbers[field] = word(&ram, at).into();
        }
        (ram, event)
    }

    #[test]
    fn entry_and_output_must_remain_in_fixed_ordinary_slot_domain() {
        let (ram, event) = input_and_event();
        assert_eq!(eligibility(0, &event, &event, &ram, &ram).unwrap(), 0);
        for (at, value) in [
            (0x9a6, 1),
            (0xa76, 1),
            (0x93f, 0x8000),
            (0xac4, 1),
            (0xac6, 1),
            (0xcd2, 1),
            (0xc4a, 0x86f3),
            (0xc2e, 1),
            (0xc68, 0xb169),
            (0xc2c, 0),
        ] {
            let mut bad = ram.clone();
            set(&mut bad, at, value);
            assert!(eligibility(0, &event, &event, &bad, &ram).is_err(), "{at:x}");
        }
        let mut bad = event.clone();
        bad.slot = 5;
        assert!(eligibility(0, &bad, &bad, &ram, &ram).is_err());
        let mut bad = event.clone();
        bad.numbers[8] = 1;
        assert!(eligibility(0, &bad, &event, &ram, &ram).is_err());
        assert!(eligibility(0, &event, &event, &ram[..100], &ram).is_err());
    }
}
