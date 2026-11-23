//! The opening room's ROM-imported audio and its loaded senders, joined to the native capture.

use super::{InputTape, encode, group};
use crate::lock::{Lock, RowId, Sha256};
use crate::replay::{Outcome, data_failure};
use audio_transport::{ReleaseHistory, Request, SenderProgress, SenderState, SoundGroup, State};
use data::assets::world::rooms::MAGNET_STAIRS_ROOM;
use game::production::{ProductionContent, ProductionSeed, audio};
use serde::Deserialize;
use std::path::Path;

const MEMBER: &str = "opening-ceres-audio-v1/origin.toml";
const PIN: &str = "d9605917649a048cec3e66f1dd5debb703849b8095b54776d31fb88f87b94ed2";

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
    fn load(bytes: &[u8], current_tape: &str) -> Result<Self, Outcome> {
        if Sha256::try_from(PIN).ok() != Some(Sha256::of(bytes)) {
            return Err(data_failure(None, "opening Ceres audio origin pin"));
        }
        let source = std::str::from_utf8(bytes)
            .map_err(|_| data_failure(None, "opening Ceres audio origin encoding"))?;
        let packet: Self = toml::from_str(source)
            .map_err(|_| data_failure(None, "opening Ceres audio origin fields"))?;
        let locked = Lock::parse(include_str!("../../../../../scenarios/locks/lsnes.toml"))
            .ok()
            .and_then(|lock| {
                lock.row(&RowId("chain-any-0003-DFD7".to_owned()))
                    .and_then(|row| row.seed_record)
            });
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
    rom: &Path,
    native: &Path,
    tape: &InputTape,
    tape_hash: &str,
) -> Result<(ProductionSeed, ProductionContent), Outcome> {
    let image = std::fs::read(rom).map_err(|_| data_failure(None, "opening Ceres ROM"))?;
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
    let bytes = std::fs::read(native.join(MEMBER))
        .map_err(|_| data_failure(None, "opening Ceres audio origin file"))?;
    let packet = Packet::load(&bytes, tape_hash)?;
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
