use super::super::index::Index;
use super::*;
use game::production::{
    ProductionFault, Unsupported,
    appearance::TouchdownGraphics,
    movement_touchdown::{Outputs, Publication},
    sender::Demo,
    world::Boundary,
};
use game::state::owned::OwnedState;

fn admitted_windows() -> [AdmittedCase; 2] {
    let cases = Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
    let sources = Sources {
        cache: std::env::var_os("SM_CACHE").unwrap().into(),
        native: std::env::var_os("SM_AUDIO_NATIVE").unwrap().into(),
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        lock: std::env::var_os("SM_LOCK").unwrap().into(),
        ..Sources::default()
    };

    WINDOWS.map(|(_, cut, _)| {
        let id = format!("movement/soft-touchdown-{cut}");
        let case = cases.cases.iter().find(|case| case.id.0 == id).unwrap();
        admit(case, &sources).unwrap()
    })
}

fn opportunity(inputs: &Inputs) -> Input<'_> {
    Input {
        movement: inputs.movement,
        world: &inputs.world,
        suppression: inputs.suppression,
        audio: crate::replay::sender_offer::offers_only(),
        graphics_area: TouchdownArea::DryClear,
    }
}

#[test]
#[ignore = "pinned native Movement and Contact inputs"]
fn incoherent_figure_refuses_and_restored_owner_plays_on() {
    for mut admitted in admitted_windows() {
        let InputReader::MovementTouchdown(inputs) = admitted.inputs else {
            panic!("touchdown inputs");
        };
        let before = admitted.production.save().unwrap();
        let wrong_figure = AdvanceInput::MovementTouchdown(Input {
            movement: MovementInputs {
                sprite_before_movement: Sprite::ScrewAttack,
                ..inputs.movement
            },
            ..opportunity(&inputs)
        });
        assert!(matches!(
            admitted.production.advance(wrong_figure),
            Err(ProductionFault::Unsupported(
                Unsupported::MovementTouchdownCurrent
            ))
        ));
        assert!(
            admitted.production.save().unwrap() == before,
            "incoherent figure rolled back"
        );
        inputs
            .advance(&mut admitted.production, inputs.cut)
            .unwrap();
        let saved = admitted.production.save().unwrap();
        let mut restored =
            Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
        assert!(
            restored == admitted.production,
            "complete post-opportunity restore"
        );
        // A repeated opportunity is one more frame: the restored owner plays it as the direct one.
        let direct = admitted
            .production
            .advance(AdvanceInput::MovementTouchdown(opportunity(&inputs)));
        let resumed = restored.advance(AdvanceInput::MovementTouchdown(opportunity(&inputs)));
        assert!(direct == resumed, "restored and direct next frames agree");
        assert!(
            restored == admitted.production,
            "restored and direct owners agree"
        );
    }
}

fn advance_without_enqueue(owner: &mut Production, input: Input<'_>) -> Box<Outputs> {
    let OwnedState::MovementTouchdown(before) = owner.state() else {
        panic!("touchdown owned state");
    };
    let contact = &before.audio.0.senders[audio_transport::SoundGroup::Contact];
    let queue = contact.queue.clone();
    let progress = contact.progress.clone();
    let ProductionOutputs::MovementTouchdown(outputs) = owner
        .advance(AdvanceInput::MovementTouchdown(input))
        .unwrap()
    else {
        panic!("touchdown outputs");
    };
    let OwnedState::MovementTouchdown(state) = owner.state() else {
        panic!("touchdown owned state");
    };
    let contact = &state.audio.0.senders[audio_transport::SoundGroup::Contact];
    assert_eq!(contact.queue, queue);
    assert_eq!(contact.progress, progress);
    outputs
}

#[test]
#[ignore = "pinned native Movement and Contact inputs"]
fn graphics_and_contact_obey_separate_permissions() {
    let open = MovementWorld::new(
        1,
        vec![types::Block::Air],
        vec![false],
        vec![],
        Boundary::Air,
    )
    .unwrap();
    for admitted in admitted_windows() {
        let InputReader::MovementTouchdown(inputs) = admitted.inputs else {
            panic!("touchdown inputs");
        };
        let mut owner = admitted.production.clone();
        let mut no_floor = opportunity(&inputs);
        no_floor.world = &open;
        let no_contact = advance_without_enqueue(&mut owner, no_floor);
        assert!(no_contact.movement.floor_response.is_none());
        assert!(!no_contact.movement.touchdown_graphics);
        assert_eq!(no_contact.publications.len(), 0);

        let mut owner = admitted.production.clone();
        let mut cinematic = opportunity(&inputs);
        cinematic.movement.cinematic_active = true;
        let silent = advance_without_enqueue(&mut owner, cinematic);
        assert!(silent.movement.floor_response.is_some());
        assert_eq!(silent.movement.touchdown_requests.len(), 0);
        assert!(matches!(
            silent.publications.as_slice(),
            [Publication::Graphics(
                TouchdownGraphics::ClearLandingSlots { .. }
            )]
        ));

        let mut owner = admitted.production;
        let mut demo = opportunity(&inputs);
        demo.suppression.demo = Demo::Active;
        let suppressed = advance_without_enqueue(&mut owner, demo);
        assert!(matches!(
            suppressed.publications.as_slice(),
            [
                Publication::Contact {
                    cue: CueId::Land,
                    allowance: audio_transport::QueueAllowance::Six
                },
                Publication::Graphics(TouchdownGraphics::ClearLandingSlots { .. }),
            ]
        ));
    }
}
