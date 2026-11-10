//! The harness cases (MODEL §1 decision 13): recorded frames whose rule reads a game clip, which
//! the spec's vectors replace with invented ones.
//!
//! Each case is the decode notes' ("The harness cases"): the frame from record N, a quiescent
//! cut, to the next one, over the importer's clips and paces. It must be compared, match on every
//! unmasked field, and bring its entry into play. The constructed vectors carry the same rules
//! over invented clips.

use clean_model_samus_looks::BehaviourId;

/// A recorded frame that reads a game clip.
pub struct Case {
    /// The entry it brings into play.
    pub entry: BehaviourId,
    /// The row.
    pub row: &'static str,
    /// The record the frame starts at (N).
    pub record: u64,
}

const fn case(entry: BehaviourId, row: &'static str, record: u64) -> Case {
    Case { entry, row, record }
}

/// The decode notes' harness cases, by entry.
pub const CASES: [Case; 39] = {
    use BehaviourId as B;
    [
        case(B::FastFall, "chain-0232-D5A7", 224),
        case(B::FastFall, "chain-0232-D5A7", 230),
        case(B::FastFall, "samus-grapple-release", 254),
        case(B::FastFall, "samus-shinespark-up-left", 394),
        case(B::BoostingLap, "boss-phantoon", 187),
        case(B::BoostingLap, "samus-shinespark-left", 50),
        case(B::BoostingLap, "samus-shinespark-right", 50),
        case(B::BoostingLap, "samus-shinespark-up", 50),
        case(B::BoostingLap, "samus-shinespark-up-left", 50),
        case(B::WallJumpBranch, "samus-screw-attack", 73),
        case(B::WallJumpBranch, "samus-walljump-16", 57),
        case(B::WallJumpBranch, "samus-walljump-22", 63),
        case(B::WallJumpBranch, "samus-walljump-28", 69),
        case(B::WallJumpBranch, "chain-0043-9AD9", 61),
        case(B::WallJumpBranch, "chain-0043-9AD9", 192),
        case(B::WallJumpBranch, "chain-0043-9AD9", 205),
        case(B::WallJumpReady, "boss-phantoon", 3),
        case(B::WallJumpReady, "chain-0014-96BA", 50),
        case(B::WallJumpReady, "chain-0043-9AD9", 22),
        case(B::WallJumpReady, "chain-0043-9AD9", 36),
        case(B::ClipEnds, "samus-screw-attack", 243),
        case(B::ClipEnds, "samus-screw-attack", 315),
        case(B::ClipControls, "chain-0057-9FBA", 232),
        case(B::ClipControls, "chain-0057-9FBA", 241),
        case(B::Breathing, "area-maridia", 247),
        case(B::Breathing, "area-maridia", 337),
        case(B::StartingClip, "boss-phantoon", 2),
        case(B::StartingClip, "boss-phantoon", 61),
        case(B::StartingClip, "chain-0014-96BA", 20),
        case(B::BoostStages, "area-maridia", 13),
        case(B::BoostStages, "boss-phantoon", 127),
        case(B::BoostStages, "boss-phantoon", 152),
        case(B::BoostStages, "boss-phantoon", 172),
        case(B::BoostStages, "samus-shinespark-left", 72),
        case(B::BoostStages, "samus-shinespark-left", 93),
        case(B::RisingHold, "boss-kraid-defeated", 297),
        case(B::RisingHold, "boss-kraid-defeated", 315),
        case(B::RunPace, "boss-kraid-defeated", 988),
        case(B::RunPace, "chain-0012-91F8", 638),
    ]
};
