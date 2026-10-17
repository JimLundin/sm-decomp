# RustySNES hardware correction

`rustysnes-hdma.patch` applies to commit
`98f8d398020b8798b16076fae315960031c6b3b7`. It adds eight master clocks per
active HDMA channel per scanline, including lines that hold the previous data.
The original loop charged transferred bytes and table reloads but omitted this
channel overhead. It also stops charging global scanline overhead when all
enabled channels have terminated. An enable bit remaining in `$420C` does not
keep a terminated channel active.

The hardware rule comes from [Super Famicom Dev — DMA and HDMA](https://wiki.superfamicom.org/dma-and-hdma):
“Each active channel incurs another 8 master cycles overhead ... for every
scanline, whether or not a transfer actually occurs.” The same document limits
scanline overhead to periods with at least one unterminated channel. The
existing global overhead amount is retained for active periods.

The regression in `crates/snes/tests/hdma_timing.rs` fails on the original
source (16 clocks instead of 24) and passes after applying the patch. Strict
lsnes game-loop checkpoints also match through frame 4,000 after the patch,
where the original core first drifted at frame 2,323 after correcting labels.
The terminated-channel regression fails at eight versus zero clocks before
the second correction. A cached door-transition replay then passes frames
17,975–17,989, where the uncorrected core misses NMI at frame 17,976. Fresh
power-on validation passes through 18,000 frames; see `docs/LSNES_BASELINE.md` for acceptance.

The Rust `tools/bootstrap` command `prepare-rustysnes` verifies the pinned archive and the original and
patched file hashes. It refuses unexpected local edits. `just setup` and CI
apply this patch before Cargo builds. The root Cargo patch selects that local
hardware core and its unchanged sibling chip crates. Downloaded sources remain
ignored; only this small patch and its regression are maintained here.

RustySNES's source retains its upstream copyright and MIT OR Apache-2.0
license, available in `deps/rustysnes/LICENSE-MIT` and `LICENSE-APACHE`.

## Clocked APU bus

`rustysnes-clocked-apu.patch` adds an opt-in `clocked-smp` feature, enabled by
this project's `lsnes-profile` hardware dependency. The original APU plans an entire SPC700
instruction eagerly: even port reads occur before their scheduled bus cycle.
Only its output writes were deferred. A CPU write between opcode fetch and the
port read is therefore missed until a later instruction, slowing the upload.

The new path caches completed bus reads and resumes the existing decoder from
the instruction's initial registers. Exactly one real bus operation advances
at a time. Port reads sample between their half cycles; RAM, timers, DSP, and
port writes advance as clocks elapse. It adds no allocation in the clock pump.
The existing decoder and original execution path remain in upstream source.

Public timing reference: [fullsnes — SNES APU Low Level Timings](https://problemkaputt.de/fullsnes.htm#snesapulowleveltimings).
The implementation uses RustySNES's own CPU bus contract and cached black-box
lsnes trace outputs. No reference-emulator execution source was used for this
change. Upstream provenance notices are retained; this does not resolve the
upstream provenance review recorded in `docs/provenance-open-question.md`.

`just hardware-check` tests the actual patched source in an isolated workspace
with the root dependency lock. Tests exercise reads on both sides of the sample
cycle, a different ALU instruction, deferred writes, and a mid-read save/restore.
Feature-enabled APU states carry an additional marker and execution data;
feature-disabled states are rejected by the clocked path. Do not interchange
save files between these modes. The checksum manifest covers every patch target.


The `lsnes-profile` feature also chooses a deterministic APU clock ratio of
4,101,184 / 42,954,540 SMP base clocks per NTSC master tick (32,040.5 samples/s).
The original hardware ratio stays available when the feature is disabled.
The APU crystal is independent of the video crystal and varies between
machines; TAS portability requires a matching recording profile. A 12,000-frame
replay with this profile passes all 10,472 strict semantic checkpoints. The
same replay with 32,040 samples/s misses NMI after the reset-to-IPL handshake.
An uninterrupted 11,000-tick black-box measurement gives 32,040.495 samples/s;
other clean segments agree within 0.03 Hz. Full any% replay is in progress.

## Clocked DMA bus

`rustysnes-clocked-dma.patch` adds the default-off `clocked-dma` feature,
enabled by the project's `lsnes-profile`. Public timing specification:
[Super Famicom Dev — Timing, DMA](https://wiki.superfamicom.org/timing#dma-114).
The register write arms DMA, one more CPU cycle runs, the transfer waits for
its eight-clock divider, and return waits for a whole following CPU cycle.
Strictly positive entry and exit waits reproduce the documentation's worked
examples. The underlying byte-transfer implementation is retained.

A `begin_cpu_cycle` hook preserves complete cycles across split read phases.
The interpreter and generated-code bus adapters both call it. The regression
first fails because the original register write transfers immediately. Tests
also cover differing resume speeds and an armed-DMA save/restore. Feature
states carry `DMA1` in BUS0 and reject states from incompatible modes.

Apply this patch after the HDMA and clocked-APU patches. Its checksum manifest
uses their intermediate source identities and covers both the hardware core
and the CPU hook. Setup and doctor verify the final source. The upstream
execution path remains available with this feature disabled.

## Recording-core reset phase

`rustysnes-lsnes-reset.patch` applies only under `lsnes-profile`. An original
black-box trace of a synthetic ROM starts at V=0,H=186; RustySNES previously
started at H=16 after its two reset-vector reads. The profile advances hardware
170 clocks before those reads at power-on. Soft reset preserves the running
clock phase. This is an explicit recording-core initialization setting.
With matching startup phase, the synthetic probe's initial CPU and DMA loop
instruction times agree exactly, including the documented 48/54-clock waits.
The reset regression locks the measured first-opcode phase and soft reset.
A fresh full game replay is required to establish its TAS acceptance interval.

## Clocked interrupt sampling

`rustysnes-clocked-interrupts.patch` applies after the clocked-DMA patch.
It enables the CPU's default-off `clocked-interrupts` feature for this profile.
[Super Famicom Dev — Timing, Interrupts](https://wiki.superfamicom.org/timing#interrupts-28)
places the interrupt check before the final complete CPU cycle and identifies
interrupt entry's first cycle as a discarded opcode fetch at PB:PC. The bus
keeps the final-cycle line samples and serializes them with its DMA extension.
Checkpoint observers suppress this discarded fetch.

[WDC W65C816S datasheet](https://www.westerndesigncenter.com/wdc/documentation/w65c816s.pdf),
Table 5-7, 19a, exposes PB:PC+1 during the implied instruction's final cycle.
Eight synthetic-ROM black-box traces show a pending interrupt makes that
cycle a memory-speed read rather than a fixed six-clock idle. The regression
checks original-core handler times and return addresses across all eight
phases; the generated NOP adapter has a matching differential regression.
No commercial ROM is needed for these tests.

The current profile also delays sampling a newly raised VBlank NMI by six
master clocks from RustySNES's dot-level event. This reproduces those probes,
but the exact event-to-sample phase is still provisional. IRQ mask sampling
uses the old I flag for CLI/SEI/PLP/REP/SEP, matching five original-core
cartridge captures of handler clocks, return addresses and stacked flags.
The generated adapter consumes the same sample and records the old mask
for generated REP/SEP. CPU0 includes an IM01 extension, so earlier profile
save files and diagnostic caches are incompatible. Full TAS synchronization
still requires the independent movie acceptance gate.


The interrupt patch also corrects the profile's refresh phase using the public
Timing document: the first pause starts at H538, later pauses at H534 or H538
on the same eight-clock divider. The prior fixed H536 path remains with the
feature disabled. A no-ROM regression checks the first two pause boundaries.


The interrupt patch also carries the profile's HDMA CPU-cycle scheduling.
Setup near dot 6 and transfers at dot 278 pause after complete CPU cycles,
then align entry and resume. Terminated channels add no pause. NOP's implied
cycle can perform a discarded memory-speed read when paused; the generated
adapter forwards the same halt query. Per-channel overhead includes a reload
of the line counter; indirect pointer reads add their own cost. These changes
are gated by `clocked-dma`. The black-box synthetic cartridge regression in
`crates/snes/tests/hdma_timing.rs` checks 105 original-core boundary observations.
See `docs/LSNES_BASELINE.md` for the independent TAS acceptance status.
# Passive readiness observation

`rustysnes-readiness-observation.patch` is applied after the recording-profile
patches. It adds an explicitly armed event buffer at existing CPU and internal
DMA bus operations, plus passive complete pending-hardware serialization. It
does not read extra registers, poll interrupts, advance clocks, or change chip
execution. Events distinguish cycle starts, real reads/writes, clock spans,
interrupt entry and DMA A/B transfers. Instruction metadata classifies the
already recorded first opcode fetch and final implied cycle.

Ordinary bus serialization supplies hardware, latches, input and pending DMA;
additional serialization includes CPU-visible DMA scratch and per-dot PPU
pixels/carry/cursors omitted from ordinary frame saves. Observer buffers are
excluded from state equality. Readiness comparison normalizes only the
interpreter's private per-step cycle accumulator, which is reset on every
instruction; architectural cycles and pending old-I override are retained.
Checkpoint restoration must preserve raw bookkeeping separately.

Synthetic reference/subject instruction equality, transient-store detection,
and observed/unobserved internal DMA equivalence exercise passivity. Patch
source identities are checked by preparation, doctor and readiness preflight.
