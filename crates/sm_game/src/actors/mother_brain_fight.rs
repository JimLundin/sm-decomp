//! Mother Brain's fight: the first phase behind the glass, the fake death
//! and its tube collapse, the second phase's walking, bombs, laser and
//! rainbow beam, the baby Metroid cutscene, the third phase and the death and
//! escape sequence, with the brain's instruction list and palettes.
//!
//! Was `MotherBrain*`, `MotherBrainBody*`, `MotherBrainsBody*` and `MotherBomb*`
//! in the original decompilation (bank $A9). Slot words are named through
//! `mother_brain`'s typed body and brain views. The bank's dispatch ids are
//! unchanged; the functions and their reads and writes keep the original
//! order. What still lives in the compatibility runtime is reached through
//! [`Host`], with the original names kept on the forwarding helpers at the end
//! of this module.
#![forbid(unsafe_code)]
#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unused_parens
)]
#![allow(unused_mut, unused_assignments, clippy::all)]
use crate::actors::host::abs16;
use crate::actors::mother_brain::{BRAIN, MotherBrainBody, MotherBrainBrain};
use crate::actors::mother_brain_effects::{
    generate_explosions as generate_body_explosions,
    handle_rainbow_palette as handle_body_rainbow_palette,
    write_palette as write_mother_brain_palette,
};
use crate::actors::mother_brain_neck::function as neck;
use crate::actors::session_slots::{SlotAccess, session_slot};
use crate::constants::bank_8d::*;
use crate::constants::bank_86::*;
use crate::constants::bank_a6::*;
use crate::constants::bank_a9::*;
use crate::constants::enemy_properties::*;
use crate::enemy_projectiles::shared_pre::Math_MultByCos;
use crate::enemy_script::EnemyScript;
use crate::menus::menu_tilemap::native_record;
use crate::menus::prelude::{EnemyData, do_until};
use crate::mother_brain_tables;
use crate::palette_copies::{copy_rom_colors_to_palette, copy_rom_colors_to_target_palette};
use crate::ram::RawRam as _;
use crate::ram::word;
use crate::word_transfers;

/// A bank-local address, as the original's `VoidP`.
pub(crate) type VoidP = u16;
type CoroutineRet = u8;
type Enemy_DeadMonsters = crate::actors::layouts::EnemyLayout72Late;
const kCoroutineNone: i32 = 0;

/// The bank $A9 functions Mother Brain's body and brain run, named after the
/// bank log's headings.
pub mod function {
    /// $A9:87A2: Mother Brain brain function - set up brain and neck to be drawn.
    pub const BRAIN_SET_UP_BRAIN_AND_NECK_TO_BE_DRAWN: u16 = 0x87a2;
    /// $A9:87D0: Mother Brain brain function - set up brain to be drawn.
    pub const BRAIN_SET_UP_BRAIN_TO_BE_DRAWN: u16 = 0x87d0;
    /// $A9:87E1: Mother Brain body function - first phase.
    pub const FIRST_PHASE: u16 = 0x87e1;
    /// $A9:8829: Mother Brain body function - fake death - descent - lock Samus and set scroll region.
    pub const FAKE_DEATH_DESCENT_LOCK_SAMUS_AND_SET_SCROLL_REGION: u16 = 0x8829;
    /// $A9:884D: Mother Brain body function - fake death - descent - queue Mother Brain music.
    pub const FAKE_DEATH_DESCENT_QUEUE_MOTHER_BRAIN_MUSIC: u16 = 0x884d;
    /// $A9:886C: Mother Brain body function - fake death - descent - unlock Samus.
    pub const FAKE_DEATH_DESCENT_UNLOCK_SAMUS: u16 = 0x886c;
    /// $A9:8884: Mother Brain body function - fake death - descent - begin screen flashing and lower acid.
    pub const FAKE_DEATH_DESCENT_BEGIN_SCREEN_FLASHING_AND_LOWER_ACID: u16 = 0x8884;
    /// $A9:88B2: Mother Brain body function - fake death - descent - transition Mother Brain palette to grey.
    pub const FAKE_DEATH_DESCENT_TRANSITION_MOTHER_BRAIN_PALETTE_TO_GREY: u16 = 0x88b2;
    /// $A9:88D3: Mother Brain body function - fake death - descent - collapse tubes.
    pub const FAKE_DEATH_DESCENT_COLLAPSE_TUBES: u16 = 0x88d3;
    /// $A9:8949: Mother brain body subfunction - wait until there are less than 4 enemy projectiles alive and spawn tubes falling enemy 0.
    pub const TUBES_WAIT_AND_SPAWN_TUBES_FALLING_ENEMY_0: u16 = 0x8949;
    /// $A9:896E: Mother brain body subfunction - clear bottom-left tube.
    pub const TUBES_CLEAR_BOTTOM_LEFT_TUBE: u16 = 0x896e;
    /// $A9:8983: Mother brain body subfunction - spawn top-right tube falling enemy projectile.
    pub const TUBES_SPAWN_TOP_RIGHT_TUBE_FALLING_PROJECTILE: u16 = 0x8983;
    /// $A9:89B5: Mother brain body subfunction - spawn top-left tube falling enemy projectile.
    pub const TUBES_SPAWN_TOP_LEFT_TUBE_FALLING_PROJECTILE: u16 = 0x89b5;
    /// $A9:89D2: Mother brain body subfunction - clear ceiling block column 6.
    pub const TUBES_CLEAR_CEILING_BLOCK_COLUMN_6: u16 = 0x89d2;
    /// $A9:89E7: Mother brain body subfunction - spawn tubes falling enemy 1.
    pub const TUBES_SPAWN_TUBES_FALLING_ENEMY_1: u16 = 0x89e7;
    /// $A9:89FA: Mother brain body subfunction - clear bottom-right tube.
    pub const TUBES_CLEAR_BOTTOM_RIGHT_TUBE: u16 = 0x89fa;
    /// $A9:8A0F: Mother brain body subfunction - spawn tubes falling enemy 2.
    pub const TUBES_SPAWN_TUBES_FALLING_ENEMY_2: u16 = 0x8a0f;
    /// $A9:8A22: Mother brain body subfunction - clear bottom-middle-left tube.
    pub const TUBES_CLEAR_BOTTOM_MIDDLE_LEFT_TUBE: u16 = 0x8a22;
    /// $A9:8A37: Mother brain body subfunction - spawn top-middle-left tube falling enemy projectile.
    pub const TUBES_SPAWN_TOP_MIDDLE_LEFT_TUBE_FALLING_PROJECTILE: u16 = 0x8a37;
    /// $A9:8A54: Mother brain body subfunction - clear ceiling tube column 7.
    pub const TUBES_CLEAR_CEILING_TUBE_COLUMN_7: u16 = 0x8a54;
    /// $A9:8A69: Mother brain body subfunction - spawn top-middle-right tube falling enemy projectile.
    pub const TUBES_SPAWN_TOP_MIDDLE_RIGHT_TUBE_FALLING_PROJECTILE: u16 = 0x8a69;
    /// $A9:8A86: Mother brain body subfunction - clear ceiling tube column 8.
    pub const TUBES_CLEAR_CEILING_TUBE_COLUMN_8: u16 = 0x8a86;
    /// $A9:8A9B: Mother brain body subfunction - spawn tubes falling enemy 3.
    pub const TUBES_SPAWN_TUBES_FALLING_ENEMY_3: u16 = 0x8a9b;
    /// $A9:8AAE: Mother brain body subfunction - clear bottom-middle-right tube.
    pub const TUBES_CLEAR_BOTTOM_MIDDLE_RIGHT_TUBE: u16 = 0x8aae;
    /// $A9:8AC3: Mother brain body subfunction - spawn tubes falling enemy 4.
    pub const TUBES_SPAWN_TUBES_FALLING_ENEMY_4: u16 = 0x8ac3;
    /// $A9:8AD6: Mother brain body subfunction - clear bottom-middle tubes.
    pub const TUBES_CLEAR_BOTTOM_MIDDLE_TUBES: u16 = 0x8ad6;
    /// $A9:8C9E: Mother Brain body function - fake death - ascent - draw room background on BG1 - rows 4/5.
    pub const FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_4_5: u16 = 0x8c9e;
    /// $A9:8CB5: Mother Brain body function - fake death - ascent - draw room background on BG1 - rows 6/7.
    pub const FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_6_7: u16 = 0x8cb5;
    /// $A9:8CCC: Mother Brain body function - fake death - ascent - draw room background on BG1 - rows 8/9.
    pub const FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_8_9: u16 = 0x8ccc;
    /// $A9:8CE3: Mother Brain body function - fake death - ascent - draw room background on BG1 - rows Ah/Bh.
    pub const FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_A_B: u16 = 0x8ce3;
    /// $A9:8CFA: Mother Brain body function - fake death - ascent - draw room background on BG1 - rows Ch/Dh.
    pub const FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_C_D: u16 = 0x8cfa;
    /// $A9:8D11: Mother Brain body function - fake death - ascent - set up Mother Brain phase 2 graphics.
    pub const FAKE_DEATH_ASCENT_SET_UP_MOTHER_BRAIN_PHASE_2_GRAPHICS: u16 = 0x8d11;
    /// $A9:8D49: Mother Brain body function - fake death - ascent - set up Mother Brain phase 2 brain.
    pub const FAKE_DEATH_ASCENT_SET_UP_MOTHER_BRAIN_PHASE_2_BRAIN: u16 = 0x8d49;
    /// $A9:8D79: Mother Brain body function - fake death - ascent - pause for suspense.
    pub const FAKE_DEATH_ASCENT_PAUSE_FOR_SUSPENSE: u16 = 0x8d79;
    /// $A9:8D8B: Mother Brain body function - fake death - ascent - prepare Mother Brain for rising.
    pub const FAKE_DEATH_ASCENT_PREPARE_MOTHER_BRAIN_FOR_RISING: u16 = 0x8d8b;
    /// $A9:8DB4: Mother Brain body function - fake death - ascent - load Mother Brain's legs tiles.
    pub const FAKE_DEATH_ASCENT_LOAD_MOTHER_BRAINS_LEGS_TILES: u16 = 0x8db4;
    /// $A9:8DC3: Mother Brain body function - fake death - ascent - continue pausing for suspense.
    pub const FAKE_DEATH_ASCENT_CONTINUE_PAUSING_FOR_SUSPENSE: u16 = 0x8dc3;
    /// $A9:8DEC: Mother Brain body function - fake death - ascent - start music and earthquake.
    pub const FAKE_DEATH_ASCENT_START_MUSIC_AND_EARTHQUAKE: u16 = 0x8dec;
    /// $A9:8E4D: Mother Brain body function - fake death - ascent - raise Mother Brain.
    pub const FAKE_DEATH_ASCENT_RAISE_MOTHER_BRAIN: u16 = 0x8e4d;
    /// $A9:8E95: Mother Brain body function - fake death - ascent - wait until Mother Brain has finished uncrouching.
    pub const FAKE_DEATH_ASCENT_WAIT_UNTIL_MOTHER_BRAIN_HAS_FINISHED_UNCROUCHING: u16 = 0x8e95;
    /// $A9:8EAA: Mother Brain body function - fake death - ascent - transition Mother Brain from grey and lower her head.
    pub const FAKE_DEATH_ASCENT_TRANSITION_MOTHER_BRAIN_FROM_GREY_AND_LOWER_HER_HEAD: u16 = 0x8eaa;
    /// $A9:8EF5: Mother Brain body function - second phase - stretching - shake head menacingly.
    pub const PHASE_2_STRETCHING_SHAKE_HEAD_MENACINGLY: u16 = 0x8ef5;
    /// $A9:8F14: Mother Brain body function - second phase - stretching - bring head back up.
    pub const PHASE_2_STRETCHING_BRING_HEAD_BACK_UP: u16 = 0x8f14;
    /// $A9:8F33: Mother Brain body function - second phase - stretching - finish stretching.
    pub const PHASE_2_STRETCHING_FINISH_STRETCHING: u16 = 0x8f33;
    /// $A9:AEE1: Mother Brain body function - third phase - death sequence - move to back of room.
    pub const PHASE_3_DEATH_MOVE_TO_BACK_OF_ROOM: u16 = 0xaee1;
    /// $A9:AF12: Mother Brain body function - third phase - death sequence - idle whilst exploding.
    pub const PHASE_3_DEATH_IDLE_WHILST_EXPLODING: u16 = 0xaf12;
    /// $A9:AF21: Mother Brain body function - third phase - death sequence - stumble to middle of room and drool.
    pub const PHASE_3_DEATH_STUMBLE_TO_MIDDLE_OF_ROOM_AND_DROOL: u16 = 0xaf21;
    /// $A9:AF54: Mother Brain body function - third phase - death sequence - disable brain effects.
    pub const PHASE_3_DEATH_DISABLE_BRAIN_EFFECTS: u16 = 0xaf54;
    /// $A9:AF9D: Mother Brain body function - third phase - death sequence - set up body fade out.
    pub const PHASE_3_DEATH_SET_UP_BODY_FADE_OUT: u16 = 0xaf9d;
    /// $A9:AFB6: Mother Brain body function - third phase - death sequence - fade out body.
    pub const PHASE_3_DEATH_FADE_OUT_BODY: u16 = 0xafb6;
    /// $A9:B013: Mother Brain body function - third phase - death sequence - final few explosions.
    pub const PHASE_3_DEATH_FINAL_FEW_EXPLOSIONS: u16 = 0xb013;
    /// $A9:B115: Mother Brain body function - third phase - death sequence - realise decapitation.
    pub const PHASE_3_DEATH_REALISE_DECAPITATION: u16 = 0xb115;
    /// $A9:B12D: Mother Brain body function - third phase - death sequence - brain falls to ground.
    pub const PHASE_3_DEATH_BRAIN_FALLS_TO_GROUND: u16 = 0xb12d;
    /// $A9:B15E: Mother Brain body function - third phase - death sequence - load corpse tiles.
    pub const PHASE_3_DEATH_LOAD_CORPSE_TILES: u16 = 0xb15e;
    /// $A9:B173: Mother Brain body function - third phase - death sequence - set up fade to grey.
    pub const PHASE_3_DEATH_SET_UP_FADE_TO_GREY: u16 = 0xb173;
    /// $A9:B189: Mother Brain body function - third phase - death sequence - fade to grey.
    pub const PHASE_3_DEATH_FADE_TO_GREY: u16 = 0xb189;
    /// $A9:B1B8: Mother Brain body function - third phase - death sequence - corpse tips over.
    pub const PHASE_3_DEATH_CORPSE_TIPS_OVER: u16 = 0xb1b8;
    /// $A9:B1D5: Mother Brain body function - third phase - death sequence - corpse rots away.
    pub const PHASE_3_DEATH_CORPSE_ROTS_AWAY: u16 = 0xb1d5;
    /// $A9:B211: Mother Brain body function - third phase - death sequence - 20 frame delay.
    pub const PHASE_3_DEATH_20_FRAME_DELAY: u16 = 0xb211;
    /// $A9:B258: Mother Brain body function - third phase - death sequence - load escape timer tiles.
    pub const PHASE_3_DEATH_LOAD_ESCAPE_TIMER_TILES: u16 = 0xb258;
    /// $A9:B26D: Mother Brain body function - third phase - death sequence - start escape sequence.
    pub const PHASE_3_DEATH_START_ESCAPE_SEQUENCE: u16 = 0xb26d;
    /// $A9:B2D1: Mother Brain body function - third phase - death sequence - spawn time bomb set Japanese text.
    pub const PHASE_3_DEATH_SPAWN_TIME_BOMB_SET_JAPANESE_TEXT: u16 = 0xb2d1;
    /// $A9:B2E3: Mother Brain body function - third phase - death sequence - type out Zebes escape text.
    pub const PHASE_3_DEATH_TYPE_OUT_ZEBES_ESCAPE_TEXT: u16 = 0xb2e3;
    /// $A9:B2F9: Mother Brain body function - third phase - death sequence - escape door is exploding, start escape timer.
    pub const PHASE_3_DEATH_ESCAPE_DOOR_IS_EXPLODING_START_ESCAPE_TIMER: u16 = 0xb2f9;
    /// $A9:B32A: Mother Brain body function - third phase - death sequence - blow up escape door.
    pub const PHASE_3_DEATH_BLOW_UP_ESCAPE_DOOR: u16 = 0xb32a;
    /// $A9:B33C: Mother Brain body function - third phase - death sequence - keep earthquake going.
    pub const PHASE_3_DEATH_KEEP_EARTHQUAKE_GOING: u16 = 0xb33c;
    /// $A9:B605: Mother Brain body function - second phase - thinking.
    pub const PHASE_2_THINKING: u16 = 0xb605;
    /// $A9:B64B: Mother Brain body function - second phase - try attack.
    pub const PHASE_2_TRY_ATTACK: u16 = 0xb64b;
    /// $A9:B781: Mother Brain body function - firing bomb - decide on walking.
    pub const FIRING_BOMB_DECIDE_ON_WALKING: u16 = 0xb781;
    /// $A9:B7AC: Mother Brain body function - firing bomb - walking backwards.
    pub const FIRING_BOMB_WALKING_BACKWARDS: u16 = 0xb7ac;
    /// $A9:B7C6: Mother Brain body function - firing bomb - crouch.
    pub const FIRING_BOMB_CROUCH: u16 = 0xb7c6;
    /// $A9:B7E8: Mother Brain body function - firing bomb - fired bomb.
    pub const FIRING_BOMB_FIRED_BOMB: u16 = 0xb7e8;
    /// $A9:B7F8: Mother Brain body function - firing bomb - stand up.
    pub const FIRING_BOMB_STAND_UP: u16 = 0xb7f8;
    /// $A9:B80E: Mother Brain body function - second phase - firing laser - position head quickly.
    pub const PHASE_2_FIRING_LASER_POSITION_HEAD_QUICKLY: u16 = 0xb80e;
    /// $A9:B839: Mother Brain body function - second phase - firing laser - position head slowly and fire.
    pub const PHASE_2_FIRING_LASER_POSITION_HEAD_SLOWLY_AND_FIRE: u16 = 0xb839;
    /// $A9:B863: Mother Brain body function - second phase - firing laser - finish attack.
    pub const PHASE_2_FIRING_LASER_FINISH_ATTACK: u16 = 0xb863;
    /// $A9:B87D: Mother Brain body function - second phase - firing hand beam.
    pub const PHASE_2_FIRING_HAND_BEAM: u16 = 0xb87d;
    /// $A9:B8EB: Mother Brain body function - second phase - firing rainbow beam - extend neck.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_EXTEND_NECK: u16 = 0xb8eb;
    /// $A9:B91A: Mother Brain body function - second phase - firing rainbow beam - start charging rainbow beam.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_START_CHARGING_RAINBOW_BEAM: u16 = 0xb91a;
    /// $A9:B92B: Mother Brain body function - second phase - firing rainbow beam - retract neck.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_RETRACT_NECK: u16 = 0xb92b;
    /// $A9:B93F: Mother Brain body function - second phase - firing rainbow beam - wait for beam to charge.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_WAIT_FOR_BEAM_TO_CHARGE: u16 = 0xb93f;
    /// $A9:B951: Mother Brain body function - second phase - firing rainbow beam - extend neck down.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_EXTEND_NECK_DOWN: u16 = 0xb951;
    /// $A9:B975: Mother Brain body function - second phase - firing rainbow beam - start firing rainbow beam.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_START_FIRING_RAINBOW_BEAM: u16 = 0xb975;
    /// $A9:B9E5: Mother Brain body function - second phase - firing rainbow beam - move Samus towards wall.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_MOVE_SAMUS_TOWARDS_WALL: u16 = 0xb9e5;
    /// $A9:BA00: Mother Brain body function - second phase - firing rainbow beam - 1 frame delay.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_1_FRAME_DELAY: u16 = 0xba00;
    /// $A9:BA27: Mother Brain body function - second phase - firing rainbow beam - start draining Samus.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_START_DRAINING_SAMUS: u16 = 0xba27;
    /// $A9:BA3C: Mother Brain body function - second phase - firing rainbow beam - draining Samus.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_DRAINING_SAMUS: u16 = 0xba3c;
    /// $A9:BA5E: Mother Brain body function - second phase - firing rainbow beam - finish firing rainbow beam.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_FINISH_FIRING_RAINBOW_BEAM: u16 = 0xba5e;
    /// $A9:BAC4: Mother Brain body function - second phase - firing rainbow beam - let Samus fall.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_LET_SAMUS_FALL: u16 = 0xbac4;
    /// $A9:BAD1: Mother Brain body function - second phase - firing rainbow beam - wait for Samus to hit ground.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_WAIT_FOR_SAMUS_TO_HIT_GROUND: u16 = 0xbad1;
    /// $A9:BADD: Mother Brain body function - second phase - firing rainbow beam - lower head.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_LOWER_HEAD: u16 = 0xbadd;
    /// $A9:BB06: Mother Brain body function - second phase - firing rainbow beam - decide next action.
    pub const PHASE_2_FIRING_RAINBOW_BEAM_DECIDE_NEXT_ACTION: u16 = 0xbb06;
    /// $A9:BD45: Mother Brain body function - second phase - finish Samus off - get Samus to low health.
    pub const PHASE_2_FINISH_SAMUS_OFF_GET_SAMUS_TO_LOW_HEALTH: u16 = 0xbd45;
    /// $A9:BD98: Mother Brain body function - second phase - finish Samus off - stand up.
    pub const PHASE_2_FINISH_SAMUS_OFF_STAND_UP: u16 = 0xbd98;
    /// $A9:BDA9: Mother Brain body function - second phase - finish Samus off - admire job well done.
    pub const PHASE_2_FINISH_SAMUS_OFF_ADMIRE_JOB_WELL_DONE: u16 = 0xbda9;
    /// $A9:BDC1: Mother Brain body function - second phase - finish Samus off - charge final rainbow beam.
    pub const PHASE_2_FINISH_SAMUS_OFF_CHARGE_FINAL_RAINBOW_BEAM: u16 = 0xbdc1;
    /// $A9:BDD2: Mother Brain body function - second phase - finish Samus off - charge final rainbow beam / load Shitroid.
    pub const PHASE_2_FINISH_SAMUS_OFF_CHARGE_FINAL_RAINBOW_BEAM_LOAD_BABY_METROID: u16 = 0xbdd2;
    /// $A9:BDED: Mother Brain body function - second phase - finish Samus off - fire final rainbow beam.
    pub const PHASE_2_FINISH_SAMUS_OFF_FIRE_FINAL_RAINBOW_BEAM: u16 = 0xbded;
    /// Nothing: the baby Metroid takes over the cutscene.
    pub const PHASE_2_CUTSCENE_WAIT: u16 = 0xbe1a;
    /// $A9:BE5D: Mother Brain body function - drained by Shitroid - regain balance.
    pub const DRAINED_BY_BABY_METROID_REGAIN_BALANCE: u16 = 0xbe5d;
    /// $A9:BE96: Mother Brain body function - drained by Shitroid - firing rainbow beam.
    pub const DRAINED_BY_BABY_METROID_FIRING_RAINBOW_BEAM: u16 = 0xbe96;
    /// $A9:BF0E: Mother Brain body function - drained by Shitroid - rainbow beam has run out.
    pub const DRAINED_BY_BABY_METROID_RAINBOW_BEAM_HAS_RUN_OUT: u16 = 0xbf0e;
    /// $A9:BF41: Mother Brain body function - drained by Shitroid - move to back of room.
    pub const DRAINED_BY_BABY_METROID_MOVE_TO_BACK_OF_ROOM: u16 = 0xbf41;
    /// $A9:BF56: Mother Brain body function - drained by Shitroid - go into low power mode.
    pub const DRAINED_BY_BABY_METROID_GO_INTO_LOW_POWER_MODE: u16 = 0xbf56;
    /// $A9:BF7D: Mother Brain body function - drained by Shitroid - prepare transition to grey.
    pub const DRAINED_BY_BABY_METROID_PREPARE_TRANSITION_TO_GREY: u16 = 0xbf7d;
    /// $A9:BF95: Mother Brain body function - drained by Shitroid - transition to grey.
    pub const DRAINED_BY_BABY_METROID_TRANSITION_TO_GREY: u16 = 0xbf95;
    /// $A9:BFD0: Mother Brain painful walking function - walk forwards.
    pub const PAINFUL_WALK_FORWARDS: u16 = 0xbfd0;
    /// $A9:BFE8: Mother Brain painful walking function - walking forwards.
    pub const PAINFUL_WALKING_FORWARDS: u16 = 0xbfe8;
    /// $A9:C004: Mother Brain painful walking function - walk backwards.
    pub const PAINFUL_WALK_BACKWARDS: u16 = 0xc004;
    /// $A9:C01C: Mother Brain painful walking function - walking backwards.
    pub const PAINFUL_WALKING_BACKWARDS: u16 = 0xc01c;
    /// $A9:C059: Mother Brain body function - second phase - revive self - inanimate grey.
    pub const PHASE_2_REVIVE_SELF_INANIMATE_GREY: u16 = 0xc059;
    /// $A9:C066: Mother Brain body function - second phase - revive self - show signs of life.
    pub const PHASE_2_REVIVE_SELF_SHOW_SIGNS_OF_LIFE: u16 = 0xc066;
    /// $A9:C082: Mother Brain body function - second phase - revive self - prepare transition from grey.
    pub const PHASE_2_REVIVE_SELF_PREPARE_TRANSITION_FROM_GREY: u16 = 0xc082;
    /// $A9:C08F: Mother Brain body function - second phase - revive self - transition from grey.
    pub const PHASE_2_REVIVE_SELF_TRANSITION_FROM_GREY: u16 = 0xc08f;
    /// $A9:C0BA: Mother Brain body function - second phase - revive self - wake up.
    pub const PHASE_2_REVIVE_SELF_WAKE_UP: u16 = 0xc0ba;
    /// $A9:C0E4: Mother Brain body function - second phase - revive self - wake-up stretch.
    pub const PHASE_2_REVIVE_SELF_WAKE_UP_STRETCH: u16 = 0xc0e4;
    /// $A9:C0FB: Mother Brain body function - second phase - revive self - walk up to Shitroid.
    pub const PHASE_2_REVIVE_SELF_WALK_UP_TO_BABY_METROID: u16 = 0xc0fb;
    /// $A9:C11E: Mother Brain body function - second phase - revive self - prepare neck for Shitroid murder.
    pub const PHASE_2_REVIVE_SELF_PREPARE_NECK_FOR_BABY_METROID_MURDER: u16 = 0xc11e;
    /// $A9:C147: Mother Brain body function - second phase - revive self - finish preparing for Shitroid murder.
    pub const PHASE_2_REVIVE_SELF_FINISH_PREPARING_FOR_BABY_METROID_MURDER: u16 = 0xc147;
    /// $A9:C15C: Mother Brain body function - second phase - murder Shitroid - attack Shitroid.
    pub const PHASE_2_MURDER_BABY_METROID_ATTACK_BABY_METROID: u16 = 0xc15c;
    /// $A9:C182: Mother Brain body function - second phase - murder Shitroid - Shitroid attack cooldown.
    pub const PHASE_2_MURDER_BABY_METROID_BABY_METROID_ATTACK_COOLDOWN: u16 = 0xc182;
    /// Nothing: the baby Metroid's last attack takes over.
    pub const FINAL_BABY_METROID_WAIT: u16 = 0xc1a6;
    /// $A9:C1F0: Mother Brain body function - third phase - recover from cutscene - set up for fighting.
    pub const PHASE_3_RECOVER_FROM_CUTSCENE_SET_UP_FOR_FIGHTING: u16 = 0xc1f0;
    /// $A9:C209: Mother Brain body function - third phase - fighting - main.
    pub const PHASE_3_FIGHTING_MAIN: u16 = 0xc209;
    /// $A9:C24E: Mother Brain body function - third phase - fighting - attack cooldown.
    pub const PHASE_3_FIGHTING_ATTACK_COOLDOWN: u16 = 0xc24e;
    /// $A9:C26A: Mother Brain walking function - try to inch forward.
    pub const WALK_TRY_TO_INCH_FORWARD: u16 = 0xc26a;
    /// $A9:C2B3: Mother Brain walking function - retreat quickly.
    pub const WALK_RETREAT_QUICKLY: u16 = 0xc2b3;
    /// $A9:C2D2: Mother Brain walking function - retreat slowly.
    pub const WALK_RETREAT_SLOWLY: u16 = 0xc2d2;
}

// Each named function is the one its dispatch entry runs.
const _: () = {
    use function as f;
    const fn at(word: u16) -> u32 {
        0xa9_0000 | word as u32
    }
    assert!(at(f::BRAIN_SET_UP_BRAIN_AND_NECK_TO_BE_DRAWN) == 11110306);
    assert!(at(f::BRAIN_SET_UP_BRAIN_TO_BE_DRAWN) == 11110352);
    assert!(at(f::FIRST_PHASE) == 11110369);
    assert!(at(f::FAKE_DEATH_DESCENT_LOCK_SAMUS_AND_SET_SCROLL_REGION) == 11110441);
    assert!(at(f::FAKE_DEATH_DESCENT_QUEUE_MOTHER_BRAIN_MUSIC) == 11110477);
    assert!(at(f::FAKE_DEATH_DESCENT_UNLOCK_SAMUS) == 11110508);
    assert!(at(f::FAKE_DEATH_DESCENT_BEGIN_SCREEN_FLASHING_AND_LOWER_ACID) == 11110532);
    assert!(at(f::FAKE_DEATH_DESCENT_TRANSITION_MOTHER_BRAIN_PALETTE_TO_GREY) == 11110578);
    assert!(at(f::FAKE_DEATH_DESCENT_COLLAPSE_TUBES) == 11110611);
    assert!(at(f::TUBES_WAIT_AND_SPAWN_TUBES_FALLING_ENEMY_0) == 11110729);
    assert!(at(f::TUBES_CLEAR_BOTTOM_LEFT_TUBE) == 11110766);
    assert!(at(f::TUBES_SPAWN_TOP_RIGHT_TUBE_FALLING_PROJECTILE) == 11110787);
    assert!(at(f::TUBES_SPAWN_TOP_LEFT_TUBE_FALLING_PROJECTILE) == 11110837);
    assert!(at(f::TUBES_CLEAR_CEILING_BLOCK_COLUMN_6) == 11110866);
    assert!(at(f::TUBES_SPAWN_TUBES_FALLING_ENEMY_1) == 11110887);
    assert!(at(f::TUBES_CLEAR_BOTTOM_RIGHT_TUBE) == 11110906);
    assert!(at(f::TUBES_SPAWN_TUBES_FALLING_ENEMY_2) == 11110927);
    assert!(at(f::TUBES_CLEAR_BOTTOM_MIDDLE_LEFT_TUBE) == 11110946);
    assert!(at(f::TUBES_SPAWN_TOP_MIDDLE_LEFT_TUBE_FALLING_PROJECTILE) == 11110967);
    assert!(at(f::TUBES_CLEAR_CEILING_TUBE_COLUMN_7) == 11110996);
    assert!(at(f::TUBES_SPAWN_TOP_MIDDLE_RIGHT_TUBE_FALLING_PROJECTILE) == 11111017);
    assert!(at(f::TUBES_CLEAR_CEILING_TUBE_COLUMN_8) == 11111046);
    assert!(at(f::TUBES_SPAWN_TUBES_FALLING_ENEMY_3) == 11111067);
    assert!(at(f::TUBES_CLEAR_BOTTOM_MIDDLE_RIGHT_TUBE) == 11111086);
    assert!(at(f::TUBES_SPAWN_TUBES_FALLING_ENEMY_4) == 11111107);
    assert!(at(f::TUBES_CLEAR_BOTTOM_MIDDLE_TUBES) == 11111126);
    assert!(at(f::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_4_5) == 11111582);
    assert!(at(f::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_6_7) == 11111605);
    assert!(at(f::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_8_9) == 11111628);
    assert!(at(f::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_A_B) == 11111651);
    assert!(at(f::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_C_D) == 11111674);
    assert!(at(f::FAKE_DEATH_ASCENT_SET_UP_MOTHER_BRAIN_PHASE_2_GRAPHICS) == 11111697);
    assert!(at(f::FAKE_DEATH_ASCENT_SET_UP_MOTHER_BRAIN_PHASE_2_BRAIN) == 11111753);
    assert!(at(f::FAKE_DEATH_ASCENT_PAUSE_FOR_SUSPENSE) == 11111801);
    assert!(at(f::FAKE_DEATH_ASCENT_PREPARE_MOTHER_BRAIN_FOR_RISING) == 11111819);
    assert!(at(f::FAKE_DEATH_ASCENT_LOAD_MOTHER_BRAINS_LEGS_TILES) == 11111860);
    assert!(at(f::FAKE_DEATH_ASCENT_CONTINUE_PAUSING_FOR_SUSPENSE) == 11111875);
    assert!(at(f::FAKE_DEATH_ASCENT_START_MUSIC_AND_EARTHQUAKE) == 11111916);
    assert!(at(f::FAKE_DEATH_ASCENT_RAISE_MOTHER_BRAIN) == 11112013);
    assert!(at(f::FAKE_DEATH_ASCENT_WAIT_UNTIL_MOTHER_BRAIN_HAS_FINISHED_UNCROUCHING) == 11112085);
    assert!(
        at(f::FAKE_DEATH_ASCENT_TRANSITION_MOTHER_BRAIN_FROM_GREY_AND_LOWER_HER_HEAD) == 11112106
    );
    assert!(at(f::PHASE_2_STRETCHING_SHAKE_HEAD_MENACINGLY) == 11112181);
    assert!(at(f::PHASE_2_STRETCHING_BRING_HEAD_BACK_UP) == 11112212);
    assert!(at(f::PHASE_2_STRETCHING_FINISH_STRETCHING) == 11112243);
    assert!(at(f::PHASE_3_DEATH_MOVE_TO_BACK_OF_ROOM) == 11120353);
    assert!(at(f::PHASE_3_DEATH_IDLE_WHILST_EXPLODING) == 11120402);
    assert!(at(f::PHASE_3_DEATH_STUMBLE_TO_MIDDLE_OF_ROOM_AND_DROOL) == 11120417);
    assert!(at(f::PHASE_3_DEATH_DISABLE_BRAIN_EFFECTS) == 11120468);
    assert!(at(f::PHASE_3_DEATH_SET_UP_BODY_FADE_OUT) == 11120541);
    assert!(at(f::PHASE_3_DEATH_FADE_OUT_BODY) == 11120566);
    assert!(at(f::PHASE_3_DEATH_FINAL_FEW_EXPLOSIONS) == 11120659);
    assert!(at(f::PHASE_3_DEATH_REALISE_DECAPITATION) == 11120917);
    assert!(at(f::PHASE_3_DEATH_BRAIN_FALLS_TO_GROUND) == 11120941);
    assert!(at(f::PHASE_3_DEATH_LOAD_CORPSE_TILES) == 11120990);
    assert!(at(f::PHASE_3_DEATH_SET_UP_FADE_TO_GREY) == 11121011);
    assert!(at(f::PHASE_3_DEATH_FADE_TO_GREY) == 11121033);
    assert!(at(f::PHASE_3_DEATH_CORPSE_TIPS_OVER) == 11121080);
    assert!(at(f::PHASE_3_DEATH_CORPSE_ROTS_AWAY) == 11121109);
    assert!(at(f::PHASE_3_DEATH_20_FRAME_DELAY) == 11121169);
    assert!(at(f::PHASE_3_DEATH_LOAD_ESCAPE_TIMER_TILES) == 11121240);
    assert!(at(f::PHASE_3_DEATH_START_ESCAPE_SEQUENCE) == 11121261);
    assert!(at(f::PHASE_3_DEATH_SPAWN_TIME_BOMB_SET_JAPANESE_TEXT) == 11121361);
    assert!(at(f::PHASE_3_DEATH_TYPE_OUT_ZEBES_ESCAPE_TEXT) == 11121379);
    assert!(at(f::PHASE_3_DEATH_ESCAPE_DOOR_IS_EXPLODING_START_ESCAPE_TIMER) == 11121401);
    assert!(at(f::PHASE_3_DEATH_BLOW_UP_ESCAPE_DOOR) == 11121450);
    assert!(at(f::PHASE_3_DEATH_KEEP_EARTHQUAKE_GOING) == 11121468);
    assert!(at(f::PHASE_2_THINKING) == 11122181);
    assert!(at(f::PHASE_2_TRY_ATTACK) == 11122251);
    assert!(at(f::FIRING_BOMB_DECIDE_ON_WALKING) == 11122561);
    assert!(at(f::FIRING_BOMB_WALKING_BACKWARDS) == 11122604);
    assert!(at(f::FIRING_BOMB_CROUCH) == 11122630);
    assert!(at(f::FIRING_BOMB_FIRED_BOMB) == 11122664);
    assert!(at(f::FIRING_BOMB_STAND_UP) == 11122680);
    assert!(at(f::PHASE_2_FIRING_LASER_POSITION_HEAD_QUICKLY) == 11122702);
    assert!(at(f::PHASE_2_FIRING_LASER_POSITION_HEAD_SLOWLY_AND_FIRE) == 11122745);
    assert!(at(f::PHASE_2_FIRING_LASER_FINISH_ATTACK) == 11122787);
    assert!(at(f::PHASE_2_FIRING_HAND_BEAM) == 11122813);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_EXTEND_NECK) == 11122923);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_START_CHARGING_RAINBOW_BEAM) == 11122970);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_RETRACT_NECK) == 11122987);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_WAIT_FOR_BEAM_TO_CHARGE) == 11123007);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_EXTEND_NECK_DOWN) == 11123025);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_START_FIRING_RAINBOW_BEAM) == 11123061);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_MOVE_SAMUS_TOWARDS_WALL) == 11123173);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_1_FRAME_DELAY) == 11123200);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_START_DRAINING_SAMUS) == 11123239);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_DRAINING_SAMUS) == 11123260);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_FINISH_FIRING_RAINBOW_BEAM) == 11123294);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_LET_SAMUS_FALL) == 11123396);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_WAIT_FOR_SAMUS_TO_HIT_GROUND) == 11123409);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_LOWER_HEAD) == 11123421);
    assert!(at(f::PHASE_2_FIRING_RAINBOW_BEAM_DECIDE_NEXT_ACTION) == 11123462);
    assert!(at(f::PHASE_2_FINISH_SAMUS_OFF_GET_SAMUS_TO_LOW_HEALTH) == 11124037);
    assert!(at(f::PHASE_2_FINISH_SAMUS_OFF_STAND_UP) == 11124120);
    assert!(at(f::PHASE_2_FINISH_SAMUS_OFF_ADMIRE_JOB_WELL_DONE) == 11124137);
    assert!(at(f::PHASE_2_FINISH_SAMUS_OFF_CHARGE_FINAL_RAINBOW_BEAM) == 11124161);
    assert!(
        at(f::PHASE_2_FINISH_SAMUS_OFF_CHARGE_FINAL_RAINBOW_BEAM_LOAD_BABY_METROID) == 11124178
    );
    assert!(at(f::PHASE_2_FINISH_SAMUS_OFF_FIRE_FINAL_RAINBOW_BEAM) == 11124205);
    assert!(at(f::PHASE_2_CUTSCENE_WAIT) == 11124250);
    assert!(at(f::DRAINED_BY_BABY_METROID_REGAIN_BALANCE) == 11124317);
    assert!(at(f::DRAINED_BY_BABY_METROID_FIRING_RAINBOW_BEAM) == 11124374);
    assert!(at(f::DRAINED_BY_BABY_METROID_RAINBOW_BEAM_HAS_RUN_OUT) == 11124494);
    assert!(at(f::DRAINED_BY_BABY_METROID_MOVE_TO_BACK_OF_ROOM) == 11124545);
    assert!(at(f::DRAINED_BY_BABY_METROID_GO_INTO_LOW_POWER_MODE) == 11124566);
    assert!(at(f::DRAINED_BY_BABY_METROID_PREPARE_TRANSITION_TO_GREY) == 11124605);
    assert!(at(f::DRAINED_BY_BABY_METROID_TRANSITION_TO_GREY) == 11124629);
    assert!(at(f::PAINFUL_WALK_FORWARDS) == 11124688);
    assert!(at(f::PAINFUL_WALKING_FORWARDS) == 11124712);
    assert!(at(f::PAINFUL_WALK_BACKWARDS) == 11124740);
    assert!(at(f::PAINFUL_WALKING_BACKWARDS) == 11124764);
    assert!(at(f::PHASE_2_REVIVE_SELF_INANIMATE_GREY) == 11124825);
    assert!(at(f::PHASE_2_REVIVE_SELF_SHOW_SIGNS_OF_LIFE) == 11124838);
    assert!(at(f::PHASE_2_REVIVE_SELF_PREPARE_TRANSITION_FROM_GREY) == 11124866);
    assert!(at(f::PHASE_2_REVIVE_SELF_TRANSITION_FROM_GREY) == 11124879);
    assert!(at(f::PHASE_2_REVIVE_SELF_WAKE_UP) == 11124922);
    assert!(at(f::PHASE_2_REVIVE_SELF_WAKE_UP_STRETCH) == 11124964);
    assert!(at(f::PHASE_2_REVIVE_SELF_WALK_UP_TO_BABY_METROID) == 11124987);
    assert!(at(f::PHASE_2_REVIVE_SELF_PREPARE_NECK_FOR_BABY_METROID_MURDER) == 11125022);
    assert!(at(f::PHASE_2_REVIVE_SELF_FINISH_PREPARING_FOR_BABY_METROID_MURDER) == 11125063);
    assert!(at(f::PHASE_2_MURDER_BABY_METROID_ATTACK_BABY_METROID) == 11125084);
    assert!(at(f::PHASE_2_MURDER_BABY_METROID_BABY_METROID_ATTACK_COOLDOWN) == 11125122);
    assert!(at(f::FINAL_BABY_METROID_WAIT) == 11125158);
    assert!(at(f::PHASE_3_RECOVER_FROM_CUTSCENE_SET_UP_FOR_FIGHTING) == 11125232);
    assert!(at(f::PHASE_3_FIGHTING_MAIN) == 11125257);
    assert!(at(f::PHASE_3_FIGHTING_ATTACK_COOLDOWN) == 11125326);
    assert!(at(f::WALK_TRY_TO_INCH_FORWARD) == 11125354);
    assert!(at(f::WALK_RETREAT_QUICKLY) == 11125427);
    assert!(at(f::WALK_RETREAT_SLOWLY) == 11125458);
};

/// Bank $A9 instruction lists for the body and brain.
pub mod animation {
    /// Body: walking forwards, medium.
    pub const BODY_WALK_FORWARDS_MEDIUM: u16 = 0x97a4;
    /// Body: walking backwards, fast.
    pub const BODY_WALK_BACKWARDS_FAST: u16 = 0x98c6;
    /// Body: standing up after crouching, slow.
    pub const BODY_STAND_UP_FROM_CROUCH_SLOW: u16 = 0x99aa;
    /// Body: standing up after crouching, fast.
    pub const BODY_STAND_UP_FROM_CROUCH_FAST: u16 = 0x99c6;
    /// Body: standing up after leaning down.
    pub const BODY_STAND_UP_FROM_LEANING_DOWN: u16 = 0x99e2;
    /// Body: leaning down.
    pub const BODY_LEAN_DOWN: u16 = 0x99f2;
    /// Body: crouched.
    pub const BODY_CROUCHED: u16 = 0x9a02;
    /// Body: crouch, slow.
    pub const BODY_CROUCH_SLOW: u16 = 0x9a0a;
    /// Body: crouch, fast.
    pub const BODY_CROUCH_FAST: u16 = 0x9a26;
    /// Body: hand beam mode.
    pub const BODY_HAND_BEAM_MODE: u16 = 0x9a42;
    /// Brain: stretching, phase 2.
    pub const BRAIN_STRETCHING_PHASE_2: u16 = 0x9b7f;
    /// Brain: stretching, phase 3.
    pub const BRAIN_STRETCHING_PHASE_3: u16 = 0x9bb3;
    /// Brain: decapitated.
    pub const BRAIN_DECAPITATED: u16 = 0x9c29;
    /// Brain: dying drool.
    pub const BRAIN_DYING_DROOL: u16 = 0x9c39;
    /// Brain: firing the rainbow beam.
    pub const BRAIN_FIRING_RAINBOW_BEAM: u16 = 0x9c77;
    /// Brain: neutral, phase 2.
    pub const BRAIN_NEUTRAL_PHASE_2: u16 = 0x9c87;
    /// Brain: corpse.
    pub const BRAIN_CORPSE: u16 = 0x9d25;
    /// Brain: attacking with four blue rings, phase 2.
    pub const BRAIN_FOUR_BLUE_RINGS_PHASE_2: u16 = 0x9d3d;
    /// Brain: attacking with two blue rings, phase 2.
    pub const BRAIN_TWO_BLUE_RINGS_PHASE_2: u16 = 0x9d7f;
    /// Brain: attacking the baby Metroid.
    pub const BRAIN_ATTACKING_BABY_METROID: u16 = 0x9db1;
    /// Brain: attacking Samus with four blue rings, phase 3.
    pub const BRAIN_FOUR_BLUE_RINGS_PHASE_3: u16 = 0x9dbb;
    /// Brain: attacking with a bomb, phase 2.
    pub const BRAIN_BOMB_PHASE_2: u16 = 0x9ecc;
    /// Brain: attacking with a bomb, phase 3.
    pub const BRAIN_BOMB_PHASE_3: u16 = 0x9f00;
    /// Brain: attacking with the laser.
    pub const BRAIN_LASER: u16 = 0x9f34;
    /// Brain: charging the rainbow beam.
    pub const BRAIN_CHARGING_RAINBOW_BEAM: u16 = 0x9f6c;
}

// Each named instruction list is the bank's.
const _: () = {
    use animation as a;
    assert!(a::BODY_WALK_FORWARDS_MEDIUM == addr_kMotherBrain_Ilist_97A4 as u16);
    assert!(a::BODY_WALK_BACKWARDS_FAST == addr_kMotherBrain_Ilist_98C6 as u16);
    assert!(a::BODY_STAND_UP_FROM_CROUCH_SLOW == addr_kMotherBrain_Ilist_99AA as u16);
    assert!(a::BODY_STAND_UP_FROM_CROUCH_FAST == addr_kMotherBrain_Ilist_99C6 as u16);
    assert!(a::BODY_STAND_UP_FROM_LEANING_DOWN == addr_kMotherBrain_Ilist_99E2 as u16);
    assert!(a::BODY_LEAN_DOWN == addr_kMotherBrain_Ilist_99F2 as u16);
    assert!(a::BODY_CROUCHED == addr_kMotherBrain_Ilist_9A02 as u16);
    assert!(a::BODY_CROUCH_SLOW == addr_kMotherBrain_Ilist_9A0A as u16);
    assert!(a::BODY_CROUCH_FAST == addr_kMotherBrain_Ilist_9A26 as u16);
    assert!(a::BODY_HAND_BEAM_MODE == addr_kMotherBrain_Ilist_9A42 as u16);
    assert!(a::BRAIN_STRETCHING_PHASE_2 == addr_kMotherBrain_Ilist_9B7F as u16);
    assert!(a::BRAIN_STRETCHING_PHASE_3 == addr_kMotherBrain_Ilist_9BB3 as u16);
    assert!(a::BRAIN_DECAPITATED == addr_kMotherBrain_Ilist_9C29 as u16);
    assert!(a::BRAIN_DYING_DROOL == addr_kMotherBrain_Ilist_9C39 as u16);
    assert!(a::BRAIN_FIRING_RAINBOW_BEAM == addr_kMotherBrain_Ilist_9C77 as u16);
    assert!(a::BRAIN_NEUTRAL_PHASE_2 == addr_kMotherBrain_Ilist_9C87 as u16);
    assert!(a::BRAIN_CORPSE == addr_kMotherBrain_Ilist_9D25 as u16);
    assert!(a::BRAIN_FOUR_BLUE_RINGS_PHASE_2 == addr_kMotherBrain_Ilist_9D3D as u16);
    assert!(a::BRAIN_TWO_BLUE_RINGS_PHASE_2 == addr_kMotherBrain_Ilist_9D7F as u16);
    assert!(a::BRAIN_ATTACKING_BABY_METROID == addr_kMotherBrain_Ilist_9DB1 as u16);
    assert!(a::BRAIN_FOUR_BLUE_RINGS_PHASE_3 == addr_kMotherBrain_Ilist_9DBB as u16);
    assert!(a::BRAIN_BOMB_PHASE_2 == addr_kMotherBrain_Ilist_9ECC as u16);
    assert!(a::BRAIN_BOMB_PHASE_3 == addr_kMotherBrain_Ilist_9F00 as u16);
    assert!(a::BRAIN_LASER == addr_kMotherBrain_Ilist_9F34 as u16);
    assert!(a::BRAIN_CHARGING_RAINBOW_BEAM == addr_kMotherBrain_Ilist_9F6C as u16);
};

/// Bank $84 PLMs that redraw Mother Brain's room.
mod plm {
    /// Mother Brain's room escape door.
    pub const ESCAPE_DOOR: u16 = 0xb677;
    /// Mother Brain's background row 2.
    pub const BACKGROUND_ROW_2: u16 = 0xb67b;
    /// Mother Brain's background row 3.
    pub const BACKGROUND_ROW_3: u16 = 0xb67f;
    /// Mother Brain's background row 4.
    pub const BACKGROUND_ROW_4: u16 = 0xb683;
    /// Mother Brain's background row 5.
    pub const BACKGROUND_ROW_5: u16 = 0xb687;
    /// Mother Brain's background row 6.
    pub const BACKGROUND_ROW_6: u16 = 0xb68b;
    /// Mother Brain's background row 7.
    pub const BACKGROUND_ROW_7: u16 = 0xb68f;
    /// Mother Brain's background row 8.
    pub const BACKGROUND_ROW_8: u16 = 0xb693;
    /// Mother Brain's background row 9.
    pub const BACKGROUND_ROW_9: u16 = 0xb697;
    /// Mother Brain's background row Ah.
    pub const BACKGROUND_ROW_A: u16 = 0xb69b;
    /// Mother Brain's background row Bh.
    pub const BACKGROUND_ROW_B: u16 = 0xb69f;
    /// Mother Brain's background row Ch.
    pub const BACKGROUND_ROW_C: u16 = 0xb6a3;
    /// Mother Brain's background row Dh.
    pub const BACKGROUND_ROW_D: u16 = 0xb6a7;
    /// Clear a ceiling block in Mother Brain's room.
    pub const CLEAR_CEILING_BLOCK: u16 = 0xb6b3;
    /// Clear a ceiling tube in Mother Brain's room.
    pub const CLEAR_CEILING_TUBE: u16 = 0xb6b7;
    /// Clear Mother Brain's bottom-middle-side tube.
    pub const CLEAR_BOTTOM_MIDDLE_SIDE_TUBE: u16 = 0xb6bb;
    /// Clear Mother Brain's bottom-middle tubes.
    pub const CLEAR_BOTTOM_MIDDLE_TUBES: u16 = 0xb6bf;
    /// Clear Mother Brain's bottom-left tube.
    pub const CLEAR_BOTTOM_LEFT_TUBE: u16 = 0xb6c3;
    /// Clear Mother Brain's bottom-right tube.
    pub const CLEAR_BOTTOM_RIGHT_TUBE: u16 = 0xb6c7;
}

/// $A9:B439 and $A9:B44B: the brain's and a neck segment's hitbox definitions.
const BRAIN_HITBOX: u16 = 0xb439;
const NECK_HITBOX: u16 = 0xb44b;
const _: () =
    assert!(BRAIN_HITBOX == addr_word_A9B427 as u16 + 0x12 && NECK_HITBOX == BRAIN_HITBOX + 0x12);

/// $A9:87C9 and $A9:87DD: the graphics drawn hooks that draw the brain
/// with and without the neck.
pub const DRAW_BRAIN_AND_NECK_HOOK: u32 = 0xa9_87c9;
pub const DRAW_BRAIN_HOOK: u32 = 0xa9_87dd;
// They are the ids `actor_species_draw_hook` dispatches on.
const _: () = assert!(DRAW_BRAIN_AND_NECK_HOOK == 11110345 && DRAW_BRAIN_HOOK == 11110365);
const GRAPHICS_DRAWN_HOOK: usize = 0x178c;

/// Set the enemy graphics drawn hook, low byte first and the bank last.
pub(crate) fn set_graphics_drawn_hook(hook: u32, game: &mut (impl State + ?Sized)) {
    write_long_pointer(GRAPHICS_DRAWN_HOOK, hook, game);
}

/// Store a long pointer at `at`, low byte first and the bank last.
fn write_long_pointer(at: usize, pointer: u32, game: &mut (impl State + ?Sized)) {
    let bytes = pointer.to_le_bytes();
    game.compatibility_write_byte(at, bytes[0]);
    game.compatibility_write_byte(at + 1, bytes[1]);
    game.compatibility_write_byte(at + 2, bytes[2]);
}

/// $A9:8786: an RTL, the graphics drawn hook that draws nothing.
const NO_DRAW_HOOK: u32 = 0xa9_8786;
const _: () = assert!(NO_DRAW_HOOK == 11110278);
/// $7E:0604: the unpause hook; $A9:8763: Mother Brain body's, which
/// `gameplay_frame` dispatches on.
const UNPAUSE_HOOK: usize = 0x604;
const UNPAUSE_BODY_HOOK: u32 = 0xa9_8763;
const _: () = assert!(UNPAUSE_BODY_HOOK == 11110243);

/// Mother Brain's palette fades, by colour table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fade {
    ToBlack,
    FakeDeathToGray,
    FakeDeathFromGray,
    DrainedFromGray,
    DrainedToGray,
    RealDeathToGray,
    LightsOn,
}

/// How Samus meets one of Mother Brain's part hitboxes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contact {
    None,
    Pushed,
    Hurt,
}

/// What Mother Brain's state steps read and write: slot words (the Session's
/// inside a lease, the RAM image's on the game), named RAM words and the ROM
/// records. Routines written against it run on either.
pub trait State: SlotAccess + crate::ram::CompatibilityRam + crate::records::Records {}
impl<T: SlotAccess + crate::ram::CompatibilityRam + crate::records::Records + ?Sized> State for T {}

/// A lease on the game for one frame of an AI: the state steps run on it, and
/// `foreign` runs the rest on the game it leases.
pub trait Lease: State {
    type Game: Host + ?Sized;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
}

/// What the fight still reaches in the compatibility runtime.
pub trait Host:
    crate::actors::host::BossEffects
    + crate::actors::mother_brain_effects::Host
    + crate::actors::mother_brain_neck::Host
    + crate::actors::mother_brain_tubes::Host
    + crate::records::Records
    + crate::cartridge::NativeImage
    + SlotAccess
{
    /// The original's unreachable-state failure. Was `Unreachable`.
    fn bank_unreachable(&mut self) -> bool;
    /// Spawn a population from bank `bank`, in one actor lease.
    fn spawn_legacy_population(&mut self, bank: u8, population: u16) -> u16;
    /// The spawn's extra parameter. Was `eproj_spawn_r22`.
    fn set_projectile_spawn_r22(&mut self, value: u16);
    /// A dud shot reaction, in one actor lease.
    fn create_dud(&mut self);
    /// Step a palette fade; nonzero once done. Was `mother_brain_visuals::fade`.
    fn fade(&mut self, kind: Fade, frame: u16) -> u8;
    fn update_health_palette(&mut self);
    fn seal_wall(&mut self);
    fn calculate_rainbow_window(&mut self);
    fn flicker_body(&mut self);
    fn drain_health_by_rainbow_beam(&mut self);
    fn drain_ammo_by_rainbow_beam(&mut self);
    fn add_glass_hits(&mut self, slot: usize, amount: u16);
    fn spawn_palette_effect(&mut self, definition: u16);
    /// Queue a sprite tile transfer list; nonzero once done.
    fn queue_mother_brain_sprite_tiles(&mut self, bank: u8, start: u16) -> u8;
    fn process_corpse_rotting(&mut self, k: u16) -> u8;
    fn process_corpse_rotting_vram_transfers(&mut self, k: u16);
    fn start_zebes_escape_typewriter(&mut self);
    fn type_escape_text(&mut self, tiles: u16) -> u8;
    fn set_boss_bit_for_current_area(&mut self, mask: u16);
    fn set_event_happened(&mut self, event: u16);
    fn check_event_happened(&mut self, event: u16) -> u16;
    fn spawn_rainbow_hdma(&mut self) -> u16;
    fn spawn_rising_hdma(&mut self) -> u16;
    fn player_mother_brain(&mut self, command: u16);
    fn load_fx_entry(&mut self, index: u16);
    fn native_word(&mut self, offset: usize) -> u16;
    fn set_native_word(&mut self, offset: usize, value: u16);
    fn samus_contact(&mut self, boxes: u16, x: u16, y: u16) -> Contact;
    /// Ridley's shared helper. Was `Ridley_Func_98`.
    fn ridley_func_98(&mut self);
    fn move_samus_horizontally_towards_wall(&mut self, amount: u16) -> u8;
    fn move_samus_vertically_towards_ceiling_floor(&mut self, amount: u16) -> u8;
    /// Start the corpse rotting of slot `k` from `definition`. Was
    /// `InitializeEnemyCorpseRotting`.
    fn initialize_corpse_rotting(&mut self, k: u16, definition: u16);
    /// Queue a sound; the value the original left in A. Was `queue_sfx`.
    fn queue_sound_result(&mut self, channel: usize, sound: u8, max_queued: u8) -> u16;
    /// The lease an AI's frame runs its state steps in.
    type Lease<'a>: Lease<Game = Self>
    where
        Self: 'a;
    fn lease(&mut self) -> Self::Lease<'_>;
}

// Forwarding helpers under the original names, so the moved bodies are unchanged.
pub(crate) fn Unreachable(game: &mut impl Host) -> bool {
    game.bank_unreachable()
}
pub(crate) fn SpawnEprojWithRoomGfx(definition: u16, parameter: u16, game: &mut impl Host) -> u16 {
    game.spawn_room_projectile(definition, parameter)
}
#[derive(Clone, Copy)]
pub(crate) struct SpawnHardcodedPlmArgs {
    block_x: u8,
    block_y: u8,
    header: VoidP,
}
impl SpawnHardcodedPlmArgs {
    pub(crate) fn new(block_x: u8, block_y: u8, header: VoidP) -> Self {
        Self {
            block_x,
            block_y,
            header,
        }
    }
}
pub(crate) fn spawn_hardcoded(args: SpawnHardcodedPlmArgs, game: &mut impl Host) {
    game.spawn_plm(args.block_x, args.block_y, args.header)
}
pub(crate) fn QueueMusic_Delayed8(track: u16, game: &mut impl Host) {
    game.queue_music_delayed(track)
}
pub(crate) fn QueueSfx1_Max6(sound: u16, game: &mut impl Host) {
    game.queue_sound(0, sound as u8, 6)
}
pub(crate) fn QueueSfx2_Max3(sound: u16, game: &mut impl Host) {
    game.queue_sound(1, sound as u8, 3)
}
pub(crate) fn QueueSfx2_Max6(sound: u16, game: &mut impl Host) {
    game.queue_sound(1, sound as u8, 6)
}
pub(crate) fn QueueSfx3_Max3(sound: u16, game: &mut impl Host) {
    game.queue_sound(2, sound as u8, 3)
}
pub(crate) fn QueueSfx3_Max6(sound: u16, game: &mut impl Host) {
    game.queue_sound(2, sound as u8, 6)
}
pub(crate) fn CallSomeSamusCode(command: u16, game: &mut impl Host) -> u16 {
    game.command_player(command)
}
pub(crate) fn native_word(offset: usize, game: &mut impl Host) -> u16 {
    game.native_word(offset)
}
pub(crate) fn set_native_word(offset: usize, value: u16, game: &mut impl Host) {
    game.set_native_word(offset, value)
}
pub(crate) fn queue_mother_brain_sprite_tiles(bank: u8, start: u16, game: &mut impl Host) -> u8 {
    game.queue_mother_brain_sprite_tiles(bank, start)
}
pub(crate) fn ProcessCorpseRotting(k: u16, game: &mut impl Host) -> u8 {
    game.process_corpse_rotting(k)
}
pub(crate) fn ProcessCorpseRottingVramTransfers(k: u16, game: &mut impl Host) {
    game.process_corpse_rotting_vram_transfers(k)
}
pub(crate) fn start_zebes_escape_typewriter(game: &mut impl Host) {
    game.start_zebes_escape_typewriter()
}
pub(crate) fn type_escape_text(tiles: u16, game: &mut impl Host) -> u8 {
    game.type_escape_text(tiles)
}
pub(crate) fn SetBossBitForCurArea(mask: u16, game: &mut impl Host) {
    game.set_boss_bit_for_current_area(mask)
}
pub(crate) fn SetEventHappened(event: u16, game: &mut impl Host) {
    game.set_event_happened(event)
}
pub(crate) fn CheckEventHappened(event: u16, game: &mut impl Host) -> u16 {
    game.check_event_happened(event)
}
pub(crate) fn spawn_rainbow_hdma(game: &mut impl Host) -> u16 {
    game.spawn_rainbow_hdma()
}
pub(crate) fn spawn_rising_hdma(game: &mut impl Host) -> u16 {
    game.spawn_rising_hdma()
}
pub(crate) fn player_mother_brain(command: u16, game: &mut impl Host) {
    game.player_mother_brain(command)
}
pub(crate) fn LoadFxEntry(index: u16, game: &mut impl Host) {
    game.load_fx_entry(index)
}
pub(crate) fn samus_contact(boxes: u16, x: u16, y: u16, game: &mut impl Host) -> Contact {
    game.samus_contact(boxes, x, y)
}
pub(crate) fn Ridley_Func_98(game: &mut impl Host) {
    game.ridley_func_98()
}
pub(crate) fn move_samus_horizontally_towards_wall(amount: u16, game: &mut impl Host) -> u8 {
    game.move_samus_horizontally_towards_wall(amount)
}
pub(crate) fn move_samus_vertically_towards_ceiling_floor(amount: u16, game: &mut impl Host) -> u8 {
    game.move_samus_vertically_towards_ceiling_floor(amount)
}
pub(crate) fn add_glass_hits(slot: usize, amount: u16, game: &mut impl Host) {
    game.add_glass_hits(slot, amount)
}
pub(crate) fn spawn_palette_effect(definition: u16, game: &mut impl Host) {
    game.spawn_palette_effect(definition)
}

pub static g_byte_A98F7D: [u8; 2] = [9, 0x12];
pub static g_word_A98F7F: [u16; 8] = [0x3d, 0x54, 0x20, 0x35, 0x5a, 0x43, 0x67, 0x29];
pub static g_word_A993BB: [i16; 4] = [0, -1, 0, 1];
pub static g_word_A993C3: [i16; 4] = [0, 1, -1, 1];
pub static g_word_A99E0F: [u16; 13] = [
    0x6f, 0x6f, 0x6f, 0x7e, 0x6f, 0x6f, 0x7e, 0x6f, 0x6f, 0x7e, 0x7e, 0x6f, 0x6f,
];
pub static g_word_A9B393: [u16; 8] = [8, 0x6c, 0x18, 0x80, 9, 0x90, 0x18, 0x74];
pub static g_byte_A9B546: [u8; 8] = [0, 1, 1, 0, 0, 0, 0, 0];
pub static g_byte_A9B5A1: [u8; 8] = [2, 1, 1, 0, 0, 0, 0, 0];
pub static g_word_A9B72C: [i16; 28] = [
    -1, -17, 0, 0, -1, -1, 0, -1, 0, 0, -1, 0, 0, 0, -17, -1, -1, -1, 0, 0, 0, -17, -1, -1, -1, -1,
    -1, -1,
];
pub static g_word_A9BCA6: [i16; 8] = [-8, 6, -4, 2, 3, -6, 8, 0];
pub static g_word_A9BCB6: [i16; 8] = [-7, 2, 5, -4, 6, -2, -6, 7];
pub static g_byte_A9BEEE: [u8; 16] = [2, 0, 2, 0, 6, 0, 6, 0, 8, 0, 8, 0, 10, 0, 10, 0];
pub static g_word_A9BEFE: [u16; 8] = [0x500, 0x500, 0x200, 0x200, 0xc0, 0xc0, 0x40, 0x40];
pub static g_word_A9C049: [i16; 8] = [0x10, 0x10, 0x20, 0x20, 0x30, 0x30, 0x40, 0x40];

pub fn MotherBrain_CorpseRottingFinished(game: &mut impl Host) {
    let E1: i32 = 0x40;
    crate::actors::host::BossEffects::set_projectile_spawn_point(
        game,
        (session_slot!(game, MotherBrainBrain[E1].base.x_pos) as i32
            + (word!(game.random_number) as i32 & 0x1f)
            - 16i32) as u16,
        ((session_slot!(game, MotherBrainBrain[E1].base.y_pos)).wrapping_add(16)),
    );
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xa, game);
    if (word!(game.random_enemy_counter) & 7) == 0 {
        QueueSfx2_Max3(0x10, game);
    }
}

crate::dispatch::callback_dispatch! {
    pub fn CallMotherBrainFunc(ea: u32, game: &mut impl Host) => ea;
    call (game); default { Unreachable(game); };
    @body checked;
    @functions [pub fn (game: &mut impl Host)];
    [11110306] MotherBrainsBrain_SetupBrainAndNeckToDraw [] {
        if word!(game.time_is_frozen_flag) == 0 {
            MotherBrain_HandleNeck(game);
        }
        let E1: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E1].base.x_pos = session_slot!(game, MotherBrainBrain[E1].segment_5_x));
        session_slot!(game, MotherBrainBrain[E1].base.y_pos = (session_slot!(game, MotherBrainBrain[E1].segment_5_y)).wrapping_sub(21));
        set_graphics_drawn_hook(DRAW_BRAIN_AND_NECK_HOOK, game);
    };
    [11110429] MotherBrainBody_FakeDeath_Descent_0_Pause [] {
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_DESCENT_LOCK_SAMUS_AND_SET_SCROLL_REGION);
        session_slot!(game, MotherBrainBody[E].function_timer = 64);
        MotherBrainBody_FakeDeath_Descent_1(game);
    };
    [11110611] MotherBrainBody_FakeDeath_Descent_6 [] {
        let E: i32 = 0x40;
        CallMotherBrainFunc(((session_slot!(game, MotherBrainBrain[E].subfunction)) as u32) | (0xa90000u32), game);
        MotherBrain_HandleFakeDeathExplosions(game);
    };
    [11110729] MotherBrainBody_0_Wait [] {
        let mut v1: i16 = 0;
        for v0 in (0u16..36).step_by(2) {
            if { let at: usize = 0x1997 + (v0 as i32 as isize ) as usize; u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) }
                == 0
            {
                v1 += 1;
            }
        }
        if (v1).wrapping_sub(4) >= 0 {
            game.spawn_legacy_population(0xa9, addr_stru_A98AE5 as u16);
            session_slot!(game, MotherBrainBrain[0x40i32].subfunction = function::TUBES_CLEAR_BOTTOM_LEFT_TUBE);
        }
    };
    [11110766] MotherBrainBody_1_ClearBottomLeftTube [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x5, 0x9, plm::CLEAR_BOTTOM_LEFT_TUBE), game);
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_SPAWN_TOP_RIGHT_TUBE_FALLING_PROJECTILE);
        session_slot!(game, MotherBrainBrain[E].subfunction_timer = 32);
    };
    [11110816] MotherBrainBody_3_ClearCeilingBlock9 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x9, 0x2, plm::CLEAR_CEILING_BLOCK), game);
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_SPAWN_TOP_LEFT_TUBE_FALLING_PROJECTILE);
        session_slot!(game, MotherBrainBrain[E].subfunction_timer = 32);
    };
    [11110866] MotherBrainBody_4_ClearCeilingBlock6 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x6, 0x2, plm::CLEAR_CEILING_BLOCK), game);
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_SPAWN_TUBES_FALLING_ENEMY_1);
        session_slot!(game, MotherBrainBrain[E].subfunction_timer = 32);
    };
    [11110906] MotherBrainBody_6_ClearBottomRightTube [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xa, 0x9, plm::CLEAR_BOTTOM_RIGHT_TUBE), game);
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_SPAWN_TUBES_FALLING_ENEMY_2);
        session_slot!(game, MotherBrainBrain[E].subfunction_timer = 32);
    };
    [11110946] MotherBrainBody_8_ClearBottomMiddleLeftTube [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x6, 0xa, plm::CLEAR_BOTTOM_MIDDLE_SIDE_TUBE), game);
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_SPAWN_TOP_MIDDLE_LEFT_TUBE_FALLING_PROJECTILE);
        session_slot!(game, MotherBrainBrain[E].subfunction_timer = 32);
    };
    [11110996] MotherBrainBody_10_ClearCeilingTubeColumn7 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x7, 0x2, plm::CLEAR_CEILING_TUBE), game);
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_SPAWN_TOP_MIDDLE_RIGHT_TUBE_FALLING_PROJECTILE);
        session_slot!(game, MotherBrainBrain[E].subfunction_timer = 32);
    };
    [11111046] MotherBrainBody_12_ClearCeilingTubeColumn8 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x8, 0x2, plm::CLEAR_CEILING_TUBE), game);
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_SPAWN_TUBES_FALLING_ENEMY_3);
        session_slot!(game, MotherBrainBrain[E].subfunction_timer = 32);
    };
    [11111086] MotherBrainBody_14_ClearBottomMiddleRightTube [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x9, 0xa, plm::CLEAR_BOTTOM_MIDDLE_SIDE_TUBE), game);
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_SPAWN_TUBES_FALLING_ENEMY_4);
        session_slot!(game, MotherBrainBrain[E].subfunction_timer = 2);
    };
    [11111126] MotherBrainBody_16_ClearBottomMiddleTubes [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x7, 0x7, plm::CLEAR_BOTTOM_MIDDLE_TUBES), game);
        session_slot!(game, MotherBrainBrain[0x40i32].subfunction = addr_locret_A98AE4 as u16);
    };
    [11111559] MotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x2, plm::BACKGROUND_ROW_2), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x3, plm::BACKGROUND_ROW_3), game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_4_5);
    };
    [11111582] MotherBrainBody_FakeDeath_Ascent_1_DrawBG1Row45 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x4, plm::BACKGROUND_ROW_4), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x5, plm::BACKGROUND_ROW_5), game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_6_7);
    };
    [11111605] MotherBrainBody_FakeDeath_Ascent_2_DrawBG1Row67 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x6, plm::BACKGROUND_ROW_6), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x7, plm::BACKGROUND_ROW_7), game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_8_9);
    };
    [11111628] MotherBrainBody_FakeDeath_Ascent_3_DrawBG1Row89 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x8, plm::BACKGROUND_ROW_8), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x9, plm::BACKGROUND_ROW_9), game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_A_B);
    };
    [11111651] MotherBrainBody_FakeDeath_Ascent_4_DrawBG1RowAB [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0xa, plm::BACKGROUND_ROW_A), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0xb, plm::BACKGROUND_ROW_B), game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_C_D);
    };
    [11111674] MotherBrainBody_FakeDeath_Ascent_5_DrawBG1RowCD [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0xc, plm::BACKGROUND_ROW_C), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0xd, plm::BACKGROUND_ROW_D), game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_SET_UP_MOTHER_BRAIN_PHASE_2_GRAPHICS);
    };
    [11111697] MotherBrainBody_FakeDeath_Ascent_6_SetupPhase2Gfx [] {
        { let value: u16 = 257; let at: usize = 0x91b; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); };
        { let at: usize = 0x59; let bytes: [u8; 2] = ((u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])) & (0xfffcu16)).to_le_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }
        copy_rom_colors_to_palette(
            0x142,
            0xa9,
            (addr_kMotherBrainPalette_1 as i32 + 2) as u16,
            0xf,
            game,
        );
        copy_rom_colors_to_palette(
            0x162,
            0xa9,
            (addr_kMotherBrainPalette_0 as i32 + 2) as u16,
            0xf,
            game,
        );
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_SET_UP_MOTHER_BRAIN_PHASE_2_BRAIN);
        word!(game.nmi_flag_bg2_enemy_vram_transfer = 1);
        session_slot!(game, MotherBrainBrain[0x40i32].unpause_hook = 1);
    };
    [11111753] MotherBrainBody_FakeDeath_Ascent_7_SetupPhase2Brain [] {
        word!(game.fx_layer_blending_config_a = 52);
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].function = function::BRAIN_SET_UP_BRAIN_AND_NECK_TO_BE_DRAWN);
        let E0: i32 = 0;
        session_slot!(game, MotherBrainBody[E0].base.properties = (session_slot!(game, MotherBrainBody[E0].base.properties) as i32 & !(kEnemyProps_Tangible as i32)) as u16);
        session_slot!(game, MotherBrainBrain[E].base.properties = session_slot!(game, MotherBrainBrain[E].base.properties) & !(0x400));
        session_slot!(game, MotherBrainBrain[E].base.health = 18000);
        session_slot!(game, MotherBrainBody[E0].function = function::FAKE_DEATH_ASCENT_PAUSE_FOR_SUSPENSE);
        session_slot!(game, MotherBrainBody[E0].function_timer = 128);
        MotherBrainBody_FakeDeath_Ascent_8_Pause(game);
    };
    [11111860] MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles [] {
        if queue_mother_brain_sprite_tiles(0xa9, addr_stru_A98F8F as u16, game) != 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_CONTINUE_PAUSING_FOR_SUSPENSE);
            MotherBrainBody_FakeDeath_Ascent_11_ContinuePause(game);
        }
    };
    [11111916] MotherBrainBody_FakeDeath_Ascent_12_StartMusic [] {
        MotherBrain_SetBodyInstrs(animation::BODY_CROUCHED, game);
        let E1: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E1].base.instruction_timer = 1);
        let E0: i32 = 0;
        session_slot!(game, MotherBrainBody[E0].base.properties = (session_slot!(game, MotherBrainBody[E0].base.properties) as i32 & !(kEnemyProps_Invisible as i32)) as u16);
        session_slot!(game, MotherBrainBody[E0].base.x_pos = 59);
        session_slot!(game, MotherBrainBody[E0].base.y_pos = 279);
        word! {game; reg_BG2HOFS = -27i32 as u16; reg_BG2VOFS = -217i32 as u16; }
        QueueMusic_Delayed8(5, game);
        word! {game; earthquake_type = 2; earthquake_timer = 256; }
        session_slot! { game, MotherBrainBrain[E1]; neck_speed = 80, neck_moving = 1, lower_motion = 8, upper_motion = 6 };
        session_slot!(game, MotherBrainBody[E0].function = function::FAKE_DEATH_ASCENT_RAISE_MOTHER_BRAIN);
    };
    [11120417] MotherBrain_Phase3_Death_2 [] {
        MotherBrain_GenerateSmokyExplosions(game);
        if MotherBrain_MakeWalkForwards(2, 0x60, game) & 1 != 0 {
            MotherBrain_SetBrainInstrs(animation::BRAIN_DYING_DROOL, game);
            let E1: i32 = 0x40;
            session_slot! { game, MotherBrainBrain[E1]; lower_motion = 6, upper_motion = 6 }
            session_slot!(game, MotherBrainBrain[E1].neck_speed = 1280);
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_DISABLE_BRAIN_EFFECTS);
            session_slot!(game, MotherBrainBody[E].function_timer = 32);
        }
    };
    [11120566] MotherBrain_Phase3_Death_5 [] {
        game.flicker_body();
        MotherBrain_GenerateMixedExplosions(game);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[E].function_timer = 16);
            let v2: u16 = (session_slot!(game, MotherBrainBody[E].grey_transition)).wrapping_add(1);
            session_slot!(game, MotherBrainBody[E].grey_transition = v2);
            if game.fade(Fade::ToBlack, (v2).wrapping_sub(1)) & 1 != 0 {
                word!(game.enemy_bg2_tilemap_size = 710);
                word_transfers!(game, descending, 356; le(tilemap_stuff, 355) <- value(824));
                word!(game.nmi_flag_bg2_enemy_vram_transfer = 1);
                session_slot!(game, MotherBrainBody[E].base.properties = session_slot!(game, MotherBrainBody[E].base.properties) & 0xdeff | 0x100);
                session_slot!(game, MotherBrainBody[E].base.extra_properties = 0);
                session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_FINAL_FEW_EXPLOSIONS);
                session_slot!(game, MotherBrainBody[E].function_timer = 16);
            }
        }
    };
    [11120917] MotherBrain_Phase3_Death_7 [] {
        MotherBrain_SetBrainInstrs(animation::BRAIN_DECAPITATED, game);
        session_slot!(game, MotherBrainBrain[0x40i32].function = function::BRAIN_SET_UP_BRAIN_TO_BE_DRAWN);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = 0);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_BRAIN_FALLS_TO_GROUND);
        MotherBrain_Phase3_Death_8(game);
    };
    [11120990] MotherBrain_Phase3_Death_9 [] {
        if queue_mother_brain_sprite_tiles(0xa9, addr_stru_A99003 as u16, game) != 0 {
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_SET_UP_FADE_TO_GREY);
            session_slot!(game, MotherBrainBody[E].function_timer = 32);
        }
    };
    [11121109] MotherBrain_Phase3_Death_13 [] {
        if ProcessCorpseRotting(0x40, game) & 1 != 0 {
            let dms_var_53: u16 = session_slot!(game, Enemy_DeadMonsters[0x40i32].var_53);
            ProcessCorpseRottingVramTransfers(dms_var_53, game);
        } else {
            let mut E1: i32 = 0x40;
            session_slot!(game, Enemy_DeadMonsters[E1].base.properties = session_slot!(game, Enemy_DeadMonsters[E1].base.properties) & 0xdeff | 0x100);
            session_slot!(game, Enemy_DeadMonsters[E1].base.extra_properties = 0);
            QueueMusic_Delayed8(0, game);
            QueueMusic_Delayed8(0xff24, game);
            E1 = 0;
            session_slot!(game, Enemy_DeadMonsters[E1].var_A = function::PHASE_3_DEATH_20_FRAME_DELAY);
            session_slot!(game, Enemy_DeadMonsters[E1].var_F = 20);
            MotherBrain_Phase3_Death_14_20framedelay(game);
        };
    };
    [11121240] MotherBrain_Phase3_Death_15_LoadEscapeTimerTiles [] {
        if queue_mother_brain_sprite_tiles(0xa6, addr_stru_A6C4CB as u16, game) != 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_3_DEATH_START_ESCAPE_SEQUENCE);
            MotherBrain_Phase3_Death_16_StartEscape(game);
        }
    };
    [11121261] MotherBrain_Phase3_Death_16_StartEscape [] {
        if queue_mother_brain_sprite_tiles(0xa9, addr_stru_A9902F as u16, game) != 0 {
            copy_rom_colors_to_palette(
                0x122,
                0xa9,
                (addr_kMotherBrainPalette_5 as i32 + 2) as u16,
                0xe,
                game,
            );
            QueueMusic_Delayed8(7, game);
            word! {game; earthquake_type = 5; earthquake_timer = -1i32 as u16; }
            spawn_palette_effect(addr_kPalfx_FFC9 as u16, game);
            spawn_palette_effect(addr_kPalfx_FFCD as u16, game);
            spawn_palette_effect(addr_kPalfx_FFD1 as u16, game);
            spawn_palette_effect(addr_kPalfx_FFD5 as u16, game);
            session_slot!(game, MotherBrainBrain[0x40i32].unpause_hook = 0);
            start_zebes_escape_typewriter(game);
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function_timer = 32);
            let mut v1: u16 = function::PHASE_3_DEATH_SPAWN_TIME_BOMB_SET_JAPANESE_TEXT;
            if word!(game.japanese_text_flag) == 0 {
                v1 = function::PHASE_3_DEATH_TYPE_OUT_ZEBES_ESCAPE_TEXT;
            }
            session_slot!(game, MotherBrainBody[E].function = v1);
        }
    };
    [11121379] MotherBrain_Phase3_Death_18_TypesZebesText [] {
        if type_escape_text(0x2610, game) & 1 != 0 {
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_ESCAPE_DOOR_IS_EXPLODING_START_ESCAPE_TIMER);
            session_slot!(game, MotherBrainBody[E].function_timer = 32);
        }
    };
    [11121401] MotherBrain_Phase3_Death_19_EscapeDoorExploding [] {
        MotherBrain_GenerateEscapeDoorExploding(game);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0 {
            CallSomeSamusCode(0xf, game);
            word!(game.timer_status = 2);
            SetBossBitForCurArea(2, game);
            SetEventHappened(0xe, game);
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_BLOW_UP_ESCAPE_DOOR);
            let E1: i32 = 0x40;
            session_slot! { game, MotherBrainBrain[E1]; subfunction = 0, subfunction_timer = 0 }
        }
    };
    [11121450] MotherBrain_Phase3_Death_20_BlowUpEscapeDoor [] {
        MotherBrain_ExplodeEscapeDoor(game);
        session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_3_DEATH_KEEP_EARTHQUAKE_GOING);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0, 0x6, plm::ESCAPE_DOOR), game);
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    [11121468] MotherBrain_Phase3_Death_21_KeepEarthquakeGoing [] {
        if word!(game.earthquake_timer) == 0 {
            word!(game.earthquake_timer -= 1);
        }
    };
    @functions [pub fn (game: &mut impl Host)];
    [11122561] MotherBrain_FiringBomb_DecideOnWalking [] {
        if word!(game.random_number) >= 0xff80 {
            MotherBrain_FiringBomb_DecideOnCrouching(game);
            return;
        }
        let mut v0: u16 = 64;
        if word!(game.random_number) < 0x6000 {
            v0 = 96;
        }
        let E: i32 = 0;
        if ((v0) as i16).wrapping_sub((session_slot!(game, MotherBrainBody[E].base.x_pos)) as i16) >= 0 || {
            session_slot!(game, MotherBrainBody[E].function_timer = v0);
            MotherBrain_MakeWalkBackwards(v0, 6, game) & 1 != 0
        } {
            MotherBrain_FiringBomb_DecideOnCrouching(game);
        } else {
            session_slot!(game, MotherBrainBody[E].function = function::FIRING_BOMB_WALKING_BACKWARDS);
        };
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    [11122630] MotherBrain_FiringBomb_Crouch [] {
        if MotherBrain_MakeHerCrouch(game) & 1 != 0 {
            MotherBrain_B7CB(game);
        }
    };
    [11122680] MotherBrain_FiringBomb_Standup [] {
        if MotherBrain_MakeHerStandUp(game) & 1 != 0 {
            MotherBrain_FiringBomb_Finish(game);
        }
    };
    [11122702] MotherBomb_FiringLaser_PositionHead [] {
        let mut v0: i16 = 8;
        let E1: i32 = 0x40;
        if ((session_slot!(game, MotherBrainBrain[E1].base.y_pos)) as i16).wrapping_sub((word!(game.samus_y_pos)) as i16) < 0 {
            v0 = 6;
        }
        session_slot!(game, MotherBrainBrain[E1].lower_motion = v0 as u16);
        session_slot!(game, MotherBrainBrain[E1].upper_motion = v0 as u16);
        session_slot!(game, MotherBrainBrain[E1].neck_speed = 512);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_LASER_POSITION_HEAD_SLOWLY_AND_FIRE);
        session_slot!(game, MotherBrainBody[E].function_timer = 4);
    };
    [11122813] MotherBomb_FiringDeathBeam [] {
        let v0: i32 = session_slot!(game, MotherBrainBody[0i32].hand_beam_phase) as i32;
        match v0 {
            0 => MotherBomb_FiringDeathBeam_0(game),
            1 => MotherBomb_FiringDeathBeam_1(game),
            2 => MotherBomb_FiringDeathBeam_2(game),
            3 => MotherBomb_FiringDeathBeam_3(game),
            _ => panic!("index out of bounds: the len is 4 but the index is {}", v0),
        }
    };
    [11122923] MotherBomb_FiringRainbowBeam_0 [] {
        MotherBrain_SetBrainInstrs(animation::BRAIN_NEUTRAL_PHASE_2, game);
        let E1: i32 = 0x40;
        session_slot! { game, MotherBrainBrain[E1]; neck_speed = 64, neck_moving = 1, lower_motion = 2, upper_motion = 4 };
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_START_CHARGING_RAINBOW_BEAM);
        session_slot!(game, MotherBrainBody[E].function_timer = 256);
    };
    @functions [pub fn (game: &mut impl Host)];
    [11122987] MotherBomb_FiringRainbowBeam_2_RetractNeck [] {
        if MotherBrain_WalkBackwardsSlowlyAndRetractHead(0x28, game) & 1 != 0 {
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_WAIT_FOR_BEAM_TO_CHARGE);
            session_slot!(game, MotherBrainBody[E].function_timer = 256);
            MotherBomb_FiringRainbowBeam_3_Wait(game);
        }
    };
    [11123025] MotherBomb_FiringRainbowBeam_4_ExtendNeckDown [] {
        word!(game.cooldown_timer = 8);
        let E1: i32 = 0x40;
        session_slot! { game, MotherBrainBrain[E1]; lower_motion = 6, upper_motion = 6 }
        session_slot!(game, MotherBrainBrain[E1].neck_speed = 1280);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_START_FIRING_RAINBOW_BEAM);
        session_slot!(game, MotherBrainBody[E].function_timer = 16);
        MotherBomb_FiringRainbowBeam_5_StartFiring(game);
    };
    [11123061] MotherBomb_FiringRainbowBeam_5_StartFiring [] {
        MotherBrain_AimBeamAndIncrWidth(game);
        if word!(game.power_bomb_flag) == 0 {
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
            if (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0 && word!(game.power_bomb_flag) == 0 {
                word!(game.cooldown_timer = 0);
                MotherBrain_BodyRainbowBeamPalAnimIndex0(game);
                MotherBrain_SetBrainInstrs(animation::BRAIN_FIRING_RAINBOW_BEAM, game);
                session_slot!(game, MotherBrainBody[E].beam_width = 512);
                session_slot!(game, MotherBrainBody[E].hdma_object = spawn_rainbow_hdma(game));
                let E1: i32 = 0x40;
                session_slot! { game, MotherBrainBrain[E1]; neck_speed = 64, neck_moving = 1, lower_motion = 2, upper_motion = 4 };
                session_slot! { game, MotherBrainBody[E]; cooldown = 0, explosion_timer = 0 }
                let mut v4: u16 = 5;
                if ((word!(game.samus_health)) as i16).wrapping_sub(700) < 0 {
                    v4 = 24;
                }
                CallSomeSamusCode(v4, game);
                session_slot!(game, MotherBrainBody[E].beam_sounds_left = 6);
                session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_MOVE_SAMUS_TOWARDS_WALL);
            }
        }
    };
    [11123173] MotherBomb_FiringRainbowBeam_6_MoveSamusToWall [] {
        MotherBrain_PlayRainbowBeamSfx(game);
        handle_body_rainbow_palette(game);
        MotherBrain_AimBeamAndIncrWidth(game);
        MotherBrain_HandleRainbowBeamExplosions(game);
        if MotherBrain_MoveSamusTowardsWallDueToBeam(game) & 1 != 0 {
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_1_FRAME_DELAY);
            session_slot!(game, MotherBrainBody[E].function_timer = 0);
        }
    };
    [11123200] MotherBomb_FiringRainbowBeam_7_DelayFrame [] {
        MotherBrain_PlayRainbowBeamSfx(game);
        handle_body_rainbow_palette(game);
        MotherBrain_AimBeamAndIncrWidth(game);
        MotherBrain_HandleRainbowBeamExplosions(game);
        MotherBrain_MoveSamusTowardsWallDueToBeam(game);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0 {
            word! { game, 8; earthquake_type, earthquake_timer }
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_START_DRAINING_SAMUS);
        }
    };
    [11123239] MotherBomb_FiringRainbowBeam_8_StartDrainSamus [] {
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_DRAINING_SAMUS);
        session_slot!(game, MotherBrainBody[E].function_timer = 299);
        word! {game; earthquake_timer = 299; earthquake_type = 8; }
        MotherBomb_FiringRainbowBeam_9_DrainingSamus(game);
    };
    [11123260] MotherBomb_FiringRainbowBeam_9_DrainingSamus [] {
        MotherBrain_PlayRainbowBeamSfx(game);
        handle_body_rainbow_palette(game);
        MotherBrain_AimBeamAndIncrWidth(game);
        MotherBrain_HandleRainbowBeamExplosions(game);
        game.drain_health_by_rainbow_beam();
        game.drain_ammo_by_rainbow_beam();
        MotherBrain_MoveSamusTowardsMiddleOfWall(game);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_FINISH_FIRING_RAINBOW_BEAM);
        }
    };
    [11123294] MotherBomb_FiringRainbowBeam_10_FinishFiringRainbow [] {
        MotherBrain_PlayRainbowBeamSfx(game);
        handle_body_rainbow_palette(game);
        MotherBrain_AimBeam(game);
        MotherBrain_HandleRainbowBeamExplosions(game);
        let E: i32 = 0;
        let v1: u16 = (session_slot!(game, MotherBrainBody[E].beam_width)).wrapping_sub(384);
        session_slot!(game, MotherBrainBody[E].beam_width = v1);
        if v1 as i32 - 512i32 & 0x8000 != 0 {
            session_slot!(game, MotherBrainBody[E].beam_width = 512);
            session_slot!(game, MotherBrainBody[E].cooldown = -256i32 as u16);
            session_slot!(game, MotherBrainBody[E].explosion_timer = 0);
            crate::hdma_objects::free_hdma_object(session_slot!(game, MotherBrainBody[E].hdma_object), game);
            word!(game.earthquake_timer = 0);
            MotherBrain_SetBrainInstrs(animation::BRAIN_NEUTRAL_PHASE_2, game);
            MotherBrain_SetupBrainNormalPal(game);
            MotherBrain_WriteDefaultPalette(game);
            QueueSfx1_Max6(2, game);
            session_slot!(game, MotherBrainBody[E].beam_sound_playing = 0);
            CallSomeSamusCode(1, game);
            word!(game.cooldown_timer = 8);
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_LET_SAMUS_FALL);
        }
    };
    [11123396] MotherBomb_FiringRainbowBeam_11_LetSamusFall [] {
        player_mother_brain(0, game);
        session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_2_FIRING_RAINBOW_BEAM_WAIT_FOR_SAMUS_TO_HIT_GROUND);
        MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround(game);
    };
    [11123409] MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround [] {
        if MotherBrain_MoveSamusForFallingAfterBeam(game) & 1 != 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_2_FIRING_RAINBOW_BEAM_LOWER_HEAD);
        }
    };
    [11124037] MotherBrain_Phase2Cut_0 [] {
        let v0: u16 = { let amount = 0x50; game.suit_damage(amount) };
        if ((4 * v0 as i32 + 20 - word!(game.samus_health) as i32) as i16) >= 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_2_FINISH_SAMUS_OFF_STAND_UP);
        } else if (word!(game.random_number) & 0xfff) < 0xfa0 {
            if (session_slot!(game, MotherBrainBody[0i32].base.frame_counter) & 0x1f) == 0 {
                MotherBrain_MaybeStandupOrLeanDown(game);
            }
        } else {
            let mut v1: u16 = 0;
            if ((({ let amount = 0xa0; game.suit_damage(amount) }) as i32 + 20 - word!(game.samus_health) as i32)
                as i16)
                >= 0
                || {
                    v1 = animation::BRAIN_BOMB_PHASE_2;
                    (word!(game.random_number) & 0xfff) < 0xff0
                }
            {
                v1 = animation::BRAIN_TWO_BLUE_RINGS_PHASE_2;
            }
            MotherBrain_SetBrainInstrs(v1, game);
        };
    };
    [11124120] MotherBrain_Phase2Cut_1 [] {
        if MotherBrain_MakeHerStandUp(game) & 1 != 0 {
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FINISH_SAMUS_OFF_ADMIRE_JOB_WELL_DONE);
            session_slot!(game, MotherBrainBody[E].function_timer = 16);
            MotherBrain_Phase2Cut_2(game);
        }
    };
    [11124178] MotherBrain_Phase2Cut_4 [] {
        if queue_mother_brain_sprite_tiles(0xa9, addr_stru_A98FE5 as u16, game) != 0 {
            MotherBrain_RetractHead(game);
            MotherBrain_SpawnBabyMetroidInCutscene(game);
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FINISH_SAMUS_OFF_FIRE_FINAL_RAINBOW_BEAM);
            session_slot!(game, MotherBrainBody[E].function_timer = 256);
        }
    };
    [11124317] MotherBrain_DrainedByBabyMetroid_1 [] {
        handle_body_rainbow_palette(game);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[E].function = function::DRAINED_BY_BABY_METROID_FIRING_RAINBOW_BEAM);
            let E1: i32 = 0x40;
            session_slot!(game, MotherBrainBrain[E1].painful_walk_function = function::PAINFUL_WALK_FORWARDS);
            session_slot!(game, MotherBrainBrain[E1].painful_walk_stage = 0);
            session_slot! { game, MotherBrainBrain[E1]; painful_walk_delay = 2, lower_motion = 2 }
            session_slot!(game, MotherBrainBrain[E1].upper_motion = 4);
        }
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    [11124545] MotherBrain_DrainedByBabyMetroid_4 [] {
        if MotherBrain_MakeWalkBackwards(0x28, 0, game) & 1 != 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::DRAINED_BY_BABY_METROID_GO_INTO_LOW_POWER_MODE);
            session_slot!(game, MotherBrainBrain[0x40i32].upper_motion = 0);
            MotherBrain_DrainedByBabyMetroid_5(game);
        }
    };
    @functions [pub fn (game: &mut impl Host)];
    [11124922] MotherBrain_Phase2_Revive_4 [] {
        if MotherBrain_MakeHerStandUp(game) & 1 != 0 {
            let E1: i32 = 0x40;
            session_slot! { game, MotherBrainBrain[E1]; lower_motion = 6, upper_motion = 6 }
            session_slot!(game, MotherBrainBrain[E1].neck_speed = 1280);
            session_slot!(game, MotherBrainBrain[E1].neck_moving = 1);
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_REVIVE_SELF_WAKE_UP_STRETCH);
            session_slot!(game, MotherBrainBody[E].function_timer = 16);
            MotherBrain_Phase2_Revive_5(game);
        }
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    [11125063] MotherBrain_Phase2_Revive_8 [] {
        if MotherBrain_MakeHerStandUp(game) & 1 != 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_2_MURDER_BABY_METROID_ATTACK_BABY_METROID);
            MotherBrain_MakeWalkForwards(0xa, 0x50, game);
            MotherBrain_Phase2_MurderBabyMetroid_1(game);
        }
    };
    [11125084] MotherBrain_Phase2_MurderBabyMetroid_1 [] {
        MotherBrain_MaybeStandupOrLeanDown(game);
        if (word!(game.random_number) & 0x8000) != 0 {
            let mut v0: u16 = animation::BRAIN_FOUR_BLUE_RINGS_PHASE_3;
            if session_slot!(game, MotherBrainBrain[0x40i32].baby_metroid) != 0 {
                v0 = animation::BRAIN_ATTACKING_BABY_METROID;
            }
            MotherBrain_SetBrainInstrs(v0, game);
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_MURDER_BABY_METROID_BABY_METROID_ATTACK_COOLDOWN);
            session_slot!(game, MotherBrainBody[E].function_timer = 64);
        }
    };
    [11125134] MotherBrain_Phase2_PrepareForFinalBabyMetroid [] {
        MotherBrain_MakeHerStandUp(game);
        MotherBrain_MakeWalkBackwards(0x40, 4, game);
    };
    [11125146] MotherBrain_Phase2_ExecuteFinalkBabyMetroid [] {
        MotherBrain_SetBrainInstrs(animation::BRAIN_ATTACKING_BABY_METROID, game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FINAL_BABY_METROID_WAIT);
    };
    @functions [pub fn (game: &mut impl Host)];
    [11125257] MotherBrain_Phase3_Fighting_Main [] {
        if session_slot!(game, MotherBrainBrain[0x40i32].base.health) != 0 {
            MotherBrain_Phase3_NeckHandler(game);
            MotherBrain_Phase3_WalkHandler(game);
            let E: i32 = 0;
            if session_slot!(game, MotherBrainBody[E].pose) == 0 && session_slot!(game, MotherBrainBody[E].attacks_disabled) == 0 && (word!(game.random_number) & 0x8000) != 0 {
                let mut v1: u16 = animation::BRAIN_BOMB_PHASE_3;
                if (word!(game.random_number) as u8) >= 0x80 {
                    v1 = animation::BRAIN_FOUR_BLUE_RINGS_PHASE_3;
                }
                MotherBrain_SetBrainInstrs(v1, game);
                session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_FIGHTING_ATTACK_COOLDOWN);
                session_slot!(game, MotherBrainBody[E].function_timer = 64);
            }
        } else {
            session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_3_DEATH_MOVE_TO_BACK_OF_ROOM);
        };
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    @prefix { let E1: i32 = 0x40; };
    [11123421] MotherBomb_FiringRainbowBeam_13_LowerHead [] {
        session_slot! { game, MotherBrainBrain[E1]; neck_speed = 64, neck_moving = 1, lower_motion = 2, upper_motion = 4 };
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_DECIDE_NEXT_ACTION);
        session_slot!(game, MotherBrainBody[E].function_timer = 128);
    };
    @functions [pub fn (game: &mut impl Host)];
    @prefix { let E1: i32 = 0x40; };
    [11124374] MotherBrain_DrainedByBabyMetroid_2 [] {
        if session_slot!(game, MotherBrainBrain[E1].shake_timer) == 0 {
            session_slot!(game, MotherBrainBrain[E1].shake_timer = 50);
        }
        handle_body_rainbow_palette(game);
        CallMotherBrainPainfulWalk(((session_slot!(game, MotherBrainBrain[E1].painful_walk_function)) as u32) | (0xa90000u32), game);
        let v2: u16 = (2u16).wrapping_mul(session_slot!(game, MotherBrainBrain[E1].painful_walk_stage));
        session_slot!(game, MotherBrainBrain[E1].painful_walk_delay = g_byte_A9BEEE[v2 as usize] as u16);
        session_slot!(game, MotherBrainBrain[E1].neck_speed = g_word_A9BEFE[(v2 >> 1) as usize]);
        if session_slot!(game, MotherBrainBrain[E1].painful_walk_stage) == 6 {
            let E: i32 = 0;
            session_slot!(game, MotherBrainBody[E].beam_sound_playing = 0);
            session_slot!(game, MotherBrainBrain[E1].brain_palette_enabled = 0);
            MotherBrain_WritePhase2DeathPalette(game);
            QueueSfx1_Max6(2, game);
            session_slot!(game, MotherBrainBody[E].function = function::DRAINED_BY_BABY_METROID_RAINBOW_BEAM_HAS_RUN_OUT);
        }
    };
    [11124494] MotherBrain_DrainedByBabyMetroid_3 [] {
        CallMotherBrainPainfulWalk(((session_slot!(game, MotherBrainBrain[E1].painful_walk_function)) as u32) | (0xa90000u32), game);
        if session_slot!(game, MotherBrainBrain[E1].painful_walk_stage) as i32 - 8i32 & 0x8000 == 0 {
            session_slot!(game, MotherBrainBrain[E1].neck_speed = 64);
            session_slot! { game, MotherBrainBrain[E1]; lower_motion = 8, upper_motion = 8 }
            MotherBrain_SetBrainInstrs(animation::BRAIN_DYING_DROOL, game);
            session_slot!(game, MotherBrainBody[0i32].function = function::DRAINED_BY_BABY_METROID_MOVE_TO_BACK_OF_ROOM);
            MotherBrain_DrainedByBabyMetroid_4(game);
        }
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    @prefix { let E1: i32 = 0x40; };
    [11124566] MotherBrain_DrainedByBabyMetroid_5 [] {
        if (session_slot!(game, MotherBrainBrain[E1].lower_motion) | session_slot!(game, MotherBrainBrain[E1].upper_motion)) == 0 {
            session_slot!(game, MotherBrainBrain[E1].drool_enabled = (session_slot!(game, MotherBrainBrain[E1].upper_motion)) | (session_slot!(game, MotherBrainBrain[E1].lower_motion)));
            let E: i32 = 0;
            if session_slot!(game, MotherBrainBody[E].pose) == 0 {
                MotherBrain_SetBodyInstrs(animation::BODY_CROUCH_FAST, game);
                session_slot!(game, MotherBrainBody[E].function = function::DRAINED_BY_BABY_METROID_PREPARE_TRANSITION_TO_GREY);
                session_slot!(game, MotherBrainBody[E].function_timer = 64);
            }
        }
    };
    @functions [pub fn (game: &mut impl Host)];
    @prefix { let E: i32 = 0; };
    [11110578] MotherBrainBody_FakeDeath_Descent_5 [] {
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[E].function_timer = 8);
            let v2: u16 = (session_slot!(game, MotherBrainBody[E].grey_transition)).wrapping_add(1);
            session_slot!(game, MotherBrainBody[E].grey_transition = v2);
            if game.fade(Fade::FakeDeathToGray, (v2).wrapping_sub(1)) & 1 != 0 {
                session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_DESCENT_COLLAPSE_TUBES);
            }
        }
        MotherBrainBody_FakeDeath_Descent_6(game);
    };
    MotherBrain_HandleFakeDeathExplosions [] {
        let mut v1: i16 = ((session_slot!(game, MotherBrainBody[E].fake_death_explosion_timer)) as i16).wrapping_sub(1);
        if v1 < 0 {
            session_slot!(game, MotherBrainBody[E].fake_death_explosion_timer = 8);
            let mut v2: i16 = ((session_slot!(game, MotherBrainBody[E].fake_death_explosion_index)) as i16).wrapping_sub(1);
            if v2 < 0 {
                v2 = 7;
            }
            session_slot!(game, MotherBrainBody[E].fake_death_explosion_index = v2 as u16);
            let v3: i32 = ((4u16).wrapping_mul((v2) as u16)) as i32 >> 1;
            let explosion = mother_brain_tables::fake_death_explosion(game, (v3 >> 1) as isize);
            crate::actors::host::BossEffects::set_projectile_spawn_point(game, explosion.x, explosion.y);
            let v4: u16 = (if word!(game.random_number) < 0x4000 {
                12
            } else {
                3
            }) as u16;
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v4, game);
            QueueSfx2_Max3(0x24, game);
        } else {
            session_slot!(game, MotherBrainBody[E].fake_death_explosion_timer = v1 as u16);
        };
    };
    [11112013] MotherBrainBody_FakeDeath_Ascent_13_Raise [] {
        if !((word!(game.nmi_frame_counter_word) & 3) != 0 || {
            MotherBrain_SpawnDustCloudsForAscent(game);
            word!(game.reg_BG2VOFS += 2);
            let mut v1: u16 = (session_slot!(game, MotherBrainBody[E].base.y_pos)).wrapping_sub(2);
            session_slot!(game, MotherBrainBody[E].base.y_pos = v1);
            v1 >= 0xbd
        }) {
            word!(game.enemy_bg2_tilemap_size = 320);
            session_slot!(game, MotherBrainBody[E].base.y_pos = 188);
            word!(game.earthquake_timer = 0);
            crate::hdma_objects::free_hdma_object(session_slot!(game, MotherBrainBody[E].hdma_object), game);
            MotherBrain_SetBodyInstrs(animation::BODY_STAND_UP_FROM_CROUCH_SLOW, game);
            session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_ASCENT_WAIT_UNTIL_MOTHER_BRAIN_HAS_FINISHED_UNCROUCHING);
            MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(game);
        }
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    @prefix { let E: i32 = 0; };
    [11112085] MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching [] {
        if session_slot!(game, MotherBrainBody[E].pose) == 0 {
            session_slot!(game, MotherBrainBody[E].grey_transition = 0);
            session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_ASCENT_TRANSITION_MOTHER_BRAIN_FROM_GREY_AND_LOWER_HER_HEAD);
            session_slot!(game, MotherBrainBody[E].function_timer = 0);
        }
    };
    @functions [pub fn (game: &mut impl Host)];
    @prefix { let E: i32 = 0; };
    [11120353] MotherBrain_Phase3_Death_0 [] {
        session_slot!(game, MotherBrainBody[E].base.properties = (session_slot!(game, MotherBrainBody[E].base.properties)) | ((kEnemyProps_Tangible) as u16));
        let E1: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E1].base.properties = (session_slot!(game, MotherBrainBrain[E1].base.properties)) | (0x400u16));
        session_slot!(game, MotherBrainBody[E].hitboxes = 0);
        if MotherBrain_MakeWalkBackwards(0x28, 6, game) & 1 != 0 {
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_IDLE_WHILST_EXPLODING);
            session_slot!(game, MotherBrainBody[E].function_timer = 128);
            MotherBrain_Phase3_Death_1(game);
        }
    };
    [11120941] MotherBrain_Phase3_Death_8 [] {
        let mut v1: u16 = (session_slot!(game, MotherBrainBody[E].function_timer)).wrapping_add(32);
        session_slot!(game, MotherBrainBody[E].function_timer = v1);
        let E1: i32 = 0x40;
        let mut v3: i16 = ((session_slot!(game, MotherBrainBrain[E1].base.y_pos)) as i16).wrapping_add((v1.to_ne_bytes()[1]) as i16);
        if (v3 as u16) >= 0xc4 {
            crate::actors::mother_brain_tubes::Host::start_earthquake(game, 2);
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_LOAD_CORPSE_TILES);
            session_slot!(game, MotherBrainBody[E].function_timer = 256);
            v3 = 196;
        }
        session_slot!(game, MotherBrainBrain[E1].base.y_pos = v3 as u16);
    };
    [11121361] MotherBrain_Phase3_Death_17_SpawnTimeBomb [] {
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_TYPE_OUT_ZEBES_ESCAPE_TEXT);
            SpawnEprojWithRoomGfx(
                addr_kEproj_TimeBombSetJapaneseText as u16,
                function::PHASE_3_DEATH_TYPE_OUT_ZEBES_ESCAPE_TEXT,
                game,
            );
        }
        MotherBrain_Phase3_Death_18_TypesZebesText(game);
    };
    MotherBrainsBrain_Shot [] {
        if session_slot!(game, MotherBrainBody[E].form) != 0 {
            MotherBrain_Phase23_ShotReaction(game);
            if session_slot!(game, MotherBrainBody[E].form) == 1 {
                game.create_dud();
            } else {
                game.shot_reaction(crate::actors::host::Shot::ReactionOnly);
            }
        } else {
            let v1: u16 = ((game.ram().read_byte(0xc18 + 2 * word!(game.collision_detection_index) as usize + 1))
                as u16)
                & (7u16);
            if g_byte_A9B546[v1 as usize] != 0 {
                add_glass_hits(
                    39,
                    u16::from(g_byte_A9B546[v1 as usize]),
                    game,
                );
                QueueSfx2_Max6(0x6e, game);
                let mut v2: i16 = 13;
                let E1: i32 = 0x40;
                let flash_timer: u16 = session_slot!(game, MotherBrainBrain[E1].base.flash_timer);
                if flash_timer != 0 && flash_timer & 1 != 0 {
                    v2 = 14;
                }
                session_slot!(game, MotherBrainBrain[E1].base.flash_timer = v2 as u16);
                game.shot_reaction(crate::actors::host::Shot::ReactionOnly);
            }
        };
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    @prefix { let E: i32 = 0; };
    MotherBrain_Phase3_BeamShotReaction [] {
        let mut v1: i16 = ((session_slot!(game, MotherBrainBody[E].walk_counter)) as i16).wrapping_sub(266);
        if v1 < 0 {
            session_slot!(game, MotherBrainBrain[0x40i32].neck_function = neck::SETUP_HYPER_BEAM_RECOIL);
            v1 = 0;
            session_slot!(game, MotherBrainBody[E].function_timer = 0);
        }
        session_slot!(game, MotherBrainBody[E].walk_counter = v1 as u16);
    };
    [11122181] MotherBrain_Body_Phase2_Thinking [] {
        let E1: i32 = 0x40;
        if session_slot!(game, MotherBrainBrain[E1].base.health) == 0 {
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_EXTEND_NECK);
            MotherBomb_FiringRainbowBeam_0(game);
            return;
        }
        if session_slot!(game, MotherBrainBody[E].pose) == 0 {
            let keep_walking: bool = if session_slot!(game, MotherBrainBrain[E1].base.health) as i32 - 4500i32 & 0x8000 == 0 {
                word!(game.random_number) >= 0x1000
            } else if word!(game.random_number) < 0x2000 {
                true
            } else if word!(game.random_number) >= 0xa000 {
                false
            } else {
                session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_HAND_BEAM);
                return;
            };
            if keep_walking {
                MotherBrain_HandleWalking(game)
            } else {
                session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_TRY_ATTACK);
            }
        }
    };
    @functions [pub fn (game: &mut impl Host)];
    @prefix { let E: i32 = 0; };
    [11122251] MotherBrain_Body_Phase2_TryAttack [] {
        let mut v0: i16 = (2i16).wrapping_mul((session_slot!(game, MotherBrainBody[E].attack_phase)) as i16);
        if v0 != 0 {
            if v0 == 2 {
                MotherBrain_Phase2_Attack_Cooldown(game);
            } else {
                if v0 != 4 {
                    Unreachable(game);
                    loop {}
                }
                MotherBrain_Phase2_Attack_End(game);
            }
        } else {
            session_slot!(game, MotherBrainBody[E].cooldown = 64);
            session_slot!(game, MotherBrainBody[E].attack_phase = session_slot!(game, MotherBrainBody[E].attack_phase).wrapping_add(1));
            if MotherBrain_Phase2_DecideAttackStrategy_DoubleRet(game) != 0 {
                return;
            }
            let mut v2: u16 = addr_byte_A9B6DC as u16;
            let E1: i32 = 0x40;
            if abs16((session_slot!(game, MotherBrainBrain[E1].base.y_pos) as i32 + 4 - word!(game.samus_y_pos) as i32) as u16) < 0x20 {
                v2 = addr_byte_A9B6DF as u16;
            }
            let mut v4: u16 = 0;
            let v5 = crate::cartridge::native_record(0xa9, v2, game);
            if (word!(game.random_number) as u8) >= v5.byte(0) {
                v4 = 2;
                if (word!(game.random_number) as u8) >= v5.byte(1) {
                    v4 = 4;
                    if (word!(game.random_number) as u8) >= v5.byte(2) {
                        v4 = 6;
                    }
                }
            }
            let v6: u16 =
                mother_brain_tables::brain_attack(game, (v4 >> 1) as isize).instruction_list;
            if v6 == animation::BRAIN_BOMB_PHASE_2 {
                if session_slot!(game, MotherBrainBrain[E1].bombs) as i32 - 1i32 & 0x8000 != 0 {
                    session_slot!(game, MotherBrainBody[E].function = function::FIRING_BOMB_DECIDE_ON_WALKING);
                    MotherBrain_FiringBomb_DecideOnWalking(game);
                }
            } else if v6 == animation::BRAIN_LASER {
                session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_LASER_POSITION_HEAD_QUICKLY);
                MotherBomb_FiringLaser_PositionHead(game);
            } else {
                MotherBrain_SetBrainInstrs(v6, game);
            }
        };
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    @prefix { let E: i32 = 0; };
    MotherBrain_Phase2_Attack_Cooldown [] {
        let fresh9: u16 = session_slot!(game, MotherBrainBody[E].cooldown);
        session_slot!(game, MotherBrainBody[E].cooldown = session_slot!(game, MotherBrainBody[E].cooldown).wrapping_sub(1));
        if fresh9 == 1 {
            session_slot!(game, MotherBrainBody[E].attack_phase = session_slot!(game, MotherBrainBody[E].attack_phase).wrapping_add(1));
        }
    };
    MotherBrain_Phase2_Attack_End [] {
        session_slot!(game, MotherBrainBody[E].attack_phase = 0);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_THINKING);
    };
    @functions [pub fn (game: &mut impl Host)];
    @prefix { let E: i32 = 0; };
    [11122604] MotherBrain_FiringBomb_WalkingBackwards [] {
        if MotherBrain_MakeWalkBackwards(session_slot!(game, MotherBrainBody[E].function_timer), 6, game) & 1 != 0 {
            MotherBrain_FiringBomb_DecideOnCrouching(game);
        }
    };
    MotherBrain_PlayRainbowBeamSfx [] {
        let mut mbn_var_15: i16 = session_slot!(game, MotherBrainBody[E].beam_sounds_left) as i16;
        if mbn_var_15 >= 0 {
            session_slot!(game, MotherBrainBody[E].beam_sounds_left = ((mbn_var_15) as u16).wrapping_sub(1));
            QueueSfx1_Max6(0x40, game);
            session_slot!(game, MotherBrainBody[E].beam_sound_playing = 1);
        }
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    @prefix { let E: i32 = 0; };
    MotherBrain_AimBeamAndIncrWidth [] {
        let mut v1: i16 = ((session_slot!(game, MotherBrainBody[E].beam_width)) as i16).wrapping_add(384);
        if session_slot!(game, MotherBrainBody[E].beam_width) as i32 - 2688i32 & 0x8000 == 0 {
            v1 = 3072;
        }
        session_slot!(game, MotherBrainBody[E].beam_width = v1 as u16);
        MotherBrain_AimBeam(game);
    };
    @functions [pub fn (game: &mut impl Host)];
    @prefix { let E: i32 = 0; };
    MotherBrain_HandleRainbowBeamExplosions [] {
        session_slot!(game, MotherBrainBody[E].explosion_timer = session_slot!(game, MotherBrainBody[E].explosion_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[E].explosion_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[E].explosion_timer = 8);
            session_slot!(game, MotherBrainBody[E].cooldown = session_slot!(game, MotherBrainBody[E].cooldown).wrapping_add(1));
            let v2: i32 = session_slot!(game, MotherBrainBody[E].cooldown) as i32 & 7;
            crate::actors::host::BossEffects::set_projectile_spawn_point(game, g_word_A9BCA6[v2 as usize] as u16, g_word_A9BCB6[v2 as usize] as u16);
            SpawnEprojWithRoomGfx(
                addr_kEproj_MotherBrainRainbowBeamExplosion as u16,
                g_word_A9BCB6[v2 as usize] as u16,
                game,
            );
            QueueSfx2_Max6(0x24, game);
        }
    };
    [11124280] MotherBrain_DrainedByBabyMetroid_0 [] {
        session_slot!(game, MotherBrainBody[E].form = 3);
        let E1: i32 = 0x40;
        session_slot! { game, MotherBrainBrain[E1]; lower_motion = 8, upper_motion = 8 }
        session_slot!(game, MotherBrainBrain[E1].neck_speed = 1792);
        session_slot!(game, MotherBrainBody[E].function = function::DRAINED_BY_BABY_METROID_REGAIN_BALANCE);
        session_slot!(game, MotherBrainBody[E].function_timer = 48);
        MotherBrain_DrainedByBabyMetroid_1(game);
    };
    [11124629] MotherBrain_DrainedByBabyMetroid_7 [] {
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
        let v2: bool = (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0;
        if v2 && {
            session_slot!(game, MotherBrainBody[E].function_timer = 16);
            let mut v3: u16 = (session_slot!(game, MotherBrainBody[E].grey_transition)).wrapping_add(1);
            session_slot!(game, MotherBrainBody[E].grey_transition = v3);
            game.fade(Fade::DrainedToGray, (v3).wrapping_sub(1)) & 1 != 0
        } {
            let E1: i32 = 0x40;
            session_slot!(game, MotherBrainBrain[E1].base.health = -29536i32 as u16);
            session_slot!(game, MotherBrainBody[E].corpse_state = 1);
            session_slot!(game, MotherBrainBrain[E1].breath_enabled = 0);
            session_slot!(game, MotherBrainBody[E].form = 2);
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_REVIVE_SELF_INANIMATE_GREY);
        }
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    @prefix { let E: i32 = 0; };
    [11124825] MotherBrain_Phase2_Revive_0 [] {
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_REVIVE_SELF_SHOW_SIGNS_OF_LIFE);
        session_slot!(game, MotherBrainBody[E].function_timer = 768);
    };
    @functions [pub fn (game: &mut impl Host)];
    @prefix { let E: i32 = 0; };
    [11124866] MotherBrain_Phase2_Revive_2 [] {
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_REVIVE_SELF_TRANSITION_FROM_GREY);
        session_slot!(game, MotherBrainBody[E].grey_transition = 0);
        MotherBrain_Phase2_Revive_3(game);
    };
    [11124879] MotherBrain_Phase2_Revive_3 [] {
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
        let v1: bool = (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0;
        if v1 && {
            session_slot!(game, MotherBrainBody[E].function_timer = 16);
            let mut v2: u16 = (session_slot!(game, MotherBrainBody[E].grey_transition)).wrapping_add(1);
            session_slot!(game, MotherBrainBody[E].grey_transition = v2);
            game.fade(Fade::DrainedFromGray, (v2).wrapping_sub(1)) & 1 != 0
        } {
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_REVIVE_SELF_WAKE_UP);
            session_slot!(game, MotherBrainBrain[0x40i32].brain_palette_enabled = 1);
            MotherBrain_SetupBrainNormalPal(game);
            MotherBrain_Phase2_Revive_4(game);
        }
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    @prefix { let E: i32 = 0; };
    [11124987] MotherBrain_Phase2_Revive_6 [] {
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
        let v1: bool = (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0;
        if v1 && MotherBrain_MakeWalkForwards(4, 0x50, game) & 1 != 0 {
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_REVIVE_SELF_PREPARE_NECK_FOR_BABY_METROID_MURDER);
            session_slot!(game, MotherBrainBody[E].corpse_state = 2);
            session_slot!(game, MotherBrainBrain[0x40i32].health_palette_enabled = 1);
        }
    };
    [11125022] MotherBrain_Phase2_Revive_7 [] {
        session_slot!(game, MotherBrainBody[E].ring_volleys = 0);
        let E1: i32 = 0x40;
        session_slot! { game, MotherBrainBrain[E1]; neck_moving = 1, lower_motion = 2, upper_motion = 4, neck_speed = 64 };
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_REVIVE_SELF_FINISH_PREPARING_FOR_BABY_METROID_MURDER);
        MotherBrain_Phase2_Revive_8(game);
    };
    [11125199] MotherBrain_Phase3_Recover_MakeDistance [] {
        session_slot!(game, MotherBrainBody[E].form = 4);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_RECOVER_FROM_CUTSCENE_SET_UP_FOR_FIGHTING);
        session_slot!(game, MotherBrainBody[E].function_timer = 32);
        MotherBrain_MakeWalkBackwards((session_slot!(game, MotherBrainBody[E].base.x_pos)).wrapping_sub(14), 2, game);
    };
    MotherBrain_Phase3_Walk_TryToInchForward [] {
        let mbn_var_07: u16 = session_slot!(game, MotherBrainBody[E].walk_counter);
        if mbn_var_07 != 0 {
            let v3: u16 = (mbn_var_07).wrapping_add(32);
            session_slot!(game, MotherBrainBody[E].walk_counter = v3);
            if v3 >= 0x100 {
                let v4: u16 = (session_slot!(game, MotherBrainBody[E].base.x_pos)).wrapping_add(1);
                let E1: i32 = 0x40;
                session_slot!(game, MotherBrainBrain[E1].target_x = v4);
                if MotherBrain_MakeWalkForwards(
                    ((word!(game.random_number) as i32 & 2) + 4) as u16,
                    session_slot!(game, MotherBrainBrain[E1].target_x),
                    game,
                ) & 1
                    != 0
                {
                    session_slot!(game, MotherBrainBody[E].walk_counter = 128);
                }
            }
        } else {
            let v6: u16 = (session_slot!(game, MotherBrainBody[E].base.x_pos)).wrapping_sub(14);
            let E1_0: i32 = 0x40;
            session_slot!(game, MotherBrainBrain[E1_0].target_x = v6);
            session_slot!(game, MotherBrainBrain[E1_0].walk_function = function::WALK_RETREAT_QUICKLY);
            MotherBrain_Phase3_Walk_RetreatQuickly(game);
        };
    };
    MotherBrain_HandleWalking [] {
        if session_slot!(game, MotherBrainBody[E].pose) == 0 {
            let mbn_var_07: u16 = session_slot!(game, MotherBrainBody[E].walk_counter);
            let can_pause_walk: bool = if mbn_var_07 != 0 {
                let mut v2: u16 = (mbn_var_07).wrapping_add(6);
                session_slot!(game, MotherBrainBody[E].walk_counter = v2);
                !(v2 >= 0x100)
                    && ({
                        if session_slot!(game, MotherBrainBody[E].base.x_pos) as i32 - 48i32 & 0x8000 == 0 {
                            return;
                        }
                        true
                    })
            } else {
                session_slot!(game, MotherBrainBody[E].walk_counter = 1);
                if session_slot!(game, MotherBrainBody[E].base.x_pos) as i32 - 48i32 & 0x8000 == 0 {
                    MotherBrain_SetBodyInstrs(animation::BODY_WALK_BACKWARDS_FAST, game);
                    return;
                }
                true
            };
            if can_pause_walk && (word!(game.random_number) as i32 & 0xfff) - 4032i32 & 0x8000 != 0 {
                return;
            }
            session_slot!(game, MotherBrainBody[E].walk_counter = 128);
            if session_slot!(game, MotherBrainBody[E].base.x_pos) as i32 - 128i32 & 0x8000 != 0 {
                MotherBrain_SetBodyInstrs(animation::BODY_WALK_FORWARDS_MEDIUM, game);
            }
        }
    };
    MotherBrain_Pal_BeginScreenFlash [] {
        session_slot!(game, MotherBrainBody[E].room_palette_list = addr_stru_A9D046 as u16);
        session_slot!(game, MotherBrainBody[E].room_palette_timer = 1);
    };
    @functions [pub fn (game: &mut impl Host)];
    @prefix { let E: i32 = 0; };
    MotherBrain_Pal_EndScreenFlash [] {
        session_slot! { game, MotherBrainBody[E]; room_palette_list = 0, room_palette_timer = 0 }
        MotherBrain_Pal_WriteRoomPal(addr_word_A9D082 as u16, game);
    };
    MotherBrain_HandleBrainPal [] {
        if session_slot!(game, MotherBrainBody[E].form) != 1 {
            let mut mbn_var_D_high: i8 = session_slot!(byte game, MotherBrainBody[E].brain_palette_counter[1]) as i8;
            if mbn_var_D_high != 0 {
                session_slot!(byte game, MotherBrainBody[E].brain_palette_counter[1] = ((mbn_var_D_high) as u8).wrapping_sub(1));
            } else {
                let r18: u16 = mother_brain_tables::brain_palette_set(
                    game,
                    (session_slot!(byte game, MotherBrainBody[E].brain_palette_reload[1]) >> 1) as isize,
                )
                .colors;
                session_slot!(byte game, MotherBrainBody[E].brain_palette_counter[1] = session_slot!(game, MotherBrainBody[E].brain_palette_reload) as u8);
                let mut mbn_var_D: i8 = session_slot!(game, MotherBrainBody[E].brain_palette_counter) as i8;
                if mbn_var_D != 0 || session_slot!(game, MotherBrainBrain[0x40i32].function) as i32 == 0xa987a2 & 0xffff
                {
                    let v4: u8 = (mbn_var_D as i32 + 1 & 7) as u8;
                    session_slot!(byte game, MotherBrainBody[E].brain_palette_counter[0] = v4);
                    let mut v5: u16 = 290;
                    if session_slot!(game, MotherBrainBody[E].brain_palette) != 512 {
                        v5 = 482;
                    }
                    copy_rom_colors_to_palette(v5, 0xa9, (r18 as i32 + 8 * v4 as i32) as u16, 3, game);
                }
            }
        }
    };
    @forward [pub fn (game: &mut impl Host)];
    [11110352]
    MotherBrainsBrain_SetupBrainToDraw => set_graphics_drawn_hook(DRAW_BRAIN_HOOK, game);
    MotherBrainsBrain_GfxDrawHook => MotherBrain_DrawBrain(game);
    @countdown [pub fn (game: &mut impl Host)];
    before {
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].subfunction_timer = session_slot!(game, MotherBrainBrain[E].subfunction_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBrain[E].subfunction_timer) & 0x8000) != 0;
    [11110787] MotherBrainBody_2_SpawnTopRightTubeFalling {
        crate::actors::host::BossEffects::set_projectile_spawn_point(game, 152, 47);
        SpawnEprojWithRoomGfx(0xcc5b, 0x2f, game);
        session_slot!(game, MotherBrainBrain[E].subfunction = -30304i32 as u16);
    };
    [11110837] MotherBrainBody_4_SpawnTopLeftTubeFalling {
        crate::actors::host::BossEffects::set_projectile_spawn_point(game, 104, 47);
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopLeft as u16,
            0x2f,
            game,
        );
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_CLEAR_CEILING_BLOCK_COLUMN_6);
    };
    [11110887] MotherBrainBody_5_SpawnTubeFallingEnemy1 {
        game.spawn_legacy_population(0xa9, addr_stru_A98AF5 as u16);
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_CLEAR_BOTTOM_RIGHT_TUBE);
    };
    [11110927] MotherBrainBody_7_SpawnTubeFallingEnemy2 {
        game.spawn_legacy_population(0xa9, addr_stru_A98B05 as u16);
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_CLEAR_BOTTOM_MIDDLE_LEFT_TUBE);
    };
    [11110967] MotherBrainBody_9_SpawnTopMiddleLeftFalling {
        crate::actors::host::BossEffects::set_projectile_spawn_point(game, 120, 59);
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopMiddleLeft as u16,
            0x3b,
            game,
        );
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_CLEAR_CEILING_TUBE_COLUMN_7);
    };
    [11111017] MotherBrainBody_11_SpawnTopMiddleRightFalling {
        crate::actors::host::BossEffects::set_projectile_spawn_point(game, 136, 59);
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopMiddleRight as u16,
            0x3b,
            game,
        );
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_CLEAR_CEILING_TUBE_COLUMN_8);
    };
    [11111067] MotherBrainBody_13_SpawnTubeFallingEnemy3 {
        ({ let (bank, population) = (0xa9, ((0x868b15i32) as u16)); game.spawn_legacy_population(bank, population) });
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_CLEAR_BOTTOM_MIDDLE_RIGHT_TUBE);
    };
    [11111107] MotherBrainBody_15_SpawnTubeFallingEnemy4 {
        game.spawn_legacy_population(0xa9, addr_stru_A98B25 as u16);
        session_slot!(game, MotherBrainBrain[E].subfunction = function::TUBES_CLEAR_BOTTOM_MIDDLE_TUBES);
    };
    @countdown [pub fn (game: &mut impl Host)];
    before {
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0;
    [11110441] MotherBrainBody_FakeDeath_Descent_1 {
        CallSomeSamusCode(0, game);
        { let value: u16 = game.ram().read_byte(0xcd20 + 0) as u16; let at: usize = 0xcd20; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); };
        session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_DESCENT_QUEUE_MOTHER_BRAIN_MUSIC);
        session_slot!(game, MotherBrainBody[E].function_timer = 32);
        MotherBrainBody_FakeDeath_Descent_2(game);
    };
    [11110477] MotherBrainBody_FakeDeath_Descent_2 {
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff21, game);
        session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_DESCENT_UNLOCK_SAMUS);
        session_slot!(game, MotherBrainBody[E].function_timer = 12);
        MotherBrainBody_FakeDeath_Descent_3(game);
    };
    [11110508] MotherBrainBody_FakeDeath_Descent_3 {
        CallSomeSamusCode(1, game);
        session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_DESCENT_BEGIN_SCREEN_FLASHING_AND_LOWER_ACID);
        session_slot!(game, MotherBrainBody[E].function_timer = 8);
        MotherBrainBody_FakeDeath_Descent_4(game);
    };
    [11111801] MotherBrainBody_FakeDeath_Ascent_8_Pause {
        session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_ASCENT_PREPARE_MOTHER_BRAIN_FOR_RISING);
        session_slot!(game, MotherBrainBody[E].function_timer = 32);
        MotherBrainBody_FakeDeath_Ascent_9_PrepareRise(game);
    };
    @countdown [pub fn (game: &mut impl Host)];
    before {
        MotherBrain_GenerateSmokyExplosions(game);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0;
    [11120402] MotherBrain_Phase3_Death_1 {
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_STUMBLE_TO_MIDDLE_OF_ROOM_AND_DROOL);
    };
    [11120468] MotherBrain_Phase3_Death_3 {
        let E1: i32 = 0x40;
        session_slot! { game, MotherBrainBrain[E1]; lower_motion = 0, upper_motion = 0, drool_enabled = 0, breath_enabled = 0, brain_palette_enabled = 0, health_palette_enabled = 0 }
        word_transfers!(game, descending, 15; le(palette_buffer, 255) <- le(palette_buffer, 159));
        game.update_health_palette();
        session_slot!(game, MotherBrainBody[E].brain_palette = 3584);
        session_slot! { game, MotherBrainBrain[E1]; subfunction = 0, subfunction_timer = 0 }
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_SET_UP_BODY_FADE_OUT);
        MotherBrain_Phase3_Death_4(game);
    };
    @countdown [pub fn (game: &mut impl Host)];
    before {
        MotherBrain_GenerateMixedExplosions(game);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0;
    [11120541] MotherBrain_Phase3_Death_4 {
        session_slot!(game, MotherBrainBody[E].grey_transition = 0);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_FADE_OUT_BODY);
        session_slot!(game, MotherBrainBody[E].function_timer = 0);
        MotherBrain_Phase3_Death_5(game);
    };
    [11120659] MotherBrain_Phase3_Death_6 {
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_REALISE_DECAPITATION);
    };
    @countdown [pub fn (game: &mut impl Host)];
    before {
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0;
    [11122970] MotherBomb_FiringRainbowBeam_1_StartCharge {
        MotherBrain_SetBrainInstrs(animation::BRAIN_CHARGING_RAINBOW_BEAM, game);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_RETRACT_NECK);
        MotherBomb_FiringRainbowBeam_2_RetractNeck(game);
    };
    [11123007] MotherBomb_FiringRainbowBeam_3_Wait {
        QueueSfx2_Max6(0x71, game);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_EXTEND_NECK_DOWN);
        MotherBomb_FiringRainbowBeam_4_ExtendNeckDown(game);
    };
    [11123462] MotherBomb_FiringRainbowBeam_14_DecideNextAction {
        if word!(game.samus_health) as i32 - 400i32 & 0x8000 != 0 {
            MotherBrain_MakeWalkForwards(0xa, (session_slot!(game, MotherBrainBody[E].base.x_pos)).wrapping_add(16), game);
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FINISH_SAMUS_OFF_GET_SAMUS_TO_LOW_HEALTH);
        } else {
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_RAINBOW_BEAM_EXTEND_NECK);
        }
    };
    @countdown [pub fn (game: &mut impl Host)];
    before {
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].function_timer = session_slot!(game, MotherBrainBody[E].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBody[E].function_timer) & 0x8000) != 0;
    [11110532] MotherBrainBody_FakeDeath_Descent_4 {
        MotherBrain_Pal_BeginScreenFlash(game);
        LoadFxEntry(2, game);
        session_slot!(game, MotherBrainBrain[0x40i32].subfunction = function::TUBES_WAIT_AND_SPAWN_TUBES_FALLING_ENEMY_0);
        session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_DESCENT_TRANSITION_MOTHER_BRAIN_PALETTE_TO_GREY);
        session_slot! { game, MotherBrainBody[E]; function_timer = 0, grey_transition = 0 }
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xe, 0x2, plm::CLEAR_CEILING_BLOCK), game);
    };
    [11111819] MotherBrainBody_FakeDeath_Ascent_9_PrepareRise {
        session_slot!(game, MotherBrainBody[E].hdma_object = spawn_rising_hdma(game));
        let E1: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E1].base.properties = (session_slot!(game, MotherBrainBrain[E1].base.properties)) | (0x100u16));
        MotherBrain_SetBrainInstrs(addr_stru_A99C21 as u16, game);
        session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_ASCENT_LOAD_MOTHER_BRAINS_LEGS_TILES);
        session_slot!(game, MotherBrainBody[E].function_timer = 256);
        MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles(game);
    };
    [11111875] MotherBrainBody_FakeDeath_Ascent_11_ContinuePause {
        session_slot!(game, MotherBrainBody[E].base.x_pos = 59);
        session_slot!(game, MotherBrainBody[E].base.y_pos = 279);
        word! {game; reg_BG2HOFS = -27i32 as u16; reg_BG2VOFS = -217i32 as u16; }
        session_slot!(game, MotherBrainBody[E].hitboxes = 7);
        session_slot!(game, MotherBrainBody[E].function = function::FAKE_DEATH_ASCENT_START_MUSIC_AND_EARTHQUAKE);
    };
    [11112106] MotherBrainBody_FakeDeath_Ascent_15_TransitionFromGrey {
        session_slot!(game, MotherBrainBody[E].function_timer = 4);
        let v3: u16 = (session_slot!(game, MotherBrainBody[E].grey_transition)).wrapping_add(1);
        session_slot!(game, MotherBrainBody[E].grey_transition = v3);
        if game.fade(Fade::FakeDeathFromGray, (v3).wrapping_sub(1)) & 1 != 0 {
            let E1: i32 = 0x40;
            session_slot!(game, MotherBrainBrain[E1].brain_palette_enabled = 1);
            session_slot!(game, MotherBrainBody[E].form = 2);
            session_slot!(game, MotherBrainBrain[E1].drool_enabled = 1);
            session_slot! { game, MotherBrainBrain[E1]; lower_motion = 6, upper_motion = 6 }
            session_slot!(game, MotherBrainBrain[E1].neck_speed = 1280);
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_STRETCHING_SHAKE_HEAD_MENACINGLY);
            session_slot!(game, MotherBrainBody[E].function_timer = 23);
        }
    };
    [11112181] MotherBrainBody_2ndphase_16_ShakeHeadMenacingly {
        MotherBrain_SetBrainInstrs(animation::BRAIN_STRETCHING_PHASE_2, game);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_STRETCHING_BRING_HEAD_BACK_UP);
        session_slot!(game, MotherBrainBrain[0x40i32].neck_speed = 64);
        session_slot!(game, MotherBrainBody[E].function_timer = 256);
        MotherBrainBody_2ndphase_17_BringHeadBackUp(game);
    };
    [11112212] MotherBrainBody_2ndphase_17_BringHeadBackUp {
        let E1: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E1].lower_motion = 2);
        session_slot!(game, MotherBrainBrain[E1].upper_motion = 4);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_STRETCHING_FINISH_STRETCHING);
        session_slot!(game, MotherBrainBody[E].function_timer = 64);
        MotherBrainBody_2ndphase_18_FinishStretching(game);
    };
    [11112243] MotherBrainBody_2ndphase_18_FinishStretching {
        session_slot!(game, MotherBrainBrain[0x40i32].breath_enabled = 1);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_THINKING);
    };
    [11121011] MotherBrain_Phase3_Death_10 {
        session_slot!(game, MotherBrainBody[E].grey_transition = 0);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_FADE_TO_GREY);
        session_slot!(game, MotherBrainBody[E].function_timer = 0);
    };
    [11121033] MotherBrain_Phase3_Death_11 {
        let v2: u16 = (session_slot!(game, MotherBrainBody[E].grey_transition)).wrapping_add(1);
        session_slot!(game, MotherBrainBody[E].grey_transition = v2);
        if game.fade(Fade::RealDeathToGray, (v2).wrapping_sub(1)) & 1 != 0 {
            MotherBrain_SetBrainInstrs(animation::BRAIN_CORPSE, game);
            session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_CORPSE_TIPS_OVER);
            session_slot!(game, MotherBrainBody[E].function_timer = 256);
        } else {
            session_slot!(game, MotherBrainBody[E].function_timer = 16);
        }
    };
    [11121080] MotherBrain_Phase3_Death_12 {
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_CORPSE_ROTS_AWAY);
        let E1: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E1].base.properties = (session_slot!(game, MotherBrainBrain[E1].base.properties)) | (0x400u16));
        session_slot!(game, MotherBrainBody[E].hitboxes = 0);
    };
    [11121169] MotherBrain_Phase3_Death_14_20framedelay {
        let E1: i32 = 0x40;
        session_slot! { game, MotherBrainBrain[E1].base; x_pos = 0, y_pos = 0 }
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_DEATH_LOAD_ESCAPE_TIMER_TILES);
    };
    [11122664] MotherBrain_FiringBomb_Fired {
        if MotherBrain_MakeHerStandUp(game) & 1 != 0 {
            MotherBrain_FiringBomb_Finish(game);
        } else {
            session_slot!(game, MotherBrainBody[E].function = function::FIRING_BOMB_STAND_UP);
            MotherBrain_FiringBomb_Standup(game);
        }
    };
    [11122745] MotherBomb_FiringLaser_PositionHeadSlowlyFire {
        let mut v2: i16 = 256;
        let E1: i32 = 0x40;
        if (session_slot!(game, MotherBrainBrain[E1].neck_speed) & 0x8000) != 0 {
            v2 = -256i16;
        }
        session_slot!(game, MotherBrainBrain[E1].neck_speed = v2 as u16);
        MotherBrain_SetBrainInstrs(animation::BRAIN_LASER, game);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FIRING_LASER_FINISH_ATTACK);
        session_slot!(game, MotherBrainBody[E].function_timer = 16);
    };
    [11122787] MotherBomb_FiringLaser_FinishAttack {
        let E1: i32 = 0x40;
        session_slot! { game, MotherBrainBrain[E1]; lower_motion = 4, upper_motion = 4 }
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_THINKING);
        MotherBrain_Body_Phase2_Thinking(game);
    };
    [11124137] MotherBrain_Phase2Cut_2 {
        MotherBrain_SetBrainInstrs(animation::BRAIN_STRETCHING_PHASE_2, game);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FINISH_SAMUS_OFF_CHARGE_FINAL_RAINBOW_BEAM);
        session_slot!(game, MotherBrainBody[E].function_timer = 256);
    };
    [11124161] MotherBrain_Phase2Cut_3 {
        MotherBrain_SetBrainInstrs(animation::BRAIN_CHARGING_RAINBOW_BEAM, game);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_FINISH_SAMUS_OFF_CHARGE_FINAL_RAINBOW_BEAM_LOAD_BABY_METROID);
        MotherBrain_Phase2Cut_4(game);
    };
    [11124205] MotherBrain_Phase2Cut_5 {
        MotherBrain_BodyRainbowBeamPalAnimIndex0(game);
        MotherBrain_SetBrainInstrs(animation::BRAIN_FIRING_RAINBOW_BEAM, game);
        let E1: i32 = 0x40;
        session_slot! { game, MotherBrainBrain[E1]; lower_motion = 6, upper_motion = 6 }
        session_slot!(game, MotherBrainBrain[E1].neck_speed = 1280);
        QueueSfx2_Max6(0x71, game);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_CUTSCENE_WAIT);
    };
    [11124605] MotherBrain_DrainedByBabyMetroid_6 {
        session_slot!(game, MotherBrainBody[E].grey_transition = 0);
        session_slot!(game, MotherBrainBody[E].function = function::DRAINED_BY_BABY_METROID_TRANSITION_TO_GREY);
        session_slot!(game, MotherBrainBody[E].function_timer = 16);
        MotherBrain_DrainedByBabyMetroid_7(game);
    };
    [11124838] MotherBrain_Phase2_Revive_1 {
        let E1: i32 = 0x40;
        session_slot! { game, MotherBrainBrain[E1]; breath_enabled = 1, drool_enabled = 1 }
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_REVIVE_SELF_PREPARE_TRANSITION_FROM_GREY);
        session_slot!(game, MotherBrainBody[E].function_timer = 224);
        MotherBrain_Phase2_Revive_2(game);
    };
    [11124964] MotherBrain_Phase2_Revive_5 {
        MotherBrain_SetBrainInstrs(animation::BRAIN_STRETCHING_PHASE_3, game);
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_REVIVE_SELF_WALK_UP_TO_BABY_METROID);
        session_slot!(game, MotherBrainBody[E].function_timer = 128);
        MotherBrain_Phase2_Revive_6(game);
    };
    [11125122] MotherBrain_Phase2_MurderBabyMetroid_2 {
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_2_MURDER_BABY_METROID_ATTACK_BABY_METROID);
    };
    [11125232] MotherBrain_Phase3_Recover_SetupForFight {
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_FIGHTING_MAIN);
        let E1: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E1].neck_function = neck::NORMAL);
        session_slot!(game, MotherBrainBrain[E1].walk_function = function::WALK_TRY_TO_INCH_FORWARD);
        MotherBrain_Phase3_Fighting_Main(game);
    };
    [11125326] MotherBrain_Phase3_Fighting_Cooldown {
        session_slot!(game, MotherBrainBody[E].function = function::PHASE_3_FIGHTING_MAIN);
    };
    @external {
        11110369 => [{
                    Unreachable(game);
                }];
        11124250 => [()];
        11125158 => [()];
        11111140 => [()]; 11125133 => [()];
    }
}
/// Run Mother Brain's body or brain function `ea` on the lease when it needs no more than the
/// state, and on the game otherwise.
fn run_mother_brain_function<H: Host, L: Lease<Game = H> + ?Sized>(ea: u32, lease: &mut L) {
    match ea {
        11121468 => MotherBrain_Phase3_Death_21_KeepEarthquakeGoing(lease),
        11122630 => MotherBrain_FiringBomb_Crouch(lease),
        11122680 => MotherBrain_FiringBomb_Standup(lease),
        11122702 => MotherBomb_FiringLaser_PositionHead(lease),
        11122813 => MotherBomb_FiringDeathBeam(lease),
        11122923 => MotherBomb_FiringRainbowBeam_0(lease),
        11124545 => MotherBrain_DrainedByBabyMetroid_4(lease),
        11125063 => MotherBrain_Phase2_Revive_8(lease),
        11125084 => MotherBrain_Phase2_MurderBabyMetroid_1(lease),
        11125134 => MotherBrain_Phase2_PrepareForFinalBabyMetroid(lease),
        11125146 => MotherBrain_Phase2_ExecuteFinalkBabyMetroid(lease),
        11123421 => MotherBomb_FiringRainbowBeam_13_LowerHead(lease),
        11124566 => MotherBrain_DrainedByBabyMetroid_5(lease),
        11112085 => MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(lease),
        11122181 => MotherBrain_Body_Phase2_Thinking(lease),
        11124825 => MotherBrain_Phase2_Revive_0(lease),
        11124987 => MotherBrain_Phase2_Revive_6(lease),
        11125022 => MotherBrain_Phase2_Revive_7(lease),
        11125199 => MotherBrain_Phase3_Recover_MakeDistance(lease),
        11124250 | 11125158 | 11111140 => {}
        _ => lease.foreign(|game| CallMotherBrainFunc(ea, game)),
    }
}

pub fn MotherBrainsBody_UnpauseHook(game: &mut impl Host) -> CoroutineRet {
    let E: i32 = 0;
    if session_slot!(game, MotherBrainBody[E].beam_sound_playing) != 0 {
        QueueSfx1_Max6(0x40, game);
    }
    if (session_slot!(game, MotherBrainBody[E].base.extra_properties) & 4) != 0 {
        word! {game; enemy_bg2_tilemap_size = 2048; nmi_flag_bg2_enemy_vram_transfer = 1; }
    }
    kCoroutineNone as CoroutineRet
}
pub fn MotherBrain_DrawBrainNeck_EnemyGfxDrawHook(game: &mut impl Host) {
    {
        MotherBrain_DrawBrain(game);
        MotherBrain_DrawNeck(game);
    }
}
pub fn MotherBrainsBody_FirstPhase_DoubleRet(game: &mut impl Host) {
    {
        if CheckEventHappened(2, game) != 0 {
            let E1: i32 = 0x40;
            session_slot!(
                game,
                MotherBrainBrain[E1].shake_timer = word!(game.earthquake_timer)
            );
            if word!(game.samus_x_pos) as i32 - 236i32 & 0x8000 != 0
                && session_slot!(game, MotherBrainBrain[E1].base.health) == 0
            {
                let E: i32 = 0;
                session_slot! { game, MotherBrainBody[E]; delete_turrets = 1, form = 1 }
                game.disable_minimap_for_boss();
                QueueMusic_Delayed8(6, game);
                game.seal_wall();
            }
        }
        MotherBrain_SamusCollDetect(game);
    }
}
/// The falling tubes run in `crate::actors::mother_brain_tubes`.
pub fn MotherBrainsTubesFalling_Main(k: u16, game: &mut impl Host) {
    crate::actors::mother_brain_tubes::main_ai(game, k);
}
crate::dispatch::callback_dispatch! {
    pub fn CallMotherBrainInstr(ea: u32, k: u16, game: &mut impl Host) -> u16 => ea;
    call (k, game); default Unreachable(game) as u16;
    @body checked;
    @functions [pub fn (k: u16, game: &mut impl Host) -> u16];
    [11115279] MotherBrain_Instr_Goto [native_record(0xa90000 | u32::from(k), game).word(0)] { };
    [11115284] MotherBrain_Instr_EnableNeckMovementGoto [native_record(0xa90000 | u32::from(k), game).word(0)] {
        session_slot!(game, MotherBrainBrain[0x40].neck_moving = 1);
    };
    @functions [pub fn (k: u16, game: &mut (impl State + ?Sized)) -> u16];
    [11115296]
    MotherBrain_Instr_DisableNeckMovement [k] { session_slot!(game, MotherBrainBrain[0x40].neck_moving = 0); };
    @functions [pub fn (k: u16, game: &mut impl Host) -> u16];
    @prefix { let v2 = native_record(0xa90000 | u32::from(k), game); };
    [11115304] MotherBrain_Instr_QueueSfx2 [(k).wrapping_add(2)] {  QueueSfx2_Max6(v2.word(0), game); };
    [11115314] MotherBrain_Instr_QueueSfx3 [(k).wrapping_add(2)] {  QueueSfx3_Max6(v2.word(0), game); };
    @functions [pub fn (k: u16, game: &mut impl Host) -> u16];
    [11115324] MotherBrain_Instr_SpawnDroolEproj [k] {
        let E: i32 = 0x40;
        if session_slot!(game, MotherBrainBrain[E].drool_enabled) != 0 {
            let mut v2: u16 = (session_slot!(game, MotherBrainBrain[E].drool_parameter)).wrapping_add(1);
            if session_slot!(game, MotherBrainBrain[E].drool_parameter) as i32 - 5i32 & 0x8000 == 0 {
                v2 = 0;
            }
            session_slot!(game, MotherBrainBrain[E].drool_parameter = v2);
            let v3: u16 = (if session_slot!(game, MotherBrainBrain[E].neck_speed) as i32 - 128i32 & 0x8000 != 0 {
                addr_kEproj_MotherBrainDrool as i32
            } else {
                addr_kEproj_MotherBrainDyingDrool as i32
            }) as u16;
            SpawnEprojWithRoomGfx(v3, session_slot!(game, MotherBrainBrain[E].drool_parameter), game);
        }
    };
    [11115373] MotherBrain_Instr_SpawnPurpleBreath [k] {
        SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainPurpleBreathBig as u16, 0x9b6d, game);
    };
    @functions [pub fn (k: u16, game: &mut (impl State + ?Sized)) -> u16];
    [11115383]
    MotherBrain_Instr_SetMainShakeTimer50 [k] { session_slot!(game, MotherBrainBrain[0x40].shake_timer = 50); };
    @functions [pub fn (_k: u16, game: &mut (impl State + ?Sized)) -> u16];
    [11115621] MotherBrain_Instr_GotoEitherOr [result] {
        let result: u16 = addr_stru_A99C5F as u16;
        if (word!(game.random_number) & 0xfff) >= 0xfe0 {
            return addr_stru_A99C47 as u16;
        }
    };
    @functions [pub fn (k: u16, game: &mut (impl State + ?Sized)) -> u16];
    [11115693] MotherBrain_Instr_MaybeGoto [k] {
        if word!(game.random_number) < 0xf000 {
            return addr_stru_A99C9F as u16;
        }
    };
    [11115789] MotherBrain_Instr_MaybeGoto2 [k] {
        if (word!(game.random_number) & 0xfff) < 0xec0 {
            return MotherBrain_Instr_Goto2(k);
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Host) -> u16];
    [11116023] MotherBrain_Instr_QueueBabyMetroidAttackSfx [k] {
        if session_slot!(game, MotherBrainBody[0].ring_volleys) != 11 {
            QueueSfx2_Max6(g_word_A99E0F[0i32 as usize], game);
        }
    };
    [11116073] MotherBrain_Instr_SpawnBlueRingEproj [k] {
        let E: i32 = 0;
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainBlueRingLasers as u16,
            session_slot!(game, MotherBrainBody[E].ring_target_angle),
            game,
        );
    };
    @functions [pub fn (k: u16, game: &mut (impl State + ?Sized)) -> u16];
    [11116087] MotherBrain_Instr_AimRingsAtBabyMetroid [k] {
        let E: i32 = 0x40;
        let E1: i32 = session_slot!(game, MotherBrainBrain[E].baby_metroid) as i32;
        MotherBrain_Instr_AimRings(
            (session_slot!(game, MotherBrainBody[E1].base.x_pos) as i32 - session_slot!(game, MotherBrainBrain[E].base.x_pos) as i32 - 10i32) as u16,
            (session_slot!(game, MotherBrainBody[E1].base.y_pos) as i32 - session_slot!(game, MotherBrainBrain[E].base.y_pos) as i32 - 16i32) as u16,
            game,
        );
    };
    [11116123] MotherBrain_Instr_AimRingsAtSamus [k] {
        let E: i32 = 0x40;
        MotherBrain_Instr_AimRings(
            (word!(game.samus_x_pos) as i32 - session_slot!(game, MotherBrainBrain[E].base.x_pos) as i32 - 10i32) as u16,
            (word!(game.samus_y_pos) as i32 - session_slot!(game, MotherBrainBrain[E].base.y_pos) as i32 - 16i32) as u16,
            game,
        );
    };
    [11116195] MotherBrain_Instr_IncrBabyMetroidAttackCtr [k] {
        let E: i32 = 0;
        let mut v3: i16 = ((session_slot!(game, MotherBrainBody[E].ring_volleys)) as i16).wrapping_add(1);
        if (v3 as u16) >= 0xc {
            v3 = 12;
        }
        session_slot!(game, MotherBrainBody[E].ring_volleys = v3 as u16);
    };
    [11116213]
    MotherBrain_Instr_SetBabyMetroidAttackCtr0 [k] { session_slot!(game, MotherBrainBody[0].ring_volleys = 0); };
    @functions [pub fn (k: u16, game: &mut impl Host) -> u16];
    [11116221] MotherBrain_Instr_SpawnBombEproj [(k).wrapping_add(2)] {
        let v2 = native_record(0xa90000 | u32::from(k), game);
        SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainBomb as u16, v2.word(0), game);
    };
    [11116358] MotherBrain_Instr_SpawnLaserEproj [k] {
        let E: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E].neck_moving = 0);
        game.set_projectile_spawn_r22(1);
        crate::actors::host::BossEffects::set_projectile_spawn_point(game, ((session_slot!(game, MotherBrainBrain[E].base.x_pos)).wrapping_add(16)), ((session_slot!(game, MotherBrainBrain[E].base.y_pos)).wrapping_add(4)));
        SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 1, game);
    };
    [11116420] MotherBrain_Instr_SpawnRainbowEproj [k] {
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainRainbowBeamCharging as u16,
            0x9f84,
            game,
        );
    };
    [11116430] MotherBrain_Instr_SetupFxForRainbowBeam [k] {
        session_slot!(game, MotherBrainBrain[0x40].breath_enabled = 0);
        MotherBrain_SetupBrainPalForLaser(game);
        QueueSfx2_Max6(0x7f, game);
    };
    @external {
        11115809 => [MotherBrain_Instr_Goto2(k)];
    }
}
pub fn MotherBrain_Func_1_DoubleRet(game: &mut impl Host) -> i32 {
    {
        let E: i32 = 0;
        let mut pc: u16 = session_slot!(game, MotherBrainBody[E].brain_instructions);
        if pc & 0x8000 == 0 {
            return -1i32;
        }
        let mut p = native_record(0xa90000 | u32::from(pc), game);
        if word!(game.time_is_frozen_flag) != 0 {
            return p.word(2) as i32;
        }
        let mut v5: u16 = p.word(0);
        let mut execute_instruction: bool = (v5 & 0x8000 != 0)
            || ({
                if ((v5) as i16).wrapping_sub(
                    (session_slot!(game, MotherBrainBody[E].brain_instruction_timer)) as i16,
                ) >= 0
                {
                    session_slot!(
                        game,
                        MotherBrainBody[E].brain_instruction_timer =
                            session_slot!(game, MotherBrainBody[E].brain_instruction_timer)
                                .wrapping_add(1)
                    );
                    return p.word(2) as i32;
                }
                pc = (pc).wrapping_add(4);
                false
            });
        loop {
            if execute_instruction {
                pc =
                    CallMotherBrainInstr(((v5) as u32) | (0xa90000u32), (pc).wrapping_add(2), game);
                execute_instruction = false;
            } else {
                p = native_record(0xa90000 | u32::from(pc), game);
                v5 = p.word(0);
                if v5 & 0x8000 == 0 {
                    break;
                } else {
                    execute_instruction = true;
                }
            }
        }
        session_slot!(game, MotherBrainBody[E].brain_instruction_timer = 1);
        session_slot!(game, MotherBrainBody[E].brain_instructions = pc);
        p.word(2) as i32
    }
}
pub fn MotherBrain_DrawNeck(game: &mut impl Host) {
    {
        if (session_slot!(game, MotherBrainBody[0i32].base.properties) as i32
            & kEnemyProps_Invisible as i32)
            == 0
        {
            let E: i32 = 0x40;
            MotherBrain_DrawNeckSegment(
                session_slot!(game, MotherBrainBrain[E].segment_5_x),
                session_slot!(game, MotherBrainBrain[E].segment_5_y),
                game,
            );
            MotherBrain_DrawNeckSegment(
                session_slot!(game, MotherBrainBrain[E].segment_4_x),
                session_slot!(game, MotherBrainBrain[E].segment_4_y),
                game,
            );
            MotherBrain_DrawNeckSegment(
                session_slot!(game, MotherBrainBrain[E].segment_3_x),
                session_slot!(game, MotherBrainBrain[E].segment_3_y),
                game,
            );
            MotherBrain_DrawNeckSegment(
                session_slot!(game, MotherBrainBrain[E].segment_2_x),
                session_slot!(game, MotherBrainBrain[E].segment_2_y),
                game,
            );
            MotherBrain_DrawNeckSegment(
                session_slot!(game, MotherBrainBrain[E].segment_1_x),
                session_slot!(game, MotherBrainBrain[E].segment_1_y),
                game,
            );
        }
    }
}
pub fn MotherBrain_DrawNeckSegment(x: u16, y: u16, game: &mut impl Host) {
    {
        let v0: i32 =
            (session_slot!(game, MotherBrainBrain[0x40i32].base.flash_timer) as i32 & 6) >> 1;
        MotherBrain_AddSpritemapToOam(
            addr_kMotherBrain_Sprmap_A694 as u16,
            (x).wrapping_add((g_word_A993BB[v0 as usize]) as u16),
            (y).wrapping_add((g_word_A993C3[v0 as usize]) as u16),
            session_slot!(game, MotherBrainBody[0i32].neck_palette),
            game,
        );
    }
}
pub fn MotherBrain_AddSpritemapToOam(j: u16, r18: u16, r20: u16, r22: u16, game: &mut impl Host) {
    let source = native_record(0xa90000 | u32::from(j), game);
    let mut n: i32 = source.word(0) as i32;
    let mut offset: usize = 2;
    let mut idx: i32 = word!(game.oam_next_ptr) as i32;
    let layer1 = crate::camera::current(game).layer1;
    do_until! { n == 0 ;
        let y: i16 = (r20 as i32 + source.byte(offset + 2) as i8 as i32 - layer1[1] as i32) as i16;
        if y >= 0 {
            let oam: usize = usize::try_from(0x370i32 + idx).expect("OAM entry outside RAM");
            let x: u16 =
                (r18 as i32 + source.word(offset) as i32 - layer1[0] as i32) as u16;
            game.ram().write_byte(oam, x as u8);
            game.ram().write_byte(oam + 1, y as u8);
            set_native_word(oam + 2, r22 | source.word(offset + 3), game);
            word!(bits_i32, game.oam_ext[(idx >> 5)] |= ((x as i32 & 0x100) >> 8 | ((source.word(offset) as i16) < 0) as i32 * 2)
                    << 2 * (idx >> 2 & 7));
            idx = idx + 4 & 0x1ff;
        }
        offset += 5;
        n -= 1;
    }
    word!(game.oam_next_ptr = idx as u16);
}
pub fn MotherBrain_CalculateRainbowBeamHdma(game: &mut impl Host) {
    {
        game.calculate_rainbow_window();
    }
}
pub fn MotherBrain_Instr_Goto2(_k: u16) -> u16 {
    addr_stru_A99CD1 as u16
}
pub fn MotherBrain_Instr_AimRings(x: u16, y: u16, game: &mut (impl State + ?Sized)) {
    {
        let mut v0: u16 =
            -(crate::actors::queries::angle_from_delta(x, y) as i32 + 0x80i32) as u8 as u16;
        let clamp_ring_angle: bool = if v0 as i32 - 16i32 & 0x80 == 0 {
            !((v0 as u8) < 0x48)
        } else {
            (v0 as i32 + 64 & 0x80 != 0)
                || ({
                    {
                        v0 = u16::from_ne_bytes([16, v0.to_ne_bytes()[1]]);
                    }
                    false
                })
        };
        if clamp_ring_angle {
            {
                v0 = u16::from_ne_bytes([72, v0.to_ne_bytes()[1]]);
            }
        }
        session_slot!(game, MotherBrainBody[0i32].ring_target_angle = v0);
    }
}
crate::dispatch::forward_calls! { pub fn (game: &mut impl Host);
    MotherBrain_GenerateSmokyExplosions =>
        generate_body_explosions(0x10, addr_word_A9B10F as u16, 2, game);
    MotherBrain_GenerateMixedExplosions =>
        generate_body_explosions(8, addr_word_A9B109 as u16, 4, game);
}
pub fn MotherBrain_ExplodeEscapeDoor(game: &mut impl Host) {
    {
        for v0 in 0u16..8 {
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainExplodedEscapeDoor as u16, v0, game);
        }
    }
}
pub fn MotherBrain_SamusCollDetect(game: &mut impl Host) {
    {
        let E0: i32 = 0;
        let mut R26: u16 = session_slot!(game, MotherBrainBody[E0].hitboxes);
        let mut v1: i8 = ((R26) as i8) & (1i8);
        R26 = R26 >> 1;
        if v1 == 0
            || MotherBrain_SamusCollDetectPart(
                addr_word_A9B427 as u16,
                session_slot!(game, MotherBrainBody[E0].base.x_pos),
                session_slot!(game, MotherBrainBody[E0].base.y_pos),
                game,
            ) & 1
                == 0
        {
            let E: i32 = 0x40;
            v1 = ((R26) as i8) & (1i8);
            R26 = R26 >> 1;
            if v1 == 0
                || MotherBrain_SamusCollDetectPart(
                    BRAIN_HITBOX,
                    session_slot!(game, MotherBrainBrain[E].base.x_pos),
                    session_slot!(game, MotherBrainBrain[E].base.y_pos),
                    game,
                ) & 1
                    == 0
            {
                v1 = ((R26) as i8) & (1i8);
                R26 = R26 >> 1;
                if v1 != 0
                    && MotherBrain_SamusCollDetectPart(
                        NECK_HITBOX,
                        session_slot!(game, MotherBrainBrain[E].segment_2_x),
                        session_slot!(game, MotherBrainBrain[E].segment_2_y),
                        game,
                    ) & 1
                        == 0
                    && MotherBrain_SamusCollDetectPart(
                        NECK_HITBOX,
                        session_slot!(game, MotherBrainBrain[E].segment_3_x),
                        session_slot!(game, MotherBrainBrain[E].segment_3_y),
                        game,
                    ) & 1
                        == 0
                {
                    MotherBrain_SamusCollDetectPart(
                        NECK_HITBOX,
                        session_slot!(game, MotherBrainBrain[E].segment_4_x),
                        session_slot!(game, MotherBrainBrain[E].segment_4_y),
                        game,
                    );
                }
            }
        }
    }
}
pub fn MotherBrain_SamusCollDetectPart(k: u16, r18: u16, r20: u16, game: &mut impl Host) -> u8 {
    use crate::actors::mother_brain_fight::Contact;
    match samus_contact(k, r18, r20, game) {
        Contact::None => 0,
        Contact::Pushed => 1,
        Contact::Hurt => {
            {
                MotherBrain_HurtSamus(game)
            };
            1
        }
    }
}
pub fn MotherBrain_Phase23_ShotReaction(game: &mut impl Host) {
    {
        let mut v1: i16 = 0;
        let E: i32 = 0;
        if session_slot!(game, MotherBrainBody[E].form) == 4
            && MotherBrain_DetermineShotReactionType(game) == 2
        {
            MotherBrain_Phase3_BeamShotReaction(game);
        } else {
            if MotherBrain_DetermineShotReactionType(game) == 1 || {
                v1 = ((session_slot!(game, MotherBrainBody[E].walk_counter)) as i16)
                    .wrapping_sub(256);
                v1 < 0
            } {
                v1 = 0;
            }
            session_slot!(game, MotherBrainBody[E].walk_counter = v1 as u16);
        };
    }
}
pub fn MotherBrain_DetermineShotReactionType(game: &mut impl Host) -> u16 {
    g_byte_A9B5A1[(game
        .ram()
        .read_byte(0xc18 + 2 * word!(game.collision_detection_index) as usize + 1)
        as i32
        & 7) as usize] as u16
}
pub fn MotherBrain_HurtSamus(game: &mut impl Host) {
    {
        Ridley_Func_98(game);
        word!(game.samus_invincibility_timer = 96);
        word!(game.samus_knockback_timer = 5);
        word!(
            game.knockback_x_dir = (((word!(game.samus_x_pos)) as i16).wrapping_sub(
                (session_slot!(
                    game,
                    MotherBrainBody[word!(game.cur_enemy_index) as i32]
                        .base
                        .x_pos
                )) as i16,
            ) >= 0) as u16
        );
    }
}
pub fn MotherBrain_Phase2_DecideAttackStrategy_DoubleRet(game: &mut impl Host) -> u8 {
    {
        if g_word_A9B72C[game.ram().read_byte(0xa1f) as usize] != 0 {
            if word!(game.random_number) as u8 as i32 - 128i32 & 0x8000 != 0 {
                0
            } else {
                let E1: i32 = 0x40;
                if session_slot!(game, MotherBrainBrain[E1].bombs) as i32 - 1i32 & 0x8000 != 0 {
                    session_slot!(
                        game,
                        MotherBrainBody[0i32].function = function::FIRING_BOMB_DECIDE_ON_WALKING
                    );
                    MotherBrain_FiringBomb_DecideOnWalking(game);
                    return 1;
                }
                0
            }
        } else {
            if word!(game.random_number) as u8 as i32 - 128i32 & 0x8000 != 0 {
                session_slot!(
                    game,
                    MotherBrainBody[0i32].function =
                        function::PHASE_2_FIRING_LASER_POSITION_HEAD_QUICKLY
                );
                MotherBomb_FiringLaser_PositionHead(game);
            } else {
                MotherBrain_SetBrainInstrs(animation::BRAIN_FOUR_BLUE_RINGS_PHASE_2, game);
            }
            1
        }
    }
}
pub fn MotherBrain_FiringBomb_DecideOnCrouching(game: &mut impl Host) {
    {
        if game.next_random() >= 0x8000 {
            session_slot!(
                game,
                MotherBrainBody[0i32].function = function::FIRING_BOMB_CROUCH
            );
            MotherBrain_FiringBomb_Crouch(game);
        } else {
            MotherBrain_B7CB(game);
        };
    }
}
pub fn MotherBrain_B7CB(game: &mut (impl State + ?Sized)) {
    {
        let mut v0: u16 = animation::BRAIN_BOMB_PHASE_2;
        let E: i32 = 0;
        if session_slot!(game, MotherBrainBody[E].corpse_state) != 0 {
            v0 = animation::BRAIN_BOMB_PHASE_3;
        }
        MotherBrain_SetBrainInstrs(v0, game);
        session_slot!(
            game,
            MotherBrainBody[E].function = function::FIRING_BOMB_FIRED_BOMB
        );
        session_slot!(game, MotherBrainBody[E].function_timer = 44);
    }
}
pub fn MotherBrain_FiringBomb_Finish(game: &mut (impl State + ?Sized)) {
    let mut v0: u16 = function::PHASE_2_THINKING;
    let E: i32 = 0;
    if session_slot!(game, MotherBrainBody[E].corpse_state) != 0 {
        v0 = function::PHASE_2_MURDER_BABY_METROID_ATTACK_BABY_METROID;
    }
    session_slot!(game, MotherBrainBody[E].function = v0);
}
pub fn MotherBomb_FiringDeathBeam_0(game: &mut (impl State + ?Sized)) {
    {
        if MotherBrain_MakeWalkBackwards(0x28, 8, game) & 1 != 0 {
            let E1: i32 = 0x40;
            session_slot!(game, MotherBrainBrain[E1].lower_motion = 8);
            session_slot!(game, MotherBrainBrain[E1].upper_motion = 6);
            let E: i32 = 0;
            session_slot!(
                game,
                MotherBrainBody[E].hand_beam_phase =
                    session_slot!(game, MotherBrainBody[E].hand_beam_phase).wrapping_add(1)
            );
        }
    }
}
pub fn MotherBomb_FiringDeathBeam_1(game: &mut (impl State + ?Sized)) {
    {
        if session_slot!(game, MotherBrainBrain[0x40i32].bombs) == 0 {
            MotherBrain_SetBodyInstrs(animation::BODY_HAND_BEAM_MODE, game);
            let E: i32 = 0;
            session_slot!(
                game,
                MotherBrainBody[E].hand_beam_phase =
                    session_slot!(game, MotherBrainBody[E].hand_beam_phase).wrapping_add(1)
            );
        }
    }
}
pub fn MotherBomb_FiringDeathBeam_2(_game: &mut (impl State + ?Sized)) {}
pub fn MotherBomb_FiringDeathBeam_3(game: &mut (impl State + ?Sized)) {
    {
        MotherBrain_SetBrainInstrs(animation::BRAIN_NEUTRAL_PHASE_2, game);
        let E1: i32 = 0x40;
        session_slot!(game, MotherBrainBrain[E1].lower_motion = 2);
        session_slot!(game, MotherBrainBrain[E1].upper_motion = 4);
        let E: i32 = 0;
        session_slot!(game, MotherBrainBody[E].hand_beam_phase = 0);
        session_slot!(
            game,
            MotherBrainBody[E].function = function::PHASE_2_THINKING
        );
    }
}
pub fn MotherBrain_WalkBackwardsSlowlyAndRetractHead(a: u16, game: &mut impl Host) -> u8 {
    {
        let rv: u8 = MotherBrain_MakeWalkBackwards(a, 0xa, game);
        if rv != 0 {
            MotherBrain_RetractHead(game);
        }
        rv
    }
}
pub fn MotherBrain_MoveSamusTowardsWallDueToBeam(game: &mut impl Host) -> u8 {
    {
        let v0: u8 = move_samus_horizontally_towards_wall(0x1000, game);
        if v0 == 0 {
            let v1: u16 = Math_MultByCos(
                0x1000,
                session_slot!(game, MotherBrainBody[0i32].beam_angle),
            );
            move_samus_vertically_towards_ceiling_floor(v1, game);
        }
        v0
    }
}
pub fn MotherBrain_MoveSamusTowardsMiddleOfWall(game: &mut impl Host) {
    {
        let mut v0: u16 = 64;
        if (124i16).wrapping_sub((word!(game.samus_y_pos)) as i16) < 0 {
            v0 = -64i32 as u16;
        }
        move_samus_vertically_towards_ceiling_floor(v0, game);
    }
}
pub fn MotherBrain_MoveSamusForFallingAfterBeam(game: &mut impl Host) -> u8 {
    {
        let E: i32 = 0;
        let mut v1: i16 =
            ((session_slot!(game, MotherBrainBody[E].cooldown)) as i16).wrapping_add(2);
        if v1 >= 0 {
            v1 = 0;
        }
        session_slot!(game, MotherBrainBody[E].cooldown = v1 as u16);
        move_samus_horizontally_towards_wall(v1 as u16, game);
        let v2: u16 = (session_slot!(game, MotherBrainBody[E].explosion_timer)).wrapping_add(24);
        session_slot!(game, MotherBrainBody[E].explosion_timer = v2);
        (move_samus_vertically_towards_ceiling_floor(v2, game)) & (1u8)
    }
}
pub fn MotherBrain_BodyRainbowBeamPalAnimIndex0(game: &mut (impl State + ?Sized)) {
    session_slot!(game, MotherBrainBrain[0x40i32].beam_palette_index = 0);
}
pub fn MotherBrain_WriteDefaultPalette(game: &mut impl Host) {
    {
        copy_rom_colors_to_palette(
            0x82,
            0xa9,
            (addr_kMotherBrainsBrain_Palette as i32 + 2) as u16,
            0xf,
            game,
        );
        copy_rom_colors_to_palette(
            0x122,
            0xa9,
            (addr_kMotherBrainsBrain_Palette as i32 + 2) as u16,
            0xf,
            game,
        );
        copy_rom_colors_to_palette(
            0x162,
            0xa9,
            (addr_kMotherBrainPalette_0 as i32 + 2) as u16,
            0xf,
            game,
        );
    }
}
pub fn MotherBrain_WritePhase2DeathPalette(game: &mut impl Host) {
    {
        write_mother_brain_palette(
            mother_brain_tables::rainbow_beam_palette(game, 6).colors,
            game,
        );
    }
}
pub fn MotherBrain_SpawnBabyMetroidInCutscene(game: &mut impl Host) {
    session_slot!(
        game,
        MotherBrainBrain[0x40i32].baby_metroid = {
            let (bank, population) = (0xa9, addr_stru_A9BE28 as u16);
            game.spawn_legacy_population(bank, population)
        }
    );
}
crate::dispatch::callback_dispatch! {
    pub fn CallMotherBrainPainfulWalk(ea: u32, game: &mut impl Host) => ea;
    call (game); default { Unreachable(game); };
    @body checked;
    @countdown [pub fn (game: &mut (impl State + ?Sized))];
    before {
        let E: i32 = 0x40;
        let v1: u16 = (session_slot!(game, MotherBrainBrain[E].painful_walk_timer)).wrapping_sub(1);
        session_slot!(game, MotherBrainBrain[E].painful_walk_timer = v1);
    }
    when v1 == 0;
    [11124712] MotherBrain_PainfulWalkingForwards {
        session_slot!(game, MotherBrainBrain[E].painful_walk_stage = session_slot!(game, MotherBrainBrain[E].painful_walk_stage).wrapping_add(1));
        session_slot!(game, MotherBrainBrain[E].painful_walk_function = function::PAINFUL_WALK_BACKWARDS);
    };
    [11124764] MotherBrain_PainfulWalkingBackwards {
        session_slot!(game, MotherBrainBrain[E].painful_walk_stage = session_slot!(game, MotherBrainBrain[E].painful_walk_stage).wrapping_add(1));
        session_slot!(game, MotherBrainBrain[E].painful_walk_function = function::PAINFUL_WALK_FORWARDS);
    };
    @external {
        11124688 => MotherBrain_PainfulWalkForwards;
        11124740 => MotherBrain_PainfulWalkBackwards;
    }
}
pub fn MotherBrain_MaybeStandupOrLeanDown(game: &mut (impl State + ?Sized)) {
    {
        let mbn_var_02: u16 = session_slot!(game, MotherBrainBody[0i32].pose);
        if mbn_var_02 != 0 {
            if mbn_var_02 == 6 && (word!(game.random_number) as u8) >= 0xc0 {
                MotherBrain_MakeHerStandUp(game);
            }
        } else if (word!(game.random_number) as u8) >= 0xc0 {
            MotherBrain_MakeHerLeanDown(game);
        }
    }
}
crate::dispatch::address_dispatch! {
    pub fn CallMotherBrainWallkFunc1A(ea: u32, game: &mut impl Host) => ea;
        call (game); default { Unreachable(game); };
    {
        11125354 => MotherBrain_Phase3_Walk_TryToInchForward;
        11125427 => MotherBrain_Phase3_Walk_RetreatQuickly;
        11125458 => MotherBrain_Phase3_Walk_RetreatSlowly;
    }
}
pub fn MotherBrain_Phase3_WalkHandler(game: &mut impl Host) {
    {
        if session_slot!(game, MotherBrainBody[0i32].pose) == 0 {
            CallMotherBrainWallkFunc1A(
                ((session_slot!(game, MotherBrainBrain[0x40i32].walk_function)) as u32)
                    | (0xa90000u32),
                game,
            );
        }
    }
}
pub fn MotherBrain_SetToTryToInchForward(a: u16, game: &mut (impl State + ?Sized)) {
    let E: i32 = 0;
    session_slot!(game, MotherBrainBody[E].walk_counter = a);
    let E1: i32 = 0x40;
    session_slot!(
        game,
        MotherBrainBrain[E1].walk_function = function::WALK_TRY_TO_INCH_FORWARD
    );
    session_slot!(
        game,
        MotherBrainBrain[E1].target_x =
            (session_slot!(game, MotherBrainBody[E].base.x_pos)).wrapping_add(1)
    );
}
crate::dispatch::callback_dispatch! {
    pub fn CallMotherBrainNeckFunc(ea: u32, game: &mut impl Host) => ea;
    call (game); default { Unreachable(game); };
    @body checked;
    @functions [pub fn (game: &mut impl Host)];
    @prefix { let E: i32 = 0x40; };
    MotherBrain_SpawnDustCloudsForAscent [] {
        session_slot!(game, MotherBrainBrain[E].subfunction_timer = session_slot!(game, MotherBrainBrain[E].subfunction_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBrain[E].subfunction_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBrain[E].subfunction_timer = 7);
        }
        crate::actors::host::BossEffects::set_projectile_spawn_point(game, g_word_A98F7F[session_slot!(game, MotherBrainBrain[E].subfunction_timer) as usize], 212);
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as u16,
            g_byte_A98F7D[(((word!(game.random_number)) & (0x100u16)) >> 8) as usize] as u16,
            game,
        );
        QueueSfx2_Max3(0x29, game);
    };
    MotherBrain_DrawBrain [] {
        if session_slot!(game, MotherBrainBrain[E].breath_enabled) != 0 && session_slot!(game, MotherBrainBrain[E].breath_active) == 0 && (word!(game.random_number) & 0x8000) == 0 {
            SpawnEprojWithRoomGfx(
                addr_kEproj_MotherBrainPurpleBreathSmall as u16,
                word!(game.random_number),
                game,
            );
        }
        let tt: i32 = MotherBrain_Func_1_DoubleRet(game);
        if tt < 0 {
            return;
        }
        let v2: u16 = tt as u16;
        let mut mbn_var_0D: u16 = session_slot!(game, MotherBrainBody[0i32].brain_palette);
        if session_slot!(game, MotherBrainBrain[E].base.flash_timer) & 1 != 0 {
            mbn_var_0D = 0;
        }
        let r22: u16 = mbn_var_0D;
        let mbn_var_00: u16 = session_slot!(game, MotherBrainBrain[E].shake_timer);
        let mut flash_timer: u16 = 0;
        if mbn_var_00 != 0 {
            flash_timer = (mbn_var_00).wrapping_sub(1);
            session_slot!(game, MotherBrainBrain[E].shake_timer = flash_timer);
        } else {
            flash_timer = session_slot!(game, MotherBrainBrain[E].base.flash_timer);
            if flash_timer == 0 {
                flash_timer = session_slot!(game, MotherBrainBrain[E].base.shake_timer);
            }
        }
        let v6: i32 = (flash_timer as i32 & 6) >> 1;
        let r18: u16 = (session_slot!(game, MotherBrainBrain[E].base.x_pos)).wrapping_add((g_word_A993BB[v6 as usize]) as u16);
        if ((r18 as i32 + 32 - crate::camera::current(game).layer1[0] as i32) as i16) >= 0 {
            let r20: u16 = (session_slot!(game, MotherBrainBrain[E].base.y_pos)).wrapping_add((g_word_A993C3[v6 as usize]) as u16);
            MotherBrain_AddSpritemapToOam(v2, r18, r20, r22, game);
        }
    };
    MotherBrain_GenerateEscapeDoorExploding [] {
        let mut v1: i16 = ((session_slot!(game, MotherBrainBrain[E].subfunction)) as i16).wrapping_sub(1);
        session_slot!(game, MotherBrainBrain[E].subfunction = v1 as u16);
        if v1 < 0 {
            session_slot!(game, MotherBrainBrain[E].subfunction = 4);
            let mut v2: i16 = ((session_slot!(game, MotherBrainBrain[E].subfunction_timer)) as i16).wrapping_sub(1);
            session_slot!(game, MotherBrainBrain[E].subfunction_timer = v2 as u16);
            if v2 < 0 {
                session_slot!(game, MotherBrainBrain[E].subfunction_timer = 3);
            }
            let v3: i32 = ((4u16).wrapping_mul(session_slot!(game, MotherBrainBrain[E].subfunction_timer))) as i32 >> 1;
            crate::actors::host::BossEffects::set_projectile_spawn_point(game, g_word_A9B393[v3 as usize], g_word_A9B393[(v3 + 1) as usize]);
            let mut v4: u16 = 3;
            if game.next_random() < 0x4000 {
                v4 = 12;
            }
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v4, game);
            QueueSfx2_Max3(0x24, game);
        }
    };
    @functions [pub fn (game: &mut (impl State + ?Sized))];
    @prefix { let E: i32 = 0x40; };
    MotherBrain_AimBeam [] {
        session_slot!(game, MotherBrainBody[0i32].beam_angle = -(crate::actors::queries::angle_from_delta((word!(game.samus_x_pos) as i32 - session_slot!(game, MotherBrainBrain[E].base.x_pos) as i32 - 16i32) as u16, (word!(game.samus_y_pos) as i32 - session_slot!(game, MotherBrainBrain[E].base.y_pos) as i32 - 4i32) as u16) as i32
            + 0x80i32) as u8 as u16);
    };
    MotherBrain_PainfulWalkForwards [] {
        if MotherBrain_MakeWalkForwards(session_slot!(game, MotherBrainBrain[E].painful_walk_delay), 0x48, game) & 1 != 0 {
            session_slot!(game, MotherBrainBrain[E].painful_walk_function = function::PAINFUL_WALKING_FORWARDS);
            MotherBrain_SetPainfulWalkingTimer(game);
        }
    };
    MotherBrain_PainfulWalkBackwards [] {
        if MotherBrain_MakeWalkBackwards(0x28, session_slot!(game, MotherBrainBrain[E].painful_walk_delay), game) & 1 != 0 {
            session_slot!(game, MotherBrainBrain[E].painful_walk_function = function::PAINFUL_WALKING_BACKWARDS);
            MotherBrain_SetPainfulWalkingTimer(game);
        }
    };
    MotherBrain_SetPainfulWalkingTimer [] {
        session_slot!(game, MotherBrainBrain[E].painful_walk_timer = g_word_A9C049[session_slot!(game, MotherBrainBrain[E].painful_walk_stage) as usize].to_ne_bytes()[0]
            as u16);
    };
    MotherBrain_Phase3_Walk_RetreatQuickly [] {
        if MotherBrain_MakeWalkBackwards(session_slot!(game, MotherBrainBrain[E].target_x), 2, game) & 1 != 0 {
            session_slot!(game, MotherBrainBrain[E].target_x = (session_slot!(game, MotherBrainBody[0i32].base.x_pos)).wrapping_sub(14));
            session_slot!(game, MotherBrainBrain[E].walk_function = function::WALK_RETREAT_SLOWLY);
        }
    };
    MotherBrain_Phase3_Walk_RetreatSlowly [] {
        if MotherBrain_MakeWalkBackwards(session_slot!(game, MotherBrainBrain[E].target_x), 4, game) & 1 != 0 {
            MotherBrain_SetToTryToInchForward(0x40, game);
        }
    };
    @functions [pub fn (game: &mut impl Host)];
    @prefix { let E: i32 = 0x40; };
    MotherBrain_HandlePalette [] {
        if session_slot!(game, MotherBrainBrain[E].brain_palette_enabled) != 0 {
            MotherBrain_HandleBrainPal(game);
        }
        if session_slot!(game, MotherBrainBrain[E].health_palette_enabled) != 0 {
            game.update_health_palette();
        }
    };
}
/// The neck runs in `crate::actors::mother_brain_neck`.
pub fn MotherBrain_HandleNeck(game: &mut impl Host) {
    crate::actors::mother_brain_neck::handle_neck(game);
}
pub fn MotherBrain_RetractHead(game: &mut impl Host) {
    crate::actors::mother_brain_neck::retract_head(game);
}
/// The phase-3 neck functions run in `crate::actors::mother_brain_neck`.
pub fn MotherBrain_Phase3_NeckHandler(game: &mut impl Host) {
    crate::actors::mother_brain_neck::run_phase3(game);
}
pub fn MoveEnemyWithVelocity(game: &mut (impl State + ?Sized)) {
    {
        let v0: i32 = word!(game.cur_enemy_index) as i32;
        let mut ai_var_B: u16 = session_slot!(game, EnemyData[v0].ai_var_B);
        let mut carry: i32 = session_slot!(byte game, EnemyData[v0].base.x_subpos[1]) as i32
            + ai_var_B.to_ne_bytes()[0] as i32;
        session_slot!(byte game, EnemyData[v0].base.x_subpos[1] = carry as u8);
        session_slot!(
            game,
            EnemyData[v0].base.x_pos = (session_slot!(game, EnemyData[v0].base.x_pos) as i32
                + ((ai_var_B >> 8) as i8 as i32 + (carry >> 8)))
                as u16
        );
        let mut ai_var_C: u16 = session_slot!(game, EnemyData[v0].ai_var_C);
        carry = session_slot!(byte game, EnemyData[v0].base.y_subpos[1]) as i32
            + ai_var_C.to_ne_bytes()[0] as i32;
        session_slot!(byte game, EnemyData[v0].base.y_subpos[1] = carry as u8);
        session_slot!(
            game,
            EnemyData[v0].base.y_pos = (session_slot!(game, EnemyData[v0].base.y_pos) as i32
                + ((ai_var_C >> 8) as i8 as i32 + (carry >> 8)))
                as u16
        );
    }
}
pub fn MotherBrain_SetBodyInstrs(a: u16, game: &mut (impl State + ?Sized)) {
    let E: i32 = 0;
    session_slot! { game, MotherBrainBody[E].base; current_instruction = a, instruction_timer = 1 }
    session_slot!(game, MotherBrainBody[E].base.timer = 0);
}
pub fn MotherBrain_SetBrainUnusedInstrs(a: u16, game: &mut (impl State + ?Sized)) {
    let E: i32 = 0x40;
    session_slot! { game, MotherBrainBrain[E].base; current_instruction = a, instruction_timer = 1 }
    session_slot!(game, MotherBrainBrain[E].base.timer = 0);
}
pub fn MotherBrain_SetBrainInstrs(a: u16, game: &mut (impl State + ?Sized)) {
    let E: i32 = 0;
    session_slot!(game, MotherBrainBody[E].brain_instructions = a);
    session_slot!(game, MotherBrainBody[E].brain_instruction_timer = 1);
}
pub fn Enemy_SetInstrList(k: u16, a: u16, game: &mut (impl State + ?Sized)) {
    let v2: i32 = k as i32;
    session_slot! { game, EnemyData[v2].base; current_instruction = a, instruction_timer = 1 }
    session_slot!(game, EnemyData[v2].base.timer = 0);
}
pub fn Enemy_IncreaseYpos(k: u16, mut a: u16, game: &mut (impl State + ?Sized)) {
    {
        let v2: i32 = k as i32;
        let carry: i32 = session_slot!(byte game, EnemyData[v2].base.y_subpos[1]) as i32
            + a.to_ne_bytes()[0] as i32;
        session_slot!(byte game, EnemyData[v2].base.y_subpos[1] = carry as u8);
        session_slot!(
            game,
            EnemyData[v2].base.y_pos = (session_slot!(game, EnemyData[v2].base.y_pos) as i32
                + ((a >> 8) as i8 as i32 + (carry >> 8)))
                as u16
        );
    }
}
pub fn MotherBrain_MakeWalkForwards(j: u16, a: u16, game: &mut (impl State + ?Sized)) -> u8 {
    {
        let E: i32 = 0;
        if ((a) as i16).wrapping_sub((session_slot!(game, MotherBrainBody[E].base.x_pos)) as i16)
            >= 0
        {
            if session_slot!(game, MotherBrainBody[E].pose) != 0 {
                return 0;
            }
            if session_slot!(game, MotherBrainBody[E].base.x_pos) as i32 - 128i32 & 0x8000 != 0 {
                MotherBrain_SetBodyInstrs(
                    mother_brain_tables::walk_forwards(game, (j >> 1) as isize).instruction_list,
                    game,
                );
                return 0;
            }
        }
        1
    }
}
pub fn MotherBrain_MakeWalkBackwards(a: u16, j: u16, game: &mut (impl State + ?Sized)) -> u8 {
    {
        let E: i32 = 0;
        if ((a) as i16).wrapping_sub((session_slot!(game, MotherBrainBody[E].base.x_pos)) as i16)
            < 0
        {
            if session_slot!(game, MotherBrainBody[E].pose) != 0 {
                return 0;
            }
            if session_slot!(game, MotherBrainBody[E].base.x_pos) as i32 - 48i32 & 0x8000 == 0 {
                MotherBrain_SetBodyInstrs(
                    mother_brain_tables::walk_backwards(game, (j >> 1) as isize).instruction_list,
                    game,
                );
                return 0;
            }
        }
        1
    }
}
pub fn MotherBrain_MakeHerStandUp(game: &mut (impl State + ?Sized)) -> u8 {
    {
        let mbn_var_02: u16 = session_slot!(game, MotherBrainBody[0i32].pose);
        if mbn_var_02 != 0 {
            let mut v1: u16 = animation::BODY_STAND_UP_FROM_CROUCH_FAST;
            if mbn_var_02 != 3 {
                if mbn_var_02 != 6 {
                    return 0;
                }
                v1 = animation::BODY_STAND_UP_FROM_LEANING_DOWN;
            }
            MotherBrain_SetBodyInstrs(v1, game);
            return 0;
        }
        1
    }
}
pub fn MotherBrain_MakeHerCrouch(game: &mut (impl State + ?Sized)) -> u8 {
    {
        let mbn_var_02: u16 = session_slot!(game, MotherBrainBody[0i32].pose);
        if mbn_var_02 != 0 {
            if mbn_var_02 == 3 {
                return 1;
            }
        } else {
            MotherBrain_SetBodyInstrs(animation::BODY_CROUCH_SLOW, game);
        }
        0
    }
}
pub fn MotherBrain_MakeHerLeanDown(game: &mut (impl State + ?Sized)) -> u8 {
    {
        let mbn_var_02: u16 = session_slot!(game, MotherBrainBody[0i32].pose);
        if mbn_var_02 != 0 {
            if mbn_var_02 == 6 {
                return 1;
            }
        } else {
            MotherBrain_SetBodyInstrs(animation::BODY_LEAN_DOWN, game);
        }
        0
    }
}
pub fn MotherBrain_Pal_ProcessInvincibility(game: &mut (impl State + ?Sized)) {
    if session_slot!(game, MotherBrainBody[0i32].form) == 4 {
        let flash_timer: u16 = session_slot!(game, MotherBrainBrain[0x40i32].base.flash_timer);
        if flash_timer as i32 >> 1 != 0 && flash_timer & 1 == 0 {
            for i in (0i32..=28).rev().step_by(2) {
                let v2: i32 = i >> 1;
                let v3: u16 = word!(game.palette_buffer[(v2 + 129)]);
                word! {game; palette_buffer[(v2 + 145)] = v3; palette_buffer[(v2 + 177)] = v3;
                palette_buffer[(v2 + 65)] = v3; }
            }
        }
    }
}
pub fn MotherBrain_Pal_HandleRoomPal(game: &mut impl Host) {
    {
        let j: u16 = HandleMotherBrainInstructionList(0x781c, game);
        if j != 0 {
            MotherBrain_Pal_WriteRoomPal(j, game);
        }
    }
}
pub fn MotherBrain_Pal_WriteRoomPal(j: u16, game: &mut impl Host) {
    {
        copy_rom_colors_to_palette(0x68, 0xa9, j, 12, game);
        copy_rom_colors_to_palette(0xa6, 0xa9, (j).wrapping_add(24), 12, game);
        copy_rom_colors_to_palette(0xe6, 0xa9, (j).wrapping_add(24), 12, game);
    }
}
pub fn HandleMotherBrainInstructionList(a: u16, game: &mut impl Host) -> u16 {
    {
        let st: usize = a as usize;
        if native_word(st, game) & 0x8000 == 0 {
            return 0;
        }
        let mut v2: u16 = native_word(st, game);
        let mut v3 = native_record(0xa90000 | u32::from(v2), game);
        let mut execute_instruction: bool = (v3.word(0) & 0x8000 != 0)
            || ({
                if native_word(st + 2, game) != v3.word(0) {
                    set_native_word(st + 2, native_word(st + 2, game).wrapping_add(1), game);
                    return v3.word(2);
                }
                v2 = v2.wrapping_add(4);
                false
            });
        loop {
            if execute_instruction {
                v2 = CallMotherBrainInstr(
                    u32::from(v3.word(0)) | 0xa90000,
                    v2.wrapping_add(2),
                    game,
                );
                execute_instruction = false;
            } else {
                v3 = native_record(0xa90000 | u32::from(v2), game);
                if v3.word(0) & 0x8000 == 0 {
                    break;
                } else {
                    execute_instruction = true;
                }
            }
        }
        if v3.word(0) == 0 {
            set_native_word(st, 0, game);
            set_native_word(st + 2, 0, game);
            0
        } else {
            set_native_word(st + 2, 1, game);
            set_native_word(st, v2, game);
            v3.word(2)
        }
    }
}
pub fn MotherBrain_SetupBrainNormalPal(game: &mut (impl State + ?Sized)) {
    session_slot!(game, MotherBrainBody[0i32].brain_palette_reload = 10);
}
pub fn MotherBrain_SetupBrainPalForLaser(game: &mut (impl State + ?Sized)) {
    session_slot!(game, MotherBrainBody[0i32].brain_palette_reload = 514);
}

// Mother Brain's AI and instruction entries, forwarded from the bank's dispatch.
pub fn MotherBrainsBody_Init(game: &mut impl Host) {
    word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
    MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9C13 as u16, game);
    let E: i32 = 0;
    session_slot!(game, MotherBrainBody[E].base.vram_tiles_index = 0);
    session_slot!(
        game,
        MotherBrainBody[E].base.properties =
            (session_slot!(game, MotherBrainBody[E].base.properties) as i32
                | (kEnemyProps_BlockPlasmaBeam as i32
                    | kEnemyProps_Tangible as i32
                    | kEnemyProps_Invisible as i32)) as u16
    );
    session_slot!(game, MotherBrainBody[E].base.palette_index = 0);
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
    session_slot!(game, MotherBrainBody[E].form = 0);
    let E1: i32 = 0x40;
    session_slot!(game, MotherBrainBrain[E1].unpause_hook = 0);
    session_slot!(game, MotherBrainBody[E].hitboxes = 2);
    session_slot!(
        game,
        MotherBrainBrain[E1].function = function::BRAIN_SET_UP_BRAIN_TO_BE_DRAWN
    );
    session_slot!(game, MotherBrainBody[E].function = function::FIRST_PHASE);
    LoadFxEntry(1, game);
    for v3 in 0u16..12 {
        SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurrets as u16, v3, game);
    }
}
pub fn MotherBrainsBrain_Init(game: &mut impl Host) {
    game.initialize_corpse_rotting(0x40, addr_stru_A9DE08 as u16);
    let E1: i32 = 0x40;
    session_slot!(game, MotherBrainBrain[E1].base.health = 3000);
    MotherBrain_SetBrainUnusedInstrs(addr_kMotherBrain_Ilist_9C13 as u16, game);
    session_slot! { game, MotherBrainBrain[E1].base; vram_tiles_index = 0, properties = (session_slot!(game, MotherBrainBrain[E1].base.properties)) | (0x1100u16), palette_index = 512 };
    let E: i32 = 0;
    session_slot! { game, MotherBrainBody[E]; neck_palette = 512, brain_palette = 512 }
    MotherBrain_SetBrainInstrs(addr_stru_A99C21 as u16, game);
    MotherBrain_SetupBrainNormalPal(game);
}
/// Main AI - Mother Brain body: one lease for the frame; the body function,
/// palette and Samus collision run on the game.
pub fn MotherBrainsBody_Hurt(game: &mut impl Host) {
    let mut lease = game.lease();
    let lease = &mut lease;
    lease.foreign(|game| MotherBrain_Pal_HandleRoomPal(game));
    let E: i32 = 0;
    if session_slot!(lease, MotherBrainBody[E].function) == function::FIRST_PHASE {
        lease.foreign(|game| MotherBrainsBody_FirstPhase_DoubleRet(game));
        return;
    }
    let body_function = session_slot!(lease, MotherBrainBody[E].function);
    run_mother_brain_function((body_function as u32) | (0xa90000u32), lease);
    lease.foreign(|game| {
        MotherBrain_HandlePalette(game);
        MotherBrain_SamusCollDetect(game);
    });
    MotherBrain_Pal_ProcessInvincibility(lease);
    if session_slot!(lease, MotherBrainBrain[0x40i32].unpause_hook) != 0 {
        write_long_pointer(UNPAUSE_HOOK, UNPAUSE_BODY_HOOK, lease);
    }
}
pub fn MotherBrainsBody_Powerbomb(game: &mut impl Host) {
    game.power_bomb_reaction(crate::actors::host::Death::LeavePending);
    MotherBrainsBrain_Hurt(game);
}
/// Main AI - Mother Brain brain: one lease for the frame; the brain function
/// runs on the game.
pub fn MotherBrainsBrain_Hurt(game: &mut impl Host) {
    let mut lease = game.lease();
    let lease = &mut lease;
    set_graphics_drawn_hook(NO_DRAW_HOOK, lease);
    if (session_slot!(lease, MotherBrainBrain[BRAIN].base.properties) & 0x100) != 0 {
        let brain_function = session_slot!(lease, MotherBrainBrain[BRAIN].function);
        run_mother_brain_function((brain_function as u32) | (0xa90000u32), lease);
    }
}
pub fn MotherBrainsBody_Shot(game: &mut impl Host) {
    game.create_dud();
}
pub fn MotherBrainsBrain_Touch(game: &mut impl Host) {
    if game.ram().read_byte(0xa1f) == 3 {
        let mut v0: i16 = 13;
        let flash_timer: u16 = session_slot!(game, MotherBrainBrain[BRAIN].base.flash_timer);
        if flash_timer != 0 && flash_timer & 1 != 0 {
            v0 = 14;
        }
        session_slot!(game, MotherBrainBrain[BRAIN].base.flash_timer = v0 as u16);
    }
}
pub fn MotherBrain_Instr_SpawnEprojToOffset(
    _k: u16,
    jp: EnemyScript,
    game: &mut impl Host,
) -> EnemyScript {
    let E: i32 = 0;
    {
        let (x, y) = (
            ((session_slot!(game, MotherBrainBody[E].base.x_pos)).wrapping_add(jp.word(0))),
            ((session_slot!(game, MotherBrainBody[E].base.y_pos)).wrapping_add(jp.word(2))),
        );
        crate::actors::host::BossEffects::set_projectile_spawn_point(game, x, y);
    }
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, jp.word(4), game);
    jp.offset(3)
}
pub fn MotherBrain_Instr_SpawnDeathBeamEproj(
    _k: u16,
    jp: EnemyScript,
    game: &mut impl Host,
) -> EnemyScript {
    let spawn_parameter = game.queue_sound_result(1, 0x63, 6);
    game.spawn_projectile(
        spawn_parameter,
        0x40,
        addr_kEproj_MotherBrainDeathBeamCharging as u16,
    );
    jp
}
pub fn MotherBrain_Instr_IncrBeamAttackPhase(
    _k: u16,
    jp: EnemyScript,
    game: &mut (impl State + ?Sized),
) -> EnemyScript {
    let E: i32 = 0;
    session_slot!(
        game,
        MotherBrainBody[E].hand_beam_phase =
            session_slot!(game, MotherBrainBody[E].hand_beam_phase).wrapping_add(1)
    );
    jp
}
