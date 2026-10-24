//! Proposed ordinary visual state, declarations only; no gameplay implementation.
#![forbid(unsafe_code)]
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
#[derive(Clone, Debug, PartialEq)]
pub struct Visuals {
    pub(in crate::game) fireflea: FirefleaLighting,
    pub colors: [u16; 256],
    pub targets: [u16; 256],
    pub palette_effects: PaletteEffects,
    pub animated_tiles: AnimatedTiles,
}
impl Default for Visuals {
    fn default() -> Self {
        Self {
            fireflea: Default::default(),
            colors: [0; 256],
            targets: [0; 256],
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
pub(in crate::game) struct FirefleaLighting {
    pub(in crate::game) timer: u16,
    pub(in crate::game) flash_index: u16,
    pub(in crate::game) compatibility_word: u16,
    pub(in crate::game) darkness: u16,
    pub(in crate::game) compatibility_limit: u16,
    pub(in crate::game) compatibility_shade: u16,
}
