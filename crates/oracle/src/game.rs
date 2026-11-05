//! What the harness knows of the game: only where its main thread waits for
//! NMI, so an NMI elsewhere can be flagged as an overrun. Bank logs:
//! https://patrickjohnston.org/bank/80#f8338 and /bank/85#f8136.

/// A loop in which the main thread waits for the next NMI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaitSite {
    /// The routine, as the bank logs name it.
    pub routine: &'static str,
    /// The routine's first instruction: the main thread has finished its frame.
    pub entry: u32,
    /// The instructions of the loop: an NMI interrupts one of them.
    pub loop_pcs: &'static [u32],
}

/// The game's wait-for-NMI loops.
pub const WAIT_SITES: [WaitSite; 3] = [
    WaitSite {
        routine: "$80:8338 Wait for NMI (LDA $05B4; BNE)",
        entry: 0x80_8338,
        loop_pcs: &[0x80_8343, 0x80_8346],
    },
    WaitSite {
        routine: "$85:8136 Wait for lag frame (LDA $05B8 compare loop)",
        entry: 0x85_8136,
        loop_pcs: &[0x85_813c, 0x85_813f],
    },
    WaitSite {
        routine: "$85:84A3 Message box input loop (spins on $4212 and the joypad)",
        entry: 0x85_84a3,
        loop_pcs: &[
            0x85_84a3, 0x85_84a6, 0x85_84a8, 0x85_84aa, 0x85_84ad, 0x85_84af, 0x85_84b2,
        ],
    },
];

/// Whether an NMI that interrupted `pc` found the main thread waiting.
pub fn is_wait_site(pc: u32) -> bool {
    WAIT_SITES.iter().any(|site| site.loop_pcs.contains(&pc))
}
