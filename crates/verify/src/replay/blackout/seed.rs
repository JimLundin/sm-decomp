//! Admit physical room and cartridge owners only once, including an in-flight Mode 7 write.
use super::{Outcome, data_failure, digits, production, word};
use crate::replay::{health_tail, hud, save_files::Capture};
use canonical::model::Model;
use data::rom::{Lorom, RomAddress};
use game::exact::save_files::Format;
use game::production::blackout::{Owners, Platform, State};
use game::state::video::{Colour, Oam, OamEntry, Video, VramWord};
use oracle_record::FrameRecord;
use save_files::SaveFormat;
use types::{
    RandomNumber, RandomSequence,
    ceres_blackout::{
        BlackoutLevel, BlackoutLight, BlackoutPhase, BlackoutState, BlackoutTimer, FinishedPasses,
        PreparedPicture, PublishedPicture, ShaftStage, TimerActivity, TimerCentiseconds,
        TimerSprite, WorkPass,
    },
    file_select::Slot,
    save_files::{BatteryImage, PendingAction},
};

fn oam(bytes: &[u8]) -> Option<Oam> {
    let (low, high) = bytes.split_at_checked(512)?;
    let mut oam = Oam {
        high: high.try_into().ok()?,
        ..Oam::default()
    };
    for (entry, &[x, y, tile, attributes]) in oam.low.iter_mut().zip(low.as_chunks::<4>().0) {
        *entry = OamEntry {
            x,
            y,
            tile,
            attributes,
        };
    }
    Some(oam)
}

pub(in crate::replay) fn read(
    r: &FrameRecord,
    rom: Lorom<'_>,
    format: &Format,
    late: bool,
) -> Result<State, Outcome> {
    let bad = || data_failure(Some(r.index), "blackout start owners");
    let ram = &r.wram;
    let light = |n| BlackoutLevel::new(n).map_or(BlackoutLight::ForcedBlank, BlackoutLight::Lit);
    let timer_sprites = |o: &Oam| {
        o.low
            .iter()
            .take(17)
            .map(|e| TimerSprite(u16::from_le_bytes([e.tile, e.attributes])))
            .collect()
    };
    let capture = Capture::from_record(r).ok_or_else(bad)?;
    let battery = BatteryImage::new(r.sram.to_vec()).ok_or_else(bad)?;
    let mut files = production::save_files::power_on(format, &battery);
    let slot = match word(ram, 0x952)? {
        0 => Slot::First,
        1 => Slot::Second,
        2 => Slot::Third,
        _ => return Err(bad()),
    };
    files.active = Some(slot);
    files.pending = PendingAction::Idle;
    let mut run = format.decode(&files.cartridge.slots[slot].payload);
    run.profile = capture.profile;
    run.progress = capture.progress;
    run.maps = capture.maps;
    files.working.profile = run.profile.clone();
    files.working.maps = run.maps.clone();
    let mut video = display(r)?;
    let platform = if late {
        video.batch.vram = uploads(r)?;
        if word(ram, 0x334)? != 9 || word(ram, 0x2d4)? != 4 || super::byte(ram, 0x2d0)? != 128 {
            return Err(bad());
        }
        let source = RomAddress::new(super::byte(ram, 0x2d3)?, word(ram, 0x2d1)?);
        Some(Platform {
            start: VramWord(word(ram, 0x2d6)?),
            tiles: import::countdown::platform_tiles(rom, source).map_err(|_| bad())?,
        })
    } else {
        None
    };
    let scene = BlackoutState {
        phase: if late {
            BlackoutPhase::AwaitBlank
        } else {
            match word(ram, 0x998)? {
                8 => BlackoutPhase::Shaft,
                32 => BlackoutPhase::Hold(
                    types::ceres_blackout::WaitPasses::new(
                        u8::try_from(word(ram, 0xaa0)?).map_err(|_| bad())?,
                    )
                    .ok_or_else(bad)?,
                ),
                33 => BlackoutPhase::Fade,
                _ => return Err(bad()),
            }
        },
        light: light(super::byte(ram, 0x51)? & 15),
        published: PublishedPicture {
            light: light(r.ppu.inidisp & 15),
            timer_sprites: timer_sprites(&video.memory.oam),
        },
        pass: if late {
            WorkPass::Prepared(PreparedPicture {
                timer_sprites: timer_sprites(&video.batch.oam),
            })
        } else {
            WorkPass::Finished
        },
        timer: BlackoutTimer {
            activity: TimerActivity::Active,
            time: TimerCentiseconds(digits(ram)?),
        },
        shaft: ShaftStage::Rotating,
        finished: FinishedPasses(word(ram, 0x5b6)?),
    };
    Ok(State {
        owners: Model(Owners {
            scene,
            countdown: hud::countdown_origin(ram)?,
            files: production::save_files::State::new(files),
            run,
            live_map: capture.live_map,
            location: (capture.area, capture.station),
            health: health_tail::decode(ram, Some(r.index))?,
            random: RandomSequence {
                number: RandomNumber(word(ram, 0x5e5)?),
            },
        }),
        video,
        platform,
        hud: None,
    })
}

/// Resolve every queued regular transfer from the admitted WRAM. This seed only has WRAM sources.
fn uploads(record: &FrameRecord) -> Result<Vec<game::state::video::VramTransfer>, Outcome> {
    use crate::source::{VRAM_WRITE_QUEUE, VRAM_WRITE_QUEUE_TAIL};
    use game::state::video::{BusAddress, VramSource, VramStep, VramTransfer};
    let bad = || data_failure(Some(record.index), "pending regular room transfer");
    let ram = &record.wram;
    let length = VRAM_WRITE_QUEUE_TAIL.read(ram).map_err(|_| bad())?;
    let (queue, partial) = VRAM_WRITE_QUEUE
        .read(ram)
        .map_err(|_| bad())?
        .get(..usize::from(length))
        .ok_or_else(bad)?
        .as_chunks::<7>();
    if !partial.is_empty() {
        return Err(bad());
    }
    queue
        .iter()
        .map(|&[ll, lh, sl, sh, bank, dl, dh]| {
            let length = u16::from_le_bytes([ll, lh]);
            let offset = u16::from_le_bytes([sl, sh]);
            let source = usize::from(bank.checked_sub(0x7e).filter(|n| *n < 2).ok_or_else(bad)?)
                << 16
                | usize::from(offset);
            let raw = ram
                .get(source..)
                .and_then(|data| data.get(..usize::from(length)))
                .ok_or_else(bad)?;
            let (pairs, partial) = raw.as_chunks::<2>();
            if !partial.is_empty() {
                return Err(bad());
            }
            let destination = u16::from_le_bytes([dl, dh]);
            Ok(VramTransfer {
                to: VramWord(destination & 0x7fff),
                step: if destination & 0x8000 == 0 {
                    VramStep::Row
                } else {
                    VramStep::Column
                },
                source: VramSource::Words {
                    from: BusAddress { bank, offset },
                    words: pairs.iter().copied().map(u16::from_le_bytes).collect(),
                },
            })
        })
        .collect()
}

fn display(r: &FrameRecord) -> Result<Box<Video>, Outcome> {
    let bad = || data_failure(Some(r.index), "blackout video origin");
    let ram = &r.wram;
    let mut video = Box::<Video>::default();
    video.memory.vram = r.vram.to_vec();
    video.memory.cgram = r
        .cgram
        .iter()
        .copied()
        .map(Colour)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| bad())?;
    video.memory.oam = oam(&r.oam).ok_or_else(bad)?;
    video.committed = crate::ppu::registers(&r.ppu);
    video.batch.registers = video.committed;
    video.batch.oam = oam(ram.get(0x370..0x590).ok_or_else(bad)?).ok_or_else(bad)?;
    for (colour, bytes) in video
        .batch
        .cgram
        .iter_mut()
        .zip(ram.get(0xc000..0xc200).ok_or_else(bad)?.as_chunks::<2>().0)
    {
        *colour = Colour(u16::from_le_bytes(*bytes));
    }

    Ok(video)
}
