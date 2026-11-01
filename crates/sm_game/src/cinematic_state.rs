//! The cinematic state: the words the title sequence, the intro, the ending
//! and the credits share, by their Session field.
//!
//! # Design
//!
//! **Cohort.** The cinematic RAM names ($7E:198D..$7E:1BA4) reuse the enemy
//! projectile table's bytes outside gameplay, so the Session holds them as
//! `enemy_projectiles`' fields: each [`Word`] and [`Array`] names the field
//! its RAM bytes encode. The Mode 7 zoom ($198F), which the projectile code
//! leaves unused, is `enemy_projectiles.zoom`.
//!
//! **Consumers.** The cinematic game states (the title sequence, the intro
//! and Ceres, the ending and credits) each hold a [`Cinematic`] copy of the
//! table for their frame through [`crate::session_cohort::with`]: their
//! functions, objects and pre-instructions read and store its fields by the
//! RAM names, and the words they changed are stored back once the frame's
//! work returns. The cinematics reach the other owners' words they share
//! through those owners' typed copies and Session APIs.

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
}

/// The cinematic words as a copy of the Session's enemy projectile table, by
/// their RAM names, for cinematic code that holds the table for its call
/// through [`crate::session_cohort::with`].
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Cinematic(pub crate::session::EnemyProjectiles);

impl crate::session_cohort::Cohort for Cinematic {
    fn words(&mut self, visit: impl FnMut(usize, &mut u16)) {
        self.0.words(visit);
    }
    fn session_words(session: &mut crate::session::Session, visit: impl FnMut(usize, &mut u16)) {
        crate::session::EnemyProjectiles::session_words(session, visit);
    }
}

impl Cinematic {
    /// The table's word at RAM address `at`.
    fn word_at(&mut self, at: usize) -> &mut u16 {
        self.0
            .word_at(at)
            .expect("cinematic word outside the projectile table")
    }
    /// `word`, for code that selects it.
    pub fn word(&mut self, word: Word) -> &mut u16 {
        self.word_at(word as usize)
    }
    /// Word `index` of `array`, for code that selects the table.
    pub fn array<I: TryInto<isize>>(&mut self, array: Array, index: I) -> &mut u16 {
        self.word_at(at(array, index))
    }
    /// The word `offset` bytes into `array`, for tables their objects index
    /// by byte offset.
    pub fn offset(&mut self, array: Array, offset: usize) -> &mut u16 {
        self.word_at(array as usize + offset)
    }
    /// Add `value` to the 16.16 pair `high`:`low`.
    pub fn add_pair(&mut self, high: Word, low: Word, value: u32) {
        let pair = u32::from(*self.word(low)) | (u32::from(*self.word(high)) << 16);
        let [low_0, low_1, high_0, high_1] = pair.wrapping_add(value).to_le_bytes();
        *self.word(low) = u16::from_le_bytes([low_0, low_1]);
        *self.word(high) = u16::from_le_bytes([high_0, high_1]);
    }
    /// Clear every cinematic object word, $7E:1C1D down to $7E:198D.
    pub fn clear(&mut self) {
        for at in (Word::Mode7Angle as usize..=0x1c1d).step_by(2) {
            *self.word_at(at) = 0;
        }
    }
}

macro_rules! cinematic_fields {
    (words { $($word:ident = $variant:ident;)+ } arrays { $($array:ident = $table:ident;)+ }) => {
        impl Cinematic {
            $(
                #[doc = concat!("[`Word::", stringify!($variant), "`].")]
                pub fn $word(&mut self) -> &mut u16 {
                    self.word_at(Word::$variant as usize)
                }
            )+
            $(
                #[doc = concat!("Word `index` of [`Array::", stringify!($table), "`].")]
                pub fn $array<I: TryInto<isize>>(&mut self, index: I) -> &mut u16 {
                    self.word_at(at(Array::$table, index))
                }
            )+
        }
    };
}

cinematic_fields! {
    words {
    mode7_angle = Mode7Angle;
    mode7_zoom = Mode7Zoom;
    x_sub = XSub;
    x = X;
    y_sub = YSub;
    y = Y;
    x_subspeed = XSubspeed;
    x_speed = XSpeed;
    y_subspeed = YSubspeed;
    y_speed = YSpeed;
    mode7_object_index = Mode7ObjectIndex;
    mode7_spawn_parameter = Mode7SpawnParameter;
    bg_object_index = BgObjectIndex;
    enable_objects = EnableObjects;
    enable_bg_tilemap = EnableBgTilemap;
    bg_vram_address = BgVramAddress;
    credits_instruction_list = CreditsInstructionList;
    credits_instruction_timer = CreditsInstructionTimer;
    credits_timer = CreditsTimer;
    credits_pre_instruction = CreditsPreInstruction;
    credits_enabled = CreditsEnabled;
    credits_tilemap_row = CreditsTilemapRow;
    last_credits_update_y = LastCreditsUpdateY;
    shooting_stars_enabled = ShootingStarsEnabled;
    text_glow_object_index = TextGlowObjectIndex;
    function_timer = FunctionTimer;
    var13 = Var13;
    var14 = Var14;
    var17 = Var17;
    frame_counter = FrameCounter;
    skip_to_title_index = SkipToTitleIndex;
    intro_samus_display = IntroSamusDisplay;
    object_index = ObjectIndex;
    spawn_parameter = SpawnParameter;
    intro_frame_counter = IntroFrameCounter;
    intro_text_click_flag = IntroTextClickFlag;
    intro_japanese_text_timer = IntroJapaneseTextTimer;
    }
    arrays {
    mode7_instruction_pointer = Mode7InstructionPointer;
    mode7_pre_instruction = Mode7PreInstruction;
    mode7_instruction_timer = Mode7InstructionTimer;
    mode7_goto_timer = Mode7GotoTimer;
    bg_arr1 = BgArr1;
    bg_instruction_pointer = BgInstructionPointer;
    bg_pre_instruction = BgPreInstruction;
    bg_instruction_timer = BgInstructionTimer;
    bg_arr6 = BgArr6;
    text_glow_pointers = TextGlowPointers;
    text_glow_timers = TextGlowTimers;
    text_glow_x_positions = TextGlowXPositions;
    text_glow_y_positions = TextGlowYPositions;
    text_glow_palettes = TextGlowPalettes;
    spr_what_to_draw = SprWhatToDraw;
    bg_arr7 = BgArr7;
    bg_arr8 = BgArr8;
    bg_arr9 = BgArr9;
    spr_arr6 = SprArr6;
    spr_arr7 = SprArr7;
    spr_instruction_pointer = SprInstructionPointer;
    spr_pre_instruction = SprPreInstruction;
    spr_instruction_timer = SprInstructionTimer;
    spr_goto_timer = SprGotoTimer;
    }
}

/// The address of word `index` of `array`; like the named tables, the index
/// may run past the table's end or before its start.
pub fn at<I: TryInto<isize>>(array: Array, index: I) -> usize {
    let index: isize = index.try_into().ok().expect("word index overflow");
    (array as usize)
        .checked_add_signed(index.checked_mul(2).expect("word index overflow"))
        .expect("word address outside RAM")
}

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
