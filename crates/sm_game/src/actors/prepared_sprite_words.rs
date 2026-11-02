//! Bounded sprite words use the actual active/tail storage.
#![forbid(unsafe_code)]
use super::PreparedActorState;
use crate::sprite_effect_host::SpriteMember;
impl PreparedActorState<'_> {
    pub(super) fn typed_sprite_word(&self, member: SpriteMember, index: u16) -> Option<u16> {
        let index = usize::from(index);
        if let Some(slot) = self.owner.gameplay.sprite_effects.active.get(index) {
            return Some(match member {
                SpriteMember::Instruction => slot.instruction,
                SpriteMember::Timer => slot.timer,
                SpriteMember::Appearance => slot.appearance,
                SpriteMember::X => slot.x.whole,
                SpriteMember::XFraction => slot.x.fraction,
                SpriteMember::Y => slot.y.whole,
                SpriteMember::YFraction => slot.y.fraction,
                SpriteMember::DisableFlags => slot.disable_flags,
            });
        }
        let index = index.checked_sub(32).filter(|index| *index < 32)?;
        let tail = &self.owner.gameplay.sprite_compatibility;
        Some(match member {
            SpriteMember::Instruction => tail.instruction[index],
            SpriteMember::Timer => tail.timer[index],
            SpriteMember::Appearance => tail.appearance[index],
            SpriteMember::X => tail.x[index],
            SpriteMember::XFraction => tail.x_fraction[index],
            SpriteMember::Y => tail.y[index],
            SpriteMember::YFraction => tail.y_fraction[index],
            SpriteMember::DisableFlags => tail.disable_flags[index],
        })
    }
    pub(super) fn store_typed_sprite_byte(
        &mut self,
        member: SpriteMember,
        index: u16,
        lane: usize,
        value: u8,
    ) -> bool {
        let index = usize::from(index);
        let field = if let Some(slot) = self.owner.gameplay.sprite_effects.active.get_mut(index) {
            match member {
                SpriteMember::Instruction => &mut slot.instruction,
                SpriteMember::Timer => &mut slot.timer,
                SpriteMember::Appearance => &mut slot.appearance,
                SpriteMember::X => &mut slot.x.whole,
                SpriteMember::XFraction => &mut slot.x.fraction,
                SpriteMember::Y => &mut slot.y.whole,
                SpriteMember::YFraction => &mut slot.y.fraction,
                SpriteMember::DisableFlags => &mut slot.disable_flags,
            }
        } else {
            let Some(index) = index.checked_sub(32).filter(|index| *index < 32) else {
                return false;
            };
            let tail = &mut self.owner.gameplay.sprite_compatibility;
            match member {
                SpriteMember::Instruction => &mut tail.instruction[index],
                SpriteMember::Timer => &mut tail.timer[index],
                SpriteMember::Appearance => &mut tail.appearance[index],
                SpriteMember::X => &mut tail.x[index],
                SpriteMember::XFraction => &mut tail.x_fraction[index],
                SpriteMember::Y => &mut tail.y[index],
                SpriteMember::YFraction => &mut tail.y_fraction[index],
                SpriteMember::DisableFlags => &mut tail.disable_flags[index],
            }
        };
        let mut bytes = field.to_le_bytes();
        bytes[lane] = value;
        *field = u16::from_le_bytes(bytes);
        true
    }
}
