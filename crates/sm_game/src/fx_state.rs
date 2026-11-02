//! FX numerical state not already held by Room, Visuals or DisplayControls.
#![forbid(unsafe_code)]

/// All supplied words preserve full u16 values and wrapping motion semantics.
/// Source buffers and window controls have separate exclusive canonical owners.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FxState {
    initialized: bool,
    pub blending: u16,
    pub sky_wave_phase: u16,
    pub sky_screen_row: u16,
    pub sky_visible_height: u16,
    pub sky_compatibility_word: u16,
    pub earthquake_sound_index: u16,
    pub earthquake_sound_timer: u16,
    pub hud_scroll: [u16; 2],
    pub effect_scroll: [u16; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Blending,
    SkyWavePhase,
    SkyScreenRow,
    SkyVisibleHeight,
    SkyCompatibilityWord,
    EarthquakeSoundIndex,
    EarthquakeSoundTimer,
    HudHorizontal,
    HudVertical,
    EffectHorizontal,
    EffectVertical,
}
impl Field {
    pub const ALL: [Self; 11] = [
        Self::Blending,
        Self::SkyWavePhase,
        Self::SkyScreenRow,
        Self::SkyVisibleHeight,
        Self::SkyCompatibilityWord,
        Self::EarthquakeSoundIndex,
        Self::EarthquakeSoundTimer,
        Self::HudHorizontal,
        Self::HudVertical,
        Self::EffectHorizontal,
        Self::EffectVertical,
    ];
    pub fn get(self, state: &FxState) -> u16 {
        match self {
            Self::Blending => state.blending,
            Self::SkyWavePhase => state.sky_wave_phase,
            Self::SkyScreenRow => state.sky_screen_row,
            Self::SkyVisibleHeight => state.sky_visible_height,
            Self::SkyCompatibilityWord => state.sky_compatibility_word,
            Self::EarthquakeSoundIndex => state.earthquake_sound_index,
            Self::EarthquakeSoundTimer => state.earthquake_sound_timer,
            Self::HudHorizontal => state.hud_scroll[0],
            Self::HudVertical => state.hud_scroll[1],
            Self::EffectHorizontal => state.effect_scroll[0],
            Self::EffectVertical => state.effect_scroll[1],
        }
    }
    pub fn get_mut(self, state: &mut FxState) -> &mut u16 {
        match self {
            Self::Blending => &mut state.blending,
            Self::SkyWavePhase => &mut state.sky_wave_phase,
            Self::SkyScreenRow => &mut state.sky_screen_row,
            Self::SkyVisibleHeight => &mut state.sky_visible_height,
            Self::SkyCompatibilityWord => &mut state.sky_compatibility_word,
            Self::EarthquakeSoundIndex => &mut state.earthquake_sound_index,
            Self::EarthquakeSoundTimer => &mut state.earthquake_sound_timer,
            Self::HudHorizontal => &mut state.hud_scroll[0],
            Self::HudVertical => &mut state.hud_scroll[1],
            Self::EffectHorizontal => &mut state.effect_scroll[0],
            Self::EffectVertical => &mut state.effect_scroll[1],
        }
    }
}
impl FxState {
    /// Supply the complete actual image; Default does not certify initialization.
    pub fn supplied(words: [u16; 11]) -> Self {
        Self {
            initialized: true,
            blending: words[0],
            sky_wave_phase: words[1],
            sky_screen_row: words[2],
            sky_visible_height: words[3],
            sky_compatibility_word: words[4],
            earthquake_sound_index: words[5],
            earthquake_sound_timer: words[6],
            hud_scroll: [words[7], words[8]],
            effect_scroll: [words[9], words[10]],
        }
    }
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    pub fn supplied_words(self) -> [u16; 11] {
        [
            self.blending,
            self.sky_wave_phase,
            self.sky_screen_row,
            self.sky_visible_height,
            self.sky_compatibility_word,
            self.earthquake_sound_index,
            self.earthquake_sound_timer,
            self.hud_scroll[0],
            self.hud_scroll[1],
            self.effect_scroll[0],
            self.effect_scroll[1],
        ]
    }
}

/// Source/nonrepresentative byte selectors for the complete migrated family.
/// FX38, Window38, common HDMA172; initialization is encoded as wire bit255.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlOwnership {
    pub initialized: bool,
    pub source_lanes: [u64; 4],
}
impl ControlOwnership {
    pub const LANES: usize = 248;
    pub const VALID: [u64; 4] = [u64::MAX, u64::MAX, u64::MAX, (1_u64 << 56) - 1];
    pub fn source_owns(self, lane: usize) -> bool {
        assert!(lane < Self::LANES);
        (self.source_lanes[lane / 64] & (1_u64 << (lane % 64))) != 0
    }
    pub fn select_source(&mut self, lane: usize) {
        assert!(lane < Self::LANES);
        self.source_lanes[lane / 64] |= 1_u64 << (lane % 64);
    }
}
