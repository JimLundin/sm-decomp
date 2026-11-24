//! Disjoint original inputs and observations for the connected ordinary recipe.

use super::{appearance, camera, compare::Observation, data_failure, movement, result::Outcome};
use crate::lock::Sha256;
use game::production::connected::{
    self,
    inputs::{Activity, CameraInputs, Lead, Screen},
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

pub(super) const MANIFEST: &str =
    "df96f1be4f5a8e54802d6b55e8e9c05f9ab3cf37aeed04a27d9d8b6f3efdda4c";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Opportunity {
    cut: u64,
    controller: String,
    motion: String,
    contact: String,
    animation: String,
    pose_update: String,
    clip_initialization: String,
    late_pose: String,
    camera: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    row: String,
    initial_cut: u64,
    last_cut: u64,
    source_v1: String,
    source_v2: String,
    source_v3: String,
    extension: String,
    opportunity: Vec<Opportunity>,
    file: Vec<File>,
}

pub(super) const RUN_SLOPE_MANIFEST: &str =
    "f6440b4daf6ddf75fa62172108d706fa33e8b462c4429358082c100924e70e4d";

pub(super) fn fixture(case: &super::index::Case) -> Result<&'static str, Outcome> {
    match case.id.0.as_str() {
        "connected-ordinary/carried-running-camera" => Ok("connected-ordinary-v1"),
        "connected-ordinary/run-release-floor-aim" => Ok("connected-run-slope-v1"),
        _ => Err(data_failure(None, "connected reviewed case")),
    }
}

pub(super) struct Content {
    pub movement: movement::Content,
    pub appearance: appearance::Content,
}

#[derive(Clone)]
pub(super) struct Inputs {
    steps: BTreeMap<u64, connected::inputs::Inputs>,
    records: BTreeMap<u64, Vec<u8>>,
}

fn word(bytes: &[u8], at: usize) -> Result<u16, Outcome> {
    let raw = bytes
        .get(at..at.saturating_add(2))
        .ok_or_else(|| data_failure(None, "connected current word"))?;
    Ok(u16::from_le_bytes(raw.try_into().map_err(|_| {
        data_failure(None, "connected current word")
    })?))
}

fn camera_input(bytes: &[u8]) -> Result<CameraInputs, Outcome> {
    let screens = bytes
        .get(0xcd20..0xcd52)
        .ok_or_else(|| data_failure(None, "connected camera screens"))?
        .iter()
        .map(|value| match value {
            0 => Screen::Closed,
            1 => Screen::Short,
            _ => Screen::Open,
        })
        .collect();
    let lead = match word(bytes, 0x0941)? {
        0 => Lead::Ahead,
        2 => Lead::Near,
        4 => Lead::Left,
        6 => Lead::Right,
        _ => return Err(data_failure(None, "connected camera lead")),
    };
    Ok(CameraInputs {
        screens,
        size: [word(bytes, 0x07a9)?, word(bytes, 0x07ab)?],
        heights: [word(bytes, 0x07ad)?, word(bytes, 0x07af)?],
        lead,
    })
}

fn dry_and_bound(controller: &[u8], animation: &[u8], late: &[u8]) -> Result<(), Outcome> {
    for source in [controller, animation, late] {
        if word(source, 0x09b6)? != 0x8000 {
            return Err(data_failure(None, "connected current run binding"));
        }
    }
    for source in [animation, late] {
        if word(source, 0x0a66)? != 0
            || word(source, 0x0a9c)? != 0
            || word(source, 0x195e)? & 0x8000 == 0
            || word(source, 0x1962)? & 0x8000 == 0
        {
            return Err(data_failure(None, "connected current dry liquid"));
        }
    }
    Ok(())
}

fn step(
    op: &Opportunity,
    files: &BTreeMap<String, Vec<u8>>,
    content: &Content,
) -> Result<connected::inputs::Inputs, Outcome> {
    let get = |name: &str| {
        files
            .get(name)
            .map(Vec::as_slice)
            .ok_or_else(|| data_failure(None, "connected source member"))
    };
    let controller = get(&op.controller)?;
    let motion = get(&op.motion)?;
    let contact = get(&op.contact)?;
    let animation = get(&op.animation)?;
    let late = get(&op.clip_initialization)?;
    let cam = get(&op.camera)?;
    for source in [
        controller,
        motion,
        contact,
        animation,
        late,
        cam,
        get(&op.pose_update)?,
        get(&op.late_pose)?,
    ] {
        if source.len() != 131_072 {
            return Err(data_failure(None, "complete current source image"));
        }
    }
    dry_and_bound(controller, animation, late)?;
    let borrowed = movement::borrowed(controller, motion, &content.movement)?;
    if borrowed.cinematic
        || borrowed.elevator
        || word(motion, 0x0998)? != 8
        || word(motion, 0x0a78)? != 0
        || [0x18aa, 0x0a52, 0x0a56, 0x0cf8, 0x07e9]
            .into_iter()
            .any(|at| word(motion, at).ok() != Some(0))
    {
        return Err(data_failure(None, "qualified ordinary current family"));
    }
    // These unchanged unselected quantities are read at their actual source opportunities.
    if word(motion, 0x09a2)? != word(animation, 0x09a2)?
        || word(motion, 0x09a2)? != word(late, 0x09a2)?
        || word(motion, 0x079b)? != word(cam, 0x079b)?
    {
        return Err(data_failure(None, "connected unselected input lifetime"));
    }
    // Her stride reads the activities at the contact opportunity and her pose selection reads
    // the cinematic later in the frame: the frame's one input holds both readings when they agree.
    let activities = appearance::activities(contact)?;
    if activities[samus_appearance::Activity::Cinematic] != (word(late, 0x1f51)? != 0) {
        return Err(data_failure(None, "connected cinematic lifetime"));
    }
    let waiting = animation
        .get(0x0648)
        .copied()
        .ok_or_else(|| data_failure(None, "connected queue source"))?
        .wrapping_sub(
            *animation
                .get(0x0645)
                .ok_or_else(|| data_failure(None, "connected queue source"))?,
        )
        & 0x0f;
    Ok(connected::inputs::Inputs {
        held: borrowed.held,
        pressed: borrowed.pressed,
        items: borrowed.items,
        parity: borrowed.parity,
        energy: word(animation, 0x09c2)?,
        liquid: samus_appearance::Liquid::NONE,
        sounds_waiting: samus_appearance::SoundsWaiting::new(waiting)
            .ok_or_else(|| data_failure(None, "connected queue occupancy"))?,
        activities,
        charge: appearance::charge(animation)?,
        geometry: borrowed.world,
        camera: camera_input(cam)?,
        activity: Activity::Inactive,
    })
}

/// Room facts read once from one image and held: the played run's seed-held inputs.
/// Controls and parity are the caller's per cut; they start released and even.
pub(super) fn seed_inputs(
    seed: &[u8],
    items: types::Set<types::Item>,
    geometry: game::production::world::MovementWorld,
) -> Result<connected::inputs::Inputs, Outcome> {
    dry_and_bound(seed, seed, seed)?;
    held_inputs(
        seed,
        items,
        appearance::activities(seed)?,
        geometry,
        camera_input(seed)?,
    )
}

/// The seed-held inputs with the run's `activities`, `geometry` and `camera`: the rest read
/// from `seed`, its sound queue, her energy and her shot's charge.
pub(super) fn held_inputs(
    seed: &[u8],
    items: types::Set<types::Item>,
    activities: types::Set<samus_appearance::Activity>,
    geometry: game::production::world::MovementWorld,
    camera: CameraInputs,
) -> Result<connected::inputs::Inputs, Outcome> {
    let queue = |at: usize| {
        seed.get(at)
            .copied()
            .ok_or_else(|| data_failure(None, "seed queue source"))
    };
    let waiting = queue(0x0648)?.wrapping_sub(queue(0x0645)?) & 0x0f;
    Ok(connected::inputs::Inputs {
        held: types::Buttons::default(),
        pressed: types::Buttons::default(),
        items,
        parity: types::Parity::Even,
        energy: word(seed, 0x09c2)?,
        liquid: samus_appearance::Liquid::NONE,
        sounds_waiting: samus_appearance::SoundsWaiting::new(waiting)
            .ok_or_else(|| data_failure(None, "seed queue occupancy"))?,
        activities,
        // Held from the seed like the room's facts: only a passage's first entry reads it, and
        // no frame these inputs play has one.
        charge: appearance::charge(seed)?,
        geometry,
        camera,
        activity: Activity::Inactive,
    })
}

impl Inputs {
    pub fn load(root: &Path, content: &Content) -> Result<Self, Outcome> {
        let raw = std::fs::read(root.join("manifest.toml"))
            .map_err(|_| data_failure(None, "connected input manifest"))?;
        let pin = Sha256::of(&raw).to_string();
        if pin != MANIFEST && pin != RUN_SLOPE_MANIFEST {
            return Err(data_failure(None, "connected input pin"));
        }
        let manifest: Manifest = toml::from_str(
            std::str::from_utf8(&raw).map_err(|_| data_failure(None, "connected input text"))?,
        )
        .map_err(|_| data_failure(None, "connected input schema"))?;
        let reviewed = match pin.as_str() {
            MANIFEST => {
                manifest.initial_cut == 6
                    && manifest.last_cut == 15
                    && manifest.source_v1
                        == "e37b5523e368d84c00a0b9625d9ea13f0e88ba553d74839e95f97f9e910f442d"
                    && manifest.source_v2
                        == "9904841986a89999ab1a2f5f6c2601c5551b1609b1c584c3e4c4dc2eb0bbed5f"
                    && manifest.source_v3
                        == "0f5926a507f54c85dfe36e2fcbc05187d919d857c929128b501d7dd253e7c727"
                    && manifest.extension
                        == "0556ce279ca4375be03e5060bc665598645c76fa9e7f1cf16def69667cb79d00"
            }
            RUN_SLOPE_MANIFEST => {
                manifest.initial_cut == 126
                    && manifest.last_cut == 142
                    && manifest.source_v1
                        == "514ca9649b265dbcca139fc1f7607415ea236f1b76df2aef0710ec9d976276f3"
                    && manifest.source_v2
                        == "ba6dd5da351381c431badca440fc3f253e45b131bb685674223d7fcf13399e58"
                    && manifest.source_v3
                        == "87e42123a8e2f269e8c0d8b3527dc2b6fab4b7dbb603b00b5eab00df619a1ece"
                    && manifest.extension
                        == "4d769f8f398f1d8dfba35c21082c953d238bb02879ee369bcc4b2cbb979cdcc3"
            }
            _ => false,
        };
        if manifest.format != "connected-ordinary-inputs-v1"
            || manifest.row != "chain-any-0003-DFD7"
            || !reviewed
        {
            return Err(data_failure(None, "connected reviewed scope"));
        }
        let mut files = BTreeMap::new();
        for member in manifest.file {
            if Path::new(&member.path)
                .components()
                .any(|part| !matches!(part, std::path::Component::Normal(_)))
            {
                return Err(data_failure(None, "connected member path"));
            }
            let bytes = std::fs::read(root.join(&member.path))
                .map_err(|_| data_failure(None, "connected current source"))?;
            if Sha256::of(&bytes)
                != Sha256::try_from(member.sha256.as_str())
                    .map_err(|_| data_failure(None, "connected member pin"))?
                || files.insert(member.path, bytes).is_some()
            {
                return Err(data_failure(None, "connected member pin"));
            }
        }
        let mut steps = BTreeMap::new();
        for op in manifest.opportunity {
            let inputs = step(&op, &files, content)?;
            if steps.insert(op.cut, inputs).is_some() {
                return Err(data_failure(None, "connected unique opportunity"));
            }
        }
        if steps.keys().copied().collect::<Vec<_>>()
            != (manifest.initial_cut.saturating_add(1)..=manifest.last_cut).collect::<Vec<_>>()
        {
            return Err(data_failure(None, "connected complete interval"));
        }
        let records = (manifest.initial_cut..=manifest.last_cut)
            .map(|cut| {
                let name = format!("record-{cut:02}.bin");
                let bytes = files
                    .remove(&name)
                    .ok_or_else(|| data_failure(None, "connected aligned record"))?;
                Ok((cut, bytes))
            })
            .collect::<Result<BTreeMap<_, _>, Outcome>>()?;
        Ok(Self { steps, records })
    }

    pub fn current(&self, cut: u64) -> Result<&connected::inputs::Inputs, Outcome> {
        self.steps
            .get(&cut)
            .ok_or_else(|| data_failure(Some(cut), "connected ordinary opportunity"))
    }

    pub fn aligned(&self, cut: u64, record: &oracle_record::FrameRecord) -> Result<(), Outcome> {
        let bytes = self
            .records
            .get(&cut)
            .ok_or_else(|| data_failure(Some(cut), "connected aligned source"))?;
        let original = oracle_record::lsnes::read(bytes)
            .map_err(|_| data_failure(Some(cut), "connected original record"))?;
        if original.wram != record.wram
            || original.cpu != record.cpu
            || original.cut != record.cut
            || original.video_frame != record.video_frame
            || original.pad != record.pad
            || original.joypad_reads != record.joypad_reads
            || original.wait_entry != record.wait_entry
            || original.overscan != record.overscan
        {
            return Err(data_failure(Some(cut), "connected cache/source alignment"));
        }
        Ok(())
    }
}

pub(super) fn state_observation(state: &samus_movement::Samus) -> Observation {
    Observation {
        field: "movement.state.complete".into(),
        bytes: canonical::to_bytes(&canonical::model::Model::<game::state::owned::MovementPin>(
            *state,
        )),
    }
}

pub(super) fn playing_observation(figure: samus_appearance::Figure) -> Observation {
    Observation {
        field: "appearance.playing".into(),
        bytes: canonical::to_bytes(&canonical::model::Model::<connected::PlayingPin>(figure)),
    }
}

pub(super) fn expected(
    bytes: &[u8],
    content: &appearance::Content,
    case: &super::index::Case,
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    let mut fields = vec![state_observation(&movement::seed(bytes)?)];
    fields.extend(movement::expected(
        bytes,
        &[
            super::index::ObservationId::PositionX,
            super::index::ObservationId::PositionY,
        ],
        &case.row,
        cut,
    )?);
    let (state, playing) = appearance::running_animation(bytes, content)?;
    fields.extend(appearance::initial(state, content));
    fields.push(playing_observation(playing));
    fields.extend(camera::expected(bytes)?);
    fields.extend(movement::expected(
        bytes,
        &[
            super::index::ObservationId::TravelLeft,
            super::index::ObservationId::TravelRight,
            super::index::ObservationId::TravelUp,
            super::index::ObservationId::TravelDown,
        ],
        &case.row,
        cut,
    )?);
    Ok(fields)
}

pub(super) fn actual(
    state: &connected::State,
    content: &appearance::Content,
    outputs: Option<&connected::Outputs>,
) -> Vec<Observation> {
    let mut fields = vec![
        state_observation(&state.movement.0),
        Observation::quantity(
            "movement.position.x",
            i64::from(state.movement.0.position.x.0),
        ),
        Observation::quantity(
            "movement.position.y",
            i64::from(state.movement.0.position.y.0),
        ),
    ];
    fields.extend(appearance::actual(&state.appearance.0, content));
    fields.push(playing_observation(state.playing.0));
    fields.extend(camera::initial(&state.camera.0));
    for (direction, name) in [
        (types::Direction::Left, "movement.travel.Left"),
        (types::Direction::Right, "movement.travel.Right"),
        (types::Direction::Up, "movement.travel.Up"),
        (types::Direction::Down, "movement.travel.Down"),
    ] {
        fields.push(Observation::quantity(
            name,
            outputs.map_or(0, |out| i64::from(out.movement.travelled[direction].0.0)),
        ));
    }
    fields
}

#[cfg(test)]
mod tests {
    fn diagnostic(args: std::fmt::Arguments<'_>) {
        use std::io::Write as _;
        writeln!(std::io::stdout().lock(), "{args}").unwrap();
    }
    fn index() -> super::super::index::Index {
        super::super::index::Index::parse(include_str!("../../../../scenarios/replay/index.toml"))
            .unwrap()
    }
    fn sources() -> super::super::Sources {
        let path = |name| std::env::var_os(name).unwrap().into();
        super::super::Sources {
            cache: path("SM_CACHE"),
            rom: path("SM_ROM"),
            native: path("SM_CONNECTED_SOURCE"),
            lock: path("SM_LOCK"),
            ..Default::default()
        }
    }

    #[test]
    #[ignore = "private connected source, cache and player ROM"]
    fn connected_common_runner() {
        let index = index();
        let case = index
            .cases
            .iter()
            .find(|case| case.id.0 == "connected-ordinary/carried-running-camera")
            .unwrap();
        let sources = sources();
        let result = super::super::run(case, &sources);
        diagnostic(format_args!(
            "CONNECTED_RESULT {:?} counts={:?} digest={}",
            result.outcome, result.counts, result.observation_sha256
        ));
        assert!(result.passed());
    }
    #[test]
    #[ignore = "complete private pinned cache, native fixtures and player ROM"]
    fn connected_complete_index_preserves_all_reviewed_floors() {
        let snapshot = super::super::index::IndexSnapshot::parse(include_str!(
            "../../../../scenarios/replay/index.toml"
        ))
        .unwrap();
        let index = snapshot.index();
        let sources = sources();
        let results: Vec<_> = index
            .cases
            .iter()
            .map(|case| {
                let result = super::super::run(case, &sources);
                diagnostic(format_args!(
                    "REPLAY_BASELINE id={} outcome={:?} counts={:?} digest={}",
                    case.id.0, result.outcome, result.counts, result.observation_sha256
                ));
                result
            })
            .collect();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
        let baseline =
            super::super::baseline::Baseline::read(&root.join("scenarios/replay/baseline.toml"))
                .unwrap();
        baseline.check(&snapshot, &results).unwrap();
        diagnostic(format_args!(
            "CONNECTED_FULL_BASELINE cases={} all_floor_atoms_held=true",
            results.len()
        ));
    }
    #[test]
    #[ignore = "private reviewed current Contact and wrap source"]
    fn connected_marked_contact_and_first_wrap_have_complete_publications() {
        use game::production::ProductionOutputs;
        use game::state::owned::OwnedState;
        use samus_appearance::{GraphicalContact, Sound};
        let index = index();
        let sources = sources();
        let case = index
            .cases
            .iter()
            .find(|case| case.id.0 == "connected-ordinary/carried-running-camera")
            .unwrap();
        let mut admitted = super::super::recorded::admit(case, &sources).unwrap();
        let mut holds = 0usize;
        let mut advances = 0usize;
        let mut wraps = Vec::new();
        for cut in 7..=15 {
            let OwnedState::Connected(before) = admitted.production.state() else {
                panic!()
            };
            let playhead = before.appearance.0.animation.playhead;
            let contact = GraphicalContact {
                figure: before.playing.0,
                playhead,
            };
            let ProductionOutputs::Connected(output) = admitted
                .inputs
                .advance(&mut admitted.production, cut)
                .unwrap()
            else {
                panic!()
            };
            let expected_contact = (cut == 10).then_some(contact);
            let expected_sound = (cut == 10).then_some(Sound::Footstep.offer());
            let graphics_match = output.contact == expected_contact;
            if !graphics_match {
                diagnostic(format_args!(
                    "CONNECTED_PUBLICATION_DIFFERENCE cut={cut} field=appearance.graphics"
                ));
            }
            assert!(graphics_match);
            let sounds_match = output.sounds == expected_sound.into_iter().collect::<Vec<_>>();
            if !sounds_match {
                diagnostic(format_args!(
                    "CONNECTED_PUBLICATION_DIFFERENCE cut={cut} field=appearance.sounds"
                ));
            }
            assert!(sounds_match);
            let OwnedState::Connected(after) = admitted.production.state() else {
                panic!()
            };
            let step = after.appearance.0.animation.playhead.step;
            if step == playhead.step {
                holds += 1;
            } else {
                advances += 1;
            }
            if step < playhead.step {
                wraps.push(cut);
            }
        }
        assert_eq!((holds, advances), (5, 4));
        assert_eq!(wraps, vec![14]);
        diagnostic(format_args!(
            "CONNECTED_PUBLICATIONS holds={holds} advances={advances} first_wrap=14 marked_contact=10 graphics=1 sounds=1"
        ));
    }
}
