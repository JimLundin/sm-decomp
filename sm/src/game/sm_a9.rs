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

pub use crate::types::Consts_B7;

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
            (word!(game.samus_y_pos) as i32 + (a >> 8) as i8 as i32 + (carry >> 8)) as u16;
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
                    game.samus_prev_y_pos = u16::from_le_bytes([
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
            game.samus_prev_y_pos =
                u16::from_le_bytes([game.ram().read_byte(0xafa), game.ram().read_byte(0xafb)])
        );
        word! { game, 0; samus_y_subpos, samus_prev_y_subpos }
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
            (word!(game.samus_x_pos) as i32 + (a >> 8) as i8 as i32 + (carry >> 8)) as u16;
        if v4 as i32 - 235i32 & 0x8000 != 0 {
            word! {game; samus_x_pos = v4; samus_prev_x_pos = v4; }
            0
        } else {
            word! { game, 235; samus_x_pos, samus_prev_x_pos }
            word! { game, 0; samus_x_subpos, samus_prev_x_subpos }
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
        11110023 => mother_brain_fight::MotherBrainsBody_Init;
        11110149 => mother_brain_fight::MotherBrainsBrain_Init;
        11110206 => mother_brain_fight::MotherBrainsBody_Hurt;
        11110279 => mother_brain_fight::MotherBrainsBody_Powerbomb;
        11110283 => mother_brain_fight::MotherBrainsBrain_Hurt;
        11111221 => sm_game::actors::mother_brain_tubes::initialize;
        11121923 => mother_brain_fight::MotherBrainsBody_Shot;
        11122118 => mother_brain_fight::MotherBrainsBrain_Touch;
        11129608 => corpses::DeadTorizo_Init;
        11129704 => corpses::DeadTorizo_Main;
        11129898 => corpses::DeadTorizo_Powerbomb;
        11129907 => corpses::DeadTorizo_Shot;
        11130806 => corpses::DeadSidehopper_Init;
        11130953 => corpses::DeadZoomer_Init;
        11130998 => corpses::DeadRipper_Init;
        11131039 => corpses::DeadSkree_Init;
        11131084 => corpses::DeadSidehopper_Powerbomb;
        11131099 => corpses::DeadSidehopper_Main;
        11132141 => corpses::DeadZoomer_Powerbomb;
        11132152 => corpses::DeadZoomer_Shot;
        11132157 => corpses::DeadRipper_Powerbomb;
        11132168 => corpses::DeadRipper_Shot;
        11132173 => corpses::DeadSkree_Powerbomb;
        11132184 => corpses::DeadSkree_Shot;
        11132189 => corpses::DeadSidehopper_Shot;
        11132228 => corpses::DeadSidehopper_Touch;
        11126544 => baby_metroid_cutscene::BabyMetroidInCutscene_Init;
        11126649 => baby_metroid_cutscene::BabyMetroidInCutscene_Main;
        11128579 => baby_metroid_cutscene::BabyMetroidInCutscene_Touch;
        11136823 => baby_metroid_encounter::BabyMetroid_Init;
        11136954 => baby_metroid_encounter::BabyMetroid_Powerbomb;
        11136965 => baby_metroid_encounter::BabyMetroid_Main;
        11138953 => baby_metroid_encounter::BabyMetroid_Touch;
        11139138 => baby_metroid_encounter::BabyMetroid_Shot;
    }
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { match crate::game::mother_brain_body::dispatch_instruction(ea, j, game) { Some(script) => script, None => { crate::rtl::Unreachable(game); EnemyScript::Stop } } };
    @external {
        11115208 => mother_brain_fight::MotherBrain_Instr_SpawnEprojToOffset;
        11115247 => mother_brain_fight::MotherBrain_Instr_SpawnDeathBeamEproj;
        11115269 => mother_brain_fight::MotherBrain_Instr_IncrBeamAttackPhase;
        11136208 => corpses::sub_A9ECD0;
        11139476 => baby_metroid_encounter::BabyMetroid_Instr_5;
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
        unsafe { mother_brain_visuals::lights_off(self) };
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
        use crate::game::mother_brain_visuals::Fade as Visual;
        use sm_game::actors::mother_brain_fight::Fade;
        let kind = match kind {
            Fade::ToBlack => Visual::ToBlack,
            Fade::FakeDeathToGray => Visual::FakeDeathToGray,
            Fade::FakeDeathFromGray => Visual::FakeDeathFromGray,
            Fade::DrainedFromGray => Visual::DrainedFromGray,
            Fade::DrainedToGray => Visual::DrainedToGray,
            Fade::RealDeathToGray => Visual::RealDeathToGray,
            Fade::LightsOn => Visual::LightsOn,
        };
        unsafe { crate::game::mother_brain_visuals::fade(kind, frame, self) }
    }

    fn update_health_palette(&mut self) {
        unsafe { crate::game::mother_brain_visuals::update_health_palette(self) };
    }

    fn seal_wall(&mut self) {
        unsafe { crate::game::mother_brain_visuals::seal_wall(self) };
    }

    fn calculate_rainbow_window(&mut self) {
        crate::game::beam_windows::calculate_rainbow_window(self);
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
        corpses::ProcessCorpseRotting(k, self)
    }

    fn process_corpse_rotting_vram_transfers(&mut self, k: u16) {
        corpses::ProcessCorpseRottingVramTransfers(k, self);
    }

    fn start_zebes_escape_typewriter(&mut self) {
        crate::game::escape_typewriter::start_zebes_escape_typewriter(self);
    }

    fn type_escape_text(&mut self, tiles: u16) -> u8 {
        crate::game::escape_typewriter::type_escape_text(tiles, self)
    }

    fn set_boss_bit_for_current_area(&mut self, mask: u16) {
        crate::game::gameplay_flag_runtime::SetBossBitForCurArea(mask, self);
    }

    fn set_event_happened(&mut self, event: u16) {
        crate::game::gameplay_flag_runtime::SetEventHappened(event, self);
    }

    fn check_event_happened(&mut self, event: u16) -> u16 {
        crate::game::gameplay_flag_runtime::CheckEventHappened(event, self)
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

    fn ridley_func_98(&mut self) {
        sm_game::actors::ridley::deal_contact_damage(self);
    }

    fn move_samus_horizontally_towards_wall(&mut self, amount: u16) -> u8 {
        crate::game::sm_a9::move_samus_horizontally_towards_wall(amount, self)
    }

    fn initialize_corpse_rotting(&mut self, k: u16, definition: u16) {
        corpses::InitializeEnemyCorpseRotting(k, definition, self);
    }

    fn queue_sound_result(&mut self, channel: usize, sound: u8, max_queued: u8) -> u16 {
        crate::game::music::queue_sfx(channel, sound, max_queued, self)
    }

    fn move_samus_vertically_towards_ceiling_floor(&mut self, amount: u16) -> u8 {
        crate::game::sm_a9::move_samus_vertically_towards_ceiling_floor(amount, self)
    }
}
