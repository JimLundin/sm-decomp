//! `sm_data`: identities, typed content decoded once from the ROM, profiles and the
//! portable records that cross from `sm_game` to `sm_runtime`.

pub mod ids {
    /// A room's lasting identity. Never its header address or its index in a world.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RoomId(pub u16);
    /// One state of a room (the state chosen by its conditions on entry).
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct RoomStateId {
        pub room: RoomId,
        pub index: u8,
    }
    /// A physical doorway of a room. Map Rando's `DoorPtrPair` and sm-json-data's
    /// `(room, door node)` resolve to one of these at import.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct DoorwayId(pub u16);
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ItemLocationId(pub u8);
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct LockId(pub u16);
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct PoseId(pub u8);
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct SpeciesId(pub u16);
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct AssetId(pub u32);
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct SongSetId(pub u8);

    /// The area a room belongs to: keys boss and event state and the vanilla
    /// area-indexed tables.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum HomeArea {
        Crateria,
        Brinstar,
        Norfair,
        WreckedShip,
        Maridia,
        Tourian,
        Ceres,
        Debug,
    }
    /// The area a placement puts a room in: keys the map page, explored bits, map
    /// station and area themes. Equal to the home area in the vanilla world.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct MapArea(pub HomeArea);
}

pub mod program {
    //! The program families' shared step form. The decoder in `sm_data::import`
    //! builds every `Program<F>`; the engine never reads list bytes.
    use core::fmt::Debug;
    use core::marker::PhantomData;

    /// A frame count as the original's instruction timers hold it.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Frames(pub u16);

    /// One slot-table family of the original (enemies, PLMs, enemy projectiles,
    /// sprite objects, HDMA objects, palette effects, animated tiles, cinematic
    /// objects) or a single-cursor consumer (Samus animation, demo input).
    pub trait Family: Copy + Debug + 'static {
        /// What a timed entry shows: a spritemap, a block draw, an HDMA table, ...
        type Payload: Copy + Debug;
        /// The family's own instructions.
        type Op: Copy + Debug;
        /// The per-frame callback a cursor carries instead of a function pointer.
        type PreOp: Copy + Debug + Default;
    }

    #[derive(Debug)]
    pub struct ProgramId<F>(pub u16, PhantomData<F>);
    impl<F> Clone for ProgramId<F> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<F> Copy for ProgramId<F> {}
    impl<F> PartialEq for ProgramId<F> {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    impl<F> ProgramId<F> {
        pub const fn new(index: u16) -> Self {
            Self(index, PhantomData)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct StepIndex(pub u16);

    /// A place in a program: every address the original jumps into or compares
    /// against becomes a label.
    #[derive(Debug)]
    pub struct Label<F> {
        pub program: ProgramId<F>,
        pub at: StepIndex,
    }
    impl<F> Clone for Label<F> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<F> Copy for Label<F> {}
    impl<F> PartialEq for Label<F> {
        fn eq(&self, other: &Self) -> bool {
            self.program == other.program && self.at == other.at
        }
    }

    /// The instructions every family shares.
    #[derive(Debug)]
    pub enum CoreOp<F: Family> {
        Delete,
        Sleep,
        Goto(Label<F>),
        /// Link and return: the PLM family's one-deep call.
        Link(Label<F>),
        Return,
        SetPreOp(F::PreOp),
        ClearPreOp,
        SetTimer(u16),
        DecrementTimerAndBranch(Label<F>),
        QueueSound(super::sound::SoundRequest),
    }
    impl<F: Family> Clone for CoreOp<F> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<F: Family> Copy for CoreOp<F> {}

    #[derive(Debug)]
    pub enum Step<F: Family> {
        Show { frames: Frames, payload: F::Payload },
        Core(CoreOp<F>),
        Family(F::Op),
    }
    impl<F: Family> Clone for Step<F> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<F: Family> Copy for Step<F> {}

    #[derive(Debug)]
    pub struct Program<F: Family> {
        pub steps: Box<[Step<F>]>,
    }

    /// Every program of a family, plus the labels the engine names.
    #[derive(Debug)]
    pub struct Programs<F: Family> {
        pub programs: Box<[Program<F>]>,
    }
    impl<F: Family> Programs<F> {
        pub fn step(&self, at: Label<F>) -> Option<Step<F>> {
            self.programs
                .get(usize::from(at.program.0))?
                .steps
                .get(usize::from(at.at.0))
                .copied()
        }
    }
}

pub mod world {
    //! Room content, placement and topology are three records (Map Rando report,
    //! decision 2).
    use super::ids::{DoorwayId, HomeArea, LockId, MapArea, RoomId};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DoorAxis {
        Horizontal,
        Vertical,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DoorwayKind {
        Normal(DoorAxis),
        Sand,
        Elevator,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Facing {
        Left,
        Right,
        Up,
        Down,
    }
    /// What arriving through this doorway does, owned by the doorway.
    #[derive(Clone, Debug)]
    pub struct Arrival {
        pub facing: Facing,
        pub spawn_distance: u16,
        pub cap_position: (u8, u8),
        /// Typed door actions (scroll region changes, ...) instead of door ASM.
        pub actions: Box<[DoorAction]>,
    }
    #[derive(Clone, Copy, Debug)]
    pub enum DoorAction {
        SetScroll { region: u8, scroll: Scroll },
        StartTreadmill,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Scroll {
        Red,
        Blue,
        Green,
    }
    #[derive(Clone, Debug)]
    pub struct Doorway {
        pub id: DoorwayId,
        pub room: RoomId,
        pub kind: DoorwayKind,
        pub arrival: Arrival,
    }
    #[derive(Clone, Copy, Debug)]
    pub struct Placement {
        pub room: RoomId,
        pub map_area: MapArea,
        pub map_x: u8,
        pub map_y: u8,
        pub sub_area: u8,
    }
    #[derive(Clone, Copy, Debug)]
    pub enum DoorwayEnd {
        Connected(DoorwayId),
        Wall,
    }
    #[derive(Clone, Copy, Debug)]
    pub struct Connection {
        pub a: DoorwayId,
        pub b: DoorwayEnd,
        pub lock: Option<LockId>,
    }
    #[derive(Clone, Debug)]
    pub struct Topology {
        pub connections: Box<[Connection]>,
    }
    #[derive(Clone, Debug)]
    pub struct World {
        pub placements: Box<[Placement]>,
        pub topology: Topology,
    }
    /// The room record a placement refers to; its home area is part of its identity.
    #[derive(Clone, Debug)]
    pub struct Room {
        pub id: RoomId,
        pub home_area: HomeArea,
        pub doorways: Box<[DoorwayId]>,
    }
}

pub mod display {
    //! The display program the renderer consumes. Typed register records over a
    //! layout-exact video memory image; no register addresses, no raw bytes in
    //! engine logic.

    /// A scanline, 0 at the top of the picture.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Line(pub u16);
    impl Line {
        pub const VISIBLE: u16 = 224;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub enum BgMode {
        Mode0,
        #[default]
        Mode1,
        Mode1Bg3Priority,
        Mode7,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Scroll {
        pub x: u16,
        pub y: u16,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Layer {
        pub tilemap_base: u16,
        pub tile_base: u16,
        pub scroll: Scroll,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Screens {
        pub main: LayerSet,
        pub sub: LayerSet,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct LayerSet {
        pub bg: [bool; 4],
        pub obj: bool,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct WindowEdges {
        pub left: u8,
        pub right: u8,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct ColorMath {
        pub subtract: bool,
        pub half: bool,
        pub layers: LayerSet,
        pub backdrop: bool,
        pub fixed: [u8; 3],
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Brightness(pub u8);

    /// The architectural PPU state that decides pixels.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Registers {
        pub forced_blank: bool,
        pub brightness: Brightness,
        pub mode: BgMode,
        pub layers: [Layer; 4],
        pub screens: Screens,
        pub windows: [WindowEdges; 2],
        pub color_math: ColorMath,
        pub mode7_matrix: [i16; 4],
    }

    /// One register change taking effect from a line.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Change {
        Scroll { layer: u8, scroll: Scroll },
        Mode(BgMode),
        Window { window: u8, edges: WindowEdges },
        Screens(Screens),
        ColorMath(ColorMath),
        ForcedBlank(bool),
        Layer { layer: u8, layer_state: Layer },
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct LineChange {
        pub line: Line,
        pub change: Change,
    }

    /// When an upload reaches video memory.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CommitPoint {
        /// The requested NMI's batch, before line 0 is drawn.
        Nmi,
        /// Inside a forced-blank range starting at this line (door IRQ update).
        BlankLine(Line),
        /// The main strand, with the whole display forced blank (loading).
        Loading,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Target {
        Vram { word: u16 },
        Cgram { color: u8 },
        Oam { entry: u8 },
        OamHigh { byte: u8 },
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Upload {
        pub at: CommitPoint,
        pub target: Target,
        pub words: Box<[u16]>,
    }

    /// The frame's display: committed uploads in order, the registers at line 0
    /// and every change at its line.
    #[derive(Clone, Debug, Default)]
    pub struct DisplayFrame {
        pub uploads: Vec<Upload>,
        pub start: Registers,
        pub changes: Vec<LineChange>,
    }

    /// SNES-layout video memory: the verification streams and the renderer both
    /// read it as laid out on the console.
    #[derive(Clone, Debug)]
    pub struct VideoMemory {
        pub vram: Box<[u16]>,
        pub cgram: Box<[u16]>,
        pub oam: Box<[u16]>,
        pub oam_high: Box<[u8]>,
    }
    impl Default for VideoMemory {
        fn default() -> Self {
            Self {
                vram: vec![0; 0x8000].into_boxed_slice(),
                cgram: vec![0; 0x100].into_boxed_slice(),
                oam: vec![0; 0x100].into_boxed_slice(),
                oam_high: vec![0; 0x20].into_boxed_slice(),
            }
        }
    }
}

pub mod sound {
    //! Sound commands, typed, with the byte each puts on its APU port.
    use super::ids::SongSetId;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Port {
        Music,
        Library1,
        Library2,
        Library3,
    }
    impl Port {
        pub const ALL: [Self; 4] = [Self::Music, Self::Library1, Self::Library2, Self::Library3];
        pub fn index(self) -> usize {
            match self {
                Self::Music => 0,
                Self::Library1 => 1,
                Self::Library2 => 2,
                Self::Library3 => 3,
            }
        }
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SoundId(pub u8);
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Library {
        One,
        Two,
        Three,
    }
    /// What a program or a rule asks the CPU-side sound handler for.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SoundRequest {
        Effect { library: Library, sound: SoundId },
        Music(MusicRequest),
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum MusicRequest {
        Track(u8),
        SongSet(SongSetId),
        Stop,
    }
    /// The latest value written to each port in a Tick, as the oracle's port queue
    /// records it. `None` is an unwritten port.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct PortFrame(pub [Option<u8>; 4]);
    impl PortFrame {
        pub fn write(&mut self, port: Port, value: u8) {
            self.0[port.index()] = Some(value);
        }
    }
    /// One Tick's sound output: the port bytes (the compared stream) and the
    /// typed commands they encode (what the sequencer plays).
    #[derive(Clone, Debug, Default)]
    pub struct SoundFrame {
        pub ports: PortFrame,
        pub uploads: Vec<SongSetId>,
    }
}

pub mod profile {
    //! Gameplay options are typed fields owned by the rule they change. Every
    //! field's default is vanilla.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub enum SuitRule {
        #[default]
        Vanilla,
        Complementary,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub enum DoorSpeed {
        #[default]
        Vanilla,
        Doubled,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub enum FailurePolicy {
        #[default]
        Reproduce,
        Death,
        Warn,
        Continue,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct GameplayProfile {
        pub suits: SuitRule,
        pub doors: DoorSpeed,
        pub failures: FailurePolicy,
        pub respin: bool,
    }
    impl GameplayProfile {
        pub const VANILLA: Self = Self {
            suits: SuitRule::Vanilla,
            doors: DoorSpeed::Vanilla,
            failures: FailurePolicy::Reproduce,
            respin: false,
        };
        pub const MAP_RANDO_BASE: Self = Self {
            suits: SuitRule::Complementary,
            ..Self::VANILLA
        };
    }
    /// The timing model the engine reproduces: part of the gameplay fingerprint.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ReferenceProfile {
        /// Ticks per second as an exact ratio of the NTSC master clock.
        pub cadence: (u64, u64),
        pub lag: LagModel,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum LagModel {
        /// The faithful oracle's rule: a Tick ends where the code waits for NMI.
        Structural,
        /// Hardware lag supplied as a recorded per-Tick marker (not built).
        RecordedOverrun,
    }
}
