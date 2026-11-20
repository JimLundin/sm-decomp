use super::*;
use interface_hud::{Ammo, CancelButton, ControlAnswer, SelectButton, SelectableItem, Selection};
use samus_appearance::CannonDelay;

fn answer(selection: Selection) -> ControlAnswer {
    ControlAnswer {
        inputs: interface_hud::ItemControlInputs {
            select: SelectButton::NotPressed,
            cancel: CancelButton::NotPressed,
            can_switch: true,
            ammunition: interface_hud::Ammunition::new(enum_map::EnumMap::default()).unwrap(),
            equipped: interface_hud::Equipped::default(),
        },
        selection,
        switched: None,
    }
}

fn work(delay: u8) -> Work {
    let mut work = super::tests::work();
    work.cannon = Some(Model(CannonDelay::new(delay).unwrap()));
    work.cannon_before = work.cannon;
    work
}

#[test]
fn complete_cannon_shapes_remain_pinned() {
    assert_eq!(canonical::model::pin_holds::<CannonDelayPin>(), Ok(()));
    assert_eq!(
        canonical::model::pin_holds::<CannonObservationPin>(),
        Ok(())
    );
}

#[test]
fn actual_control_completion_retains_one_answer_and_animation_across_fresh_restore() {
    let mut work = work(2);
    let before_animation = work.model.0.state;
    let controls = answer(Selection::Item(SelectableItem::Ammo(Ammo::Missiles)));
    work.choose_cannon(Selection::None, &controls).unwrap();
    assert_eq!(work.cannon.as_ref().unwrap().0.remaining(), 2);
    let bytes = canonical::to_bytes(&work);
    drop(work);
    let mut restored: Work = canonical::from_bytes(&bytes).unwrap();
    restored.validate().unwrap();
    restored.complete_cannon(&controls).unwrap();
    assert_eq!(restored.cannon.as_ref().unwrap().0.remaining(), 1);
    assert_eq!(restored.model.0.state, before_animation);
    let bytes = canonical::to_bytes(&restored);
    drop(restored);
    let mut restored: Work = canonical::from_bytes(&bytes).unwrap();
    restored.validate().unwrap();
    let before = restored.clone();
    assert_eq!(restored.complete_cannon(&controls), Err(WorkFault::Cannon));
    assert_eq!(restored, before);
}

#[test]
fn unavailable_replaced_or_altered_cannon_owner_rejects_atomically() {
    let controls = answer(Selection::None);
    let mut missing = super::tests::work();
    let before = missing.clone();
    assert_eq!(
        missing.choose_cannon(Selection::None, &controls),
        Err(WorkFault::Cannon)
    );
    assert_eq!(missing, before);
    let mut work = work(2);
    work.choose_cannon(Selection::None, &controls).unwrap();
    work.cannon = Some(Model(CannonDelay::new(0).unwrap()));
    let before = work.clone();
    assert_eq!(
        work.validate_cannon_choice(Selection::None, &controls),
        Err(WorkFault::Cannon)
    );
    assert_eq!(work.validate(), Err(WorkFault::Cannon));
    assert_eq!(work.complete_cannon(&controls), Err(WorkFault::Cannon));
    assert_eq!(work, before);
}

#[test]
fn item_identity_is_independent_of_switch_events_and_one_shot_mode() {
    let item = SelectableItem::GrappleBeam;
    let mut controls = answer(Selection::OneShot(item));
    controls.switched = Some(interface_hud::HudEvent::Switched { item: Some(item) });
    let mut work = work(1);
    work.choose_cannon(Selection::Item(item), &controls)
        .unwrap();
    work.complete_cannon(&controls).unwrap();
    assert!(work.cannon.as_ref().unwrap().0.may_follow());
}
