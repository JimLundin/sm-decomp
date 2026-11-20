//! Current drawing admission, complete profiles and atomic receipt validation.

#[path = "../src/replay/frame_interrupt_inputs.rs"]
mod frame_interrupt_inputs;
#[path = "../src/replay/frame_profiles.rs"]
mod profiles;

use frame_interrupt_inputs::{InterruptInputFault, current, profiles, terminal_schedule};
use game::production::frame::interrupts::{
    DrawingContent, DrawingFault, DrawingReceipt, DrawingRequest, DrawingTarget, NextRequest,
    VideoSettings,
};
use profiles as frame_profiles;

fn invented_origin() -> Vec<u8> {
    let mut bytes = vec![0; 171];
    bytes.extend(4_u16.to_le_bytes());
    bytes.resize(131_072, 0);
    bytes
}

#[test]
fn current_request_and_complete_next_request_are_checked() {
    let mut bytes = invented_origin();
    assert_eq!(current(&bytes).unwrap().request, DrawingRequest::Begin);
    *bytes.get_mut(171).unwrap() = 6;
    assert_eq!(current(&bytes).unwrap().request, DrawingRequest::End);
    *bytes.get_mut(171).unwrap() = 5;
    assert_eq!(current(&bytes), Err(InterruptInputFault::Request));
    *bytes.get_mut(171).unwrap() = 4;
    *bytes.get_mut(167).unwrap() = 1;
    assert_eq!(current(&bytes), Err(InterruptInputFault::NextRequest));
    assert_eq!(current(&bytes[..1]), Err(InterruptInputFault::Length));

    assert_eq!(
        terminal_schedule(TERMINAL_SCHEDULE),
        Ok([DrawingRequest::End, DrawingRequest::Begin])
    );
    let early = TERMINAL_SCHEDULE.replacen(
        "publish_terminal_only_when_native_return_complete = true",
        "publish_terminal_only_when_native_return_complete = false",
        1,
    );
    assert_eq!(
        terminal_schedule(&early),
        Err(InterruptInputFault::Provenance)
    );
}

const TERMINAL_SCHEDULE: &str = r#"
format = "irq-terminal-hud-schedule-v1"
schedule_owner = "HardwareHudOverlaySchedule"
next_request = "None"
next_request_is_terminal_schedule = false
completion_authority = "ActualFullQualifiedHandlerProfileIncludingNativeReturn"
no_additional_zero_cost_schedule_stage = true
consume_once_at_actual_handler_completion = true
terminal_identity_binding = "ActualAdmittedInterruptCompletionIdentity"
source_coordinates_role = "ProvenanceOnly"
expected_callback_authority = false

[[terminal]]
handler = "HudDrawingBegin"
terminal_schedule = "HudDrawingEnd"
source_last_return_function = "rti"
native_entry_separate_hardware_work = true
native_return_already_in_full_profile_bill = true
publish_terminal_only_when_native_return_complete = true

[[terminal]]
handler = "HudDrawingEnd"
terminal_schedule = "HudDrawingBegin"
source_last_return_function = "rti"
native_entry_separate_hardware_work = true
native_return_already_in_full_profile_bill = true
publish_terminal_only_when_native_return_complete = true
"#;

const fn invented_content() -> DrawingContent {
    DrawingContent {
        hud: VideoSettings {
            background_three_map: 9,
            colour_window: 0,
            colour_math: 0,
            main_layers: 4,
        },
        after_begin: DrawingTarget {
            horizontal: 173,
            vertical: 27,
        },
        after_end: DrawingTarget {
            horizontal: 2,
            vertical: 0,
        },
    }
}

#[test]
#[ignore = "requires independently qualified current IRQ video owner provenance"]
fn unchanged_current_mirrors_require_complete_preceding_writer_proof() {
    let path = std::env::var_os("SM_FRAME_IRQ_VIDEO_PROVENANCE").expect("IRQ video provenance");
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        verify::lock::Sha256::of(&bytes).to_string(),
        "bb2846ae1dcf5d70a4637312cc837a0e5ca62c98d8dcf551c20f9d3816e91320"
    );
    let text = std::str::from_utf8(&bytes).unwrap();
    let inputs = current(&invented_origin()).unwrap();
    let [begin, end] = frame_interrupt_inputs::receipts(inputs, text).unwrap();
    assert_eq!(begin.dependency.request, DrawingRequest::Begin);
    assert_eq!(end.dependency.request, DrawingRequest::End);
    assert!(begin.dependency.video.eq(&end.dependency.video));
    let stale = text.replacen(
        "origin_equal_at_handler_entry = true",
        "origin_equal_at_handler_entry = false",
        1,
    );
    assert_eq!(
        frame_interrupt_inputs::receipts(inputs, &stale),
        Err(InterruptInputFault::Provenance)
    );
    let writer = text.replacen(
        "preceding_source_writer_indices = []",
        "preceding_source_writer_indices = [1]",
        1,
    );
    assert_eq!(
        frame_interrupt_inputs::receipts(inputs, &writer),
        Err(InterruptInputFault::Provenance)
    );
}

#[test]
#[ignore = "requires qualified current origin, checked player ROM and immutable IRQ operand recipe"]
fn qualified_source_import_uses_rom_operands_and_current_owner_fields() {
    let path = std::env::var_os("SM_FRAME_IRQ_CONTENT_RECIPE").expect("IRQ content recipe");
    let recipe = std::fs::read(path).unwrap();
    assert_eq!(
        verify::lock::Sha256::of(&recipe).to_string(),
        "9e9d1a5f2e14b28ad69354721ae128e6fe6eb911d9ea6ddc7211f26ef1858e4e"
    );
    let recipe = std::str::from_utf8(&recipe).unwrap();
    let path = std::env::var_os("SM_FRAME_PLAYER_ROM").expect("checked player ROM");
    let rom = std::fs::read(path).unwrap();
    assert_eq!(
        verify::lock::Sha256::of(&rom).to_string(),
        "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72"
    );
    let content = frame_interrupt_inputs::content(&rom, recipe).unwrap();
    assert!(content.hud.colour_window == 0 && content.hud.colour_math == 0);
    let path = std::env::var_os("SM_FRAME_CURRENT_ORIGIN").expect("qualified current origin");
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        verify::lock::Sha256::of(&bytes).to_string(),
        "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db"
    );
    let inputs = current(&bytes).unwrap();
    assert_eq!(inputs.request, DrawingRequest::Begin);
    let modified = recipe.replacen(
        "origin_kind = \"PreHandlerMirror\"",
        "origin_kind = \"ContentConstant\"",
        1,
    );
    assert!(matches!(
        frame_interrupt_inputs::content(&rom, &modified),
        Err(InterruptInputFault::Content)
    ));
}

#[test]
#[ignore = "requires independently frozen complete physical profile packet"]
fn complete_handlers_restore_and_revalidate_before_any_binding() {
    let mut templates = complete_packet().unwrap();
    assert!(
        templates
            .portions
            .iter()
            .any(|portion| portion.profile == frame_profiles::ProfileName::HudDrawingBegin)
    );
    assert!(
        templates
            .materializations
            .iter()
            .any(|buffer| buffer.name == "HudTilemap" && !buffer.initial_seed)
    );
    let drawing = profiles(&templates, invented_content()).unwrap();
    let restored: game::production::frame::interrupts::DrawingProfiles =
        canonical::from_bytes(&canonical::to_bytes(&drawing)).unwrap();
    assert!(drawing.eq(&restored));
    for request in [DrawingRequest::Begin, DrawingRequest::End] {
        let mut inputs = current(&invented_origin()).unwrap();
        inputs.request = request;
        inputs.video.main_layers = 19;
        let receipt = DrawingReceipt { dependency: inputs };
        let expected = drawing.bind(inputs, Some(&receipt)).unwrap();
        let writes = expected
            .runs
            .iter()
            .filter_map(|run| match &run.exposure.effect {
                data::timing::DeviceEffect::Publish(write) => Some(write),
                data::timing::DeviceEffect::None | data::timing::DeviceEffect::Sample(_) => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(writes.len(), 8);
        let name = match request {
            DrawingRequest::Begin => frame_profiles::ProfileName::HudDrawingBegin,
            DrawingRequest::End => frame_profiles::ProfileName::HudDrawingEnd,
        };
        let source_profile = templates
            .profiles
            .iter()
            .find(|profile| profile.name == name)
            .unwrap();
        let bound = source_profile
            .bind(
                &writes
                    .iter()
                    .map(|write| (*write).clone())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        assert_eq!(bound.name, name);
        assert!(bound.profile.eq(&expected));
        assert_eq!(
            bound
                .samples
                .iter()
                .map(|binding| binding.device)
                .collect::<Vec<_>>(),
            [
                data::timing::Sample::AcknowledgeIrq,
                data::timing::Sample::Blanking
            ]
        );
        assert!(
            bound
                .samples
                .iter()
                .all(|binding| binding.arithmetic.is_none())
        );
        assert_eq!(
            writes.get(3).unwrap().value,
            if request == DrawingRequest::Begin {
                4
            } else {
                19
            }
        );
        assert_eq!(drawing.bind(inputs, None), Err(DrawingFault::Missing));
        inputs.video.colour_math = 22;
        assert_eq!(
            drawing.bind(inputs, Some(&receipt)),
            Err(DrawingFault::Stale)
        );
        inputs.next_request = NextRequest::Pending;
        let receipt = DrawingReceipt { dependency: inputs };
        assert_eq!(
            drawing.bind(inputs, Some(&receipt)),
            Err(DrawingFault::NextRequest)
        );
    }
    break_begin_publication(&mut templates).unwrap();
    assert_eq!(
        profiles(&templates, invented_content()),
        Err(InterruptInputFault::Profile)
    );
}

fn break_begin_publication(templates: &mut frame_profiles::Templates) -> Option<()> {
    let begin = templates
        .profiles
        .iter_mut()
        .find(|profile| profile.name == frame_profiles::ProfileName::HudDrawingBegin)?;
    let write = begin
        .runs
        .iter_mut()
        .find_map(|run| match &mut run.effect {
            frame_profiles::Effect::Publication(publication) => Some(publication),
            frame_profiles::Effect::None | frame_profiles::Effect::Sample(_, _) => None,
        })?;
    "PreparedVideoSettings".clone_into(&mut write.producer);
    Some(())
}

fn complete_packet() -> Result<frame_profiles::Templates, frame_profiles::ProfileFault> {
    use frame_profiles::ProfileFault;
    let directory = std::env::var_os("SM_FRAME_PROFILE_PACKET").ok_or(ProfileFault::Unsupported)?;
    let texts = [
        "profiles.toml",
        "portions.toml",
        "publication-bindings.toml",
        "sample-bindings.toml",
        "materialization-bindings.toml",
        "arithmetic-receipts.toml",
    ]
    .map(|name| {
        std::fs::read_to_string(std::path::Path::new(&directory).join(name))
            .map_err(|_| ProfileFault::Format)
    });
    let [
        profiles,
        portions,
        publications,
        samples,
        materializations,
        arithmetic,
    ] = texts;
    frame_profiles::decode([
        &profiles?,
        &portions?,
        &publications?,
        &samples?,
        &materializations?,
        &arithmetic?,
    ])
}
