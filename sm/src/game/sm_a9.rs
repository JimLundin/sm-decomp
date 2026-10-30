use crate::game::actors;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use sm_game::actors::corpses;
use sm_game::actors::mother_brain_fight;
use sm_game::actors::{baby_metroid_cutscene, baby_metroid_encounter};
use sm_game::enemy_script::EnemyScript;

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;

use crate::game::mother_brain_samus_vitals as vitals;
use crate::game::mother_brain_visuals;
use sm_game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;

pub use crate::types::ConstsB7;

/// Mother Brain's fight runs in `sm_game::actors::mother_brain_fight`.
pub use sm_game::actors::mother_brain_fight::*;
/// Kept unsafe for the player collision ownership checks, which wrap it in an unsafe block.
pub unsafe fn MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(a: u16, game: &mut Game) -> u8 {
    move_samus_vertically_towards_ceiling_floor(a, game)
}
pub(crate) fn move_samus_vertically_towards_ceiling_floor(mut a: u16, game: &mut Game) -> u8 {
    {
        let carry: i32 = game.ram().read_byte(0xafc + 1) as i32 + a.to_ne_bytes()[0] as i32;
        {
            let value: u8 = carry as u8;
            let at: usize = 0xafc + 1;
            game.ram().write_byte(at, value);
        };
        {
            let value: u8 = carry as u8;
            let at: usize = 0xb16 + 1;
            game.ram().write_byte(at, value);
        };
        let v4: u16 =
            (word!(game.SAMUS_Y_POS) as i32 + (a >> 8) as i8 as i32 + (carry >> 8)) as u16;
        let mut v6: u16 = if v4 as i32 - 48i32 & 0x8000 != 0 {
            48
        } else {
            if v4 as i32 - 192i32 & 0x8000 != 0 {
                {
                    let at: usize = 0xafa;
                    let bytes: [u8; 2] = (v4).to_le_bytes();
                    game.ram().write_byte(at, bytes[0]);
                    game.ram().write_byte(at + 1, bytes[1]);
                }
                word!(
                    game.SAMUS_PREV_Y_POS = u16::from_le_bytes([
                        game.ram().read_byte(0xafa),
                        game.ram().read_byte(0xafb)
                    ])
                );
                return 0;
            }
            192
        };
        {
            let at: usize = 0xafa;
            let bytes: [u8; 2] = (v6).to_le_bytes();
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
        word!(
            game.SAMUS_PREV_Y_POS =
                u16::from_le_bytes([game.ram().read_byte(0xafa), game.ram().read_byte(0xafb)])
        );
        word! { game, 0; SAMUS_Y_SUBPOS, SAMUS_PREV_Y_SUBPOS }
        1
    }
}
/// Kept unsafe for the player collision ownership checks, which wrap it in an unsafe block.
pub unsafe fn MotherBrain_MoveSamusHorizTowardsWall(a: u16, game: &mut Game) -> u8 {
    move_samus_horizontally_towards_wall(a, game)
}
pub(crate) fn move_samus_horizontally_towards_wall(mut a: u16, game: &mut Game) -> u8 {
    {
        let carry: i32 = game.ram().read_byte(0xaf8 + 1) as i32 + a.to_ne_bytes()[0] as i32;
        {
            let value: u8 = carry as u8;
            let at: usize = 0xaf8 + 1;
            game.ram().write_byte(at, value);
        };
        {
            let value: u8 = carry as u8;
            let at: usize = 0xb12 + 1;
            game.ram().write_byte(at, value);
        };
        let v4: u16 =
            (word!(game.SAMUS_X_POS) as i32 + (a >> 8) as i8 as i32 + (carry >> 8)) as u16;
        if v4 as i32 - 235i32 & 0x8000 != 0 {
            word! {game; SAMUS_X_POS = v4; SAMUS_PREV_X_POS = v4; }
            0
        } else {
            word! { game, 235; SAMUS_X_POS, SAMUS_PREV_X_POS }
            word! { game, 0; SAMUS_X_SUBPOS, SAMUS_PREV_X_SUBPOS }
            1
        }
    }
}
crate::forward_calls! { pub fn (a: u16, r18: u16, game: &mut Game) -> u16;
    ComputeSinMult => sub_A9C46C(r18, a, game);
    ComputeCosMult => sub_A9C46C((r18).wrapping_add(64), a, game);
}
pub fn sub_A9C46C(a: u16, j: u16, game: &mut Game) -> u16 {
    {
        let mut v2: u16 = kSinCosTable8bit_Sext[((a as i32 & 0xff) + 64) as usize] as u16;
        // Hardware multiply through the PPU's Mode 7 registers.
        unsafe {
            crate::rtl::register_writes! { game;
                byte(M7A as u16, v2 as u8); byte(M7A as u16, v2.to_ne_bytes()[1]);
                byte(M7B as u16, j as u8);
            }
            ReadRegWord(MPYM as u16, game)
        }
    }
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @forward [pub fn (game: &mut Game)];
    [11108367] Enemy_GrappleReact_CancelBeam_A9 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11108417] Enemy_NormalFrozenAI_A9 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
    @external {
        11110023 => mother_brain_fight::initialize_body;
        11110149 => mother_brain_fight::initialize_brain;
        11110206 => mother_brain_fight::body_main_ai;
        11110279 => mother_brain_fight::power_bomb_reaction;
        11110283 => mother_brain_fight::brain_main_ai;
        11111221 => sm_game::actors::mother_brain_tubes::initialize;
        11121923 => mother_brain_fight::body_shot;
        11122118 => mother_brain_fight::brain_touch;
        11129608 => corpses::initialize_torizo_corpse;
        11129704 => corpses::torizo_corpse_main_ai;
        11129898 => corpses::torizo_corpse_power_bomb_reaction;
        11129907 => corpses::torizo_corpse_touch_or_shot;
        11130806 => corpses::initialize_sidehopper_corpse;
        11130953 => corpses::initialize_geemer_corpse;
        11130998 => corpses::initialize_ripper_corpse;
        11131039 => corpses::initialize_skree_corpse;
        11131084 => corpses::sidehopper_corpse_power_bomb_reaction;
        11131099 => corpses::corpse_main_ai;
        11132141 => corpses::power_bomb_reaction_geemer_corpse;
        11132152 => corpses::touch_or_shot_geemer_corpse;
        11132157 => corpses::ripper_corpse_power_bomb_reaction;
        11132168 => corpses::ripper_corpse_touch_or_shot;
        11132173 => corpses::skree_corpse_power_bomb_reaction;
        11132184 => corpses::skree_corpse_touch_or_shot;
        11132189 => corpses::sidehopper_corpse_shot;
        11132228 => corpses::sidehopper_corpse_touch;
        11126544 => baby_metroid_cutscene::initialize;
        11126649 => baby_metroid_cutscene::main_ai;
        11128579 => baby_metroid_cutscene::touch;
        11136823 => baby_metroid_encounter::initialize;
        11136954 => baby_metroid_encounter::power_bomb_reaction;
        11136965 => baby_metroid_encounter::main_ai;
        11138953 => baby_metroid_encounter::touch;
        11139138 => baby_metroid_encounter::shot;
    }
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { match crate::game::mother_brain_body::dispatch_instruction(ea, j, game) { Some(script) => script, None => { crate::rtl::Unreachable(game); EnemyScript::Stop } } };
    @external {
        11115208 => mother_brain_fight::spawn_dust_cloud_at_offset;
        11115247 => mother_brain_fight::spawn_hand_beam_projectile;
        11115269 => mother_brain_fight::increment_hand_beam_attack_phase;
        11136208 => corpses::sidehopper_corpse_end_hop;
        11139476 => baby_metroid_encounter::branch_or_queue_remorse_sound;
    }
}

impl baby_metroid_encounter::Host for Game {
    fn drain_health_by_baby_metroid(&mut self) {
        vitals::drain_health_by_baby_metroid(self);
    }

    fn homing_probe(&mut self) {
        crate::verify_probe::hit("homing.metroid");
    }
}

impl baby_metroid_cutscene::Host for Game {
    fn lights_off(&mut self) {
        mother_brain_visuals::lights_off(self);
    }

    fn heal_by_baby_metroid(&mut self) -> bool {
        vitals::heal_by_baby_metroid(self)
    }
}

impl corpses::Host for Game {
    fn dead_torizo_contact(&mut self) -> bool {
        actors::part_hitboxes::dead_torizo_contact(self)
    }
}

// The game is its own lease: the fight's lease code runs on it directly
// outside a frame's Session lease.
impl sm_game::actors::mother_brain_fight::Lease for Game {
    type Game = Game;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Game) -> T) -> T {
        action(self)
    }
}

impl sm_game::actors::mother_brain_fight::Host for Game {
    type Lease<'a> = crate::game::session_runtime::Runtime<'a>;
    fn lease(&mut self) -> crate::game::session_runtime::Runtime<'_> {
        crate::game::session_runtime::Runtime::new(self)
    }

    fn bank_unreachable(&mut self) -> bool {
        crate::rtl::Unreachable(self)
    }

    fn spawn_legacy_population(&mut self, bank: u8, population: u16) -> u16 {
        crate::game::actors::population::spawn_legacy(
            &mut crate::game::session_runtime::Runtime::new(self),
            bank,
            population,
        )
    }

    fn set_projectile_spawn_r22(&mut self, value: u16) {
        self.eproj_spawn_r22 = value;
    }

    fn create_dud(&mut self) {
        crate::game::actors::combat::create_dud(&mut crate::game::session_runtime::Runtime::new(
            self,
        ));
    }

    fn fade(&mut self, kind: sm_game::actors::mother_brain_fight::Fade, frame: u16) -> u8 {
        crate::game::mother_brain_visuals::fade(kind, frame, self)
    }

    fn update_health_palette(&mut self) {
        crate::game::mother_brain_visuals::update_health_palette(self);
    }

    fn seal_wall(&mut self) {
        crate::game::mother_brain_visuals::seal_wall(self);
    }

    fn flicker_body(&mut self) {
        crate::game::mother_brain_visuals::flicker_body(self);
    }

    fn drain_health_by_rainbow_beam(&mut self) {
        crate::game::mother_brain_samus_vitals::drain_health_by_rainbow_beam(self);
    }

    fn drain_ammo_by_rainbow_beam(&mut self) {
        crate::game::mother_brain_samus_vitals::drain_ammo_by_rainbow_beam(self);
    }

    fn add_glass_hits(&mut self, slot: usize, amount: u16) {
        crate::game::room_mechanisms::add_glass_hits(slot, amount, self);
    }

    fn spawn_palette_effect(&mut self, definition: u16) {
        crate::game::palette_effects::spawn(definition, self);
    }

    fn queue_mother_brain_sprite_tiles(&mut self, bank: u8, start: u16) -> u8 {
        crate::game::rom_vram_entries::queue_mother_brain_sprite_tiles(bank, start, self)
    }

    fn process_corpse_rotting(&mut self, k: u16) -> u8 {
        corpses::process_corpse_rotting(k, self)
    }

    fn process_corpse_rotting_vram_transfers(&mut self, k: u16) {
        corpses::process_corpse_rotting_vram_transfers(k, self);
    }

    fn start_zebes_escape_typewriter(&mut self) {
        crate::game::escape_typewriter::start_zebes_escape_typewriter(self);
    }

    fn type_escape_text(&mut self, tiles: u16) -> u8 {
        crate::game::escape_typewriter::type_escape_text(tiles, self)
    }

    fn set_boss_bit_for_current_area(&mut self, mask: u16) {
        crate::game::gameplay_flag_runtime::set_boss_bits_for_current_area(mask, self);
    }

    fn set_event_happened(&mut self, event: u16) {
        crate::game::gameplay_flag_runtime::mark_event(event, self);
    }

    fn check_event_happened(&mut self, event: u16) -> u16 {
        crate::game::gameplay_flag_runtime::event_has_happened(event, self)
    }

    fn spawn_rainbow_hdma(&mut self) -> u16 {
        crate::game::mother_brain_visuals::spawn_rainbow_hdma(self)
    }

    fn spawn_rising_hdma(&mut self) -> u16 {
        crate::game::mother_brain_visuals::spawn_rising_hdma(self)
    }

    fn player_mother_brain(&mut self, command: u16) {
        crate::game::session_runtime::player_mother_brain(command, self);
    }

    fn load_fx_entry(&mut self, index: u16) {
        crate::game::room_state::load_fx_entry(index, self);
    }

    fn native_word(&mut self, offset: usize) -> u16 {
        crate::game::display_refresh::native_word(offset, self)
    }

    fn set_native_word(&mut self, offset: usize, value: u16) {
        crate::game::display_refresh::set_native_word(offset, value, self);
    }

    fn samus_contact(
        &mut self,
        boxes: u16,
        x: u16,
        y: u16,
    ) -> sm_game::actors::mother_brain_fight::Contact {
        use crate::game::actors::part_hitboxes::Contact as Part;
        use sm_game::actors::mother_brain_fight::Contact;
        match crate::game::actors::part_hitboxes::samus_contact(boxes, x, y, self) {
            Part::None => Contact::None,
            Part::Pushed => Contact::Pushed,
            Part::Hurt => Contact::Hurt,
        }
    }

    fn deal_contact_damage(&mut self) {
        sm_game::actors::ridley::deal_contact_damage(self);
    }

    fn move_samus_horizontally_towards_wall(&mut self, amount: u16) -> u8 {
        crate::game::sm_a9::move_samus_horizontally_towards_wall(amount, self)
    }

    fn initialize_corpse_rotting(&mut self, k: u16, definition: u16) {
        corpses::initialize_enemy_corpse_rotting(k, definition, self);
    }

    fn queue_sound_result(&mut self, channel: usize, sound: u8, max_queued: u8) -> u16 {
        crate::game::music::queue_sfx(channel, sound, max_queued, self)
    }

    fn move_samus_vertically_towards_ceiling_floor(&mut self, amount: u16) -> u8 {
        crate::game::sm_a9::move_samus_vertically_towards_ceiling_floor(amount, self)
    }
}
