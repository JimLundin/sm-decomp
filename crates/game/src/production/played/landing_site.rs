//! The first work interval of the Landing Site loader.

use canonical::Canonical;
use data::assets::landing_site::Graphics;
use data::map::MapCells;
pub use data::room::Area;
use types::RandomSequence;

use super::zebes::{Flow, LoadDestination};
use crate::commit::{VRAM_CHANNEL, write_words};
use crate::exact::direct_dma::{self, Transfer};
use crate::state::video::{
    BgMode, Brightness, ColourMath, DisplaySettings, FixedColour, Layer, Layers, Mosaic,
    ObjectSelect, Registers, SpriteCount, TileBases, TilemapBase, Video, VramWord, Windows,
};

/// Cells and map-station availability remembered for one area.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExploredMap {
    /// Explored map cells.
    pub cells: MapCells,
    /// Map-station data is available.
    pub available: bool,
}

/// The active map and the stored map this entrance selects.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AreaMaps {
    /// Active area's explored map.
    pub active: ExploredMap,
    /// Crateria's stored explored map.
    pub crateria: ExploredMap,
}

/// Prepared gameplay tilemaps in work RAM.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct ShadowTilemaps {
    /// BG1's 1,024 words, including the last carried word.
    pub bg1: Vec<u16>,
    /// BG2's 1,024 words, including the last carried word.
    pub bg2: Vec<u16>,
}

/// Selected entrance for an opened Landing Site load.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entrance {
    /// The prior entrance remains selected.
    Carried,
    /// Crateria's cutscene entrance.
    LandingSite,
}

/// Work that belongs to the unfinished loader.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Flight may still be running, or its destination waiting for a work opening.
    Pending,
    /// BG3 has drained and the sprite upload is in progress.
    Sprites(Transfer),
    /// Sprite DMA drained and the next phase can begin.
    Ready,
}

/// CPU video I/O and its carried configuration while loading.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VideoControl {
    /// Automatic joypad polling enabled.
    pub auto_read: bool,
    /// NMI, IRQ and HDMA disabled.
    pub interrupts_off: bool,
    /// Fast ROM access enabled.
    pub fast_rom: bool,
}

/// Complete receiving state, carried across the unfinished work.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Progress of the loader's own unfinished work.
    pub phase: Phase,
    /// The selected area used by the live map.
    pub area: Area,
    /// Selection made for the pending room placement.
    pub entrance: Entrance,
    /// Stored and active maps.
    pub maps: AreaMaps,
    /// The tilemaps waiting for upload.
    pub shadow: ShadowTilemaps,
    /// The display controls written directly at setup.
    pub control: VideoControl,
}

/// T1 work and transfer progress available before the cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// No additional receiving work reached.
    Hold,
    /// Open work, drain BG3 and advance to the supplied sprite progress.
    Open {
        /// Cumulative accepted sprite bytes.
        accepted: u16,
        /// Whether the read at that offset has already been selected.
        selected: bool,
    },
    /// Advance an already-open sprite transfer.
    Transfer {
        /// Cumulative accepted sprite bytes.
        accepted: u16,
        /// Whether the read at that offset has already been selected.
        selected: bool,
    },
    /// Complete the write still in the DMA pipeline.
    Drain,
}

/// Unavailable loader dependency or inconsistent work receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The dispatch target or scheduler phase cannot start the requested work.
    Work,
    /// Required video storage or graphics are invalid.
    Content,
    /// The transfer cannot advance within the available source.
    Dma(direct_dma::Fault),
}

impl State {
    /// Carry the outgoing area, explored map storage and shadow tilemaps.
    #[must_use]
    pub const fn new(area: Area, maps: AreaMaps, shadow: ShadowTilemaps) -> Self {
        Self {
            phase: Phase::Pending,
            area,
            entrance: Entrance::Carried,
            maps,
            shadow,
            control: VideoControl {
                auto_read: false,
                interrupts_off: false,
                fast_rom: false,
            },
        }
    }

    /// Dispatch pending work once, or advance an in-flight byte sequence. No NMI commit occurs.
    ///
    /// # Errors
    /// Wrong work order, destinations and unsupported graphics source reads reject the receipt.
    pub fn advance(
        &mut self,
        destination: Flow,
        random: &mut RandomSequence,
        video: &mut Video,
        content: &Graphics,
        work: Work,
    ) -> Result<(), Fault> {
        match (self.phase, work) {
            (_, Work::Hold) => Ok(()),
            (Phase::Pending, Work::Open { accepted, selected })
                if destination == Flow::Loading(LoadDestination::LandingSite) =>
            {
                let mut transfer = Transfer::new(
                    content.sprite_source,
                    VramWord(0x6000),
                    content.sprite_length,
                );
                transfer
                    .validate(&video.memory.vram, &content.sprites, accepted, selected)
                    .map_err(Fault::Dma)?;
                self.open(random, video, content)?;
                transfer
                    .advance(&mut video.memory.vram, &content.sprites, accepted, selected)
                    .map_err(Fault::Dma)?;
                project(video, &transfer);
                self.phase = Phase::Sprites(transfer);
                Ok(())
            }
            (Phase::Sprites(mut transfer), Work::Transfer { accepted, selected }) => {
                transfer
                    .advance(&mut video.memory.vram, &content.sprites, accepted, selected)
                    .map_err(Fault::Dma)?;
                project(video, &transfer);
                self.phase = Phase::Sprites(transfer);
                Ok(())
            }
            (Phase::Sprites(mut transfer), Work::Drain) => {
                transfer.drain(&mut video.memory.vram).map_err(Fault::Dma)?;
                project(video, &transfer);
                self.phase = Phase::Ready;
                Ok(())
            }
            _ => Err(Fault::Work),
        }
    }

    fn open(
        &mut self,
        random: &mut RandomSequence,
        video: &mut Video,
        content: &Graphics,
    ) -> Result<(), Fault> {
        if self.shadow.bg1.len() != 1024
            || self.shadow.bg2.len() != 1024
            || content.background.len() != 4096
        {
            return Err(Fault::Content);
        }
        game_random::advance_random(random);
        self.area = Area::Crateria;
        self.entrance = Entrance::LandingSite;
        self.maps.active = self.maps.crateria;
        self.control = VideoControl {
            auto_read: true,
            interrupts_off: true,
            fast_rom: true,
        };
        setup_display(&mut video.committed);
        setup_display(&mut video.batch.registers);
        // The direct COLDATA writes clear the live colour. The next NMI uses its queued writes.
        video.committed.colour_math.fixed = FixedColour::default();
        video.batch.sprites = SpriteCount(0);
        video.memory.oam.clear_first_position();
        for (index, tile) in self.shadow.bg1.iter_mut().take(1023).enumerate() {
            *tile = if index < 127 { 0x2c0f } else { 0x006f };
        }
        self.shadow
            .bg2
            .iter_mut()
            .take(1023)
            .for_each(|tile| *tile = 0);
        write_words(
            &mut video.memory.vram,
            VramWord(0x4000),
            1,
            content.background.iter().copied(),
        );
        Ok(())
    }
}

fn setup_display(registers: &mut Registers) {
    let mut layers = Layers::default();
    for layer in [Layer::Bg1, Layer::Bg2, Layer::Bg3, Layer::Objects] {
        layers.insert(layer);
    }
    *registers = Registers {
        forced_blank: true,
        brightness: Brightness(0),
        objects: ObjectSelect(3),
        mode: BgMode(9),
        mosaic: Mosaic(0),
        tilemaps: [
            TilemapBase(0x51),
            TilemapBase(0x49),
            TilemapBase(0x5a),
            TilemapBase(0),
        ],
        tile_bases: [TileBases(0), TileBases(4)],
        windows: Windows::default(),
        main_screen: layers,
        sub_screen: Layers::default(),
        main_window_mask: Layers::default(),
        sub_window_mask: Layers::default(),
        colour_math: ColourMath {
            fixed: registers.colour_math.fixed,
            ..ColourMath::default()
        },
        settings: DisplaySettings(0),
        ..*registers
    };
}

fn project(video: &mut Video, transfer: &Transfer) {
    if let Some(channel) = video.dma.get_mut(VRAM_CHANNEL) {
        transfer.project_channel(channel);
    }
}

/// Carried values selected by the scoped receiving recipe.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Run {
    /// The requested load context handed over by the flight.
    pub flow: Flow,
    /// The receiving loader.
    pub loader: State,
    /// Live PPU storage, register mirrors and queued display work.
    pub video: Video,
    /// Most recent display publication from the preceding owner.
    pub publication: Registers,
    /// Shared random sequence at the current work interval.
    pub random: u16,
    /// Finished scheduler work carried from the flight.
    pub finished: u16,
    /// Current whole energy and stored capacity.
    pub energy: [u16; 2],
    /// Current room, door, camera and Samus placement.
    pub placement: Placement,
    /// The flight's affine scroll and matrix mirrors.
    pub affine: [u16; 8],
    /// The entrance selected before the load opens.
    pub previous_entrance: u16,
}

/// Room and player placement that remains in force until later loading.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    /// Room whose state still supplies this placement.
    pub room: data::id::RoomId,
    /// Current doorway.
    pub door: Option<data::id::DoorwayId>,
    /// Camera whole coordinates.
    pub camera: [u16; 2],
    /// Samus's whole coordinates.
    pub samus: [u16; 2],
}

impl Run {
    pub(in crate::production) fn advance(
        &mut self,
        content: &Graphics,
        work: Work,
    ) -> Result<(), Fault> {
        let mut random = RandomSequence {
            number: types::RandomNumber(self.random),
        };
        self.loader
            .advance(self.flow, &mut random, &mut self.video, content, work)?;
        self.random = random.number.0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::video::ColourWrite;

    #[test]
    fn direct_black_preserves_the_next_nmi_colour_writes() {
        let queued = [ColourWrite(0x25), ColourWrite(0x4c), ColourWrite(0x91)];
        let mut video = Video::default();
        video.committed.colour_math.fixed = FixedColour {
            red: 2,
            green: 3,
            blue: 4,
        };
        video.batch.fixed_colour = queued;
        let mut loader = State::new(
            Area::Ceres,
            AreaMaps::default(),
            ShadowTilemaps {
                bg1: vec![0; 1024],
                bg2: vec![0; 1024],
            },
        );
        let graphics = Graphics {
            background: vec![0; 4096],
            sprites: vec![0],
            sprite_source: data::dma::BusAddress::default(),
            sprite_length: 1,
        };
        let mut random = RandomSequence {
            number: types::RandomNumber(0),
        };
        loader
            .advance(
                Flow::Loading(LoadDestination::LandingSite),
                &mut random,
                &mut video,
                &graphics,
                Work::Open {
                    accepted: 0,
                    selected: false,
                },
            )
            .unwrap();
        assert_eq!(video.committed.colour_math.fixed, FixedColour::default());
        assert_eq!(video.batch.fixed_colour, queued);
        video.commit_registers();
        assert_eq!(
            video.committed.colour_math.fixed,
            FixedColour {
                red: 5,
                green: 12,
                blue: 17
            }
        );
    }
}
