use crate::game::actors;
use crate::game::palette_copies::copy_rom_colors_to_target_palette;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use sm_game::actors::corpses;
use sm_game::actors::mother_brain::{MotherBrainBody, MotherBrainBrain};
use sm_game::actors::{shitroid_cutscene, shitroid_encounter};
use sm_game::enemy_script::EnemyScript;

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;

use crate::game::enemy_projectiles::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::mother_brain_samus_vitals as vitals;
use crate::game::mother_brain_visuals;
use crate::game::room_state::load_fx_entry as LoadFxEntry;
use sm_game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;

use crate::ram_map::slot;

pub use crate::types::Consts_B7;

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct MotherBrainInstrExecState {
        pub ip, timer: u16;
    }
}

crate::type_aliases! {
    pub Enemy_Shitroid, Enemy_DeadTorizo, Enemy_DeadSidehopper =
        crate::game::enemy_layouts::EnemyLayout40;
}

pub type Enemy_DeadMonsters = crate::game::enemy_layouts::EnemyLayout72Late;

pub type Enemy_ShitroidInCutscene = crate::game::enemy_layouts::EnemyLayout72;

pub type Enemy_DeadEnemy = crate::game::enemy_layouts::EnemyLayout40Late;

crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
    fnEproj_Init_0x8bde_SkreeDownLeft = 0x868b15, fnMotherBrainsBody_UnpauseHook = 0xa98763,
    fnMotherBrainsBody_FirstPhase_DoubleRet = 0xa987e1,
}
crate::game::constants::typed_constants! { u32;
    fnMotherBrainBody_FakeDeath_Descent_0_Pause = 11110429,
}
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
    @functions [pub fn (game: &mut Game)];
    [11110023] MotherBrainsBody_Init [] {
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9C13 as u16, game);
        let E: i32 = 0;
        slot!(game, MotherBrainBody[E].base.vram_tiles_index = 0);
        slot!(game, MotherBrainBody[E].base.properties = (slot!(game, MotherBrainBody[E].base.properties) as i32
            | (kEnemyProps_BlockPlasmaBeam as i32
                | kEnemyProps_Tangible as i32
                | kEnemyProps_Invisible as i32)) as u16);
        slot!(game, MotherBrainBody[E].base.palette_index = 0);
        copy_rom_colors_to_target_palette(
            0xa9,
            0x162,
            (addr_kMotherBrainPalette_4 as i32 + 2) as u16,
            0xf,
            game,
        );
        copy_rom_colors_to_target_palette(
            0xa9,
            0x1e2,
            (addr_kMotherBrainPalette_3 as i32 + 2) as u16,
            0xf,
            game,
        );
        slot!(game, MotherBrainBody[E].form = 0);
        let E1: i32 = 0x40;
        slot!(game, MotherBrainBrain[E1].unpause_hook = 0);
        slot!(game, MotherBrainBody[E].hitboxes = 2);
        slot!(game, MotherBrainBrain[E1].function = (0xa987d0i32) as u16);
        slot!(game, MotherBrainBody[E].function = (0xa987e1i32) as u16);
        LoadFxEntry(1, game);
        for v3 in 0u16..12 {
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurrets as u16, v3, game);
        }
    };
    [11110149] MotherBrainsBrain_Init [] {
        corpses::InitializeEnemyCorpseRotting(0x40, addr_stru_A9DE08 as u16, game);
        let E1: i32 = 0x40;
        slot!(game, MotherBrainBrain[E1].base.health = 3000);
        MotherBrain_SetBrainUnusedInstrs(addr_kMotherBrain_Ilist_9C13 as u16, game);
        slot! { game, MotherBrainBrain[E1].base; vram_tiles_index = 0, properties = (slot!(game, MotherBrainBrain[E1].base.properties)) | (0x1100u16), palette_index = 512 };
        let E: i32 = 0;
        slot! { game, MotherBrainBody[E]; neck_palette = 512, brain_palette = 512 }
        MotherBrain_SetBrainInstrs(addr_stru_A99C21 as u16, game);
        MotherBrain_SetupBrainNormalPal(game);
    };
    [11110206] MotherBrainsBody_Hurt [] {
        MotherBrain_Pal_HandleRoomPal(game);
        let E: i32 = 0;
        if slot!(game, MotherBrainBody[E].function) == (fnMotherBrainsBody_FirstPhase_DoubleRet as u16) {
            MotherBrainsBody_FirstPhase_DoubleRet(game);
            return;
        }
        CallMotherBrainFunc(((slot!(game, MotherBrainBody[E].function)) as u32) | (0xa90000u32), game);
        MotherBrain_HandlePalette(game);
        MotherBrain_SamusCollDetect(game);
        MotherBrain_Pal_ProcessInvincibility(game);
        if slot!(game, MotherBrainBrain[0x40i32].unpause_hook) != 0 {
            { let value: u32 = fnMotherBrainsBody_UnpauseHook as u32; { let value: u16 = value as u16; let at: usize = 0x604; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }; game.ram().write_byte(0x604 + 2, (value >> 16) as u8); };
        }
    };
    [11110279] MotherBrainsBody_Powerbomb [] {
        actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
        MotherBrainsBrain_Hurt(game);
    };
    [11110283] MotherBrainsBrain_Hurt [] {
        { let value: u32 = 0xa98786; { let value: u16 = value as u16; let at: usize = 0x178c; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }; game.ram().write_byte(0x178c + 2, (value >> 16) as u8); };
        let v0: i32 = 0;
        if (slot!(game, EnemyData[v0 + core::mem::size_of::<EnemyData>() as i32].properties) & 0x100) != 0 {
            CallMotherBrainFunc(((slot!(game, EnemyData[v0 + core::mem::size_of::<EnemyData>() as i32].ai_var_A)) as u32) | (0xa90000u32), game);
        }
    };
    [11111221] MotherBrainsTubesFalling_Init [] { sm_game::actors::mother_brain_tubes::initialize(game); };
    [11121923] MotherBrainsBody_Shot [] {
        actors::combat::create_dud(&mut Runtime::new(game));
    };
    [11122118] MotherBrainsBrain_Touch [] {
        if game.ram().read_byte(0xa1f) == 3 {
            let mut v0: i16 = 13;
            let v1: i32 = 0;
            let flash_timer: u16 = slot!(game, EnemyData[v1 + core::mem::size_of::<EnemyData>() as i32].flash_timer);
            if flash_timer != 0 && flash_timer & 1 != 0 {
                v0 = 14;
            }
            slot!(game, EnemyData[v1 + core::mem::size_of::<EnemyData>() as i32].flash_timer = v0 as u16);
        }
    };
    @forward [pub fn (game: &mut Game)];
    [11108367] Enemy_GrappleReact_CancelBeam_A9 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11108417] Enemy_NormalFrozenAI_A9 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
    @external {
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
        11126544 => shitroid_cutscene::ShitroidInCutscene_Init;
        11126649 => shitroid_cutscene::ShitroidInCutscene_Main;
        11128579 => shitroid_cutscene::ShitroidInCutscene_Touch;
        11136823 => shitroid_encounter::Shitroid_Init;
        11136954 => shitroid_encounter::Shitroid_Powerbomb;
        11136965 => shitroid_encounter::Shitroid_Main;
        11138953 => shitroid_encounter::Shitroid_Touch;
        11139138 => shitroid_encounter::Shitroid_Shot;
    }
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { match crate::game::mother_brain_body::dispatch_instruction(ea, j, game) { Some(script) => script, None => { crate::rtl::Unreachable(game); EnemyScript::Stop } } };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11115208] MotherBrain_Instr_SpawnEprojToOffset [jp.offset(3)] {
        let E: i32 = 0;
        game.eproj_spawn_pt = Point16U {
            x: ((slot!(game, MotherBrainBody[E].base.x_pos)).wrapping_add(jp.word(0))),
            y: ((slot!(game, MotherBrainBody[E].base.y_pos)).wrapping_add(jp.word(2))),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, jp.word(4), game);
    };
    [11115247] MotherBrain_Instr_SpawnDeathBeamEproj [jp] {
        let spawn_parameter = crate::game::music::queue_sfx(1, 0x63, 6, game);
        SpawnEprojWithGfx(
            spawn_parameter,
            0x40,
            addr_kEproj_MotherBrainDeathBeamCharging as u16,
            game,
        );
    };
    [11115269] MotherBrain_Instr_IncrBeamAttackPhase [jp] {
        let E: i32 = 0;
        slot!(game, MotherBrainBody[E].hand_beam_phase = slot!(game, MotherBrainBody[E].hand_beam_phase).wrapping_add(1));
    };
    @external {
        11136208 => corpses::sub_A9ECD0;
        11139476 => shitroid_encounter::Shitroid_Instr_5;
    }
}

impl shitroid_encounter::Host for Game {
    fn drain_health_by_shitroid(&mut self) {
        vitals::drain_health_by_baby_metroid(self);
    }

    fn homing_probe(&mut self) {
        crate::verify_probe::hit("homing.metroid");
    }
}

impl shitroid_cutscene::Host for Game {
    fn lights_off(&mut self) {
        unsafe { mother_brain_visuals::lights_off(self) };
    }

    fn heal_by_shitroid(&mut self) -> bool {
        vitals::heal_by_baby_metroid(self)
    }
}

impl corpses::Host for Game {
    fn dead_torizo_contact(&mut self) -> bool {
        actors::part_hitboxes::dead_torizo_contact(self)
    }
}

impl sm_game::actors::mother_brain_fight::Host for Game {
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
        unsafe { crate::game::beam_windows::calculate_rainbow_window(self) };
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
        crate::game::sm_a6::Ridley_Func_98(self);
    }

    fn move_samus_horizontally_towards_wall(&mut self, amount: u16) -> u8 {
        crate::game::sm_a9::move_samus_horizontally_towards_wall(amount, self)
    }

    fn move_samus_vertically_towards_ceiling_floor(&mut self, amount: u16) -> u8 {
        crate::game::sm_a9::move_samus_vertically_towards_ceiling_floor(amount, self)
    }
}
