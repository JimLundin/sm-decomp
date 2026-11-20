use super::*;
use crate::production::timing::WorkId;
use interface_hud::{
    Ammunition, CancelButton, Equipped, HudGrid, HudState, MapCellSet, SelectButton,
    SelectableItem, Shown, Stock,
};

fn owner() -> Hud {
    Hud::new(HudState {
        countdown: interface_hud::Countdown::Dormant,
        grid: HudGrid::filled(HudPictures::default().blank_cell()),
        shown: Shown::default(),
        selection: Selection::None,
        explored: MapCellSet::default(),
        mini_map: MiniMapPower::On,
    })
}

fn inputs(grapple_beam: bool) -> ItemControlInputs {
    ItemControlInputs {
        select: SelectButton::Pressed,
        cancel: CancelButton::NotPressed,
        can_switch: true,
        ammunition: Ammunition::new(enum_map::enum_map! {
            _ => Stock { count: 0, capacity: 11 },
        })
        .unwrap(),
        equipped: Equipped {
            grapple_beam,
            x_ray_scope: false,
        },
    }
}

#[test]
fn complete_control_answer_shape_remains_pinned() {
    assert_eq!(canonical::model::pin_holds::<ControlAnswerPin>(), Ok(()));
}

#[test]
fn completed_controls_receipt_restores_full_inputs_and_sole_completed_owner() {
    let mut hud = owner();
    let actual_inputs = inputs(true);
    choose_controls(&mut hud, actual_inputs).unwrap();
    let chosen = canonical::to_bytes(&hud);
    drop(hud);
    let mut hud: Hud = canonical::from_bytes(&chosen).unwrap();
    let receipt = complete_controls_receipt(&mut hud, WorkId(37)).unwrap();
    assert_eq!(receipt.work, WorkId(37));
    assert_eq!(receipt.answer.0.inputs, actual_inputs);
    assert_eq!(
        receipt.answer.0.selection,
        Selection::Item(SelectableItem::GrappleBeam)
    );
    assert_eq!(
        receipt.answer.0.switched,
        Some(HudEvent::Switched {
            item: Some(SelectableItem::GrappleBeam)
        })
    );
    let receipt_bytes = canonical::to_bytes(&receipt);
    let completed_bytes = canonical::to_bytes(&hud);
    drop(hud);
    let receipt: ControlsCompletion = canonical::from_bytes(&receipt_bytes).unwrap();
    let mut hud: Hud = canonical::from_bytes(&completed_bytes).unwrap();
    assert_eq!(
        hud.controls(),
        Some(&Observation::Completed(receipt.answer.0.clone()))
    );
    let completed = hud.clone();
    assert_eq!(
        complete_controls_receipt(&mut hud, WorkId(38)),
        Err(StageFault::Order)
    );
    assert_eq!(complete_controls(&mut hud), Err(StageFault::Order));
    assert_eq!(hud, completed);
    assert_eq!(receipt.work, WorkId(37));
}

#[test]
fn completed_controls_receipt_requires_chosen_work_and_captures_current_inputs() {
    let mut hud = owner();
    let initial = hud.clone();
    assert_eq!(
        complete_controls_receipt(&mut hud, WorkId(1)),
        Err(StageFault::Order)
    );
    assert_eq!(hud, initial);
    let earlier = inputs(true);
    let current = inputs(false);
    choose_controls(&mut hud, current).unwrap();
    let receipt = complete_controls_receipt(&mut hud, WorkId(2)).unwrap();
    assert_eq!(receipt.answer.0.inputs, current);
    assert_ne!(receipt.answer.0.inputs, earlier);
    assert_eq!(receipt.answer.0.selection, Selection::None);
    assert_eq!(
        receipt.answer.0.switched,
        Some(HudEvent::Switched { item: None })
    );
    assert_eq!(hud.events(), [HudEvent::Switched { item: None }]);
}
