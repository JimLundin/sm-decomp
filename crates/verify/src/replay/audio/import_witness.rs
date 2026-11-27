use super::{group, load};
use crate::lock::Sha256;
use crate::replay::index::{Index, Start};
use data::assets::audio::ResidentAudio;
use data::assets::world::rooms::{
    CERES_ELEVATOR_ROOM, DEAD_SCIENTIST_ROOM, MAGNET_STAIRS_ROOM, MORPH_BALL_ROOM,
};
use data::rom::Rom;
use import::audio::{ImportError, resident_ceres};
use std::path::PathBuf;

/// Dense timbre order, then each instrument's selected sample's geometry and stream identity.
fn resource_receipt(bank: &ResidentAudio) -> Sha256 {
    let mut bytes = Vec::new();
    for instrument in &bank.instruments {
        let sample = bank.samples.get(usize::from(instrument.sample.0)).unwrap();
        bytes.extend(instrument.sample.0.to_le_bytes());
        let envelope = instrument.envelope;
        bytes.extend([envelope.attack_decay, envelope.sustain, envelope.gain]);
        bytes.extend(instrument.pitch_scale.to_le_bytes());
        bytes.extend(sample.frames.to_le_bytes());
        bytes.extend(sample.loop_from.unwrap_or(u32::MAX).to_le_bytes());
        bytes.extend(sample.source_sha256);
    }
    Sha256::of(&bytes)
}

#[test]
#[ignore = "requires the approved ROM and pinned native observations"]
fn opening_ceres_rom_content_matches_resident_observations() {
    let rom = PathBuf::from(std::env::var_os("SM_ROM").unwrap());
    let rom = Rom::new(std::fs::read(rom).unwrap()).unwrap();
    let imported = resident_ceres(&rom, MAGNET_STAIRS_ROOM).unwrap();
    for room in [CERES_ELEVATOR_ROOM, DEAD_SCIENTIST_ROOM] {
        assert!(
            resident_ceres(&rom, room).unwrap() == imported,
            "carried room music"
        );
    }
    assert_eq!(
        resident_ceres(&rom, MORPH_BALL_ROOM).err(),
        Some(ImportError::RoomSelection(MORPH_BALL_ROOM))
    );

    let index = Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
    let case = index
        .cases
        .iter()
        .find(|case| case.id.0 == "audio/native-control")
        .unwrap();
    let Start::NativeSeed { path, sha256 } = &case.start else {
        panic!("native control seed");
    };
    let native = PathBuf::from(std::env::var_os("SM_AUDIO_NATIVE").unwrap());
    let packet = case.audio_packet().unwrap();
    let (observed, _) = load(&native.join(path), sha256, case.end.cut_inclusive, packet).unwrap();
    assert!(
        imported.content.tracks == observed.content.tracks,
        "song graph"
    );
    assert!(
        imported.content.effects == observed.content.effects,
        "cue graphs"
    );
    assert!(
        imported.music_reply == observed.seed.resident_music_reply,
        "music reply"
    );
    assert_eq!(imported.echo_delay, observed.seed.song.mix.echo_delay);
    assert_eq!(imported.aliases.len(), observed.aliases.len());
    for (actual, expected) in imported.aliases.iter().zip(&observed.aliases) {
        assert_eq!(
            (actual.group, actual.byte, actual.cue),
            (group(expected.group).unwrap(), expected.byte, expected.cue)
        );
    }

    // Independently observed in the locked room-entry seed, with dense timbres assigned in
    // the native graph's order. Includes the identity of every addressed sample stream.
    assert_eq!(imported.instruments.len(), packet.instruments);
    assert_eq!(imported.samples.len(), packet.samples);
    assert_eq!(
        resource_receipt(&imported),
        Sha256::try_from(packet.resources_sha256.as_str()).unwrap()
    );
}
