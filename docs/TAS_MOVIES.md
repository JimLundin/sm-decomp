# Super Metroid TAS movie sources

Checked on 2026-10-09 against TASVideos' official
[Super Metroid game page](https://tasvideos.org/121G), publication pages,
submission notes, and the downloaded emulator replay archives.
These are controller recordings for testing the recompilation loop.

## Initial corpus

The current baseline (any%), low%, and 100% publications all use the same
Japan/USA ROM as this repository. TASVideos credits Sniq as the author of all
three and lists lsnes rr2-β23 as their recording emulator.

| Category | Publication / submission | Published | Frames | Local replay |
|---|---|---|---:|---|
| Baseline / any%, avoiding major skips | [3653M](https://tasvideos.org/3653M) / [5833S](https://tasvideos.org/5833S) | 2018-03-20 | 129,712 | `movies/super-metroid-any-3653.lsmv` |
| Low%, 13% items | [3273M](https://tasvideos.org/3273M) / [5291S](https://tasvideos.org/5291S) | 2016-12-02 | 167,797 | `movies/super-metroid-low-3273.lsmv` |
| 100% items | [4010M](https://tasvideos.org/4010M) / [6381S](https://tasvideos.org/6381S) | 2019-08-01 | 222,788 | `movies/super-metroid-100-4010.lsmv` |

The frame counts, emulator versions, authors, and submitted ROM names come
from the linked submission pages. The current category selection comes from
the game page; these are the published category leaders listed when checked,
not a claim about unpublished work or later revisions.

The game-end-glitch publication [5238M](https://tasvideos.org/5238M) uses the
European release, according to the game page. It is excluded from the initial
corpus because its ROM identity differs from `rom/sm.sfc`.

## Downloading and provenance

Use the publication pages above when sharing links or crediting these movies.
Their **Emulator Replay** download buttons point to the following machine
download handlers, which the repository downloader uses:

| Publication | Download handler |
|---|---|
| 3653M | `https://tasvideos.org/3653M?handler=Download` |
| 3273M | `https://tasvideos.org/3273M?handler=Download` |
| 4010M | `https://tasvideos.org/4010M?handler=Download` |

The downloads are outer ZIP archives containing `.lsmv` files; the downloader
extracts the replay into `movies/`. These `.lsmv` files are themselves ZIP
archives in lsnes' text movie format. Originals stay local and gitignored;
the manifest records provenance and checksums. The published author credits
are authoritative even though the any% and 100% archives have empty `authors`
members.

## ROM and initialization

The official [game version table](https://tasvideos.org/121G) identifies the
Japan/USA image as `Super Metroid (JU) (M2) [!].smc`, with these hashes:

- SHA-1: `da957f0d63d14cb441d215462904c4fa8519c613`
- MD5: `21f3e98df4780ee1c667b84e57d88675`

All three submission pages call the submitted image
`Super Metroid (Japan, USA) (En,Ja).sfc JPN/USA`. The first SHA-1 matches the
ROM identity enforced in [SETUP.md](SETUP.md).

Inspection of all three downloaded replay archives found:

| Member | Value |
|---|---|
| `coreversion` | `bsnes v085 (Compatibility core)` |
| `gametype` | `snes_ntsc` |
| `systemid` | `lsnes-rr1` |
| `rom.sha256` | `12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72` |
| `starttime.second` | `1000000000` |
| `starttime.subsecond` | `0` |

None contains `savestate`, `savestate.anchor`, `moviesram.*`, or
`ramcontent.*`. Under the official
[LSMV format rules](https://tasvideos.org/EmulatorResources/Lsnes/LSMV), absence
of the first three means a clean-state recording. The submission pages have
the legacy metadata value **Start Type: Unknown**; the clean-state conclusion
comes from archive inspection and the format rules, not that metadata field.

Clean state still depends on the emulator's initialization behavior. Recording
on bsnes does not prove that the same inputs synchronize on RustySNES or
MesenCE. Preserve the original archives and check the first divergence before
using a long movie as evidence of correct recompilation.

## Import and synchronization limits

The importer accepts these clean-start `.lsmv` movies, BizHawk `.bk2`,
extracted BizHawk input logs, and plain `.inputs`. `just import-movies`
verified all three archives and converted exactly 129,712, 167,797, and
222,788 frame records. Direct `.lsmv` replay checks the ROM SHA-256;
converted `.inputs` files carry no ROM identity.

The [official LSMV specification](https://tasvideos.org/EmulatorResources/Lsnes/LSMV)
defines `input` as one **subframe** per line. New-frame markers, resets,
controller ports, and button order must be decoded explicitly. For the SNES
gamepad, the documented order is `BYsSudlrAXLR`; `s` is Select and `S` is
Start. All three downloaded movies currently have one `F`-marked line per
frame and no continuation subframes; their line counts match the submission
frame counts. That property should be validated when importing rather than
assumed for every `.lsmv` file.

The importer checks that property and rejects reset, continuation subframe,
extra controller, and customized initialization records. It does not convert
them into apparently equivalent frame inputs.

## Replay observations

On 2026-10-09, the any% movie was replayed from power-on on the pinned
RustySNES core and MesenCE 2.2.1. Both reached these sampled milestones:

| Printed frame | Game state | Room |
|---:|---|---|
| 600 through 7,800, sampled every 600 | `$001E`, intro cinematic | `$0000` |
| 8,400 | `$001F`, setup new game | `$DF45`, Ceres elevator |
| 9,000 | `$0008`, main gameplay | `$DF45`, Ceres elevator |

Mesen initially had both controller ports set to `None`, which silently
ignored input injection. `just mesen-replay` now explicitly sets port 1 to
`SnesController`, disables port 2, and checks injection with `emu.getInput`.
Its first end-frame event occurs before its first controller poll, so input
records are indexed by poll rather than by the callback's frame counter.
Mesen's printed boundary is not identical to RustySNES's; the table confirms
coarse game progression, not exact frame alignment or matching WRAM hashes.

The fallback-only subject matched the reference's full 128 KiB WRAM for the
any% first 9,000 frames through entry into Ceres gameplay, and for the low%
and 100% first 600 frames each.
This checks the runtime bus adapter against the same emulator backend;
agreement cannot establish the backend's accuracy. Prefix call targets and
M/X variants were collected for discovery. Generated/interpreted execution
also matched CPU/clocks, WRAM, and framebuffer through 9,000 any% frames,
executing 25,359 generated instructions; low% and 100% each matched 600.

The original **lsnes rr2-β23 / bsnes v085 Compatibility** core is built locally
with `just setup-lsnes`. Its independent replay exposed three issues: different
checkpoint frame boundaries, missing HDMA channel overhead, and APU phase drift
during the Ceres upload. The local hardware patches are maintained
with checksums and ROM-free regression tests. All 10,472 initialized checkpoints
through frame 12,000 pass with the lsnes APU rate profile; full any% validation
is running.

`just sync-check` captures both cores through 12,000 and compares each visit to
`RunOneFrameOfGame` at `$82:8948`. This strict gate preserves duplicate visits
and checks frame, room, position, health, items, and in-game clock without
resynchronization. Full movie completion remains unverified. Build details,
provenance, and diagnosis are in [LSNES_BASELINE.md](LSNES_BASELINE.md).
