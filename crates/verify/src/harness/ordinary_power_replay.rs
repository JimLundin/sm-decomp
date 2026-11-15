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
    appearance_equal: u64,
    appearance_null_equal: u64,
    appearance_requests: u64,
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
    contact_marks: &'a [u8],
}

/// Queue insertion observations are independent of the semantic producer under test.
/// Sender writes and ACK/retry transitions never provide expected appearance requests.
fn appearance_expected(
    previous: &oracle_record::FrameRecord,
    next: &oracle_record::FrameRecord,
) -> Vec<game::samus_appearance_bridge::producer::Request> {
    use game::samus_appearance_bridge::producer::Request;
    use samus_appearance::SoundRequest;
    let mut requests = Vec::new();
    for library in [0_usize, 2] {
        let before = previous.wram[0x646_usize.saturating_add(library)];
        let after = next.wram[0x646_usize.saturating_add(library)];
        let appended = after.wrapping_sub(before) & 15;
        for index in 0..appended {
            let slot = before.wrapping_add(index) & 15;
            let code = next.wram[0x656_usize
                .saturating_add(16_usize.saturating_mul(library))
                .saturating_add(usize::from(slot))];
            let request = match (library, code) {
                (0, 0x31) => Some(SoundRequest::PlainSpin),
                (0, 0x32) => Some(SoundRequest::SpinEnded),
                (0, 2) => Some(SoundRequest::SamusSilence),
                (2, 6) => Some(SoundRequest::Footstep),
                // CeresDoorOpening, weapon requests and the out-of-scope
                // SamusLandedOrWallJumped family are excluded from this S5 inventory.
                _ => None,
            };
            requests.extend(request.map(Request::Appearance));
        }
    }
    requests
}

fn appearance_word(record: &oracle_record::FrameRecord, at: usize) -> u16 {
    u16::from_le_bytes(record.wram[at..at.saturating_add(2)].try_into().unwrap())
}

fn graphical_contacts(
    previous: &oracle_record::FrameRecord,
    contact: bool,
    content: &game::pose::Content,
) -> Vec<samus_appearance::GraphicalContact> {
    if !contact {
        return Vec::new();
    }
    let pose = data::pose::PoseId(appearance_word(previous, 0x0a1c));
    let figure = game::samus_appearance_bridge::figure_of(pose).unwrap();
    let step = content
        .samus_clips
        .of(pose)
        .unwrap()
        .step_of(data::pose::animation::AnimationFrame(appearance_word(
            previous, 0x0a96,
        )))
        .unwrap();
    vec![samus_appearance::GraphicalContact {
        figure,
        playhead: samus_appearance::Playhead {
            step,
            time_left: samus_appearance::Frames(
                u8::try_from(appearance_word(previous, 0x0a94)).unwrap(),
            ),
        },
    }]
}

/// Source opportunities use independently retained quantities and raw imported mark bytes.
/// Queue observations cross-check admission; they do not define a silent missing request.
fn appearance_sources(
    previous: &oracle_record::FrameRecord,
    next: &oracle_record::FrameRecord,
    marks: &[u8],
) -> (Vec<game::samus_appearance_bridge::producer::Request>, bool) {
    use game::samus_appearance_bridge::producer::Request;
    use samus_appearance::SoundRequest;
    let word = appearance_word;
    let from = word(previous, 0x0a1f) & 255;
    let current = word(next, 0x0a1f) & 255;
    let spinning = |movement| matches!(movement, 3 | 20);
    let contact = if from == 1 && word(previous, 0x0a94) == 1 {
        let marker = marks
            .get(usize::from(word(previous, 0x0a96)))
            .expect("running source picture must have independently imported contact content");
        *marker != 0
    } else {
        false
    };
    let mut requests = Vec::new();
    if contact
        && word(previous, 0x0b3e) < 0x0400
        && [0x1f51, 0x179c, 0x0a68]
            .into_iter()
            .all(|at| word(next, at) == 0)
    {
        requests.push(Request::Appearance(SoundRequest::Footstep));
    }
    if word(previous, 0x0a1c) != word(next, 0x0a1c) && current == 3 {
        let items = word(next, 0x09a2);
        // Independently checked capability rejects liquid surfaces and special equipped items.
        assert_eq!(items & 0x228, 0);
        if !spinning(from) && word(next, 0x1f51) == 0 {
            requests.push(Request::Appearance(SoundRequest::PlainSpin));
        }
    }
    if spinning(word(next, 0x0a11) & 255) && !spinning(current) {
        requests.push(Request::Appearance(SoundRequest::SpinEnded));
    }
    (requests, contact)
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
        let inserted = appearance_expected(previous, reference);
        let (wanted, contact) = appearance_sources(previous, reference, inputs.contact_marks);
        assert_eq!(
            wanted, inserted,
            "source opportunities must equal independent accepted queue insertions"
        );
        let output = result.unwrap();
        let contacts_match =
            output.appearance.contacts == graphical_contacts(previous, contact, inputs.content);
        proof.appearance_equal = proof.appearance_equal.saturating_add(u64::from(
            output.appearance.requests == wanted && contacts_match,
        ));
        proof.appearance_null_equal = proof
            .appearance_null_equal
            .saturating_add(u64::from(wanted.is_empty()));
        proof.appearance_requests = proof
            .appearance_requests
            .saturating_add(u64::try_from(wanted.len()).unwrap());
        if output.appearance.requests != wanted || !contacts_match {
            proof
                .first_mismatches
                .entry("appearance producer ordered requests".into())
                .or_insert(frame.0);
        }
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
    let contact_marks = data::rom::Lorom::new(&image)
        .read(data::rom::RomAddress::new(0x90, 0xA424), 10)
        .unwrap()
        .to_vec();
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
    let mut session = Session::open(seed.state, rules, Arc::clone(&content))
        .expect("ordinary replay seed must open its owned production Session");
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
            contact_marks: &contact_marks,
        },
        &mut proof,
    );
    prove_appearance(&proof);
    prove_callable_passage(&session, &records);
}

fn prove_appearance(proof: &ReplayProof) {
    let _ = writeln!(
        std::io::stdout(),
        "ordinary appearance producer compared={} equal={} null={} requests={}",
        proof.reached,
        proof.appearance_equal,
        proof.appearance_null_equal,
        proof.appearance_requests
    );
    assert_eq!(proof.appearance_equal, proof.reached);
}

fn prove_callable_passage(
    session: &Session,
    records: &[(crate::lock::Frame, oracle_record::FrameRecord)],
) {
    // R1 dispatch is still held. Exercise only its callable producer boundary on the last
    // actual retained State, without running a sender, making readiness or reseeding.
    let previous = &records.iter().find(|(frame, _)| frame.0 == 182).unwrap().1;
    let entry = &records.iter().find(|(frame, _)| frame.0 == 183).unwrap().1;
    let retained = session.state().samus.animation;
    assert!(
        session
            .state()
            .projectiles
            .ordinary()
            .unwrap()
            .appearance()
            .is_some()
    );
    let current = game::samus_appearance_bridge::figure_of(session.state().samus.pose)
        .unwrap()
        .pose
        .action;
    let shoot = session
        .state()
        .controller
        .held
        .contains(game::state::Button::X);
    let mut held = game::samus_appearance_bridge::producer::Outputs::default();
    // The checked no-charge ordinary owner preserves unequipped charge and zero history.
    game::samus_appearance_bridge::producer::passage_entry(
        current,
        shoot,
        samus_appearance::Charge::new(0).unwrap(),
        &mut held,
    );
    assert_eq!(held.requests, appearance_expected(previous, entry));
    assert_eq!(session.state().samus.animation, retained);
    let _ = writeln!(
        std::io::stdout(),
        "ordinary held passage producer callable-only equal=1 requests={}",
        held.requests.len()
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

#[test]
fn time_up_event_refuses_native_seed_and_restoration() {
    use game::samus_appearance_bridge::producer::ExcludedWriter;
    use game::state::projectile::OrdinaryCapability;
    let Some((cache_root, rom)) = crate::harness::tests::environment() else {
        return;
    };
    let lock_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let lock = Lock::parse(&std::fs::read_to_string(lock_path).unwrap()).unwrap();
    let row = lock.row(&RowId("chain-any-0003-DFD7".into())).unwrap();
    let cache = Cache::open(&cache_root, row).unwrap();
    let (_, mut record) = cache.records_without_pictures().next().unwrap().unwrap();
    let image = std::fs::read(rom).unwrap();
    let content = Arc::new(crate::bundle::content(image.clone()).unwrap());
    let rooms = crate::bundle::rooms(&content, &image).unwrap();
    let weapons = crate::harness::weapons::Weapons::load(&image).unwrap();
    let power = content.ordinary_power.as_ref().unwrap();
    let facts = crate::harness::weapons::Weapons::ordinary_seed_facts(&record).unwrap();
    assert!(!facts.appearance.unwrap().active[ExcludedWriter::TimeUp]);
    assert!(OrdinaryCapability::from_seed(facts).is_ok());
    let seed = crate::gate::Seed::decode_ordinary(&record, &rooms, &weapons, power).unwrap();
    let restored = canonical::from_bytes::<State>(&canonical::to_bytes(&seed.state)).unwrap();
    assert!(Session::open(restored, seed.rules, Arc::clone(&content)).is_ok());
    record.wram[0x0943..0x0945].copy_from_slice(&0_u16.to_le_bytes());
    record.wram[0xd821] |= 0x40;
    let facts = crate::harness::weapons::Weapons::ordinary_seed_facts(&record).unwrap();
    assert!(OrdinaryCapability::from_seed(facts).is_err());
    assert!(facts.appearance.unwrap().active[ExcludedWriter::TimeUp]);
    assert!(matches!(
        crate::gate::Seed::decode_ordinary(&record, &rooms, &weapons, power),
        Err(crate::decode::DecodeError::OrdinaryPowerOwner)
    ));
}
