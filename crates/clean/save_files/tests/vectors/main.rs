//! Runs every vector of `save-files.toml` through the model, with the vectors' invented format.
#![cfg(test)]

mod fixture;
mod sha256;

use enum_map::EnumMap;
use save_files::{
    BehaviourId, CATALOGUE, ChoiceProgress, ChoiceStage, ReadOutcome, ReadReceipt, Refusal,
    SaveCut, SaveFiles, SaveFormat, StartData, WriteStage, selection_of,
};
use serde::Deserialize;
use types::file_select::Slot;
use types::save_files::{
    BatteryImage, Cartridge, CheckPair, Destination, EncodedRun, ExploredCells, IMAGE_BYTES,
    OpaqueValue, PAYLOAD_WORDS, RETAINED_BYTES, RememberedSelection, RetainedBytes,
    RetainedCartridgeData, RunSnapshot, SPARE_WORDS, SaveFilesState, SlotRecord, StoredArea,
    WorkingRun,
};
use types::{Area, Checked, StationId};

use fixture::{Fixture, mark_of};

const VECTORS: &str = include_str!("../../../../../scenarios/vectors/save-files.toml");

/// The mark of a vector that names none: the fixture's other mark, neither completed nor
/// incomplete.
const DEFAULT_MARK: &str = "Other";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorFile {
    vector: Vec<Vector>,
}

/// A run given by its typed fields, each encoded in the fixture's word.
#[derive(Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunFields {
    energy: Option<u16>,
    capacity: Option<u16>,
    destination: Option<String>,
    area: Option<String>,
    station: Option<u16>,
    kept_profile: Option<u16>,
    placement_disabled: Option<u16>,
    startup_word: Option<u16>,
    event: Option<u16>,
    boss: Option<u16>,
    item_count: Option<u16>,
    map: Option<u16>,
    tail: Option<u16>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SlotSpec {
    words: Option<Vec<u16>>,
    run: Option<RunFields>,
    primary: Option<[u16; 2]>,
    mirror: Option<[u16; 2]>,
}

/// A route or area by name, or by its raw code.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum Code {
    Name(String),
    Raw(u16),
}

/// Declares the facts a vector may want, and their check against what was observed.
macro_rules! facts {
    ($($field:ident: $kind:ty),* $(,)?) => {
        #[derive(Debug, Default, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Facts {
            $($field: Option<$kind>,)*
        }

        impl Facts {
            fn check(&self, id: &str, observed: &Self) {
                $(if let Some(want) = &self.$field {
                    assert_eq!(
                        Some(want),
                        observed.$field.as_ref(),
                        "{id}: {}",
                        stringify!($field)
                    );
                })*
            }
        }
    };
}

facts! {
    present: Vec<bool>,
    cursor: String,
    slot: String,
    selection: [u16; 2],
    energy: u16,
    capacity: u16,
    destination: Code,
    valid: bool,
    unchanged: bool,
    cleared: Vec<String>,
    checks_preserved: bool,
    second_valid: bool,
    inspected: Vec<String>,
    mark: String,
    demos: u8,
    image_digest: String,
    area: Code,
    station: u16,
    kept_profile: u16,
    saved_map: u16,
    tail: u16,
    primary: [u16; 2],
    mirror: [u16; 2],
    play_time: [u8; 4],
    placement_disabled: u16,
    startup_word: u16,
    map_collected: bool,
    pending_primary: [u16; 2],
    pending_mirror: [u16; 2],
    pending_digest: String,
    copied_words: Vec<u16>,
    working_profile: u16,
    working_map: u16,
    choice_digest: String,
    choice_saved: bool,
    run_ready_at_selection: bool,
    same_as_continuous: bool,
    round_trip: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    id: String,
    entries: Vec<String>,
    row: Option<String>,
    frames: Option<Vec<u32>>,
    action: Option<String>,
    slot: Option<String>,
    #[serde(rename = "A")]
    slot_a: Option<SlotSpec>,
    #[serde(rename = "B")]
    slot_b: Option<SlotSpec>,
    #[serde(rename = "C")]
    slot_c: Option<SlotSpec>,
    mark: Option<String>,
    selection: Option<[u16; 2]>,
    retained_fill: Option<u8>,
    spare_word: Option<u16>,
    run: Option<RunFields>,
    live_map: Option<u8>,
    working: Option<RunFields>,
    changed_after_checkpoint: Option<RunFields>,
    want: Option<Facts>,
}

fn slot_named(name: &str) -> Slot {
    match name {
        "A" => Slot::First,
        "B" => Slot::Second,
        "C" => Slot::Third,
        other => panic!("unknown slot {other}"),
    }
}

fn slot_name(slot: Slot) -> String {
    match slot {
        Slot::First => "A",
        Slot::Second => "B",
        Slot::Third => "C",
    }
    .to_owned()
}

fn destination_code(name: &str) -> u16 {
    match name {
        "Intro" => fixture::INTRO,
        "CeresArrival" => fixture::CERES_ARRIVAL,
        "ZebesFlight" => fixture::ZEBES_FLIGHT,
        other => panic!("unknown destination {other}"),
    }
}

fn area_code(name: &str) -> u16 {
    match name {
        "Crateria" => fixture::CRATERIA,
        "Ceres" => fixture::CERES,
        other => panic!("unknown area {other}"),
    }
}

fn mark_named(name: &str) -> u8 {
    match name {
        "Completed" => fixture::COMPLETED,
        "Incomplete" => fixture::INCOMPLETE,
        "Other" => fixture::OTHER,
        other => panic!("unknown mark {other}"),
    }
}

fn payload_of(mut words: Vec<u16>) -> EncodedRun {
    words.resize(PAYLOAD_WORDS, 0);
    EncodedRun::new(words).unwrap()
}

/// A typed run's words, in the fixture's order, zero where omitted.
fn run_words(run: &RunFields) -> EncodedRun {
    let mut words = vec![0; PAYLOAD_WORDS];
    let fields = [
        (fixture::ENERGY, run.energy),
        (fixture::CAPACITY, run.capacity),
        (
            fixture::DESTINATION,
            Some(destination_code(
                run.destination.as_deref().unwrap_or("Intro"),
            )),
        ),
        (
            fixture::AREA,
            Some(area_code(run.area.as_deref().unwrap_or("Crateria"))),
        ),
        (fixture::STATION, run.station),
        (fixture::KEPT_PROFILE, run.kept_profile),
        (fixture::PLACEMENT_DISABLED, run.placement_disabled),
        (fixture::STARTUP_WORD, run.startup_word),
        (fixture::EVENT, run.event),
        (fixture::BOSS, run.boss),
        (fixture::ITEM_COUNT, run.item_count),
        (fixture::MAP, run.map),
        (fixture::TAIL, run.tail),
    ];
    for (at, value) in fields {
        words[at] = value.unwrap_or(0);
    }
    payload_of(words)
}

fn pair_of(words: Option<[u16; 2]>) -> CheckPair {
    let [sum, complement] = words.unwrap_or([0, 0]);
    CheckPair { sum, complement }
}

fn pair_words(pair: CheckPair) -> [u16; 2] {
    [pair.sum, pair.complement]
}

fn record_of(spec: &SlotSpec) -> SlotRecord {
    let payload = match (&spec.words, &spec.run) {
        (Some(words), None) => payload_of(words.clone()),
        (None, Some(run)) => run_words(run),
        (None, None) => EncodedRun::zeroed(),
        (Some(_), Some(_)) => panic!("a slot takes either words or a run"),
    };
    SlotRecord {
        payload,
        primary: pair_of(spec.primary),
        mirror: pair_of(spec.mirror),
    }
}

impl Vector {
    fn spec(&self, slot: Slot) -> SlotSpec {
        match slot {
            Slot::First => &self.slot_a,
            Slot::Second => &self.slot_b,
            Slot::Third => &self.slot_c,
        }
        .clone()
        .unwrap_or_default()
    }

    fn slot(&self) -> Slot {
        slot_named(self.slot.as_deref().expect("the vector names a slot"))
    }

    fn cartridge(&self) -> Cartridge {
        let [slot_word, check_word] = self.selection.unwrap_or([u16::MAX, u16::MAX]);
        let fill = self.retained_fill.unwrap_or(0);
        Cartridge {
            slots: EnumMap::from_fn(|slot| record_of(&self.spec(slot))),
            selection: RememberedSelection {
                slot_word,
                check_word,
            },
            mark: mark_of(mark_named(self.mark.as_deref().unwrap_or(DEFAULT_MARK))),
            retained: RetainedCartridgeData {
                bytes: RetainedBytes::new(vec![fill; RETAINED_BYTES]).unwrap(),
                spare_words: [self.spare_word.unwrap_or(0); SPARE_WORDS],
            },
        }
    }

    fn owner(&self) -> SaveFiles<Fixture> {
        let working = Fixture.decode(&run_words(&self.working.clone().unwrap_or_default()));
        SaveFiles::new(
            Fixture,
            SaveFilesState::power_on(
                self.cartridge(),
                WorkingRun {
                    profile: working.profile,
                    maps: working.maps,
                },
            ),
        )
    }

    fn live_run(&self, fields: &RunFields) -> (SaveCut, RunSnapshot, ExploredCells) {
        let run = Fixture.decode(&run_words(fields));
        (
            cut_for(&run),
            run,
            fixture::map_with(self.live_map.unwrap_or(0)),
        )
    }
}

/// The cut that saves a run: the one whose destination it holds.
fn cut_for(run: &RunSnapshot) -> SaveCut {
    match (run.restart.destination, run.restart.area) {
        (Destination::CeresArrival, _) => SaveCut::CeresArrival,
        (Destination::ZebesFlight, StoredArea::Known(area)) => SaveCut::CeresEscape {
            area,
            station: run.restart.station,
        },
        (other, area) => panic!("no cut saves {other:?} in {area:?}"),
    }
}

fn digest(cartridge: &Cartridge) -> String {
    sha256::hex_digest(Fixture.image(cartridge).as_slice())
}

fn raw_code(value: OpaqueValue) -> Code {
    match value {
        OpaqueValue::Byte(byte) => Code::Raw(u16::from(byte)),
        OpaqueValue::Word(word) => Code::Raw(word),
    }
}

fn code_of_destination(destination: Destination) -> Code {
    match destination {
        Destination::Unrecognized(value) => raw_code(value),
        named @ (Destination::Intro | Destination::CeresArrival | Destination::ZebesFlight) => {
            Code::Name(format!("{named:?}"))
        }
    }
}

fn code_of_area(area: StoredArea) -> Code {
    match area {
        StoredArea::Known(known) => Code::Name(format!("{known:?}")),
        StoredArea::Unrecognized(value) => raw_code(value),
    }
}

fn observe_run(facts: &mut Facts, run: &RunSnapshot) {
    let profile = &run.profile;
    facts.destination = Some(code_of_destination(run.restart.destination));
    facts.area = Some(code_of_area(run.restart.area));
    facts.station = Some(run.restart.station.0);
    facts.energy = Some(profile.energy.current);
    facts.capacity = Some(profile.energy.maximum);
    facts.kept_profile = profile.retained.0.first().copied();
    facts.saved_map = Some(u16::from(fixture::kept_map_byte(&run.maps)));
    facts.tail = run.tail.0.first().copied();
    let time = profile.play_time;
    facts.play_time = Some([
        time.hours.get(),
        time.minutes.get(),
        time.seconds.get(),
        time.frames.get(),
    ]);
    facts.placement_disabled = Some(profile.placement_disabled);
    facts.startup_word = Some(profile.startup_word);
    facts.map_collected = Some(run.progress.maps_collected.values().any(|&taken| taken));
}

/// The words of a payload up to its last nonzero one.
fn trimmed(payload: &EncodedRun) -> Vec<u16> {
    let words = payload.as_slice();
    let end = words.iter().rposition(|&word| word != 0);
    end.map_or_else(Vec::new, |last| words[..=last].to_vec())
}

fn cleared_slots(before: &Cartridge, after: &Cartridge) -> Vec<String> {
    EnumMap::<Slot, ()>::default()
        .into_iter()
        .map(|(slot, ())| slot)
        .filter(|&slot| {
            let (was, is) = (&before.slots[slot].payload, &after.slots[slot].payload);
            was != is && *is == EncodedRun::zeroed()
        })
        .map(slot_name)
        .collect()
}

fn checks_preserved(before: &Cartridge, after: &Cartridge) -> bool {
    before
        .slots
        .iter()
        .zip(after.slots.iter())
        .all(|(was, is)| was.1.primary == is.1.primary && was.1.mirror == is.1.mirror)
}

fn observe_cartridge(facts: &mut Facts, before: &Cartridge, after: &Cartridge) {
    facts.unchanged = Some(before == after);
    facts.cleared = Some(cleared_slots(before, after));
    facts.checks_preserved = Some(checks_preserved(before, after));
    facts.selection = Some([after.selection.slot_word, after.selection.check_word]);
    facts.image_digest = Some(digest(after));
}

fn observe_working(facts: &mut Facts, owner: &SaveFiles<Fixture>) {
    let working = &owner.state.working;
    facts.working_profile = working.profile.retained.0.first().copied();
    facts.working_map = Some(u16::from(fixture::kept_map_byte(&working.maps)));
}

fn observe_start(facts: &mut Facts, owner: &SaveFiles<Fixture>, start: &StartData) {
    observe_run(facts, &start.run);
    let slot = owner.state.active.expect("a choice sets the active slot");
    let record = &owner.state.cartridge.slots[slot];
    facts.slot = Some(slot_name(slot));
    facts.valid = Some(start.receipt.is_valid());
    facts.copied_words = Some(trimmed(&start.receipt.copied));
    facts.primary = Some(pair_words(record.primary));
    facts.mirror = Some(pair_words(record.mirror));
}

fn read(vector: &Vector, times: usize) -> Facts {
    let mut owner = vector.owner();
    let before = owner.state.cartridge.clone();
    let receipts: Vec<_> = (0..times)
        .map(|_| owner.read_slot(vector.slot()).unwrap())
        .collect();
    let receipt = &receipts[0];
    let mut facts = Facts {
        valid: Some(receipt.is_valid()),
        second_valid: receipts.get(1).map(ReadReceipt::is_valid),
        copied_words: Some(trimmed(&receipt.copied)),
        round_trip: Some(Fixture.encode(&Fixture.decode(&receipt.copied)) == receipt.copied),
        ..Facts::default()
    };
    match &receipt.outcome {
        ReadOutcome::Decoded(run) => observe_run(&mut facts, run),
        ReadOutcome::Empty { area, station } => {
            facts.area = Some(code_of_area(StoredArea::Known(*area)));
            facts.station = Some(station.0);
        }
    }
    observe_cartridge(&mut facts, &before, &owner.state.cartridge);
    observe_working(&mut facts, &owner);
    facts
}

fn boot(vector: &Vector) -> Facts {
    let mut owner = vector.owner();
    let before = owner.state.cartridge.clone();
    let report = owner.boot_files().unwrap();
    let mark = owner.state.cartridge.mark;
    let mut facts = Facts {
        inspected: Some(report.inspected.iter().map(|r| slot_name(r.slot)).collect()),
        mark: ["Completed", "Incomplete", "Other"]
            .into_iter()
            .find(|&name| mark_of(mark_named(name)) == mark)
            .map(str::to_owned),
        demos: Some(report.demos.sets()),
        ..Facts::default()
    };
    observe_cartridge(&mut facts, &before, &owner.state.cartridge);
    facts
}

fn menu_facts(owner: &mut SaveFiles<Fixture>, facts: &mut Facts) {
    let report = owner.menu_files().unwrap();
    facts.cursor = Some(slot_name(report.cursor));
    facts.present = Some(report.receipts.iter().map(ReadReceipt::is_valid).collect());
}

fn menu(vector: &Vector) -> Facts {
    let mut owner = vector.owner();
    let before = owner.state.cartridge.clone();
    let mut facts = Facts::default();
    menu_facts(&mut owner, &mut facts);
    observe_cartridge(&mut facts, &before, &owner.state.cartridge);
    facts
}

fn loaded(progress: ChoiceProgress) -> StartData {
    match progress {
        ChoiceProgress::Loaded(start) => *start,
        ChoiceProgress::SelectionWritten => panic!("the choice was asked to load"),
    }
}

fn choose(vector: &Vector) -> Facts {
    let mut owner = vector.owner();
    let before = owner.state.cartridge.clone();
    let mut facts = Facts::default();
    menu_facts(&mut owner, &mut facts);
    let progress = owner.choose_file(vector.slot(), ChoiceStage::ChoiceLoaded);
    let start = loaded(progress.unwrap());
    observe_cartridge(&mut facts, &before, &owner.state.cartridge);
    observe_start(&mut facts, &owner, &start);
    facts
}

fn choose_then_save(vector: &Vector) -> Facts {
    let mut owner = vector.owner();
    let before = owner.state.cartridge.clone();
    let mut facts = Facts::default();
    menu_facts(&mut owner, &mut facts);
    let progress = owner.choose_file(vector.slot(), ChoiceStage::ChoiceLoaded);
    let start = loaded(progress.unwrap());
    facts.choice_digest = Some(digest(&owner.state.cartridge));
    let blank = ExploredCells::zeroed();
    owner
        .write_file(
            SaveCut::CeresArrival,
            &start.run,
            &blank,
            WriteStage::ChecksWritten,
        )
        .unwrap();
    observe_cartridge(&mut facts, &before, &owner.state.cartridge);
    observe_start(&mut facts, &owner, &start);
    facts
}

/// The State through a checkpoint: written out as text and read back.
fn restored(owner: &SaveFiles<Fixture>) -> SaveFiles<Fixture> {
    let text = toml::to_string(&owner.state).unwrap();
    SaveFiles::new(Fixture, toml::from_str(&text).unwrap())
}

fn observe_saved(facts: &mut Facts, owner: &SaveFiles<Fixture>, slot: Slot) {
    let record = &owner.state.cartridge.slots[slot];
    observe_run(facts, &Fixture.decode(&record.payload));
    facts.slot = owner.state.active.map(slot_name);
    facts.primary = Some(pair_words(record.primary));
    facts.mirror = Some(pair_words(record.mirror));
}

fn save(vector: &Vector, resumed: bool) -> Facts {
    let slot = vector.slot();
    let mut owner = vector.owner();
    owner.state.active = Some(slot);
    let before = owner.state.cartridge.clone();
    let mut continuous = owner.clone();
    let (cut, run, live_map) = vector.live_run(vector.run.as_ref().expect("a save's run"));
    owner
        .write_file(cut, &run, &live_map, WriteStage::PayloadWritten)
        .unwrap();
    let record = &owner.state.cartridge.slots[slot];
    let mut facts = Facts {
        pending_primary: Some(pair_words(record.primary)),
        pending_mirror: Some(pair_words(record.mirror)),
        pending_digest: Some(digest(&owner.state.cartridge)),
        ..Facts::default()
    };
    if resumed {
        owner = restored(&owner);
        let changed = vector.changed_after_checkpoint.clone().unwrap_or_default();
        let (later_cut, later, later_map) = vector.live_run(&changed);
        let refused = owner.write_file(later_cut, &later, &later_map, WriteStage::ChecksWritten);
        assert_eq!(refused, Err(Refusal::ActionPending), "{}", vector.id);
    }
    owner.continue_write().unwrap();
    continuous
        .write_file(cut, &run, &live_map, WriteStage::ChecksWritten)
        .unwrap();
    facts.same_as_continuous = Some(owner == continuous);
    observe_cartridge(&mut facts, &before, &owner.state.cartridge);
    observe_saved(&mut facts, &owner, slot);
    facts
}

fn resume_choice(vector: &Vector) -> Facts {
    let slot = vector.slot();
    let mut owner = vector.owner();
    let before = owner.state.cartridge.clone();
    let mut facts = Facts::default();
    menu_facts(&mut owner, &mut facts);
    let mut continuous = owner.clone();
    let progress = owner
        .choose_file(slot, ChoiceStage::SelectionWritten)
        .unwrap();
    facts.run_ready_at_selection = Some(matches!(progress, ChoiceProgress::Loaded(_)));
    facts.choice_saved = Some(owner.state.cartridge.selection == selection_of(slot));
    facts.pending_digest = Some(digest(&owner.state.cartridge));
    let mut owner = restored(&owner);
    let start = owner.continue_choice().unwrap();
    let whole = loaded(
        continuous
            .choose_file(slot, ChoiceStage::ChoiceLoaded)
            .unwrap(),
    );
    facts.same_as_continuous = Some(owner == continuous && start == whole);
    observe_cartridge(&mut facts, &before, &owner.state.cartridge);
    observe_start(&mut facts, &owner, &start);
    facts
}

/// An oracle record: its rows are observed on the original. Here its requests run in order on a
/// blank cartridge, and any expected values it gives are checked.
fn oracle(vector: &Vector) -> Facts {
    let blank = BatteryImage::new(vec![0; IMAGE_BYTES]).unwrap();
    let cartridge = Fixture.cartridge(&blank);
    let working = Fixture.decode(&EncodedRun::zeroed());
    let mut owner = SaveFiles::new(
        Fixture,
        SaveFilesState::power_on(
            cartridge,
            WorkingRun {
                profile: working.profile,
                maps: working.maps,
            },
        ),
    );
    let mut facts = Facts::default();
    let last = vector.entries[0].as_str();
    owner.boot_files().unwrap();
    if last == "BootFiles" {
        return facts;
    }
    menu_facts(&mut owner, &mut facts);
    if last == "MenuFiles" {
        return facts;
    }
    let start = loaded(
        owner
            .choose_file(Slot::First, ChoiceStage::ChoiceLoaded)
            .unwrap(),
    );
    facts.selection = Some([
        owner.state.cartridge.selection.slot_word,
        owner.state.cartridge.selection.check_word,
    ]);
    observe_start(&mut facts, &owner, &start);
    if last == "ChooseFile" {
        return facts;
    }
    let row = vector.row.as_deref().unwrap_or_default();
    let cut = if row.contains("arrival") {
        SaveCut::CeresArrival
    } else {
        SaveCut::CeresEscape {
            area: Area::Crateria,
            station: StationId(0),
        }
    };
    let blank_map = ExploredCells::zeroed();
    owner
        .write_file(cut, &start.run, &blank_map, WriteStage::ChecksWritten)
        .unwrap();
    facts
}

fn run_vector(vector: &Vector) -> Facts {
    match vector.action.as_deref() {
        None => oracle(vector),
        Some("read") => read(vector, 1),
        Some("read_twice") => read(vector, 2),
        Some("boot") => boot(vector),
        Some("menu") => menu(vector),
        Some("choose") => choose(vector),
        Some("choose_then_save") => choose_then_save(vector),
        Some("save") => save(vector, false),
        Some("resume_save") => save(vector, true),
        Some("resume_choice") => resume_choice(vector),
        Some(other) => panic!("{}: unknown action {other}", vector.id),
    }
}

fn vectors() -> Vec<Vector> {
    toml::from_str::<VectorFile>(VECTORS).unwrap().vector
}

#[test]
fn every_vector_passes() {
    for vector in vectors() {
        assert!(
            vector.action.is_some() || vector.frames.is_some(),
            "{}: neither an action nor an oracle record",
            vector.id
        );
        let observed = run_vector(&vector);
        if let Some(want) = &vector.want {
            want.check(&vector.id, &observed);
        }
    }
}

fn id_name(id: BehaviourId) -> String {
    format!("{id:?}")
}

#[test]
fn the_catalogue_names_each_spec_entry_once() {
    let names: Vec<String> = CATALOGUE.iter().map(|b| id_name(b.id)).collect();
    assert_eq!(
        names,
        [
            "ReadSlot",
            "BootFiles",
            "MenuFiles",
            "ChooseFile",
            "WriteFile"
        ]
    );
}

#[test]
fn every_entry_checked_by_vectors_has_a_vector() {
    let vectors = vectors();
    for behaviour in CATALOGUE {
        let named = vectors
            .iter()
            .any(|v| v.entries.contains(&id_name(behaviour.id)));
        assert!(
            named || behaviour.checked == Checked::OriginalOnly,
            "{:?} has no vector",
            behaviour.id
        );
    }
    for vector in &vectors {
        for entry in &vector.entries {
            assert!(
                CATALOGUE.iter().any(|b| &id_name(b.id) == entry),
                "{}: unknown entry {entry}",
                vector.id
            );
        }
    }
}

#[test]
fn an_arbitrary_image_round_trips_byte_for_byte() {
    let bytes: Vec<u8> = (0..IMAGE_BYTES)
        .map(|at| u8::try_from(at.wrapping_mul(31) % 251).unwrap())
        .collect();
    let image = BatteryImage::new(bytes).unwrap();
    assert_eq!(Fixture.image(&Fixture.cartridge(&image)), image);
}

#[test]
fn requests_wait_for_a_pending_action_and_a_choice_for_the_menu() {
    let vector = &vectors()[0];
    let mut owner = vector.owner();
    assert_eq!(
        owner.choose_file(Slot::First, ChoiceStage::ChoiceLoaded),
        Err(Refusal::MenuUninspected)
    );
    owner.menu_files().unwrap();
    owner
        .choose_file(Slot::Second, ChoiceStage::SelectionWritten)
        .unwrap();
    assert_eq!(owner.read_slot(Slot::First), Err(Refusal::ActionPending));
    assert_eq!(owner.boot_files(), Err(Refusal::ActionPending));
    assert_eq!(owner.continue_write(), Err(Refusal::NothingToComplete));
    owner.continue_choice().unwrap();
    assert_eq!(owner.continue_choice(), Err(Refusal::NothingToComplete));
}
