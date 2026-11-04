//! Canonical game mode, elapsed game clock, debug controls and saved bindings.
//! Unknown numeric modes are preserved. Input/continuation provenance stays in Control.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Clock {
    pub frames: u16,
    pub seconds: u16,
    pub minutes: u16,
    pub hours: u16,
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct DebugControls {
    pub enabled: u16,
    pub chords: [u16; 2],
    pub ammunition: [u16; 3],
    pub flags: u16,
    pub camera_toggle: u16,
    pub camera: [u16; 2],
}
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct GameLoop {
    pub mode: u16,
    pub clock: Clock,
    pub debug: DebugControls,
    pub directions: [u16; 4],
    pub saved_bindings: [u16; 7],
    initialized: bool,
}
impl GameLoop {
    /// Every value is explicitly supplied by the initialized game or decoded checkpoint.
    pub fn initialized(
        mode: u16,
        clock: Clock,
        debug: DebugControls,
        directions: [u16; 4],
        saved_bindings: [u16; 7],
    ) -> Self {
        Self {
            mode,
            clock,
            debug,
            directions,
            saved_bindings,
            initialized: true,
        }
    }
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    /// Stores a temporary in the saved button configuration's word, as the
    /// grapple and the rotating Ceres elevator leave it there.
    pub fn store_temporary(&mut self, temporary: Temporary, value: u16) {
        self.saved_bindings[temporary as usize] = value;
    }
}
/// The temporaries the original keeps in the saved button configuration
/// ($7E:0D82..$0D8F), one word each, in address order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Temporary {
    /// $0D82: the grapple sample's angle * 2 ($94:A957's input), the
    /// connecting grapple's jump target ($9B:B9B5), or Samus's X position the
    /// rotating Ceres elevator restores ($90:8C27, $90:BC46).
    Angle,
    /// $0D84: the grapple sample's distance from the beam end, or Samus's Y
    /// position the rotating Ceres elevator restores.
    Distance,
    /// $0D86: the swing's next beam end angle * 2 ($94:AD5E, $AE2E).
    NewAngle,
    /// $0D88: the swing's target beam end angle * 2 ($94:AD49, $AE19).
    TargetAngle,
    /// $0D8A: the block collision's remaining substeps ($94:A89D) or the
    /// length change's new beam length ($94:AC6E, $ACCB).
    Counter,
    /// $0D8C: the length change's target beam length ($94:AC53, $ACB0).
    TargetLength,
    /// $0D8E: the length change's Samus front boundary offset ($94:AC59, $ACB6).
    FrontOffset,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    GameState,
    GameTimeFrames,
    GameTimeSeconds,
    GameTimeMinutes,
    GameTimeHours,
    EnableDebug,
    JoypadDbg1,
    JoypadDbg2,
    JoypadDbgMissilesSwap,
    JoypadDbgSuperMissilesSwap,
    JoypadDbgPowerBombsSwap,
    JoypadDbgFlags,
    DebugSaveloadScrollposToggle,
    DebugSavedXscroll,
    DebugSavedYscroll,
    ButtonConfigUp,
    ButtonConfigDown,
    ButtonConfigLeft,
    ButtonConfigRight,
    ButtonConfigShootXSaved,
    ButtonConfigJumpASaved,
    ButtonConfigRunBSaved,
    ButtonConfigItemcancelYSaved,
    ButtonConfigItemswitchSaved,
    ButtonConfigAimDownLSaved,
    ButtonConfigAimUpRSaved,
}
impl Word {
    pub const ALL: &[Self] = &[
        Self::GameState,
        Self::GameTimeFrames,
        Self::GameTimeSeconds,
        Self::GameTimeMinutes,
        Self::GameTimeHours,
        Self::EnableDebug,
        Self::JoypadDbg1,
        Self::JoypadDbg2,
        Self::JoypadDbgMissilesSwap,
        Self::JoypadDbgSuperMissilesSwap,
        Self::JoypadDbgPowerBombsSwap,
        Self::JoypadDbgFlags,
        Self::DebugSaveloadScrollposToggle,
        Self::DebugSavedXscroll,
        Self::DebugSavedYscroll,
        Self::ButtonConfigUp,
        Self::ButtonConfigDown,
        Self::ButtonConfigLeft,
        Self::ButtonConfigRight,
        Self::ButtonConfigShootXSaved,
        Self::ButtonConfigJumpASaved,
        Self::ButtonConfigRunBSaved,
        Self::ButtonConfigItemcancelYSaved,
        Self::ButtonConfigItemswitchSaved,
        Self::ButtonConfigAimDownLSaved,
        Self::ButtonConfigAimUpRSaved,
    ];
    /// Compatibility layout, used only by codecs and native publication adapters.
    pub fn address(self) -> usize {
        match self {
            Self::GameState => crate::ram::words::GAME_STATE,
            Self::GameTimeFrames => crate::ram::words::GAME_TIME_FRAMES,
            Self::GameTimeSeconds => crate::ram::words::GAME_TIME_SECONDS,
            Self::GameTimeMinutes => crate::ram::words::GAME_TIME_MINUTES,
            Self::GameTimeHours => crate::ram::words::GAME_TIME_HOURS,
            Self::EnableDebug => crate::ram::words::ENABLE_DEBUG,
            Self::JoypadDbg1 => crate::ram::words::JOYPAD_DBG_1,
            Self::JoypadDbg2 => crate::ram::words::JOYPAD_DBG_2,
            Self::JoypadDbgMissilesSwap => crate::ram::words::JOYPAD_DBG_MISSILES_SWAP,
            Self::JoypadDbgSuperMissilesSwap => crate::ram::words::JOYPAD_DBG_SUPER_MISSILES_SWAP,
            Self::JoypadDbgPowerBombsSwap => crate::ram::words::JOYPAD_DBG_POWER_BOMBS_SWAP,
            Self::JoypadDbgFlags => crate::ram::words::JOYPAD_DBG_FLAGS,
            Self::DebugSaveloadScrollposToggle => {
                crate::ram::words::DEBUG_SAVELOAD_SCROLLPOS_TOGGLE
            }
            Self::DebugSavedXscroll => crate::ram::words::DEBUG_SAVED_XSCROLL,
            Self::DebugSavedYscroll => crate::ram::words::DEBUG_SAVED_YSCROLL,
            Self::ButtonConfigUp => crate::ram::words::BUTTON_CONFIG_UP,
            Self::ButtonConfigDown => crate::ram::words::BUTTON_CONFIG_DOWN,
            Self::ButtonConfigLeft => crate::ram::words::BUTTON_CONFIG_LEFT,
            Self::ButtonConfigRight => crate::ram::words::BUTTON_CONFIG_RIGHT,
            Self::ButtonConfigShootXSaved => crate::ram::words::BUTTON_CONFIG_SHOOT_X_SAVED,
            Self::ButtonConfigJumpASaved => crate::ram::words::BUTTON_CONFIG_JUMP_A_SAVED,
            Self::ButtonConfigRunBSaved => crate::ram::words::BUTTON_CONFIG_RUN_B_SAVED,
            Self::ButtonConfigItemcancelYSaved => {
                crate::ram::words::BUTTON_CONFIG_ITEMCANCEL_Y_SAVED
            }
            Self::ButtonConfigItemswitchSaved => crate::ram::words::BUTTON_CONFIG_ITEMSWITCH_SAVED,
            Self::ButtonConfigAimDownLSaved => crate::ram::words::BUTTON_CONFIG_AIM_DOWN_L_SAVED,
            Self::ButtonConfigAimUpRSaved => crate::ram::words::BUTTON_CONFIG_AIM_UP_R_SAVED,
        }
    }
    pub fn get(self, state: &GameLoop) -> &u16 {
        match self {
            Self::GameState => &state.mode,
            Self::GameTimeFrames => &state.clock.frames,
            Self::GameTimeSeconds => &state.clock.seconds,
            Self::GameTimeMinutes => &state.clock.minutes,
            Self::GameTimeHours => &state.clock.hours,
            Self::EnableDebug => &state.debug.enabled,
            Self::JoypadDbg1 => &state.debug.chords[0],
            Self::JoypadDbg2 => &state.debug.chords[1],
            Self::JoypadDbgMissilesSwap => &state.debug.ammunition[0],
            Self::JoypadDbgSuperMissilesSwap => &state.debug.ammunition[1],
            Self::JoypadDbgPowerBombsSwap => &state.debug.ammunition[2],
            Self::JoypadDbgFlags => &state.debug.flags,
            Self::DebugSaveloadScrollposToggle => &state.debug.camera_toggle,
            Self::DebugSavedXscroll => &state.debug.camera[0],
            Self::DebugSavedYscroll => &state.debug.camera[1],
            Self::ButtonConfigUp => &state.directions[0],
            Self::ButtonConfigDown => &state.directions[1],
            Self::ButtonConfigLeft => &state.directions[2],
            Self::ButtonConfigRight => &state.directions[3],
            Self::ButtonConfigShootXSaved => &state.saved_bindings[0],
            Self::ButtonConfigJumpASaved => &state.saved_bindings[1],
            Self::ButtonConfigRunBSaved => &state.saved_bindings[2],
            Self::ButtonConfigItemcancelYSaved => &state.saved_bindings[3],
            Self::ButtonConfigItemswitchSaved => &state.saved_bindings[4],
            Self::ButtonConfigAimDownLSaved => &state.saved_bindings[5],
            Self::ButtonConfigAimUpRSaved => &state.saved_bindings[6],
        }
    }
    pub fn get_mut(self, state: &mut GameLoop) -> &mut u16 {
        match self {
            Self::GameState => &mut state.mode,
            Self::GameTimeFrames => &mut state.clock.frames,
            Self::GameTimeSeconds => &mut state.clock.seconds,
            Self::GameTimeMinutes => &mut state.clock.minutes,
            Self::GameTimeHours => &mut state.clock.hours,
            Self::EnableDebug => &mut state.debug.enabled,
            Self::JoypadDbg1 => &mut state.debug.chords[0],
            Self::JoypadDbg2 => &mut state.debug.chords[1],
            Self::JoypadDbgMissilesSwap => &mut state.debug.ammunition[0],
            Self::JoypadDbgSuperMissilesSwap => &mut state.debug.ammunition[1],
            Self::JoypadDbgPowerBombsSwap => &mut state.debug.ammunition[2],
            Self::JoypadDbgFlags => &mut state.debug.flags,
            Self::DebugSaveloadScrollposToggle => &mut state.debug.camera_toggle,
            Self::DebugSavedXscroll => &mut state.debug.camera[0],
            Self::DebugSavedYscroll => &mut state.debug.camera[1],
            Self::ButtonConfigUp => &mut state.directions[0],
            Self::ButtonConfigDown => &mut state.directions[1],
            Self::ButtonConfigLeft => &mut state.directions[2],
            Self::ButtonConfigRight => &mut state.directions[3],
            Self::ButtonConfigShootXSaved => &mut state.saved_bindings[0],
            Self::ButtonConfigJumpASaved => &mut state.saved_bindings[1],
            Self::ButtonConfigRunBSaved => &mut state.saved_bindings[2],
            Self::ButtonConfigItemcancelYSaved => &mut state.saved_bindings[3],
            Self::ButtonConfigItemswitchSaved => &mut state.saved_bindings[4],
            Self::ButtonConfigAimDownLSaved => &mut state.saved_bindings[5],
            Self::ButtonConfigAimUpRSaved => &mut state.saved_bindings[6],
        }
    }
}
