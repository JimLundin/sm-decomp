//! Semantic opening-room input joined to the independently pinned resident transport tape.

use super::{InputTape, group};
use crate::lock::{Lock, RowId, Sha256};
use crate::replay::{Outcome, data_failure};
use audio_transport::{
    AudioCyclePhase, AudioInstant, ReleaseHistory, Request, SenderState, SoundGroup,
};
use data::assets::world::rooms::MAGNET_STAIRS_ROOM;
use game::production::{ProductionContent, ProductionSeed, Senders, audio};
use serde::Deserialize;
use std::path::Path;

const MEMBER: &str = "opening-ceres-audio-v1/origin.toml";
const PIN: &str = "d9605917649a048cec3e66f1dd5debb703849b8095b54776d31fb88f87b94ed2";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Groups<T> {
    action: T,
    ambient: T,
    contact: T,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    schema: String,
    room: String,
    locked_seed_sha256: String,
    control_sha256: String,
    expiry_sha256: String,
    cpu_hz: u64,
    oscillator_hz: u64,
    loaded_smp_offset: i64,
    first_poll_num: u64,
    first_poll_den: u64,
    timer_phase: u64,
    timer_unread: u8,
    requests: [Request; 3],
    history: Groups<ReleaseHistory>,
    sender: Groups<SenderState>,
}

impl Packet {
    fn load(bytes: &[u8], current_tape: &str) -> Result<(audio::Origin, audio::Load), Outcome> {
        if Sha256::try_from(PIN).ok() != Some(Sha256::of(bytes)) {
            return Err(data_failure(None, "opening Ceres audio origin pin"));
        }
        let source = std::str::from_utf8(bytes)
            .map_err(|_| data_failure(None, "opening Ceres audio origin encoding"))?;
        let packet: Self = toml::from_str(source)
            .map_err(|_| data_failure(None, "opening Ceres audio origin fields"))?;
        packet.admit(current_tape)
    }

    fn admit(self, current_tape: &str) -> Result<(audio::Origin, audio::Load), Outcome> {
        let locked = Lock::parse(include_str!("../../../../../scenarios/locks/lsnes.toml"))
            .ok()
            .and_then(|lock| {
                lock.row(&RowId("chain-any-0003-DFD7".to_owned()))
                    .and_then(|row| row.seed_record)
            });
        let seed_identity = Sha256::try_from(self.locked_seed_sha256.as_str())
            .map_err(|_| data_failure(None, "opening Ceres locked seed identity"))?;
        if self.schema != "opening-ceres-audio-v1"
            || self.room != "magnet-stairs"
            || Some(seed_identity) != locked
            || (self.control_sha256 != current_tape && self.expiry_sha256 != current_tape)
        {
            return Err(data_failure(None, "opening Ceres audio origin lineage"));
        }

        let first_poll = AudioInstant::new(self.first_poll_num.into(), self.first_poll_den.into())
            .map_err(|_| data_failure(None, "opening Ceres first poll fraction"))?;
        let clock =
            audio::ClockRelation::new(self.cpu_hz, self.oscillator_hz, self.loaded_smp_offset);
        let origin = audio::Origin::opening_ceres(seed_identity.0, clock, &first_poll)
            .map_err(|_| data_failure(None, "opening Ceres audio clock relation"))?;
        let Groups {
            action,
            ambient,
            contact,
        } = self.sender;
        let senders = Senders::new(action, ambient, contact)
            .map_err(|_| data_failure(None, "opening Ceres sender groups"))?;
        let [action, ambient, contact] = self.requests;
        let requests = enum_map::EnumMap::from_fn(|slot| match slot {
            SoundGroup::Action => action,
            SoundGroup::Ambient => ambient,
            SoundGroup::Contact => contact,
        });
        let mut history = enum_map::EnumMap::default();
        history[SoundGroup::Action] = self.history.action;
        history[SoundGroup::Ambient] = self.history.ambient;
        history[SoundGroup::Contact] = self.history.contact;
        let timer_phase =
            AudioCyclePhase::try_from(audio_transport::time::cycles(self.timer_phase))
                .map_err(|_| data_failure(None, "opening Ceres prescaler"))?;
        Ok((
            origin,
            audio::Load {
                senders,
                requests,
                history,
                timer_phase,
                timer_unread: self.timer_unread,
            },
        ))
    }
}

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
    let (origin, load) = Packet::load(&bytes, tape_hash)?;
    let content = audio::Content::new(bank, origin)
        .map_err(|_| data_failure(None, "resident audio bank admission"))?;
    let loaded = audio::State::opening_ceres(Box::new(tape.seed.clone()), load, &content)
        .map_err(|_| data_failure(None, "locked resident audio state"))?;
    Ok((
        ProductionSeed::ResidentAudio(Box::new(loaded)),
        ProductionContent::ResidentAudio(Box::new(content)),
    ))
}

#[cfg(test)]
mod tests;
