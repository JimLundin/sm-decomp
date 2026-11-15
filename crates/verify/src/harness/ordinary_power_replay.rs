//! One seeded production replay; oracle State is used only after each actual engine step.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::Path;
use std::sync::Arc;

use canonical::Canonical;
use game::state::{State, Tick};
use game::{Schedule, Session, SessionFault, TickInput, TimingSource};
use weapons_projectiles as beam;

use crate::cache::Cache;
use crate::decode::decode_ordinary;
use crate::lock::{Lock, RowId, Sha256, Start};

// Only structural path segments are reportable; Difference also owns observed bytes.
fn difference_path(difference: &canonical::Difference) -> String {
    let mut path = String::new();
    for (position, segment) in difference.path().iter().enumerate() {
        match (position, segment) {
            (0, canonical::Segment::Field(name) | canonical::Segment::Variant(name)) => {
                write!(path, "{name}").unwrap();
            }
            (_, canonical::Segment::Field(name) | canonical::Segment::Variant(name)) => {
                write!(path, ".{name}").unwrap();
            }
            (0, canonical::Segment::Len) => path.push_str("len"),
            (_, canonical::Segment::Len) => path.push_str(".len"),
            (_, canonical::Segment::Index(index)) => write!(path, "[{index}]").unwrap(),
        }
    }
    if path.is_empty() {
        path.push_str("(value)");
    }
    path
}

// Diagnose every top-level block concurrently without changing either compared State.
fn first_cut_paths(actual: &State, expected: &State) -> Vec<String> {
    let mut paths = Vec::new();
    macro_rules! compare {
        ($($field:ident).+) => {
            if let Some(difference) = actual.$($field).+.diff(&expected.$($field).+) {
                paths.push(format!("{}:{}", stringify!($($field).+), difference_path(&difference)));
            }
        };
    }
    compare!(clock);
    compare!(rng);
    compare!(controller);
    compare!(mode);
    compare!(main);
    compare!(irq);
    compare!(timing);
    compare!(video);
    compare!(samus);
    compare!(camera);
    compare!(movement_travel);
    compare!(pending_aim);
    compare!(atmospherics);
    compare!(room);
    compare!(elevator);
    compare!(enemy_palette_cycle);
    compare!(ceres);
    compare!(hud);
    compare!(message);
    compare!(collected_items);
    compare!(sram);
    compare!(projectiles);
    compare!(clock.tick);
    compare!(rng.calls);
    compare!(timing.owed);
    compare!(video.batch.registers.colour_math.fixed);
    compare!(samus.aim);
    compare!(samus.invincibility);
    compare!(room.entered_by);
    compare!(room.drawing.queues);
    compare!(room.drawing.routine_count);
    compare!(room.room);
    compare!(room.entered_by);
    compare!(room.blocks);
    compare!(room.width);
    compare!(room.height);
    compare!(room.layer2);
    compare!(room.level_tail);
    compare!(room.area);
    compare!(room.tileset);
    compare!(room.tilemap_gap);
    compare!(room.layer_blending);
    compare!(room.scroll_areas);
    compare!(room.scrolls_below);
    compare!(room.camera);
    compare!(room.plms);
    compare!(room.item_gfx);
    compare!(room.enemies);
    compare!(room.drawing);
    compare!(room.drawing.hook);
    let actual_plm_generations = actual.room.plms.slots.map(|slot| slot.generation);
    let expected_plm_generations = expected.room.plms.slots.map(|slot| slot.generation);
    if let Some(difference) = actual_plm_generations.diff(&expected_plm_generations) {
        paths.push(format!(
            "room.plms.slots.generation:{}",
            difference_path(&difference)
        ));
    }
    let actual_enemy_generations = actual.room.enemies.slots.map(|slot| slot.generation);
    let expected_enemy_generations = expected.room.enemies.slots.map(|slot| slot.generation);
    if let Some(difference) = actual_enemy_generations.diff(&expected_enemy_generations) {
        paths.push(format!(
            "room.enemies.slots.generation:{}",
            difference_path(&difference)
        ));
    }
    paths
}

// V2 closed-interval certificate is evaluated only on independently decoded records.
fn closed_outer_boundary(state: &State, record: &oracle_record::FrameRecord) -> bool {
    use game::state::strand::{Frame, MainLoopAt, Wait};
    matches!(record.cut, oracle_record::Cut::NmiReturn(nmi)
        if nmi.kind == oracle_record::NmiKind::Requested)
        && state.main.waiting == Some(Wait::Nmi)
        && matches!(
            state.main.frames.as_slice(),
            [Frame::MainLoop(MainLoopAt::Start)]
        )
}

fn fault_name(fault: SessionFault) -> String {
    match fault {
        SessionFault::StrandEmpty => "StrandEmpty".into(),
        SessionFault::StrandTooDeep => "StrandTooDeep".into(),
        SessionFault::StepBudgetExceeded => "StepBudgetExceeded".into(),
        SessionFault::TickAbandoned => "TickAbandoned".into(),
        SessionFault::Plm(_) => "Plm".into(),
        SessionFault::Blocks(_) => "Blocks".into(),
        SessionFault::Projectile(_) => "Projectile".into(),
        SessionFault::Power(fault) => format!("Power::{fault:?}"),
        SessionFault::Samus(_) => "Samus".into(),
        SessionFault::Room(_) => "Room".into(),
        SessionFault::Hud(_) => "Hud".into(),
        SessionFault::Passage(fault) => format!("Passage::{fault:?}"),
    }
}

fn milestone(state: &State, frame: u64) -> bool {
    let Some(owner) = state.projectiles.ordinary() else {
        return false;
    };
    let flying = owner
        .model
        .0
        .slots
        .values()
        .any(|shot| matches!(shot, beam::ShotState::Flying { .. }));
    let consumed = owner
        .model
        .0
        .slots
        .values()
        .any(|shot| matches!(shot, beam::ShotState::Flying { consumed: true, .. }));
    match frame {
        157..=159 => owner.model.0.count.0 == 1 && flying && !consumed,
        160 => {
            let phase = if state.clock.counters.completed.0.is_multiple_of(2) {
                beam::FlickerPhase::Even
            } else {
                beam::FlickerPhase::Odd
            };
            let view = beam::DrawView::Normal(beam::ScreenOrigin(beam::Axes {
                x: state.room.camera.x.pixels,
                y: state.room.camera.y.pixels,
            }));
            owner.model.0.count.0 == 1
                && consumed
                && !beam::pictures(&owner.model.0, phase, &view)
                    .flying
                    .is_empty()
        }
        161 => owner.model.0.count.0 == 0 && !flying,
        _ => true,
    }
}

#[derive(Default)]
struct ReplayProof {
    prior_closed: bool,
    prior_gameplay: bool,
    rng_receipt: Option<u32>,
    rng_expectation_missing: u64,
    reached: u64,
    compared: u64,
    whole_state_equal: u64,
    owner_equal: u64,
    oam_equal: u64,
    milestones: [bool; 5],
    first_mismatches: BTreeMap<String, u64>,
    runtime_boundary: Option<String>,
    runtime_fault: Option<SessionFault>,
    reference_boundary: Option<String>,
    state_uncompared: u64,
}

impl ReplayProof {
    fn expect(
        &mut self,
        reference_decoded: Result<crate::decode::Decoded, crate::decode::DecodeError>,
        reference: &oracle_record::FrameRecord,
        actual: &State,
        frame: u64,
        oam_matches: bool,
    ) -> Option<(crate::decode::Decoded, u32)> {
        let expected = match reference_decoded {
            Ok(decoded) => decoded,
            Err(error) => {
                self.reference_boundary.get_or_insert_with(|| {
                    format!("{} {:?}", frame, crate::lockstep::ErrorKind::of(&error))
                });
                self.state_uncompared = self.state_uncompared.saturating_add(1);
                self.prior_closed = false;
                self.prior_gameplay = false;
                self.rng_receipt = None;
                if (157..=161).contains(&frame) {
                    let index = usize::try_from(frame.checked_sub(157).unwrap()).unwrap();
                    self.milestones[index] = milestone(actual, frame);
                    let _ = writeln!(
                        std::io::stdout(),
                        "ordinary milestone {} behavior={} State=uncompared owner=uncompared wholeOAM={}",
                        frame,
                        self.milestones[index],
                        oam_matches
                    );
                }
                // A missing independent expectation does not stop or reseed production.
                return None;
            }
        };
        let next_closed = closed_outer_boundary(&expected.state, reference);
        self.rng_receipt = self
            .rng_receipt
            .filter(|_| self.prior_closed && self.prior_gameplay && next_closed)
            .map(|count| count.wrapping_add(1));
        self.prior_closed = next_closed;
        self.prior_gameplay = expected.state.mode == game::state::GameMode::Gameplay;
        let Some(calls) = self.rng_receipt else {
            self.rng_expectation_missing = self.rng_expectation_missing.saturating_add(1);
            self.state_uncompared = self.state_uncompared.saturating_add(1);
            let _ = writeln!(
                std::io::stdout(),
                "ordinary reference RNG phase expectation missing at cut {frame}"
            );
            return None;
        };
        Some((expected, calls))
    }
    fn compare(
        &mut self,
        actual: &State,
        expected: &mut crate::decode::Decoded,
        frame: u64,
        calls: u32,
        oam_matches: bool,
    ) {
        expected.state.rng.calls = calls;
        // Decoder defaults this Session-local count because the console has no
        // equivalent. The row's native seed establishes zero; each indexed cut
        // follows exactly frame+1 steps, independently of the actual Session.
        expected.state.clock.tick = Tick(frame.checked_add(1).unwrap());
        if frame == 0 {
            for path in first_cut_paths(actual, &expected.state) {
                let _ = writeln!(std::io::stdout(), "ordinary first-cut canonical gap {path}");
            }
        }
        self.compared = self.compared.saturating_add(1);
        let state_difference = actual.diff(&expected.state);
        let owner_difference = actual.projectiles.diff(&expected.state.projectiles);
        self.whole_state_equal = self
            .whole_state_equal
            .saturating_add(u64::from(state_difference.is_none()));
        self.owner_equal = self
            .owner_equal
            .saturating_add(u64::from(owner_difference.is_none()));
        if let Some(difference) = state_difference {
            self.first_mismatches
                .entry(format!("whole State:{}", difference_path(&difference)))
                .or_insert(frame);
        }
        if let Some(difference) = owner_difference {
            self.first_mismatches
                .entry(format!(
                    "complete ordinary owner:{}",
                    difference_path(&difference)
                ))
                .or_insert(frame);
        }

        for difference in crate::lockstep::differences(actual, &expected.state, expected.quiescence)
        {
            self.first_mismatches
                .entry(format!("decoded field:{}", difference_path(&difference)))
                .or_insert(frame);
        }
        if (157..=161).contains(&frame) {
            let index = usize::try_from(frame.checked_sub(157).unwrap()).unwrap();
            self.milestones[index] = milestone(actual, frame);
            let _ = writeln!(
                std::io::stdout(),
                "ordinary milestone {} behavior={} State={} owner={} wholeOAM={}",
                frame,
                self.milestones[index],
                actual == &expected.state,
                actual.projectiles == expected.state.projectiles,
                oam_matches
            );
        }
    }
    fn finish(&self, actual: &State) {
        let reached = self.reached;
        let compared = self.compared;
        let whole_state_equal = self.whole_state_equal;
        let owner_equal = self.owner_equal;
        let oam_equal = self.oam_equal;
        let state_uncompared = self.state_uncompared;
        let rng_expectation_missing = self.rng_expectation_missing;
        let _ = writeln!(
            std::io::stdout(),
            "ordinary replay reached={reached} State-compared={compared} whole-State={whole_state_equal} complete-owner={owner_equal} whole-OAM={oam_equal} State-uncompared={state_uncompared} RNG-expectation-missing={rng_expectation_missing}; runtime-boundary={} first-reference-boundary={}",
            self.runtime_boundary
                .as_deref()
                .unwrap_or("record cache exhausted"),
            self.reference_boundary.as_deref().unwrap_or("none")
        );
        for (path, frame) in &self.first_mismatches {
            let _ = writeln!(
                std::io::stdout(),
                "ordinary first mismatch {path} at {frame}"
            );
        }
        assert!(
            self.reached >= 183,
            "ordinary production accepted full-cut minimum must not regress"
        );
        assert!(
            self.runtime_fault
                == Some(SessionFault::Passage(
                    game::selected_doorway::PassageFault::MissingRuntime
                )),
            "production must reach the actual named missing-passage dispatch after the complete touch"
        );
        assert!(
            matches!(actual.mode, game::state::GameMode::PassageEntry { .. }),
            "missing passage refusal retains the complete selected touch state"
        );
        assert!(
            self.milestones.into_iter().all(|held| held),
            "continuous production replay must reach and prove every minimum157..161 milestone"
        );
        assert!(
            self.compared == self.reached && self.state_uncompared == 0,
            "complete State and owner proof require independent comparison coverage at every reached cut"
        );
        assert!(
            self.whole_state_equal == self.compared,
            "complete State comparison differs"
        );
        assert!(
            self.owner_equal == self.compared,
            "complete ordinary owner comparison differs"
        );
        assert!(
            self.oam_equal == self.reached,
            "whole OAM comparison differs"
        );
    }
}

fn reference_expected(
    reference: &oracle_record::FrameRecord,
    previous: &oracle_record::FrameRecord,
    inputs: &ReplayInputs<'_>,
) -> Result<crate::decode::Decoded, crate::decode::DecodeError> {
    decode_ordinary(reference, inputs.rooms, inputs.weapons, inputs.power).or_else(|error| {
        if !matches!(
            error,
            crate::decode::DecodeError::Unsupported(crate::decode::Unsupported::GameState(_))
        ) {
            return Err(error);
        }
        crate::decode::decode_ordinary_completed_entry(
            reference,
            previous,
            inputs.rooms,
            inputs.weapons,
            inputs.power,
            inputs.content.world.as_ref().unwrap(),
        )
    })
}

struct ReplayInputs<'a> {
    records: &'a [(crate::lock::Frame, oracle_record::FrameRecord)],
    pads: &'a [game::state::Buttons],
    timing: &'a TimingSource,
    seed_record: &'a oracle_record::FrameRecord,
    rooms: &'a crate::decode::RoomMap,
    weapons: &'a crate::harness::weapons::Weapons,
    power: &'a game::weapons_projectiles_bridge::PowerContent,
    content: &'a game::pose::Content,
}

fn run_replay(session: &mut Session, inputs: &ReplayInputs<'_>, proof: &mut ReplayProof) {
    let records = inputs.records;
    let pads = inputs.pads;
    let timing = inputs.timing;
    let seed_record = inputs.seed_record;
    for (position, (frame, reference)) in records.iter().enumerate() {
        let before = session.state().clone();
        let result = session.step(TickInput {
            pad: pads[position],
            timing: timing.at(Tick(frame.0)),
        });
        if let Err(fault) = result {
            // Unsupported continuation must not publish a partially applied W1 frame.
            assert!(
                session.state() == &before,
                "ordinary rejected step must preserve its prior State"
            );
            proof.runtime_fault = Some(fault);
            proof.runtime_boundary = Some(format!("{} {}", frame.0, fault_name(fault)));
            break;
        }
        if matches!(
            session.state().mode,
            game::state::GameMode::PassageEntry { .. }
        ) {
            let _ = writeln!(
                std::io::stdout(),
                "ordinary selected doorway retained at cut {}",
                frame.0
            );
        }
        proof.reached = proof.reached.saturating_add(1);
        // Canonical OAM is all 512 low bytes followed by all 32 high bytes, without masks.
        let encoded_oam = canonical::to_bytes(&session.state().video.memory.oam);
        let oam_matches = encoded_oam.as_slice() == reference.oam.as_ref();
        proof.oam_equal = proof.oam_equal.saturating_add(u64::from(oam_matches));
        if !oam_matches {
            proof
                .first_mismatches
                .entry("whole OAM".into())
                .or_insert(frame.0);
        }
        let previous = if position == 0 {
            seed_record
        } else {
            &records[position.checked_sub(1).unwrap()].1
        };
        let reference_decoded = reference_expected(reference, previous, inputs);
        let Some((mut expected, calls)) = proof.expect(
            reference_decoded,
            reference,
            session.state(),
            frame.0,
            oam_matches,
        ) else {
            continue;
        };
        proof.compare(session.state(), &mut expected, frame.0, calls, oam_matches);
        // A mismatch is reported but does not re-seed or end the continuous production run.
    }
    proof.finish(session.state());
}

#[test]
fn m0_ordinary_owner_runs_once_from_seed_through_first_actual_unsupported_boundary() {
    let Some((cache_root, rom)) = crate::harness::tests::environment() else {
        return;
    };
    let lock_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let lock = Lock::parse(&std::fs::read_to_string(lock_path).unwrap()).unwrap();
    let row = lock.row(&RowId("chain-any-0003-DFD7".into())).unwrap();
    let cache = Cache::open(&cache_root, row).unwrap();
    let records = cache
        .records_without_pictures()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    assert!(!records.is_empty());

    let bundle_root = std::env::var_os("SM_BUNDLE")
        .expect("public ordinary admission requires original pinned bundle");
    let image = std::fs::read(rom).unwrap();
    let content = Arc::new(crate::bundle::content(image.clone()).unwrap());
    let rooms = crate::bundle::rooms(&content, &image).unwrap();
    let weapons = crate::harness::weapons::Weapons::load(&image).unwrap();
    let power = content.ordinary_power.as_ref().unwrap();
    // The native state is before row frame zero; its accompanying record is the
    // preceding cut, certified by the pinned relock bundle (ORACLE section 3).
    let Start::Seed { capture, index } = &row.start else {
        panic!("ordinary replay requires a native preceding seed");
    };
    assert!(
        row.seed_words.is_none(),
        "ordinary replay requires an unedited native seed"
    );
    let seed_path = Path::new(&bundle_root)
        .join("seeds")
        .join(capture)
        .join(format!("{index:04}.lss"));
    assert!(
        Some(Sha256::of(&std::fs::read(&seed_path).unwrap())) == row.seed_record,
        "native seed state must match the scenario lock pin"
    );
    let seed_bytes = std::fs::read(seed_path.with_extension("record")).unwrap();
    let seed_record = oracle_record::lsnes::read(&seed_bytes).unwrap();
    let admitted = crate::bundle::RowInputs::read_ordinary(
        Path::new(&bundle_root),
        row,
        &rooms,
        &weapons,
        power,
    )
    .unwrap();
    assert!(
        admitted.seed.state.projectiles.ordinary().is_some(),
        "public ordinary admission must select the checked owner"
    );
    let pads = admitted.pads;
    assert!(
        u64::try_from(pads.len()).unwrap() == row.frames.0.checked_add(1).unwrap(),
        "public pads include next-frame preview"
    );
    assert!(
        records
            .iter()
            .enumerate()
            .all(|(position, (frame, _))| u64::try_from(position).unwrap() == frame.0)
            && u64::try_from(records.len()).unwrap() == row.frames.0,
        "cached cuts align with original inputs"
    );
    let seed = admitted.seed;
    let rules = seed.rules;
    let timing = TimingSource::Recorded(Schedule::new(
        row.overruns.iter().map(|frame| Tick(frame.0)).collect(),
        row.nmi_off.iter().map(|frame| Tick(frame.0)).collect(),
    ));
    let mut proof = ReplayProof {
        prior_closed: closed_outer_boundary(&seed.state, &seed_record),
        prior_gameplay: seed.state.mode == game::state::GameMode::Gameplay,
        rng_receipt: Some(0),
        ..ReplayProof::default()
    };
    // This is the sole Session construction and sole assignment from a decoded State.
    let mut session = match Session::open(seed.state, rules, Arc::clone(&content)) {
        Ok(session) => session,
        Err(error) => {
            let _ = writeln!(
                std::io::stdout(),
                "ordinary replay seed {} open unsupported {error:?}",
                seed_record.index
            );
            panic!("ordinary replay seed cannot open its owned production Session");
        }
    };
    run_replay(
        &mut session,
        &ReplayInputs {
            records: &records,
            pads: &pads,
            timing: &timing,
            seed_record: &seed_record,
            rooms: &rooms,
            weapons: &weapons,
            power,
            content: &content,
        },
        &mut proof,
    );
}

#[test]
fn completed_entry_certificate_rejects_later_passage_and_invalid_entry_facts() {
    let Some((cache_root, rom)) = crate::harness::tests::environment() else {
        return;
    };
    let lock_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let lock = Lock::parse(&std::fs::read_to_string(lock_path).unwrap()).unwrap();
    let row = lock.row(&RowId("chain-any-0003-DFD7".into())).unwrap();
    let cache = Cache::open(&cache_root, row).unwrap();
    let records = cache
        .records_without_pictures()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    let image = std::fs::read(rom).unwrap();
    let content = crate::bundle::content(image.clone()).unwrap();
    let rooms = crate::bundle::rooms(&content, &image).unwrap();
    let weapons = crate::harness::weapons::Weapons::load(&image).unwrap();
    let entry = records
        .windows(2)
        .find(|pair| {
            crate::source::GAME_STATE.read(&pair[1].1.wram).unwrap() == 9
                && (crate::source::WramWord {
                    name: "ENTRY_FUNCTION",
                    offset: 0x099C,
                })
                .read(&pair[1].1.wram)
                .unwrap()
                    == 0xE17D
        })
        .unwrap();
    let previous = &entry[0].1;
    let reference = &entry[1].1;
    let power = content.ordinary_power.as_ref().unwrap();
    let world = content.world.as_ref().unwrap();
    let facts = crate::harness::weapons::Weapons::ordinary_seed_facts(reference).unwrap();
    assert!(
        weapons.ordinary_owner(reference, facts, power).is_err(),
        "generic gameplay admission stays closed"
    );
    for (offset, value) in [
        (0x0998, 10_u16),
        (0x099C, 0_u16),
        (0x078D, 0_u16),
        (0x09B2, 0_u16),
        (0x0A76, 1_u16),
    ] {
        let mut invalid = reference.clone();
        let mut wram = invalid.wram.to_vec();
        wram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        invalid.wram = wram.into();
        assert!(
            crate::decode::decode_ordinary_completed_entry(
                &invalid, previous, &rooms, &weapons, power, world
            )
            .is_err(),
            "later passage, invalid entry identity/binding or unsupported producer must reject"
        );
    }
}

#[test]
fn public_ordinary_seed_admission_rejects_an_unsupported_owner_without_legacy_fallback() {
    let Some((cache_root, rom)) = crate::harness::tests::environment() else {
        return;
    };
    let lock_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let lock = Lock::parse(&std::fs::read_to_string(lock_path).unwrap()).unwrap();
    let row = lock.row(&RowId("chain-any-0003-DFD7".into())).unwrap();
    let cache = Cache::open(&cache_root, row).unwrap();
    let (_, mut record) = cache.records_without_pictures().next().unwrap().unwrap();
    let image = std::fs::read(rom).unwrap();
    let content = crate::bundle::content(image.clone()).unwrap();
    let rooms = crate::bundle::rooms(&content, &image).unwrap();
    let weapons = crate::harness::weapons::Weapons::load(&image).unwrap();
    let power = content.ordinary_power.as_ref().unwrap();
    let mut wram = record.wram.to_vec();
    wram[0x0A76..0x0A78].copy_from_slice(&1_u16.to_le_bytes());
    record.wram = wram.into();
    assert!(
        matches!(
            crate::gate::Seed::decode_ordinary(&record, &rooms, &weapons, power),
            Err(crate::decode::DecodeError::OrdinaryPowerOwner)
        ),
        "ordinary capability error cannot return a Legacy seed"
    );
}
