# Recording-emulator synchronization baseline

Checked 2026-10-09. The original any% movie replays headlessly in **lsnes
rr2-β23 / bsnes v085 Compatibility** on this host. Its independent checkpoint
capture drives diagnosis of hardware and movie timing. Generated/interpreter
comparison separately checks our recompilation against shared hardware.

## Source identity and local build

The [official TASVideos lsnes page](https://tasvideos.org/EmulatorResources/Lsnes) links the [rr2-β23 binary/source archive](https://files.tasvideos.org/emulatordownloads/lsnes/lsnes-rr2-beta23.7z). Its `sources.tar` contains the actual lsnes and bsnes sources. The local source has `VERSION=2-β23`; `bsnes/bsnes.mk` specifies `BSNES_VERSION=085` and debugger support. The downloaded movie's `coreversion` is `bsnes v085 (Compatibility core)`, agreeing with startup diagnostics.

| Artifact | Local path | SHA-256 |
|---|---|---|
| Official archive | `deps/downloads/lsnes-rr2-beta23.7z` | `aa56d499d0393e24c8d7c9eaf0ed337916f07c02fdb2d281613bcb1cd052a1d7` |
| Original source tar | `deps/lsnes/sources.tar` | `e9c0336218a47586e85b7c38341114995b3b7cdd727baf7c09c307d82072ef25` |

Unpacked source is `deps/lsnes/source`. Private Ubuntu 20.04 development packages are extracted into `deps/lsnes-build-deps` using `dpkg-deb -x`; there is no system package installation. Required packages are Boost 1.71 headers and filesystem/system/iostreams development and runtime packages, Lua 5.2 headers/interpreter/runtime, and libcurl OpenSSL development headers. Their original `.deb` files remain in `deps/downloads`. The host already has g++, make, zlib headers, pkg-config, and the curl runtime.

The local `options.build` config selects native regex/threads, dummy graphics/sound/joystick, `BUILD_BSNES=yes`, `BSNES_IS_COMPAT=yes`, and disables Gambatte, Opus, liblzma, libgcrypt, and samplerate features. Private include paths go in `USER_CFLAGS`; private library paths and rpath go in `USER_LDFLAGS` and `USER_HOSTCCFLAGS`. `HOSTCC=$(CC) $(USER_HOSTCCFLAGS) -DNATIVE_THREADS` supplies the private headers to the host controller generator too.

Compatibility changes are limited to build plumbing:

- Add `<functional>` to `include/library/httpreq.hpp` and `include/library/loadlib.hpp`.
- Add `<vector>` before namespaces in `bsnes/snes/snes.hpp`, because the legacy sinc resampler relies on that missing include.
- Add `-std=gnu++14` to `USER_CFLAGS` and replace bsnes' `-std=gnu++0x` with `-std=gnu++14`; modern GCC rejects the old multi-statement constexpr templates in C++11 mode.
- Create private `lua` / `luac` symlinks to `lua5.2` / `luac5.2`, and a private `libcurl.so` symlink to `/usr/lib/x86_64-linux-gnu/libcurl.so.4` (the development package's target library is installed on the host).
- Add top-level `headless` target depending on the version/host helpers, running `$(MAKE) -C src precheck` and `$(MAKE) -C src util/lsnes-dumpavi.util`. This skips the wxWidgets GUI executable.

No emulation or controller logic was changed. The tracked setup script applies
these changes idempotently and creates the private build configuration:

```sh
just setup-lsnes
cargo run --manifest-path tools/bootstrap/Cargo.toml --locked --offline -- setup-lsnes --offline  # verify cached downloads and rebuild
```

Download URLs and hashes are in [lsnes-dependencies.toml](lsnes-dependencies.toml).
The build log is `deps/lsnes/build.log`. The underlying build command is:

```sh
cd deps/lsnes/source
PATH=/home/jim/sm-decomp/deps/lsnes-build-deps/usr/bin:$PATH \
PKG_CONFIG_PATH=/home/jim/sm-decomp/deps/lsnes-build-deps/usr/lib/x86_64-linux-gnu/pkgconfig \
make -j4 headless
```

The resulting executable is `deps/lsnes/source/src/util/lsnes-dumpavi.util`. `--dumper=list` lists `INTERNAL-NULL`, which performs replay without writing audio/video files. These options are documented in the [archive's manual](../deps/lsnes/source/manual.txt); `src/video/null.cpp` implements the null dumper. If a previous failed build left an empty `__all__.files`, regenerate it after fixing the failure: Make otherwise considers that truncated file current.

## Capturing checkpoints

Use `--rom-a=rom/sm.sfc`, the manual's explicit first ROM slot form. The historical `--rom` validation in `src/core/romloader.cpp` has an inverted type check and rejects the correct ROM. No source patch is needed with the explicit slot option. The original movie archive supplies its ROM hash and initialization. Startup reports `snes_ntsc` and bsnes v085 Compatibility.

The script observes CPU execution at `$82:8948` and reads semantic WRAM words. This is the entry of `RunOneFrameOfGame` in [snesrev's original-address annotation](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/src/sm_82.c); field addresses come from its [variables header](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/src/variables.h). The `items` column is **collected items**, not equipped items. Multiple visits in a video frame must remain distinct.

```lua
local out = assert(io.open('/home/jim/sm-decomp/out/lsnes-any-checkpoints.csv', 'w'))
out:write('frame,event,game_state,room,x,y,health,items,time_frames,time_seconds,time_minutes,time_hours\n')
local offsets = {0x0998,0x079B,0x0AF6,0x0AFA,0x09C2,0x09A4,0x09DA,0x09DC,0x09DE,0x09E0}
local event = 0
local function checkpoint()
  event = event + 1
  local values = {tostring(movie.currentframe()), tostring(event)}
  for _,address in ipairs(offsets) do
    values[#values+1] = string.format('%04X', memory.readword('WRAM', address))
  end
  out:write(table.concat(values, ','), '\n')
  out:flush()
end
memory.registerexec('BUS', 0x828948, checkpoint)
```

`sm-pipeline lsnes-replay` creates this observer and an isolated configuration
directory, verifies the movie's ROM and core identity, and runs the null dumper:

```sh
just lsnes-replay movies/super-metroid-any-3653.lsmv 12000 out/sync/lsnes.csv
just sync-smoke  # captures and checks the passing boot prefix through 390
just sync-check  # captures and strictly checks both cores through 12000
```

For a manual capture, save the Lua above as `/tmp/lsnes-checkpoints-loop.lua`, then:

```sh
XDG_CONFIG_HOME="$PWD/out/lsnes-config" \
deps/lsnes/source/src/util/lsnes-dumpavi.util \
  --dumper=INTERNAL-NULL --rom-a=rom/sm.sfc --length=12000 \
  --lua=/tmp/lsnes-checkpoints-loop.lua movies/super-metroid-any-3653.lsmv
```

`memory.registerexec`, `memory.readword`, and `movie.currentframe` come directly from rr2-β23's `src/lua/memory.cpp`, `src/lua/address.cpp`, and `src/lua/movie.cpp`. They neither alter movie input nor RAM. The game-loop seam avoids comparing incompatible video output phases. Raw RAM hashes remain unsuitable across these cores: original bsnes config initializes WRAM to `0x55` (`bsnes/snes/config/config.cpp`), while RustySNES initializes it to zero.

## Synchronization diagnosis

The initial event-52 mismatch (398 versus 397) was a checkpoint-label error:
lsnes's callback advances the movie frame at V=241, whereas RustySNES wraps at
V=0. `--lsnes-frame-labels` adjusts observations without moving input or CPU
execution. Recording checkpoints observe actual opcode fetches so interrupt
preemption does not create a duplicate visit.

After correcting labels, the first real drift was event 1,818 / frame 2,323.
RustySNES's HDMA loop omitted eight master clocks per active channel per line,
including held-data lines. A synthetic bus regression first failed at 16 versus
24 clocks; the hardware-doc correction passes. The patched replay agrees on
all 7,741 initialized checkpoint events before Ceres loading.

The next mismatch appeared at event 7,742: original frame 8,508 versus candidate
8,509. A narrowed trace locates the lost frame in the sound-program upload:
original reaches WaitForNMI at V=209,H=1318 in frame 8,466; RustySNES reaches it
at V=247 in that frame and misses the NMI at V=225. From then on the CPU work
agrees with a one-frame delay until differing controller inputs cause gameplay
drift. This is a hardware timing issue, separate from generated CPU execution.

A regression confirms that RustySNES samples the CPU-to-APU port at instruction
start instead of the scheduled bus cycle. The `clocked-smp` patch executes bus
operations as clocks elapse and passes its port timing/save-restore regressions.
That correction alone still fails at event 7,742. Per-byte traces locate the
main delay before IPL transfer: original sends the first command at V=30 in
frame 8,440, while the clocked candidate sends it around V=64. Relative timing
of subsequent 16,208 command/data writes stays close. The driver is waiting on
an APU timer; its period agrees while accumulated phase differs.

An explicit `lsnes-profile` selects 32,040.5 samples/s instead of RustySNES's
32,040. The 0.5 Hz difference accumulates through the intro. With this profile,
all **10,472 checkpoints in frames 272–12,000 pass**, with no tolerance or
resynchronization (`out/sync/rate-12000.json`). The candidate now reaches the
same Ceres rooms and exact movement/health/game clock. An uninterrupted black-box trace between movie frames 2,746 and 4,066 measures
11,000 timer ticks over 471,902,768 master clocks (NTSC short lines accounted
for), implying **32,040.495 samples/s**. Other uninterrupted stretches give
32,040.489 and 32,040.522. These traces substantiate the explicit profile rather
than selecting a rate solely to make the TAS pass. The timer can be reset or
its software reads can lag during DMA, so calibration uses uninterrupted
segments. Full any% replay is in progress.
See [the maintained patches](../tools/patches/README.md). No game code or input
records were adjusted to compensate for the hardware timing.

```sh
just sync-check                   # strict 12,000-frame independent baseline
just hardware-check               # APU clock/port regressions, no game ROM
just iterate                      # generated versus interpreter comparison
```

Both gates are required: independent TAS checkpoints check hardware/input
accuracy; mixed execution compares CPU/clocks, full WRAM and video against the
same hardware backend. Full any%, low%, and 100% completion remains unverified.


The extended any% run found the next mismatch at event 15,175,
original frame 17,976 versus candidate 17,977 during a door transition in room
97B5. All 15,174 prior initialized checkpoints match. The first position
mismatch follows at event 15,684 (original frame 18,588). Full-movie
synchronization is therefore not yet established. Instruction traces show
identical CPU work but about 3,900 extra master clocks before WaitForNMI.
The enabled channel terminates at scanline 32; RustySNES continued charging
eight clocks per line afterward. The documented correction removes global
overhead when no enabled channel remains active. Its regression first fails
at eight versus zero clocks, then passes. A cached replay passes all 11 events
in frames 17,975–17,989 (`out/sync/door-fixed-report.json`). Fresh power-on
validation passes all 15,184 initialized events through frame 18,000
(`out/sync/hdma-complete-prefix.json`), including the formerly failing door.
The candidate with HDMA corrections alone later fails a checkpoint frame label
at event 15,324 (original 18,228 versus candidate 18,227), and gameplay position
at event 24,809 (original frame 34,896). The full 129,712-frame lsnes oracle capture has
completed successfully; completion metadata binds its CSV hash to that count.

The frame-label trace identifies several DMA transfers before a checkpoint at
V=241,H=12 on the recording core, versus V=240,H≈1288 on the candidate.
RustySNES executes DMA immediately at the register write and omits entry/exit
alignment. The documented `clocked-dma` patch allows one more CPU cycle,
aligns entry to the eight-clock divider, and aligns exit to the following CPU
cycle. Both interpreter and generated adapters use a complete-cycle hook.
The documentation's worked examples and armed-DMA save/restore tests pass.
The cached boundary moves much closer (about 20 clocks early), but its strict
label remains unresolved. A new power-on/full replay is running with this
patch. The prior HDMA-only 19,000-frame generated/interpreter comparison passed
48,345 generated instructions; it is being repeated for the DMA change.


A synthetic three-byte DMA test cartridge gives a tighter independent oracle.
The recording core's first opcode is at V=0,H=186, whereas RustySNES began at
H=16. The explicit `lsnes-profile` now sets this 170-clock startup difference
before reset-vector reads, leaving soft reset unchanged. With matching startup
phase, initial synthetic CPU and DMA instruction times agree exactly.
DMA alone still gives occasional frame-label differences after frame 18,000;
its first mismatch is event 15,314 / original frame 18,217. The new startup
profile is undergoing fresh power-on validation.


## Current interrupt investigation

The original any% capture completed all 129,712 frames, yielding 93,694
checkpoint visits. The candidate with DMA alignment, recording reset phase,
final-cycle interrupt samples, and discarded interrupt fetches passes all
15,184 strict visits through frame 18,000. It first differs in frame label at
event 15,314 (18,217 versus 18,218), then in vertical position at event 24,809
(34,896 versus 34,897, Y=$06E0 versus $06E2). This candidate was interrupted
after finding that failure; its partial CSV cannot certify full completion.

A tighter synthetic regression varies the instruction start phase with zero
to seven leading NOPs. Original-core handler times at V=225 are H=94,88,88,86,
80,80,78,92, respectively, with corresponding returned PCs verified. The
candidate originally entered four phases two clocks early. The implied
instruction's final cycle drives PB:PC+1 (WDC Table 5-7, 19a); a pending
interrupt turns that cycle into a memory-speed discarded read. Adding this
behavior matches all eight oracle phases. It is also forwarded through the
runtime bus and emitted NOP/DEX bodies, with a generated/interpreter regression.

`just check`, `just lint`, patch reapplication on fresh pinned source, and
`just doctor` pass with these changes. Save/restore retains sampled interrupts;
checkpoint observations exclude discarded interrupt fetches. A fresh 19,000-frame replay with implied-cycle and refresh corrections still
first differs at event 15,314 (frame 18,217 versus 18,218). Full any%, low%, and 100% synchronization remains
unverified. The six-clock VBlank-event sampling delay is provisional, and IRQ
I-flag sampling for CLI/SEI/PLP/REP/SEP remains an open accuracy item.


A frame-boundary observation probe also checks the checkpoint seam: a NOP
starting at V240,H1360 produces the original core's frame callback before its
execute callback. Thus an execute checkpoint can belong to the following
frame when its opcode fetch crosses the boundary. Observing after the read's
first half remains correct; moving labels to the instruction start would
hide that case. This was checked with black-box Lua callbacks, not core source.


The latest strict cached replay still first differs at event 15,314 / frame
18,217. An early diagnostic recorded observations after executing the opcode;
that file must not be used for acceptance. Repeating the interval with an
actual-fetch observer restores the same first failure. The corrected cache is
`out/sync/current-18190.state`; it reduces the feedback loop to about ten
seconds for an 810-frame interval. Original trace:
`out/sync/current-lsnes.txt`. Candidate trace: `out/sync/current-timing.csv`.

WRAM refresh was fixed at H536 on every line. The documented eight-clock
phase requires the first H538 and subsequent H534/H538 starts; its regression
first fails at 577 versus 537 clocks, then passes. This removes some local
refresh shifts but does not fix the cached TAS boundary alone. A 22-phase
FastROM LDA/BNE synthetic sweep matches original NMI handler times and returned
PCs exactly. The real game's NMI entry still differs by a few clocks because
its preceding hardware phase differs.

A separate HDMA test exposes mid-cycle pauses versus original complete-cycle
pauses, with phase-dependent resume timing. Initial alignment experiments were
reverted; the independently validated correction is described below. The 19,000-frame generated/interpreter replay
with implied-cycle corrections (before the refresh adjustment) passes 48,345
generated instructions and 272,930,861 interpreted instructions, comparing
CPU/clocks, complete WRAM, and framebuffer. It establishes iteration readiness,
not full original-core TAS acceptance.


### HDMA cycle-boundary regression

The recording profile now defers HDMA to complete CPU-cycle boundaries,
with positive eight-clock entry alignment and CPU-cycle-relative resume.
Frame initialization runs near dot 6 even when no channels are enabled;
visible transfers include scanline zero and stop charging waits after all
channels terminate. An implied final cycle paused by HDMA performs the
memory-speed discarded read observed in black-box phase probes.
Line-counter reload reads share the existing eight-clock channel overhead;
the feature-disabled implementation retains its prior cost.

Public timing sources: [Timing](https://wiki.superfamicom.org/timing#hdma-134)
and [DMA and HDMA](https://wiki.superfamicom.org/dma-and-hdma#hdma-51).
A synthetic cartridge enables HDMA during VBlank and mixes NOP/NOP/BRA.
Its 33,098 original-core instruction observations match through the captured
interval. `crates/snes/tests/hdma_oracle.csv` retains 105 boundary observations
covering setup, pause phases, reloads, and frame rollover. The regression
fails on the previous maintained patch at master 357394 versus 357378 and
passes with these changes. No original emulator execution source was read.
The separate `$80` line-counter interpretation remains unresolved and unchanged.

A fresh replay of this final patch passes all 15,184 strict checkpoints
through 18,000 frames. The first later mismatch is event 15,292: original
frame 18,195 versus candidate 18,196, with identical recorded semantic fields.
The completed 36,000-frame replay first differs in gameplay at event 24,934:
original frame 35,203 versus candidate 35,204, room $9CB3, X=44 versus 43.
The previous frame-34,896 position failure is removed. These are diagnostic
milestones; the strict comparison still fails at frame 18,195.
Synthetic agreement and workspace checks do not establish full TAS synchronization.

### IRQ mask sampling

A delayed HV-IRQ cartridge (HTIME=152, VTIME=2) matches original-core handler
times in all 22 FastROM LDA/BNE start phases. A simple adjustment to IRQ
signal timing is therefore not justified by these probes.

A pending-IRQ cartridge exposes a separate mask-sampling defect. Before the
fix, CLI enters the handler at H1032; the original executes the following NOP
and enters at H1044. Public timing documentation specifies the old I flag
for CLI, SEI, PLP, REP and SEP. The maintained patch now keeps that mask for
the next instruction-boundary poll, while the interrupt stack retains the
updated flags. All five original-core handler/return/status cases pass.
Save/restore and generated NOP/REP/SEP regressions also pass. CPU0 carries an
IM01 extension; caches from the preceding profile are incompatible.

The completed 19,000-frame generated/interpreter replay before the mask fix
passes CPU/clocks, complete WRAM and framebuffer, with 48,345 generated and
272,286,496 interpreted instructions. Workspace checks, lint, doctor and
fresh-source patch application pass with the mask fix. The fresh 36,000-frame
replay in `out/sync/irq-mask-checkpoints.csv` completed successfully. Strict
comparison passes 15,184 events through frame 18,000 and still first differs
at event 15,292 / frame 18,195. The first semantic mismatch remains event
24,934 / frame 35,203. Thus the mask defect is fixed, but it did not resolve
the current TAS divergence. Reports: `out/sync/irq-mask-18000.json` and
`out/sync/irq-mask-36000.json`.

### Broader HDMA phase probes

`just probe-hdma indirect branch 4` assembles a small cartridge from public
register documentation, runs the original core as a black-box oracle, and
compares instruction-address/master-clock observations. All four phases pass
(28,172–28,173 observations each). `just probe-hdma indirect nop 8` exercises
held lines, reloads and termination using NOP/NOP/BRA, and currently fails
some phases by two clocks at setup or later pauses. Direct-transfer variants
also expose errors of two to six clocks. These are diagnostic failures, not
part of the passing workspace gate. Reports and independent synthetic ROMs
are under `out/sync/hdma-probes`.

The next investigation is implied-instruction timing at HDMA pause and resume.
Changing only the resume-cycle speed or adding a fixed delay does not match
all phases; those experiments were reverted. The maintained hardware patch
is unchanged by this probe work. Probe traces observe instruction starts to
match the original trace API; movie checkpoints retain their actual-fetch
observer and remain the independent acceptance gate. The fresh diagnostic
cache `out/sync/irq-mask-18190.state` belongs to the IM01 profile.
