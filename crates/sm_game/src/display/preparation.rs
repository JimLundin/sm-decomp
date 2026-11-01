//! Frame display preparation controls and the non-minimap HUD owner.
//! Legacy residency is transient; existing Display/Minimap fields stay canonical.
#![forbid(unsafe_code)]
use crate::scene::state::{DisplayState, NmiState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Brightness,
    ObjectLayout,
    Mode,
    FakeMode,
    Mosaic,
    Screens0,
    Screens1,
    Screens2,
    Background3Map,
    Screens3,
    Characters0,
    Characters1,
    AffineSelection,
    WindowSelection0,
    WindowSelection1,
    WindowSelection2,
    WindowPositions0,
    WindowPositions1,
    WindowPositions2,
    WindowPositions3,
    WindowLogic0,
    WindowLogic1,
    MainLayers,
    PreviousMainLayers,
    SubLayers,
    MainWindow,
    SubWindow,
    NextColorSelection,
    ColorSelection,
    PreviousColorSelection,
    NextColorMath,
    ColorMath,
    PreviousColorMath,
    FixedColor0,
    FixedColor1,
    FixedColor2,
    Geometry,
    AffineWords0Low,
    AffineWords0High,
    AffineWords1Low,
    AffineWords1High,
    AffineWords2Low,
    AffineWords2High,
    AffineWords3Low,
    AffineWords3High,
    AffineWords4Low,
    AffineWords4High,
    AffineWords5Low,
    AffineWords5High,
    NmiEnable,
    HdmaEnable,
    NmiPending,
    NmiFrameByte,
    NmiFrameWordLow,
    NmiFrameWordHigh,
    NmiIncludingLagLow,
    NmiIncludingLagHigh,
    NmiMissed,
    NmiMaxMissed,
    NmiAcknowledgment,
    CommittedFakeMode,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    PendingInterrupt,
    CurrentInterrupt,
    HudPalette,
    HudAutoCancel,
    PreviousHealth,
    PreviousMissiles,
    PreviousSupers,
    PreviousBombs,
    PreviousSelection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Controls {
    brightness: u8,
    object_layout: u8,
    mode: u8,
    fake_mode: u8,
    mosaic: u8,
    screens: [u8; 4],
    background3_map: u8,
    characters: [u8; 2],
    affine_selection: u8,
    window_selection: [u8; 3],
    window_positions: [u8; 4],
    window_logic: [u8; 2],
    main_layers: u8,
    previous_main_layers: u8,
    sub_layers: u8,
    main_window: u8,
    sub_window: u8,
    next_color_selection: u8,
    color_selection: u8,
    previous_color_selection: u8,
    next_color_math: u8,
    color_math: u8,
    previous_color_math: u8,
    fixed_color: [u8; 3],
    geometry: u8,
    affine_words: [u16; 6],
    nmi_enable: u8,
    hdma_enable: u8,
    committed_fake_mode: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    controls: Controls,
    nmi: NmiState,
    interrupt_pending: u16,
    interrupt_current: u16,
    hud_palette: u16,
    hud_auto_cancel: u16,
    previous_health: u16,
    previous_missiles: u16,
    previous_supers: u16,
    previous_bombs: u16,
    previous_selection: u16,
    // The fifteen minimap cells remain with their existing owner.
    hud: [u16; 81],
}
pub enum Domain {
    Unowned,
    Owned { state: Box<State>, epoch: u64 },
    InSession { epoch: u64 },
    Leased,
    Revoked,
}
fn replace_lane(word: &mut u16, lane: usize, value: u8) {
    let mut bytes = word.to_le_bytes();
    bytes[lane] = value;
    *word = u16::from_le_bytes(bytes);
}
/// Existing full HUD index -> owned non-minimap storage index.
pub fn hud_index(index: usize) -> Option<usize> {
    if index >= 96 {
        return None;
    }
    let row = index / 32;
    let column = index % 32;
    if (26..31).contains(&column) {
        return None;
    }
    Some(row * 27 + column - if column == 31 { 5 } else { 0 })
}
pub fn hud_cell(index: usize) -> usize {
    sm_data::table::check_index(index, 81);
    let row = index / 27;
    let column = index % 27;
    row * 32 + if column == 26 { 31 } else { column }
}
impl State {
    /// Decode the selected complete subset; no media or Display is manufactured.
    pub fn read(
        mut read: impl FnMut(Field) -> u8,
        mut words: impl FnMut(Word) -> u16,
        mut tile: impl FnMut(usize) -> u16,
    ) -> Self {
        let controls = Controls {
            brightness: read(Field::Brightness),
            object_layout: read(Field::ObjectLayout),
            mode: read(Field::Mode),
            fake_mode: read(Field::FakeMode),
            mosaic: read(Field::Mosaic),
            screens: [
                read(Field::Screens0),
                read(Field::Screens1),
                read(Field::Screens2),
                read(Field::Screens3),
            ],
            background3_map: read(Field::Background3Map),
            characters: [read(Field::Characters0), read(Field::Characters1)],
            affine_selection: read(Field::AffineSelection),
            window_selection: [
                read(Field::WindowSelection0),
                read(Field::WindowSelection1),
                read(Field::WindowSelection2),
            ],
            window_positions: [
                read(Field::WindowPositions0),
                read(Field::WindowPositions1),
                read(Field::WindowPositions2),
                read(Field::WindowPositions3),
            ],
            window_logic: [read(Field::WindowLogic0), read(Field::WindowLogic1)],
            main_layers: read(Field::MainLayers),
            previous_main_layers: read(Field::PreviousMainLayers),
            sub_layers: read(Field::SubLayers),
            main_window: read(Field::MainWindow),
            sub_window: read(Field::SubWindow),
            next_color_selection: read(Field::NextColorSelection),
            color_selection: read(Field::ColorSelection),
            previous_color_selection: read(Field::PreviousColorSelection),
            next_color_math: read(Field::NextColorMath),
            color_math: read(Field::ColorMath),
            previous_color_math: read(Field::PreviousColorMath),
            fixed_color: [
                read(Field::FixedColor0),
                read(Field::FixedColor1),
                read(Field::FixedColor2),
            ],
            geometry: read(Field::Geometry),
            affine_words: [
                u16::from_le_bytes([read(Field::AffineWords0Low), read(Field::AffineWords0High)]),
                u16::from_le_bytes([read(Field::AffineWords1Low), read(Field::AffineWords1High)]),
                u16::from_le_bytes([read(Field::AffineWords2Low), read(Field::AffineWords2High)]),
                u16::from_le_bytes([read(Field::AffineWords3Low), read(Field::AffineWords3High)]),
                u16::from_le_bytes([read(Field::AffineWords4Low), read(Field::AffineWords4High)]),
                u16::from_le_bytes([read(Field::AffineWords5Low), read(Field::AffineWords5High)]),
            ],
            nmi_enable: read(Field::NmiEnable),
            hdma_enable: read(Field::HdmaEnable),
            committed_fake_mode: read(Field::CommittedFakeMode),
        };
        let nmi = NmiState {
            pending: read(Field::NmiPending),
            frame_byte: read(Field::NmiFrameByte),
            frame_word: u16::from_le_bytes([
                read(Field::NmiFrameWordLow),
                read(Field::NmiFrameWordHigh),
            ]),
            including_lag: u16::from_le_bytes([
                read(Field::NmiIncludingLagLow),
                read(Field::NmiIncludingLagHigh),
            ]),
            missed: read(Field::NmiMissed),
            max_missed: read(Field::NmiMaxMissed),
            acknowledgment: read(Field::NmiAcknowledgment),
        };
        let mut hud = [0; 81];
        for (index, value) in hud.iter_mut().enumerate() {
            *value = tile(hud_cell(index));
        }
        Self {
            controls,
            nmi,
            interrupt_pending: words(Word::PendingInterrupt),
            interrupt_current: words(Word::CurrentInterrupt),
            hud_palette: words(Word::HudPalette),
            hud_auto_cancel: words(Word::HudAutoCancel),
            previous_health: words(Word::PreviousHealth),
            previous_missiles: words(Word::PreviousMissiles),
            previous_supers: words(Word::PreviousSupers),
            previous_bombs: words(Word::PreviousBombs),
            previous_selection: words(Word::PreviousSelection),
            hud,
        }
    }
    pub fn byte(&self, field: Field) -> u8 {
        match field {
            Field::Brightness => self.controls.brightness,
            Field::ObjectLayout => self.controls.object_layout,
            Field::Mode => self.controls.mode,
            Field::FakeMode => self.controls.fake_mode,
            Field::Mosaic => self.controls.mosaic,
            Field::Screens0 => self.controls.screens[0],
            Field::Screens1 => self.controls.screens[1],
            Field::Screens2 => self.controls.screens[2],
            Field::Background3Map => self.controls.background3_map,
            Field::Screens3 => self.controls.screens[3],
            Field::Characters0 => self.controls.characters[0],
            Field::Characters1 => self.controls.characters[1],
            Field::AffineSelection => self.controls.affine_selection,
            Field::WindowSelection0 => self.controls.window_selection[0],
            Field::WindowSelection1 => self.controls.window_selection[1],
            Field::WindowSelection2 => self.controls.window_selection[2],
            Field::WindowPositions0 => self.controls.window_positions[0],
            Field::WindowPositions1 => self.controls.window_positions[1],
            Field::WindowPositions2 => self.controls.window_positions[2],
            Field::WindowPositions3 => self.controls.window_positions[3],
            Field::WindowLogic0 => self.controls.window_logic[0],
            Field::WindowLogic1 => self.controls.window_logic[1],
            Field::MainLayers => self.controls.main_layers,
            Field::PreviousMainLayers => self.controls.previous_main_layers,
            Field::SubLayers => self.controls.sub_layers,
            Field::MainWindow => self.controls.main_window,
            Field::SubWindow => self.controls.sub_window,
            Field::NextColorSelection => self.controls.next_color_selection,
            Field::ColorSelection => self.controls.color_selection,
            Field::PreviousColorSelection => self.controls.previous_color_selection,
            Field::NextColorMath => self.controls.next_color_math,
            Field::ColorMath => self.controls.color_math,
            Field::PreviousColorMath => self.controls.previous_color_math,
            Field::FixedColor0 => self.controls.fixed_color[0],
            Field::FixedColor1 => self.controls.fixed_color[1],
            Field::FixedColor2 => self.controls.fixed_color[2],
            Field::Geometry => self.controls.geometry,
            Field::AffineWords0Low => self.controls.affine_words[0].to_le_bytes()[0],
            Field::AffineWords0High => self.controls.affine_words[0].to_le_bytes()[1],
            Field::AffineWords1Low => self.controls.affine_words[1].to_le_bytes()[0],
            Field::AffineWords1High => self.controls.affine_words[1].to_le_bytes()[1],
            Field::AffineWords2Low => self.controls.affine_words[2].to_le_bytes()[0],
            Field::AffineWords2High => self.controls.affine_words[2].to_le_bytes()[1],
            Field::AffineWords3Low => self.controls.affine_words[3].to_le_bytes()[0],
            Field::AffineWords3High => self.controls.affine_words[3].to_le_bytes()[1],
            Field::AffineWords4Low => self.controls.affine_words[4].to_le_bytes()[0],
            Field::AffineWords4High => self.controls.affine_words[4].to_le_bytes()[1],
            Field::AffineWords5Low => self.controls.affine_words[5].to_le_bytes()[0],
            Field::AffineWords5High => self.controls.affine_words[5].to_le_bytes()[1],
            Field::NmiEnable => self.controls.nmi_enable,
            Field::HdmaEnable => self.controls.hdma_enable,
            Field::NmiPending => self.nmi.pending,
            Field::NmiFrameByte => self.nmi.frame_byte,
            Field::NmiFrameWordLow => self.nmi.frame_word.to_le_bytes()[0],
            Field::NmiFrameWordHigh => self.nmi.frame_word.to_le_bytes()[1],
            Field::NmiIncludingLagLow => self.nmi.including_lag.to_le_bytes()[0],
            Field::NmiIncludingLagHigh => self.nmi.including_lag.to_le_bytes()[1],
            Field::NmiMissed => self.nmi.missed,
            Field::NmiMaxMissed => self.nmi.max_missed,
            Field::NmiAcknowledgment => self.nmi.acknowledgment,
            Field::CommittedFakeMode => self.controls.committed_fake_mode,
        }
    }
    pub fn set_byte(&mut self, field: Field, value: u8) {
        match field {
            Field::Brightness => self.controls.brightness = value,
            Field::ObjectLayout => self.controls.object_layout = value,
            Field::Mode => self.controls.mode = value,
            Field::FakeMode => self.controls.fake_mode = value,
            Field::Mosaic => self.controls.mosaic = value,
            Field::Screens0 => self.controls.screens[0] = value,
            Field::Screens1 => self.controls.screens[1] = value,
            Field::Screens2 => self.controls.screens[2] = value,
            Field::Background3Map => self.controls.background3_map = value,
            Field::Screens3 => self.controls.screens[3] = value,
            Field::Characters0 => self.controls.characters[0] = value,
            Field::Characters1 => self.controls.characters[1] = value,
            Field::AffineSelection => self.controls.affine_selection = value,
            Field::WindowSelection0 => self.controls.window_selection[0] = value,
            Field::WindowSelection1 => self.controls.window_selection[1] = value,
            Field::WindowSelection2 => self.controls.window_selection[2] = value,
            Field::WindowPositions0 => self.controls.window_positions[0] = value,
            Field::WindowPositions1 => self.controls.window_positions[1] = value,
            Field::WindowPositions2 => self.controls.window_positions[2] = value,
            Field::WindowPositions3 => self.controls.window_positions[3] = value,
            Field::WindowLogic0 => self.controls.window_logic[0] = value,
            Field::WindowLogic1 => self.controls.window_logic[1] = value,
            Field::MainLayers => self.controls.main_layers = value,
            Field::PreviousMainLayers => self.controls.previous_main_layers = value,
            Field::SubLayers => self.controls.sub_layers = value,
            Field::MainWindow => self.controls.main_window = value,
            Field::SubWindow => self.controls.sub_window = value,
            Field::NextColorSelection => self.controls.next_color_selection = value,
            Field::ColorSelection => self.controls.color_selection = value,
            Field::PreviousColorSelection => self.controls.previous_color_selection = value,
            Field::NextColorMath => self.controls.next_color_math = value,
            Field::ColorMath => self.controls.color_math = value,
            Field::PreviousColorMath => self.controls.previous_color_math = value,
            Field::FixedColor0 => self.controls.fixed_color[0] = value,
            Field::FixedColor1 => self.controls.fixed_color[1] = value,
            Field::FixedColor2 => self.controls.fixed_color[2] = value,
            Field::Geometry => self.controls.geometry = value,
            Field::AffineWords0Low => replace_lane(&mut self.controls.affine_words[0], 0, value),
            Field::AffineWords0High => replace_lane(&mut self.controls.affine_words[0], 1, value),
            Field::AffineWords1Low => replace_lane(&mut self.controls.affine_words[1], 0, value),
            Field::AffineWords1High => replace_lane(&mut self.controls.affine_words[1], 1, value),
            Field::AffineWords2Low => replace_lane(&mut self.controls.affine_words[2], 0, value),
            Field::AffineWords2High => replace_lane(&mut self.controls.affine_words[2], 1, value),
            Field::AffineWords3Low => replace_lane(&mut self.controls.affine_words[3], 0, value),
            Field::AffineWords3High => replace_lane(&mut self.controls.affine_words[3], 1, value),
            Field::AffineWords4Low => replace_lane(&mut self.controls.affine_words[4], 0, value),
            Field::AffineWords4High => replace_lane(&mut self.controls.affine_words[4], 1, value),
            Field::AffineWords5Low => replace_lane(&mut self.controls.affine_words[5], 0, value),
            Field::AffineWords5High => replace_lane(&mut self.controls.affine_words[5], 1, value),
            Field::NmiEnable => self.controls.nmi_enable = value,
            Field::HdmaEnable => self.controls.hdma_enable = value,
            Field::NmiPending => self.nmi.pending = value,
            Field::NmiFrameByte => self.nmi.frame_byte = value,
            Field::NmiFrameWordLow => replace_lane(&mut self.nmi.frame_word, 0, value),
            Field::NmiFrameWordHigh => replace_lane(&mut self.nmi.frame_word, 1, value),
            Field::NmiIncludingLagLow => replace_lane(&mut self.nmi.including_lag, 0, value),
            Field::NmiIncludingLagHigh => replace_lane(&mut self.nmi.including_lag, 1, value),
            Field::NmiMissed => self.nmi.missed = value,
            Field::NmiMaxMissed => self.nmi.max_missed = value,
            Field::NmiAcknowledgment => self.nmi.acknowledgment = value,
            Field::CommittedFakeMode => self.controls.committed_fake_mode = value,
        }
    }
    pub fn word(&self, word: Word) -> u16 {
        match word {
            Word::PendingInterrupt => self.interrupt_pending,
            Word::CurrentInterrupt => self.interrupt_current,
            Word::HudPalette => self.hud_palette,
            Word::HudAutoCancel => self.hud_auto_cancel,
            Word::PreviousHealth => self.previous_health,
            Word::PreviousMissiles => self.previous_missiles,
            Word::PreviousSupers => self.previous_supers,
            Word::PreviousBombs => self.previous_bombs,
            Word::PreviousSelection => self.previous_selection,
        }
    }
    pub fn set_word(&mut self, word: Word, value: u16) {
        match word {
            Word::PendingInterrupt => self.interrupt_pending = value,
            Word::CurrentInterrupt => self.interrupt_current = value,
            Word::HudPalette => self.hud_palette = value,
            Word::HudAutoCancel => self.hud_auto_cancel = value,
            Word::PreviousHealth => self.previous_health = value,
            Word::PreviousMissiles => self.previous_missiles = value,
            Word::PreviousSupers => self.previous_supers = value,
            Word::PreviousBombs => self.previous_bombs = value,
            Word::PreviousSelection => self.previous_selection = value,
        }
    }
    pub fn tile(&self, index: usize) -> Option<u16> {
        hud_index(index).map(|at| self.hud[at])
    }
    pub fn set_tile(&mut self, index: usize, value: u16) -> bool {
        let Some(at) = hud_index(index) else {
            return false;
        };
        self.hud[at] = value;
        true
    }
}
pub fn display_byte(display: &DisplayState, field: Field) -> u8 {
    match field {
        Field::Brightness => display.controls.brightness,
        Field::ObjectLayout => display.controls.object_layout,
        Field::Mode => display.controls.mode,
        Field::FakeMode => display.controls.fake_mode,
        Field::Mosaic => display.controls.mosaic,
        Field::Screens0 => display.controls.screens[0],
        Field::Screens1 => display.controls.screens[1],
        Field::Screens2 => display.controls.screens[2],
        Field::Background3Map => display.backgrounds[2].screen_layout,
        Field::Screens3 => display.controls.screens[3],
        Field::Characters0 => display.controls.characters[0],
        Field::Characters1 => display.controls.characters[1],
        Field::AffineSelection => display.controls.affine_selection,
        Field::WindowSelection0 => display.controls.window_selection[0],
        Field::WindowSelection1 => display.controls.window_selection[1],
        Field::WindowSelection2 => display.controls.window_selection[2],
        Field::WindowPositions0 => display.controls.window_positions[0],
        Field::WindowPositions1 => display.controls.window_positions[1],
        Field::WindowPositions2 => display.controls.window_positions[2],
        Field::WindowPositions3 => display.controls.window_positions[3],
        Field::WindowLogic0 => display.controls.window_logic[0],
        Field::WindowLogic1 => display.controls.window_logic[1],
        Field::MainLayers => display.controls.main_layers,
        Field::PreviousMainLayers => display.controls.previous_main_layers,
        Field::SubLayers => display.controls.sub_layers,
        Field::MainWindow => display.controls.main_window,
        Field::SubWindow => display.controls.sub_window,
        Field::NextColorSelection => display.controls.next_color_selection,
        Field::ColorSelection => display.controls.color_selection,
        Field::PreviousColorSelection => display.controls.previous_color_selection,
        Field::NextColorMath => display.controls.next_color_math,
        Field::ColorMath => display.controls.color_math,
        Field::PreviousColorMath => display.controls.previous_color_math,
        Field::FixedColor0 => display.controls.fixed_color[0],
        Field::FixedColor1 => display.controls.fixed_color[1],
        Field::FixedColor2 => display.controls.fixed_color[2],
        Field::Geometry => display.controls.geometry,
        Field::AffineWords0Low => display.controls.affine_words[0].to_le_bytes()[0],
        Field::AffineWords0High => display.controls.affine_words[0].to_le_bytes()[1],
        Field::AffineWords1Low => display.controls.affine_words[1].to_le_bytes()[0],
        Field::AffineWords1High => display.controls.affine_words[1].to_le_bytes()[1],
        Field::AffineWords2Low => display.controls.affine_words[2].to_le_bytes()[0],
        Field::AffineWords2High => display.controls.affine_words[2].to_le_bytes()[1],
        Field::AffineWords3Low => display.controls.affine_words[3].to_le_bytes()[0],
        Field::AffineWords3High => display.controls.affine_words[3].to_le_bytes()[1],
        Field::AffineWords4Low => display.controls.affine_words[4].to_le_bytes()[0],
        Field::AffineWords4High => display.controls.affine_words[4].to_le_bytes()[1],
        Field::AffineWords5Low => display.controls.affine_words[5].to_le_bytes()[0],
        Field::AffineWords5High => display.controls.affine_words[5].to_le_bytes()[1],
        Field::NmiEnable => display.controls.nmi_enable,
        Field::HdmaEnable => display.controls.hdma_enable,
        Field::NmiPending => display.nmi.pending,
        Field::NmiFrameByte => display.nmi.frame_byte,
        Field::NmiFrameWordLow => display.nmi.frame_word.to_le_bytes()[0],
        Field::NmiFrameWordHigh => display.nmi.frame_word.to_le_bytes()[1],
        Field::NmiIncludingLagLow => display.nmi.including_lag.to_le_bytes()[0],
        Field::NmiIncludingLagHigh => display.nmi.including_lag.to_le_bytes()[1],
        Field::NmiMissed => display.nmi.missed,
        Field::NmiMaxMissed => display.nmi.max_missed,
        Field::NmiAcknowledgment => display.nmi.acknowledgment,
        Field::CommittedFakeMode => display.controls.committed_fake_mode,
    }
}
pub fn set_display_byte(display: &mut DisplayState, field: Field, value: u8) {
    match field {
        Field::Brightness => display.controls.brightness = value,
        Field::ObjectLayout => display.controls.object_layout = value,
        Field::Mode => display.controls.mode = value,
        Field::FakeMode => display.controls.fake_mode = value,
        Field::Mosaic => display.controls.mosaic = value,
        Field::Screens0 => display.controls.screens[0] = value,
        Field::Screens1 => display.controls.screens[1] = value,
        Field::Screens2 => display.controls.screens[2] = value,
        Field::Background3Map => display.backgrounds[2].screen_layout = value,
        Field::Screens3 => display.controls.screens[3] = value,
        Field::Characters0 => display.controls.characters[0] = value,
        Field::Characters1 => display.controls.characters[1] = value,
        Field::AffineSelection => display.controls.affine_selection = value,
        Field::WindowSelection0 => display.controls.window_selection[0] = value,
        Field::WindowSelection1 => display.controls.window_selection[1] = value,
        Field::WindowSelection2 => display.controls.window_selection[2] = value,
        Field::WindowPositions0 => display.controls.window_positions[0] = value,
        Field::WindowPositions1 => display.controls.window_positions[1] = value,
        Field::WindowPositions2 => display.controls.window_positions[2] = value,
        Field::WindowPositions3 => display.controls.window_positions[3] = value,
        Field::WindowLogic0 => display.controls.window_logic[0] = value,
        Field::WindowLogic1 => display.controls.window_logic[1] = value,
        Field::MainLayers => display.controls.main_layers = value,
        Field::PreviousMainLayers => display.controls.previous_main_layers = value,
        Field::SubLayers => display.controls.sub_layers = value,
        Field::MainWindow => display.controls.main_window = value,
        Field::SubWindow => display.controls.sub_window = value,
        Field::NextColorSelection => display.controls.next_color_selection = value,
        Field::ColorSelection => display.controls.color_selection = value,
        Field::PreviousColorSelection => display.controls.previous_color_selection = value,
        Field::NextColorMath => display.controls.next_color_math = value,
        Field::ColorMath => display.controls.color_math = value,
        Field::PreviousColorMath => display.controls.previous_color_math = value,
        Field::FixedColor0 => display.controls.fixed_color[0] = value,
        Field::FixedColor1 => display.controls.fixed_color[1] = value,
        Field::FixedColor2 => display.controls.fixed_color[2] = value,
        Field::Geometry => display.controls.geometry = value,
        Field::AffineWords0Low => replace_lane(&mut display.controls.affine_words[0], 0, value),
        Field::AffineWords0High => replace_lane(&mut display.controls.affine_words[0], 1, value),
        Field::AffineWords1Low => replace_lane(&mut display.controls.affine_words[1], 0, value),
        Field::AffineWords1High => replace_lane(&mut display.controls.affine_words[1], 1, value),
        Field::AffineWords2Low => replace_lane(&mut display.controls.affine_words[2], 0, value),
        Field::AffineWords2High => replace_lane(&mut display.controls.affine_words[2], 1, value),
        Field::AffineWords3Low => replace_lane(&mut display.controls.affine_words[3], 0, value),
        Field::AffineWords3High => replace_lane(&mut display.controls.affine_words[3], 1, value),
        Field::AffineWords4Low => replace_lane(&mut display.controls.affine_words[4], 0, value),
        Field::AffineWords4High => replace_lane(&mut display.controls.affine_words[4], 1, value),
        Field::AffineWords5Low => replace_lane(&mut display.controls.affine_words[5], 0, value),
        Field::AffineWords5High => replace_lane(&mut display.controls.affine_words[5], 1, value),
        Field::NmiEnable => display.controls.nmi_enable = value,
        Field::HdmaEnable => display.controls.hdma_enable = value,
        Field::NmiPending => display.nmi.pending = value,
        Field::NmiFrameByte => display.nmi.frame_byte = value,
        Field::NmiFrameWordLow => replace_lane(&mut display.nmi.frame_word, 0, value),
        Field::NmiFrameWordHigh => replace_lane(&mut display.nmi.frame_word, 1, value),
        Field::NmiIncludingLagLow => replace_lane(&mut display.nmi.including_lag, 0, value),
        Field::NmiIncludingLagHigh => replace_lane(&mut display.nmi.including_lag, 1, value),
        Field::NmiMissed => display.nmi.missed = value,
        Field::NmiMaxMissed => display.nmi.max_missed = value,
        Field::NmiAcknowledgment => display.nmi.acknowledgment = value,
        Field::CommittedFakeMode => display.controls.committed_fake_mode = value,
    }
}
pub fn display_word(display: &DisplayState, word: Word) -> u16 {
    match word {
        Word::PendingInterrupt => display.interrupt.pending,
        Word::CurrentInterrupt => display.interrupt.current,
        Word::HudPalette => display.hud.palette,
        Word::HudAutoCancel => display.hud.auto_cancel,
        Word::PreviousHealth => display.hud.previous_health,
        Word::PreviousMissiles => display.hud.previous_ammunition[0],
        Word::PreviousSupers => display.hud.previous_ammunition[1],
        Word::PreviousBombs => display.hud.previous_ammunition[2],
        Word::PreviousSelection => display.hud.previous_selection,
    }
}
pub fn set_display_word(display: &mut DisplayState, word: Word, value: u16) {
    match word {
        Word::PendingInterrupt => display.interrupt.pending = value,
        Word::CurrentInterrupt => display.interrupt.current = value,
        Word::HudPalette => display.hud.palette = value,
        Word::HudAutoCancel => display.hud.auto_cancel = value,
        Word::PreviousHealth => display.hud.previous_health = value,
        Word::PreviousMissiles => display.hud.previous_ammunition[0] = value,
        Word::PreviousSupers => display.hud.previous_ammunition[1] = value,
        Word::PreviousBombs => display.hud.previous_ammunition[2] = value,
        Word::PreviousSelection => display.hud.previous_selection = value,
    }
}
/// Original stores have different authority/admission: owned-byte stores versus
/// named/native stores on Game. Runtime's named stores use its current projection.
#[derive(Clone, Copy)]
pub enum Store {
    Owned,
    Named,
}
pub trait Access {
    fn display_control_byte(&self, field: Field) -> u8;
    fn store_display_control_byte(&mut self, field: Field, value: u8, store: Store);
    fn display_history_word(&self, word: Word) -> u16;
    fn store_display_history_word(&mut self, word: Word, value: u16, store: Store);
    fn hud_tile(&self, index: isize) -> u16;
    fn store_hud_tile(&mut self, index: isize, value: u16, store: Store);
    fn physical_display_control_byte(&self, field: Field) -> u8;
    fn store_physical_display_control_byte(&mut self, field: Field, value: u8);
}

/// A captured semantic destination. No address or whole-RAM borrow escapes.
#[derive(Clone, Copy)]
pub enum Place {
    Controls(Field, Field),
    History(Word),
    Hud(isize),
}
impl Place {
    pub fn read(self, host: &(impl Access + ?Sized)) -> u16 {
        match self {
            Self::Controls(low, high) => u16::from_le_bytes([
                host.display_control_byte(low),
                host.display_control_byte(high),
            ]),
            Self::History(word) => host.display_history_word(word),
            Self::Hud(index) => host.hud_tile(index),
        }
    }
    pub fn write(self, host: &mut (impl Access + ?Sized), value: u16) {
        match self {
            Self::Controls(low, high) => {
                let [lo, hi] = value.to_le_bytes();
                host.store_display_control_byte(low, lo, Store::Named);
                host.store_display_control_byte(high, hi, Store::Named);
            }
            Self::History(word) => host.store_display_history_word(word, value, Store::Named),
            Self::Hud(index) => host.store_hud_tile(index, value, Store::Named),
        }
    }
}
#[macro_export]
macro_rules! frame_display_word {
    // Compound arithmetic wraps at 16 bits; bits/shifts keep ordinary u16 operations.
    // Delegate destination capture, old load, RHS evaluation and store to the update arm.
    ($game:ident.$field:ident$([$index:expr])? &= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| old & $value $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? += $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| old.wrapping_add($value) $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? -= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| old.wrapping_sub($value) $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? >>= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| old >> $value $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? ^= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| old ^ $value $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? |= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| old | $value $(, $result)?)
    };
    // Explicit promoted operations retain their original RHS type and final u16 cast.
    // Checked operations use ordinary Rust overflow/shift rules before truncation.
    (bits_i32, $game:ident.$field:ident$([$index:expr])? |= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| ((old as i32) | $value) as u16 $(, $result)?)
    };
    (checked_i32, $game:ident.$field:ident$([$index:expr])? *= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| ((old as i32) * $value) as u16 $(, $result)?)
    };
    (checked_i32, $game:ident.$field:ident$([$index:expr])? += $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| ((old as i32) + $value) as u16 $(, $result)?)
    };
    (checked_i32, $game:ident.$field:ident$([$index:expr])? -= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| ((old as i32) - $value) as u16 $(, $result)?)
    };
    (checked_i32, $game:ident.$field:ident$([$index:expr])? <<= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| ((old as i32) << $value) as u16 $(, $result)?)
    };
    (wrapping_u32, $game:ident.$field:ident$([$index:expr])? += $value:expr $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |old| (old as u32).wrapping_add($value) as u16 $(, $result)?)
    };

    ($game:ident.$field:ident[$index:expr]) => {
        $crate::frame_display_word!(@read $game,
            $crate::frame_display_word!(@offset $field[$index]))
    };
    ($game:ident.$field:ident) => {
        $crate::frame_display_word!(@read $game, $crate::frame_display_word!(@place $field))
    };
    ($game:ident.$field:ident[$index:expr] = $value:expr $(,)?) => {
        $crate::frame_display_word!(@write $game,
            $crate::frame_display_word!(@offset $field[$index]), $value)
    };
    ($game:ident.$field:ident = $value:expr $(,)?) => {
        $crate::frame_display_word!(@write $game, $crate::frame_display_word!(@place $field), $value)
    };
    ($game:ident; $($field:ident$([$index:expr])? = $value:expr;)+) => {
        $($crate::frame_display_word!($game.$field$([$index])? = $value);)+
    };
    ($game:ident, $value:literal; $($field:ident$([$index:expr])?),+ $(,)?) => {
        $($crate::frame_display_word!($game.$field$([$index])? = $value);)+
    };
    ($game:ident.$field:ident$([$index:expr])?, $method:ident($argument:expr $(,)?) $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |word| word.$method($argument) $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])?, |$old:ident| { $value:expr } $(, $result:ident)? $(,)?) => {
        $crate::frame_display_word!($game.$field$([$index])?, |$old| $value $(, $result)?)
    };
    ($game:ident.$field:ident[$index:expr], |$old:ident| $value:expr $(, $result:ident)? $(,)?) => {{
        let destination = $crate::frame_display_word!(@offset $field[$index]);
        let $old = $crate::frame_display_word!(@read $game, destination);
        $crate::frame_display_word!(@write $game, destination, $value);
        $crate::frame_display_word!(@result $game, destination $(, $result)?)
    }};
    ($game:ident.$field:ident, |$old:ident| $value:expr $(, $result:ident)? $(,)?) => {{
        let destination = $crate::frame_display_word!(@place $field);
        let $old = $crate::frame_display_word!(@read $game, destination);
        $crate::frame_display_word!(@write $game, destination, $value);
        $crate::frame_display_word!(@result $game, destination $(, $result)?)
    }};
    (@offset HudTiles[$index:expr]) => {
        $crate::display::preparation::Place::Hud($index as isize)
    };
    (@read $game:ident, $place:expr) => {
        match $place { place => place.read(&*$game) }
    };
    (@write $game:ident, $place:expr, $value:expr) => {
        // Retain RHS/index temporaries until both low/high stores finish.
        match ($value, $place) {
            (value, place) => place.write(&mut *$game, value)
        }
    };
    (@result $game:ident, $destination:ident) => { () };
    (@result $game:ident, $destination:ident, reread) => {
        $crate::frame_display_word!(@read $game, $destination)
    };
    (@place BrightnessPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Brightness, $crate::display::preparation::Field::ObjectLayout) };
    (@place ModePair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Mode, $crate::display::preparation::Field::FakeMode) };
    (@place FakeModePair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::FakeMode, $crate::display::preparation::Field::Mosaic) };
    (@place MosaicPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Mosaic, $crate::display::preparation::Field::Screens0) };
    (@place Screens0Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Screens0, $crate::display::preparation::Field::Screens1) };
    (@place Screens1Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Screens1, $crate::display::preparation::Field::Screens2) };
    (@place Screens2Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Screens2, $crate::display::preparation::Field::Background3Map) };
    (@place Background3MapPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Background3Map, $crate::display::preparation::Field::Screens3) };
    (@place Screens3Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Screens3, $crate::display::preparation::Field::Characters0) };
    (@place Characters0Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Characters0, $crate::display::preparation::Field::Characters1) };
    (@place Characters1Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Characters1, $crate::display::preparation::Field::AffineSelection) };
    (@place AffineSelectionPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::AffineSelection, $crate::display::preparation::Field::WindowSelection0) };
    (@place WindowSelection0Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::WindowSelection0, $crate::display::preparation::Field::WindowSelection1) };
    (@place WindowSelection1Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::WindowSelection1, $crate::display::preparation::Field::WindowSelection2) };
    (@place WindowSelection2Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::WindowSelection2, $crate::display::preparation::Field::WindowPositions0) };
    (@place WindowPositions0Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::WindowPositions0, $crate::display::preparation::Field::WindowPositions1) };
    (@place WindowPositions1Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::WindowPositions1, $crate::display::preparation::Field::WindowPositions2) };
    (@place WindowPositions2Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::WindowPositions2, $crate::display::preparation::Field::WindowPositions3) };
    (@place WindowPositions3Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::WindowPositions3, $crate::display::preparation::Field::WindowLogic0) };
    (@place WindowLogic0Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::WindowLogic0, $crate::display::preparation::Field::WindowLogic1) };
    (@place WindowLogic1Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::WindowLogic1, $crate::display::preparation::Field::MainLayers) };
    (@place MainLayersPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::MainLayers, $crate::display::preparation::Field::PreviousMainLayers) };
    (@place PreviousMainLayersPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::PreviousMainLayers, $crate::display::preparation::Field::SubLayers) };
    (@place SubLayersPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::SubLayers, $crate::display::preparation::Field::MainWindow) };
    (@place MainWindowPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::MainWindow, $crate::display::preparation::Field::SubWindow) };
    (@place SubWindowPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::SubWindow, $crate::display::preparation::Field::NextColorSelection) };
    (@place NextColorSelectionPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::NextColorSelection, $crate::display::preparation::Field::ColorSelection) };
    (@place ColorSelectionPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::ColorSelection, $crate::display::preparation::Field::PreviousColorSelection) };
    (@place PreviousColorSelectionPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::PreviousColorSelection, $crate::display::preparation::Field::NextColorMath) };
    (@place NextColorMathPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::NextColorMath, $crate::display::preparation::Field::ColorMath) };
    (@place ColorMathPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::ColorMath, $crate::display::preparation::Field::PreviousColorMath) };
    (@place PreviousColorMathPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::PreviousColorMath, $crate::display::preparation::Field::FixedColor0) };
    (@place FixedColor0Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::FixedColor0, $crate::display::preparation::Field::FixedColor1) };
    (@place GeometryPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::Geometry, $crate::display::preparation::Field::AffineWords0Low) };
    (@place AffineWords0Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::AffineWords0Low, $crate::display::preparation::Field::AffineWords0High) };
    (@place AffineWords1Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::AffineWords1Low, $crate::display::preparation::Field::AffineWords1High) };
    (@place AffineWords2Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::AffineWords2Low, $crate::display::preparation::Field::AffineWords2High) };
    (@place AffineWords3Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::AffineWords3Low, $crate::display::preparation::Field::AffineWords3High) };
    (@place AffineWords4Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::AffineWords4Low, $crate::display::preparation::Field::AffineWords4High) };
    (@place AffineWords5Pair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::AffineWords5Low, $crate::display::preparation::Field::AffineWords5High) };
    (@place NmiEnablePair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::NmiEnable, $crate::display::preparation::Field::HdmaEnable) };
    (@place PendingInterrupt) => { $crate::display::preparation::Place::History($crate::display::preparation::Word::PendingInterrupt) };
    (@place CurrentInterrupt) => { $crate::display::preparation::Place::History($crate::display::preparation::Word::CurrentInterrupt) };
    (@place NmiPendingPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::NmiPending, $crate::display::preparation::Field::NmiFrameByte) };
    (@place NmiFrameBytePair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::NmiFrameByte, $crate::display::preparation::Field::NmiFrameWordLow) };
    (@place NmiFrameWordPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::NmiFrameWordLow, $crate::display::preparation::Field::NmiFrameWordHigh) };
    (@place NmiIncludingLagPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::NmiIncludingLagLow, $crate::display::preparation::Field::NmiIncludingLagHigh) };
    (@place NmiMissedPair) => { $crate::display::preparation::Place::Controls($crate::display::preparation::Field::NmiMissed, $crate::display::preparation::Field::NmiMaxMissed) };
    (@place HudPalette) => { $crate::display::preparation::Place::History($crate::display::preparation::Word::HudPalette) };
    (@place PreviousHealth) => { $crate::display::preparation::Place::History($crate::display::preparation::Word::PreviousHealth) };
    (@place PreviousMissiles) => { $crate::display::preparation::Place::History($crate::display::preparation::Word::PreviousMissiles) };
    (@place PreviousSupers) => { $crate::display::preparation::Place::History($crate::display::preparation::Word::PreviousSupers) };
    (@place PreviousBombs) => { $crate::display::preparation::Place::History($crate::display::preparation::Word::PreviousBombs) };
    (@place PreviousSelection) => { $crate::display::preparation::Place::History($crate::display::preparation::Word::PreviousSelection) };
}

#[macro_export]
macro_rules! frame_display_byte {
    ($game:expr, $field:expr => $value:expr) => {
        match ($field, $value) {
            (field, value) => $crate::display::preparation::Access::store_display_control_byte(
                &mut *$game,
                field,
                value,
                $crate::display::preparation::Store::Owned,
            ),
        }
    };
}
#[macro_export]
macro_rules! frame_display_history {
    ($game:expr, $word:expr => $value:expr) => {
        match ($word, $value) {
            (word, value) => $crate::display::preparation::Access::store_display_history_word(
                &mut *$game,
                word,
                value,
                $crate::display::preparation::Store::Named,
            ),
        }
    };
}
