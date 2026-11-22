//! Independently admitted adjacent tail inputs, separate from return comparisons.

use super::{super::Member, quiet_profile};
use crate::{
    lock::Sha256,
    replay::{Sources, data_failure, index::Case, result::Outcome},
};
use data::pose::{PoseId, animation::AnimationFrame};
use game::{production::frame, samus_appearance_bridge::Clips};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    format: u16,
    member: [FixtureMember; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureMember {
    path: std::path::PathBuf,
    bytes: usize,
    sha256: String,
}

impl Source {
    fn text(&self, directory: &std::path::Path, name: &str) -> Result<String, Outcome> {
        let fault = || data_failure(None, "adjacent Frame tail member identity");
        let mut matches = self
            .member
            .iter()
            .filter(|member| member.path == std::path::Path::new(name));
        let member = matches.next().ok_or_else(fault)?;
        if matches.next().is_some() {
            return Err(fault());
        }
        let text = Member {
            path: directory.join(&member.path),
            sha256: member.sha256.clone(),
        }
        .text()?;
        if text.len() != member.bytes {
            return Err(fault());
        }
        Ok(text)
    }
}

#[derive(Deserialize)]
struct ControllerOrigin {
    format: u16,
    origin_sha256: String,
    initial_repeat_delay_offset: usize,
    current_repeat_timer_offset: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContactReply {
    format: u16,
    line: String,
    observation: String,
    reply: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DrawingSelection {
    format: u16,
    pose_id: u16,
    animation_frame: u16,
    upper_pieces: u16,
    lower_pieces: u16,
}

#[derive(Deserialize)]
struct Camera {
    format: u16,
    current: Centre,
    previous: Centre,
    view: View,
    screen: [Screens; 2],
}

#[derive(Deserialize)]
struct Centre {
    pixels: [u16; 2],
    fractions: [u16; 2],
}

impl From<Centre> for frame::camera_stage::RequiredCentre {
    fn from(centre: Centre) -> Self {
        Self {
            pixels: centre.pixels,
            fractions: centre.fractions,
        }
    }
}

#[derive(Deserialize)]
struct View {
    x: u16,
    fraction: u16,
    y: u16,
}

#[derive(Deserialize)]
struct Screens {
    cell: usize,
    screens: [Screen; 2],
}

#[derive(Deserialize)]
enum Screen {
    Closed,
    Open,
    Short,
}

impl From<Screen> for frame::camera_stage::Screen {
    fn from(value: Screen) -> Self {
        match value {
            Screen::Closed => Self::Closed,
            Screen::Open => Self::Open,
            Screen::Short => Self::Short,
        }
    }
}

pub(super) fn bind(
    case: &Case,
    sources: &Sources,
    wram: &[u8],
    clips: &Clips,
    state: &mut frame::State,
    content: &mut frame::Content,
) -> Result<(), Outcome> {
    let continuation = case
        .end
        .native_returns
        .as_ref()
        .and_then(|returns| returns.continuation.as_ref());
    let mut declarations = case
        .borrowed
        .iter()
        .filter(|source| source.dependency == "frame.adjacent-tail");
    let fault = || data_failure(None, "qualified adjacent Frame tail inputs");
    let Some(reference) = continuation else {
        return if declarations.next().is_none() {
            Ok(())
        } else {
            Err(fault())
        };
    };
    if declarations.next().is_none()
        || declarations.next().is_some()
        || content.adjacent_animation.is_none()
        || content.adjacent_scene_countdown.is_none()
        || case.first_cut() != Some(1)
        || case.adjacent_native_end() != 2
    {
        return Err(fault());
    }
    let path = sources.native.join(&reference.path);
    let directory = path.parent().ok_or_else(fault)?.to_owned();
    let manifest = Member {
        path,
        sha256: reference.sha256.clone(),
    };
    let source: Source = toml::from_str(&manifest.text()?).map_err(|_| fault())?;
    if source.format != 1 {
        return Err(fault());
    }
    let origin = Sha256::of(wram);
    bind_controller(
        &source.text(&directory, "controller-origin.toml")?,
        wram,
        state,
    )?;
    bind_camera(
        &source.text(&directory, "camera-current.toml")?,
        content,
        false,
    )?;
    let profiles = source.text(&directory, "profiles.toml")?;
    let cartridge = data::rom::Rom::new(std::fs::read(&sources.rom).map_err(|_| fault())?)
        .map_err(|_| fault())?;
    let drawing = Member {
        path: sources.native.join("adjacent-drawing-v1/selection.toml"),
        sha256: "1a800a4b8342f593b90b7c745130db2918f8035d757e3fa3a2b984995487fc2c".into(),
    };
    bind_video(&profiles, &drawing.text()?, clips, &cartridge, content)?;
    bind_contact(
        &source.text(&directory, "contact-reply-input.toml")?,
        &profiles,
        content,
    )?;
    content
        .interruption
        .as_mut()
        .ok_or_else(fault)?
        .admit_adjacent(frame::interruption_caller::Admission::inactive(origin.0))
        .map_err(|_| fault())?;
    Ok(())
}

fn bind_controller(text: &str, wram: &[u8], state: &mut frame::State) -> Result<(), Outcome> {
    let fault = || data_failure(None, "adjacent controller initial delay and timer");
    let origin = Sha256::of(wram);
    let controller: ControllerOrigin = toml::from_str(text).map_err(|_| fault())?;
    if controller.format != 1 || Sha256::try_from(controller.origin_sha256.as_str()) != Ok(origin) {
        return Err(fault());
    }
    let word = |offset: usize| {
        let bytes = wram
            .get(offset..offset.saturating_add(2))
            .ok_or_else(fault)?;
        Ok(u16::from_le_bytes(bytes.try_into().map_err(|_| fault())?))
    };
    state
        .frame_history
        .as_mut()
        .ok_or_else(fault)?
        .admit_controller_repeat(
            word(controller.initial_repeat_delay_offset)?,
            word(controller.current_repeat_timer_offset)?,
        );
    Ok(())
}

pub(super) fn bind_camera(
    text: &str,
    content: &mut frame::Content,
    recurring: bool,
) -> Result<(), Outcome> {
    use game::production::frame::s6_camera::Contract;

    let fault = || data_failure(None, "adjacent Camera at-use predicates");
    let camera: Camera = toml::from_str(text).map_err(|_| fault())?;
    if camera.format != 1 {
        return Err(fault());
    }
    let admit = if recurring {
        Contract::admit_recurring
    } else {
        Contract::admit_adjacent
    };
    admit(
        content.s6_camera.as_mut().ok_or_else(fault)?,
        camera.current.into(),
        camera.previous.into(),
        frame::camera_stage::RequiredView {
            x: camera.view.x,
            fraction: camera.view.fraction,
            y: camera.view.y,
        },
        camera
            .screen
            .into_iter()
            .map(|screen| frame::camera_stage::RequiredScreens {
                cell: screen.cell,
                screens: screen.screens.map(Into::into),
            })
            .collect(),
    );
    Ok(())
}

fn bind_video(
    profiles: &str,
    drawing: &str,
    clips: &Clips,
    cartridge: &data::rom::Rom,
    content: &mut frame::Content,
) -> Result<(), Outcome> {
    let fault = || data_failure(None, "adjacent Drawing and finalization physical contracts");
    let drawing: DrawingSelection = toml::from_str(drawing).map_err(|_| fault())?;
    let pose = PoseId(drawing.pose_id);
    let clip = clips.of(pose).ok_or_else(fault)?;
    if drawing.format != 1
        || drawing.upper_pieces.checked_add(drawing.lower_pieces) != Some(9)
        || clip.figure.pose.action != types::Action::Running
        || pose
            .has_cannon_draw(cartridge.lorom())
            .map_err(|_| fault())?
    {
        return Err(fault());
    }
    let step = clip
        .step_of(AnimationFrame(drawing.animation_frame))
        .ok_or_else(fault)?;
    let quiet = |name, totals| quiet_profile::named(profiles, name, totals).map_err(|_| fault());
    content
        .s6_drawing
        .as_mut()
        .ok_or_else(fault)?
        .admit_adjacent(
            &quiet("AdjacentDrawing", [5094, 32586])?,
            clip.figure,
            frame::rendering_stage::ShownStep(step.0),
        )
        .map_err(|_| fault())?;
    let finalization = finalization_before_history(quiet("AdjacentOam", [550, 3578])?, content)?;
    let returns = content.return_work.as_mut().ok_or_else(fault)?;
    returns.adjacent_finalization = Some(finalization.bind(&[]).map_err(|_| fault())?);
    returns.adjacent_nmi = Some(
        frame::frame_return::ControllerTail::new(
            &returns.nmi,
            1367,
            &quiet("AdjacentNmiRepeatTail", [137, 928])?,
        )
        .map_err(|_| data_failure(None, "adjacent NMI controller exposure split"))?,
    );
    Ok(())
}

pub(super) fn finalization_before_history(
    mut physical: data::timing::Template,
    content: &frame::Content,
) -> Result<data::timing::Template, Outcome> {
    let fault = || data_failure(None, "OAM readiness handoff to final history");
    // The native observation includes the read of the list already owned by History.
    let ready = physical.runs.pop().ok_or_else(fault)?;
    let read = data::timing::Template { runs: vec![ready] }
        .bind(&[])
        .map_err(|_| fault())?;
    let tail = content.frame_tail.as_ref().ok_or_else(fault)?;
    if read.runs.first() != tail.history.runs.first() {
        return Err(fault());
    }
    Ok(physical)
}

fn bind_contact(text: &str, profiles: &str, content: &mut frame::Content) -> Result<(), Outcome> {
    let fault = || data_failure(None, "adjacent Contact at-use input");
    let reply: ContactReply = toml::from_str(text).map_err(|_| fault())?;
    if reply.format != 1
        || reply.line != "Contact"
        || reply.observation != "AwaitCueReply"
        || reply.reply != "Absent"
    {
        return Err(fault());
    }
    let binding = content.sender_cue.ok_or_else(fault)?;
    let [prefix, remainder] = quiet_profile::contact_retry(profiles, binding.port)
        .map_err(|_| data_failure(None, "adjacent Contact reply physical profile"))?;
    let contact = frame::sender_contract::ContactRetry::new(
        &prefix,
        remainder,
        audio_transport::Request::Absent,
    )
    .map_err(|_| fault())?;
    content
        .sender_physical
        .as_mut()
        .ok_or_else(fault)?
        .contact_retry = Some(contact);
    Ok(())
}
