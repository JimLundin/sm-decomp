//! Ordered player preparation, separate from later movement and frame completion.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Entry {
    Normal,
    Demo,
    ProjectilesOnly,
    Inactive,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Step {
    Radius,
    Input,
    SuitIndex,
    Acceleration,
    InsideBlock,
    Weapons,
    Projectiles,
}
#[derive(Clone, Copy)]
pub enum Input {
    Held,
    Pressed,
    FilteredHeld,
    FilteredPressed,
    AimHistory,
    MovementAndFacing,
}
#[derive(Clone, Copy)]
pub enum Field {
    NextPose,
    InterruptedPose,
    TransitionalPose,
    MomentumRoutine,
    SpecialGraphics,
    HurtSwitch,
    DemoHeld,
    DemoPressed,
    DemoFilteredHeld,
    DemoFilteredPressed,
    AnimationSkip,
    PendingAim,
    AimHistory,
    PreviousMovementAndFacing,
}
pub trait Context<O> {
    fn read(&self, owner: &O, input: Input) -> u16;
    fn write(&mut self, owner: &mut O, field: Field, value: u16);
    fn clear_launch_history(&mut self, owner: &mut O);
    fn step(&mut self, owner: O, step: Step) -> O;
}
pub fn prepare<O>(mut owner: O, entry: Entry, context: &mut impl Context<O>) -> O {
    match entry {
        Entry::Inactive => return owner,
        Entry::ProjectilesOnly => owner = context.step(owner, Step::Projectiles),
        Entry::Normal | Entry::Demo => {
            for field in [
                Field::NextPose,
                Field::InterruptedPose,
                Field::TransitionalPose,
            ] {
                context.write(&mut owner, field, 0xffff);
            }
            for field in [
                Field::MomentumRoutine,
                Field::SpecialGraphics,
                Field::HurtSwitch,
            ] {
                context.write(&mut owner, field, 0);
            }
            if entry == Entry::Demo {
                for (input, field) in [
                    (Input::Held, Field::DemoHeld),
                    (Input::Pressed, Field::DemoPressed),
                    (Input::FilteredHeld, Field::DemoFilteredHeld),
                    (Input::FilteredPressed, Field::DemoFilteredPressed),
                ] {
                    let value = context.read(&owner, input);
                    context.write(&mut owner, field, value);
                }
            }
            owner = context.step(owner, Step::Radius);
            for step in if entry == Entry::Normal {
                [Step::Input, Step::SuitIndex]
            } else {
                [Step::SuitIndex, Step::Input]
            } {
                owner = context.step(owner, step);
            }
            owner = context.step(owner, Step::Acceleration);
            owner = context.step(owner, Step::InsideBlock);
            owner = context.step(owner, Step::Weapons);
        }
    }
    context.clear_launch_history(&mut owner);
    context.write(&mut owner, Field::AnimationSkip, 0);
    context.write(&mut owner, Field::PendingAim, 0);
    let history = context.read(&owner, Input::AimHistory).wrapping_shl(8);
    context.write(&mut owner, Field::AimHistory, history);
    let movement = context.read(&owner, Input::MovementAndFacing);
    context.write(&mut owner, Field::PreviousMovementAndFacing, movement);
    owner
}
