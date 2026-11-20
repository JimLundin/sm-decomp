use super::*;
use audio_transport::{CpuMasterTicks, Request, SenderProgress, SenderState, SoundGroup};

fn empty() -> SenderState {
    SenderState {
        queue: std::collections::VecDeque::new(),
        progress: SenderProgress::Ready,
        reply: Request::Absent,
        now: CpuMasterTicks(0),
        pending: None,
    }
}

#[test]
fn origin_positions_are_retained_even_for_empty_fifos() {
    let senders = Senders::new(empty(), empty(), empty()).unwrap();
    let mut wram = vec![0; 131_072];
    for (group, cursor) in [2, 5, 9].into_iter().enumerate() {
        wram[0x0643 + group] = cursor;
        wram[0x0646 + group] = cursor;
    }
    let imported = queues(&wram, &senders).unwrap();
    assert_eq!(imported.window(SoundGroup::Action).positions(), [2, 2]);
    assert_eq!(imported.window(SoundGroup::Ambient).positions(), [5, 5]);
    assert_eq!(imported.window(SoundGroup::Contact).positions(), [9, 9]);
    wram[0x0646] = 3;
    assert_eq!(queues(&wram, &senders), Err(Fault::Queue));
    assert_eq!(queues(&wram[..10], &senders), Err(Fault::Origin));
}

fn run(
    count: u32,
    cycle: data::timing::Cycle,
    kind: data::timing::ExposureKind,
    effect: data::timing::TemplateEffect,
) -> data::timing::TemplateRun {
    data::timing::TemplateRun {
        count,
        cycle,
        kind,
        effect,
        poll_before: true,
    }
}

fn contact_templates(
    port: data::timing::PublicationPort,
) -> (data::timing::Template, data::timing::Template) {
    use data::timing::{Cycle, ExposureKind, Template, TemplateEffect};
    let body = Template {
        runs: vec![
            run(33, Cycle::Six, ExposureKind::Internal, TemplateEffect::None),
            run(1, Cycle::Eight, ExposureKind::Read, TemplateEffect::None),
            run(
                1,
                Cycle::Six,
                ExposureKind::Write,
                TemplateEffect::Publish(port),
            ),
            run(31, Cycle::Six, ExposureKind::Internal, TemplateEffect::None),
        ],
    };
    let tail = Template {
        runs: vec![
            run(19, Cycle::Six, ExposureKind::Internal, TemplateEffect::None),
            run(2, Cycle::Eight, ExposureKind::Read, TemplateEffect::None),
        ],
    };
    (body, tail)
}

fn contact_receipt() -> (game::production::frame::sender_stage::Receipt, Queues) {
    use data::timing::MasterTicks;
    use game::production::frame::sender_stage::Stage;
    let mut senders = Senders::new(empty(), empty(), empty()).unwrap();
    let window = QueueWindow::new(0, 0).unwrap();
    let mut queues = Queues::new(window, window, window, &senders).unwrap();
    let before = senders.contact.0.clone();
    audio_transport::sender::offer(
        &mut senders.contact.0,
        audio_transport::CueId::Footstep,
        audio_transport::QueueAllowance::Six,
        audio_transport::OfferPermission::Enabled,
    );
    queues
        .footstep(
            &before,
            &senders.contact.0,
            audio_transport::OfferPermission::Enabled,
        )
        .unwrap();
    let mut stage = Stage::Idle;
    let receipt = stage
        .choose(
            &mut senders,
            SoundGroup::Contact,
            MasterTicks(100),
            audio_transport::sender::SenderProfile::FastBus,
        )
        .unwrap()
        .clone();
    (receipt, queues)
}

#[test]
fn full_actual_contact_choice_and_template_milestones_are_required() {
    use data::timing::PublicationPort;
    use game::production::frame::sender_stage::CueBinding;
    let port = PublicationPort(0x2143);
    let binding = CueBinding {
        cue: audio_transport::CueId::Footstep,
        port,
        value: 7,
    };
    let (body, tail) = contact_templates(port);
    let contract = Contract::new(SoundGroup::Contact, body.clone(), tail.clone()).unwrap();
    let (receipt, queues) = contact_receipt();
    let current = Current {
        downtime: 0,
        bus: BusRegime::Fast,
    };
    let qualified = contract
        .qualify(&receipt, &queues, current, binding)
        .unwrap();
    assert_eq!(qualified.publication, Some(212));
    assert_eq!(qualified.completion, 398);
    let mut different_reply = receipt.clone();
    different_reply.reply = Request::Cue(audio_transport::CueId::Footstep);
    assert!(
        contract
            .qualify(&different_reply, &queues, current, binding)
            .is_ok()
    );
    let mut wrong = receipt.clone();
    wrong.before = SenderProgress::AwaitCue(audio_transport::CueId::Footstep);
    assert!(contract.qualify(&wrong, &queues, current, binding).is_err());
    let mut wrong = receipt.clone();
    wrong
        .chosen
        .result
        .as_mut()
        .unwrap()
        .queue
        .push_back(audio_transport::CueId::Footstep);
    assert!(contract.qualify(&wrong, &queues, current, binding).is_err());
    let mut wrong = receipt.clone();
    wrong.chosen.result.as_mut().unwrap().completion = CpuMasterTicks(499);
    assert!(contract.qualify(&wrong, &queues, current, binding).is_err());
    assert!(
        contract
            .qualify(
                &receipt,
                &queues,
                Current {
                    downtime: 1,
                    ..current
                },
                binding
            )
            .is_err()
    );
    let mut wrong = receipt.clone();
    wrong.queue.push_back(audio_transport::CueId::Footstep);
    assert!(contract.qualify(&wrong, &queues, current, binding).is_err());
    assert!(
        contract
            .qualify(
                &receipt,
                &queues,
                Current {
                    bus: BusRegime::Slow,
                    ..current
                },
                binding
            )
            .is_err()
    );
    let mut wrong = body;
    wrong.runs[2].poll_before = false;
    wrong.runs[2].count = 2;
    assert!(Contract::new(SoundGroup::Contact, wrong, tail).is_err());
}

#[derive(Deserialize)]
struct Member {
    path: std::path::PathBuf,
    sha256: String,
}

#[derive(Deserialize)]
struct Manifest {
    profiles: [Member; 6],
    wram: Member,
    hardware: Member,
}

fn pinned(path: &std::path::Path, hash: &str) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        crate::lock::Sha256::of(&bytes),
        crate::lock::Sha256::try_from(hash).unwrap()
    );
    bytes
}

fn environment_text(path: &str, pin: &str) -> String {
    let path: std::path::PathBuf = std::env::var_os(path).unwrap().into();
    String::from_utf8(pinned(&path, &std::env::var(pin).unwrap()))
        .map_err(|_| ())
        .unwrap()
}

#[test]
#[ignore = "requires independently reviewed sender current/profile and bus-origin packets"]
fn actual_source_factories_reject_changed_origin_and_execution_contract() {
    let path: std::path::PathBuf = std::env::var_os("SM_FRAME_SENDER_SOURCE").unwrap().into();
    let bytes = pinned(&path, &std::env::var("SM_FRAME_SENDER_SOURCE_SHA").unwrap());
    let manifest: Manifest = toml::from_str(std::str::from_utf8(&bytes).map_err(|_| ()).unwrap())
        .map_err(|_| ())
        .unwrap();
    let root = path.parent().unwrap();
    let read = |member: &Member| {
        let path = if member.path.is_absolute() {
            member.path.clone()
        } else {
            root.join(&member.path)
        };
        pinned(&path, &member.sha256)
    };
    let texts: Vec<String> = manifest
        .profiles
        .iter()
        .map(|member| String::from_utf8(read(member)).map_err(|_| ()).unwrap())
        .collect();
    let refs: [&str; 6] = texts
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let templates = super::super::frame_origin::profiles::decode(refs).unwrap();
    let profile = environment_text("SM_FRAME_SENDER_PROFILE", "SM_FRAME_SENDER_PROFILE_SHA");
    let bus = environment_text("SM_FRAME_SENDER_BUS", "SM_FRAME_SENDER_BUS_SHA");
    let senders = Senders::new(empty(), empty(), empty()).unwrap();
    let wram = read(&manifest.wram);
    let hardware = read(&manifest.hardware);
    let (imported, physical) =
        admit(&profile, &bus, &templates, &wram, &hardware, &senders).unwrap();
    physical
        .admission
        .qualify(&senders, &imported.queues, imported.current)
        .unwrap();
    let mut changed: toml::Value = toml::from_str(&profile).map_err(|_| ()).unwrap();
    changed["execution_layout"]["body_index_register_width_bytes"] = toml::Value::Integer(2);
    assert!(matches!(
        content(&toml::to_string(&changed).unwrap(), &templates),
        Err(Fault::Contract)
    ));
    let proof: BusProof = toml::from_str(&bus).map_err(|_| ()).unwrap();
    let mut changed_hardware = hardware.clone();
    changed_hardware[proof.rom_speed_origin.initial_context_field_offset] ^= 1;
    assert!(matches!(
        origin(&bus, &wram, &changed_hardware, &senders),
        Err(Fault::Origin)
    ));
    let mut changed_wram = wram;
    changed_wram[proof.downtime_origin.private_wram_offset] ^= 1;
    assert!(matches!(
        origin(&bus, &changed_wram, &hardware, &senders),
        Err(Fault::Origin)
    ));
}
