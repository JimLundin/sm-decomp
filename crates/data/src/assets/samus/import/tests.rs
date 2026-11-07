use super::map::{MAP, id_of, key_of};
use super::*;
use crate::assets::samus::ClipEnd;
use crate::assets::samus::key::{Dress, SpinSprite, Suit};
use crate::rom::Rom;

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

/// Every dress she can wear.
fn dresses() -> Vec<Dress> {
    [
        SpinSprite::Plain,
        SpinSprite::SpaceJump,
        SpinSprite::ScrewAttack,
    ]
    .into_iter()
    .flat_map(|spin| {
        [Suit::First, Suit::VariaOrGravity]
            .into_iter()
            .map(move |suit| Dress { spin, suit })
    })
    .collect()
}

#[test]
fn every_pose_of_hers_is_one_id_and_back_in_its_dress() {
    for entry in MAP {
        assert_eq!(key_of(entry.id), Some(entry.key), "{:?}", entry.id);
        for dress in dresses() {
            let id = id_of(entry.key, dress);
            assert!(id.is_some(), "{:?} in {dress:?}", entry.id);
            if entry.worn.holds(dress) {
                assert_eq!(id, Some(entry.id), "{dress:?}");
            }
        }
    }
    assert_eq!(key_of(PoseId(0xFD)), None);
}

#[test]
fn every_pose_of_hers_imports_with_its_frames() {
    let Some(cartridge) = rom() else {
        return;
    };
    let assets = import(cartridge.lorom()).unwrap();
    assert_eq!(assets.clips.len(), MAP.len());
    for (asset, entry) in assets.clips.iter().zip(MAP) {
        assert_eq!(asset.key, entry.key);
        assert!(!asset.frames.is_empty(), "{:?}", entry.id);
        assert!(
            asset.frames.iter().any(|frame| frame.top.is_some()),
            "{:?}",
            entry.id
        );
    }
    assert!(matches!(assets.running.first(), Some(Hold::Shows(_))));
    // A sprite list or an upload the frames share is kept once.
    let frames: usize = assets.clips.iter().map(|asset| asset.frames.len()).sum();
    assert!(frames > assets.sheet.sprites.len());
    assert!(frames > assets.sheet.uploads.len());
    // A loop plays on from one of its frames; a landing hands over to standing.
    for clip in &assets.clips {
        if let ClipEnd::Loop { from } = clip.end {
            assert!(usize::from(from.0) < clip.frames.len(), "{:?}", clip.key);
        }
    }
    let landing = MAP
        .iter()
        .position(|entry| entry.id == PoseId(0xA4))
        .unwrap();
    assert_eq!(assets.clips[landing].end, ClipEnd::HandOver);
}
