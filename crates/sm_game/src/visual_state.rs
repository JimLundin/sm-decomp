//! Proposed ordinary visual state, declarations only; no gameplay implementation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaletteSlot {
    pub definition: u16,
    pub color_offset: u16,
    pub pre_action: u16,
    pub program: u16,
    pub delay: u16,
    pub counter: u16,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaletteEffects {
    pub slots: [PaletteSlot; 8],
    pub flags: u16,
    pub selected: u16,
    pub heat: u16,
    pub previous_heat: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AnimationSlot {
    pub definition: u16,
    pub counter: u16,
    pub program: u16,
    pub delay: u16,
    pub source: u16,
    pub size: u16,
    pub vram: u16,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimatedTiles {
    pub slots: [AnimationSlot; 6],
    pub flags: u16,
    pub selected: u16,
    pub last_instruction: u16,
}
/// A palette fade from `colors` toward `targets` ($C400..$C405): the step
/// reached, the fade's duration in steps and the colour the native loops
/// publish as they go.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaletteFade {
    pub step: u16,
    pub duration: u16,
    pub cursor: u16,
}
/// The screen brightness fade's frames per step and countdown ($0723, $0725).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScreenFade {
    pub delay: u16,
    pub counter: u16,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Visuals {
    pub fireflea: FirefleaLighting,
    pub colors: [u16; 256],
    pub targets: [u16; 256],
    pub palette_fade: PaletteFade,
    pub screen_fade: ScreenFade,
    pub palette_effects: PaletteEffects,
    pub animated_tiles: AnimatedTiles,
}
impl Default for Visuals {
    fn default() -> Self {
        Self {
            fireflea: Default::default(),
            colors: [0; 256],
            targets: [0; 256],
            palette_fade: PaletteFade::default(),
            screen_fade: ScreenFade::default(),
            palette_effects: PaletteEffects::default(),
            animated_tiles: AnimatedTiles::default(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerPalette {
    pub hurt_flash: u16,
    pub visor_timer: u16,
    pub suit_index: u16,
    pub xray_progress: u16,
    pub timer: u16,
    pub suit_beam_speed: u16,
    pub suit_red_green: u16,
    // Physical word DF2 also has suit-pickup/transition legacy aliases.
    pub crystal_flash_delay: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Upload {
    pub source: u16,
    pub size: u16,
    pub vram: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteBuffer {
    Current,
    Target,
}
/// Actual 65816 carry at the phase-return branch, not a Rust u8 return value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteCarry {
    Clear,
    Set,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FirefleaLighting {
    pub timer: u16,
    pub flash_index: u16,
    pub compatibility_word: u16,
    pub darkness: u16,
    pub compatibility_limit: u16,
    pub compatibility_shade: u16,
}

#[derive(Clone, Copy)]
pub enum EyeBeamPhase {
    Widening,
    Full,
    Fading,
}

#[derive(Clone, Copy)]
pub enum HazePhase {
    FadingIn,
    FadedIn,
    FadingOut,
}

#[derive(Clone, Copy)]
pub enum Weather {
    Rain,
    Spores,
    Fog,
}

#[derive(Clone, Copy)]
pub enum TourianStatuePhase {
    Waiting,
    Shaking,
    Sinking,
    Finished,
}

#[derive(Clone, Copy)]
pub enum HdmaWave {
    Phantoon,
    Intro,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CinematicDisplay {
    Title,
    Intro,
    Station,
    Landing,
    Escape,
    Credits,
}

#[derive(Clone, Copy)]
pub enum CinematicFade {
    FastIn,
    FastOut,
    SlowIn,
    SlowOut,
}

#[derive(Clone, Copy)]
pub enum CinematicTilemap {
    Text,
    FullText,
    Eyes,
    Japanese,
    JapaneseIntro,
}

#[derive(Clone, Copy)]
pub enum CeresExplosion {
    First,
    Second,
    Third,
    Final,
}

#[derive(Clone, Copy)]
pub enum EndingGraphics {
    Background,
    Samus,
}
