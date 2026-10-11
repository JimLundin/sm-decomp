# Reference provenance, simulation stability and accuracy

Reviewed 2026-10-11 in `/home/jim/sm-decomp`. This is a read-only investigation of primary source code, retained run verdicts and provenance records, with fresh upstream-ref and official-document checks. No builds, simulations, source changes, evidence mutations or fresh live-evidence authentication were performed. The optimization worktree and branch were not inspected or changed. Prior post-cleanup authentication is cited as retained evidence, not represented as a check performed for this report.

## Assessment

The source stack is sufficiently grounded to continue **bounded C-to-Rust translation**, and the retained full-input C/mixed replay provides strong reproducibility evidence for its frozen implementation and measured ownership. It does **not** establish a completed vanilla game or original-ROM timing accuracy. The original-ROM adapter currently establishes 1,097 causally ordered completed updates through original frame 1,600; it remains deliberately nonaccepting. The separate RustySNES recording profile has independently accepted a historical 18,000-frame domain, with later HDMA/timing disagreement still open. These are separate execution systems and should remain separate acceptance claims. [C live verdict][live] [Adapter verdict][adapter] [Readiness contract][readiness-sync] [Historical accepted run][ready-brief]

## Have we properly sourced the reference material?

For the selected executable baseline, mostly yes. `tools/dependencies.json` pins snesrev/sm `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, its archive URL and SHA-256. `port_family.verify_source` checks each archived file against the extracted checkout before compiling actual C. The C notice, including the separate Opus notice, is retained in `crates/decomp/LICENSE.upstream.txt`. Original ROM and extracted artifacts remain local. Routine manifests/inventory retain source symbols, addresses where supplied, definition hashes and dependency information; static inventory entries are explicitly unverified. [Dependency pins][pins] [Source verification implementation][family] [Retained notice][license] [Translation contract][port-doc]

The primary input references are also well identified: the movie manifest records author, TASVideos publication/submission/download URL, complete frame count, ROM SHA-256, recording emulator and archive digest for any%, low% and 100%. The official any% submission was checked again during this review: it identifies Sniq, lsnes rr2-β23, 129,712 frames and the Japan/USA ROM. Its `Start Type: Unknown` field is not sufficient initialization evidence; the clean-start conclusion comes from archive contents and the official LSMV rules. Those rules were checked again and define one **subframe**, rather than necessarily one video frame, per input line, with `F.|BYsSudlrAXLR` controller order and explicit reset/continuation behavior. [Movie manifest][movies] [Official submission](https://tasvideos.org/5833S) [Official LSMV specification](https://tasvideos.org/EmulatorResources/Lsnes/LSMV) [Local archive inspection][tas-doc]

The original recording core comes from the official rr2-β23 archive and source tar, both hash-pinned. Its local build changes are declared build compatibility edits; the project contract requires byte verification against the official source plus those edits. Candidate timing changes are developed from public hardware descriptions and black-box original-core measurements, rather than reading the recording core's execution implementation to fit the candidate. [Recording-core pins][lsnes-pins] [Build recipe][lsnes-doc] [Strict provenance requirements][readiness-sync]

Fresh read-only `git ls-remote` checks on 2026-10-11 returned:

| Repository/ref | Observed SHA | Meaning |
| --- | --- | --- |
| [snesrev/sm main](https://github.com/snesrev/sm/tree/578f90b3cc49557bb70060ad033bb90b8cf8ac50) | `578f90b3cc49557bb70060ad033bb90b8cf8ac50` | Selected C pin remains upstream main |
| [testyourmine/sm_rewrite main](https://github.com/testyourmine/sm_rewrite/tree/523787384a7c912153883b21714f5edabf4e2406) | `523787384a7c912153883b21714f5edabf4e2406` | Supplementary readability pin remains main |
| [InsaneFirebat/sm_disassembly main](https://github.com/InsaneFirebat/sm_disassembly/tree/7af131ebc328359e852e923603163a329c7aa7af) | `7af131ebc328359e852e923603163a329c7aa7af` | Vanilla assembly reference pin remains main |
| [MrJackSpade/SuperMetroidDecomp main](https://github.com/MrJackSpade/SuperMetroidDecomp/tree/e127065b254b22bf4d31e455a895f3452dc913a6) | `e127065b254b22bf4d31e455a895f3452dc913a6` | Newer than reviewed `216f3a1d...`; current source was not inspected or validated |

The remaining sourcing gaps are specific:

- The current C oracle is intentionally an old, buggy baseline, as its own README states. Upstream PR #50 restores Kraid scratch-memory dependencies across projectile-hit iterations; its retained actual diff deserves an original-ROM regression fixture. Main being unchanged does not make the known defect disappear. Do not automatically copy that fix into both comparison sides and treat agreement as independent validation. [Pinned C README](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/README.md) [Actual PR #50 diff][kraid-diff]
- The broad assembly reference has not been independently assembled and byte-compared in the retained source audit. Perform a vanilla-ROM rebuild comparison before elevating imported symbols/layouts to a certified address baseline. The project's build/extraction tools make this feasible. [Assembly README](https://github.com/InsaneFirebat/sm_disassembly/blob/7af131ebc328359e852e923603163a329c7aa7af/README.md) [Existing source audit][sources]
- The C# implementation is useful design and bug-case material, but the reviewed pin had no located project-level copying grant and inconsistent completeness claims. Its fast-moving current HEAD needs a targeted refresh before its recent results guide a new task. A ref check establishes freshness, not inspected behavior or permission. [Primary reviewed C# tree](https://github.com/MrJackSpade/SuperMetroidDecomp/tree/216f3a1d38c2e9d82ec9c0e626a35415b74d47c9) [Retained community audit][community]
- `tools/patches/README.md` references an upstream provenance question at `docs/provenance-open-question.md`, but that document is absent in this checkout. The missing link should be restored or the unresolved question restated explicitly; this review cannot establish what it covered or whether it was resolved. [Patch documentation][patches]

## What the simulation evidence actually proves

| Gate/system | Retained result | Accuracy boundary |
| --- | --- | --- |
| Offline actual-C/Rust families | Full 128 KiB WRAM and returns for declared cases; independent expected cases and corruption controls | Establishes the selected C semantics in that domain. Ordinary family boundaries do not observe transient store order or hardware timing. [Contract][port-doc] |
| C scheduler with selected Rust replacements | Four fresh subjects, all 129,712 input records consumed, `state_effects_match: true`, zero measured CPU fallback; prior post-cleanup authentication covers 1,345 artifacts per subject | Periodic serialized hardware/game state, terminal WRAM/SRAM and ordered PPU/APU effects agree across C/mixed. Does not prove an original-equivalent route, universal fallback absence, vanilla timing, audible fidelity or repeated in-process initialization. [Verdict][live] [Authentication][live-auth] |
| Original ROM → causal C adapter | 1,097/1,097 completed updates and 1,248 controller samples through frame 1,600; semantic and poll sequence/count checks match | Limited semantic correspondence with explicit waits retained. Hardware-frame timing, full WRAM equality and real APU handshake scheduling remain unproved; overall status is `nonaccepting`. [Verdict][adapter] |
| RustySNES generated/interpreter parity | Historical readiness source passes 18,000 frames, executing 46,503 generated and 257,700,507 interpreted instructions | Same hardware backend on both sides. Does not independently certify that backend; the recorded source is not today's decomp source. [Historical brief][ready-brief] |
| RustySNES/original lsnes strict gate | Separately verified 18,000-frame original oracle; 15,184 ordered actual-fetch visits, labels 272–18,000, ten initialized semantic words, zero offset/tolerance | Strong recording-profile evidence in that interval, with sparse game-state observations rather than complete emulator state or a hardware capture. [Provenance contract][readiness-sync] [Completion evidence][ready-ticket] |
| Later original-core debt | Historical first strict label mismatch at event 15,292, original frame 18,195 vs candidate 18,196; first semantic mismatch at event 24,934, frame 35,203, X=44 vs 43 | These later captures explicitly lack complete legacy source/binary/observer provenance and are diagnostic debt, not eligible original-oracle reuse. [Debt record][debt] |

The full C/mixed replay's measured Rust ownership is substantial but bounded: each mixed subject records 16 RoomGfx allocations, eight WithGfx allocations (four each for the two Ceres definitions), 95,325 scheduler entries with 866 nonempty admitted frames, 419 decompressions and 125,529 RNG calls. All nine supported projectile IDs execute exclusively in Rust with zero support refusals. At input record 129,712 the subjects are still in main gameplay, room `$DF8D`, position `(173,159)`. Consuming the complete recording is therefore not proof of TAS game completion. The verdict itself declares `original_rom_parity: not-tested`. [Full live verdict][live]

## Current original-movie adapter frontier

The adapter's first frame-272 mismatch was an input attribution bug: the raw host-step converter supplied a poll observed **after** the original completed update. The causal candidate restores actual polling after NMI, executes all 150 boot-prelude samples in order, performs the real reset clear, and suppresses the upstream main-entry relocation. It then restores two omitted within-dispatch polling seams: VBlank/HDMA wait and screen-enable NMI wait. Separate omitted-wait controls reproduce the old missing-poll failures; the changed-RNG control independently detects altered gameplay. [Adapter implementation][prefix-host] [Driver][prefix-driver] [Ticket 78][adapter-ticket]

The frame-1,600 extension initially omitted the final real dispatch after the last captured poll. The final host now executes that observed remaining completion and stops before needing an uncaptured sample; EOF during a required continuation rejects. The accepted narrow comparison covers all 1,097 completed updates, including terminal RNG 883, without snapshot injection or repeated final input. Native capture reuse is declared and sealed. The ordinary raw converter's mismatch remains in the same nonaccepting verdict; its top-level `first_semantic_mismatch` must not be confused with the separately passing `causal_update_prefix`. [Final adapter verdict][adapter] [Ticket 78][adapter-ticket]

No real APU wait scheduler has yet been demonstrated. Passive original observations show boot SPC upload frames 168–254 and music upload 316–355 contain many real `$2140..$2143` reads and **zero NMI/controller polls**. Inserting C gameplay updates to represent those elapsed video frames would alter the schedule. The causal comparison also excludes raw accepted-NMI/all-NMI counts and the waiting byte, retaining them with explicit reasons; it does not establish full-state parity. [Prefix contract][prefix-doc] [Comparison fields/exclusions][prefix-driver]

## Stability and false-confidence risks

The known headless C crash at host frame 43,188 is fixed: `SoftReset` deliberately writes game state `$FFFF`, and the independent host previously indexed a 45-entry table with it. The fix enters the genuine reset coroutine at the same ownership boundary as the pinned runtime, rather than guarding the bad index. Repeated 45,000-frame C processes and the later four-subject full replay establish that this crash no longer blocks the measured route. They do not explain why raw original movie inputs led to game over. The initial 45,000-frame regression manually retained the unchanged `headless_frame.h` digest after execution; the ticket explicitly does not represent that as an earlier automatic dependency seal. Later drivers snapshot/hash the header as a declared input. [Soft-reset completion][reset-ticket] [Full replay][live]

Hardware stability has improved through explicit APU bus sampling/rate, DMA alignment, refresh, HDMA pause/resume and IRQ-mask fixes with synthetic original-core probes. The 32,040.5 samples/s APU profile is supported by black-box timer observations near 32,040.495, rather than only tuning for a passing movie. It is a **recording-core profile**, not a measured claim about every SNES console. The six-clock VBlank NMI sampling delay remains provisional; wider direct/indirect HDMA NOP phase probes still differ by two to six clocks, and `$80` line-counter interpretation remains unresolved. Public timing documentation itself describes some HDMA alignment as presumed, so black-box phase cases must remain the stronger local behavioral check. [Patch details][patches] [HDMA investigation][lsnes-doc] [Public timing description](https://wiki.superfamicom.org/timing)

The most likely sources of false confidence are:

1. **Shared mistakes:** C/Rust and generated/interpreter agreement can share a wrong reference or hardware behavior. The C runtime patches its in-memory original ROM; `VerifySnapshotsEq` also copies dozens of regions between comparison snapshots. Our independent `RM_MINE` path avoids that normalized comparator, but it still uses the patched C ROM view and altered scheduler. [Primary `SnesInit`/`VerifySnapshotsEq` source](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/src/sm_cpu_infra.c) [Independent baseline contract][baseline]
2. **Sparse or empty coverage:** total scheduler entries include many empty frames. Per-ID/per-handler hits, live changed-entry coverage and branch fixtures establish ownership; function counts, global Rust hits or zero fallback in one route do not. The next Ridley script ticket specifically lacks actual per-callback interpreter counts despite projectile allocation hits. [Live verdict][live] [Ticket 138][ridley-ticket]
3. **Historical identity drift:** retained acceptance binds its frozen source/binary/configuration, not a later tree, new storage default or relocated path. `out/accepted.json` currently names a historical `/home/jim/sm-recomp/...` path; that pointer is not current decomp admission evidence. [Accepted pointer][accepted] [Live authentication scope][live-auth]
4. **Missing route and lifecycle coverage:** any%, low% and 100% archives exist, but full original-equivalent completion, alternate controller/subframe/reset patterns, later game branches, audio waveform fidelity and save/load behavior are not established by this report. Unsupported ADC overflow remains explicit recompilation debt, even though the enabled-pattern acceptance corpus passes. [Movie scope][tas-doc] [Pattern contract][patterns] [Baseline limits][baseline]

## Concrete next checks and parallel setup

1. Keep the current C/Rust gate as the translation lane. Freeze oracle revision, ROM/script views, Rust dependency hashes and fixture cases per task. Admit whole coherent boundaries before mutation, and retain distinct offline/live/original claims in status output. A faster evidence representation or reused C baseline must preserve those identities and rejection controls. [Existing translation contract][port-doc] [Live transactional admission][live-driver]
2. Give the original adapter its own work owner and queue. Extend the existing 1,600-frame causal candidate toward message boxes and Ceres, stopping at the first ordering/semantic failure. Observe completed dispatches, nested waits, controller source sequence and APU handshakes. Add explicit full-state exclusion specifications before widening original comparison; a matching semantic word subset cannot silently become full WRAM equality. [Frontier][adapter-ticket] [Observer/adapter contract][prefix-doc]
3. Give recording-profile accuracy an independent owner. Reduce the two-to-six-clock HDMA discrepancies to synthetic phase cases, retain original expected clocks, and then run a fresh identity-bound strict candidate replay through the first 18,195 failure. Extend the original provenance-sealed capture beyond 18,000 before treating later debt as acceptance evidence. Any cache used for diagnosis must match serializer/profile and first disagreeing behavior against cold execution. [Hardware debt][lsnes-doc] [Strict reuse contract][readiness-sync]
4. Add an original-ROM Kraid multi-projectile fixture for PR #50 and cross-check the upcoming Ridley E84D/E904/E909 script ownership against pinned assembly and actual interpreter hits. These are targeted ways to detect that a correct C translation faithfully preserved a C bug. [Kraid diff][kraid-diff] [Ridley task][ridley-ticket]
5. Grow a scenario matrix from actual reached rooms/boss states and missing branch counts: low% and 100% cold replays, game over/soft reset, death/pause/message/door transitions, save/load and controller edges. Add expected movie milestones and an original-observed ending/completion condition, so successful EOF consumption cannot masquerade as a successful any% route after game over/reset. Community `sm-json-data` can propose scenarios, but its navigation approximations and randomizer assumptions cannot supply an execution oracle. Acquire real original-ROM fixtures for each promoted vanilla scenario. [Scenario-source limits][sources] [Movie corpus][movies] [Current terminal state][live]

These lanes can run in parallel because their acceptance objects differ, while one integration owner controls shared runtime/ABI changes and final replay acceptance. Independent translation jobs should own different coherent modules and use frozen task packs; hardware/profile changes invalidate the affected replay identities and should not be mixed into an ongoing frozen translation acceptance run. Parallel original captures and translation replays also need a shared disk/CPU budget because large retained observer snapshots can dominate storage. This is a proposed work split, not a claim that the coordination or expanded gates are implemented. [Current run-retention contract][cleanup] [Task dependency and next ownership boundary][ridley-ticket]

[pins]: ../../tools/dependencies.json
[family]: ../../tools/port_family.py
[license]: ../../crates/decomp/LICENSE.upstream.txt
[port-doc]: ../../docs/DECOMP_PORT.md
[movies]: ../../movies/manifest.json
[tas-doc]: ../../docs/TAS_MOVIES.md
[lsnes-pins]: ../../docs/lsnes-dependencies.json
[lsnes-doc]: ../../docs/LSNES_BASELINE.md
[readiness-sync]: ../../docs/READINESS_SYNCHRONIZATION.md
[sources]: ../../docs/DECOMP_SOURCES.md
[community]: ../../docs/DECOMP_COMMUNITY_AUDIT.md
[kraid-diff]: ../../out/research/community/snesrev-sm-pr50.diff
[patches]: ../../tools/patches/README.md
[live]: ../../out/port/observed-projectile-with-gfx-live-full/20261011T142927.603436Z/verdict.json
[live-auth]: ../decomp-port/observed-projectile-with-gfx-live-authentication.json
[adapter]: ../../out/port/controller-terminal-seam/20261011T072946.717098Z/verdict.json
[ready-brief]: ../../out/runs/20261010T112635.727097Z-6bf369628146/brief.md
[ready-ticket]: ../recomp-readiness/issues/12-fresh-readiness-verdict.md
[debt]: ../../out/runs/20261010T112635.727097Z-6bf369628146/diagnostics/historical-debt/debt.json
[prefix-host]: ../../tools/decomp/prefix_host.c
[prefix-driver]: ../../tools/decomp_prefix.py
[prefix-doc]: ../../docs/DECOMP_PREFIX.md
[adapter-ticket]: ../decomp-port/issues/78-original-movie-controller-boundary.md
[reset-ticket]: ../decomp-port/issues/76-reference-soft-reset-crash.md
[ridley-ticket]: ../decomp-port/issues/138-observed-ridley-instruction-frame-ownership.md
[accepted]: ../../out/accepted.json
[patterns]: ../../docs/PATTERN_VALIDATION.md
[baseline]: ../../docs/DECOMP_BASELINE.md
[live-driver]: ../../tools/port_live.py
[cleanup]: ../../docs/agents/run-cleanup.md
