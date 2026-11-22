//! Independent physical OAM and carried owner projections for the opening ground corridor.

use super::{
    Observation, Outcome, Path, PathBuf, Sha256, data_failure, observation, read_expected,
};
use canonical::Canonical;
use game::production::{
    frame,
    timing::{Event, EventKind},
};
use serde::{Deserialize, de::DeserializeOwned};
use types::Parts;

pub(super) struct Source(PathBuf);

const MIXED: u64 = 7;
const AIM_AND_CAMERA: u64 = 10;
const CARRIED: u64 = 11;
const OAM_BYTES: usize = 544;
const AIM_AND_CAMERA_FIELDS: usize = 2;
const CARRIED_FIELDS: usize = 4;

/// Corridor fields compared over the first `cuts` returns: one OAM hash per return from the
/// mixed Drawing on, the Aim and Camera pair at its single return, and the carried owners after.
pub(super) fn field_count(cuts: usize) -> Option<usize> {
    let returns_from = |first: u64| {
        usize::try_from(first)
            .ok()
            .map(|first| cuts.saturating_sub(first))
    };
    let aim_and_camera = if returns_from(AIM_AND_CAMERA)? == 0 {
        0
    } else {
        AIM_AND_CAMERA_FIELDS
    };
    returns_from(MIXED)?
        .checked_add(aim_and_camera)?
        .checked_add(returns_from(CARRIED)?.checked_mul(CARRIED_FIELDS)?)
}

#[derive(Deserialize)]
pub(super) struct RetainedOwners {
    animation_step: Option<u16>,
    animation_hold: Option<u16>,
    first_door_phase: Option<DoorPhase>,
    first_door_wait: Option<u16>,
    second_door_wait: Option<u16>,
    viewport_x_fraction: Option<u16>,
}

#[derive(Clone, Copy, Canonical, Deserialize)]
enum DoorPhase {
    ClosingFirst,
    ClosingSecond,
    Open,
}

#[derive(Deserialize)]
struct MixedDrawing {
    format: u16,
    scope: String,
    cut: Vec<Oam>,
}

#[derive(Deserialize)]
struct Oam {
    cut: u64,
    hardware_oam_bytes: usize,
    #[serde(rename = "oam_sha256")]
    sha256: String,
}

#[derive(Deserialize)]
struct AimAndCamera {
    format: u16,
    aim_result: AimResult,
    camera_result: CameraResult,
}

#[derive(Deserialize)]
struct AimResult {
    x_whole: u16,
    x_fraction: u16,
}

#[derive(Deserialize)]
struct CameraResult {
    x_whole: u16,
    x_fraction: u16,
    y_whole: u16,
}

fn qualify<T: DeserializeOwned>(path: &Path, sha: &str, cut: u64) -> Result<T, Outcome> {
    let fault = || {
        data_failure(
            Some(cut),
            "opening mixed Drawing or Aim and Camera comparison",
        )
    };
    let pin = Sha256::try_from(sha).map_err(|_| fault())?;
    let bytes = read_expected(path, pin, cut)?;
    toml::from_str(std::str::from_utf8(&bytes).map_err(|_| fault())?).map_err(|_| fault())
}

impl Source {
    pub(super) fn new(root: &Path) -> Self {
        Self(root.to_path_buf())
    }

    pub(super) fn read(
        &self,
        cut: u64,
        native: u64,
        owners: &RetainedOwners,
    ) -> Result<Vec<Observation>, Outcome> {
        let fault = || data_failure(Some(cut), "opening ground corridor owner comparison");
        let mut fields = Vec::new();
        if native >= MIXED {
            let expected: MixedDrawing = qualify(
                &self.0.join("opening-clock-mixed-drawing-expected-v1/mixed-actor-samus-oam-expected.private.toml"),
                "8fc7886613a3f533113981e7c21a375b5eb73a7188636bc71916095bc97737e4",
                cut,
            )?;
            if expected.format != 1 || expected.scope != "MixedDoorAndSamusDrawing" {
                return Err(fault());
            }
            let oam = expected
                .cut
                .iter()
                .find(|row| row.cut == native && row.hardware_oam_bytes == OAM_BYTES)
                .ok_or_else(fault)?;
            let hash = Sha256::try_from(oam.sha256.as_str()).map_err(|_| fault())?;
            fields.push(observation("frame.opening.oam", &hash.0));
        }
        if native == AIM_AND_CAMERA {
            let expected: AimAndCamera = qualify(
                &self.0.join("opening-clock-cut10-aim-camera-expected-v2/cut10-aim-camera-expected.private.toml"),
                "58cdc1893ec1ca53cae83f6bcdf644ca89b96a30a6a65248e4d3dd5e7498abf3",
                cut,
            )?;
            if expected.format != 2 {
                return Err(fault());
            }
            let AimResult {
                x_whole,
                x_fraction,
            } = expected.aim_result;
            fields.push(observation("frame.opening.aim_x", &[x_whole, x_fraction]));
            let CameraResult {
                x_whole,
                x_fraction,
                y_whole,
            } = expected.camera_result;
            fields.push(observation(
                "frame.opening.camera",
                &[x_whole, x_fraction, y_whole],
            ));
        }
        if native >= CARRIED {
            let phase = owners.first_door_phase.ok_or_else(fault)?;
            let step = owners.animation_step.ok_or_else(fault)?;
            let hold = owners.animation_hold.ok_or_else(fault)?;
            let first = owners.first_door_wait.ok_or_else(fault)?;
            let second = owners.second_door_wait.ok_or_else(fault)?;
            let camera = owners.viewport_x_fraction.ok_or_else(fault)?;
            fields.extend(carried(phase, [first, second], [step, hold], camera));
        }
        Ok(fields)
    }
}

pub(super) fn actual(
    state: &frame::State,
    events: &[Event],
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    let fault = || data_failure(Some(cut), "opening ground corridor actual owners");
    let native = cut.checked_sub(1).ok_or_else(fault)?;
    let mut fields = Vec::new();
    if native >= MIXED {
        let bytes: Vec<u8> = events
            .iter()
            .filter_map(|event| {
                if let EventKind::Payload(payload) = &event.kind
                    && payload.target == data::timing::TransferPort(4)
                {
                    Some(payload.value)
                } else {
                    None
                }
            })
            .collect();
        if bytes.len() != OAM_BYTES {
            return Err(fault());
        }
        fields.push(observation("frame.opening.oam", &Sha256::of(&bytes).0));
    }
    if native == AIM_AND_CAMERA {
        fields.extend(aim_and_camera(state));
    }
    if native >= CARRIED {
        let (phase, waits) = door_observations(state).ok_or_else(fault)?;
        if matches!(phase, DoorPhase::Open) {
            return Err(fault());
        }
        fields.extend(carried_observations(state, phase, waits));
    }
    Ok(fields)
}

pub(super) fn initial(state: &frame::State) -> Vec<Observation> {
    let mut fields = Vec::from(aim_and_camera(state));
    if let Some((phase, waits)) = door_observations(state) {
        fields.extend(carried_observations(state, phase, waits));
    }
    fields
}

fn aim_and_camera(state: &frame::State) -> [Observation; AIM_AND_CAMERA_FIELDS] {
    let samus = state.movement.state().position.x;
    let camera = state.camera.0.position();
    [
        observation("frame.opening.aim_x", &[samus.pixel().0, samus.sub()]),
        observation(
            "frame.opening.camera",
            &[camera.x.pixel().0, camera.x.sub(), camera.y.pixel().0],
        ),
    ]
}

fn door_observations(state: &frame::State) -> Option<(DoorPhase, [u16; 2])> {
    use boss_ceres_ridley::{Phase, Quarter};
    let [first, second] = state.recurring_player.as_ref()?.doors.current_models()?;
    let (phase, first_wait) = match first.phase {
        Phase::Open { wait } => (DoorPhase::Open, wait),
        Phase::Closing {
            quarter: Quarter::First,
            wait,
        } => (DoorPhase::ClosingFirst, wait),
        Phase::Closing {
            quarter: Quarter::Second,
            wait,
        } => (DoorPhase::ClosingSecond, wait),
        Phase::Placed
        | Phase::Starting { .. }
        | Phase::Opening { .. }
        | Phase::Closed { .. }
        | Phase::Closing { .. }
        | Phase::Wall => return None,
    };
    let Phase::Closed { wait: second_wait } = second.phase else {
        return None;
    };
    Some((phase, [first_wait.frames(), second_wait.frames()]))
}

fn carried_observations(
    state: &frame::State,
    phase: DoorPhase,
    waits: [u16; 2],
) -> [Observation; CARRIED_FIELDS] {
    let clip = state.appearance.model.0.state.playhead;
    carried(
        phase,
        waits,
        [u16::from(clip.step.0), u16::from(clip.time_left.0)],
        state.camera.0.position().x.sub(),
    )
}

fn carried(
    phase: DoorPhase,
    wait: [u16; 2],
    animation: [u16; 2],
    camera_fraction: u16,
) -> [Observation; CARRIED_FIELDS] {
    [
        observation("frame.opening.first_door_phase", &phase),
        observation("frame.opening.door_waits", &wait),
        observation("frame.opening.animation_playhead", &animation),
        observation("frame.opening.camera_x_fraction", &camera_fraction),
    ]
}
