# Landing Site receiving cut, decode and replay evidence

Companion to the [receiving contract](landing-site-prefix.md). Observer and integrator material.

## Coordinates and authority

The authority is movie frame **15,196**, cut 0 of `any-zebes-arrival`. The previous
cut is movie 15,195, cut 2,123 of `chain-any-0011-DF45`. The cache's emulator video
frames are 15,197 and 15,196 respectively. Treat the row's cut 0 as a continuation
observation from the running flight in integrated replay. A scoped lane can explicitly
admit the earlier departure state and receive the same platform input.

The receiving cut is `VblankWithoutNmi`. Its CPU address is $82:8327, the next
instruction address while the second direct graphics DMA still owns the bus. The next
instruction has not reset the tilemap VRAM address. A finished-frame boolean by itself
loses the distinction between opening the load, continuing it and publishing NMI.

## Owners and source joins

| Responsibility | Source evidence and available production path |
| --- | --- |
| LandingWorkOpens | $82:8948..896B runs common work setup and $894F draws RNG before dispatch. The receiving cut advances the existing `game_random::advance_random` rule once. The completed count at $05B6 holds. The previous flight cut has already set loading flow $0998 and full health. The current flight recipe returns `LandingSiteLoader` before inspecting cadence when the request is pending. T1 needs a distinct work opening and progress receipt. A lack of completion is insufficient to advance or hold work. |
| Destination and explored map | $82:801D..804E selects the Landing Site branch from $7E:D914. $079F becomes Crateria and $078B becomes its cutscene entrance. $80:858C..85B5 copies stored Crateria bytes $7E:CD52..CE51 to $07F7..08F6 and the saved map-availability byte $7E:D908 to $0789. $80:C437 runs later. Use the typed destination already supplied by the flight. The reusable load-station table importer currently selects Ceres's first index. Importing the distinct Landing Site entrance for later room placement is separate work. HUD `NewArea::Entered` can receive the current explored bitmap. `data::map::MapCells` provides its bitmap projection. `run_progress::Progress::maps` tracks status; per-area visited cells need their own carried data. |
| GameplayDisplaySetup | $82:819B..81DC sets controller auto-read, disables NMI/IRQ and HDMA, zeroes IRQ timers and sets fast ROM. Verify mirrors $84..86 because lsnes's PPU snapshot omits those CPU fields. $82:81DD..8299 writes gameplay settings to the PPU and its mirrors, writes zero to the first two OAM bytes at the object table's start, and resets the WRAM object index $0590. `Video` separates live memory, current registers and the next NMI batch. Use direct writes to live state and carry the last publication. A shadow-only write or a whole OAM clear misstates the cut. |
| Tilemap preparation | $82:829C..82C4 uses $80:83F6..8408 to fill 1,023 words in the BG2 shadow at $3000 with zero and 1,023 words in the BG1 shadow at $4000 with $006F. It then overwrites BG1's first 127 words with $2C0F. The two boundary words at $37FE and $47FE remain inherited. The first BG1 tilemap VRAM upload starts only after this cut. Prepared buffers, or their equivalent typed authoring data and inherited tails, are later transfer inputs. |
| StandardGraphics | $82:82E2..8322 and $80:91A9 configure and use channel 1, increasing source addresses within the bank and alternating the two VRAM byte ports, incrementing the word after the high byte. The standard BG3 asset is $9A:B200 length $2000, VRAM word base $4000. The standard sprite asset starts at $9A:D200, transfer length $4000, VRAM word base $6000. Neither standard asset has an import surface on this base. Existing `exact::tilemap::direct` writes complete words and retires the channel at once. Reuse the byte/word storage and typed register data, with a resumable direct transfer path for this prefix. |
| Deferred work | $82:8325 onward uploads the two tilemap spans. $82:8054 onward initializes the palette, Samus and the room. $82:808C re-enables NMI later. The existing `room_loading::load` and station/room geometry formats are useful at that later phase. `screens-arrival` begins the Ceres arrival and owns its cutscene, not this transfer. `game_time` is the gameplay clock, not T1's load pacing. |

At this cut the display is forced blank. The source sets gameplay object sizes 8×8/16×16,
base $6000, BG1/2 graphics base $0000, BG3 graphics base $4000, BG1 tilemap $5000
64×32, BG2 $4800 64×32, and BG3 $5800 32×64. BG4 keeps no visible layer. There is no
window or colour math, and fixed colour is black. Compare the matching encoded
registers and mirrors. Preserve actual scroll and affine register contents from the
flight, as well as its cleared affine pose/matrix mirrors. The snapshot covers no
NMI commit. It cannot prove sub-byte latches, CPU scratch, open bus or displayed pixels
beyond forced blank.

## Bytes, progress and restoration

The [pinned emulator's DMA pipeline source](https://raw.githubusercontent.com/higan-emu/higan/v085/bsnes/snes/cpu/dma/dma.cpp)
orders address selection, acceptance and bus write. Selecting increments the source
register before the read. Accepting a byte pushes it into the pipeline, commits the
previous pending write, and decrements count after the transfer step. Draining a
completed channel writes its final pending byte. This explains the three different
offsets at the receiving cut. Channel 1 has selected the read at sprite byte offset
3,246, so its source register has advanced 3,247. The remaining count shows 3,246
accepted bytes. The first **3,245 bytes** are committed to VRAM. The pending byte is
offset 3,245, the high byte of the partially written word. The carried high byte
remains in VRAM until that pending write commits. The record does not expose the
selected read's sub-byte clock position or latch internals. Use accepted/selected
observations as a scoped T1 input, or let the live scheduler produce the same phase.
A scoped case must declare this borrowing and must not claim movie-only T1 timing.

Verify full VRAM after that write against the receiving cut. Build expectations from
imported assets plus the prior state; never copy the receiving image into the model.
The unmodified ranges include both standard tilemap destinations, every sprite byte
from the held high byte onward and all earlier unrelated bytes. A byte being identical
in asset and previous VRAM is not evidence that it has transferred. In this witness,
6,619 bytes differ across BG3 and the sprite prefix, but the complete transfer extent
is larger. The held model misses those differences.

The joined departure comparison reads the sprite destination before the transfer, at the
receiving cut and at the next native cut. The last changed byte in the first interval lies
immediately before the first changed byte in the second. This fixes the committed length
and identifies the pending byte by the value subsequently written at that location. The
successor supplies expectation data only. The source address register records read selection.

For a restoration check, save during the split word and advance the original state
and its restored copy under the same following transfer opportunities. Compare
pending identity, committed count, byte parity, retained bytes and phase before and
after. Keep exact T1 phase separately if later placement depends on elapsed clocks.
The admitted continuation operates inside the immutable ROM portion. The source
bank's offset wraps after $2E00 sprite bytes and the remaining $1200 reads mapped low
WRAM in the same bank. Those later reads need the process's live carried bytes at
their own opportunities, including work since the preceding cut. Neither a linear
ROM continuation nor a snapshot transplanted from a later arrival seed is valid.

## Earlier data and replay admission

The scoped proof uses shaft cut 1,004 (movie 14,076) for inherited VRAM and the shared
random sequence. Apply the existing imported rear ship map to its low-byte lane, the
flat page and the flight tile/sprite page from $8B:C6D1..C732. This reconstructs all
65,536 bytes at 2,123. The high bytes of the affine map region and all gaps are
inherited. In an uninterrupted run the preceding explosion and flight supply this
state directly; the receiving verifier supplies no next-state VRAM.

The first work opening in the flight setup occurs on overrun cut 1,005. The RNG draws
there, holds through cut 1,014 when that work finishes, and draws on each subsequent
work opening through 2,123. The loader draws once on 15,196. The stored Crateria
explored bitmap and map flag already come from shaft cut 0, movie 13,072, and are
clear in this witness. The outgoing active bitmap has explored Ceres cells.
Constructed nonempty maps distinguish copying the selected stored map from guessing
zero or copying another area's storage. The scoped proof uses carried departure
palette, OAM, room/door/placement, energy and completed count. Integrating those
preservation checks requires the evolving production owners. Keep first-OAM-pair
direct writes separate from the earlier mask for off-screen flight OAM.

The receiving comparison can grow at this cut with precisely the stated transfer
timing borrowing. Movie-only timing and continuation after source wrap, tilemap
uploads, room initialization, palette initialization and landing play are separate
obligations. A mask of the partial sprite range reduces the claim and must be named.

## Witnesses

- Recorded pair: 15,195 requests the load and 15,196 dispatches it while unfinished.
  Compare work counters, full energy, room, entrance, map, PPU registers and mirrors,
  CGRAM, initial low OAM pair, prepared tilemaps, DMA cursor and full VRAM.
- `L-open` and `L-no-finish` distinguish pending load dispatch from a whole-frame
  hold and a completed game step. `L-no-work` and `L-other-target` reject early and
  wrong-target dispatch. `L-map` uses a different remembered bitmap and flag.
- `L-display` exposes a whole OAM clear or a palette reset. `L-split` exposes an
  eager flush, an even-byte rounding or an early tilemap write. `L-resume`,
  `L-reopen` and `L-drain` check the same transfer across save, restart attempt and
  final pending drain using invented graphics.

The reference reconstructs all 65,536 carried VRAM bytes from the earlier seed and
assets and matches the RNG on all 1,120 flight cuts from that same seed. It matches
24/24 receiving and restoration check groups and all 10 invented examples. An early
drain or early tilemap write fails full VRAM at 15,196. Retaining the old map, changing
the finished count or replacing the room each fails that field on the same cut. A
missing pending byte fails the split-word restoration example. Premature dispatch and
restarting an open upload fail their own negative examples. The scoped continuation
uses two additional accepted reads from the same asset on both the running and the
restored State. It compares the resulting prefix, untouched suffix and complete State.
