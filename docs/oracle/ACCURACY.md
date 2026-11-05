# Emulator accuracy, as it bears on Super Metroid

Research note for the oracle lane (2026-11-05). It asks which emulator to
believe where our two oracles disagree on Super Metroid: LakeSnes `9db90b8`
with fix 0001, and lsnes rr2-beta23, which runs the bsnes v085 compatibility
core. The disagreements are in [MEASUREMENTS.md (f)](MEASUREMENTS.md) and
the movie sync table in [HARNESS.md](HARNESS.md#movie-sync). General accuracy
rankings appear only where they touch something SM does.

Labels: **fact** means the source code or a hardware measurement shows it.
**Report** means a named person says so in a TASVideos or forum post.
**Inference** is our reasoning from those, not yet measured.

## 1. What SM exercises

SM's timing-sensitive paths are few. All addresses are from the
[bank $80 log](https://patrickjohnston.org/bank/80).

- **APU upload by handshake.** `$80:8059` sends every byte as: write data
  and index to the ports, then spin until the SPC echoes the index. The wait
  for `AA BB` gives up after `3000h` polls (counter `$0641`, our steady
  wram difference), then hangs at `$8077`. So the length of an upload is
  bytes × round trip, and the round trip depends on both clocks and on when
  a port write becomes visible to the other side. Uploads happen at boot, at
  the title and on music changes (`$80:8F7E`, followed by "sound handler
  downtime = 8").
- **Lag is NMI-gated.** The main loop sets `$05B4` and spins at
  `$8343/$8346`. The NMI (`$80:9583`) does its uploads and DMAs only if that
  flag is set, otherwise it takes the lag branch. A frame is lag when the
  game's work runs past the next vblank start. The handler has no vblank
  guard: the bank log notes that with force blank set, NMI work may spill
  into the next frame's drawing period (`$80:836F`).
- **Auto-joypad.** The NMI's last job, `$80:9459`, spins on `$4212` bit 0
  before it reads `$4218`. The bank log says this is because the auto-read is
  only guaranteed to happen somewhere in the first scanline of vblank. When
  the auto-read starts therefore cannot change which pad SM sees, only how
  many cycles the NMI spends waiting (**fact**, from the code).
- **Tricks that depend on the platform.** On the TASVideos
  [game resources page](https://tasvideos.org/GameResources/SNES/SuperMetroid),
  Zebes's explosion trigger "does not work on Snes9x", and the spacetime beam
  gives "results that differ on different emulators". The spacetime beam
  reads and writes out of range, so its result depends on open bus and
  power-on RAM (**report**; our movies don't use it).

## 2. The emulators

| Emulator | CPU/SMP timing | PPU | DSP | SPC clock (NTSC) | Note for SM |
| --- | --- | --- | --- | --- | --- |
| bsnes v085 **accuracy** | cycle-based, cothreads | dot-based (`snes/ppu`) | cycle-based (`snes/dsp`) | 24 607 104 Hz = 32040.5 × 768 | not used for our movies |
| bsnes v085 **compatibility** (lsnes) | the same `snes/cpu` and `snes/smp` | scanline-based (`alt/ppu-compatibility`) | blargg's `SPC_DSP` (`alt/dsp`) | the same | the movies were recorded on it |
| bsnes v115 / ares | cycle-based | dot / scanline options | cycle-based | 32040 × 768 | modern reference |
| Mesen 2 / MesenCE | cycle-based. A CPU port write is visible to the SPC at once if it lands in the first half of an SPC cycle, otherwise one cycle later | dot-based | cycle-based | 32000 + `SpcClockSpeedAdjustment`, **default +40**, settable from -999 to +999 | lets us sweep the clock rate |
| snes9x 1.6x | opcode-based | scanline | blargg | 32040 (15664/328125 per master clock) | more lag than lsnes in SM, see 4 |
| LakeSnes 9db90b8 | the README says SPC and CPU timing "should be cycle-accurate now", but "communication between the CPU and SPC is also not cycle-accurate" | scanline | per sample | 32040 × 32 / (1364·262·60): **0.17 % fast**. With fix 0001, 32040 | our fast oracle |

Sources for the table:

- bsnes v085 profiles: the profile selection in `snes/Makefile` and
  `snes/profile-*.hpp`, and the clocks in `snes/config/config.cpp`. We read
  the v086 tree in
  [Dgdiniz/bsnes_v086-source](https://github.com/Dgdiniz/bsnes_v086-source/blob/master/bsnes/snes/Makefile),
  one release after v085, whose profile layout is the same. The
  compatibility profile keeps the accuracy CPU and SMP, and swaps only the
  PPU and the DSP (**fact**).
- [ares `sfc/system/system.hpp`](https://github.com/ares-emulator/ares/blob/master/ares/sfc/system/system.hpp)
  and [bsnes `sfc/system/system.cpp`](https://github.com/bsnes-emu/bsnes/blob/master/bsnes/sfc/system/system.cpp).
  bsnes falls back to 32000 × 768 for Rendering Ranger R2 alone.
- Mesen: [`Spc.cpp` `CpuWriteRegister`](https://github.com/SourMesen/Mesen2/blob/master/Core/SNES/Spc.cpp)
  and [`SnesConfig.cs`](https://github.com/nesdev-org/MesenCE/blob/master/UI/Config/SnesConfig.cs).
  MesenCE is the maintained fork. A comment there says the port write delay
  is what makes both Zenki and Kawasaki Superbike boot "at the default
  32040hz".
- [snes9x `apu/apu.cpp`](https://github.com/snes9xgit/snes9x/blob/master/apu/apu.cpp)
  and the [LakeSnes README](https://github.com/angelo-wf/LakeSnes).
- LakeSnes `snes/snes.c` runs the APU in whole SPC opcodes, catching up on
  each port access (`snes_catchupApu`). It raises NMI and starts the
  auto-read at dot 0 of line 225 (the source carries a "TODO: this starts a
  little after start of vblank"). It does model DRAM refresh (40 clocks at
  dot 536), DMA overheads and the short line 240 (**fact**, from the source).

What the cores do differently, for SM:

- **Accuracy and compatibility in v085.** They run the same S-CPU, S-SMP,
  H/V counters (`snes/ppu/counter`), DMA and HDMA. They differ only in when
  a mid-line PPU write shows on screen, and in the DSP's sub-sample timing.
  SM's CPU never waits on the PPU's rendering, and nothing we know of in its
  sound driver reads the DSP's timing back. So for SM's CPU state and lag,
  the two profiles should agree (**inference**). This fits a
  [report](https://tasvideos.org/6381S): "All Super Metroid TASes created
  with Bizhawk and Lsnes are compatible with each other". BizHawk's old core
  is bsnes v087 and hard-codes the Compatibility profile
  ([`LibsnesCore.cs`](https://github.com/TASEmulators/BizHawk/blob/master/src/BizHawk.Emulation.Cores/Consoles/Nintendo/SNES/LibsnesCore.cs)),
  so that report tells us nothing about the accuracy profile.
- **Test suites.** The TASVideos
  [SNES accuracy tests](https://tasvideos.org/EmulatorResources/SNESAccuracyTests)
  (2014) score lsnes and higan v094 at 59 of 59, BizHawk at 55 and snes9x
  1.43 at 45. The one SM-relevant row is "Aging: APU", which fails on snes9x
  1.43 and 1.51. No test in that table checks the CPU-SPC handshake timing
  or a real console's clock rate, and those are where our oracles split.

## 3. The SPC700 clock is not one number

A ceramic resonator drives the APU, not the master crystal (**fact**). The
SNESdev [Timing](https://snes.nesdev.org/wiki/Timing#Audio) page gives:
24.576 MHz nominal, so 32000 Hz by specification; real consoles are
"usually ~0.25 % higher" and rise "~0.04 %" as they warm, with the DAC rate
anywhere from 32000 to 32160 Hz.

Measurements with lidnariq's `smpspeed.sfc`
([forum thread](https://forums.nesdev.org/viewtopic.php?t=24610)), in Hz:

| Console | Cold | After about 25 min |
| --- | --- | --- |
| SFC 2/1/3, 3-chip | 32147 | 32152 |
| SFC 1-chip | 32036 | 32043 |
| SFC 1/1/1 | 32080 | 32090 |
| SFC 1/1/1, broken PPU | 32067 | – |
| SNES PAL 1-chip (oscilloscope agrees: 32.08 → 32.09 kHz) | 32083 | 32091 |
| lidnariq's own | ~32030 | rises slowly |

Fiskbit (2025, same thread) puts the range seen in other tests at about
32010-32152 Hz, and notes that emulators use 32040 Hz. rasteri (2023, same
thread) replaced a console's clocks with a single 171.818 MHz source for TAS
replay. That is, TAS console verification treats the APU clock as something
to control, not something a console gets right (**report**).

**How the emulators chose.** They all pick about 32040 Hz, a typical
measured value rather than the 32000 Hz specification: bsnes v085 uses
32040.5, while ares, bsnes v115, snes9x, Mesen and LakeSnes after fix 0001
use 32040 (**fact**, the sources in 2). LakeSnes with fix 0002 uses
bsnes v085's 32040.5, exactly (MEASUREMENTS (h)). Fix 0001 brings LakeSnes from
1364·262·60 master clocks per second to 21 477 272, 0.17 % slower.
Measuring from that patch, the pre-fix LakeSnes was 0.17 % fast, not 5 %.

**What this means for the lag after an upload.** Take an upload of N frames
whose round trip is mostly the SPC's. A clock faster by δ ends it about
N·262·δ lines earlier.

| Pair | δ | Shift over a 35-frame upload (1307-1341) |
| --- | --- | --- |
| lsnes (32040.5) against LakeSnes + fix 0001 (32040) | 16 ppm | ~0.15 line |
| LakeSnes before fix 0001 | +0.17 % | ~15 lines early. The measured upload ending 12 lines early near frame 352 fits this. |
| Real consoles, 32030 to 32152 Hz | -0.03 % to +0.35 % | ~3 lines late to ~32 lines early |

Two conclusions follow (**inference**, from the arithmetic above):

1. **There is no single correct lag after an APU upload on real hardware.**
   On a console at 32150 Hz an upload ends a few dozen lines earlier than on
   lsnes. Wherever the end of an upload falls within that distance of a
   frame boundary, consoles differ in which frame is lag. The movies define
   the lag pattern as lsnes's 32040.5 Hz.
2. **The clock rate does not explain 1342.** Our two oracles differ by
   0.15 line there. Unless the margin at 1342 is under a line, the frame
   comes from how the CPU-SPC communication is modelled. LakeSnes catches up
   in whole opcodes and its README says that communication is not
   cycle-accurate. bsnes syncs the cothreads on every port access. Mesen
   latches port writes per half SPC cycle. Our own measurement fits this:
   lsnes is about 7 bytes ahead in X yet finishes a frame later, which is a
   per-byte latency difference, not a rate.

## 4. NMI, vblank and lag between emulators

- **NTSC frame** ([SNESdev Timing](https://snes.nesdev.org/wiki/Timing)):
  1364 master clocks per line; 262 lines; vblank from line 225 to 261, 37
  lines. The auto-read runs for about the first three lines of vblank. There
  is a short window after vblank begins before it starts
  ([Controller reading](https://snes.nesdev.org/wiki/Controller_reading)).
  LakeSnes starts NMI and the auto-read exactly at dot 0 of line 225. bsnes
  starts them a few dots later. SM waits on `$4212` either way (1), so this
  costs cycles only.
- **"The NMI outlasts vblank".** Through 4800 frames this was a difference
  in where the harness cuts, not in what the game did: lsnes cuts at its
  frame boundary with the handler still running; LakeSnes runs to the RTI
  ([MEASUREMENTS (f)](MEASUREMENTS.md)). On SM the handler running into
  active display is real behaviour and the game allows for it (1). Its
  length depends on DMA and HDMA overheads, DRAM refresh and FastROM, which
  LakeSnes models, so we have no evidence yet that the CPU side differs.
- **Lag between emulators for SM** (**report**). Sniq's low% submission
  ([5291S](https://tasvideos.org/5291S)) says that "due to using a
  more-accurate Lsnes emulator, the magnitude of lag and length of door
  transitions are inherently greater than before" than on snes9x 1.43, and
  that one room "had to be executed slightly differently". So SM's lag and
  door-transition timing are known to depend on the emulator; snes9x 1.43
  is the inaccurate one.

## 5. Console verification of SM movies

TASVideos lists one console-verified SM movie:
[6888M, "HISHE"](https://tasvideos.org/6888M), a short ROM hack, recorded on
Snes9x 1.51 and played on a console through a BizHawk (`.bk2`) resync
([verified list](https://tasvideos.org/Movies-Verified-SNES)) (**fact**).
None of our movies is verified: not any% [3653M](https://tasvideos.org/3653M),
low% [3273M](https://tasvideos.org/3273M) nor 100% `4010M`. Their pages and
submissions name lsnes rr2-β23 and say nothing about hardware. The
TASVideos [verification guide](https://tasvideos.org/ConsoleVerification/Guide)
names only BizHawk for SNES dumps. So no hardware record exists that says
which emulator matches a console on our movies.

## 6. Known SM emulation issues

- The SPC upload can hang the game: `$8077` hangs after 3000h failed polls,
  so a broken handshake shows as a freeze, not a desync (bank log).
- Zebes's explosion fails on snes9x. The spacetime beam differs between
  emulators. Lag and door transitions are longer on lsnes than on snes9x
  1.43 (4 and the game resources page).
- No bsnes, Mesen, snes9x or LakeSnes issue we found reports an SM desync.
  We searched the GitHub issues for "Super Metroid"; the hits were practice
  ROM savestates and hacks. We found no public report of the Ceres sequence
  desyncing. Our LakeSnes desync at about 8567 is our own finding. It comes
  after the shifts near "NMI re-enable in loads" at 8314, which points to
  the same communication-timing cause as 1342 (**inference**).

## What this means for us

**Which oracle to trust, stream by stream.**

| Stream | Trust | Why |
| --- | --- | --- |
| Game logic and content (wram less sound bytes, vram, cgram, oam), compared allowing a one-frame shift | both | no content difference in 4800 frames |
| Frame-exact lag pattern, NMI counters, the movie's sync | lsnes | the movies were recorded there and are defined there. No console has verified them, and per 3 a console would not reproduce the lag near an upload anyway. |
| CPU-SPC handshake timing (`$0641`, the upload length, ports) | lsnes over LakeSnes | cycle-synchronised cothreads against catch-up in whole opcodes; LakeSnes's README admits the gap. Neither is proven against hardware. |
| Picture within a line | neither | both render per scanline; SM rarely needs more |
| Sound sample timing | neither, for now | lsnes uses blargg's DSP, LakeSnes runs per sample |

The policy to follow from this: treat a one-frame shift after an APU upload
as a known legacy oracle difference (`LEGACY(oracle)`), not as engine
behaviour. Pin the lag pattern to lsnes. For the engine, model the slowdown
after an upload as a function of the SPC clock, a parameter (Jim's rebuild decision: slowdown
is modelled and can be toggled), with 32040.5 Hz as the default because the
movies were recorded at it.

**What we can't settle without hardware.** The "true" lag after an upload,
which differs between consoles. Whether any real console runs our movies:
none has been verified, and it would need a TAS replay device and a clock
mod like rasteri's.

**What would settle 1342, using Mesen as a separate process.** MesenCE is
GPL-3: we run its released binary as a separate process and never link or
copy its code, as with lsnes.

1. **Margins.** On all three oracles, log the master-clock position of the
   upload's last echo (`$80:804F`) and the RTI of step 1342's NMI, with
   lines to the next vblank. If lsnes and LakeSnes straddle the boundary
   by under a line, the rate (16 ppm) could matter; otherwise it is the
   handshake latency.
2. **A third vote.** Run MesenCE headless
   (`--testRunner script.lua rom`, `emu.exit`; it plays `.bk2` movies or
   takes inputs from Lua) with `SpcClockSpeedAdjustment=40` (32040 Hz; bsnes v085's 32040.5 lies
   between 40 and 41, so run both). Record `$05B6`, `$05B8` and `$0641` for steps
   300-400 and 1300-1400. If MesenCE agrees with lsnes at 1342, LakeSnes's
   port catch-up is the bug, and a fix 0002 (sync the SPC to the CPU
   cycle on each port access, with a write latch like Mesen's) is the
   candidate. If MesenCE agrees with LakeSnes, the difference is in bsnes
   v085, and we record it in LEGACY.md as lsnes's.
3. **The console spread.** Sweep MesenCE from 32000 to 32160 Hz in steps of
   10 Hz and note the rates where frames 353 and 1342 change lag. If those
   thresholds fall within 32030-32152 Hz, the result in 3 holds for our
   movies: lag near an upload depends on the console. That decides that the
   engine's upload slowdown should be a modelled clock parameter, not a
   fixed frame count.
4. **Optional, ares** (cycle-based like bsnes v115, but no scripting
   interface) only if Mesen and lsnes disagree with each other.
