//! The cinematic state: the words the title sequence, the intro, the ending
//! and the credits share, by their Session field.
//!
//! # Design
//!
//! **Cohort.** The cinematic RAM names ($7E:198D..$7E:1BA4) reuse the enemy
//! projectile table's bytes outside gameplay, so the Session holds them as
//! `enemy_projectiles`' fields: each [`Word`] and [`Array`] names the field
//! its RAM bytes encode. The Mode 7 zoom ($198F), which the projectile code
//! leaves unused, is `enemy_projectiles.zoom`. The view also names the words
//! and tables of other Session owners the cinematics read and store (the
//! Japanese text option, the layer 1 position, Samus's and her projectiles'
//! words, the palette and the enemy BG2 tilemap they borrow), by the field
//! that holds them.
//!
//! **Consumers.** The cinematic functions run on the compatibility host
//! outside a lease. They read and store these words with [`cinematic!`],
//! through the host's projected bytes: a read sees the owned Session's
//! projection while one is installed and the byte image otherwise, and a
//! store writes the byte image, as the named words they replace did. The
//! view names the words; it does not yet move their access onto the
//! Session's fields, so `tools/cycle_report.py` still counts its call sites
//! as RAM coupling.
use crate::ram::CompatibilityRam;

/// A cinematic word, by its RAM address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    /// `MODE7_ANGLE`: `enemy_projectiles.flags`.
    Mode7Angle = 0x198d,
    /// `MODE7_ZOOM`: `enemy_projectiles.zoom`.
    Mode7Zoom = 0x198f,
    /// `CINEMATIC_X_SUB`: `enemy_projectiles.selected`.
    XSub = 0x1991,
    /// `CINEMATIC_X`: `enemy_projectiles.spawn.primary`.
    X = 0x1993,
    /// `CINEMATIC_Y_SUB`: `enemy_projectiles.spawn.secondary`.
    YSub = 0x1995,
    /// `CINEMATIC_Y`: `enemy_projectiles.slots[0].definition`.
    Y = 0x1997,
    /// `CINEMATIC_X_SUBSPEED`: `enemy_projectiles.slots[1].definition`.
    XSubspeed = 0x1999,
    /// `CINEMATIC_X_SPEED`: `enemy_projectiles.slots[2].definition`.
    XSpeed = 0x199b,
    /// `CINEMATIC_Y_SUBSPEED`: `enemy_projectiles.slots[3].definition`.
    YSubspeed = 0x199d,
    /// `CINEMATIC_Y_SPEED`: `enemy_projectiles.slots[4].definition`.
    YSpeed = 0x199f,
    /// `MODE7_CUR_INDEX`: `enemy_projectiles.slots[13].definition`.
    Mode7ObjectIndex = 0x19b1,
    /// `MODE7_SPAWN_PARAM`: `enemy_projectiles.slots[14].definition`.
    Mode7SpawnParameter = 0x19b3,
    /// `CINEMATIC_BG_OBJECT_INDEX`: `enemy_projectiles.slots[7].counter`.
    BgObjectIndex = 0x19ed,
    /// `CINEMATIC_ENABLE_OBJS`: `enemy_projectiles.slots[9].counter`.
    EnableObjects = 0x19f1,
    /// `CINEMATIC_ENABLE_BG_TILEMAP`: `enemy_projectiles.slots[10].counter`.
    EnableBgTilemap = 0x19f3,
    /// `CINEMATIC_BG_VRAM_ADDRESS`: `enemy_projectiles.slots[11].counter`.
    BgVramAddress = 0x19f5,
    /// `CREDITS_INSTR_LIST`: `enemy_projectiles.slots[12].counter`.
    CreditsInstructionList = 0x19f7,
    /// `CREDITS_INSTRUCTION_TIMER`: `enemy_projectiles.slots[13].counter`.
    CreditsInstructionTimer = 0x19f9,
    /// `CREDITS_TIMER`: `enemy_projectiles.slots[14].counter`.
    CreditsTimer = 0x19fb,
    /// `CREDITS_PRE_INSTRUCTION`: `enemy_projectiles.slots[15].counter`.
    CreditsPreInstruction = 0x19fd,
    /// `CREDITS_ENABLED`: `enemy_projectiles.slots[16].counter`.
    CreditsEnabled = 0x19ff,
    /// `CREDITS_TILEMAP_ROW`: `enemy_projectiles.slots[17].counter`.
    CreditsTilemapRow = 0x1a01,
    /// `LAST_CREDITS_UPDATE_Y`: `enemy_projectiles.slots[0].pre_action`.
    LastCreditsUpdateY = 0x1a03,
    /// `SHOOTING_STARS_ENABLED`: `enemy_projectiles.slots[1].pre_action`.
    ShootingStarsEnabled = 0x1a05,
    /// `TEXT_GLOW_OBJECT_INDEX`: `enemy_projectiles.slots[16].x.fraction`.
    TextGlowObjectIndex = 0x1a47,
    /// `CINEMATIC_FUNCTION_TIMER`: `enemy_projectiles.slots[17].x.fraction`.
    FunctionTimer = 0x1a49,
    /// `CINEMATIC_VAR13`: `enemy_projectiles.slots[0].x.whole`.
    Var13 = 0x1a4b,
    /// `CINEMATIC_VAR14`: `enemy_projectiles.slots[1].x.whole`.
    Var14 = 0x1a4d,
    /// `CINEMATIC_VAR17`: `enemy_projectiles.slots[2].x.whole`.
    Var17 = 0x1a4f,
    /// `CINEMATIC_FRAME_COUNTER`: `enemy_projectiles.slots[3].x.whole`.
    FrameCounter = 0x1a51,
    /// `SKIP_TO_TITLE_INDEX`: `enemy_projectiles.slots[4].x.whole`.
    SkipToTitleIndex = 0x1a53,
    /// `INTRO_SAMUS_DISPLAY`: `enemy_projectiles.slots[6].x.whole`.
    IntroSamusDisplay = 0x1a57,
    /// `CINEMATIC_OBJ_INDEX`: `enemy_projectiles.slots[7].x.whole`.
    ObjectIndex = 0x1a59,
    /// `CINEMATIC_SPAWN_PARAM`: `enemy_projectiles.slots[7].delay`.
    SpawnParameter = 0x1b9d,
    /// `INTRO_FRAME_COUNTER`: `enemy_projectiles.slots[8].delay`.
    IntroFrameCounter = 0x1b9f,
    /// `INTRO_TEXT_CLICK_FLAG`: `enemy_projectiles.slots[9].delay`.
    IntroTextClickFlag = 0x1ba1,
    /// `INTRO_JAPANESE_TEXT_TIMER`: `enemy_projectiles.slots[10].delay`.
    IntroJapaneseTextTimer = 0x1ba3,
    // Other owners' words the cinematics share.
    /// `JAPANESE_TEXT_FLAG`: `options.japanese_text`.
    JapaneseText = 0x09e2,
    /// `LAYER1_X_POS`: `room.camera.layer1[0]`.
    Layer1X = 0x0911,
    /// `LAYER1_Y_POS`: `room.camera.layer1[1]`.
    Layer1Y = 0x0915,
    /// `DOOR_DEF_PTR`: `room.entry.exit`, by its compatibility key, ``.
    DoorExit = 0x078d,
    /// `FX_LAYER_BLENDING_CONFIG_A`: `room.fx_blending`.
    FxBlending = 0x1982,
    /// `HUD_ITEM_INDEX`: `projectiles.controls.selected_weapon`.
    SelectedWeapon = 0x09d2,
    /// `SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX`: `projectiles.controls.automatic_weapon`.
    AutomaticWeapon = 0x0a04,
    /// `POWER_BOMB_EXPLOSION_X_POS`: `projectiles.power_bomb.x`.
    PowerBombX = 0x0ce2,
    /// `POWER_BOMB_EXPLOSION_Y_POS`: `projectiles.power_bomb.y`.
    PowerBombY = 0x0ce4,
    /// `SUBSTATE`: `shared_sequence.stage`.
    SequenceStage = 0x0dec,
    /// `SUIT_PICKUP_LIGHT_BEAM_POS`: `shared_sequence.value`.
    SequenceValue = 0x0dee,
    /// `SAMUS_POSE`: `player.pose`.
    SamusPose = 0x0a1c,
    /// `KNOCKBACK_X_DIR`: `player.knockback_x_direction`.
    KnockbackXDirection = 0x0a54,
    /// `SAMUS_Y_SUBSPEED`: `player.contact.y_speed.fraction`.
    SamusYSubspeed = 0x0b2c,
    /// `SAMUS_Y_SPEED`: `player.contact.y_speed.whole`.
    SamusYSpeed = 0x0b2e,
}

/// A cinematic word table, by the RAM address of its first word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Array {
    /// `MODE7_OBJ_INSTR_PTR`, 2 words from `enemy_projectiles.slots[5].definition`.
    Mode7InstructionPointer = 0x19a1,
    /// `MODE7_OBJ_PREINSTR_FUNC`, 2 words from `enemy_projectiles.slots[7].definition`.
    Mode7PreInstruction = 0x19a5,
    /// `MODE7_OBJ_INSTR_TIMER`, 2 words from `enemy_projectiles.slots[9].definition`.
    Mode7InstructionTimer = 0x19a9,
    /// `MODE7_OBJ_GOTO_TIMER`, 2 words from `enemy_projectiles.slots[11].definition`.
    Mode7GotoTimer = 0x19ad,
    /// `CINEMATICBG_ARR1`, 3 words from `enemy_projectiles.slots[15].definition`.
    BgArr1 = 0x19b5,
    /// `CINEMATICBG_INSTR_PTR`, 4 words from `enemy_projectiles.slots[9].graphics`.
    BgInstructionPointer = 0x19cd,
    /// `CINEMATICBG_PREINSTR`, 4 words from `enemy_projectiles.slots[13].graphics`.
    BgPreInstruction = 0x19d5,
    /// `CINEMATICBG_INSTR_TIMER`, 4 words from `enemy_projectiles.slots[17].graphics`.
    BgInstructionTimer = 0x19dd,
    /// `CINEMATICBG_ARR6`, 4 words from `enemy_projectiles.slots[3].counter`.
    BgArr6 = 0x19e5,
    /// `TEXT_GLOW_INSTRUCTION_POINTERS`, 8 words from `enemy_projectiles.slots[16].counter`.
    TextGlowPointers = 0x19f7,
    /// `TEXT_GLOW_TIMERS`, 8 words from `enemy_projectiles.slots[2].pre_action`.
    TextGlowTimers = 0x1a07,
    /// `TEXT_GLOW_X_POSITIONS`, 8 words from `enemy_projectiles.slots[10].pre_action`.
    TextGlowXPositions = 0x1a17,
    /// `TEXT_GLOW_Y_POSITIONS`, 8 words from `enemy_projectiles.slots[0].x.fraction`.
    TextGlowYPositions = 0x1a27,
    /// `TEXT_GLOW_PALETTES`, 8 words from `enemy_projectiles.slots[8].x.fraction`.
    TextGlowPalettes = 0x1a37,
    /// `CINEMATICSPR_WHATTODRAW`, 16 words from `enemy_projectiles.slots[9].x.whole`.
    SprWhatToDraw = 0x1a5d,
    /// `CINEMATICBG_ARR7`, 16 words from `enemy_projectiles.slots[7].y.fraction`.
    BgArr7 = 0x1a7d,
    /// `CINEMATICBG_ARR8`, 16 words from `enemy_projectiles.slots[5].y.whole`.
    BgArr8 = 0x1a9d,
    /// `CINEMATICBG_ARR9`, 16 words from `enemy_projectiles.slots[3].velocity.x`.
    BgArr9 = 0x1abd,
    /// `CINEMATICSPR_ARR6`, 16 words from `enemy_projectiles.slots[1].velocity.y`.
    SprArr6 = 0x1add,
    /// `CINEMATICSPR_ARR7`, 16 words from `enemy_projectiles.slots[17].velocity.y`.
    SprArr7 = 0x1afd,
    /// `CINEMATICSPR_INSTR_PTR`, 16 words from `enemy_projectiles.slots[15].behavior_a`.
    SprInstructionPointer = 0x1b1d,
    /// `CINEMATICSPR_PREINSTR_FUNC`, 16 words from `enemy_projectiles.slots[13].behavior_b`.
    SprPreInstruction = 0x1b3d,
    /// `CINEMATICSPR_INSTR_TIMER`, 16 words from `enemy_projectiles.slots[11].program`.
    SprInstructionTimer = 0x1b5d,
    /// `CINEMATICSPR_GOTO_TIMER`, 16 words from `enemy_projectiles.slots[9].spritemap`.
    SprGotoTimer = 0x1b7d,
    // Other owners' tables the cinematics share.
    /// `PALETTE_BUFFER`, 256 words: the presentation's colours.
    PaletteBuffer = 0xc000,
    /// `TILEMAP_STUFF`, 2048 words: `actors.extended.blocks[4..6]`.
    TilemapStuff = 0x2000,
    /// `PROJECTILE_X_POS`, 10 words: `projectiles.shots[i].x`.
    ProjectileX = 0x0b64,
    /// `PROJECTILE_TYPE`, 10 words: `projectiles.shots[i].kind`.
    ProjectileType = 0x0c18,
}

/// The cinematic words as the Session's `enemy_projectiles` fields, by their
/// RAM names, for code that runs on the owned table rather than the bytes.
pub struct Cinematic<'a>(pub &'a mut crate::session::EnemyProjectiles);

impl Cinematic<'_> {
    /// $1A49: `cinematic_function_timer`.
    pub fn function_timer(&mut self) -> &mut u16 {
        &mut self.0.slots[17].x.fraction
    }
}

/// Run `action` on the cinematic words as Session fields: the owned
/// Session's under one lease when one can open, otherwise a copy of the
/// table decoded from RAM and stored back if it changed.
pub fn with_fields<H, T>(game: &mut H, action: impl FnOnce(&mut Cinematic<'_>) -> T) -> T
where
    H: crate::ram::WorkRamAccess + crate::runtime_access::RuntimeAccess + ?Sized,
{
    crate::enemy_projectiles::table::with_projectiles(game, |table| action(&mut Cinematic(table)))
}

/// `word` as the host's projection holds it, low byte then high.
pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
    crate::projection::word(h, word as usize)
}

/// Store `value` to `word`, low byte then high, as the named word store did.
pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
    crate::projection::set_word(h, word as usize, value);
}

/// The address of word `index` of `array`; like the named tables, the index
/// may run past the table's end or before its start.
pub fn at<I: TryInto<isize>>(array: Array, index: I) -> usize {
    let index: isize = index.try_into().ok().expect("word index overflow");
    (array as usize)
        .checked_add_signed(index.checked_mul(2).expect("word index overflow"))
        .expect("word address outside RAM")
}

/// Add `value` to the 16.16 pair `high`:`low`, reading and then storing the
/// low word first.
pub fn add_pair<H: CompatibilityRam + ?Sized>(h: &mut H, high: Word, low: Word, value: u32) {
    let pair = u32::from(get(h, low)) | (u32::from(get(h, high)) << 16);
    let [low_0, low_1, high_0, high_1] = pair.wrapping_add(value).to_le_bytes();
    set(h, low, u16::from_le_bytes([low_0, low_1]));
    set(h, high, u16::from_le_bytes([high_0, high_1]));
}

/// Clear every cinematic object word, $7E:1C1D down to $7E:198D, a word at a
/// time, as the cinematics' 329-word clear did.
pub fn clear<H: CompatibilityRam + ?Sized>(h: &mut H) {
    for at in (Word::Mode7Angle as usize..=0x1c1d).rev().step_by(2) {
        crate::projection::set_word(h, at, 0);
    }
}

/// The word `offset` bytes into `array`, for tables their objects index by
/// byte offset.
pub fn get_offset<H: CompatibilityRam + ?Sized>(h: &H, array: Array, offset: usize) -> u16 {
    crate::projection::word(h, array as usize + offset)
}

/// Store `value` to the word `offset` bytes into `array`, low byte then high.
pub fn set_offset<H: CompatibilityRam + ?Sized>(
    h: &mut H,
    array: Array,
    offset: usize,
    value: u16,
) {
    crate::projection::set_word(h, array as usize + offset, value);
}

/// Word `index` of `array`.
pub fn get_at<H: CompatibilityRam + ?Sized, I: TryInto<isize>>(
    h: &H,
    array: Array,
    index: I,
) -> u16 {
    crate::projection::word(h, at(array, index))
}

/// Store `value` to word `index` of `array`, low byte then high.
pub fn set_at<H: CompatibilityRam + ?Sized, I: TryInto<isize>>(
    h: &mut H,
    array: Array,
    index: I,
    value: u16,
) {
    crate::projection::set_word(h, at(array, index), value);
}

/// Access to the cinematic words by name, through the host's projected RAM
/// bytes (not yet the Session's fields), in the shapes the named word
/// accesses used: `cinematic!(host.Word)` reads and `cinematic!(host.Word =
/// value)` stores; tables take `Array[index]`. `+=`/`-=`/`&=`/`|=`/`^=` and
/// `(host.Word, |old| value)` read the old value first (the destination's
/// index before it); a trailing `reread` loads the stored word again. Stores
/// evaluate the value before the destination's index. `(host; A = x; B[i] =
/// y;)` stores in order.
macro_rules! cinematic {
    ($game:ident.$array:ident[$index:expr]) => {
        $crate::cinematic_state::get_at(&*$game, $crate::cinematic_state::Array::$array, $index)
    };
    ($game:ident.$array:ident[$index:expr] = $value:expr $(,)?) => {{
        let value: u16 = $value;
        $crate::cinematic_state::set_at($game, $crate::cinematic_state::Array::$array, $index, value)
    }};
    ($game:ident.$array:ident[$index:expr], |$old:ident| $value:expr $(, $reread:ident)? $(,)?) => {
        $crate::cinematic_state::cinematic!(@update $game,
            $crate::cinematic_state::at($crate::cinematic_state::Array::$array, $index),
            |$old| $value $(, $reread)?)
    };
    ($game:ident.$array:ident[$index:expr] $op:tt $value:expr $(, $reread:ident)? $(,)?) => {
        $crate::cinematic_state::cinematic!(@update $game,
            $crate::cinematic_state::at($crate::cinematic_state::Array::$array, $index),
            |old| $crate::cinematic_state::cinematic!(@op old $op $value) $(, $reread)?)
    };
    ($game:ident.$word:ident) => {
        $crate::cinematic_state::get(&*$game, $crate::cinematic_state::Word::$word)
    };
    ($game:ident.$word:ident = $value:expr $(,)?) => {{
        let value: u16 = $value;
        $crate::cinematic_state::set($game, $crate::cinematic_state::Word::$word, value)
    }};
    ($game:ident.$word:ident, |$old:ident| $value:expr $(, $reread:ident)? $(,)?) => {
        $crate::cinematic_state::cinematic!(@update $game,
            $crate::cinematic_state::Word::$word as usize, |$old| $value $(, $reread)?)
    };
    ($game:ident.$word:ident $op:tt $value:expr $(, $reread:ident)? $(,)?) => {
        $crate::cinematic_state::cinematic!(@update $game,
            $crate::cinematic_state::Word::$word as usize,
            |old| $crate::cinematic_state::cinematic!(@op old $op $value) $(, $reread)?)
    };
    ($game:ident; $($word:ident $([$index:expr])? = $value:expr;)+) => {
        $($crate::cinematic_state::cinematic!($game.$word $([$index])? = $value);)+
    };
    ($game:ident, $value:literal; $($word:ident $([$index:expr])?),+ $(,)?) => {
        $($crate::cinematic_state::cinematic!($game.$word $([$index])? = $value);)+
    };
    (@update $game:ident, $at:expr, |$old:ident| $value:expr) => {{
        let at: usize = $at;
        let $old = $crate::projection::word(&*$game, at);
        let value: u16 = $value;
        $crate::projection::set_word($game, at, value);
    }};
    (@update $game:ident, $at:expr, |$old:ident| $value:expr, reread) => {{
        let at: usize = $at;
        $crate::cinematic_state::cinematic!(@update $game, at, |$old| $value);
        $crate::projection::word(&*$game, at)
    }};
    (@op $old:ident += $value:expr) => { $old.wrapping_add($value) };
    (@op $old:ident -= $value:expr) => { $old.wrapping_sub($value) };
    (@op $old:ident &= $value:expr) => { $old & $value };
    (@op $old:ident |= $value:expr) => { $old | $value };
    (@op $old:ident ^= $value:expr) => { $old ^ $value };
}
pub(crate) use cinematic;

#[cfg(test)]
mod tests {
    use super::{Array, Word};
    use crate::session_encoding::projected_byte;

    /// Every cinematic word, and every word of each table, is a Session field.
    #[test]
    fn the_session_projects_every_cinematic_word() {
        use Word::*;
        let session = crate::session::Session::with_world(
            crate::room_entry::world::WorldBinding::canonical_legacy(),
        );
        let words = [
            Mode7Angle,
            Mode7Zoom,
            XSub,
            X,
            YSub,
            Y,
            XSubspeed,
            XSpeed,
            YSubspeed,
            YSpeed,
            Mode7ObjectIndex,
            Mode7SpawnParameter,
            BgObjectIndex,
            EnableObjects,
            EnableBgTilemap,
            BgVramAddress,
            CreditsInstructionList,
            CreditsInstructionTimer,
            CreditsTimer,
            CreditsPreInstruction,
            CreditsEnabled,
            CreditsTilemapRow,
            LastCreditsUpdateY,
            ShootingStarsEnabled,
            TextGlowObjectIndex,
            FunctionTimer,
            Var13,
            Var14,
            Var17,
            FrameCounter,
            SkipToTitleIndex,
            IntroSamusDisplay,
            ObjectIndex,
            SpawnParameter,
            IntroFrameCounter,
            IntroTextClickFlag,
            IntroJapaneseTextTimer,
            JapaneseText,
            Layer1X,
            Layer1Y,
            DoorExit,
            FxBlending,
            SelectedWeapon,
            AutomaticWeapon,
            PowerBombX,
            PowerBombY,
            SequenceStage,
            SequenceValue,
            SamusPose,
            KnockbackXDirection,
            SamusYSubspeed,
            SamusYSpeed,
        ];
        let tables = [
            (Array::Mode7InstructionPointer, 2),
            (Array::Mode7PreInstruction, 2),
            (Array::Mode7InstructionTimer, 2),
            (Array::Mode7GotoTimer, 2),
            (Array::BgArr1, 3),
            (Array::BgInstructionPointer, 4),
            (Array::BgPreInstruction, 4),
            (Array::BgInstructionTimer, 4),
            (Array::BgArr6, 4),
            (Array::TextGlowPointers, 8),
            (Array::TextGlowTimers, 8),
            (Array::TextGlowXPositions, 8),
            (Array::TextGlowYPositions, 8),
            (Array::TextGlowPalettes, 8),
            (Array::SprWhatToDraw, 16),
            (Array::BgArr7, 16),
            (Array::BgArr8, 16),
            (Array::BgArr9, 16),
            (Array::SprArr6, 16),
            (Array::SprArr7, 16),
            (Array::SprInstructionPointer, 16),
            (Array::SprPreInstruction, 16),
            (Array::SprInstructionTimer, 16),
            (Array::SprGotoTimer, 16),
            (Array::PaletteBuffer, 256),
            (Array::TilemapStuff, 2048),
            (Array::ProjectileX, 10),
            (Array::ProjectileType, 10),
        ];
        let addresses = words.map(|word| word as usize).into_iter().chain(
            tables
                .into_iter()
                .flat_map(|(array, len)| (0..len).map(move |index| super::at(array, index))),
        );
        for at in addresses {
            for byte in [at, at + 1] {
                assert!(projected_byte(&session, byte).is_some(), "{byte:#x}");
            }
        }
    }
}
