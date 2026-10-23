//! Charged-shot glow and held-charge colors, with ordered live-owner feedback.
#![forbid(unsafe_code)]
#[derive(Clone, Copy)]
pub enum Field {
    Glow,
    Phase,
}
#[derive(Clone, Copy)]
pub enum Input {
    Hyper,
    GrappleInactive,
    Charge,
    ContactAttack,
    Suit,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Output {
    Hyper(u16),
    Charge {
        contact_attack: bool,
        suit: u16,
        phase: u16,
    },
    Highlight,
    Visor,
}
pub trait Context<O> {
    fn field(&self, owner: &O, field: Field) -> u16;
    fn set_field(&mut self, owner: &mut O, field: Field, value: u16);
    fn input(&self, owner: &O, input: Input) -> u16;
    fn output(&mut self, owner: O, output: Output) -> (O, u8);
}
pub fn advance<O>(mut owner: O, context: &mut impl Context<O>) -> (O, u8) {
    use Field::*;
    if context.field(&owner, Glow) != 0 {
        if context.input(&owner, Input::Hyper) != 0 {
            if context.field(&owner, Glow) & 1 == 0 {
                if context.field(&owner, Glow) & 0x1e == 0 {
                    context.set_field(&mut owner, Glow, 0);
                    return (owner, 1);
                }
                let phase = (context.field(&owner, Glow) & 0x1e) >> 1;
                owner = context.output(owner, Output::Hyper(phase)).0;
            }
            let glow = context.field(&owner, Glow).wrapping_sub(1);
            context.set_field(&mut owner, Glow, glow);
            return (owner, 0);
        }
        let glow = context.field(&owner, Glow).wrapping_sub(1);
        context.set_field(&mut owner, Glow, glow);
        if context.field(&owner, Glow) == 0 {
            return (owner, 1);
        }
        return (context.output(owner, Output::Highlight).0, 0);
    }
    if context.input(&owner, Input::GrappleInactive) != 0
        && context.input(&owner, Input::Charge) != 0
        && context.input(&owner, Input::Charge).wrapping_sub(60) & 0x8000 == 0
    {
        let output = Output::Charge {
            contact_attack: context.input(&owner, Input::ContactAttack) != 0,
            suit: context.input(&owner, Input::Suit),
            phase: context.field(&owner, Phase),
        };
        owner = context.output(owner, output).0;
        let phase = if context.field(&owner, Phase).wrapping_sub(10) & 0x8000 == 0 {
            0
        } else {
            context.field(&owner, Phase).wrapping_add(2)
        };
        context.set_field(&mut owner, Phase, phase);
        (owner, 0)
    } else {
        context.set_field(&mut owner, Phase, 0);
        context.output(owner, Output::Visor)
    }
}
