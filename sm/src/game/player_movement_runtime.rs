//! Compatibility content and exceptional echo aliases for owned locomotion.
use crate::{
    game::{
        liquid_surface::LiquidAtHeight,
        player_movement::{
            content::{Content, FootstepRegion, FootstepRule, SpeedRecord},
            effects::{EchoAxis, Effects},
        },
        session_runtime::Runtime,
        visual_host::{self, Source},
    },
    rom::RomRecord,
};
const CRATERIA_FOOTSTEP_FLAGS: [u8; 16] = [1, 0, 0, 0, 0, 2, 0, 4, 0, 4, 4, 4, 4, 0, 4, 0];
impl Content for Runtime<'_> {
    fn speed_record(&mut self, profile: SpeedRecord) -> RomRecord {
        let cursor = match profile {
            SpeedRecord::Selected(cursor) => cursor,
            SpeedRecord::Bomb => 0x9f25,
            SpeedRecord::GrappleAir => 0x9f31,
            SpeedRecord::GrappleWater => 0x9f3d,
            SpeedRecord::GrappleLava => 0x9f49,
        };
        if cursor & 0x8000 == 0 {
            self.foreign(|game| unsafe {
                crate::rtl::rom_bank_ptr::<0x90>(cursor, game);
            });
        }
        visual_host::Content::table(self, Source::Native, 0x900000 | u32::from(cursor))
    }
    fn liquid_speed_table(&self, liquid: LiquidAtHeight) -> Option<u16> {
        match liquid {
            LiquidAtHeight::None => None,
            LiquidAtHeight::Water => Some(0xa08d),
            LiquidAtHeight::LavaAcid => Some(0xa1dd),
        }
    }
    fn spin_sound_frames(&self) -> RomRecord {
        visual_host::Content::table(self, Source::Native, 0x90a514)
    }
    fn footstep_region(&self) -> FootstepRegion {
        use FootstepRegion::*;
        [
            Crateria, Default, Default, Default, Sand, Default, Default, Default,
        ][usize::from(self.session().room.area)]
    }
    fn footstep_room_rule(&self) -> FootstepRule {
        use FootstepRule::*;
        let room = self.session().room.entry.room_index;
        if (room as i16).wrapping_sub(16) >= 0 {
            return Default;
        }
        let flags = CRATERIA_FOOTSTEP_FLAGS[usize::from(room)];
        if flags & 1 != 0 {
            SandWhenFxTen
        } else if flags & 2 != 0 {
            SandAtOrBelow944
        } else if flags & 4 != 0 {
            Sand
        } else {
            Default
        }
    }
}
impl Effects for Runtime<'_> {
    fn boss_active(&self) -> bool {
        self.word(0x179c) != 0
    }
    fn write_echo_coordinate(&mut self, axis: EchoAxis, index: usize, value: u16) {
        let base = match axis {
            EchoAxis::X => 0xab0usize,
            EchoAxis::Y => 0xab8usize,
        };
        let at = base
            .checked_add(index.checked_mul(2).expect("echo coordinate extent"))
            .expect("echo coordinate address");
        self.write_word_le(value, at);
    }
}

pub(crate) fn ceres_movement(game: &mut crate::game_state::Game, action: fn(&mut Runtime<'_>)) {
    let mut runtime = Runtime::new(game);
    action(&mut runtime);
    runtime.foreign(|game| unsafe {
        crate::game::sm_80::ProcessTimer(game);
        if crate::ram_map::word!(game.timer_status) != 0 {
            crate::game::sm_80::DrawTimer(game);
        }
    });
}
