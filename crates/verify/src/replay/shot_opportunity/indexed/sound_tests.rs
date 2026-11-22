//! Compatibility and admission controls for the public scoped producer boundaries.

use super::*;
use crate::replay::{index::Index, passage_audio};
use audio_transport::{CueId, SenderProgress, SoundGroup, sender};
use canonical::Canonical;
use game::production::{
    AdmissionFault, ProductionFault, Unsupported, gameplay_sounds::GameplaySounds,
};

fn pending(audio: &GameplaySounds, group: SoundGroup) -> GameplaySounds {
    let mut senders = audio.senders().clone();
    let sender = match group {
        SoundGroup::Action => &mut senders.action.0,
        SoundGroup::Ambient => &mut senders.ambient.0,
        SoundGroup::Contact => &mut senders.contact.0,
    };
    sender::begin(sender, sender::SenderProfile::FastBus);
    assert!(senders.get(group).pending.is_some());
    let error = GameplaySounds::new(senders.clone(), audio.queues().clone(), audio.current());
    assert_eq!(
        error,
        Err(AdmissionFault::GameplaySoundServicePending(group))
    );

    // Construct a structurally valid encoded owner that bypasses normal admission.
    let mut encoded = canonical::to_bytes(&senders);
    audio.queues().encode(&mut encoded);
    audio.current().encode(&mut encoded);
    canonical::from_bytes(&encoded).unwrap()
}

fn shot_state(production: &Production) -> &Continuation {
    let OwnedState::ShotOpportunity(state) = production.state() else {
        panic!("wrong shot state");
    };
    state
}

#[test]
#[ignore = "requires the checked shot, passage and sender fixtures and player ROM"]
fn scoped_sound_origins_and_checkpoint_formats() {
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        native: std::env::var_os("SM_AUDIO_NATIVE").unwrap().into(),
        ..Sources::default()
    };
    let index = Index::parse(include_str!(
        "../../../../../../scenarios/replay/index.toml"
    ))
    .unwrap();
    let case = |name| index.cases.iter().find(|case| case.id.0 == name).unwrap();
    let legacy = admit(
        case("weapons-contact/M0-carried-shot-opportunities"),
        &sources,
    )
    .unwrap();
    let live = admit(case("gameplay-sound/shot-opening"), &sources).unwrap();
    shot_formats(&legacy, &live);
    shot_service_boundary(&live);
    pending_passage(case("gameplay-sound/passage-hold"), &sources);
}

fn shot_formats(legacy: &AdmittedCase, live: &AdmittedCase) {
    let ProductionContent::ShotOpportunity(content) = &live.content else {
        panic!("wrong shot content");
    };
    let origin = |bytes: &[u8]| {
        Continuation::restore(
            bytes,
            content.admitted.clone(),
            content.dress,
            &content.power,
        )
    };
    let old = shot_state(&legacy.production).save(content.dress).unwrap();
    let shot = shot_state(&live.production);
    let new = shot.save(content.dress).unwrap();
    assert_ne!(old.get(..8), new.get(..8));
    assert_eq!(origin(&old).unwrap(), *shot_state(&legacy.production));
    assert_eq!(origin(&new).unwrap(), *shot);
    let invalid = || Err(ProductionFault::Unsupported(Unsupported::ShotOpportunity));
    assert_eq!(
        origin(&canonical::to_bytes(&canonical::Schema::<u16>::new())),
        invalid()
    );
    let mut extra = new.clone();
    extra.push(0);
    assert_eq!(origin(&extra), invalid());

    let audio = shot.audio().unwrap();
    let changed = shot.clone().with_audio(pending(audio, SoundGroup::Ambient));
    let malformed = changed.save(content.dress).unwrap();
    let refused = Production::admit(
        ProductionSeed::ShotOpportunity(Box::new(changed)),
        live.content.clone(),
        live.rules.clone(),
    );
    assert_eq!(
        refused,
        Err(AdmissionFault::GameplaySoundServicePending(
            SoundGroup::Ambient
        ))
    );
    assert_eq!(origin(&malformed), invalid());
    assert_eq!(shot.save(content.dress).unwrap(), new);
    let mut senders = audio.senders().clone();
    senders.ambient.0.progress = SenderProgress::AwaitCue(CueId::BackgroundSilence);
    assert!(GameplaySounds::new(senders, audio.queues().clone(), audio.current()).is_ok());
}

fn shot_service_boundary(live: &AdmittedCase) {
    let InputReader::ShotOpportunity(tape) = &live.inputs else {
        panic!("wrong shot tape");
    };
    let mut direct = live.production.clone();
    let outputs = tape.advance(&mut direct, FIRST).unwrap();
    assert!(matches!(
        outputs,
        ProductionOutputs::ShotOpportunityWithAudio { .. }
    ));
    let mut resumed = Production::restore(
        &direct.save().unwrap(),
        live.content.clone(),
        live.rules.clone(),
    )
    .unwrap();
    assert_eq!(resumed, direct);
    assert!(matches!(
        tape.advance(&mut resumed, FIRST),
        Err(Outcome::Unsupported { .. })
    ));
    assert_eq!(resumed, direct);
}

fn pending_passage(case: &Case, sources: &Sources) {
    let admitted = passage_audio::admit(case, sources).unwrap();
    let OwnedState::PassageAudio(state) = admitted.production.state() else {
        panic!("wrong passage state");
    };
    let saved = admitted.production.save().unwrap();
    let changed = game::production::passage_audio::State::touched(pending(
        state.audio(),
        SoundGroup::Contact,
    ));
    let refused = Production::admit(
        ProductionSeed::PassageAudio(Box::new(changed)),
        admitted.content,
        admitted.rules,
    );
    assert_eq!(
        refused,
        Err(AdmissionFault::GameplaySoundServicePending(
            SoundGroup::Contact
        ))
    );
    assert_eq!(admitted.production.save().unwrap(), saved);
    assert_eq!(
        Production::restore(
            &saved,
            ProductionContent::Doors,
            ProductionRules::Doors(world_doors::Rules::default())
        ),
        Err(AdmissionFault::ContentChanged)
    );
}
