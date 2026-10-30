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
use crate::actors::corpses::Corpse;
use crate::actors::host::abs16;
use crate::actors::mother_brain::{BRAIN, MotherBrainBody, MotherBrainBrain};
use crate::actors::mother_brain_effects::{
    generate_explosions as generate_body_explosions,
    handle_rainbow_palette as handle_body_rainbow_palette,
    write_palette as write_mother_brain_palette,
};
use crate::actors::mother_brain_neck::function as neck;
use crate::actors::session_slots::{SlotAccess, session_slot};
use crate::runtime_access::SessionRam;
// The game's traits, for the calls made on it through `foreign`.
use crate::actors::host::{ActorRuntime as _, BossEffects as _, Random as _};
use crate::ram::CompatibilityRam as _;

use crate::constants::bank_8d::*;
use crate::constants::bank_86::*;
use crate::constants::bank_a6::*;
use crate::constants::bank_a9::*;
use crate::constants::enemy_properties::*;
use crate::enemy_projectiles::shared_pre::y_velocity_at_angle;
use crate::enemy_script::EnemyScript;
use crate::menus::menu_tilemap::native_record;
use crate::menus::prelude::{EnemyData, do_until};
use crate::mother_brain_tables;
use crate::palette_copies::{copy_rom_colors_to_palette, copy_rom_colors_to_target_palette};
use crate::ram::word;
use crate::word_transfers;

/// A bank-local address, as the original's `VoidP`.
type CoroutineRet = u8;
const COROUTINE_NONE: i32 = 0;

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

/// Bank $A9, where this code and its tables live.
pub(crate) const BANK: u8 = 0xa9;
pub(crate) const BANK_BASE: u32 = 0xa9_0000;

/// Palette RAM destinations, by palette and colour (colour 1 unless named).
pub(crate) mod cgram {
    /// BG palette 3 colour 4.
    pub(crate) const BG_PALETTE_3_COLOUR_4: u16 = 0x68;
    /// BG palette 4 colour 1.
    pub(crate) const BG_PALETTE_4: u16 = 0x82;
    /// BG palette 5 colour 3.
    pub(crate) const BG_PALETTE_5_COLOUR_3: u16 = 0xa6;
    /// BG palette 7 colour 3.
    pub(crate) const BG_PALETTE_7_COLOUR_3: u16 = 0xe6;
    /// Sprite palette 1 colour 0.
    pub(crate) const SPRITE_PALETTE_1_COLOUR_0: u16 = 0x120;
    /// Sprite palette 1 colour 1.
    pub(crate) const SPRITE_PALETTE_1: u16 = 0x122;
    /// Sprite palette 2 colour 0.
    pub(crate) const SPRITE_PALETTE_2_COLOUR_0: u16 = 0x140;
    /// Sprite palette 2 colour 1.
    pub(crate) const SPRITE_PALETTE_2: u16 = 0x142;
    /// Sprite palette 3 colour 1.
    pub(crate) const SPRITE_PALETTE_3: u16 = 0x162;
    /// Sprite palette 7 colour 0.
    pub(crate) const SPRITE_PALETTE_7_COLOUR_0: u16 = 0x1e0;
    /// Sprite palette 7 colour 1.
    pub(crate) const SPRITE_PALETTE_7: u16 = 0x1e2;
    /// Sprite palette 7 colour 5.
    pub(crate) const SPRITE_PALETTE_7_COLOUR_5: u16 = 0x1ea;
    /// Sprite palette 7 colour 10.
    pub(crate) const SPRITE_PALETTE_7_COLOUR_10: u16 = 0x1f4;
}

/// Bank $A6, where Ridley's escape tiles Mother Brain loads live.
const BANK_A6: u8 = 0xa6;
/// $7E:781C: the body's room palette instruction list pointer.
const ROOM_PALETTE_INSTRUCTIONS: u16 = 0x781c;
const _: () = assert!(
    0xf78 + core::mem::offset_of!(MotherBrainBody, room_palette_list)
        == ROOM_PALETTE_INSTRUCTIONS as usize
);
/// $86:CC5B: Mother Brain's top-right tube falling enemy projectile.
const TOP_RIGHT_TUBE_FALLING_PROJECTILE: u16 = 0xcc5b;
/// $A9:8B15: the tubes falling enemy 3 population (bottom-middle-right).
const TUBES_FALLING_ENEMY_3: u16 = 0x8b15;

/// Sound effect ids, named after the bank log's comments; the queueing
/// helper picks the library.
pub mod sound {
    /// Silence, in sound library 1.
    pub const SILENCE_LIBRARY_1: u16 = 0x2;
    /// Lava/acid damaging Samus.
    pub const LAVA_ACID_DAMAGE: u16 = 0x10;
    /// Mother Brain or the baby Metroid exploding.
    pub const EXPLODING: u16 = 0x13;
    /// The baby Metroid dies.
    pub const BABY_METROID_DIES: u16 = 0x19;
    /// Small explosion.
    pub const SMALL_EXPLOSION: u16 = 0x24;
    /// Mother Brain rising into phase 2.
    pub const RISING_INTO_PHASE_2: u16 = 0x29;
    /// Mother Brain's rainbow beam.
    pub const RAINBOW_BEAM: u16 = 0x40;
    /// The baby Metroid feels remorse.
    pub const BABY_METROID_REMORSE: u16 = 0x52;
    /// Mother Brain's hand beam.
    pub const HAND_BEAM: u16 = 0x63;
    /// Shot Mother Brain phase 1.
    pub const SHOT_IN_PHASE_1: u16 = 0x6e;
    /// Silence.
    pub const SILENCE: u16 = 0x71;
    /// The baby Metroid's cry.
    pub const BABY_METROID_CRY: u16 = 0x72;
    /// The baby Metroid feels guilty.
    pub const BABY_METROID_GUILTY: u16 = 0x7d;
    /// Mother Brain charging her rainbow.
    pub const CHARGING_RAINBOW: u16 = 0x7f;
}

/// Music data and tracks queued, after the bank log's comments.
pub mod music {
    /// Music stop.
    pub const STOP: u16 = 0x0;
    /// Song 0 music track.
    pub const SONG_0: u16 = 0x5;
    /// Song 1 music track.
    pub const SONG_1: u16 = 0x6;
    /// Song 2 music track (the escape music).
    pub const SONG_2: u16 = 0x7;
    /// Mother Brain music data.
    pub const MOTHER_BRAIN_DATA: u16 = 0xff21;
    /// Boss fight 1 music data.
    pub const BOSS_FIGHT_1_DATA: u16 = 0xff24;
    /// Samus theme music data.
    pub const SAMUS_THEME_DATA: u16 = 0xff48;
}

/// Samus commands run, after the bank log's comments.
pub mod samus_command {
    /// Lock Samus.
    pub const LOCK_SAMUS: u16 = 0x0;
    /// Unlock Samus.
    pub const UNLOCK_SAMUS: u16 = 0x1;
    /// Enable timer handling.
    pub const ENABLE_TIMER_HANDLING: u16 = 0xf;
    /// Enable Samus blue flashing.
    pub const ENABLE_BLUE_FLASHING: u16 = 0x12;
    /// Disable Samus blue flashing.
    pub const DISABLE_BLUE_FLASHING: u16 = 0x13;
    /// Enable rainbow Samus.
    pub const ENABLE_RAINBOW: u16 = 0x16;
    /// Disable rainbow Samus and stand her up.
    pub const DISABLE_RAINBOW_AND_STAND_UP: u16 = 0x17;
    /// Freeze drained Samus animation.
    pub const FREEZE_DRAINED_ANIMATION: u16 = 0x19;
}

/// Events marked and checked, after the bank log's comments.
pub mod event {
    /// Mother Brain's glass is destroyed.
    pub const MOTHER_BRAINS_GLASS_DESTROYED: u16 = 0x2;
    /// Zebes timebomb set.
    pub const ZEBES_TIMEBOMB_SET: u16 = 0xe;
}

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
    assert!(a::BODY_WALK_FORWARDS_MEDIUM == ADDR_K_MOTHER_BRAIN_ILIST_97A4 as u16);
    assert!(a::BODY_WALK_BACKWARDS_FAST == ADDR_K_MOTHER_BRAIN_ILIST_98C6 as u16);
    assert!(a::BODY_STAND_UP_FROM_CROUCH_SLOW == ADDR_K_MOTHER_BRAIN_ILIST_99AA as u16);
    assert!(a::BODY_STAND_UP_FROM_CROUCH_FAST == ADDR_K_MOTHER_BRAIN_ILIST_99C6 as u16);
    assert!(a::BODY_STAND_UP_FROM_LEANING_DOWN == ADDR_K_MOTHER_BRAIN_ILIST_99E2 as u16);
    assert!(a::BODY_LEAN_DOWN == ADDR_K_MOTHER_BRAIN_ILIST_99F2 as u16);
    assert!(a::BODY_CROUCHED == ADDR_K_MOTHER_BRAIN_ILIST_9A02 as u16);
    assert!(a::BODY_CROUCH_SLOW == ADDR_K_MOTHER_BRAIN_ILIST_9A0A as u16);
    assert!(a::BODY_CROUCH_FAST == ADDR_K_MOTHER_BRAIN_ILIST_9A26 as u16);
    assert!(a::BODY_HAND_BEAM_MODE == ADDR_K_MOTHER_BRAIN_ILIST_9A42 as u16);
    assert!(a::BRAIN_STRETCHING_PHASE_2 == ADDR_K_MOTHER_BRAIN_ILIST_9B7F as u16);
    assert!(a::BRAIN_STRETCHING_PHASE_3 == ADDR_K_MOTHER_BRAIN_ILIST_9BB3 as u16);
    assert!(a::BRAIN_DECAPITATED == ADDR_K_MOTHER_BRAIN_ILIST_9C29 as u16);
    assert!(a::BRAIN_DYING_DROOL == ADDR_K_MOTHER_BRAIN_ILIST_9C39 as u16);
    assert!(a::BRAIN_FIRING_RAINBOW_BEAM == ADDR_K_MOTHER_BRAIN_ILIST_9C77 as u16);
    assert!(a::BRAIN_NEUTRAL_PHASE_2 == ADDR_K_MOTHER_BRAIN_ILIST_9C87 as u16);
    assert!(a::BRAIN_CORPSE == ADDR_K_MOTHER_BRAIN_ILIST_9D25 as u16);
    assert!(a::BRAIN_FOUR_BLUE_RINGS_PHASE_2 == ADDR_K_MOTHER_BRAIN_ILIST_9D3D as u16);
    assert!(a::BRAIN_TWO_BLUE_RINGS_PHASE_2 == ADDR_K_MOTHER_BRAIN_ILIST_9D7F as u16);
    assert!(a::BRAIN_ATTACKING_BABY_METROID == ADDR_K_MOTHER_BRAIN_ILIST_9DB1 as u16);
    assert!(a::BRAIN_FOUR_BLUE_RINGS_PHASE_3 == ADDR_K_MOTHER_BRAIN_ILIST_9DBB as u16);
    assert!(a::BRAIN_BOMB_PHASE_2 == ADDR_K_MOTHER_BRAIN_ILIST_9ECC as u16);
    assert!(a::BRAIN_BOMB_PHASE_3 == ADDR_K_MOTHER_BRAIN_ILIST_9F00 as u16);
    assert!(a::BRAIN_LASER == ADDR_K_MOTHER_BRAIN_ILIST_9F34 as u16);
    assert!(a::BRAIN_CHARGING_RAINBOW_BEAM == ADDR_K_MOTHER_BRAIN_ILIST_9F6C as u16);
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
    assert!(BRAIN_HITBOX == ADDR_WORD_A9B427 as u16 + 0x12 && NECK_HITBOX == BRAIN_HITBOX + 0x12);

/// $A9:87C9 and $A9:87DD: the graphics drawn hooks that draw the brain
/// with and without the neck.
pub const DRAW_BRAIN_AND_NECK_HOOK: u32 = 0xa9_87c9;
pub const DRAW_BRAIN_HOOK: u32 = 0xa9_87dd;
// They are the ids `actor_species_draw_hook` dispatches on.
const _: () = assert!(DRAW_BRAIN_AND_NECK_HOOK == 11110345 && DRAW_BRAIN_HOOK == 11110365);
const GRAPHICS_DRAWN_HOOK: usize = 0x178c;

/// Set the enemy graphics drawn hook, low byte first and the bank last.
pub(crate) fn set_graphics_drawn_hook(hook: u32, game: &mut impl State) {
    write_long_pointer(GRAPHICS_DRAWN_HOOK, hook, game);
}

/// Store a long pointer at `at`, low byte first and the bank last.
fn write_long_pointer(at: usize, pointer: u32, game: &mut impl State) {
    let bytes = pointer.to_le_bytes();
    SessionRam::set_byte(game, at, bytes[0]);
    SessionRam::set_byte(game, at + 1, bytes[1]);
    SessionRam::set_byte(game, at + 2, bytes[2]);
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
/// inside a lease, the RAM image's on the game), named RAM words, the ROM
/// records and native image, and sound. Routines written against it run on
/// either.
pub trait State:
    SlotAccess
    + crate::ram::CompatibilityRam
    + crate::records::Records
    + crate::cartridge::NativeDiagnostics
    + crate::cartridge::LoadedImage
    + crate::effects::SoundEffects
    + crate::player_position::PlayerPosition
    + SessionRam
{
}
impl<T> State for T where
    T: SlotAccess
        + crate::ram::CompatibilityRam
        + crate::records::Records
        + crate::cartridge::NativeDiagnostics
        + crate::cartridge::LoadedImage
        + crate::effects::SoundEffects
        + crate::player_position::PlayerPosition
        + SessionRam
        + ?Sized
{
}

/// A lease on the game for one frame of an AI: the state steps run on it, and
/// `foreign` runs the rest on the game it leases.
pub trait Lease: State {
    type Game: Host;
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
    /// Ridley's contact damage, which Mother Brain shares. Was `Ridley_Func_98`.
    fn deal_contact_damage(&mut self);
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

// The calls the moved bodies make on the game, each through `foreign`.
macro_rules! on_game {
    ($($name:ident($($arg:ident: $ty:ty),*) $(-> $ret:ty)? => $method:ident;)*) => {
        $(pub(crate) fn $name($($arg: $ty,)* game: &mut impl Lease) $(-> $ret)? {
            game.foreign(|game| game.$method($($arg),*))
        })*
    };
}
on_game! {
    bank_unreachable() -> bool => bank_unreachable;
    spawn_room_projectile(definition: u16, parameter: u16) -> u16 => spawn_room_projectile;
    spawn_plm(block_x: u8, block_y: u8, header: u16) => spawn_plm;
    queue_music(track: u16) => queue_music_delayed;
    run_samus_command(command: u16) -> u16 => command_player;
    native_word(offset: usize) -> u16 => native_word;
    set_native_word(offset: usize, value: u16) => set_native_word;
    queue_mother_brain_sprite_tiles(bank: u8, start: u16) -> u8 => queue_mother_brain_sprite_tiles;
    process_corpse_rotting(k: u16) -> u8 => process_corpse_rotting;
    process_corpse_rotting_vram_transfers(k: u16) => process_corpse_rotting_vram_transfers;
    start_zebes_escape_typewriter() => start_zebes_escape_typewriter;
    type_escape_text(tiles: u16) -> u8 => type_escape_text;
    set_boss_bit_for_current_area(mask: u16) => set_boss_bit_for_current_area;
    set_event_happened(event: u16) => set_event_happened;
    check_event_happened(event: u16) -> u16 => check_event_happened;
    spawn_rainbow_hdma() -> u16 => spawn_rainbow_hdma;
    spawn_rising_hdma() -> u16 => spawn_rising_hdma;
    player_mother_brain(command: u16) => player_mother_brain;
    load_fx_entry(index: u16) => load_fx_entry;
    samus_contact(boxes: u16, x: u16, y: u16) -> Contact => samus_contact;
    deal_contact_damage() => deal_contact_damage;
    move_samus_horizontally_towards_wall(amount: u16) -> u8 => move_samus_horizontally_towards_wall;
    move_samus_vertically_towards_ceiling_floor(amount: u16) -> u8 => move_samus_vertically_towards_ceiling_floor;
    add_glass_hits(slot: usize, amount: u16) => add_glass_hits;
    spawn_palette_effect(definition: u16) => spawn_palette_effect;
}
pub(crate) fn queue_sfx1_max6(sound: u16, game: &mut impl State) {
    game.queue_sound(0, sound as u8, 6)
}
pub(crate) fn queue_sfx2_max3(sound: u16, game: &mut impl State) {
    game.queue_sound(1, sound as u8, 3)
}
pub(crate) fn queue_sfx2_max6(sound: u16, game: &mut impl State) {
    game.queue_sound(1, sound as u8, 6)
}
pub(crate) fn queue_sfx3_max3(sound: u16, game: &mut impl State) {
    game.queue_sound(2, sound as u8, 3)
}
pub(crate) fn queue_sfx3_max6(sound: u16, game: &mut impl State) {
    game.queue_sound(2, sound as u8, 6)
}

pub static FAKE_DEATH_EXPLOSION_TYPES: [u8; 2] = [9, 0x12];
pub static FAKE_DEATH_EXPLOSION_X: [u16; 8] = [0x3d, 0x54, 0x20, 0x35, 0x5a, 0x43, 0x67, 0x29];
pub static SHAKE_X_OFFSETS: [i16; 4] = [0, -1, 0, 1];
pub static SHAKE_Y_OFFSETS: [i16; 4] = [0, 1, -1, 1];
pub static BABY_METROID_ATTACK_CRIES: [u16; 13] = [
    0x6f, 0x6f, 0x6f, 0x7e, 0x6f, 0x6f, 0x7e, 0x6f, 0x6f, 0x7e, 0x7e, 0x6f, 0x6f,
];
pub static ESCAPE_DOOR_EXPLOSION_OFFSETS: [u16; 8] = [8, 0x6c, 0x18, 0x80, 9, 0x90, 0x18, 0x74];
pub static GLASS_HITS_BY_PROJECTILE: [u8; 8] = [0, 1, 1, 0, 0, 0, 0, 0];
pub static SHOT_REACTION_BY_PROJECTILE: [u8; 8] = [2, 1, 1, 0, 0, 0, 0, 0];
pub static ATTACK_STRATEGY_BY_SAMUS_MOVEMENT: [i16; 28] = [
    -1, -17, 0, 0, -1, -1, 0, -1, 0, 0, -1, 0, 0, 0, -17, -1, -1, -1, 0, 0, 0, -17, -1, -1, -1, -1,
    -1, -1,
];
pub static RAINBOW_BEAM_EXPLOSION_X: [i16; 8] = [-8, 6, -4, 2, 3, -6, 8, 0];
pub static RAINBOW_BEAM_EXPLOSION_Y: [i16; 8] = [-7, 2, 5, -4, 6, -2, -6, 7];
pub static WALKING_ANIMATION_DELAYS: [u8; 16] = [2, 0, 2, 0, 6, 0, 6, 0, 8, 0, 8, 0, 10, 0, 10, 0];
pub static NECK_ANGLE_DELTAS: [u16; 8] = [0x500, 0x500, 0x200, 0x200, 0xc0, 0xc0, 0x40, 0x40];
pub static PAINFUL_WALKING_TIMERS: [i16; 8] = [0x10, 0x10, 0x20, 0x20, 0x30, 0x30, 0x40, 0x40];

pub fn corpse_rotting_finished(game: &mut impl Lease) {
    let e1: i32 = BRAIN;
    game.foreign(|game| {
        crate::actors::host::BossEffects::set_projectile_spawn_point(
            game,
            (session_slot!(game, MotherBrainBrain[e1].base.x_pos) as i32
                + (word!(game.RANDOM_NUMBER) as i32 & 0x1f)
                - 16i32) as u16,
            (session_slot!(game, MotherBrainBrain[e1].base.y_pos)).wrapping_add(16),
        )
    });
    spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0xa, game);
    if (word!(game.RANDOM_ENEMY_COUNTER) & 7) == 0 {
        queue_sfx2_max3(sound::LAVA_ACID_DAMAGE, game);
    }
}

crate::dispatch::callback_dispatch! {
    pub fn run_function(ea: u32, game: &mut impl Lease) => ea;
    call (game); default { bank_unreachable(game); };
    @body checked;
    @functions [pub fn (game: &mut impl Lease)];
    [11110306] set_up_brain_and_neck_to_draw [] {
        if word!(game.TIME_IS_FROZEN_FLAG) == 0 {
            handle_neck(game);
        }
        let e1: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e1].base.x_pos = session_slot!(game, MotherBrainBrain[e1].segment_5_x));
        session_slot!(game, MotherBrainBrain[e1].base.y_pos = (session_slot!(game, MotherBrainBrain[e1].segment_5_y)).wrapping_sub(21));
        set_graphics_drawn_hook(DRAW_BRAIN_AND_NECK_HOOK, game);
    };
    [11110429] fake_death_descent_initial_pause [] {
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_DESCENT_LOCK_SAMUS_AND_SET_SCROLL_REGION);
        session_slot!(game, MotherBrainBody[e].function_timer = 64);
        fake_death_descent_lock_samus_and_set_scroll_region(game);
    };
    [11110611] fake_death_descent_collapse_tubes [] {
        let e: i32 = BRAIN;
        run_function(((session_slot!(game, MotherBrainBrain[e].subfunction)) as u32) | (BANK_BASE), game);
        handle_fake_death_explosions(game);
    };
    [11110729] wait_then_spawn_tubes_falling_enemy_0 [] {
        let free = game.foreign(|game| {
            crate::enemy_projectiles::table::with_slots(game, |slots| {
                slots.iter().filter(|slot| slot.definition == 0).count()
            })
        });
        if free >= 4 {
            game.foreign(|game| game.spawn_legacy_population(BANK, ADDR_STRU_A98AE5 as u16));
            session_slot!(game, MotherBrainBrain[BRAIN].subfunction = function::TUBES_CLEAR_BOTTOM_LEFT_TUBE);
        }
    };
    [11110766] clear_bottom_left_tube [] {
        spawn_plm(0x5, 0x9, plm::CLEAR_BOTTOM_LEFT_TUBE, game);
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_SPAWN_TOP_RIGHT_TUBE_FALLING_PROJECTILE);
        session_slot!(game, MotherBrainBrain[e].subfunction_timer = 32);
    };
    [11110816] clear_ceiling_block_column_9 [] {
        spawn_plm(0x9, 0x2, plm::CLEAR_CEILING_BLOCK, game);
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_SPAWN_TOP_LEFT_TUBE_FALLING_PROJECTILE);
        session_slot!(game, MotherBrainBrain[e].subfunction_timer = 32);
    };
    [11110866] clear_ceiling_block_column_6 [] {
        spawn_plm(0x6, 0x2, plm::CLEAR_CEILING_BLOCK, game);
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_SPAWN_TUBES_FALLING_ENEMY_1);
        session_slot!(game, MotherBrainBrain[e].subfunction_timer = 32);
    };
    [11110906] clear_bottom_right_tube [] {
        spawn_plm(0xa, 0x9, plm::CLEAR_BOTTOM_RIGHT_TUBE, game);
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_SPAWN_TUBES_FALLING_ENEMY_2);
        session_slot!(game, MotherBrainBrain[e].subfunction_timer = 32);
    };
    [11110946] clear_bottom_middle_left_tube [] {
        spawn_plm(0x6, 0xa, plm::CLEAR_BOTTOM_MIDDLE_SIDE_TUBE, game);
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_SPAWN_TOP_MIDDLE_LEFT_TUBE_FALLING_PROJECTILE);
        session_slot!(game, MotherBrainBrain[e].subfunction_timer = 32);
    };
    [11110996] clear_ceiling_tube_column_7 [] {
        spawn_plm(0x7, 0x2, plm::CLEAR_CEILING_TUBE, game);
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_SPAWN_TOP_MIDDLE_RIGHT_TUBE_FALLING_PROJECTILE);
        session_slot!(game, MotherBrainBrain[e].subfunction_timer = 32);
    };
    [11111046] clear_ceiling_tube_column_8 [] {
        spawn_plm(0x8, 0x2, plm::CLEAR_CEILING_TUBE, game);
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_SPAWN_TUBES_FALLING_ENEMY_3);
        session_slot!(game, MotherBrainBrain[e].subfunction_timer = 32);
    };
    [11111086] clear_bottom_middle_right_tube [] {
        spawn_plm(0x9, 0xa, plm::CLEAR_BOTTOM_MIDDLE_SIDE_TUBE, game);
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_SPAWN_TUBES_FALLING_ENEMY_4);
        session_slot!(game, MotherBrainBrain[e].subfunction_timer = 2);
    };
    [11111126] clear_bottom_middle_tubes [] {
        spawn_plm(0x7, 0x7, plm::CLEAR_BOTTOM_MIDDLE_TUBES, game);
        session_slot!(game, MotherBrainBrain[BRAIN].subfunction = ADDR_LOCRET_A98AE4 as u16);
    };
    [11111559] fake_death_ascent_draw_bg1_rows_2_3 [] {
        spawn_plm(0x2, 0x2, plm::BACKGROUND_ROW_2, game);
        spawn_plm(0x2, 0x3, plm::BACKGROUND_ROW_3, game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_4_5);
    };
    [11111582] fake_death_ascent_draw_bg1_rows_4_5 [] {
        spawn_plm(0x2, 0x4, plm::BACKGROUND_ROW_4, game);
        spawn_plm(0x2, 0x5, plm::BACKGROUND_ROW_5, game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_6_7);
    };
    [11111605] fake_death_ascent_draw_bg1_rows_6_7 [] {
        spawn_plm(0x2, 0x6, plm::BACKGROUND_ROW_6, game);
        spawn_plm(0x2, 0x7, plm::BACKGROUND_ROW_7, game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_8_9);
    };
    [11111628] fake_death_ascent_draw_bg1_rows_8_9 [] {
        spawn_plm(0x2, 0x8, plm::BACKGROUND_ROW_8, game);
        spawn_plm(0x2, 0x9, plm::BACKGROUND_ROW_9, game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_A_B);
    };
    [11111651] fake_death_ascent_draw_bg1_rows_a_b [] {
        spawn_plm(0x2, 0xa, plm::BACKGROUND_ROW_A, game);
        spawn_plm(0x2, 0xb, plm::BACKGROUND_ROW_B, game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_DRAW_ROOM_BACKGROUND_ON_BG1_ROWS_C_D);
    };
    [11111674] fake_death_ascent_draw_bg1_rows_c_d [] {
        spawn_plm(0x2, 0xc, plm::BACKGROUND_ROW_C, game);
        spawn_plm(0x2, 0xd, plm::BACKGROUND_ROW_D, game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_SET_UP_MOTHER_BRAIN_PHASE_2_GRAPHICS);
    };
    [11111697] fake_death_ascent_set_up_phase2_graphics [] {
        { let value: u16 = 257; let at: usize = crate::ram::bytes::LAYER2_SCROLL_X; let bytes = value.to_ne_bytes(); SessionRam::set_byte(game, at, bytes[0]); SessionRam::set_byte(game, at + 1, bytes[1]); };
        { let at: usize = crate::ram::mirror::BG2SC; let bytes: [u8; 2] = ((u16::from_le_bytes([SessionRam::byte(game, at), SessionRam::byte(game, at + 1)])) & (0xfffcu16)).to_le_bytes(); SessionRam::set_byte(game, at, bytes[0]); SessionRam::set_byte(game, at + 1, bytes[1]); }
        copy_rom_colors_to_palette(
            cgram::SPRITE_PALETTE_2,
            BANK,
            (ADDR_K_MOTHER_BRAIN_PALETTE_1 as i32 + 2) as u16,
            0xf,
            game,
        );
        copy_rom_colors_to_palette(
            cgram::SPRITE_PALETTE_3,
            BANK,
            (ADDR_K_MOTHER_BRAIN_PALETTE_0 as i32 + 2) as u16,
            0xf,
            game,
        );
        session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_SET_UP_MOTHER_BRAIN_PHASE_2_BRAIN);
        word!(game.NMI_FLAG_BG2_ENEMY_VRAM_TRANSFER = 1);
        session_slot!(game, MotherBrainBrain[BRAIN].unpause_hook = 1);
    };
    [11111753] fake_death_ascent_set_up_phase2_brain [] {
        word!(game.FX_LAYER_BLENDING_CONFIG_A = 52);
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].function = function::BRAIN_SET_UP_BRAIN_AND_NECK_TO_BE_DRAWN);
        let e0: i32 = 0;
        session_slot!(game, MotherBrainBody[e0].base.properties = (session_slot!(game, MotherBrainBody[e0].base.properties) as i32 & !(K_ENEMY_PROPS_TANGIBLE as i32)) as u16);
        session_slot!(game, MotherBrainBrain[e].base.properties = session_slot!(game, MotherBrainBrain[e].base.properties) & !(0x400));
        session_slot!(game, MotherBrainBrain[e].base.health = 18000);
        session_slot!(game, MotherBrainBody[e0].function = function::FAKE_DEATH_ASCENT_PAUSE_FOR_SUSPENSE);
        session_slot!(game, MotherBrainBody[e0].function_timer = 128);
        fake_death_ascent_pause(game);
    };
    [11111860] fake_death_ascent_load_leg_tiles [] {
        if queue_mother_brain_sprite_tiles(BANK, ADDR_STRU_A98F8F as u16, game) != 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::FAKE_DEATH_ASCENT_CONTINUE_PAUSING_FOR_SUSPENSE);
            fake_death_ascent_continue_pause(game);
        }
    };
    [11111916] fake_death_ascent_start_music_and_earthquake [] {
        set_body_instruction_list(animation::BODY_CROUCHED, game);
        let e1: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e1].base.instruction_timer = 1);
        let e0: i32 = 0;
        session_slot!(game, MotherBrainBody[e0].base.properties = (session_slot!(game, MotherBrainBody[e0].base.properties) as i32 & !(K_ENEMY_PROPS_INVISIBLE as i32)) as u16);
        session_slot!(game, MotherBrainBody[e0].base.x_pos = 59);
        session_slot!(game, MotherBrainBody[e0].base.y_pos = 279);
        word! {game; REG_BG2HOFS = -27i32 as u16; REG_BG2VOFS = -217i32 as u16; }
        queue_music(music::SONG_0, game);
        word! {game; EARTHQUAKE_TYPE = 2; EARTHQUAKE_TIMER = 256; }
        session_slot! { game, MotherBrainBrain[e1]; neck_speed = 80, neck_moving = 1, lower_motion = 8, upper_motion = 6 };
        session_slot!(game, MotherBrainBody[e0].function = function::FAKE_DEATH_ASCENT_RAISE_MOTHER_BRAIN);
    };
    [11120417] phase3_death_stumble_to_middle_of_room_and_drool [] {
        generate_smoky_explosions_around_body(game);
        if make_walk_forwards(2, 0x60, game) & 1 != 0 {
            set_brain_instruction_list(animation::BRAIN_DYING_DROOL, game);
            let e1: i32 = BRAIN;
            session_slot! { game, MotherBrainBrain[e1]; lower_motion = 6, upper_motion = 6 }
            session_slot!(game, MotherBrainBrain[e1].neck_speed = 1280);
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_DISABLE_BRAIN_EFFECTS);
            session_slot!(game, MotherBrainBody[e].function_timer = 32);
        }
    };
    [11120566] phase3_death_fade_out_body [] {
        game.foreign(|game| game.flicker_body());
        generate_mixed_explosions_around_body(game);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[e].function_timer = 16);
            let v2: u16 = (session_slot!(game, MotherBrainBody[e].grey_transition)).wrapping_add(1);
            session_slot!(game, MotherBrainBody[e].grey_transition = v2);
            if game.foreign(|game| game.fade(Fade::ToBlack, (v2).wrapping_sub(1))) & 1 != 0 {
                word!(game.ENEMY_BG2_TILEMAP_SIZE = 710);
                word_transfers!(game, descending, 356; le(TILEMAP_STUFF, 355) <- value(824));
                word!(game.NMI_FLAG_BG2_ENEMY_VRAM_TRANSFER = 1);
                session_slot!(game, MotherBrainBody[e].base.properties = session_slot!(game, MotherBrainBody[e].base.properties) & 0xdeff | 0x100);
                session_slot!(game, MotherBrainBody[e].base.extra_properties = 0);
                session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_FINAL_FEW_EXPLOSIONS);
                session_slot!(game, MotherBrainBody[e].function_timer = 16);
            }
        }
    };
    [11120917] phase3_death_realise_decapitation [] {
        set_brain_instruction_list(animation::BRAIN_DECAPITATED, game);
        session_slot!(game, MotherBrainBrain[BRAIN].function = function::BRAIN_SET_UP_BRAIN_TO_BE_DRAWN);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = 0);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_BRAIN_FALLS_TO_GROUND);
        phase3_death_brain_falls_to_ground(game);
    };
    [11120990] phase3_death_load_corpse_tiles [] {
        if queue_mother_brain_sprite_tiles(BANK, ADDR_STRU_A99003 as u16, game) != 0 {
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_SET_UP_FADE_TO_GREY);
            session_slot!(game, MotherBrainBody[e].function_timer = 32);
        }
    };
    [11121109] phase3_death_corpse_rots_away [] {
        if process_corpse_rotting(BRAIN as u16, game) & 1 != 0 {
            let vram_transfers: u16 = session_slot!(game, Corpse[BRAIN].vram_transfers);
            process_corpse_rotting_vram_transfers(vram_transfers, game);
        } else {
            let mut e1: i32 = BRAIN;
            session_slot!(game, MotherBrainBrain[e1].base.properties = session_slot!(game, MotherBrainBrain[e1].base.properties) & 0xdeff | 0x100);
            session_slot!(game, MotherBrainBrain[e1].base.extra_properties = 0);
            queue_music(music::STOP, game);
            queue_music(music::BOSS_FIGHT_1_DATA, game);
            e1 = 0;
            session_slot!(game, MotherBrainBody[e1].function = function::PHASE_3_DEATH_20_FRAME_DELAY);
            session_slot!(game, MotherBrainBody[e1].function_timer = 20);
            phase3_death_20_frame_delay(game);
        };
    };
    [11121240] phase3_death_load_escape_timer_tiles [] {
        if queue_mother_brain_sprite_tiles(BANK_A6, ADDR_STRU_A6C4CB as u16, game) != 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_3_DEATH_START_ESCAPE_SEQUENCE);
            phase3_death_start_escape_sequence(game);
        }
    };
    [11121261] phase3_death_start_escape_sequence [] {
        if queue_mother_brain_sprite_tiles(BANK, ADDR_STRU_A9902F as u16, game) != 0 {
            copy_rom_colors_to_palette(
                cgram::SPRITE_PALETTE_1,
                BANK,
                (ADDR_K_MOTHER_BRAIN_PALETTE_5 as i32 + 2) as u16,
                0xe,
                game,
            );
            queue_music(music::SONG_2, game);
            word! {game; EARTHQUAKE_TYPE = 5; EARTHQUAKE_TIMER = -1i32 as u16; }
            spawn_palette_effect(ADDR_K_PALFX_FFC9 as u16, game);
            spawn_palette_effect(ADDR_K_PALFX_FFCD as u16, game);
            spawn_palette_effect(ADDR_K_PALFX_FFD1 as u16, game);
            spawn_palette_effect(ADDR_K_PALFX_FFD5 as u16, game);
            session_slot!(game, MotherBrainBrain[BRAIN].unpause_hook = 0);
            start_zebes_escape_typewriter(game);
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function_timer = 32);
            let mut v1: u16 = function::PHASE_3_DEATH_SPAWN_TIME_BOMB_SET_JAPANESE_TEXT;
            if word!(game.JAPANESE_TEXT_FLAG) == 0 {
                v1 = function::PHASE_3_DEATH_TYPE_OUT_ZEBES_ESCAPE_TEXT;
            }
            session_slot!(game, MotherBrainBody[e].function = v1);
        }
    };
    [11121379] phase3_death_type_out_zebes_escape_text [] {
        if type_escape_text(0x2610, game) & 1 != 0 {
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_ESCAPE_DOOR_IS_EXPLODING_START_ESCAPE_TIMER);
            session_slot!(game, MotherBrainBody[e].function_timer = 32);
        }
    };
    [11121401] phase3_death_escape_door_exploding [] {
        generate_escape_door_explosion(game);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0 {
            run_samus_command(samus_command::ENABLE_TIMER_HANDLING, game);
            word!(game.TIMER_STATUS = 2);
            set_boss_bit_for_current_area(2, game);
            set_event_happened(event::ZEBES_TIMEBOMB_SET, game);
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_BLOW_UP_ESCAPE_DOOR);
            let e1: i32 = BRAIN;
            session_slot! { game, MotherBrainBrain[e1]; subfunction = 0, subfunction_timer = 0 }
        }
    };
    [11121450] phase3_death_blow_up_escape_door [] {
        explode_escape_door(game);
        session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_3_DEATH_KEEP_EARTHQUAKE_GOING);
        spawn_plm(0, 0x6, plm::ESCAPE_DOOR, game);
    };
    [11121468] phase3_death_keep_earthquake_going [] {
        if word!(game.EARTHQUAKE_TIMER) == 0 {
            word!(game.EARTHQUAKE_TIMER -= 1);
        }
    };
    [11122561] firing_bomb_decide_on_walking [] {
        if word!(game.RANDOM_NUMBER) >= 0xff80 {
            firing_bomb_decide_on_crouching(game);
            return;
        }
        let mut v0: u16 = 64;
        if word!(game.RANDOM_NUMBER) < 0x6000 {
            v0 = 96;
        }
        let e: i32 = 0;
        if ((v0) as i16).wrapping_sub((session_slot!(game, MotherBrainBody[e].base.x_pos)) as i16) >= 0 || {
            session_slot!(game, MotherBrainBody[e].function_timer = v0);
            make_walk_backwards(v0, 6, game) & 1 != 0
        } {
            firing_bomb_decide_on_crouching(game);
        } else {
            session_slot!(game, MotherBrainBody[e].function = function::FIRING_BOMB_WALKING_BACKWARDS);
        };
    };
    @functions [pub fn (game: &mut impl State)];
    [11122630] firing_bomb_crouch [] {
        if make_crouch(game) & 1 != 0 {
            firing_bomb_fire(game);
        }
    };
    [11122680] firing_bomb_stand_up [] {
        if make_stand_up(game) & 1 != 0 {
            firing_bomb_finish(game);
        }
    };
    [11122702] phase2_firing_laser_position_head_quickly [] {
        let mut v0: i16 = 8;
        let e1: i32 = BRAIN;
        if ((session_slot!(game, MotherBrainBrain[e1].base.y_pos)) as i16).wrapping_sub((game.player_y()) as i16) < 0 {
            v0 = 6;
        }
        session_slot!(game, MotherBrainBrain[e1].lower_motion = v0 as u16);
        session_slot!(game, MotherBrainBrain[e1].upper_motion = v0 as u16);
        session_slot!(game, MotherBrainBrain[e1].neck_speed = 512);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_LASER_POSITION_HEAD_SLOWLY_AND_FIRE);
        session_slot!(game, MotherBrainBody[e].function_timer = 4);
    };
    [11122813] phase2_firing_hand_beam [] {
        let v0: i32 = session_slot!(game, MotherBrainBody[0i32].hand_beam_phase) as i32;
        match v0 {
            0 => phase2_firing_hand_beam_back_up(game),
            1 => phase2_firing_hand_beam_wait_for_any_active_bombs(game),
            2 => phase2_firing_hand_beam_idle(game),
            3 => phase2_firing_hand_beam_finish(game),
            // LEGACY(message-text): hand-written slice-index panic text, kept because verifier cases compare panic text.
            _ => panic!("index out of bounds: the len is 4 but the index is {}", v0),
        }
    };
    [11122923] phase2_firing_rainbow_beam_extend_neck [] {
        set_brain_instruction_list(animation::BRAIN_NEUTRAL_PHASE_2, game);
        let e1: i32 = BRAIN;
        session_slot! { game, MotherBrainBrain[e1]; neck_speed = 64, neck_moving = 1, lower_motion = 2, upper_motion = 4 };
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_START_CHARGING_RAINBOW_BEAM);
        session_slot!(game, MotherBrainBody[e].function_timer = 256);
    };
    @functions [pub fn (game: &mut impl Lease)];
    [11122987] phase2_firing_rainbow_beam_retract_neck [] {
        if walk_backwards_slowly_and_retract_head(0x28, game) & 1 != 0 {
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_WAIT_FOR_BEAM_TO_CHARGE);
            session_slot!(game, MotherBrainBody[e].function_timer = 256);
            phase2_firing_rainbow_beam_wait_for_beam_to_charge(game);
        }
    };
    [11123025] phase2_firing_rainbow_beam_extend_neck_down [] {
        word!(game.COOLDOWN_TIMER = 8);
        let e1: i32 = BRAIN;
        session_slot! { game, MotherBrainBrain[e1]; lower_motion = 6, upper_motion = 6 }
        session_slot!(game, MotherBrainBrain[e1].neck_speed = 1280);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_START_FIRING_RAINBOW_BEAM);
        session_slot!(game, MotherBrainBody[e].function_timer = 16);
        phase2_firing_rainbow_beam_start_firing(game);
    };
    [11123061] phase2_firing_rainbow_beam_start_firing [] {
        aim_rainbow_beam_and_increase_width(game);
        if word!(game.POWER_BOMB_FLAG) == 0 {
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
            if (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0 && word!(game.POWER_BOMB_FLAG) == 0 {
                word!(game.COOLDOWN_TIMER = 0);
                reset_body_rainbow_beam_palette_animation(game);
                set_brain_instruction_list(animation::BRAIN_FIRING_RAINBOW_BEAM, game);
                session_slot!(game, MotherBrainBody[e].beam_width = 512);
                session_slot!(game, MotherBrainBody[e].hdma_object = spawn_rainbow_hdma(game));
                let e1: i32 = BRAIN;
                session_slot! { game, MotherBrainBrain[e1]; neck_speed = 64, neck_moving = 1, lower_motion = 2, upper_motion = 4 };
                session_slot! { game, MotherBrainBody[e]; cooldown = 0, explosion_timer = 0 }
                let mut v4: u16 = 5;
                if ((word!(game.SAMUS_HEALTH)) as i16).wrapping_sub(700) < 0 {
                    v4 = 24;
                }
                run_samus_command(v4, game);
                session_slot!(game, MotherBrainBody[e].beam_sounds_left = 6);
                session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_MOVE_SAMUS_TOWARDS_WALL);
            }
        }
    };
    [11123173] phase2_firing_rainbow_beam_move_samus_towards_wall [] {
        play_rainbow_beam_sound(game);
        handle_body_rainbow_palette(game);
        aim_rainbow_beam_and_increase_width(game);
        handle_rainbow_beam_explosions(game);
        if move_samus_towards_wall_due_to_rainbow_beam(game) & 1 != 0 {
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_1_FRAME_DELAY);
            session_slot!(game, MotherBrainBody[e].function_timer = 0);
        }
    };
    [11123200] phase2_firing_rainbow_beam_delay_one_frame [] {
        play_rainbow_beam_sound(game);
        handle_body_rainbow_palette(game);
        aim_rainbow_beam_and_increase_width(game);
        handle_rainbow_beam_explosions(game);
        move_samus_towards_wall_due_to_rainbow_beam(game);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0 {
            word! { game, 8; EARTHQUAKE_TYPE, EARTHQUAKE_TIMER }
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_START_DRAINING_SAMUS);
        }
    };
    [11123239] phase2_firing_rainbow_beam_start_draining_samus [] {
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_DRAINING_SAMUS);
        session_slot!(game, MotherBrainBody[e].function_timer = 299);
        word! {game; EARTHQUAKE_TIMER = 299; EARTHQUAKE_TYPE = 8; }
        phase2_firing_rainbow_beam_draining_samus(game);
    };
    [11123260] phase2_firing_rainbow_beam_draining_samus [] {
        play_rainbow_beam_sound(game);
        handle_body_rainbow_palette(game);
        aim_rainbow_beam_and_increase_width(game);
        handle_rainbow_beam_explosions(game);
        game.foreign(|game| {
            game.drain_health_by_rainbow_beam();
            game.drain_ammo_by_rainbow_beam();
        });
        move_samus_towards_middle_of_wall(game);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_FINISH_FIRING_RAINBOW_BEAM);
        }
    };
    [11123294] phase2_firing_rainbow_beam_finish_firing [] {
        play_rainbow_beam_sound(game);
        handle_body_rainbow_palette(game);
        aim_rainbow_beam(game);
        handle_rainbow_beam_explosions(game);
        let e: i32 = 0;
        let v1: u16 = (session_slot!(game, MotherBrainBody[e].beam_width)).wrapping_sub(384);
        session_slot!(game, MotherBrainBody[e].beam_width = v1);
        if (v1 as i32 - 512i32) & 0x8000 != 0 {
            session_slot!(game, MotherBrainBody[e].beam_width = 512);
            session_slot!(game, MotherBrainBody[e].cooldown = -256i32 as u16);
            session_slot!(game, MotherBrainBody[e].explosion_timer = 0);
            game.foreign(|game| crate::hdma_objects::free_hdma_object(session_slot!(game, MotherBrainBody[e].hdma_object), game));
            word!(game.EARTHQUAKE_TIMER = 0);
            set_brain_instruction_list(animation::BRAIN_NEUTRAL_PHASE_2, game);
            set_up_brain_normal_palette(game);
            write_default_palette(game);
            queue_sfx1_max6(sound::SILENCE_LIBRARY_1, game);
            session_slot!(game, MotherBrainBody[e].beam_sound_playing = 0);
            run_samus_command(samus_command::UNLOCK_SAMUS, game);
            word!(game.COOLDOWN_TIMER = 8);
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_LET_SAMUS_FALL);
        }
    };
    [11123396] phase2_firing_rainbow_beam_let_samus_fall [] {
        player_mother_brain(0, game);
        session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_2_FIRING_RAINBOW_BEAM_WAIT_FOR_SAMUS_TO_HIT_GROUND);
        phase2_firing_rainbow_beam_wait_for_samus_to_hit_ground(game);
    };
    [11123409] phase2_firing_rainbow_beam_wait_for_samus_to_hit_ground [] {
        if move_samus_for_falling_after_rainbow_beam(game) & 1 != 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_2_FIRING_RAINBOW_BEAM_LOWER_HEAD);
        }
    };
    [11124037] phase2_finish_samus_off_lower_health [] {
        let v0: u16 = { let amount = 0x50; game.foreign(|game| game.suit_damage(amount)) };
        if ((4 * v0 as i32 + 20 - word!(game.SAMUS_HEALTH) as i32) as i16) >= 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_2_FINISH_SAMUS_OFF_STAND_UP);
        } else if (word!(game.RANDOM_NUMBER) & 0xfff) < 0xfa0 {
            if (session_slot!(game, MotherBrainBody[0i32].base.frame_counter) & 0x1f) == 0 {
                maybe_stand_up_or_lean_down(game);
            }
        } else {
            let mut v1: u16;
            if ((({ let amount = 0xa0; game.foreign(|game| game.suit_damage(amount)) }) as i32 + 20 - word!(game.SAMUS_HEALTH) as i32)
                as i16)
                >= 0
                || {
                    v1 = animation::BRAIN_BOMB_PHASE_2;
                    (word!(game.RANDOM_NUMBER) & 0xfff) < 0xff0
                }
            {
                v1 = animation::BRAIN_TWO_BLUE_RINGS_PHASE_2;
            }
            set_brain_instruction_list(v1, game);
        };
    };
    [11124120] phase2_finish_samus_off_stand_up [] {
        if make_stand_up(game) & 1 != 0 {
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FINISH_SAMUS_OFF_ADMIRE_JOB_WELL_DONE);
            session_slot!(game, MotherBrainBody[e].function_timer = 16);
            phase2_finish_samus_off_admire_job_well_done(game);
        }
    };
    [11124178] phase2_finish_samus_off_charge_and_load_baby_metroid [] {
        if queue_mother_brain_sprite_tiles(BANK, ADDR_STRU_A98FE5 as u16, game) != 0 {
            retract_head(game);
            spawn_baby_metroid_in_cutscene(game);
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FINISH_SAMUS_OFF_FIRE_FINAL_RAINBOW_BEAM);
            session_slot!(game, MotherBrainBody[e].function_timer = 256);
        }
    };
    [11124317] drained_regain_balance [] {
        handle_body_rainbow_palette(game);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[e].function = function::DRAINED_BY_BABY_METROID_FIRING_RAINBOW_BEAM);
            let e1: i32 = BRAIN;
            session_slot!(game, MotherBrainBrain[e1].painful_walk_function = function::PAINFUL_WALK_FORWARDS);
            session_slot!(game, MotherBrainBrain[e1].painful_walk_stage = 0);
            session_slot! { game, MotherBrainBrain[e1]; painful_walk_delay = 2, lower_motion = 2 }
            session_slot!(game, MotherBrainBrain[e1].upper_motion = 4);
        }
    };
    [11124545] drained_move_to_back_of_room [] {
        if make_walk_backwards(0x28, 0, game) & 1 != 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::DRAINED_BY_BABY_METROID_GO_INTO_LOW_POWER_MODE);
            session_slot!(game, MotherBrainBrain[BRAIN].upper_motion = 0);
            drained_go_into_low_power_mode(game);
        }
    };
    [11124922] phase2_revive_self_wake_up [] {
        if make_stand_up(game) & 1 != 0 {
            let e1: i32 = BRAIN;
            session_slot! { game, MotherBrainBrain[e1]; lower_motion = 6, upper_motion = 6 }
            session_slot!(game, MotherBrainBrain[e1].neck_speed = 1280);
            session_slot!(game, MotherBrainBrain[e1].neck_moving = 1);
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_REVIVE_SELF_WAKE_UP_STRETCH);
            session_slot!(game, MotherBrainBody[e].function_timer = 16);
            phase2_revive_self_wake_up_stretch(game);
        }
    };
    @functions [pub fn (game: &mut impl State)];
    [11125063] phase2_revive_self_finish_preparing_murder [] {
        if make_stand_up(game) & 1 != 0 {
            session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_2_MURDER_BABY_METROID_ATTACK_BABY_METROID);
            make_walk_forwards(0xa, 0x50, game);
            phase2_murder_baby_metroid_attack(game);
        }
    };
    [11125084] phase2_murder_baby_metroid_attack [] {
        maybe_stand_up_or_lean_down(game);
        if (word!(game.RANDOM_NUMBER) & 0x8000) != 0 {
            let mut v0: u16 = animation::BRAIN_FOUR_BLUE_RINGS_PHASE_3;
            if session_slot!(game, MotherBrainBrain[BRAIN].baby_metroid) != 0 {
                v0 = animation::BRAIN_ATTACKING_BABY_METROID;
            }
            set_brain_instruction_list(v0, game);
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_MURDER_BABY_METROID_BABY_METROID_ATTACK_COOLDOWN);
            session_slot!(game, MotherBrainBody[e].function_timer = 64);
        }
    };
    [11125134] phase2_prepare_for_final_baby_metroid_attack [] {
        make_stand_up(game);
        make_walk_backwards(0x40, 4, game);
    };
    [11125146] phase2_execute_final_baby_metroid_attack [] {
        set_brain_instruction_list(animation::BRAIN_ATTACKING_BABY_METROID, game);
        session_slot!(game, MotherBrainBody[0i32].function = function::FINAL_BABY_METROID_WAIT);
    };
    @functions [pub fn (game: &mut impl Lease)];
    [11125257] phase3_fighting_main [] {
        if session_slot!(game, MotherBrainBrain[BRAIN].base.health) != 0 {
            phase3_neck_handler(game);
            phase3_walking_handler(game);
            let e: i32 = 0;
            if session_slot!(game, MotherBrainBody[e].pose) == 0 && session_slot!(game, MotherBrainBody[e].attacks_disabled) == 0 && (word!(game.RANDOM_NUMBER) & 0x8000) != 0 {
                let mut v1: u16 = animation::BRAIN_BOMB_PHASE_3;
                if (word!(game.RANDOM_NUMBER) as u8) >= 0x80 {
                    v1 = animation::BRAIN_FOUR_BLUE_RINGS_PHASE_3;
                }
                set_brain_instruction_list(v1, game);
                session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_FIGHTING_ATTACK_COOLDOWN);
                session_slot!(game, MotherBrainBody[e].function_timer = 64);
            }
        } else {
            session_slot!(game, MotherBrainBody[0i32].function = function::PHASE_3_DEATH_MOVE_TO_BACK_OF_ROOM);
        };
    };
    @functions [pub fn (game: &mut impl Lease)];
    @prefix { let e1: i32 = BRAIN; };
    [11123421] phase2_firing_rainbow_beam_lower_head [] {
        session_slot! { game, MotherBrainBrain[e1]; neck_speed = 64, neck_moving = 1, lower_motion = 2, upper_motion = 4 };
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_DECIDE_NEXT_ACTION);
        session_slot!(game, MotherBrainBody[e].function_timer = 128);
    };
    [11124374] drained_firing_rainbow_beam [] {
        if session_slot!(game, MotherBrainBrain[e1].shake_timer) == 0 {
            session_slot!(game, MotherBrainBrain[e1].shake_timer = 50);
        }
        handle_body_rainbow_palette(game);
        run_painful_walking_function(((session_slot!(game, MotherBrainBrain[e1].painful_walk_function)) as u32) | (BANK_BASE), game);
        let v2: u16 = (2u16).wrapping_mul(session_slot!(game, MotherBrainBrain[e1].painful_walk_stage));
        session_slot!(game, MotherBrainBrain[e1].painful_walk_delay = WALKING_ANIMATION_DELAYS[v2 as usize] as u16);
        session_slot!(game, MotherBrainBrain[e1].neck_speed = NECK_ANGLE_DELTAS[(v2 >> 1) as usize]);
        if session_slot!(game, MotherBrainBrain[e1].painful_walk_stage) == 6 {
            let e: i32 = 0;
            session_slot!(game, MotherBrainBody[e].beam_sound_playing = 0);
            session_slot!(game, MotherBrainBrain[e1].brain_palette_enabled = 0);
            write_phase2_death_palette(game);
            queue_sfx1_max6(sound::SILENCE_LIBRARY_1, game);
            session_slot!(game, MotherBrainBody[e].function = function::DRAINED_BY_BABY_METROID_RAINBOW_BEAM_HAS_RUN_OUT);
        }
    };
    [11124494] drained_rainbow_beam_ran_out [] {
        run_painful_walking_function(((session_slot!(game, MotherBrainBrain[e1].painful_walk_function)) as u32) | (BANK_BASE), game);
        if (session_slot!(game, MotherBrainBrain[e1].painful_walk_stage) as i32 - 8i32) & 0x8000 == 0 {
            session_slot!(game, MotherBrainBrain[e1].neck_speed = 64);
            session_slot! { game, MotherBrainBrain[e1]; lower_motion = 8, upper_motion = 8 }
            set_brain_instruction_list(animation::BRAIN_DYING_DROOL, game);
            session_slot!(game, MotherBrainBody[0i32].function = function::DRAINED_BY_BABY_METROID_MOVE_TO_BACK_OF_ROOM);
            drained_move_to_back_of_room(game);
        }
    };
    [11124566] drained_go_into_low_power_mode [] {
        if (session_slot!(game, MotherBrainBrain[e1].lower_motion) | session_slot!(game, MotherBrainBrain[e1].upper_motion)) == 0 {
            session_slot!(game, MotherBrainBrain[e1].drool_enabled = (session_slot!(game, MotherBrainBrain[e1].upper_motion)) | (session_slot!(game, MotherBrainBrain[e1].lower_motion)));
            let e: i32 = 0;
            if session_slot!(game, MotherBrainBody[e].pose) == 0 {
                set_body_instruction_list(animation::BODY_CROUCH_FAST, game);
                session_slot!(game, MotherBrainBody[e].function = function::DRAINED_BY_BABY_METROID_PREPARE_TRANSITION_TO_GREY);
                session_slot!(game, MotherBrainBody[e].function_timer = 64);
            }
        }
    };
    @functions [pub fn (game: &mut impl Lease)];
    @prefix { let e: i32 = 0; };
    [11110578] fake_death_descent_transition_palette_to_grey [] {
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[e].function_timer = 8);
            let v2: u16 = (session_slot!(game, MotherBrainBody[e].grey_transition)).wrapping_add(1);
            session_slot!(game, MotherBrainBody[e].grey_transition = v2);
            if game.foreign(|game| game.fade(Fade::FakeDeathToGray, (v2).wrapping_sub(1))) & 1 != 0 {
                session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_DESCENT_COLLAPSE_TUBES);
            }
        }
        fake_death_descent_collapse_tubes(game);
    };
    handle_fake_death_explosions [] {
        let v1: i16 = ((session_slot!(game, MotherBrainBody[e].fake_death_explosion_timer)) as i16).wrapping_sub(1);
        if v1 < 0 {
            session_slot!(game, MotherBrainBody[e].fake_death_explosion_timer = 8);
            let mut v2: i16 = ((session_slot!(game, MotherBrainBody[e].fake_death_explosion_index)) as i16).wrapping_sub(1);
            if v2 < 0 {
                v2 = 7;
            }
            session_slot!(game, MotherBrainBody[e].fake_death_explosion_index = v2 as u16);
            let v3: i32 = ((4u16).wrapping_mul((v2) as u16)) as i32 >> 1;
            let explosion = mother_brain_tables::fake_death_explosion(game, (v3 >> 1) as isize);
            game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, explosion.x, explosion.y));
            let v4: u16 = (if word!(game.RANDOM_NUMBER) < 0x4000 {
                12
            } else {
                3
            }) as u16;
            spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, v4, game);
            queue_sfx2_max3(sound::SMALL_EXPLOSION, game);
        } else {
            session_slot!(game, MotherBrainBody[e].fake_death_explosion_timer = v1 as u16);
        };
    };
    [11112013] fake_death_ascent_raise [] {
        if !((word!(game.NMI_FRAME_COUNTER_WORD) & 3) != 0 || {
            spawn_dust_clouds_for_ascent(game);
            word!(game.REG_BG2VOFS += 2);
            let v1: u16 = (session_slot!(game, MotherBrainBody[e].base.y_pos)).wrapping_sub(2);
            session_slot!(game, MotherBrainBody[e].base.y_pos = v1);
            v1 >= 0xbd
        }) {
            word!(game.ENEMY_BG2_TILEMAP_SIZE = 320);
            session_slot!(game, MotherBrainBody[e].base.y_pos = 188);
            word!(game.EARTHQUAKE_TIMER = 0);
            game.foreign(|game| crate::hdma_objects::free_hdma_object(session_slot!(game, MotherBrainBody[e].hdma_object), game));
            set_body_instruction_list(animation::BODY_STAND_UP_FROM_CROUCH_SLOW, game);
            session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_ASCENT_WAIT_UNTIL_MOTHER_BRAIN_HAS_FINISHED_UNCROUCHING);
            fake_death_ascent_wait_for_uncrouching(game);
        }
    };
    [11112085] fake_death_ascent_wait_for_uncrouching [] {
        if session_slot!(game, MotherBrainBody[e].pose) == 0 {
            session_slot!(game, MotherBrainBody[e].grey_transition = 0);
            session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_ASCENT_TRANSITION_MOTHER_BRAIN_FROM_GREY_AND_LOWER_HER_HEAD);
            session_slot!(game, MotherBrainBody[e].function_timer = 0);
        }
    };
    [11120353] phase3_death_move_to_back_of_room [] {
        session_slot!(game, MotherBrainBody[e].base.properties = (session_slot!(game, MotherBrainBody[e].base.properties)) | ((K_ENEMY_PROPS_TANGIBLE) as u16));
        let e1: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e1].base.properties = (session_slot!(game, MotherBrainBrain[e1].base.properties)) | (0x400u16));
        session_slot!(game, MotherBrainBody[e].hitboxes = 0);
        if make_walk_backwards(0x28, 6, game) & 1 != 0 {
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_IDLE_WHILST_EXPLODING);
            session_slot!(game, MotherBrainBody[e].function_timer = 128);
            phase3_death_idle_whilst_exploding(game);
        }
    };
    [11120941] phase3_death_brain_falls_to_ground [] {
        let v1: u16 = (session_slot!(game, MotherBrainBody[e].function_timer)).wrapping_add(32);
        session_slot!(game, MotherBrainBody[e].function_timer = v1);
        let e1: i32 = BRAIN;
        let mut v3: i16 = ((session_slot!(game, MotherBrainBrain[e1].base.y_pos)) as i16).wrapping_add((v1.to_ne_bytes()[1]) as i16);
        if (v3 as u16) >= 0xc4 {
            game.foreign(|game| crate::actors::mother_brain_tubes::Host::start_earthquake(game, 2));
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_LOAD_CORPSE_TILES);
            session_slot!(game, MotherBrainBody[e].function_timer = 256);
            v3 = 196;
        }
        session_slot!(game, MotherBrainBrain[e1].base.y_pos = v3 as u16);
    };
    [11121361] phase3_death_spawn_time_bomb_set_text [] {
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_TYPE_OUT_ZEBES_ESCAPE_TEXT);
            spawn_room_projectile(
                ADDR_K_EPROJ_TIME_BOMB_SET_JAPANESE_TEXT as u16,
                function::PHASE_3_DEATH_TYPE_OUT_ZEBES_ESCAPE_TEXT,
                game,
            );
        }
        phase3_death_type_out_zebes_escape_text(game);
    };
    brain_shot [] {
        if session_slot!(game, MotherBrainBody[e].form) != 0 {
            phase2_or_phase3_shot_reaction(game);
            if session_slot!(game, MotherBrainBody[e].form) == 1 {
                game.foreign(|game| game.create_dud());
            } else {
                game.foreign(|game| game.shot_reaction(crate::actors::host::Shot::ReactionOnly));
            }
        } else {
            let v1: u16 = ((SessionRam::byte(game, crate::ram::word_arrays::PROJECTILE_TYPE + 2 * word!(game.COLLISION_DETECTION_INDEX) as usize + 1))
                as u16)
                & (7u16);
            if GLASS_HITS_BY_PROJECTILE[v1 as usize] != 0 {
                add_glass_hits(
                    39,
                    u16::from(GLASS_HITS_BY_PROJECTILE[v1 as usize]),
                    game,
                );
                queue_sfx2_max6(sound::SHOT_IN_PHASE_1, game);
                let mut v2: i16 = 13;
                let e1: i32 = BRAIN;
                let flash_timer: u16 = session_slot!(game, MotherBrainBrain[e1].base.flash_timer);
                if flash_timer != 0 && flash_timer & 1 != 0 {
                    v2 = 14;
                }
                session_slot!(game, MotherBrainBrain[e1].base.flash_timer = v2 as u16);
                game.foreign(|game| game.shot_reaction(crate::actors::host::Shot::ReactionOnly));
            }
        };
    };
    @functions [pub fn (game: &mut impl State)];
    @prefix { let e: i32 = 0; };
    phase3_beam_shot_reaction [] {
        let mut v1: i16 = ((session_slot!(game, MotherBrainBody[e].walk_counter)) as i16).wrapping_sub(266);
        if v1 < 0 {
            session_slot!(game, MotherBrainBrain[BRAIN].neck_function = neck::SETUP_HYPER_BEAM_RECOIL);
            v1 = 0;
            session_slot!(game, MotherBrainBody[e].function_timer = 0);
        }
        session_slot!(game, MotherBrainBody[e].walk_counter = v1 as u16);
    };
    [11122181] phase2_thinking [] {
        let e1: i32 = BRAIN;
        if session_slot!(game, MotherBrainBrain[e1].base.health) == 0 {
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_EXTEND_NECK);
            phase2_firing_rainbow_beam_extend_neck(game);
            return;
        }
        if session_slot!(game, MotherBrainBody[e].pose) == 0 {
            let keep_walking: bool = if (session_slot!(game, MotherBrainBrain[e1].base.health) as i32 - 4500i32) & 0x8000 == 0 {
                word!(game.RANDOM_NUMBER) >= 0x1000
            } else if word!(game.RANDOM_NUMBER) < 0x2000 {
                true
            } else if word!(game.RANDOM_NUMBER) >= 0xa000 {
                false
            } else {
                session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_HAND_BEAM);
                return;
            };
            if keep_walking {
                handle_walking(game)
            } else {
                session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_TRY_ATTACK);
            }
        }
    };
    @functions [pub fn (game: &mut impl Lease)];
    @prefix { let e: i32 = 0; };
    [11122251] phase2_try_attack [] {
        let v0: i16 = (2i16).wrapping_mul((session_slot!(game, MotherBrainBody[e].attack_phase)) as i16);
        if v0 != 0 {
            if v0 == 2 {
                phase2_attack_cooldown(game);
            } else {
                if v0 != 4 {
                    bank_unreachable(game);
                    // The original hangs here.
                    loop {
                        core::hint::spin_loop();
                    }
                }
                phase2_end_attack(game);
            }
        } else {
            session_slot!(game, MotherBrainBody[e].cooldown = 64);
            session_slot!(game, MotherBrainBody[e].attack_phase = session_slot!(game, MotherBrainBody[e].attack_phase).wrapping_add(1));
            if decide_phase2_attack_strategy(game) != 0 {
                return;
            }
            let mut v2: u16 = ADDR_BYTE_A9B6DC as u16;
            let e1: i32 = BRAIN;
            if abs16((session_slot!(game, MotherBrainBrain[e1].base.y_pos) as i32 + 4 - game.player_y() as i32) as u16) < 0x20 {
                v2 = ADDR_BYTE_A9B6DF as u16;
            }
            let mut v4: u16 = 0;
            let v5 = crate::cartridge::native_record(BANK, v2, game);
            if (word!(game.RANDOM_NUMBER) as u8) >= v5.byte(0) {
                v4 = 2;
                if (word!(game.RANDOM_NUMBER) as u8) >= v5.byte(1) {
                    v4 = 4;
                    if (word!(game.RANDOM_NUMBER) as u8) >= v5.byte(2) {
                        v4 = 6;
                    }
                }
            }
            let v6: u16 =
                mother_brain_tables::brain_attack(game, (v4 >> 1) as isize).instruction_list;
            if v6 == animation::BRAIN_BOMB_PHASE_2 {
                if (session_slot!(game, MotherBrainBrain[e1].bombs) as i32 - 1i32) & 0x8000 != 0 {
                    session_slot!(game, MotherBrainBody[e].function = function::FIRING_BOMB_DECIDE_ON_WALKING);
                    firing_bomb_decide_on_walking(game);
                }
            } else if v6 == animation::BRAIN_LASER {
                session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_LASER_POSITION_HEAD_QUICKLY);
                phase2_firing_laser_position_head_quickly(game);
            } else {
                set_brain_instruction_list(v6, game);
            }
        };
    };
    phase2_attack_cooldown [] {
        let fresh9: u16 = session_slot!(game, MotherBrainBody[e].cooldown);
        session_slot!(game, MotherBrainBody[e].cooldown = session_slot!(game, MotherBrainBody[e].cooldown).wrapping_sub(1));
        if fresh9 == 1 {
            session_slot!(game, MotherBrainBody[e].attack_phase = session_slot!(game, MotherBrainBody[e].attack_phase).wrapping_add(1));
        }
    };
    phase2_end_attack [] {
        session_slot!(game, MotherBrainBody[e].attack_phase = 0);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_THINKING);
    };
    [11122604] firing_bomb_walking_backwards [] {
        if make_walk_backwards(session_slot!(game, MotherBrainBody[e].function_timer), 6, game) & 1 != 0 {
            firing_bomb_decide_on_crouching(game);
        }
    };
    play_rainbow_beam_sound [] {
        let mbn_var_15: i16 = session_slot!(game, MotherBrainBody[e].beam_sounds_left) as i16;
        if mbn_var_15 >= 0 {
            session_slot!(game, MotherBrainBody[e].beam_sounds_left = ((mbn_var_15) as u16).wrapping_sub(1));
            queue_sfx1_max6(sound::RAINBOW_BEAM, game);
            session_slot!(game, MotherBrainBody[e].beam_sound_playing = 1);
        }
    };
    aim_rainbow_beam_and_increase_width [] {
        let mut v1: i16 = ((session_slot!(game, MotherBrainBody[e].beam_width)) as i16).wrapping_add(384);
        if (session_slot!(game, MotherBrainBody[e].beam_width) as i32 - 2688i32) & 0x8000 == 0 {
            v1 = 3072;
        }
        session_slot!(game, MotherBrainBody[e].beam_width = v1 as u16);
        aim_rainbow_beam(game);
    };
    handle_rainbow_beam_explosions [] {
        session_slot!(game, MotherBrainBody[e].explosion_timer = session_slot!(game, MotherBrainBody[e].explosion_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBody[e].explosion_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBody[e].explosion_timer = 8);
            session_slot!(game, MotherBrainBody[e].cooldown = session_slot!(game, MotherBrainBody[e].cooldown).wrapping_add(1));
            let v2: i32 = session_slot!(game, MotherBrainBody[e].cooldown) as i32 & 7;
            game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, RAINBOW_BEAM_EXPLOSION_X[v2 as usize] as u16, RAINBOW_BEAM_EXPLOSION_Y[v2 as usize] as u16));
            spawn_room_projectile(
                ADDR_K_EPROJ_MOTHER_BRAIN_RAINBOW_BEAM_EXPLOSION as u16,
                RAINBOW_BEAM_EXPLOSION_Y[v2 as usize] as u16,
                game,
            );
            queue_sfx2_max6(sound::SMALL_EXPLOSION, game);
        }
    };
    [11124280] drained_taken_aback [] {
        session_slot!(game, MotherBrainBody[e].form = 3);
        let e1: i32 = BRAIN;
        session_slot! { game, MotherBrainBrain[e1]; lower_motion = 8, upper_motion = 8 }
        session_slot!(game, MotherBrainBrain[e1].neck_speed = 1792);
        session_slot!(game, MotherBrainBody[e].function = function::DRAINED_BY_BABY_METROID_REGAIN_BALANCE);
        session_slot!(game, MotherBrainBody[e].function_timer = 48);
        drained_regain_balance(game);
    };
    [11124629] drained_transition_to_grey [] {
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
        let v2: bool = (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0;
        if v2 && {
            session_slot!(game, MotherBrainBody[e].function_timer = 16);
            let v3: u16 = (session_slot!(game, MotherBrainBody[e].grey_transition)).wrapping_add(1);
            session_slot!(game, MotherBrainBody[e].grey_transition = v3);
            game.foreign(|game| game.fade(Fade::DrainedToGray, (v3).wrapping_sub(1))) & 1 != 0
        } {
            let e1: i32 = BRAIN;
            session_slot!(game, MotherBrainBrain[e1].base.health = -29536i32 as u16);
            session_slot!(game, MotherBrainBody[e].corpse_state = 1);
            session_slot!(game, MotherBrainBrain[e1].breath_enabled = 0);
            session_slot!(game, MotherBrainBody[e].form = 2);
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_REVIVE_SELF_INANIMATE_GREY);
        }
    };
    [11124825] phase2_revive_self_inanimate_grey [] {
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_REVIVE_SELF_SHOW_SIGNS_OF_LIFE);
        session_slot!(game, MotherBrainBody[e].function_timer = 768);
    };
    [11124866] phase2_revive_self_prepare_transition_from_grey [] {
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_REVIVE_SELF_TRANSITION_FROM_GREY);
        session_slot!(game, MotherBrainBody[e].grey_transition = 0);
        phase2_revive_self_transition_from_grey(game);
    };
    [11124879] phase2_revive_self_transition_from_grey [] {
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
        let v1: bool = (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0;
        if v1 && {
            session_slot!(game, MotherBrainBody[e].function_timer = 16);
            let v2: u16 = (session_slot!(game, MotherBrainBody[e].grey_transition)).wrapping_add(1);
            session_slot!(game, MotherBrainBody[e].grey_transition = v2);
            game.foreign(|game| game.fade(Fade::DrainedFromGray, (v2).wrapping_sub(1))) & 1 != 0
        } {
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_REVIVE_SELF_WAKE_UP);
            session_slot!(game, MotherBrainBrain[BRAIN].brain_palette_enabled = 1);
            set_up_brain_normal_palette(game);
            phase2_revive_self_wake_up(game);
        }
    };
    @functions [pub fn (game: &mut impl State)];
    @prefix { let e: i32 = 0; };
    [11124987] phase2_revive_self_walk_up_to_baby_metroid [] {
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
        let v1: bool = (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0;
        if v1 && make_walk_forwards(4, 0x50, game) & 1 != 0 {
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_REVIVE_SELF_PREPARE_NECK_FOR_BABY_METROID_MURDER);
            session_slot!(game, MotherBrainBody[e].corpse_state = 2);
            session_slot!(game, MotherBrainBrain[BRAIN].health_palette_enabled = 1);
        }
    };
    [11125022] phase2_revive_self_prepare_neck_for_murder [] {
        session_slot!(game, MotherBrainBody[e].ring_volleys = 0);
        let e1: i32 = BRAIN;
        session_slot! { game, MotherBrainBrain[e1]; neck_moving = 1, lower_motion = 2, upper_motion = 4, neck_speed = 64 };
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_REVIVE_SELF_FINISH_PREPARING_FOR_BABY_METROID_MURDER);
        phase2_revive_self_finish_preparing_murder(game);
    };
    [11125199] phase3_recover_make_distance [] {
        session_slot!(game, MotherBrainBody[e].form = 4);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_RECOVER_FROM_CUTSCENE_SET_UP_FOR_FIGHTING);
        session_slot!(game, MotherBrainBody[e].function_timer = 32);
        make_walk_backwards((session_slot!(game, MotherBrainBody[e].base.x_pos)).wrapping_sub(14), 2, game);
    };
    walk_try_to_inch_forward [] {
        let mbn_var_07: u16 = session_slot!(game, MotherBrainBody[e].walk_counter);
        if mbn_var_07 != 0 {
            let v3: u16 = (mbn_var_07).wrapping_add(32);
            session_slot!(game, MotherBrainBody[e].walk_counter = v3);
            if v3 >= 0x100 {
                let v4: u16 = (session_slot!(game, MotherBrainBody[e].base.x_pos)).wrapping_add(1);
                let e1: i32 = BRAIN;
                session_slot!(game, MotherBrainBrain[e1].target_x = v4);
                if make_walk_forwards(
                    ((word!(game.RANDOM_NUMBER) as i32 & 2) + 4) as u16,
                    session_slot!(game, MotherBrainBrain[e1].target_x),
                    game,
                ) & 1
                    != 0
                {
                    session_slot!(game, MotherBrainBody[e].walk_counter = 128);
                }
            }
        } else {
            let v6: u16 = (session_slot!(game, MotherBrainBody[e].base.x_pos)).wrapping_sub(14);
            let e1_0: i32 = BRAIN;
            session_slot!(game, MotherBrainBrain[e1_0].target_x = v6);
            session_slot!(game, MotherBrainBrain[e1_0].walk_function = function::WALK_RETREAT_QUICKLY);
            walk_retreat_quickly(game);
        };
    };
    handle_walking [] {
        if session_slot!(game, MotherBrainBody[e].pose) == 0 {
            let mbn_var_07: u16 = session_slot!(game, MotherBrainBody[e].walk_counter);
            let can_pause_walk: bool = if mbn_var_07 != 0 {
                let v2: u16 = (mbn_var_07).wrapping_add(6);
                session_slot!(game, MotherBrainBody[e].walk_counter = v2);
                !(v2 >= 0x100)
                    && ({
                        if (session_slot!(game, MotherBrainBody[e].base.x_pos) as i32 - 48i32) & 0x8000 == 0 {
                            return;
                        }
                        true
                    })
            } else {
                session_slot!(game, MotherBrainBody[e].walk_counter = 1);
                if (session_slot!(game, MotherBrainBody[e].base.x_pos) as i32 - 48i32) & 0x8000 == 0 {
                    set_body_instruction_list(animation::BODY_WALK_BACKWARDS_FAST, game);
                    return;
                }
                true
            };
            if can_pause_walk && ((word!(game.RANDOM_NUMBER) as i32 & 0xfff) - 4032i32) & 0x8000 != 0 {
                return;
            }
            session_slot!(game, MotherBrainBody[e].walk_counter = 128);
            if (session_slot!(game, MotherBrainBody[e].base.x_pos) as i32 - 128i32) & 0x8000 != 0 {
                set_body_instruction_list(animation::BODY_WALK_FORWARDS_MEDIUM, game);
            }
        }
    };
    begin_screen_flashing [] {
        session_slot!(game, MotherBrainBody[e].room_palette_list = ADDR_STRU_A9D046 as u16);
        session_slot!(game, MotherBrainBody[e].room_palette_timer = 1);
    };
    end_screen_flashing [] {
        session_slot! { game, MotherBrainBody[e]; room_palette_list = 0, room_palette_timer = 0 }
        write_room_palette(ADDR_WORD_A9D082 as u16, game);
    };
    handle_brain_palette [] {
        if session_slot!(game, MotherBrainBody[e].form) != 1 {
            let mbn_var_d_high: i8 = session_slot!(byte game, MotherBrainBody[e].brain_palette_counter[1]) as i8;
            if mbn_var_d_high != 0 {
                session_slot!(byte game, MotherBrainBody[e].brain_palette_counter[1] = ((mbn_var_d_high) as u8).wrapping_sub(1));
            } else {
                let r18: u16 = mother_brain_tables::brain_palette_set(
                    game,
                    (session_slot!(byte game, MotherBrainBody[e].brain_palette_reload[1]) >> 1) as isize,
                )
                .colors;
                session_slot!(byte game, MotherBrainBody[e].brain_palette_counter[1] = session_slot!(game, MotherBrainBody[e].brain_palette_reload) as u8);
                let mbn_var_d: i8 = session_slot!(game, MotherBrainBody[e].brain_palette_counter) as i8;
                if mbn_var_d != 0 || session_slot!(game, MotherBrainBrain[BRAIN].function) == function::BRAIN_SET_UP_BRAIN_AND_NECK_TO_BE_DRAWN
                {
                    let v4: u8 = ((mbn_var_d as i32 + 1) & 7) as u8;
                    session_slot!(byte game, MotherBrainBody[e].brain_palette_counter[0] = v4);
                    let mut v5: u16 = 290;
                    if session_slot!(game, MotherBrainBody[e].brain_palette) != 512 {
                        v5 = 482;
                    }
                    copy_rom_colors_to_palette(v5, BANK, (r18 as i32 + 8 * v4 as i32) as u16, 3, game);
                }
            }
        }
    };
    @forward [pub fn (game: &mut impl Lease)];
    [11110352]
    set_up_brain_to_draw => set_graphics_drawn_hook(DRAW_BRAIN_HOOK, game);
    draw_brain_hook => draw_brain(game);
    @countdown [pub fn (game: &mut impl Lease)];
    before {
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].subfunction_timer = session_slot!(game, MotherBrainBrain[e].subfunction_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBrain[e].subfunction_timer) & 0x8000) != 0;
    [11110787] spawn_top_right_tube_falling {
        game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, 152, 47));
        spawn_room_projectile(TOP_RIGHT_TUBE_FALLING_PROJECTILE, 0x2f, game);
        session_slot!(game, MotherBrainBrain[e].subfunction = -30304i32 as u16);
    };
    [11110837] spawn_top_left_tube_falling {
        game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, 104, 47));
        spawn_room_projectile(
            ADDR_K_EPROJ_MOTHER_BRAIN_TUBE_FALLING_TOP_LEFT as u16,
            0x2f,
            game,
        );
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_CLEAR_CEILING_BLOCK_COLUMN_6);
    };
    [11110887] spawn_tubes_falling_enemy_1 {
        game.foreign(|game| game.spawn_legacy_population(BANK, ADDR_STRU_A98AF5 as u16));
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_CLEAR_BOTTOM_RIGHT_TUBE);
    };
    [11110927] spawn_tubes_falling_enemy_2 {
        game.foreign(|game| game.spawn_legacy_population(BANK, ADDR_STRU_A98B05 as u16));
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_CLEAR_BOTTOM_MIDDLE_LEFT_TUBE);
    };
    [11110967] spawn_top_middle_left_tube_falling {
        game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, 120, 59));
        spawn_room_projectile(
            ADDR_K_EPROJ_MOTHER_BRAIN_TUBE_FALLING_TOP_MIDDLE_LEFT as u16,
            0x3b,
            game,
        );
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_CLEAR_CEILING_TUBE_COLUMN_7);
    };
    [11111017] spawn_top_middle_right_tube_falling {
        game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, 136, 59));
        spawn_room_projectile(
            ADDR_K_EPROJ_MOTHER_BRAIN_TUBE_FALLING_TOP_MIDDLE_RIGHT as u16,
            0x3b,
            game,
        );
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_CLEAR_CEILING_TUBE_COLUMN_8);
    };
    [11111067] spawn_tubes_falling_enemy_3 {
        ({ let (bank, population) = (BANK, TUBES_FALLING_ENEMY_3); game.foreign(|game| game.spawn_legacy_population(bank, population)) });
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_CLEAR_BOTTOM_MIDDLE_RIGHT_TUBE);
    };
    [11111107] spawn_tubes_falling_enemy_4 {
        game.foreign(|game| game.spawn_legacy_population(BANK, ADDR_STRU_A98B25 as u16));
        session_slot!(game, MotherBrainBrain[e].subfunction = function::TUBES_CLEAR_BOTTOM_MIDDLE_TUBES);
    };
    @countdown [pub fn (game: &mut impl Lease)];
    before {
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0;
    [11110441] fake_death_descent_lock_samus_and_set_scroll_region {
        run_samus_command(samus_command::LOCK_SAMUS, game);
        { let value: u16 = SessionRam::byte(game, crate::ram::bytes::SCROLLS) as u16; let at: usize = crate::ram::bytes::SCROLLS; let bytes = value.to_ne_bytes(); SessionRam::set_byte(game, at, bytes[0]); SessionRam::set_byte(game, at + 1, bytes[1]); };
        session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_DESCENT_QUEUE_MOTHER_BRAIN_MUSIC);
        session_slot!(game, MotherBrainBody[e].function_timer = 32);
        fake_death_descent_queue_music(game);
    };
    [11110477] fake_death_descent_queue_music {
        queue_music(music::STOP, game);
        queue_music(music::MOTHER_BRAIN_DATA, game);
        session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_DESCENT_UNLOCK_SAMUS);
        session_slot!(game, MotherBrainBody[e].function_timer = 12);
        fake_death_descent_unlock_samus(game);
    };
    [11110508] fake_death_descent_unlock_samus {
        run_samus_command(samus_command::UNLOCK_SAMUS, game);
        session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_DESCENT_BEGIN_SCREEN_FLASHING_AND_LOWER_ACID);
        session_slot!(game, MotherBrainBody[e].function_timer = 8);
        fake_death_descent_begin_screen_flashing_and_lower_acid(game);
    };
    [11111801] fake_death_ascent_pause {
        session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_ASCENT_PREPARE_MOTHER_BRAIN_FOR_RISING);
        session_slot!(game, MotherBrainBody[e].function_timer = 32);
        fake_death_ascent_prepare_rise(game);
    };
    @countdown [pub fn (game: &mut impl Lease)];
    before {
        generate_smoky_explosions_around_body(game);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0;
    [11120402] phase3_death_idle_whilst_exploding {
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_STUMBLE_TO_MIDDLE_OF_ROOM_AND_DROOL);
    };
    [11120468] phase3_death_disable_brain_effects {
        let e1: i32 = BRAIN;
        session_slot! { game, MotherBrainBrain[e1]; lower_motion = 0, upper_motion = 0, drool_enabled = 0, breath_enabled = 0, brain_palette_enabled = 0, health_palette_enabled = 0 }
        word_transfers!(game, descending, 15; le(PALETTE_BUFFER, 255) <- le(PALETTE_BUFFER, 159));
        game.foreign(|game| game.update_health_palette());
        session_slot!(game, MotherBrainBody[e].brain_palette = 3584);
        session_slot! { game, MotherBrainBrain[e1]; subfunction = 0, subfunction_timer = 0 }
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_SET_UP_BODY_FADE_OUT);
        phase3_death_set_up_body_fade_out(game);
    };
    @countdown [pub fn (game: &mut impl Lease)];
    before {
        generate_mixed_explosions_around_body(game);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0;
    [11120541] phase3_death_set_up_body_fade_out {
        session_slot!(game, MotherBrainBody[e].grey_transition = 0);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_FADE_OUT_BODY);
        session_slot!(game, MotherBrainBody[e].function_timer = 0);
        phase3_death_fade_out_body(game);
    };
    [11120659] phase3_death_final_few_explosions {
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_REALISE_DECAPITATION);
    };
    @countdown [pub fn (game: &mut impl Lease)];
    before {
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0;
    [11122970] phase2_firing_rainbow_beam_start_charging {
        set_brain_instruction_list(animation::BRAIN_CHARGING_RAINBOW_BEAM, game);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_RETRACT_NECK);
        phase2_firing_rainbow_beam_retract_neck(game);
    };
    [11123007] phase2_firing_rainbow_beam_wait_for_beam_to_charge {
        queue_sfx2_max6(sound::SILENCE, game);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_EXTEND_NECK_DOWN);
        phase2_firing_rainbow_beam_extend_neck_down(game);
    };
    [11123462] phase2_firing_rainbow_beam_decide_next_action {
        if (word!(game.SAMUS_HEALTH) as i32 - 400i32) & 0x8000 != 0 {
            make_walk_forwards(0xa, (session_slot!(game, MotherBrainBody[e].base.x_pos)).wrapping_add(16), game);
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FINISH_SAMUS_OFF_GET_SAMUS_TO_LOW_HEALTH);
        } else {
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_RAINBOW_BEAM_EXTEND_NECK);
        }
    };
    @countdown [pub fn (game: &mut impl Lease)];
    before {
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].function_timer = session_slot!(game, MotherBrainBody[e].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, MotherBrainBody[e].function_timer) & 0x8000) != 0;
    [11110532] fake_death_descent_begin_screen_flashing_and_lower_acid {
        begin_screen_flashing(game);
        load_fx_entry(2, game);
        session_slot!(game, MotherBrainBrain[BRAIN].subfunction = function::TUBES_WAIT_AND_SPAWN_TUBES_FALLING_ENEMY_0);
        session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_DESCENT_TRANSITION_MOTHER_BRAIN_PALETTE_TO_GREY);
        session_slot! { game, MotherBrainBody[e]; function_timer = 0, grey_transition = 0 }
        spawn_plm(0xe, 0x2, plm::CLEAR_CEILING_BLOCK, game);
    };
    [11111819] fake_death_ascent_prepare_rise {
        session_slot!(game, MotherBrainBody[e].hdma_object = spawn_rising_hdma(game));
        let e1: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e1].base.properties = (session_slot!(game, MotherBrainBrain[e1].base.properties)) | (0x100u16));
        set_brain_instruction_list(ADDR_STRU_A99C21 as u16, game);
        session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_ASCENT_LOAD_MOTHER_BRAINS_LEGS_TILES);
        session_slot!(game, MotherBrainBody[e].function_timer = 256);
        fake_death_ascent_load_leg_tiles(game);
    };
    [11111875] fake_death_ascent_continue_pause {
        session_slot!(game, MotherBrainBody[e].base.x_pos = 59);
        session_slot!(game, MotherBrainBody[e].base.y_pos = 279);
        word! {game; REG_BG2HOFS = -27i32 as u16; REG_BG2VOFS = -217i32 as u16; }
        session_slot!(game, MotherBrainBody[e].hitboxes = 7);
        session_slot!(game, MotherBrainBody[e].function = function::FAKE_DEATH_ASCENT_START_MUSIC_AND_EARTHQUAKE);
    };
    [11112106] fake_death_ascent_transition_from_grey {
        session_slot!(game, MotherBrainBody[e].function_timer = 4);
        let v3: u16 = (session_slot!(game, MotherBrainBody[e].grey_transition)).wrapping_add(1);
        session_slot!(game, MotherBrainBody[e].grey_transition = v3);
        if game.foreign(|game| game.fade(Fade::FakeDeathFromGray, (v3).wrapping_sub(1))) & 1 != 0 {
            let e1: i32 = BRAIN;
            session_slot!(game, MotherBrainBrain[e1].brain_palette_enabled = 1);
            session_slot!(game, MotherBrainBody[e].form = 2);
            session_slot!(game, MotherBrainBrain[e1].drool_enabled = 1);
            session_slot! { game, MotherBrainBrain[e1]; lower_motion = 6, upper_motion = 6 }
            session_slot!(game, MotherBrainBrain[e1].neck_speed = 1280);
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_STRETCHING_SHAKE_HEAD_MENACINGLY);
            session_slot!(game, MotherBrainBody[e].function_timer = 23);
        }
    };
    [11112181] phase2_stretching_shake_head_menacingly {
        set_brain_instruction_list(animation::BRAIN_STRETCHING_PHASE_2, game);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_STRETCHING_BRING_HEAD_BACK_UP);
        session_slot!(game, MotherBrainBrain[BRAIN].neck_speed = 64);
        session_slot!(game, MotherBrainBody[e].function_timer = 256);
        phase2_stretching_bring_head_back_up(game);
    };
    [11112212] phase2_stretching_bring_head_back_up {
        let e1: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e1].lower_motion = 2);
        session_slot!(game, MotherBrainBrain[e1].upper_motion = 4);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_STRETCHING_FINISH_STRETCHING);
        session_slot!(game, MotherBrainBody[e].function_timer = 64);
        phase2_stretching_finish(game);
    };
    [11112243] phase2_stretching_finish {
        session_slot!(game, MotherBrainBrain[BRAIN].breath_enabled = 1);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_THINKING);
    };
    [11121011] phase3_death_set_up_fade_to_grey {
        session_slot!(game, MotherBrainBody[e].grey_transition = 0);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_FADE_TO_GREY);
        session_slot!(game, MotherBrainBody[e].function_timer = 0);
    };
    [11121033] phase3_death_fade_to_grey {
        let v2: u16 = (session_slot!(game, MotherBrainBody[e].grey_transition)).wrapping_add(1);
        session_slot!(game, MotherBrainBody[e].grey_transition = v2);
        if game.foreign(|game| game.fade(Fade::RealDeathToGray, (v2).wrapping_sub(1))) & 1 != 0 {
            set_brain_instruction_list(animation::BRAIN_CORPSE, game);
            session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_CORPSE_TIPS_OVER);
            session_slot!(game, MotherBrainBody[e].function_timer = 256);
        } else {
            session_slot!(game, MotherBrainBody[e].function_timer = 16);
        }
    };
    [11121080] phase3_death_corpse_tips_over {
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_CORPSE_ROTS_AWAY);
        let e1: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e1].base.properties = (session_slot!(game, MotherBrainBrain[e1].base.properties)) | (0x400u16));
        session_slot!(game, MotherBrainBody[e].hitboxes = 0);
    };
    [11121169] phase3_death_20_frame_delay {
        let e1: i32 = BRAIN;
        session_slot! { game, MotherBrainBrain[e1].base; x_pos = 0, y_pos = 0 }
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_DEATH_LOAD_ESCAPE_TIMER_TILES);
    };
    [11122664] firing_bomb_fired {
        if make_stand_up(game) & 1 != 0 {
            firing_bomb_finish(game);
        } else {
            session_slot!(game, MotherBrainBody[e].function = function::FIRING_BOMB_STAND_UP);
            firing_bomb_stand_up(game);
        }
    };
    [11122745] phase2_firing_laser_position_head_slowly_and_fire {
        let mut v2: i16 = 256;
        let e1: i32 = BRAIN;
        if (session_slot!(game, MotherBrainBrain[e1].neck_speed) & 0x8000) != 0 {
            v2 = -256i16;
        }
        session_slot!(game, MotherBrainBrain[e1].neck_speed = v2 as u16);
        set_brain_instruction_list(animation::BRAIN_LASER, game);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FIRING_LASER_FINISH_ATTACK);
        session_slot!(game, MotherBrainBody[e].function_timer = 16);
    };
    [11122787] phase2_firing_laser_finish_attack {
        let e1: i32 = BRAIN;
        session_slot! { game, MotherBrainBrain[e1]; lower_motion = 4, upper_motion = 4 }
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_THINKING);
        phase2_thinking(game);
    };
    [11124137] phase2_finish_samus_off_admire_job_well_done {
        set_brain_instruction_list(animation::BRAIN_STRETCHING_PHASE_2, game);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FINISH_SAMUS_OFF_CHARGE_FINAL_RAINBOW_BEAM);
        session_slot!(game, MotherBrainBody[e].function_timer = 256);
    };
    [11124161] phase2_finish_samus_off_charge_final_rainbow_beam {
        set_brain_instruction_list(animation::BRAIN_CHARGING_RAINBOW_BEAM, game);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_FINISH_SAMUS_OFF_CHARGE_FINAL_RAINBOW_BEAM_LOAD_BABY_METROID);
        phase2_finish_samus_off_charge_and_load_baby_metroid(game);
    };
    [11124205] phase2_finish_samus_off_fire_final_rainbow_beam {
        reset_body_rainbow_beam_palette_animation(game);
        set_brain_instruction_list(animation::BRAIN_FIRING_RAINBOW_BEAM, game);
        let e1: i32 = BRAIN;
        session_slot! { game, MotherBrainBrain[e1]; lower_motion = 6, upper_motion = 6 }
        session_slot!(game, MotherBrainBrain[e1].neck_speed = 1280);
        queue_sfx2_max6(sound::SILENCE, game);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_CUTSCENE_WAIT);
    };
    [11124605] drained_prepare_transition_to_grey {
        session_slot!(game, MotherBrainBody[e].grey_transition = 0);
        session_slot!(game, MotherBrainBody[e].function = function::DRAINED_BY_BABY_METROID_TRANSITION_TO_GREY);
        session_slot!(game, MotherBrainBody[e].function_timer = 16);
        drained_transition_to_grey(game);
    };
    [11124838] phase2_revive_self_show_signs_of_life {
        let e1: i32 = BRAIN;
        session_slot! { game, MotherBrainBrain[e1]; breath_enabled = 1, drool_enabled = 1 }
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_REVIVE_SELF_PREPARE_TRANSITION_FROM_GREY);
        session_slot!(game, MotherBrainBody[e].function_timer = 224);
        phase2_revive_self_prepare_transition_from_grey(game);
    };
    [11124964] phase2_revive_self_wake_up_stretch {
        set_brain_instruction_list(animation::BRAIN_STRETCHING_PHASE_3, game);
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_REVIVE_SELF_WALK_UP_TO_BABY_METROID);
        session_slot!(game, MotherBrainBody[e].function_timer = 128);
        phase2_revive_self_walk_up_to_baby_metroid(game);
    };
    [11125122] phase2_murder_baby_metroid_cooldown {
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_2_MURDER_BABY_METROID_ATTACK_BABY_METROID);
    };
    [11125232] phase3_recover_set_up_for_fighting {
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_FIGHTING_MAIN);
        let e1: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e1].neck_function = neck::NORMAL);
        session_slot!(game, MotherBrainBrain[e1].walk_function = function::WALK_TRY_TO_INCH_FORWARD);
        phase3_fighting_main(game);
    };
    [11125326] phase3_fighting_attack_cooldown {
        session_slot!(game, MotherBrainBody[e].function = function::PHASE_3_FIGHTING_MAIN);
    };
    @external {
        11110369 => [{
                    bank_unreachable(game);
                }];
        11124250 => [()];
        11125158 => [()];
        11111140 => [()]; 11125133 => [()];
    }
}
pub fn body_unpause_hook(game: &mut impl State) -> CoroutineRet {
    let e: i32 = 0;
    if session_slot!(game, MotherBrainBody[e].beam_sound_playing) != 0 {
        queue_sfx1_max6(sound::RAINBOW_BEAM, game);
    }
    if (session_slot!(game, MotherBrainBody[e].base.extra_properties) & 4) != 0 {
        word! {game; ENEMY_BG2_TILEMAP_SIZE = 2048; NMI_FLAG_BG2_ENEMY_VRAM_TRANSFER = 1; }
    }
    COROUTINE_NONE as CoroutineRet
}
pub fn draw_brain_and_neck_hook(game: &mut impl Lease) {
    {
        draw_brain(game);
        draw_neck(game);
    }
}
pub fn body_phase1(game: &mut impl Lease) {
    {
        if check_event_happened(event::MOTHER_BRAINS_GLASS_DESTROYED, game) != 0 {
            let e1: i32 = BRAIN;
            session_slot!(
                game,
                MotherBrainBrain[e1].shake_timer = word!(game.EARTHQUAKE_TIMER)
            );
            if (game.player_x() as i32 - 236i32) & 0x8000 != 0
                && session_slot!(game, MotherBrainBrain[e1].base.health) == 0
            {
                let e: i32 = 0;
                session_slot! { game, MotherBrainBody[e]; delete_turrets = 1, form = 1 }
                game.foreign(|game| game.disable_minimap_for_boss());
                queue_music(music::SONG_1, game);
                game.foreign(|game| game.seal_wall());
            }
        }
        samus_collision_detection(game);
    }
}
/// The falling tubes run in `crate::actors::mother_brain_tubes`.
pub fn tubes_falling_main_ai(k: u16, game: &mut impl Host) {
    crate::actors::mother_brain_tubes::main_ai(&mut game.lease(), k);
}
crate::dispatch::callback_dispatch! {
    pub fn dispatch_instruction(ea: u32, k: u16, game: &mut impl Lease) -> u16 => ea;
    call (k, game); default bank_unreachable(game) as u16;
    @body checked;
    @functions [pub fn (k: u16, game: &mut impl Lease) -> u16];
    [11115279] go_to [native_record(BANK_BASE | u32::from(k), game).word(0)] { };
    [11115284] enable_neck_movement_and_go_to [native_record(BANK_BASE | u32::from(k), game).word(0)] {
        session_slot!(game, MotherBrainBrain[BRAIN].neck_moving = 1);
    };
    [11115296]
    disable_neck_movement [k] { session_slot!(game, MotherBrainBrain[BRAIN].neck_moving = 0); };
    @functions [pub fn (k: u16, game: &mut impl Lease) -> u16];
    @prefix { let v2 = native_record(BANK_BASE | u32::from(k), game); };
    [11115304] queue_library2_sound [(k).wrapping_add(2)] {  queue_sfx2_max6(v2.word(0), game); };
    [11115314] queue_library3_sound [(k).wrapping_add(2)] {  queue_sfx3_max6(v2.word(0), game); };
    @functions [pub fn (k: u16, game: &mut impl Lease) -> u16];
    [11115324] spawn_drool_projectile [k] {
        let e: i32 = BRAIN;
        if session_slot!(game, MotherBrainBrain[e].drool_enabled) != 0 {
            let mut v2: u16 = (session_slot!(game, MotherBrainBrain[e].drool_parameter)).wrapping_add(1);
            if (session_slot!(game, MotherBrainBrain[e].drool_parameter) as i32 - 5i32) & 0x8000 == 0 {
                v2 = 0;
            }
            session_slot!(game, MotherBrainBrain[e].drool_parameter = v2);
            let v3: u16 = (if (session_slot!(game, MotherBrainBrain[e].neck_speed) as i32 - 128i32) & 0x8000 != 0 {
                ADDR_K_EPROJ_MOTHER_BRAIN_DROOL as i32
            } else {
                ADDR_K_EPROJ_MOTHER_BRAIN_DYING_DROOL as i32
            }) as u16;
            spawn_room_projectile(v3, session_slot!(game, MotherBrainBrain[e].drool_parameter), game);
        }
    };
    [11115373] spawn_big_purple_breath [k] {
        spawn_room_projectile(ADDR_K_EPROJ_MOTHER_BRAIN_PURPLE_BREATH_BIG as u16, 0x9b6d, game);
    };
    [11115383]
    set_brain_main_shake_timer_50 [k] { session_slot!(game, MotherBrainBrain[BRAIN].shake_timer = 50); };
    @functions [pub fn (_k: u16, game: &mut impl Lease) -> u16];
    [11115621] go_to_9c5f_or_occasionally_9c47 [result] {
        let result: u16 = ADDR_STRU_A99C5F as u16;
        if (word!(game.RANDOM_NUMBER) & 0xfff) >= 0xfe0 {
            return ADDR_STRU_A99C47 as u16;
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Lease) -> u16];
    [11115693] likely_go_to_9c9f [k] {
        if word!(game.RANDOM_NUMBER) < 0xf000 {
            return ADDR_STRU_A99C9F as u16;
        }
    };
    [11115789] likely_go_to_9cd1 [k] {
        if (word!(game.RANDOM_NUMBER) & 0xfff) < 0xec0 {
            return go_to_9cd1(k);
        }
    };
    [11116023] queue_baby_metroid_attack_sound [k] {
        if session_slot!(game, MotherBrainBody[0].ring_volleys) != 11 {
            queue_sfx2_max6(BABY_METROID_ATTACK_CRIES[0i32 as usize], game);
        }
    };
    [11116073] spawn_blue_ring_projectile [k] {
        let e: i32 = 0;
        spawn_room_projectile(
            ADDR_K_EPROJ_MOTHER_BRAIN_BLUE_RING_LASERS as u16,
            session_slot!(game, MotherBrainBody[e].ring_target_angle),
            game,
        );
    };
    [11116087] aim_blue_rings_at_baby_metroid [k] {
        let e: i32 = BRAIN;
        let e1: i32 = session_slot!(game, MotherBrainBrain[e].baby_metroid) as i32;
        aim_blue_rings(
            (session_slot!(game, MotherBrainBody[e1].base.x_pos) as i32 - session_slot!(game, MotherBrainBrain[e].base.x_pos) as i32 - 10i32) as u16,
            (session_slot!(game, MotherBrainBody[e1].base.y_pos) as i32 - session_slot!(game, MotherBrainBrain[e].base.y_pos) as i32 - 16i32) as u16,
            game,
        );
    };
    [11116123] aim_blue_rings_at_samus [k] {
        let e: i32 = BRAIN;
        aim_blue_rings(
            (game.player_x() as i32 - session_slot!(game, MotherBrainBrain[e].base.x_pos) as i32 - 10i32) as u16,
            (game.player_y() as i32 - session_slot!(game, MotherBrainBrain[e].base.y_pos) as i32 - 16i32) as u16,
            game,
        );
    };
    [11116195] increment_baby_metroid_attack_counter [k] {
        let e: i32 = 0;
        let mut v3: i16 = ((session_slot!(game, MotherBrainBody[e].ring_volleys)) as i16).wrapping_add(1);
        if (v3 as u16) >= 0xc {
            v3 = 12;
        }
        session_slot!(game, MotherBrainBody[e].ring_volleys = v3 as u16);
    };
    [11116213]
    reset_baby_metroid_attack_counter [k] { session_slot!(game, MotherBrainBody[0].ring_volleys = 0); };
    [11116221] spawn_bomb [(k).wrapping_add(2)] {
        let v2 = native_record(BANK_BASE | u32::from(k), game);
        spawn_room_projectile(ADDR_K_EPROJ_MOTHER_BRAIN_BOMB as u16, v2.word(0), game);
    };
    [11116358] spawn_laser_projectile [k] {
        let e: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e].neck_moving = 0);
        game.foreign(|game| {
            game.set_projectile_spawn_r22(1);
            crate::actors::host::BossEffects::set_projectile_spawn_point(game, (session_slot!(game, MotherBrainBrain[e].base.x_pos)).wrapping_add(16), (session_slot!(game, MotherBrainBrain[e].base.y_pos)).wrapping_add(4));
        });
        spawn_room_projectile(ADDR_STRU_86A17B as u16, 1, game);
    };
    [11116420] spawn_rainbow_beam_charging [k] {
        spawn_room_projectile(
            ADDR_K_EPROJ_MOTHER_BRAIN_RAINBOW_BEAM_CHARGING as u16,
            0x9f84,
            game,
        );
    };
    [11116430] set_up_effects_for_charging_rainbow_beam [k] {
        session_slot!(game, MotherBrainBrain[BRAIN].breath_enabled = 0);
        set_up_brain_palette_for_laser(game);
        queue_sfx2_max6(sound::CHARGING_RAINBOW, game);
    };
    @external {
        11115809 => [go_to_9cd1(k)];
    }
}
pub fn process_brain_instruction_list(game: &mut impl Lease) -> i32 {
    {
        let e: i32 = 0;
        let mut pc: u16 = session_slot!(game, MotherBrainBody[e].brain_instructions);
        if pc & 0x8000 == 0 {
            return -1i32;
        }
        let mut p = native_record(BANK_BASE | u32::from(pc), game);
        if word!(game.TIME_IS_FROZEN_FLAG) != 0 {
            return p.word(2) as i32;
        }
        let mut v5: u16 = p.word(0);
        let mut execute_instruction: bool = (v5 & 0x8000 != 0)
            || ({
                if ((v5) as i16).wrapping_sub(
                    (session_slot!(game, MotherBrainBody[e].brain_instruction_timer)) as i16,
                ) >= 0
                {
                    session_slot!(
                        game,
                        MotherBrainBody[e].brain_instruction_timer =
                            session_slot!(game, MotherBrainBody[e].brain_instruction_timer)
                                .wrapping_add(1)
                    );
                    return p.word(2) as i32;
                }
                pc = (pc).wrapping_add(4);
                false
            });
        loop {
            if execute_instruction {
                pc = dispatch_instruction(((v5) as u32) | (BANK_BASE), (pc).wrapping_add(2), game);
                execute_instruction = false;
            } else {
                p = native_record(BANK_BASE | u32::from(pc), game);
                v5 = p.word(0);
                if v5 & 0x8000 == 0 {
                    break;
                } else {
                    execute_instruction = true;
                }
            }
        }
        session_slot!(game, MotherBrainBody[e].brain_instruction_timer = 1);
        session_slot!(game, MotherBrainBody[e].brain_instructions = pc);
        p.word(2) as i32
    }
}
pub fn draw_neck(game: &mut impl Lease) {
    {
        if (session_slot!(game, MotherBrainBody[0i32].base.properties) as i32
            & K_ENEMY_PROPS_INVISIBLE as i32)
            == 0
        {
            let e: i32 = BRAIN;
            draw_neck_segment(
                session_slot!(game, MotherBrainBrain[e].segment_5_x),
                session_slot!(game, MotherBrainBrain[e].segment_5_y),
                game,
            );
            draw_neck_segment(
                session_slot!(game, MotherBrainBrain[e].segment_4_x),
                session_slot!(game, MotherBrainBrain[e].segment_4_y),
                game,
            );
            draw_neck_segment(
                session_slot!(game, MotherBrainBrain[e].segment_3_x),
                session_slot!(game, MotherBrainBrain[e].segment_3_y),
                game,
            );
            draw_neck_segment(
                session_slot!(game, MotherBrainBrain[e].segment_2_x),
                session_slot!(game, MotherBrainBrain[e].segment_2_y),
                game,
            );
            draw_neck_segment(
                session_slot!(game, MotherBrainBrain[e].segment_1_x),
                session_slot!(game, MotherBrainBrain[e].segment_1_y),
                game,
            );
        }
    }
}
pub fn draw_neck_segment(x: u16, y: u16, game: &mut impl Lease) {
    {
        let v0: i32 =
            (session_slot!(game, MotherBrainBrain[BRAIN].base.flash_timer) as i32 & 6) >> 1;
        add_spritemap_to_oam(
            ADDR_K_MOTHER_BRAIN_SPRMAP_A694 as u16,
            (x).wrapping_add((SHAKE_X_OFFSETS[v0 as usize]) as u16),
            (y).wrapping_add((SHAKE_Y_OFFSETS[v0 as usize]) as u16),
            session_slot!(game, MotherBrainBody[0i32].neck_palette),
            game,
        );
    }
}
pub fn add_spritemap_to_oam(j: u16, r18: u16, r20: u16, r22: u16, game: &mut impl Lease) {
    let source = native_record(BANK_BASE | u32::from(j), game);
    let mut n: i32 = source.word(0) as i32;
    let mut offset: usize = 2;
    let mut idx: i32 = word!(game.OAM_NEXT_PTR) as i32;
    let layer1 = crate::camera::current(game).layer1;
    do_until! { n == 0 ;
        let y: i16 = (r20 as i32 + source.byte(offset + 2) as i8 as i32 - layer1[1] as i32) as i16;
        if y >= 0 {
            let oam: usize = usize::try_from(0x370i32 + idx).expect("OAM entry outside RAM");
            let x: u16 =
                (r18 as i32 + source.word(offset) as i32 - layer1[0] as i32) as u16;
            SessionRam::set_byte(game, oam, x as u8);
            SessionRam::set_byte(game, oam + 1, y as u8);
            set_native_word(oam + 2, r22 | source.word(offset + 3), game);
            word!(bits_i32, game.OAM_EXT[(idx >> 5)] |= (((x as i32 & 0x100) >> 8) | (((source.word(offset) as i16) < 0) as i32 * 2)) << (2 * (idx >> 2 & 7)));
            idx = (idx + 4) & 0x1ff;
        }
        offset += 5;
        n -= 1;
    }
    word!(game.OAM_NEXT_PTR = idx as u16);
}
pub fn go_to_9cd1(_k: u16) -> u16 {
    ADDR_STRU_A99CD1 as u16
}
pub fn aim_blue_rings(x: u16, y: u16, game: &mut impl State) {
    {
        let mut v0: u16 =
            -(crate::actors::queries::angle_from_delta(x, y) as i32 + 0x80i32) as u8 as u16;
        let clamp_ring_angle: bool = if (v0 as i32 - 16i32) & 0x80 == 0 {
            !((v0 as u8) < 0x48)
        } else {
            ((v0 as i32 + 64) & 0x80 != 0)
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
crate::dispatch::forward_calls! { pub fn (game: &mut impl Lease);
    generate_smoky_explosions_around_body =>
        generate_body_explosions(0x10, ADDR_WORD_A9B10F as u16, 2, game);
    generate_mixed_explosions_around_body =>
        generate_body_explosions(8, ADDR_WORD_A9B109 as u16, 4, game);
}
pub fn explode_escape_door(game: &mut impl Lease) {
    {
        for v0 in 0u16..8 {
            spawn_room_projectile(
                ADDR_K_EPROJ_MOTHER_BRAIN_EXPLODED_ESCAPE_DOOR as u16,
                v0,
                game,
            );
        }
    }
}
pub fn samus_collision_detection(game: &mut impl Lease) {
    {
        let e0: i32 = 0;
        let mut r26: u16 = session_slot!(game, MotherBrainBody[e0].hitboxes);
        let mut v1: i8 = ((r26) as i8) & (1i8);
        r26 >>= 1;
        if v1 == 0
            || part_samus_collision_detection(
                ADDR_WORD_A9B427 as u16,
                session_slot!(game, MotherBrainBody[e0].base.x_pos),
                session_slot!(game, MotherBrainBody[e0].base.y_pos),
                game,
            ) & 1
                == 0
        {
            let e: i32 = BRAIN;
            v1 = ((r26) as i8) & (1i8);
            r26 >>= 1;
            if v1 == 0
                || part_samus_collision_detection(
                    BRAIN_HITBOX,
                    session_slot!(game, MotherBrainBrain[e].base.x_pos),
                    session_slot!(game, MotherBrainBrain[e].base.y_pos),
                    game,
                ) & 1
                    == 0
            {
                v1 = ((r26) as i8) & (1i8);
                if v1 != 0
                    && part_samus_collision_detection(
                        NECK_HITBOX,
                        session_slot!(game, MotherBrainBrain[e].segment_2_x),
                        session_slot!(game, MotherBrainBrain[e].segment_2_y),
                        game,
                    ) & 1
                        == 0
                    && part_samus_collision_detection(
                        NECK_HITBOX,
                        session_slot!(game, MotherBrainBrain[e].segment_3_x),
                        session_slot!(game, MotherBrainBrain[e].segment_3_y),
                        game,
                    ) & 1
                        == 0
                {
                    part_samus_collision_detection(
                        NECK_HITBOX,
                        session_slot!(game, MotherBrainBrain[e].segment_4_x),
                        session_slot!(game, MotherBrainBrain[e].segment_4_y),
                        game,
                    );
                }
            }
        }
    }
}
pub fn part_samus_collision_detection(k: u16, r18: u16, r20: u16, game: &mut impl Lease) -> u8 {
    use crate::actors::mother_brain_fight::Contact;
    match samus_contact(k, r18, r20, game) {
        Contact::None => 0,
        Contact::Pushed => 1,
        Contact::Hurt => {
            hurt_samus(game);
            1
        }
    }
}
pub fn phase2_or_phase3_shot_reaction(game: &mut impl State) {
    {
        let mut v1: i16;
        let e: i32 = 0;
        if session_slot!(game, MotherBrainBody[e].form) == 4
            && determine_shot_reaction_type(game) == 2
        {
            phase3_beam_shot_reaction(game);
        } else {
            if determine_shot_reaction_type(game) == 1 || {
                v1 = ((session_slot!(game, MotherBrainBody[e].walk_counter)) as i16)
                    .wrapping_sub(256);
                v1 < 0
            } {
                v1 = 0;
            }
            session_slot!(game, MotherBrainBody[e].walk_counter = v1 as u16);
        };
    }
}
pub fn determine_shot_reaction_type(game: &mut impl State) -> u16 {
    SHOT_REACTION_BY_PROJECTILE[(SessionRam::byte(
        game,
        crate::ram::word_arrays::PROJECTILE_TYPE
            + 2 * word!(game.COLLISION_DETECTION_INDEX) as usize
            + 1,
    ) as i32
        & 7) as usize] as u16
}
pub fn hurt_samus(game: &mut impl Lease) {
    {
        deal_contact_damage(game);
        word!(game.SAMUS_INVINCIBILITY_TIMER = 96);
        word!(game.SAMUS_KNOCKBACK_TIMER = 5);
        word!(
            game.KNOCKBACK_X_DIR = (((game.player_x()) as i16).wrapping_sub(
                (session_slot!(
                    game,
                    MotherBrainBody[word!(game.CUR_ENEMY_INDEX) as i32]
                        .base
                        .x_pos
                )) as i16,
            ) >= 0) as u16
        );
    }
}
pub fn decide_phase2_attack_strategy(game: &mut impl Lease) -> u8 {
    {
        if ATTACK_STRATEGY_BY_SAMUS_MOVEMENT
            [SessionRam::byte(game, crate::ram::bytes::SAMUS_MOVEMENT_TYPE) as usize]
            != 0
        {
            if (word!(game.RANDOM_NUMBER) as u8 as i32 - 128i32) & 0x8000 != 0 {
                0
            } else {
                let e1: i32 = BRAIN;
                if (session_slot!(game, MotherBrainBrain[e1].bombs) as i32 - 1i32) & 0x8000 != 0 {
                    session_slot!(
                        game,
                        MotherBrainBody[0i32].function = function::FIRING_BOMB_DECIDE_ON_WALKING
                    );
                    firing_bomb_decide_on_walking(game);
                    return 1;
                }
                0
            }
        } else {
            if (word!(game.RANDOM_NUMBER) as u8 as i32 - 128i32) & 0x8000 != 0 {
                session_slot!(
                    game,
                    MotherBrainBody[0i32].function =
                        function::PHASE_2_FIRING_LASER_POSITION_HEAD_QUICKLY
                );
                phase2_firing_laser_position_head_quickly(game);
            } else {
                set_brain_instruction_list(animation::BRAIN_FOUR_BLUE_RINGS_PHASE_2, game);
            }
            1
        }
    }
}
pub fn firing_bomb_decide_on_crouching(game: &mut impl Lease) {
    {
        if game.foreign(|game| game.next_random()) >= 0x8000 {
            session_slot!(
                game,
                MotherBrainBody[0i32].function = function::FIRING_BOMB_CROUCH
            );
            firing_bomb_crouch(game);
        } else {
            firing_bomb_fire(game);
        };
    }
}
pub fn firing_bomb_fire(game: &mut impl State) {
    {
        let mut v0: u16 = animation::BRAIN_BOMB_PHASE_2;
        let e: i32 = 0;
        if session_slot!(game, MotherBrainBody[e].corpse_state) != 0 {
            v0 = animation::BRAIN_BOMB_PHASE_3;
        }
        set_brain_instruction_list(v0, game);
        session_slot!(
            game,
            MotherBrainBody[e].function = function::FIRING_BOMB_FIRED_BOMB
        );
        session_slot!(game, MotherBrainBody[e].function_timer = 44);
    }
}
pub fn firing_bomb_finish(game: &mut impl State) {
    let mut v0: u16 = function::PHASE_2_THINKING;
    let e: i32 = 0;
    if session_slot!(game, MotherBrainBody[e].corpse_state) != 0 {
        v0 = function::PHASE_2_MURDER_BABY_METROID_ATTACK_BABY_METROID;
    }
    session_slot!(game, MotherBrainBody[e].function = v0);
}
pub fn phase2_firing_hand_beam_back_up(game: &mut impl State) {
    {
        if make_walk_backwards(0x28, 8, game) & 1 != 0 {
            let e1: i32 = BRAIN;
            session_slot!(game, MotherBrainBrain[e1].lower_motion = 8);
            session_slot!(game, MotherBrainBrain[e1].upper_motion = 6);
            let e: i32 = 0;
            session_slot!(
                game,
                MotherBrainBody[e].hand_beam_phase =
                    session_slot!(game, MotherBrainBody[e].hand_beam_phase).wrapping_add(1)
            );
        }
    }
}
pub fn phase2_firing_hand_beam_wait_for_any_active_bombs(game: &mut impl State) {
    {
        if session_slot!(game, MotherBrainBrain[BRAIN].bombs) == 0 {
            set_body_instruction_list(animation::BODY_HAND_BEAM_MODE, game);
            let e: i32 = 0;
            session_slot!(
                game,
                MotherBrainBody[e].hand_beam_phase =
                    session_slot!(game, MotherBrainBody[e].hand_beam_phase).wrapping_add(1)
            );
        }
    }
}
pub fn phase2_firing_hand_beam_idle(_game: &mut impl State) {}
pub fn phase2_firing_hand_beam_finish(game: &mut impl State) {
    {
        set_brain_instruction_list(animation::BRAIN_NEUTRAL_PHASE_2, game);
        let e1: i32 = BRAIN;
        session_slot!(game, MotherBrainBrain[e1].lower_motion = 2);
        session_slot!(game, MotherBrainBrain[e1].upper_motion = 4);
        let e: i32 = 0;
        session_slot!(game, MotherBrainBody[e].hand_beam_phase = 0);
        session_slot!(
            game,
            MotherBrainBody[e].function = function::PHASE_2_THINKING
        );
    }
}
pub fn walk_backwards_slowly_and_retract_head(a: u16, game: &mut impl Lease) -> u8 {
    {
        let rv: u8 = make_walk_backwards(a, 0xa, game);
        if rv != 0 {
            retract_head(game);
        }
        rv
    }
}
pub fn move_samus_towards_wall_due_to_rainbow_beam(game: &mut impl Lease) -> u8 {
    {
        let v0: u8 = move_samus_horizontally_towards_wall(0x1000, game);
        if v0 == 0 {
            let v1: u16 = y_velocity_at_angle(
                0x1000,
                session_slot!(game, MotherBrainBody[0i32].beam_angle),
            );
            move_samus_vertically_towards_ceiling_floor(v1, game);
        }
        v0
    }
}
pub fn move_samus_towards_middle_of_wall(game: &mut impl Lease) {
    {
        let mut v0: u16 = 64;
        if (124i16).wrapping_sub((game.player_y()) as i16) < 0 {
            v0 = -64i32 as u16;
        }
        move_samus_vertically_towards_ceiling_floor(v0, game);
    }
}
pub fn move_samus_for_falling_after_rainbow_beam(game: &mut impl Lease) -> u8 {
    {
        let e: i32 = 0;
        let mut v1: i16 =
            ((session_slot!(game, MotherBrainBody[e].cooldown)) as i16).wrapping_add(2);
        if v1 >= 0 {
            v1 = 0;
        }
        session_slot!(game, MotherBrainBody[e].cooldown = v1 as u16);
        move_samus_horizontally_towards_wall(v1 as u16, game);
        let v2: u16 = (session_slot!(game, MotherBrainBody[e].explosion_timer)).wrapping_add(24);
        session_slot!(game, MotherBrainBody[e].explosion_timer = v2);
        (move_samus_vertically_towards_ceiling_floor(v2, game)) & (1u8)
    }
}
pub fn reset_body_rainbow_beam_palette_animation(game: &mut impl State) {
    session_slot!(game, MotherBrainBrain[BRAIN].beam_palette_index = 0);
}
pub fn write_default_palette(game: &mut impl State) {
    {
        copy_rom_colors_to_palette(
            cgram::BG_PALETTE_4,
            BANK,
            (ADDR_K_MOTHER_BRAINS_BRAIN_PALETTE as i32 + 2) as u16,
            0xf,
            game,
        );
        copy_rom_colors_to_palette(
            cgram::SPRITE_PALETTE_1,
            BANK,
            (ADDR_K_MOTHER_BRAINS_BRAIN_PALETTE as i32 + 2) as u16,
            0xf,
            game,
        );
        copy_rom_colors_to_palette(
            cgram::SPRITE_PALETTE_3,
            BANK,
            (ADDR_K_MOTHER_BRAIN_PALETTE_0 as i32 + 2) as u16,
            0xf,
            game,
        );
    }
}
pub fn write_phase2_death_palette(game: &mut impl Lease) {
    {
        write_mother_brain_palette(
            mother_brain_tables::rainbow_beam_palette(game, 6).colors,
            game,
        );
    }
}
pub fn spawn_baby_metroid_in_cutscene(game: &mut impl Lease) {
    session_slot!(
        game,
        MotherBrainBrain[BRAIN].baby_metroid = {
            let (bank, population) = (BANK, ADDR_STRU_A9BE28 as u16);
            game.foreign(|game| game.spawn_legacy_population(bank, population))
        }
    );
}
crate::dispatch::callback_dispatch! {
    pub fn run_painful_walking_function(ea: u32, game: &mut impl Lease) => ea;
    call (game); default { bank_unreachable(game); };
    @body checked;
    @countdown [pub fn (game: &mut impl State)];
    before {
        let e: i32 = BRAIN;
        let v1: u16 = (session_slot!(game, MotherBrainBrain[e].painful_walk_timer)).wrapping_sub(1);
        session_slot!(game, MotherBrainBrain[e].painful_walk_timer = v1);
    }
    when v1 == 0;
    [11124712] painful_walking_forwards {
        session_slot!(game, MotherBrainBrain[e].painful_walk_stage = session_slot!(game, MotherBrainBrain[e].painful_walk_stage).wrapping_add(1));
        session_slot!(game, MotherBrainBrain[e].painful_walk_function = function::PAINFUL_WALK_BACKWARDS);
    };
    [11124764] painful_walking_backwards {
        session_slot!(game, MotherBrainBrain[e].painful_walk_stage = session_slot!(game, MotherBrainBrain[e].painful_walk_stage).wrapping_add(1));
        session_slot!(game, MotherBrainBrain[e].painful_walk_function = function::PAINFUL_WALK_FORWARDS);
    };
    @external {
        11124688 => painful_walk_forwards;
        11124740 => painful_walk_backwards;
    }
}
pub fn maybe_stand_up_or_lean_down(game: &mut impl State) {
    {
        let mbn_var_02: u16 = session_slot!(game, MotherBrainBody[0i32].pose);
        if mbn_var_02 != 0 {
            if mbn_var_02 == 6 && (word!(game.RANDOM_NUMBER) as u8) >= 0xc0 {
                make_stand_up(game);
            }
        } else if (word!(game.RANDOM_NUMBER) as u8) >= 0xc0 {
            make_lean_down(game);
        }
    }
}
crate::dispatch::address_dispatch! {
    pub fn run_walk_function(ea: u32, game: &mut impl Lease) => ea;
        call (game); default { bank_unreachable(game); };
    {
        11125354 => walk_try_to_inch_forward;
        11125427 => walk_retreat_quickly;
        11125458 => walk_retreat_slowly;
    }
}
pub fn phase3_walking_handler(game: &mut impl Lease) {
    {
        if session_slot!(game, MotherBrainBody[0i32].pose) == 0 {
            run_walk_function(
                ((session_slot!(game, MotherBrainBrain[BRAIN].walk_function)) as u32) | (BANK_BASE),
                game,
            );
        }
    }
}
pub fn set_walk_to_try_to_inch_forward(a: u16, game: &mut impl State) {
    let e: i32 = 0;
    session_slot!(game, MotherBrainBody[e].walk_counter = a);
    let e1: i32 = BRAIN;
    session_slot!(
        game,
        MotherBrainBrain[e1].walk_function = function::WALK_TRY_TO_INCH_FORWARD
    );
    session_slot!(
        game,
        MotherBrainBrain[e1].target_x =
            (session_slot!(game, MotherBrainBody[e].base.x_pos)).wrapping_add(1)
    );
}
crate::dispatch::callback_dispatch! {
    pub fn run_neck_function(ea: u32, game: &mut impl Lease) => ea;
    call (game); default { bank_unreachable(game); };
    @body checked;
    @functions [pub fn (game: &mut impl Lease)];
    @prefix { let e: i32 = BRAIN; };
    spawn_dust_clouds_for_ascent [] {
        session_slot!(game, MotherBrainBrain[e].subfunction_timer = session_slot!(game, MotherBrainBrain[e].subfunction_timer).wrapping_sub(1));
        if (session_slot!(game, MotherBrainBrain[e].subfunction_timer) & 0x8000) != 0 {
            session_slot!(game, MotherBrainBrain[e].subfunction_timer = 7);
        }
        game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, FAKE_DEATH_EXPLOSION_X[session_slot!(game, MotherBrainBrain[e].subfunction_timer) as usize], 212));
        spawn_room_projectile(
            ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16,
            FAKE_DEATH_EXPLOSION_TYPES[(((word!(game.RANDOM_NUMBER)) & (0x100u16)) >> 8) as usize] as u16,
            game,
        );
        queue_sfx2_max3(sound::RISING_INTO_PHASE_2, game);
    };
    draw_brain [] {
        if session_slot!(game, MotherBrainBrain[e].breath_enabled) != 0 && session_slot!(game, MotherBrainBrain[e].breath_active) == 0 && (word!(game.RANDOM_NUMBER) & 0x8000) == 0 {
            spawn_room_projectile(
                ADDR_K_EPROJ_MOTHER_BRAIN_PURPLE_BREATH_SMALL as u16,
                word!(game.RANDOM_NUMBER),
                game,
            );
        }
        let tt: i32 = process_brain_instruction_list(game);
        if tt < 0 {
            return;
        }
        let v2: u16 = tt as u16;
        let mut mbn_var_0d: u16 = session_slot!(game, MotherBrainBody[0i32].brain_palette);
        if session_slot!(game, MotherBrainBrain[e].base.flash_timer) & 1 != 0 {
            mbn_var_0d = 0;
        }
        let r22: u16 = mbn_var_0d;
        let mbn_var_00: u16 = session_slot!(game, MotherBrainBrain[e].shake_timer);
        let mut flash_timer: u16;
        if mbn_var_00 != 0 {
            flash_timer = (mbn_var_00).wrapping_sub(1);
            session_slot!(game, MotherBrainBrain[e].shake_timer = flash_timer);
        } else {
            flash_timer = session_slot!(game, MotherBrainBrain[e].base.flash_timer);
            if flash_timer == 0 {
                flash_timer = session_slot!(game, MotherBrainBrain[e].base.shake_timer);
            }
        }
        let v6: i32 = (flash_timer as i32 & 6) >> 1;
        let r18: u16 = (session_slot!(game, MotherBrainBrain[e].base.x_pos)).wrapping_add((SHAKE_X_OFFSETS[v6 as usize]) as u16);
        if ((r18 as i32 + 32 - crate::camera::current(game).layer1[0] as i32) as i16) >= 0 {
            let r20: u16 = (session_slot!(game, MotherBrainBrain[e].base.y_pos)).wrapping_add((SHAKE_Y_OFFSETS[v6 as usize]) as u16);
            add_spritemap_to_oam(v2, r18, r20, r22, game);
        }
    };
    generate_escape_door_explosion [] {
        let v1: i16 = ((session_slot!(game, MotherBrainBrain[e].subfunction)) as i16).wrapping_sub(1);
        session_slot!(game, MotherBrainBrain[e].subfunction = v1 as u16);
        if v1 < 0 {
            session_slot!(game, MotherBrainBrain[e].subfunction = 4);
            let v2: i16 = ((session_slot!(game, MotherBrainBrain[e].subfunction_timer)) as i16).wrapping_sub(1);
            session_slot!(game, MotherBrainBrain[e].subfunction_timer = v2 as u16);
            if v2 < 0 {
                session_slot!(game, MotherBrainBrain[e].subfunction_timer = 3);
            }
            let v3: i32 = ((4u16).wrapping_mul(session_slot!(game, MotherBrainBrain[e].subfunction_timer))) as i32 >> 1;
            game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, ESCAPE_DOOR_EXPLOSION_OFFSETS[v3 as usize], ESCAPE_DOOR_EXPLOSION_OFFSETS[(v3 + 1) as usize]));
            let mut v4: u16 = 3;
            if game.foreign(|game| game.next_random()) < 0x4000 {
                v4 = 12;
            }
            spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, v4, game);
            queue_sfx2_max3(sound::SMALL_EXPLOSION, game);
        }
    };
    @functions [pub fn (game: &mut impl State)];
    @prefix { let e: i32 = BRAIN; };
    aim_rainbow_beam [] {
        session_slot!(game, MotherBrainBody[0i32].beam_angle = -(crate::actors::queries::angle_from_delta((game.player_x() as i32 - session_slot!(game, MotherBrainBrain[e].base.x_pos) as i32 - 16i32) as u16, (game.player_y() as i32 - session_slot!(game, MotherBrainBrain[e].base.y_pos) as i32 - 4i32) as u16) as i32
            + 0x80i32) as u8 as u16);
    };
    painful_walk_forwards [] {
        if make_walk_forwards(session_slot!(game, MotherBrainBrain[e].painful_walk_delay), 0x48, game) & 1 != 0 {
            session_slot!(game, MotherBrainBrain[e].painful_walk_function = function::PAINFUL_WALKING_FORWARDS);
            set_painful_walking_timer(game);
        }
    };
    painful_walk_backwards [] {
        if make_walk_backwards(0x28, session_slot!(game, MotherBrainBrain[e].painful_walk_delay), game) & 1 != 0 {
            session_slot!(game, MotherBrainBrain[e].painful_walk_function = function::PAINFUL_WALKING_BACKWARDS);
            set_painful_walking_timer(game);
        }
    };
    set_painful_walking_timer [] {
        session_slot!(game, MotherBrainBrain[e].painful_walk_timer = PAINFUL_WALKING_TIMERS[session_slot!(game, MotherBrainBrain[e].painful_walk_stage) as usize].to_ne_bytes()[0]
            as u16);
    };
    walk_retreat_quickly [] {
        if make_walk_backwards(session_slot!(game, MotherBrainBrain[e].target_x), 2, game) & 1 != 0 {
            session_slot!(game, MotherBrainBrain[e].target_x = (session_slot!(game, MotherBrainBody[0i32].base.x_pos)).wrapping_sub(14));
            session_slot!(game, MotherBrainBrain[e].walk_function = function::WALK_RETREAT_SLOWLY);
        }
    };
    walk_retreat_slowly [] {
        if make_walk_backwards(session_slot!(game, MotherBrainBrain[e].target_x), 4, game) & 1 != 0 {
            set_walk_to_try_to_inch_forward(0x40, game);
        }
    };
    @functions [pub fn (game: &mut impl Lease)];
    @prefix { let e: i32 = BRAIN; };
    handle_palette [] {
        if session_slot!(game, MotherBrainBrain[e].brain_palette_enabled) != 0 {
            handle_brain_palette(game);
        }
        if session_slot!(game, MotherBrainBrain[e].health_palette_enabled) != 0 {
            game.foreign(|game| game.update_health_palette());
        }
    };
}
/// The neck, on the frame's lease.
pub use crate::actors::mother_brain_neck::{
    handle_neck, retract_head, run_phase3 as phase3_neck_handler,
};
pub fn move_enemy_with_velocity(game: &mut impl State) {
    {
        let v0: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
        let ai_var_b: u16 = session_slot!(game, EnemyData[v0].ai_var_b);
        let mut carry: i32 = session_slot!(byte game, EnemyData[v0].base.x_subpos[1]) as i32
            + ai_var_b.to_ne_bytes()[0] as i32;
        session_slot!(byte game, EnemyData[v0].base.x_subpos[1] = carry as u8);
        session_slot!(
            game,
            EnemyData[v0].base.x_pos = (session_slot!(game, EnemyData[v0].base.x_pos) as i32
                + ((ai_var_b >> 8) as i8 as i32 + (carry >> 8)))
                as u16
        );
        let ai_var_c: u16 = session_slot!(game, EnemyData[v0].ai_var_c);
        carry = session_slot!(byte game, EnemyData[v0].base.y_subpos[1]) as i32
            + ai_var_c.to_ne_bytes()[0] as i32;
        session_slot!(byte game, EnemyData[v0].base.y_subpos[1] = carry as u8);
        session_slot!(
            game,
            EnemyData[v0].base.y_pos = (session_slot!(game, EnemyData[v0].base.y_pos) as i32
                + ((ai_var_c >> 8) as i8 as i32 + (carry >> 8)))
                as u16
        );
    }
}
pub fn set_body_instruction_list(a: u16, game: &mut impl State) {
    let e: i32 = 0;
    session_slot! { game, MotherBrainBody[e].base; current_instruction = a, instruction_timer = 1 }
    session_slot!(game, MotherBrainBody[e].base.timer = 0);
}
pub fn set_brain_unused_instruction_list(a: u16, game: &mut impl State) {
    let e: i32 = BRAIN;
    session_slot! { game, MotherBrainBrain[e].base; current_instruction = a, instruction_timer = 1 }
    session_slot!(game, MotherBrainBrain[e].base.timer = 0);
}
pub fn set_brain_instruction_list(a: u16, game: &mut impl State) {
    let e: i32 = 0;
    session_slot!(game, MotherBrainBody[e].brain_instructions = a);
    session_slot!(game, MotherBrainBody[e].brain_instruction_timer = 1);
}
pub fn set_enemy_instruction_list(k: u16, a: u16, game: &mut impl State) {
    let v2: i32 = k as i32;
    session_slot! { game, EnemyData[v2].base; current_instruction = a, instruction_timer = 1 }
    session_slot!(game, EnemyData[v2].base.timer = 0);
}
pub fn make_walk_forwards(j: u16, a: u16, game: &mut impl State) -> u8 {
    {
        let e: i32 = 0;
        if ((a) as i16).wrapping_sub((session_slot!(game, MotherBrainBody[e].base.x_pos)) as i16)
            >= 0
        {
            if session_slot!(game, MotherBrainBody[e].pose) != 0 {
                return 0;
            }
            if (session_slot!(game, MotherBrainBody[e].base.x_pos) as i32 - 128i32) & 0x8000 != 0 {
                set_body_instruction_list(
                    mother_brain_tables::walk_forwards(game, (j >> 1) as isize).instruction_list,
                    game,
                );
                return 0;
            }
        }
        1
    }
}
pub fn make_walk_backwards(a: u16, j: u16, game: &mut impl State) -> u8 {
    {
        let e: i32 = 0;
        if ((a) as i16).wrapping_sub((session_slot!(game, MotherBrainBody[e].base.x_pos)) as i16)
            < 0
        {
            if session_slot!(game, MotherBrainBody[e].pose) != 0 {
                return 0;
            }
            if (session_slot!(game, MotherBrainBody[e].base.x_pos) as i32 - 48i32) & 0x8000 == 0 {
                set_body_instruction_list(
                    mother_brain_tables::walk_backwards(game, (j >> 1) as isize).instruction_list,
                    game,
                );
                return 0;
            }
        }
        1
    }
}
pub fn make_stand_up(game: &mut impl State) -> u8 {
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
            set_body_instruction_list(v1, game);
            return 0;
        }
        1
    }
}
pub fn make_crouch(game: &mut impl State) -> u8 {
    {
        let mbn_var_02: u16 = session_slot!(game, MotherBrainBody[0i32].pose);
        if mbn_var_02 != 0 {
            if mbn_var_02 == 3 {
                return 1;
            }
        } else {
            set_body_instruction_list(animation::BODY_CROUCH_SLOW, game);
        }
        0
    }
}
pub fn make_lean_down(game: &mut impl State) -> u8 {
    {
        let mbn_var_02: u16 = session_slot!(game, MotherBrainBody[0i32].pose);
        if mbn_var_02 != 0 {
            if mbn_var_02 == 6 {
                return 1;
            }
        } else {
            set_body_instruction_list(animation::BODY_LEAN_DOWN, game);
        }
        0
    }
}
pub fn process_invincibility_palette(game: &mut impl State) {
    if session_slot!(game, MotherBrainBody[0i32].form) == 4 {
        let flash_timer: u16 = session_slot!(game, MotherBrainBrain[BRAIN].base.flash_timer);
        if flash_timer as i32 >> 1 != 0 && flash_timer & 1 == 0 {
            for i in (0i32..=28).rev().step_by(2) {
                let v2: i32 = i >> 1;
                let v3: u16 = word!(game.PALETTE_BUFFER[(v2 + 129)]);
                word! {game; PALETTE_BUFFER[(v2 + 145)] = v3; PALETTE_BUFFER[(v2 + 177)] = v3;
                PALETTE_BUFFER[(v2 + 65)] = v3; }
            }
        }
    }
}
pub fn handle_room_palette(game: &mut impl Lease) {
    {
        let j: u16 = handle_room_palette_instruction_list(ROOM_PALETTE_INSTRUCTIONS, game);
        if j != 0 {
            write_room_palette(j, game);
        }
    }
}
pub fn write_room_palette(j: u16, game: &mut impl State) {
    {
        copy_rom_colors_to_palette(cgram::BG_PALETTE_3_COLOUR_4, BANK, j, 12, game);
        copy_rom_colors_to_palette(
            cgram::BG_PALETTE_5_COLOUR_3,
            BANK,
            (j).wrapping_add(24),
            12,
            game,
        );
        copy_rom_colors_to_palette(
            cgram::BG_PALETTE_7_COLOUR_3,
            BANK,
            (j).wrapping_add(24),
            12,
            game,
        );
    }
}
pub fn handle_room_palette_instruction_list(a: u16, game: &mut impl Lease) -> u16 {
    {
        let st: usize = a as usize;
        if native_word(st, game) & 0x8000 == 0 {
            return 0;
        }
        let mut v2: u16 = native_word(st, game);
        let mut v3 = native_record(BANK_BASE | u32::from(v2), game);
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
                v2 = dispatch_instruction(
                    u32::from(v3.word(0)) | BANK_BASE,
                    v2.wrapping_add(2),
                    game,
                );
                execute_instruction = false;
            } else {
                v3 = native_record(BANK_BASE | u32::from(v2), game);
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
pub fn set_up_brain_normal_palette(game: &mut impl State) {
    session_slot!(game, MotherBrainBody[0i32].brain_palette_reload = 10);
}
pub fn set_up_brain_palette_for_laser(game: &mut impl State) {
    session_slot!(game, MotherBrainBody[0i32].brain_palette_reload = 514);
}

// Mother Brain's AI and instruction entries, forwarded from the bank's dispatch.
pub fn initialize_body(game: &mut impl Lease) {
    word_transfers!(game, descending, 2048; le(TILEMAP_STUFF, 2047) <- value(824));
    set_body_instruction_list(ADDR_K_MOTHER_BRAIN_ILIST_9C13 as u16, game);
    let e: i32 = 0;
    session_slot!(game, MotherBrainBody[e].base.vram_tiles_index = 0);
    session_slot!(
        game,
        MotherBrainBody[e].base.properties =
            (session_slot!(game, MotherBrainBody[e].base.properties) as i32
                | (K_ENEMY_PROPS_BLOCK_PLASMA_BEAM as i32
                    | K_ENEMY_PROPS_TANGIBLE as i32
                    | K_ENEMY_PROPS_INVISIBLE as i32)) as u16
    );
    session_slot!(game, MotherBrainBody[e].base.palette_index = 0);
    copy_rom_colors_to_target_palette(
        BANK,
        cgram::SPRITE_PALETTE_3,
        (ADDR_K_MOTHER_BRAIN_PALETTE_4 as i32 + 2) as u16,
        0xf,
        game,
    );
    copy_rom_colors_to_target_palette(
        BANK,
        cgram::SPRITE_PALETTE_7,
        (ADDR_K_MOTHER_BRAIN_PALETTE_3 as i32 + 2) as u16,
        0xf,
        game,
    );
    session_slot!(game, MotherBrainBody[e].form = 0);
    let e1: i32 = BRAIN;
    session_slot!(game, MotherBrainBrain[e1].unpause_hook = 0);
    session_slot!(game, MotherBrainBody[e].hitboxes = 2);
    session_slot!(
        game,
        MotherBrainBrain[e1].function = function::BRAIN_SET_UP_BRAIN_TO_BE_DRAWN
    );
    session_slot!(game, MotherBrainBody[e].function = function::FIRST_PHASE);
    load_fx_entry(1, game);
    for v3 in 0u16..12 {
        spawn_room_projectile(ADDR_K_EPROJ_MOTHER_BRAIN_ROOM_TURRETS as u16, v3, game);
    }
}
pub fn initialize_brain(game: &mut impl Lease) {
    game.foreign(|game| game.initialize_corpse_rotting(BRAIN as u16, ADDR_STRU_A9DE08 as u16));
    let e1: i32 = BRAIN;
    session_slot!(game, MotherBrainBrain[e1].base.health = 3000);
    set_brain_unused_instruction_list(ADDR_K_MOTHER_BRAIN_ILIST_9C13 as u16, game);
    session_slot! { game, MotherBrainBrain[e1].base; vram_tiles_index = 0, properties = (session_slot!(game, MotherBrainBrain[e1].base.properties)) | (0x1100u16), palette_index = 512 };
    let e: i32 = 0;
    session_slot! { game, MotherBrainBody[e]; neck_palette = 512, brain_palette = 512 }
    set_brain_instruction_list(ADDR_STRU_A99C21 as u16, game);
    set_up_brain_normal_palette(game);
}
/// Main AI - Mother Brain body: the frame runs on one lease.
pub fn body_main_ai(game: &mut impl Host) {
    body_frame(&mut game.lease());
}
fn body_frame(game: &mut impl Lease) {
    handle_room_palette(game);
    let e: i32 = 0;
    if session_slot!(game, MotherBrainBody[e].function) == function::FIRST_PHASE {
        body_phase1(game);
        return;
    }
    run_function(
        ((session_slot!(game, MotherBrainBody[e].function)) as u32) | (BANK_BASE),
        game,
    );
    handle_palette(game);
    samus_collision_detection(game);
    process_invincibility_palette(game);
    if session_slot!(game, MotherBrainBrain[BRAIN].unpause_hook) != 0 {
        write_long_pointer(UNPAUSE_HOOK, UNPAUSE_BODY_HOOK, game);
    }
}
pub fn power_bomb_reaction(game: &mut impl Lease) {
    game.foreign(|game| {
        game.power_bomb_reaction(crate::actors::host::Death::LeavePending);
        brain_main_ai(game);
    });
}
/// Main AI - Mother Brain brain: the frame runs on one lease.
pub fn brain_main_ai(game: &mut impl Host) {
    brain_frame(&mut game.lease());
}
fn brain_frame(game: &mut impl Lease) {
    set_graphics_drawn_hook(NO_DRAW_HOOK, game);
    if (session_slot!(game, MotherBrainBrain[BRAIN].base.properties) & 0x100) != 0 {
        run_function(
            ((session_slot!(game, MotherBrainBrain[BRAIN].function)) as u32) | (BANK_BASE),
            game,
        );
    }
}
pub fn body_shot(game: &mut impl Lease) {
    game.foreign(|game| game.create_dud());
}
pub fn brain_touch(game: &mut impl State) {
    if SessionRam::byte(game, crate::ram::bytes::SAMUS_MOVEMENT_TYPE) == 3 {
        let mut v0: i16 = 13;
        let flash_timer: u16 = session_slot!(game, MotherBrainBrain[BRAIN].base.flash_timer);
        if flash_timer != 0 && flash_timer & 1 != 0 {
            v0 = 14;
        }
        session_slot!(game, MotherBrainBrain[BRAIN].base.flash_timer = v0 as u16);
    }
}
pub fn spawn_dust_cloud_at_offset(_k: u16, jp: EnemyScript, game: &mut impl Lease) -> EnemyScript {
    let e: i32 = 0;
    {
        let (x, y) = (
            ((session_slot!(game, MotherBrainBody[e].base.x_pos)).wrapping_add(jp.word(0))),
            ((session_slot!(game, MotherBrainBody[e].base.y_pos)).wrapping_add(jp.word(2))),
        );
        game.foreign(|game| {
            crate::actors::host::BossEffects::set_projectile_spawn_point(game, x, y)
        });
    }
    spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, jp.word(4), game);
    jp.offset(3)
}
pub fn spawn_hand_beam_projectile(_k: u16, jp: EnemyScript, game: &mut impl Lease) -> EnemyScript {
    let spawn_parameter =
        game.foreign(|game| game.queue_sound_result(1, sound::HAND_BEAM as u8, 6));
    game.foreign(|game| {
        game.spawn_projectile(
            spawn_parameter,
            0x40,
            ADDR_K_EPROJ_MOTHER_BRAIN_DEATH_BEAM_CHARGING as u16,
        )
    });
    jp
}
pub fn increment_hand_beam_attack_phase(
    _k: u16,
    jp: EnemyScript,
    game: &mut impl State,
) -> EnemyScript {
    let e: i32 = 0;
    session_slot!(
        game,
        MotherBrainBody[e].hand_beam_phase =
            session_slot!(game, MotherBrainBody[e].hand_beam_phase).wrapping_add(1)
    );
    jp
}
