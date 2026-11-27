//! Observe the locked room's HUD buffer, queued source and physical tilemap independently.

use super::{Outcome, data_failure, word};
use crate::replay::{compare::Observation, hud};
use crate::source::{VRAM_WRITE_QUEUE, VRAM_WRITE_QUEUE_TAIL};
use canonical::model::Model;
use data::rom::Lorom;
use game::production::Production;
use game::production::blackout::{HudContent, LockedHud, State};
use game::state::video::{VramSource, VramStep, VramTransfer};
use interface_hud::Countdown;
use oracle_record::{Cut, FrameRecord, NmiKind};
use types::Pair;

// The native entry writes the 96 HUD words from the WRAM grid to overlay row 1.
const DESCRIPTOR: [u8; 7] = [0xc0, 0, 0x08, 0xc6, 0x7e, 0x20, 0x58];

pub(in crate::replay) fn admit(
    record: &FrameRecord,
    rom: Lorom<'_>,
    state: &mut State,
) -> Result<HudContent, Outcome> {
    let ram = &record.wram;
    let content = hud::Content::from_rom(rom)?;
    let map = content.map(ram)?.clone();
    state.hud = Some(LockedHud {
        model: Model(hud::decode_state(ram)?),
        finished: hud::frame_count(ram)?.0,
    });
    state.owners.0.countdown = Countdown::Dormant;
    Ok(HudContent {
        pictures: content.pictures,
        map,
        room: hud::room_from_words(
            [word(ram, 0x07a1)?, word(ram, 0x07a3)?],
            [word(ram, 0x07a5)?, word(ram, 0x07a7)?],
        ),
        centre: Pair {
            x: word(ram, 0x0af6)?,
            y: word(ram, 0x0afa)?,
        },
        map_collected: hud::map_collected(ram)?,
        stock: hud::stock_at_start(ram)?,
        status: hud::status(ram)?,
    })
}

fn fields(
    mut observed: Vec<Observation>,
    count: u8,
    queue: Vec<u8>,
    serviced: Vec<u8>,
    vram: &[u16],
) -> Result<Vec<Observation>, Outcome> {
    let published = vram
        .get(0x5820..0x5880)
        .ok_or_else(|| data_failure(None, "physical HUD grid"))?;
    observed.extend([
        Observation::quantity("hud.frame-count", i64::from(count)),
        Observation {
            field: "hud.pending-transfer".to_owned(),
            bytes: queue,
        },
        Observation {
            field: "hud.publication".to_owned(),
            bytes: serviced,
        },
        Observation {
            field: "hud.published-grid".to_owned(),
            bytes: published
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect(),
        },
    ]);
    Ok(observed)
}

pub(in crate::replay) fn expected(
    prior: &FrameRecord,
    record: &FrameRecord,
) -> Result<Vec<Observation>, Outcome> {
    let pending = native_queue(record)?;
    let serviced = if matches!(record.cut, Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Requested) {
        let expected_dma = [1, 0x18, 0xc8, 0xc6, 0x7e, 0, 0];
        if !record
            .ppu
            .dma
            .get(1)
            .is_some_and(|channel| channel.starts_with(&expected_dma))
        {
            return Err(data_failure(Some(record.index), "native HUD DMA receipt"));
        }
        let prepared = native_queue(prior)?;
        if prepared.is_empty() {
            // Requested NMI clears the tail but leaves the descriptor and source in WRAM.
            native_entry(record)?
        } else {
            prepared
        }
    } else {
        Vec::new()
    };
    fields(
        hud::native_fields(&record.wram)?,
        hud::frame_count(&record.wram)?.0,
        pending,
        serviced,
        &record.vram,
    )
}

fn native_queue(record: &FrameRecord) -> Result<Vec<u8>, Outcome> {
    let tail = VRAM_WRITE_QUEUE_TAIL
        .read(&record.wram)
        .map_err(|_| data_failure(Some(record.index), "native regular HUD tail"))?;
    match tail {
        0 => Ok(Vec::new()),
        7 => native_entry(record),
        _ => Err(data_failure(Some(record.index), "native regular HUD queue")),
    }
}

fn native_entry(record: &FrameRecord) -> Result<Vec<u8>, Outcome> {
    let bad = || {
        data_failure(
            Some(record.index),
            "native regular HUD descriptor and source",
        )
    };
    let ram = &record.wram;
    let descriptor = VRAM_WRITE_QUEUE
        .read(ram)
        .map_err(|_| bad())?
        .first_chunk::<7>()
        .filter(|&descriptor| *descriptor == DESCRIPTOR)
        .ok_or_else(bad)?;
    let source = crate::source::HUD_TILEMAP.read(ram).map_err(|_| bad())?;
    Ok(descriptor.iter().chain(source).copied().collect())
}

pub(in crate::replay) fn actual(
    production: &Production,
    serviced: &[VramTransfer],
) -> Result<Vec<Observation>, Outcome> {
    let bad = || data_failure(None, "production locked HUD");
    let state = production.blackout().ok_or_else(bad)?;
    let hud = state.hud.as_ref().ok_or_else(bad)?;
    fields(
        hud::state_fields(&hud.model.0)?,
        hud.finished,
        queue_bytes(&state.video.batch.vram)?,
        queue_bytes(serviced)?,
        &state.video.memory.vram,
    )
}

fn queue_bytes(transfers: &[VramTransfer]) -> Result<Vec<u8>, Outcome> {
    let bad = || data_failure(None, "production regular HUD queue");
    let mut queue = Vec::new();
    for transfer in transfers {
        let VramSource::Words { from, words } = &transfer.source else {
            return Err(bad());
        };
        let bytes = u16::try_from(words.len())
            .map_err(|_| bad())?
            .checked_mul(2)
            .ok_or_else(bad)?;
        let to = match transfer.step {
            VramStep::Row => transfer.to.0,
            VramStep::Column => transfer.to.0 | 0x8000,
        };
        queue.extend(bytes.to_le_bytes());
        queue.extend(from.offset.to_le_bytes());
        queue.push(from.bank);
        queue.extend(to.to_le_bytes());
        queue.extend(words.iter().flat_map(|word| word.to_le_bytes()));
    }
    Ok(queue)
}
