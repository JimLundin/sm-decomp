//! Development checks for the independent current appearance owner seed.

#[path = "../src/replay/frame_appearance_seed.rs"]
mod frame_appearance_seed;

use data::pose::animation::{AnimationDelays, RunPaces};
use data::pose::{PoseId, Poses};
use data::rom::Rom;
use frame_appearance_seed::{SeedFault, decode};
use game::samus_appearance_bridge::{Clips, figure_of};
use samus_appearance::Step;

fn replace_word(wram: &mut [u8], at: usize, value: u16) -> Result<(), SeedFault> {
    wram.get_mut(at..at.saturating_add(2))
        .ok_or(SeedFault::Truncated)?
        .copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[test]
#[ignore = "requires independent first-cut WRAM and player ROM"]
fn current_figure_clip_relation_and_invalid_quantity_rejection() {
    let path = std::env::var_os("SM_FRAME_APPEARANCE_WRAM").expect("independent first-cut WRAM");
    let wram = std::fs::read(path).unwrap();
    let path = std::env::var_os("SM_ROM").expect("player ROM");
    let rom = Rom::new(std::fs::read(path).unwrap()).unwrap();
    let poses = Poses::load(rom.lorom()).unwrap();
    let delays = AnimationDelays::load(rom.lorom(), &poses).unwrap();
    let paces = RunPaces::load(rom.lorom()).unwrap();
    let clips = Clips::import(&delays, &paces).unwrap();
    let seed = decode(&wram, &clips).unwrap();
    let mut unrelated = wram.clone();
    replace_word(&mut unrelated, 0x0af6, 123).unwrap();
    assert_eq!(decode(&unrelated, &clips), Ok(seed));
    for (at, value, fault) in [
        (0x0a1c, u16::MAX, SeedFault::Figure),
        (0x0a96, u16::MAX, SeedFault::Step),
        (0x0a94, 256, SeedFault::Timer),
        (0x0b3e, 0xff00, SeedFault::BoostStage),
    ] {
        let mut invalid = wram.clone();
        replace_word(&mut invalid, at, value).unwrap();
        assert_eq!(decode(&invalid, &clips), Err(fault));
    }
    assert_eq!(decode(&[], &clips), Err(SeedFault::Truncated));
    let running = clips
        .poses()
        .iter()
        .find(|entry| entry.figure.pose.action == types::Action::Running)
        .unwrap();
    let mut running_seed = wram;
    replace_word(&mut running_seed, 0x0a1c, running.id.0).unwrap();
    replace_word(
        &mut running_seed,
        0x0a96,
        running.frame_of(Step::FIRST).unwrap().0,
    )
    .unwrap();
    let actual = decode(&running_seed, &clips).unwrap();
    assert_eq!(actual.playing, figure_of(PoseId(running.id.0)).unwrap());
    assert_eq!(actual.state.playhead.step, Step::FIRST);
}
