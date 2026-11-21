use super::{MEMBER, Packet, admit};
use crate::replay::audio::load;
use audio_transport::sender::{ReplyInput, SenderOutput, SenderProfile};
use audio_transport::transport::Output;
use audio_transport::{
    CpuMasterTicks, CueId, OfferPermission, QueueAllowance, Request, SoundGroup, time,
};
use game::production::{
    AdmissionFault, AdvanceInput, Production, ProductionContent, ProductionOutputs,
    ProductionRules, SenderInput, audio,
};
use game::state::owned::OwnedState;
use std::path::PathBuf;

const TAPE: &str = "7b7eb9f99ad4e148889376e154aff5e8f30fe887d01f105ecb864c6c74d1aa52";

fn path(variable: &str) -> PathBuf {
    std::env::var_os(variable).unwrap().into()
}

fn sources() -> (Production, Box<audio::Content>, super::InputTape, Vec<u8>) {
    let native = path("SM_AUDIO_NATIVE");
    let (tape, _) = load(&native.join("control.local.toml"), TAPE, 0, None).unwrap();
    let (seed, content) = admit(&path("SM_ROM"), &native, &tape, TAPE).unwrap();
    let production = Production::admit(seed, content.clone(), ProductionRules::Audio).unwrap();
    let ProductionContent::ResidentAudio(content) = content else {
        panic!("joint production scope");
    };
    (
        production,
        content,
        tape,
        std::fs::read(native.join(MEMBER)).unwrap(),
    )
}

fn resident(production: &Production) -> &audio::State {
    let OwnedState::ResidentAudio(state) = production.state() else {
        panic!("joint production scope");
    };
    state
}

fn restore(production: &Production, content: &audio::Content) -> Production {
    let saved = production.save().unwrap();
    let restored = Production::restore(
        &saved,
        ProductionContent::ResidentAudio(Box::new(content.clone())),
        ProductionRules::Audio,
    )
    .unwrap();
    assert!(restored.eq(production));
    assert!(restored.save().unwrap().eq(&saved));
    restored
}

fn advance(production: &mut Production, halves: u64) -> Vec<Output> {
    let ProductionOutputs::Audio(outputs) = production
        .advance(AdvanceInput::Audio {
            stop: time::halves(halves),
            presentations: &[],
        })
        .unwrap()
    else {
        panic!("transport output");
    };
    outputs
}

#[test]
#[ignore = "requires the pinned ROM, audio tape and reviewed room-origin projection"]
fn first_empty_poll_survives_joint_checkpoint() {
    let (mut direct, content, tape, _) = sources();
    assert!(resident(&direct).transport().eq(&tape.seed));
    assert_eq!(advance(&mut direct, 810).len(), 0);
    let mut resumed = restore(&direct, &content);
    for halves in [811, 827] {
        let expected = vec![Output::Clock {
            at: time::halves(halves),
            batch: 0,
        }];
        assert!(advance(&mut direct, halves).eq(&expected));
        assert!(advance(&mut resumed, halves).eq(&expected));
        assert!(direct.eq(&resumed));
        assert!(restore(&direct, &content).eq(&direct));
    }
}

fn sender(
    production: &mut Production,
    group: SoundGroup,
    input: SenderInput<'_>,
) -> Vec<SenderOutput> {
    let ProductionOutputs::AudioSender(outputs) = production
        .advance(AdvanceInput::ResidentSender { group, input })
        .unwrap()
    else {
        panic!("sender output");
    };
    outputs
}

#[test]
#[ignore = "requires the pinned ROM, audio tape and reviewed room-origin projection"]
fn all_three_pending_senders_survive_joint_checkpoint() {
    let (mut direct, content, _, _) = sources();
    let cues = [
        (SoundGroup::Action, CueId::Spin),
        (SoundGroup::Ambient, CueId::BackgroundSilence),
        (SoundGroup::Contact, CueId::Footstep),
    ];
    for (group, cue) in cues {
        assert_eq!(
            sender(
                &mut direct,
                group,
                SenderInput::Offer {
                    cue,
                    allowance: QueueAllowance::One,
                    permission: OfferPermission::Enabled,
                }
            )
            .len(),
            0
        );
        assert_eq!(
            sender(
                &mut direct,
                group,
                SenderInput::Begin(SenderProfile::FastBus)
            )
            .len(),
            0
        );
        assert_eq!(
            sender(
                &mut direct,
                group,
                SenderInput::Advance {
                    stop: CpuMasterTicks(211),
                    replies: &[],
                }
            )
            .len(),
            0
        );
    }
    let mut resumed = restore(&direct, &content);
    for (group, cue) in cues {
        let expected = vec![SenderOutput::Presentation(
            CpuMasterTicks(212),
            Request::Cue(cue),
        )];
        for production in [&mut direct, &mut resumed] {
            assert!(
                sender(
                    production,
                    group,
                    SenderInput::Advance {
                        stop: CpuMasterTicks(212),
                        replies: &[],
                    }
                )
                .eq(&expected)
            );
        }
    }
    let mut after = restore(&direct, &content);
    for (group, _) in cues {
        for production in [&mut direct, &mut resumed, &mut after] {
            assert!(
                sender(
                    production,
                    group,
                    SenderInput::Advance {
                        stop: CpuMasterTicks(398),
                        replies: &[],
                    }
                )
                .eq(&[SenderOutput::Completed(CpuMasterTicks(398))])
            );
        }
    }
    assert!(direct.eq(&resumed));
    assert!(direct.eq(&after));
}

#[test]
#[ignore = "requires the pinned ROM, audio tape and reviewed room-origin projection"]
fn substituted_room_content_and_incomplete_load_are_refused() {
    let (production, content, tape, bytes) = sources();
    let (origin, _) = Packet::load(&bytes, TAPE).unwrap();
    let mut substitute = content.bank().clone();
    let sample = substitute.samples.first_mut().unwrap();
    sample.source_sha256[0] ^= 1;
    let substitute = audio::Content::new(substitute, origin).unwrap();
    assert!(matches!(
        Production::restore(
            &production.save().unwrap(),
            ProductionContent::ResidentAudio(Box::new(substitute)),
            ProductionRules::Audio,
        ),
        Err(AdmissionFault::ContentChanged)
    ));
    let mut resource_missing = content.bank().clone();
    resource_missing.samples.clear();
    assert!(matches!(
        audio::Content::new(resource_missing, origin),
        Err(AdmissionFault::ResidentAudio(audio::Fault::Invalid(_)))
    ));

    let fresh = |state, load| audio::State::opening_ceres(Box::new(state), load, &content);
    let (_, load) = Packet::load(&bytes, TAPE).unwrap();
    let mut seed = tape.seed.clone();
    seed.song.tracks.pop();
    assert!(fresh(seed, load).is_err());
    let (_, load) = Packet::load(&bytes, TAPE).unwrap();
    let mut seed = tape.seed.clone();
    seed.groups[SoundGroup::Contact].history.pop();
    assert!(fresh(seed, load).is_err());
    let (_, load) = Packet::load(&bytes, TAPE).unwrap();
    let mut seed = tape.seed;
    let shifted = seed.clock.lattice.sum(&time::cycles(8));
    seed.clock.lattice = shifted.clone();
    seed.phase = audio_transport::AudioPhase::ObserveClock { at: shifted };
    assert!(fresh(seed, load).is_err());
}

#[test]
#[ignore = "requires the reviewed room-origin projection"]
fn changed_origin_clock_or_lineage_is_refused() {
    let bytes = std::fs::read(path("SM_AUDIO_NATIVE").join(MEMBER)).unwrap();
    let packet = || toml::from_str::<Packet>(std::str::from_utf8(&bytes).unwrap()).unwrap();
    let mut wrong = packet();
    wrong.loaded_smp_offset = wrong.loaded_smp_offset.checked_add(1).unwrap();
    assert!(wrong.admit(TAPE).is_err());
    let mut wrong = packet();
    wrong.first_poll_num = wrong.first_poll_num.checked_sub(16).unwrap();
    assert!(wrong.admit(TAPE).is_err());
    let mut wrong = packet();
    wrong.room = "ceres-elevator".to_owned();
    assert!(wrong.admit(TAPE).is_err());
    let mut wrong = packet();
    wrong.locked_seed_sha256 = TAPE.to_owned();
    assert!(wrong.admit(TAPE).is_err());
    assert!(packet().admit("unrelated tape").is_err());
    let mut bytes = bytes;
    bytes.push(b' ');
    assert!(Packet::load(&bytes, TAPE).is_err());
}

#[test]
#[ignore = "requires the pinned ROM, audio tape and reviewed room-origin projection"]
fn asymmetric_physical_latches_survive_joint_checkpoint() {
    let (mut direct, content, _, _) = sources();
    let action = Request::Cue(CueId::Spin);
    let ambient = Request::Cue(CueId::BackgroundSilence);
    let reply = ReplyInput {
        at: CpuMasterTicks(80),
        reply: ambient,
    };
    assert_eq!(
        sender(
            &mut direct,
            SoundGroup::Ambient,
            SenderInput::Advance {
                stop: CpuMasterTicks(100),
                replies: &[reply],
            }
        )
        .len(),
        0
    );
    let presentation = audio_transport::transport::Input::cue(time::cycles(2), CueId::Spin);
    assert!(
        direct
            .advance(AdvanceInput::Audio {
                stop: time::cycles(4),
                presentations: &[presentation],
            })
            .unwrap()
            .eq(&ProductionOutputs::Audio(Vec::new()))
    );

    let restored = restore(&direct, &content);
    let state = resident(&restored);
    assert_eq!(state.transport().requests[SoundGroup::Action], action);
    assert_eq!(
        state.transport().groups[SoundGroup::Action].reply,
        Request::Absent
    );
    assert_eq!(
        state.senders().get(SoundGroup::Action).reply,
        Request::Absent
    );
    assert_eq!(
        state.transport().requests[SoundGroup::Ambient],
        Request::Absent
    );
    assert_eq!(state.senders().get(SoundGroup::Ambient).reply, ambient);

    // The typed relation retains the loaded lead, even though the two owners have since
    // advanced independently in this scoped construction.
    assert!(state.clock().at_cpu(&CpuMasterTicks(18)).is_none());
    let next = audio_transport::AudioInstant::new(98_165u64.into(), 2_684_659u64.into()).unwrap();
    assert_eq!(state.clock().at_cpu(&CpuMasterTicks(19)), Some(next));
}

#[test]
#[ignore = "requires the pinned ROM, audio tape and reviewed room-origin projection"]
fn room_load_joins_the_cpu_request_latch() {
    let (_, content, tape, bytes) = sources();
    let (_, mut load) = Packet::load(&bytes, TAPE).unwrap();
    load.requests[SoundGroup::Contact] = Request::Cue(CueId::Footstep);
    assert!(matches!(
        audio::State::opening_ceres(Box::new(tape.seed), load, &content),
        Err(AdmissionFault::ResidentAudio(audio::Fault::Origin))
    ));
}

#[test]
#[ignore = "requires the pinned ROM, audio tape and reviewed room-origin projection"]
fn room_load_joins_the_transport_reply_latch() {
    let (_, content, tape, bytes) = sources();
    let (_, load) = Packet::load(&bytes, TAPE).unwrap();
    let mut seed = tape.seed;
    seed.groups[SoundGroup::Contact].reply = Request::Cue(CueId::Footstep);
    assert!(matches!(
        audio::State::opening_ceres(Box::new(seed), load, &content),
        Err(AdmissionFault::ResidentAudio(audio::Fault::Origin))
    ));
}
