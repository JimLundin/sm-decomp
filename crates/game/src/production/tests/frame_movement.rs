use super::*;
use crate::production::frame::movement::{Movement, ReceiptsPin, WorkFault, WorkPin};
use samus_movement::suspended::MovementDecision;

struct CountedWorld {
    world: world::MovementWorld,
    queries: core::cell::Cell<u32>,
}
impl types::World for CountedWorld {
    fn block(&self, cell: types::Cell) -> types::Block {
        self.queries.set(self.queries.get().saturating_add(1));
        self.world.block(cell)
    }
    fn lets_through(&self, cell: types::Cell, way: types::Way) -> bool {
        self.queries.set(self.queries.get().saturating_add(1));
        self.world.lets_through(cell, way)
    }
    fn solid_enemies(&self) -> &[types::Hitbox] {
        self.queries.set(self.queries.get().saturating_add(1));
        self.world.solid_enemies()
    }
}

#[test]
fn suspended_movement_shape_remains_pinned() {
    assert_eq!(canonical::model::pin_holds::<WorkPin>(), Ok(()));
    assert_eq!(canonical::model::pin_holds::<ReceiptsPin>(), Ok(()));
}

#[test]
fn restored_world_answer_applies_once_without_requery_or_stale_state() {
    let (_, ProductionContent::Movement(content), ProductionRules::Movement(rules)) = movement()
    else {
        panic!("movement context")
    };
    let mut state = fixtures::samus();
    state.vertical.speed = Wrapping(65536);
    let mut owner = Movement::new(state);
    owner.begin().unwrap();
    let world = CountedWorld {
        world: empty_world(),
        queries: core::cell::Cell::new(0),
    };
    let frame = frame();
    loop {
        let decision = owner
            .offer(&content, &rules, &frame, &world)
            .unwrap()
            .unwrap();
        if matches!(decision, MovementDecision::MotionRequest(_)) {
            let before = *owner.state();
            let queries = world.queries.get();
            owner.complete(&content, &rules, &frame, &world).unwrap();
            assert!(world.queries.get() > queries);
            assert_eq!(owner.state(), &before);
            break;
        }
        owner.complete(&content, &rules, &frame, &world).unwrap();
    }
    let Movement::Work(work) = &owner else {
        panic!("active movement")
    };
    assert!(matches!(
        work.model.0.offered,
        Some(MovementDecision::MotionAnswer { .. })
    ));
    let completed = work.completed.clone();
    let pending = work.model.0.offered.clone().unwrap();
    assert!(
        completed
            .0
            .iter()
            .any(|decision| matches!(decision, MovementDecision::Local { .. }))
    );
    let bytes = canonical::to_bytes(&owner);
    drop(owner);
    let mut restored: Movement = canonical::from_bytes(&bytes).unwrap();
    let Movement::Work(work) = &restored else {
        panic!("active movement")
    };
    assert_eq!(work.completed, completed);
    let queries = world.queries.get();
    restored.complete(&content, &rules, &frame, &world).unwrap();
    assert_eq!(world.queries.get(), queries);
    assert!(restored.state().position.y > state.position.y);
    let Movement::Work(work) = &restored else {
        panic!("active movement")
    };
    assert_eq!(work.completed.0.len(), completed.0.len() + 1);
    assert_eq!(work.completed.0.last(), Some(&pending));
    assert_eq!(
        restored.complete(&content, &rules, &frame, &world),
        Err(WorkFault::NoOffer)
    );
    while restored
        .offer(&content, &rules, &frame, &world)
        .unwrap()
        .is_some()
    {
        restored.complete(&content, &rules, &frame, &world).unwrap();
    }
    let Movement::Work(work) = restored else {
        panic!("active movement")
    };
    assert!(work.model.0.is_complete());
    let mut uninterrupted = state;
    let expected =
        crate::production::movement::advance(&mut uninterrupted, &content, &rules, &frame, &world)
            .unwrap();
    assert_eq!(work.model.0.state, uninterrupted);
    assert_eq!(work.model.0.outputs, expected);
}
