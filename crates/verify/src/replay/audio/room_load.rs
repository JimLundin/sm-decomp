//! The opening room's ROM-imported audio and its loaded senders, joined to the native capture.

use super::{AudioPacket, InputTape, encode, group};
use crate::lock::{Lock, RowId, Sha256};
use crate::replay::{Outcome, Sources, data_failure, read_lock};
use audio_transport::{ReleaseHistory, Request, SenderProgress, SenderState, SoundGroup, State};
use data::assets::world::rooms::MAGNET_STAIRS_ROOM;
use game::production::{ProductionContent, ProductionSeed, audio};
use serde::Deserialize;

#[derive(Deserialize)]
struct Groups<T> {
    action: T,
    ambient: T,
    contact: T,
}

/// The room-load packet. It also records the measured clock relation and the driver's timer
/// phase at the load; those are T1's and the frame form reads none of them.
#[derive(Deserialize)]
struct Packet {
    schema: String,
    room: String,
    locked_seed_sha256: String,
    control_sha256: String,
    expiry_sha256: String,
    requests: [Request; 3],
    history: Groups<ReleaseHistory>,
    sender: Groups<SenderState>,
}

impl Packet {
    fn load(
        sources: &Sources,
        declaration: &AudioPacket,
        current_tape: &str,
    ) -> Result<Self, Outcome> {
        let packet: Self = declaration.origin.read(&sources.native)?;
        let lock = if sources.lock.as_os_str().is_empty() {
            Lock::parse(include_str!("../../../../../scenarios/locks/lsnes.toml"))
                .map_err(|_| data_failure(None, "record lock schema"))?
        } else {
            read_lock(sources)?
        };
        let locked = lock
            .row(&RowId(declaration.seed_row.clone()))
            .and_then(|row| row.seed_record);
        let seed_identity = Sha256::try_from(packet.locked_seed_sha256.as_str())
            .map_err(|_| data_failure(None, "opening Ceres locked seed identity"))?;
        if packet.schema != "opening-ceres-audio-v1"
            || packet.room != "magnet-stairs"
            || Some(seed_identity) != locked
            || (packet.control_sha256 != current_tape && packet.expiry_sha256 != current_tape)
        {
            return Err(data_failure(None, "opening Ceres audio origin lineage"));
        }
        Ok(packet)
    }
}

/// The three groups in their enum order: Action, Ambient, Contact.
fn by_group<T>(groups: Groups<T>) -> enum_map::EnumMap<SoundGroup, T> {
    enum_map::EnumMap::from_array([groups.action, groups.ambient, groups.contact])
}

/// Admit the opening room's load: the ROM's resident bank, the capture's loaded audio side and
/// the load packet's senders, all three Ready with empty queues.
pub(in crate::replay) fn admit(
    sources: &Sources,
    declaration: &AudioPacket,
    tape: &InputTape,
    tape_hash: &str,
) -> Result<(ProductionSeed, ProductionContent), Outcome> {
    let image = std::fs::read(&sources.rom).map_err(|_| data_failure(None, "opening Ceres ROM"))?;
    let rom =
        data::rom::Rom::new(image).map_err(|_| data_failure(None, "opening Ceres ROM identity"))?;
    let bank = import::audio::resident_ceres(&rom, MAGNET_STAIRS_ROOM)
        .map_err(|_| data_failure(None, "opening Ceres room audio import"))?;
    if bank.content != tape.content
        || bank.music_reply != tape.seed.resident_music_reply
        || bank.echo_delay != tape.seed.song.mix.echo_delay
        || bank.aliases.len() != tape.aliases.len()
        || !bank
            .aliases
            .iter()
            .zip(&tape.aliases)
            .all(|(actual, oracle)| {
                group(oracle.group).ok() == Some(actual.group)
                    && actual.byte == oracle.byte
                    && actual.cue == oracle.cue
            })
    {
        return Err(data_failure(
            None,
            "ROM-selected audio and native graph disagree",
        ));
    }
    let packet = Packet::load(sources, declaration, tape_hash)?;
    let request_lines = enum_map::EnumMap::from_array(packet.requests);
    let history = by_group(packet.history);
    let senders = by_group(packet.sender);
    let seed = &tape.seed;
    let loaded = senders
        .values()
        .all(|sender| sender.progress == SenderProgress::Ready && sender.queue.is_empty());
    let replies_match = SoundGroup::IN_TURN_ORDER.iter().all(|line| {
        encode(&tape.aliases, *line, seed.groups[*line].reply)
            == tape.initial_replies.get(super::slot(*line)).copied()
    });
    if !loaded
        || request_lines != seed.requests
        || seed
            .groups
            .iter()
            .any(|(line, saved)| saved.history != history[line])
        || !replies_match
        || tape.initial_replies.first() != Some(&seed.resident_music_reply.original_byte)
    {
        return Err(data_failure(None, "opening Ceres audio load"));
    }
    let state = State {
        senders,
        request_lines,
        groups: seed.groups.clone(),
        song: seed.song.clone(),
        music_reply: seed.resident_music_reply.clone(),
        resident_music_reply: seed.resident_music_reply.clone(),
        reservations: seed.reservations.clone(),
    };
    let content = audio::Content::resident(bank)
        .map_err(|_| data_failure(None, "resident audio bank admission"))?;
    Ok((
        ProductionSeed::Audio(Box::new(state)),
        ProductionContent::Audio(Box::new(content)),
    ))
}
