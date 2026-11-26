//! The receiving vblank and a constructed continuation of its saved byte pipeline.

use super::compare::Observation;
use super::index::Case;
use super::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, record,
};
use super::result::Outcome;
use super::{Sources, data_failure, production_failure};
use crate::cache::Cache;
use crate::lock::{Lock, RowId};
use data::assets::landing_site::Graphics;
use data::id::{DoorwayId, RoomId};
use data::map::MapCells;
use data::rom::Rom;
use game::production::played::landing_site::{
    Area, AreaMaps, Entrance, ExploredMap, Phase, Placement, Run, ShadowTilemaps, State,
    VideoControl, Work,
};
use game::production::played::zebes::{Flow, LoadDestination};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules, ProductionSeed,
};
use game::state::owned::OwnedState;
use game::state::video::{
    BBusRegister, BgMode, Brightness, ColourMath, ColourMathLayers, ColourMathSelect, ColourWrite,
    DisplaySettings, DmaChannel, DmaControl, FixedColour, Layers, Mode7, Mode7Select, Mosaic,
    NmiBatch, Oam, OamEntry, ObjectSelect, Registers, Scroll, SpriteCount, TileBases, TilemapBase,
    Video, VideoMemory, VramWord, WindowSettings, Windows,
};
use oracle_record::{Cut, FrameRecord, NmiKind, PpuRegisters};
use types::{RandomNumber, RandomSequence};

pub(super) const ROW: &str = "chain-any-0011-DF45";
pub(super) const DEPARTURE: u64 = 2123;
pub(super) const RECEIVING: u64 = 2124;
pub(super) const RESUMED: u64 = 2125;
const SHAFT_SEED: u64 = 0;
const FLIGHT_SEED: u64 = 1004;
const SHADOW_WORDS: usize = 1024;
const AFFINE: [usize; 8] = [
    0x198d, 0x198f, 0x00b1, 0x00b3, 0x0078, 0x007a, 0x007c, 0x007e,
];
const ROOM: [usize; 6] = [0x079b, 0x078d, 0x0911, 0x0915, 0x0af6, 0x0afa];
const COLOUR_QUEUE: [usize; 3] = [0x74, 0x75, 0x76];
const MIRRORS: [usize; 10] = [0x51, 0x52, 0x55, 0x57, 0x58, 0x59, 0x5a, 0x5c, 0x5d, 0x5e];

pub(super) struct Inputs {
    pub receiving: Work,
    pub resumed: Work,
}

fn cache(lock: &Lock, root: &std::path::Path, row: &str) -> Result<Cache, Outcome> {
    let row = lock
        .row(&RowId(row.to_owned()))
        .ok_or_else(|| data_failure(None, "landing record row"))?;
    Cache::open(root, row).map_err(|_| data_failure(None, "landing pinned records"))
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let lock = std::fs::read_to_string(&sources.lock)
        .ok()
        .and_then(|text| Lock::parse(&text).ok())
        .ok_or_else(|| data_failure(None, "landing record lock"))?;
    let flight = cache(&lock, &sources.cache, ROW)?;
    let early = record(&flight, SHAFT_SEED)?;
    let departure = carry_flight(sources, &flight, &early)?;
    let arrival = record(&cache(&lock, &sources.cache, "any-zebes-arrival")?, 0)?;
    let rom = std::fs::read(&sources.rom)
        .ok()
        .and_then(|bytes| Rom::new(bytes).ok())
        .ok_or_else(|| data_failure(None, "landing player ROM"))?;
    let graphics = import::landing_site::import(rom.lorom())
        .map_err(|_| data_failure(None, "landing graphics import"))?;
    let accepted = timing(&arrival, &graphics)?;
    let continued = accepted.saturating_add(2);
    let expected = observations(&departure, &arrival, &graphics, accepted, continued)?;
    let content = ProductionContent::LandingSite(Box::new(graphics));
    let rules = ProductionRules::LandingSite;
    let production = Production::admit(
        ProductionSeed::LandingSite(Box::new(departure)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(Some(DEPARTURE), "landing production admission"))?;
    let inputs = Inputs {
        receiving: Work::Open {
            accepted,
            selected: true,
        },
        resumed: Work::Transfer {
            accepted: continued,
            selected: false,
        },
    };
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::LandingSite(inputs),
        expected: ExpectedReader::LandingSite(Box::new(expected)),
        actual: ActualReader::LandingSite,
        schedule: (RECEIVING..=case.end.cut_inclusive).collect(),
    })
}

const fn finished(record: &FrameRecord) -> bool {
    matches!(record.cut, Cut::NmiReturn(nmi) if matches!(nmi.kind, NmiKind::Requested))
}

fn carry_flight(sources: &Sources, flight: &Cache, early: &FrameRecord) -> Result<Run, Outcome> {
    let mut last = record(flight, FLIGHT_SEED)?;
    let mut vram = last.vram.to_vec();
    let mut random = RandomSequence {
        number: RandomNumber(word(&last, 0x05e5)?),
    };
    let (mut production, _, _) = super::zebes::admit_seed(&sources.rom, &last)?;
    for cut in FLIGHT_SEED.saturating_add(1)..=DEPARTURE {
        let current = record(flight, cut)?;
        if finished(&last) {
            game_random::advance_random(&mut random);
        }
        let cadence = if finished(&current) {
            screens_zebes::FrameAdvance::Finished
        } else {
            screens_zebes::FrameAdvance::BorrowedLag
        };
        let ProductionOutputs::Zebes(output) = production
            .advance(AdvanceInput::Zebes(cadence))
            .map_err(|fault| production_failure(cut, fault))?
        else {
            return Err(data_failure(Some(cut), "landing preceding flight scope"));
        };
        if let Some(pages) = output.pages.as_deref() {
            pages.install(&mut vram);
        }
        if random.number.0 != word(&current, 0x05e5)? {
            return Err(Outcome::Difference {
                cut,
                field: "landing.carried.random".to_owned(),
            });
        }
        last = current;
    }
    if vram.as_slice() != last.vram.as_ref() {
        return Err(Outcome::Difference {
            cut: DEPARTURE,
            field: "landing.carried.vram".to_owned(),
        });
    }
    let OwnedState::Zebes(state) = production.state() else {
        return Err(data_failure(
            Some(DEPARTURE),
            "landing departure production scope",
        ));
    };
    if state.flow != Flow::Loading(LoadDestination::LandingSite) {
        return Err(data_failure(Some(DEPARTURE), "landing pending departure"));
    }
    let [room, door, camera_x, camera_y, samus_x, samus_y] = words(&last, ROOM)?;
    let current_area = Area::try_from(byte(&last, 0x079f)?)
        .map_err(|_| data_failure(Some(DEPARTURE), "landing departure area"))?;
    let publication = registers(&last.ppu);
    Ok(Run {
        flow: state.flow,
        random: random.number.0,
        finished: state.finished,
        energy: [state.health.0.energy.whole.get(), state.maximum],
        affine: words(&last, AFFINE)?,
        previous_entrance: word(&last, 0x078b)?,
        placement: Placement {
            room: RoomId(room),
            door: DoorwayId(door),
            camera: [camera_x, camera_y],
            samus: [samus_x, samus_y],
        },
        loader: State::new(
            current_area,
            AreaMaps {
                active: map(&last, 0x07f7, 0x0789)?,
                crateria: map(early, 0xcd52, 0xd908)?,
            },
            ShadowTilemaps {
                bg1: tilemap(&last, 0x4000)?,
                bg2: tilemap(&last, 0x3000)?,
            },
        ),
        publication,
        video: Video {
            committed: publication,
            memory: VideoMemory {
                vram,
                cgram: state.palette,
                oam: oam(&last)?,
            },
            dma: last.ppu.dma.map(channel),
            batch: NmiBatch {
                registers: publication,
                fixed_colour: queued_colour(&last)?.map(ColourWrite),
                sprites: SpriteCount(word(&last, 0x0590)?),
                ..NmiBatch::default()
            },
            ..Video::default()
        },
    })
}

fn bytes(record: &FrameRecord, address: usize, length: usize) -> Result<&[u8], Outcome> {
    record
        .wram
        .get(address..address.saturating_add(length))
        .ok_or_else(|| data_failure(Some(record.index), "landing WRAM span"))
}

fn byte(record: &FrameRecord, address: usize) -> Result<u8, Outcome> {
    record
        .wram
        .get(address)
        .copied()
        .ok_or_else(|| data_failure(Some(record.index), "landing WRAM byte"))
}

pub(super) fn queued_colour(record: &FrameRecord) -> Result<[u8; 3], Outcome> {
    let [red, green, blue] = COLOUR_QUEUE;
    Ok([
        byte(record, red)?,
        byte(record, green)?,
        byte(record, blue)?,
    ])
}

fn word(record: &FrameRecord, address: usize) -> Result<u16, Outcome> {
    let &[low, high] = bytes(record, address, 2)?
        .first_chunk()
        .ok_or_else(|| data_failure(Some(record.index), "landing WRAM word"))?;
    Ok(u16::from_le_bytes([low, high]))
}

fn words<const N: usize>(record: &FrameRecord, addresses: [usize; N]) -> Result<[u16; N], Outcome> {
    let mut values = [0; N];
    for (value, address) in values.iter_mut().zip(addresses) {
        *value = word(record, address)?;
    }
    Ok(values)
}

fn map(record: &FrameRecord, cells: usize, flag: usize) -> Result<ExploredMap, Outcome> {
    let bitmap = bytes(record, cells, 256)?
        .first_chunk()
        .ok_or_else(|| data_failure(Some(record.index), "landing explored map"))?;
    Ok(ExploredMap {
        cells: MapCells::from_bitmap(bitmap),
        available: byte(record, flag)? != 0,
    })
}

fn tilemap(record: &FrameRecord, address: usize) -> Result<Vec<u16>, Outcome> {
    Ok(bytes(record, address, SHADOW_WORDS.saturating_mul(2))?
        .as_chunks::<2>()
        .0
        .iter()
        .copied()
        .map(u16::from_le_bytes)
        .collect())
}

pub(in crate::replay) fn oam(record: &FrameRecord) -> Result<Oam, Outcome> {
    let mut result = Oam::default();
    let (low, high) = record
        .oam
        .split_at_checked(512)
        .ok_or_else(|| data_failure(Some(record.index), "landing low OAM"))?;
    if high.len() != result.high.len() {
        return Err(data_failure(Some(record.index), "landing high OAM"));
    }
    for (entry, &[x, y, tile, attributes]) in result.low.iter_mut().zip(low.as_chunks::<4>().0) {
        *entry = OamEntry {
            x,
            y,
            tile,
            attributes,
        };
    }
    result.high.copy_from_slice(high);
    Ok(result)
}

pub(in crate::replay) fn registers(ppu: &PpuRegisters) -> Registers {
    let [tm, ts, tmw, tsw] = ppu.screens;
    let layer = |bits| Layers {
        bits: u32::from(bits),
        ..Layers::default()
    };
    let [aa, ab, ba, bb, centre_x, centre_y] = ppu.m7;
    let [red, green, blue] = ppu.fixed_color;
    Registers {
        forced_blank: ppu.inidisp & 0x80 != 0,
        brightness: Brightness(ppu.inidisp & 15),
        objects: ObjectSelect(ppu.obsel),
        mode: BgMode(ppu.bgmode),
        mosaic: Mosaic(ppu.mosaic),
        tilemaps: ppu.bg_sc.map(TilemapBase),
        tile_bases: [TileBases(ppu.bg12nba), TileBases(ppu.bg34nba)],
        scroll: ppu.bg_scroll.map(|[x, y]| Scroll { x, y }),
        mode7: Mode7 {
            select: Mode7Select(ppu.m7sel),
            matrix: [aa, ab, ba, bb],
            centre: [centre_x, centre_y],
        },
        windows: Windows {
            select: ppu.window_select.map(WindowSettings),
            edges: ppu.window_positions,
            background_logic: WindowSettings(ppu.wbglog),
            object_logic: WindowSettings(ppu.wobjlog),
        },
        main_screen: layer(tm),
        sub_screen: layer(ts),
        main_window_mask: layer(tmw),
        sub_window_mask: layer(tsw),
        colour_math: ColourMath {
            select: ColourMathSelect(ppu.cgwsel),
            layers: ColourMathLayers(ppu.cgadsub),
            fixed: FixedColour { red, green, blue },
        },
        settings: DisplaySettings(ppu.setini),
    }
}

pub(in crate::replay) const fn channel(
    [
        control,
        destination,
        s0,
        s1,
        source_bank,
        c0,
        c1,
        indirect_bank,
        t0,
        t1,
        line_counter,
    ]: [u8; 11],
) -> DmaChannel {
    DmaChannel {
        control: DmaControl::from_byte(control),
        destination: BBusRegister::from_byte(destination),
        source: u16::from_le_bytes([s0, s1]),
        source_bank,
        count: u16::from_le_bytes([c0, c1]),
        indirect_bank,
        table: u16::from_le_bytes([t0, t1]),
        line_counter,
    }
}

fn timing(arrival: &FrameRecord, graphics: &Graphics) -> Result<u16, Outcome> {
    let dma = channel(
        *arrival
            .ppu
            .dma
            .get(1)
            .ok_or_else(|| data_failure(Some(RECEIVING), "landing DMA channel"))?,
    );
    let accepted = graphics
        .sprite_length
        .checked_sub(dma.count)
        .ok_or_else(|| data_failure(Some(RECEIVING), "landing DMA count"))?;
    if !matches!(arrival.cut, Cut::VblankWithoutNmi)
        || arrival.video_frame != 15_197
        || arrival.cpu.k != 0x82
        || arrival.cpu.pc != 0x8327
        || dma.source_bank != graphics.sprite_source.bank
        || dma.source
            != graphics
                .sprite_source
                .advanced(accepted.saturating_add(1))
                .offset
    {
        return Err(data_failure(
            Some(RECEIVING),
            "landing T1 selected read and NMI-off interval",
        ));
    }
    Ok(accepted)
}

pub(super) fn advance(
    inputs: &Inputs,
    production: &mut Production,
    cut: u64,
) -> Result<ProductionOutputs, Outcome> {
    let work = match cut {
        RECEIVING => inputs.receiving,
        RESUMED => inputs.resumed,
        _ => return Err(data_failure(Some(cut), "landing playback schedule")),
    };
    production
        .advance(AdvanceInput::LandingSite(work))
        .map_err(|fault| production_failure(cut, fault))
}

fn values(field: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: field.to_owned(),
        bytes,
    }
}

fn packed(field: &str, words: impl IntoIterator<Item = u16>) -> Observation {
    values(
        field,
        words.into_iter().flat_map(u16::to_le_bytes).collect(),
    )
}

fn setup_bytes(registers: &Registers) -> Vec<u8> {
    let [bg1, bg2, bg3, bg4] = registers.tilemaps;
    let [tiles12, tiles34] = registers.tile_bases;
    vec![
        (u8::from(registers.forced_blank) << 7) | registers.brightness.0,
        registers.objects.0,
        registers.mode.0,
        registers.mosaic.0,
        bg1.0,
        bg2.0,
        bg3.0,
        bg4.0,
        tiles12.0,
        tiles34.0,
    ]
}

pub(super) fn display_mirrors(batch: &NmiBatch, control: VideoControl) -> Vec<u8> {
    setup_bytes(&batch.registers)
        .into_iter()
        .chain([
            u8::from(control.auto_read),
            u8::from(!control.interrupts_off),
            u8::from(control.fast_rom),
        ])
        .chain(batch.sprites.0.to_le_bytes())
        .chain(batch.fixed_colour.map(|write| write.0))
        .collect()
}

pub(super) fn observed_mirrors(record: &FrameRecord) -> Result<Vec<u8>, Outcome> {
    let mut mirrors = MIRRORS
        .into_iter()
        .map(|at| byte(record, at))
        .collect::<Result<Vec<_>, _>>()?;
    mirrors.extend_from_slice(bytes(record, 0x84, 3)?);
    mirrors.extend(word(record, 0x0590)?.to_le_bytes());
    mirrors.extend(queued_colour(record)?);
    Ok(mirrors)
}

pub(in crate::replay) const fn io_bytes(channel: DmaChannel) -> [u8; 11] {
    let [s0, s1] = channel.source.to_le_bytes();
    let [c0, c1] = channel.count.to_le_bytes();
    let [t0, t1] = channel.table.to_le_bytes();
    [
        channel.control.to_byte(),
        channel.destination.to_byte(),
        s0,
        s1,
        channel.source_bank,
        c0,
        c1,
        channel.indirect_bank,
        t0,
        t1,
        channel.line_counter,
    ]
}

fn map_bytes(active: &ExploredMap, crateria: &ExploredMap) -> Vec<u8> {
    active
        .cells
        .to_bitmap()
        .into_iter()
        .chain([u8::from(active.available)])
        .chain(crateria.cells.to_bitmap())
        .chain([u8::from(crateria.available)])
        .collect()
}

fn payload(semantics: Vec<Observation>, outputs: Vec<Observation>) -> ExpectedCut {
    let output_count = outputs.len();
    let semantic_count = semantics.len();
    let mut fields = semantics.clone();
    fields.extend(outputs);
    ExpectedCut {
        fields,
        semantics,
        semantic_count,
        output_count,
        replies: 0,
    }
}

pub(super) fn actual(production: &Production) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::LandingSite(run) = production.state() else {
        return Err(data_failure(None, "landing selected scope"));
    };
    let loader = &run.loader;
    let video = &run.video;
    let entrance = match loader.entrance {
        Entrance::Carried => run.previous_entrance,
        Entrance::LandingSite => 0x12,
    };
    let flow = super::zebes::flow_word(run.flow);
    let target = super::zebes::destination_word(run.flow.destination());
    let phase = match loader.phase {
        Phase::Pending => Vec::new(),
        Phase::Sprites(t) => canonical::to_bytes(&t),
        Phase::Ready => vec![1],
    };
    let physical = &video.memory.oam;
    let oam = physical
        .low
        .iter()
        .flat_map(|sprite| [sprite.x, sprite.y, sprite.tile, sprite.attributes])
        .chain(physical.high)
        .collect();
    let place = run.placement;
    let [energy, maximum] = run.energy;
    Ok(vec![
        Observation::quantity("landing.random", i64::from(run.random)),
        packed(
            "landing.carried",
            [
                flow,
                target,
                run.finished,
                energy,
                maximum,
                u16::from(u8::from(loader.area)),
                entrance,
                place.room.0,
                place.door.0,
            ]
            .into_iter()
            .chain(place.camera)
            .chain(place.samus)
            .chain(run.affine),
        ),
        values(
            "landing.map",
            map_bytes(&loader.maps.active, &loader.maps.crateria),
        ),
        values("landing.pipeline", phase),
        packed("landing.vram", video.memory.vram.iter().copied()),
        packed(
            "landing.cgram",
            video.memory.cgram.iter().map(|colour| colour.0),
        ),
        values("landing.oam", oam),
        values(
            "landing.display.live",
            canonical::to_bytes(&video.committed),
        ),
        values(
            "landing.display.published",
            canonical::to_bytes(&run.publication),
        ),
        values(
            "landing.display.mirrors",
            display_mirrors(&video.batch, loader.control),
        ),
        values(
            "landing.dma",
            video.dma.into_iter().flat_map(io_bytes).collect(),
        ),
        packed(
            "landing.tilemaps",
            loader
                .shadow
                .bg2
                .iter()
                .copied()
                .chain(loader.shadow.bg1.iter().copied()),
        ),
    ])
}

fn observations(
    run: &Run,
    arrival: &FrameRecord,
    graphics: &Graphics,
    accepted: u16,
    continued: u16,
) -> Result<[ExpectedCut; 2], Outcome> {
    let mut vram = arrival.vram.to_vec();
    let first = expected(run, arrival, graphics, &vram, accepted, true)?;
    for offset in accepted.saturating_sub(1)..continued.saturating_sub(1) {
        let value = graphics
            .sprites
            .get(usize::from(offset))
            .copied()
            .ok_or_else(|| data_failure(Some(RESUMED), "landing constructed source"))?;
        let slot = vram
            .get_mut(0x6000_usize.saturating_add(usize::from(offset / 2)))
            .ok_or_else(|| data_failure(Some(RESUMED), "landing constructed destination"))?;
        let mut pair = slot.to_le_bytes();
        if let Some(lane) = pair.get_mut(usize::from(offset % 2)) {
            *lane = value;
        }
        *slot = u16::from_le_bytes(pair);
    }
    Ok([
        first,
        expected(run, arrival, graphics, &vram, continued, false)?,
    ])
}

fn expected(
    run: &Run,
    arrival: &FrameRecord,
    graphics: &Graphics,
    vram: &[u16],
    accepted: u16,
    selected: bool,
) -> Result<ExpectedCut, Outcome> {
    use game::exact::direct_dma::{PendingWrite, Transfer};
    let pending = accepted.saturating_sub(1);
    let value = graphics
        .sprites
        .get(usize::from(pending))
        .copied()
        .ok_or_else(|| data_failure(Some(RECEIVING), "landing accepted byte identity"))?;
    let mut dma = arrival.ppu.dma;
    let native = dma
        .get_mut(1)
        .ok_or_else(|| data_failure(Some(RECEIVING), "landing observed channel"))?;
    if !selected {
        let mut continued = channel(*native);
        continued.source = graphics.sprite_source.advanced(accepted).offset;
        continued.count = graphics.sprite_length.saturating_sub(accepted);
        *native = io_bytes(continued);
    }
    let transfer = Transfer {
        source: graphics.sprite_source,
        destination: VramWord(0x6000),
        length: graphics.sprite_length,
        accepted,
        committed: pending,
        pending: Some(PendingWrite {
            offset: pending,
            value,
        }),
        selected: selected.then(|| graphics.sprite_source.advanced(accepted)),
    };
    let state = vec![
        Observation::quantity("landing.random", i64::from(word(arrival, 0x05e5)?)),
        packed(
            "landing.carried",
            words(
                arrival,
                [0x0998, 0xd914, 0x05b6, 0x09c2, 0x09c4, 0x079f, 0x078b],
            )?
            .into_iter()
            .chain(words(arrival, ROOM)?)
            .chain(words(arrival, AFFINE)?),
        ),
        values(
            "landing.map",
            bytes(arrival, 0x07f7, 256)?
                .iter()
                .copied()
                .chain([byte(arrival, 0x0789)?])
                .chain(bytes(arrival, 0xcd52, 256)?.iter().copied())
                .chain([byte(arrival, 0xd908)?])
                .collect(),
        ),
        values("landing.pipeline", canonical::to_bytes(&transfer)),
    ];
    Ok(payload(
        state,
        vec![
            packed("landing.vram", vram.iter().copied()),
            packed("landing.cgram", arrival.cgram.iter().copied()),
            values("landing.oam", arrival.oam.to_vec()),
            values(
                "landing.display.live",
                canonical::to_bytes(&registers(&arrival.ppu)),
            ),
            values(
                "landing.display.published",
                canonical::to_bytes(&run.publication),
            ),
            values("landing.display.mirrors", observed_mirrors(arrival)?),
            values("landing.dma", dma.into_iter().flatten().collect()),
            packed(
                "landing.tilemaps",
                tilemap(arrival, 0x3000)?
                    .into_iter()
                    .chain(tilemap(arrival, 0x4000)?),
            ),
        ],
    ))
}
