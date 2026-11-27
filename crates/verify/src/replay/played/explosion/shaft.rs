//! Admit the interrupted shaft publication and compare its outputs before station dispatch.

use super::{carry, load_content};
use crate::replay::{blackout, compared, data_failure, explosion, result::Outcome};
use data::assets::countdown::IndirectHdmaStart;
use data::rom::{Lorom, Rom, RomAddress};
use game::exact::save_files::Format;
use game::production::played::{departure, landing_site};
use game::production::{
    Production, ProductionContent, ProductionRules, ProductionSeed, Scope, blackout as owner,
    escape_departure::{Content, Shaft, State},
};
use game::state::video::{DMA_CHANNELS, DmaChannel, Scroll};
use oracle_record::{Cut, FrameRecord, NmiKind};
use std::path::Path;

pub(in crate::replay) fn starts(record: &FrameRecord) -> Result<bool, Outcome> {
    let flow = explosion::word(record, 0x998)?;
    let locked = matches!(flow, 32 | 33)
        && matches!(record.cut, Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Requested);
    Ok(explosion::word(record, 0x93f)? & 0x8000 != 0 && (locked || interrupted(record)?))
}

pub(in crate::replay) fn interrupted(record: &FrameRecord) -> Result<bool, Outcome> {
    Ok(explosion::word(record, 0x998)? == 33
        && matches!(record.cut,
            Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Overrun
                && nmi.interrupted_pc == 0x0080_82ce))
}

pub(super) fn seed(path: &Path, record: &FrameRecord) -> Result<(Shaft, Content), Outcome> {
    let image = std::fs::read(path).map_err(|_| data_failure(None, "shaft ROM file"))?;
    let rom = Rom::new(image).map_err(|_| data_failure(None, "shaft ROM pin"))?;
    let rom = rom.lorom();
    let format = Format::load(rom).map_err(|_| data_failure(None, "cartridge format"))?;
    let figures = import::countdown::import(rom).map_err(|_| data_failure(None, "timer art"))?;
    let late = interrupted(record)?;
    let mut blackout = blackout::seed::read(record, rom, &format, late)?;
    let hud = blackout::hud::admit(record, rom, &mut blackout)?;
    let standing = blackout::standing::admit_art(record, &mut blackout, rom, late)?;
    blackout.video.dma = record.ppu.dma.map(crate::replay::landing_site::channel);
    blackout.video.batch.fixed_colour =
        crate::replay::landing_site::queued_colour(record)?.map(game::state::video::ColourWrite);
    setup_scroll(&mut blackout.video.batch.registers.scroll, record)?;
    let raster_opening = raster_opening(record, &blackout.video.dma, rom)?;
    let (placement, area, maps, shadow) = carry::room(record)?;
    let shaft = Shaft {
        blackout,
        room: placement,
        landing: landing_site::State::new(area, maps, shadow),
        progress: (&crate::replay::progress::decode(&record.wram)?).into(),
    };
    let assets = Content {
        shaft: owner::Content {
            figures,
            format,
            graphics: data::pose::tiles::Graphics::default(),
            standing: Some(standing),
            hud: Some(hud),
        },
        raster_opening,
        departure: load_content(rom)?,
    };
    Ok((shaft, assets))
}

/// Admit the shaft owners already carried by the caller. No record is needed by production.
pub(super) fn admit_owned(
    shaft: Shaft,
    assets: Content,
    cut: u64,
) -> Result<(Production, ProductionContent, ProductionRules), Outcome> {
    let content = ProductionContent::Escape(Box::new(assets));
    let rules = ProductionRules::Departure(departure::Rules::REFERENCE);
    let production = Production::admit(
        ProductionSeed::Escape(Box::new(State::Shaft(Box::new(shaft)))),
        content.clone(),
        rules.clone(),
    )
    .map_err(|why| data_failure(Some(cut), &format!("shaft intake {why:?}")))?;
    Ok((production, content, rules))
}

fn setup_scroll(scroll: &mut [Scroll; 4], record: &FrameRecord) -> Result<(), Outcome> {
    const MIRRORS: [[usize; 2]; 4] = [[0xb1, 0xb3], [0xb5, 0xb7], [0xb9, 0xbb], [0xbd, 0xbf]];
    for (layer, [x, y]) in scroll.iter_mut().zip(MIRRORS) {
        layer.x = explosion::word(record, x)?;
        layer.y = explosion::word(record, y)?;
    }
    Ok(())
}

fn raster_opening(
    record: &FrameRecord,
    dma: &[DmaChannel; DMA_CHANNELS],
    rom: Lorom<'_>,
) -> Result<[Option<IndirectHdmaStart>; DMA_CHANNELS], Outcome> {
    const HDMA_ENABLE_MIRROR: usize = 0x85;
    let mask = explosion::byte(record, HDMA_ENABLE_MIRROR)?;
    let mut opening = [None; DMA_CHANNELS];
    for (i, (slot, channel)) in opening.iter_mut().zip(dma).enumerate() {
        if mask & (1 << i) != 0 {
            let source = RomAddress::new(channel.source_bank, channel.source);
            *slot = Some(
                import::countdown::indirect_hdma_start(rom, source)
                    .map_err(|_| data_failure(Some(record.index), "shaft HDMA table"))?,
            );
        }
    }
    Ok(opening)
}

pub(super) fn check(
    production: &Production,
    output: &owner::Output,
    previous: &FrameRecord,
    current: &FrameRecord,
    cut: u64,
) -> Result<(), Outcome> {
    // The committed save and its following dispatch retain the shaft scroll.
    if explosion::pending(current)? || explosion::pending(previous)? {
        let shaft = production
            .blackout()
            .ok_or_else(|| data_failure(Some(cut), "shaft publication"))?;
        super::super::room_video::compare_shaft_bg2_scroll(&shaft.video, current, cut)?;
    }
    let disable_interrupts = output
        .requests
        .contains(&owner::Request::DisableShaftInterrupts);
    let shaft = if disable_interrupts {
        Some(
            production
                .blackout()
                .ok_or_else(|| data_failure(Some(cut), "shaft publication"))?,
        )
    } else {
        None
    };
    if let Some(shaft) = shaft {
        let actual = shaft.video.memory.vram.as_slice();
        let native = current.vram.as_ref();
        // The standing draw leaves these parts of the ordinary body strips in place.
        let mut words = (0x6060..0x6080).chain(0x60e0..0x6100).chain(0x6140..0x6180);
        if let Some(word) = words.find(|&word| actual.get(word) != native.get(word)) {
            return Err(Outcome::Difference {
                cut,
                field: format!("room-video.samus-strips[{word:04x}]"),
            });
        }
    }
    compared(
        cut,
        &blackout::observe::expected(previous, current, true)?.fields,
        &blackout::actual(production, output, true)?,
    )?;
    compared(
        cut,
        &blackout::hud::expected(previous, current)?.fields,
        &blackout::hud::actual(production, &output.regular_publication)?,
    )?;
    if let Some(shaft) = shaft
        && let Some(i) = shaft
            .video
            .dma
            .iter()
            .zip(current.ppu.dma.map(crate::replay::landing_site::channel))
            .position(|(actual, expected)| *actual != expected)
    {
        return Err(Outcome::Difference {
            cut,
            field: format!("blackout.dma[{i}]"),
        });
    }
    Ok(())
}

pub(super) fn check_cartridge(
    production: &Production,
    current: &FrameRecord,
    cut: u64,
) -> Result<(), Outcome> {
    if production.scope() != Scope::Escape {
        return Ok(());
    }
    let image = production
        .battery()
        .ok_or_else(|| data_failure(Some(cut), "departure cartridge"))?;
    if image.as_slice() != current.sram.as_ref() {
        return Err(Outcome::Difference {
            cut,
            field: "departure.cartridge".into(),
        });
    }
    Ok(())
}
