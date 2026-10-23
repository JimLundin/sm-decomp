# Engine reduction continuation - 2026-10-23

Read [PLAN.md](PLAN.md) first and use [ITERATION.md](ITERATION.md). Preserve the
complete game and exact behavior while reducing code, extracting content as
validated data and eliminating all unsafe operations. Item 4, retiring bank ownership by gameplay responsibility, remains active. Read
[the architecture migration](target-architecture/MIGRATION.md) before changing
subsystem boundaries or ownership.

## Current implementation

Active sprite iteration: room-session/next-sprite-session. Production overlay
PRODUCTION-HANDOFF.md/PRODUCTION-OVERLAY.json is frozen:32files,4new,0deletions.
Candidate candidate-511bd03d has production+composed Rust,121comparison adapters,
original/candidateDraygon modules and observer hooks,39mutation runner. First
wholeproduction build PASS42.97s source2782a16a; detachedcompile1failed private
write_sprite_bytes, pub(super) correction compiles in detached2. New adapters
added afterward still need the full nextbuild. No local exe execution.

Baseline baseline-e7cca70c build4 plus alias-reference-1 passes121pool/controller
captures+5CPU captures oncae732b3 (10.57s), exactprior117+5parity. Prehook source
1567ae81 and observerreferencecec7e085 preserved withfrozenDLLs. Only expected
poolpanics8..11 occur. CPU5 event/phase/fullRAMfootprints independently audited.
Both earlymovie attempts missed allspriteactivity except clear; sourceinspection
ofactualCeres roomdata explains why. No failingframes but no spritecoverage.

Remaining beforeadoption: original/candidate52phase poolCPU lifecycle capsule
(pool-oracle-draft), production Runtimeguard (verification/runtime), allcandidate
comparisons, relevantsharedregressions, hosted30detached+9DLLmutations andfull
build, finalaccounting. Pooloracle finalsource fixes are in progress; no CPU
capsule has run. Original positiveDraygon46 Rust is UB and stays excluded.
All local compiler/capture/replay handles through53683 are terminal; new work
must first confirm currenthandles. Sprite source remains isolated/unadopted.

Atlas is published frommain511bd03d at https://jimlundin.github.io/sm-decomp/.
Hosted40027362601 passes browser+Pagesdeploy; liveHTTP200 bytes matchGitblob.
Graph snapshot is b0565b2d (samecurrentproduction e7cca70c); regenerate after
substantial sourceadoption andreview curated evidence anchors.

User reaffirmation: easy Super Metroid Map Rando integration is a core
architecture requirement now. Custom sprites/palettes, supplied room/door graphs
and explicit opt-in re-spin/QoL rules belong at content/world/gameplay-profile
boundaries. Vanilla remains the fidelity baseline; it is not a reason to defer
those design requirements. Accepted architecture, contracts, report and validation
now record this. Lead notified to audit visual asset assumptions without changing
current ROM provenance or inserting unrelated mechanics into the visual refactor.

Active visual iteration: next-visual-session/CONTRACT.md and declarations are
frozen after two independent reviews and root metadata compilation. Union:
51053distinctWRAM bytes plus16native; no overlaps. Existing512palette and438
animation outputs all match prior integrated-1references on c30c16ba/source21a7453a;
both guards pass,92.70s. Evidence reference-c30c16ba/results.json and frozenDLL.

The original29actual65816 selector2 captures match expected
D715carry, D717/D721branch and exactlytwo misc calls. Independent audit adds174
phase/effect/footprint checks. No execution of the mismatched old Rust callback.
Reusable capture source is adopted with this handoff: verification/
samus_palette_oracle.rs, cpu_infra registration, DLL export and source
registration.323verificationRust lines, unsafe+3 verificationfunctions, export+1;
production gameplay unchanged. Root fmt/check/direct build passes20.73s on
8d7f67b461472e538a6cb16580db21d135309b9f2678d022d7c83e16e95b668a.
All originalROM/code guards passed; no ROM bytes or captures are committed.
The latest oracle-results-2 extends the original29 with four explicit callback
injections at D712; old29bytes match exactly. IDs29..32 produce carries0001,
with exact phase/timer and conditional normal-copy observations. These injected
transitions are not a claim of naturally reachable SNES states.
All224 defined-Samus predecessor cases pass in samus-defined-reference-2 on
source97e4b367. Build1 corrected a nonexistent getter; capture1 found a compiled-out
DLL export, with no cases executed. Corrected modules/lib bridges/exports follow
the existing always-compiled convention; only production observercalls aregated.
CI now also builds the normal Cargo sm_verify example to catch configuration gaps.
Final main sourcecf34ec681c8952d3f77f2b538b1cc160838577859aab0fac3ca500b32952e28a
passes rootfmt/check/direct build19.59s and all257referencebyte comparisons
(224defined+33oracle,18.98s) in main-reference-results-1. This reference increment
adds331verificationRust lines; no production gameplay change. Unsafe2612,
exports15,no_mangle79; all new declarations belong to verification.
Local oracle-c30c16ba now holds the extended checked snapshot45f182a3;
oracle-results-1 retains the original29captures/results/frozenDLL.
capture_palette_oracle.py reruns the extended33 bounded cases.
Frozen candidate source f3840efe2167f37f5e3bd48c4063c575fa8448f18417b7fd7a589f4501878f91
at next-visual-session/candidate-1eb5bed9 passes root fmt/check/direct build,
three detached compiles,1174visual comparisons,33originalCPU comparisons,
three Runtime guards,1443enemy cases/87checks,1038room comparisons and
106player groups/56comparisons. Compared artifacts remain under candidate-*-1.
Oracle comparison excludes only fixture stack[1FEB,2000) and charge scratch
[24,26) forIDs21..24; candidate excluded bytes stay at entry values.
Hosted verify/visual-session-20261023 commit7f14e21/run40021042750 passes
formatting, allthree detached executions,18/18composed mutants and full Cargo
build. Hosted40021042750 is fully successful: all nine frozen DLL baselines
pass and all nine compiling DLL faults trigger their intended checks. Combined
mutation result27/27 (18detached+9DLL), independently log-audited. Final replay passes28314frames/all38probes/0failures,
120follow-upframes,97.49s versus94.68s baseline (one paired sample).
The first replay-launch script failed reading a UTF16 log, before gameplay;
encoding corrected without source changes. No failed gameplay frames.
Complete visual Session source is adopted on main with this commit.
All314tracked sm files are byte-identical to the frozen candidate. Rootfmt/check
and direct release build pass28.18s onf3840efe; session13604 is terminal.
Candidate library/defined adapter and source registry are preserved. Pointer
writer relocation is exact, not unsafe elimination. Main MapRando docs persist.

The final matching route uses38requiredprobes,28314frames,120aftermessage;
baseline94.68s/0failures on8d7f67b4 is in main-baseline-38probes.log.

Lead enemy_projectiles_implementation owns shared state/codec/runtime/callers.
Its foreign_edges worker owns palette, setup_callbacks animation, pre_callbacks
fullSamuspalette/sm90/sm91. Collision owns hostedSamus/oraclecases/Runtimeguards;
torizo_implementation owns detachedcompositionRust/README; torizo_review owns
both mutation runners, preserving existing7+9faults and adding11reviewed drafts.
Gameplay implementation remains isolated and unadopted. Root alone integrates/
builds; preserve complete caller/deletion scope. Do not adopt partial wiring.

Complete next-enemy candidate is assembled and independently reviewed at
Local/sm-decomp-experiments/20261023/room-session/next-enemy-session/
candidate-c8d728c2. Complete candidate is adopted in the commit containing this
handoff (parent dcccfede). Root fmt/check and direct verifier build
pass; main-adoption-build.log records29.83s on exactly the same source21a7453a.
All72 adoption paths match hosted4d6806b8 after newline normalization. Only
normalized-equal untouched files were reconciled to preserve source identity.
Five owner/adapter files and five getters retire; four old restoration bodies
share Inventory::restore through seven real caller redirects. Ownership is
interval-scoped, not persistent whole-game state.

Source8ef18286 passes1440 frozen enemy/restoration comparisons,3new native
invariants,87independent checks and1038room comparisons. Frozen results are in
candidate-comparisons-1/ and candidate-room-results-1/. Hosted40012943588 failed
before gameplay because case1058's synthetic image was too small. Independently
reviewed case-only expansion adds7verification lines, no production change.
Corrected source21a7453a40213efa46d98267074b1b9c8bc2f5de2792f46a32bb80052e617e70
passes final direct verifier build (candidate-build-4.log),106player groups,
56comparisons and2runtime guards (candidate-player-regression-final.json).
Build session23568 and replay session45393 are terminal. Candidate replay passes
28314frames, all26probes and120frames after completion with zero failures:
90.08s replay/90.30s total in candidate-final-movie.log. One sample versus97.27s
baseline does not establish a stable speedup. No local heavy process active.

Validation branch verify/enemy-session-20261023 at4d6806b8 passes GitHub
CI40013612299 fully: formatting, full build, composed execution,7composed and
9DLLmutants. All compile and fail their intended behavior checks, with passing
unmodified baselines. Each mutant must compile and fail the intended behavior check;
compiler failures, unrelated panics, crashes and timeouts never count as kills.
All16faults now have individual nodes in PROGRESS.md.

Combined baseline: main-baseline-movie.log, source dd073b2a...,28314frames,
26required probes,zero failures,120frames after completion,97.27s replay.
Previous restoration CI40009086639 passes. Root alone runs the local compiler/
capture/replay lane; standalone executable checks stay on hosted Windows.

Replacement/composition copies and final accounting are synchronized; full root
fmt/check, direct build, source equality and progress pass. No local heavy process
remains. Candidate is fully adopted; continue next visual-effects contract work.
Production delta is-804lines but+1934tokens/+3915LFbytes: no net source reduction.
Verification Rust is+1239lines after fixture correction; Python support+134.
Native source unchanged. Final accounting files remain beside the candidate.
Final production107459lines; verification12032; combined119491. Production
tokens+1934/bytes+3915; verification tokens+11157/bytes+47194; Python+134lines.
Unsafe2608/raw offsets21 unchanged; cast-containing lines+11 to5720 reflect
typed adapters/call spelling. No net source reduction or stable speedup claim.

Next design: complete shared visual effects (palette effects, animated tiles,
Samus palette control), not just two runtime wrappers. Lead prepares isolated
next-visual-session contract/declarations; four source-only workers inventory
palette, animation/upload, Samus behavior and comparisons. No gameplay edits yet.
Reviewer enemy_projectiles_review investigates sm91's unsound void-to-u8 palette
callback cast against bank/oracle behavior before any replacement. Retire both
private owners, duplicate player/inventory snapshots, the unsafe callback table
and raw color copies, plus animation's enemy-spawn trampoline in the complete
adoption. Broad palette aliases and remaining legacy/reset/CPU writers mean this
is still interval ownership. Freeze/review interfaces before implementation.

Restoration verification adopted: enemy IDs1382..1439,58actual-old cases pass;
20adjacent supplemental outputs unchanged and87independent checks pass. Build/
fmt/check/source registration pass on
dd073b2a550012b4cc8218a14ab5f82f320536aa5616b9a29c3834ad7b77bef9.
Evidence: next-enemy-session/restoration-reference/results.json and frozen DLL/
bins; restoration-build.log. Process21606 complete. New output lengths up to
131391bytes require the existing export's variable-capacity buffer. No ROM needed.
Eight cfg(sm_verify_dll) observers at four restore entry/returns are inert unless
armed; owned implementation moves them to legacy Runtime restore entry only.
Production algorithms unchanged;347 verification/registration lines +32hook lines;
two verification unsafe entry declarations. No executed restoration mutants yet.

Concrete enemy declarations metadata build PASS: next-enemy-session/declarations/
lib.rs -> enemy-contract.rmeta, declaration-build.log (four visibility/style
warnings only). Prose contract accepted; concrete review active with session_review.
Four implementation lanes await that review. Composition design independently
approved. Main getter CI40007332386 and adoption CI40007039503 both PASS.

Next enemy Session design contract accepted; concrete Rust interfaces and lane
ownership still being prepared by enemy_projectiles_implementation before source
split. Independent audits confirm48998+801+20=49819 disjoint WRAM bytes plus16
native spawn bytes. Restoration and shared/native composition verification drafts
are assigned separately to torizo_review and torizo_implementation. Restoration
cases append existing enemy supplemental carrier at1382 onward; no new export.

Reference refresh completed: next-enemy-session/reference-main1f0cb0a6/results.json,
1382/1382 outputs identical to integrated-2 references,87/87 independent checks,
133.85s; source b03f593e0eed142c9dc79a287ad269b33212881de84c05457a6844cd3438a451.
Frozen DLL and every captured bin retained there. Process1740 completed; no local
compiler/capture remains running. Main CI74948127=40007039503 and getter
1f0cb0a6=40007332386 remain running; poll exact runs, do not restart them.

Shared room/player Session adopted from7ba1e33a. Full hosted CI40005712881
passes build, composed execution and all eight targeted mutations. Formatted
source8a4ee812de0eaa7b25dcecd6877c2f08d6000aae282a8efbe257ba27d286d31d
passes1037/1037 comparisons (candidate-1037-formatted-results/results.json).
Earlier same-behavior source passes106player groups/56comparisons/two guards
and28314frames/19probes/zero failures. Source review found only import/layout
formatting changes between these builds. Production+890, verification Rust+673,
combined+1563; Python+34; unsafe declarations2606 (-7). Ten superseded files
retire. Legacy callbacks still require interval publication; no persistence claim.

Next focused draft: getter-validation passes identical54query capture against
unoptimized and optimized code, SHA b59dac4de27781371d9d709b96a7b0dbff7819afc5a6bc075fd63276bb38e02f.
Optimized source71b1fad86b22adba3bf4d222f9d143f8ce21669903c1a510a2f4d159aaade943;
movie handle81316 completed PASS28314frames/19probes/zero failures,96.14s;
no measured speedup. Room cases864..879 match candidate-1037-formatted-results.
Main rebuilt source b03f593e0eed142c9dc79a287ad269b33212881de84c05457a6844cd3438a451
passes those16 cases,54getter queries and106player groups (main-getters-results.json).
Case1037 and its frozen baseline are integrated into the maintained CI runner;
four mutation definitions and kill criteria unchanged. Final review and formatting
pass. Getter delta: production+7, verification Rust+93, Python+2.
Full enemy suite does not cover these
initializers; nested EyeDoorProjectile argument path remains unexercised.
Next enemy Session contract is assigned to enemy_projectiles_implementation,
design-only under next-enemy-session. Main/candidate are not its edit targets.

Adopted through e216f0ef: twelve new reference cases
1021..1032 pass, alongside seven unchanged adjacent cases. The two scoped
callback hooks retain actual message/native algorithms and compile out of normal
production. Normal-library metadata compilation passes; final release verifier,
full root fmt/check and19 exact outputs pass on source
be544523a6cd737e3daafefe013f5550ac6049b757b7209dd164668f9433c369.
Frozen initial reference source is
fad78a44223447f1c20ed504ec1b1eebd26d75587067d4583f737ed29e224b12;
final differences are descriptive assertion labels, not fixtures or behavior.
Evidence: room-session/main-reference1032-results/results.json,
main-reference1032-build.log and baseline-current-1032-results/results.json.

The finite room_mutations.py runner builds fresh baseline and three copies of
the actual scheduler on hosted Windows: repeated resumed pre-action, repeated
delay decrement and early613 publication. Compilation must pass; each mutant
must fail its named assertion and the matching carrier assertion, returning no
observation. Hosted run40001171992 passes both frozen baselines and all three
mutants, plus the full build, format and existing independent-core checks;
these faults must be retargeted to actual typed code during replacement.
Incremental cost:369 verification/registration Rust lines,12 verifier-only hook
lines in game files (zero normal-build hook calls),181 Python lines and4 CI lines.
Normal production algorithms are unchanged. The two extra unsafe declarations
are verification entry points for the existing native message and drawing calls.

Complete shared-state candidate: room-session/candidate-e216f0ef. Build4 and
1037/1037 comparisons pass on source
0b5ef619789d78be13116549d652aca87edb5596d393441444dde47f450f0a31;
evidence: candidate-build-4.log and candidate-1037-results-2/results.json.
Comparison caught reversed D830/D870 query/mark maps; all four branches were
corrected. Cases1033..1036 independently distinguish both maps and outcomes.
Old-reference capture and review pass (reference1036-results/results.json).
This increment adds38 verification Rust lines and a reviewed fourth flag-query
mutation to room_mutations.py; hosted execution remains pending. Candidate
runner uses its actual typed query helper. Composed host compilation passes.
106 player groups,56 comparisons and both palette/animation guards pass:
candidate-player-regression.json. Candidate movie passes28314frames,19probes,
zero failures and120frames after message completion (candidate-movie-1.log).
Same-route baseline passes:88.18s versus candidate94.81s, a single sample +7.5%.
Getter aggregate cost investigation is assigned; candidate remains frozen.
Validation branch verify/shared-session-20261023 at7ba1e33a has hosted composed
execution, full build, format and four composed mutants PASS in40005712881;
four candidate room mutants remain running. Main flag-map verification commit
dcb97630 CI40005395335 passes all four room mutants and frozen baselines.
Full format changed only import/layout in two candidate verification files;
Local frontend build was blocked by Defender event1121 for khronos_api build
script, ASR01443614-CD74-433A-B99E-2ECDC07BFC25; preserve the block and use CI.
Formatted direct verifier build5 passes on source
8a4ee812de0eaa7b25dcecd6877c2f08d6000aae282a8efbe257ba27d286d31d.
Five-getter optimization is assigned to an isolated getter-optimization draft;
do not overlay while candidate validation runs. Independent design review passed.
Production+890lines, verification Rust+673 including four formatting lines,
mutation Python+34; no reduction claim. Adopted as recorded above.
Native source line endings match installed archives after normalized equality.

Room contract v2 is frozen and four isolated production lanes are active under
enemy_projectiles_implementation. Outputs: room-session/replacement; ownership
and exact paths: room-session/contract/IMPLEMENTATION.md. No candidate runtime
adoption was pending at that checkpoint. Session/caller/verification reviewers work separately; root alone
builds and captures.

New actual-old message case1020 passes: real dud pre-action clears initial
counter, suspension injects5500/delay13, completion produces5507/delay13 with
613zero. It detects repeated pre-action or timer work on resume; proposed
mutations still need execution. IDs880/1009/1018/1019 retain exact captured
outputs (ownership tests include intentional caught panics). Release DLL build,
full root format and final source hash all pass. Source
14ad11246e7fdaf2816bad4191ac214f8c66eaf9e36d68d94f630bd11a298d49;
DLL0bf5e36b2c9e8ce5980cc9121e0a6b14acd9b07f09bc54160bd61a6d0609fb9a.
Evidence: room-session/baseline-current-1020-results/results.json and
baseline-current-1020-build.log. This change adds27 verification Rust lines,
zero production lines; no new gameplay replay needed for the harness-only edit.

Next room reference set: room-session/baseline-026ab1a2/results.json captures
1020 IDs in101seconds:1017returning cases,1012/1013 expected exit2,1014 expected
nonreturn with marker. All overlapping historical baseline1018 outputs match.
Frozen DLL SHA25668b9c9f39f28c52a406573c14076f50dc374231b088f2accd8f91f40df5ecdae,
source8da52a46f05b73235b04623499f5b1c92b13da9e973c39c4ed3b966cb80180a1.
This reuses the verified player-production build; detached fixture-only changes
do not change mechanism source. It is not claimed as a current-source rebuild.
Contract files are at room-session/contract/: rustc metadata compilation and
independent ownership/alias/drawing review pass for frozen v2.

Hosted run39996910420 failed the detached fixture's first collision assertion:
center43 plus motion4:1000 has leading edge48, already touching the wall.
Both reviewers traced this to the expectation, not a production change. The
revised fixture starts center41 and checks two ticks; original center43 remains
as an explicit first-tick impact and stationary-playback test. Full root format
and standalone compilation pass; hosted run39998215217 passes all checks.
The player typed core is adopted on main through36470e60. Actual production
source remains unchanged from026ab1a2, so recorded gameplay evidence below still
applies to that source. The correction adds41 verification Rust lines; totals
are production-1937, verification+1109, combined-828 Rust lines. Prior token/byte
figures below describe026ab1a2, before this harness correction.

Mutation testing is required by PLAN. The adjacent mutations.py compiles an
unchanged baseline and two isolated production faults (ignore beam radius;
skip impact retirement). Each must compile and fail a player_checks equality
assertion; unrelated failures do not count. Hosted run39998215217 confirms the
baseline passes, missing-radius is killed at player_checks.rs:106, and skipped
retirement at line60. Execution remains hosted only. Python mutation support
adds83lines and CI wiring7lines, separate from Rust verification accounting.

The adopted player typed core's locally compared source
8da52a46f05b73235b04623499f5b1c92b13da9e973c39c4ed3b966cb80180a1 passes full
formatting, release/frontend/verifier build, standalone compilation, existing106
groups and all56 new full-RAM/status comparisons. Adjacent mechanism ownership/
qualified checks and palette/animation runtime guards pass. Final source review
accepts exact core/path/registration/probe changes.

Speed Booster from boot stops after11549frames, zero failures, all12 required
probes plus two follow-up frames. First preparation/history2218; motion/flight/
playback/draw2308; firing/initial collision8654; terrain/explosion8659; reset11546.
The route includes one gameplay room and one enemy definition, no credits.
The prefix includes scripted-sequence execution; do not claim broad game coverage.
Independent host composes actual player/animation/palette algorithms with one
palette owner and fixed prepared terrain/content; unsupported dependencies fail
explicitly. Local executable not run under the retained Defender restriction.

Pre-fixture-correction price: production -1937lines/-18470tokens/-70009LFbytes; verification/support
+1068/+9554/+34899; combined maintained Rust -869/-8916/-35110. Detached harness
is697 of the added verification lines. Fifteen outer singleton layers and old
owner bindings retire; enum/shared geometry remains for real consumers. Cargo/CI
support is separate. Final accounting: projectile-session/main-final-source-accounting.json.
Evidence: integrated-build-1.log, integrated-56-results/results.json,
integrated-1-adjacent.json, integrated-1-oracle.log, detached-integrated-compile-1.log.
Persistent whole-game ownership and remaining unsafe effects are unfinished.


Enemy-projectile owner and native ROM ownership are adopted on source
8594f83d2cca0e79fc345e5a7e83bd50ea8e6fb09b87312432206acbf0f52a5c.
GitHub run39993911476 also passes full build and formatting.
Final full formatting, release/frontend/verifier build, all1382 full-memory
comparisons and87 independent checks pass. Independent final source review
passes. Adjacent106 player groups, mechanism ownership/qualified checks and
both palette/animation runtime guards pass after native binding migration.

Speed Booster from boot stops after18216frames with zero failures and all seven
required probes plus two follow-up frames. Spawn first8323; pre/draw/frame8324;
command8325; nonzero motion18120; native read18213. Prefix is setup. Six rooms,
three enemy definitions, no credits. The earlier28314-frame attempt had zero
frame differences but FAILED the missing-motion probe; collision movement
bypassed the original helper probes. Final probe sits after successful nonzero
collision fraction/position writes. Do not count the earlier attempt as a pass.

The owner retires sm_86, eproj_script, enemy_drops, old contact/attachment
operations and actual callers; it covers801 RAM bytes plus16 native spawn bytes.
Game.g_rom is removed. Captured native RomImage records preserve distinct native
and loaded allocation provenance; remaining legacy pointer consumers derive their
pointer from the owner. Production remains interval-owned with explicit legacy
unsafe effects; no persistent Session or all-unsafe-elimination claim.

Final price against11653c1b: production+3490lines, verification+2160, combined
+5650. Includes338 lines of genuine consolidation and70 lines of native ownership
cost versus earlier drafts. Rust117396; unsafe declarations2611 (-114), counted
cast lines5725. No net reduction claim. Native ownership's57 earlier independent
checks expanded to87 with lavaquake/turret expectations, not old undefined runs.

Evidence: enemy-projectiles/integrated-2-results/results.json,
integrated-2-qualified.json, integrated-2-adjacent.json, integrated-2-oracle.log,
integrated-build-2.log, main-integration-price.json. Corrected old respawn622..625
use A2804C initializer; source-audited1358 unaffected observations retained.
Supplement20 references are under old-supplement-20-results; rejected fixtures
remain preserved separately. All artifacts stay local.

Next: player-projectile complete typed-core conversion includes5294 affected
production lines plus actual flare decisions and twelve singleton binding layers.
Contract reviewed; producers, flight and presentation/runtime work are isolated.
Expanded44 old references captured at projectile-session/expanded-44-results;
case43 intentionally panics, others do not. Snapshot baseline-11653c1b build3
passes after proven line-ending-only native inputs were aligned with installed
archive source bytes. Extra12 presentation/reset cases are captured on source
4a762fa17b7924d45a89fc403827000fd153ad406f439bbe20ab77696667bfea; case7
intentionally panics, all other cases return normally, no diagnostics. Results
are in projectile-session/presentation-reset-12-results. Build2 passes after
adding the missing source registration. Original44 inputs/results unchanged.
No independent-core/reduction claim until complete adoption is checked.

Composed animation/palette core354f1fb5 is adopted in main after GitHub
run39989858734 passed independent execution, full build and formatting. Source
`fe7c93f85d6d98e706096f001b3028b40bfcf1d5c33bc9c27e9b55c5b9e1af15`
passes full formatting, release/frontend/verifier builds, standalone compilation,
438 frozen animation comparisons, actual animation and palette runtime guards,
and the 20947-frame Speed Booster prefix with zero failures. All five required
animation probes hit (spawn20857, upload20894, command20944), plus two follow-up
frames. Prefix is setup, no credits. Final independent source review passes.
Hosted independent execution passed. Do not launch the standalone executable
locally under the retained Defender restriction.

Animation core uses ordinary state without Game/Port/RAM modules; the composed
host invokes the actual palette core synchronously with one current-colors array.
The production compatibility guard owns Option<AnimatedTiles>, publishing and
relinquishing it across legacy effects. No persistent whole-game Session claim.
Production +124 lines/+1142 lexical tokens/+4298 LF bytes; verification
+587/+5377/+20098; CI/Cargo +13/+132/+663; total +724/+6651/+25059.
Rust111746; unsafe declarations2725 unchanged, no_mangle74 (+1 guard export).
Artifacts: Local/sm-decomp-experiments/20261023/animated-session/integrated-1*,
detached-compile-1.log and integrated-build-1.log.

Enemy comparison-2 is now verified for its594-case scope: old and new outputs
match, and first354 old outputs match frozen baseline-1. Source
`0b814eceb2164f3cf426c1b75030d45fdd2809a08fd2f378b77f6d352121243b`;
results in enemy-projectiles/comparison-2-results/results.json. Headless library
check and direct production DLL build pass; no frontend or full adoption claim.
Old1362 source a10c9aa4a8b949b4eac59444b815473df83a06158339e0d3d84ae079f182af0f
passes its direct DLL build; capture is running in old-1362-results. Candidate
native-ROM owner/simplifications and qualified checks remain unverified drafts.

Animated tiles adopted on source
`23fd5fb6a109c2b6bc6105e52523a1ebb579200abb14f0dff6f61935844fccb2`.
Full cargo fmt/check, release/frontend/verifier build, independent review and
438 frozen comparisons pass. Speed Booster from boot: 20947 frames, zero failures;
spawn first 20857, frame/upload first 20894, command 20944, two follow-up frames.
All five animated_tiles probes hit. Production +199 lines; verification +35;
combined +234. Rust total111035; unsafe declarations2725. Frozen inputs/DLL and
capture results are under animated-tiles/integrated-1*. The successful replay log
is integrated-1-oracle-2.log; the first attempt had invalid CLI spacing and did
not execute. Build/source unchanged through all successful checks.

Next: isolated animation/palette composition removes animation execution's
Port/Game dependency and invokes the actual palette core with one colors owner.
Enemy-projectile typecheck-1/check-headless-3.log passes library source checking
including the candidate354 adapter; no candidate runtime equivalence yet. Full
adoption-1 caller draft is unverified, production +3757 lines before further
review, not a reduction. Expanded reference drafts remain uncaptured.

Palette core 601f8da3 is adopted in main after GitHub Windows run
39986571000 passed standalone execution, frontend build and formatting. Source
`6d2231c8c7324cdd6ed5fd26e4c309eff23b11651b983203df147f5437bcd621`
passes release/frontend/verifier builds, all512 frozen comparisons, the actual
runtime guard checks,15 actual animated-tile palette-spawn cases and the272/0
opening replay with all7required probes and120follow-up frames. No gameplay rooms.
The standalone target is `sm/src/verification/palette_detached.rs`; CI compiles it
directly with rustc before provisioning native dependencies, then runs it on the
Windows runner. Local ASR-blocked standalone execution remains blocked, not passed.
Production ownership remains interval-scoped; persistent whole-game Session is unfinished.

The complete replacement removes palette's address-based execution/Port binding.
Pure state/core/content are separate from runtime compatibility. Full cost vs
13bca2c3: +227production lines, +328verification-support lines, +13CI/Cargo lines.
Price and local build/comparison/guard/replay evidence are in palette-session/.
Shared transport remains for other systems; no global persistent Session claim.

Parallel direction (user request 2026-10-23): three original paired lanes, now with palette complete and four
additional enemy-projectile workers, with one root compiler/integration lane. Ordinary typed state and an owned
Session are the destination; wrapping existing address-based ports is insufficient.
PROGRESS is now primarily a Mermaid graph: green delivered steps, orange drafts,
grey remaining milestones, purple verification/integration. Keep its node states
current rather than expanding the prose.

Isolated work under `Local/sm-decomp-experiments/20261023` (not adopted):
- `enemy-projectiles`: reviewed 801-RAM-byte +16-native-byte state projection,
  private shared-transport import/export extension staged only in the draft,
  direct typed state/motion and full closure contract. Remaining implementation
  includes roughly 310 callback identities, drops/contact and attached-body
  producers. The 354-case core reference is now captured, all131099-byte outputs;
  expected panics10/11/12/16/347/348/349 and diagnostic8 observed. It does not cover
  the full family scope. Two additional workers now handle fixed setup/pre-action
  interfaces. A separate unbuilt family_cases.rs adds92 prospective cases and
  requires a2MiB synthetic image; do not treat it as frozen reference yet.

Old reference carrier build/capture completed on source
`108fa09ab7eb1a229a7b5a041d653eebf5855cbf1d7faf254db94b2723665601`.
Both lanes have baseline-1.dll, baseline-1-inputs/, baseline-1-source.json and
baseline-1-capture.json plus per-case baseline-1-N.bin files. Animated outputs
are413x131076bytes, one262148-byte repeated observation, and24x198659-byte hardware
observations. Enemy outputs are354x131099bytes. Captures ran independently in
separate processes with source fixed. Local combined build log is under
animated-tiles/combined-baseline-build-1.log. No capture/compiler process remains.

Next: adopt/refine typed candidates only against these frozen old references.
Palette is adopted and verified as described above. Animated tiles are adopted with the checks above.
Enemy-projectile source checking and caller closure remain in isolated drafts.
Current Rust is 110801; palette adds 227 production and 328 verification lines,
plus 13 CI/Cargo lines. Keep the local standalone ASR block intact.


Verification carriers are now physically grouped in `sm/src/verification`.
The existing logical modules and private access remain unchanged through path
registrations. Four files are byte-identical after normalization; the room-copy
test body is identical with a private parent import and re-export. This is
organization, not production reduction. Registration/imports add 13 support lines.
Current total 109412 Rust lines: 4266 in dedicated verification files and 105146
elsewhere (the latter still includes mixed verification support).

Source `e279c7b426e060bd45f58ab5b83a8aa23cea7ef23b8d6d1121c29625c042a8b2`
passes release/frontend/verifier builds, 106 projectile groups, all three
room-copy cases, mechanism ownership/qualified checks and a frozen normal output.
Source correspondence proves no production behavior changed; no gameplay replay
was warranted. Local evidence is under `20261023/verification-layout`.

Public CI repair: `64c450f2` passed GitHub Format, then failed because Ubuntu
did not provision native archives. CI now separates Ubuntu formatting/progress
from the supported Windows GNU frontend build, with all seven native archives
rebuilt from pinned sources. A backend CRLF checksum correction preserves exact
upstream content. Workflow lint and remote run 39979717734 both pass.
The manual verify.yml workflow remains separate and has not been migrated.

Earlier formatting change: repository-wide default `cargo fmt`; scoped formatting had left
GitHub's `cargo fmt --check` failing. The full check and release/frontend/verifier
build pass locally on source
`d21e2bb348caf345714758514ae94830749863a57e505b62901eff9dcf085cdf`.
Formatting affects 75 Rust files and adds 229 layout lines; source is now 108903.
This is neither new engine behavior nor an implementation-size savings claim.
The local native-config library needed regeneration and its linked SDK copy was
refreshed; source checks were preserved. PLAN/ITERATION now require CI's exact
full formatting command before Rust commits.

PaletteEffects is now adopted in main: four public operations, eight slots,
13 caller files, complete sm_8d/palette_ram retirement. It uses the existing
private ownership transport and remains interval-owned. Final source:
`6e224f28e382b7caf9c9527c099ef067633ce098c9d83434cfea6a149fb1c171`.
Release/frontend/verifier builds pass; all 512 frozen old observations match.
Any% inputs 0..271 pass 272/0, seven required palette probes, 120 follow-up frames;
no gameplay rooms or credits. All three comparison exports pass null/short/no-write,
exact known case and invalid-ID checks after sharing their copy implementation.
Independent review passes. No compiler/capture/replay session remains active.

Evidence: `Local/sm-decomp-experiments/20261023/palette-effects/` contains frozen
`baseline-2*`, `integrated-1.dll`, `integrated-1-inputs`, `integrated-1-capture.json`,
`integrated-1-exports.json`, `integrated-1-oracle.log`, and integrated-build-1.log.
The baseline fingerprint is
`b166bf625cfd98f304ab431a87cbc42f28d8e6f72a703a17d0b64b9d7c0ca7b8`.

User clarified accounting: exclude verification harness/support from production
engine reduction, keep separate intentionally and disclose both counts. New
palette carrier is `sm/src/verification/palette_effects.rs`. This adoption adds
126 production lines and 370 support lines (496 total); no savings claim. Current
Rust total 109399 includes verification; unsafe declarations 2728. Reconcile a
production-only repository total before applying the revised reduction measure.
PROGRESS now has milestone/status tables; update those as work advances.
Next responsibility remains to be selected under PLAN, with actual duplicate
production algorithms/deletion and ownership dependencies identified first.

Earlier change: shared ownership transport in `game/ownership_transport.rs`,
privately registered under projectiles and room mechanisms. Actual duplicate
transport algorithms are removed; owner codecs stay separate. Final source
`06c88508786a98d3bda75f0bfbb1b2a33d65398ac3783f7bffb81f8f2096667d` builds and passes
106 projectile groups, 104 fresh full-RAM comparisons, 1018 frozen mechanism
outcomes, strengthened ownership/qualified checks and 28314 oracle frames with
zero failures. At that milestone Rust was 108674; full delta -8 lines / -5 tokens / -133 LF bytes.

Local `20261023/shared-ownership` contains `transport-build-1.log`,
`transport-1.dll`, `transport-1-inputs`, `transport-1-capture.json`,
`transport-checks.json`, `transport-oracle.log`, `transport-price.json` and 104
fresh `projectile-before-*.bin` references. No compiler/runtime check remains live.
Read PLAN for the demonstrated alias/persistence limits.

Earlier adoption: complete room mechanisms in `sm/src/game/room_mechanisms`.
Creation/setup, rules, all 110 command identities, scheduler, drawing/graphics,
message suspension and actual callers are integrated. All 51 superseded files
are deleted. Scope and remaining ownership limits are in PLAN and PROGRESS.

Verified source: `88265b3ed879c62fb219b6829703f9560c56cc6995def849bb9c44e6102569cd`.
Release/frontend/verifier build passes. Frozen baseline IDs 0..1017 pass: 1015
normal byte-identical outputs, two exact diagnostics and one bounded nonreturn.
New IDs 1018 (ownership) and 1019 (qualified commands) pass; 106 projectile groups
pass. Speed Booster inputs 0..28313 pass, zero failing frames, required creation,
terrain/drawing/gameplay and message completion probes plus 120 follow-up frames.
Completion occurs at 28193; no credits coverage. Native pose effects remain qualified.

At the mechanism milestone, source was 108682 lines. Integration is -1154 lines; baseline preparation
was +926, so the complete mechanism effort is -228 lines/-4214 tokens/-6464 LF
bytes. No new dependencies or maintained native/tooling source. Do not report
only integration savings. Next investigation concerns persistent ownership's
external writers, adoption/reset/load and shared continuation blockers.

Local evidence: `Local/sm-decomp-experiments/20261023/room-mechanisms/` contains
`baseline-1018.dll`, its outputs/manifest, `integrated-2.dll`, source overlays,
`integrated-2-capture.json`, `integrated-2-new-checks.json`,
`integrated-2-projectile-checks.json`, `integrated-2-oracle.log`,
`integrated-2-accounting.json`, and `integrated-4-build.log`.
Do not overwrite references. No compiler or runtime check is left running.

The earlier room-copy correction remains in production; its separate +89 lines
are excluded from the mechanism accounting. Its old checkpoints may reject the
new runtime fingerprint: never bypass compatibility checks. Regenerate if needed.

The following projectile figures describe its earlier milestone, not current totals.

Production projectile consolidation is complete. game/player_projectiles now
owns ten shots, eighteen trail pairs, weapon/charge controls and launch history
through one private 391-word / 782-byte Transaction. Actual player, cinematic,
door, enemy, scene and message callers use it. Fourteen complete adapters and
superseded bank bodies are deleted. Speed Echo (90D4D2) and suit pickup (90EFD3)
execute their real motions.

Canonical, broad, odd and signed accesses use the same algorithms and byte-lane
resolver. Foreign calls publish and relinquish ownership, then import returned
RAM. Ownership does not persist across native gameplay, frame return, reset/load
or oracle intervals. Drop preserves successful mutation prefixes during unwind,
not before panic hooks or abort. Checked word writes publish the low byte before
high-byte failure; atomic slices retain preflight behavior. Signed arithmetic,
evaluation order and native-versus-LE reads remain significant.

Flare tables read loaded_rom; sequence bytes use captured g_rom, which can
differ. Native content capture/read APIs retain explicit unsafe assumptions.
Persistent ownership, backend independence, all-unsafe elimination, reduction
and boot-to-credits coverage remain unfinished.

Projectile milestone source fingerprint:
24bd3d424dea880173b5ae8ebaa9b25dd4fa9f1d0e57696deaac1621bda3e8b7.

- Release library/frontend/verifier build passes in 50.7995 seconds.
- 106 maintained groups pass. All 104 synthetic old/new scenarios compare all
  131072 RAM bytes directly, with zero differences; two groups exercise ownership
  and publication. Checks plus direct comparison take 0.8045 seconds.
- Three expected caught panics cover bytewise bounds, slice preflight and foreign
  callback unwind. Producer cases cover actual launch/motion, impact, expiry and
  Power Bomb DMA configuration. No-op family cases alone prove no flight.
- Oracle inputs 0..8779: 8780 frames, zero failures, exit 0, 24.70 seconds replay.
  Seven required probes all execute, followed by 120 frames: weapons 1649,
  history 1900, flight 85, playback 42, projectile draw 135, explosion draw 22,
  trails 3604. Last first hit is input 8659. Coverage is 427 gameplay frames,
  one room DF45, enemy E23F, no credits. Draw entries alone do not prove emitted
  OAM tiles. Synthetic cases are not exhaustive.
- Independent implementation/caller/provenance/outcome/accounting review approved.
  Formatting passes for all 19 new module files.

Main source: 108821 Rust lines / 226 files. Net increase is 1389 lines /
11644 lexical tokens / 43988 LF UTF-8 bytes: runtime/probes +596 lines,
maintained verification +793. Unsafe declarations 2767 (-22); cast lines 5892
(-26); no_mangle 67; C ABI metric 7. External retained reference/fixture source
adds 136 lines / 4316 tokens / 11355 bytes; initial ownership experiment adds
478 / 5029 / 20068. Complete incremental authored source is +2003 lines /
+20989 tokens / +75411 bytes. Staging copies count once. Archiving the rejected
experimental Session earns no production deletion credit.

Reproduce with the Windows environment in ITERATION.md:

```text
python tools/verify_dll.py --build-only --frontend-check
python tools/verify_dll.py rom/sm.smc movies/speed-booster.inputs --cargo --no-build --frames 8780 --after-hits 120 --require-hit player_projectiles.prepare.weapons --require-hit player_projectiles.history.finished --require-hit player_projectiles.flight --require-hit player_projectiles.playback.frame --require-hit player_projectiles.draw.projectile --require-hit player_projectiles.draw.explosion --require-hit trails.update
```

Maintained checks use the existing source-checked DLL workflow:

```python
import ctypes, runpy
from pathlib import Path
v = runpy.run_path("tools/verify_dll.py")
root = Path.cwd()
host = v["load_host"](root / "target/verify-dll/sm_verify.dll")
v["check_source"](host, v["source_hash"](root))
v["check_library"](host, root)
host.sm_verify_player_projectiles.restype = ctypes.c_uint32
assert host.sm_verify_player_projectiles() == 106
```

sm_verify_player_projectile_ram(id, output, capacity) copies 131072 bytes for
IDs 0..103, or returns zero on failure. Direct old/new comparisons are recorded
locally in consolidation-main-byte-comparisons.json.

Evidence root:
C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/runtime-impact-working/.

Final artifacts: consolidation-main-final-build-2.log,
consolidation-main-final-checks.json, consolidation-main-final-checks.log,
consolidation-main-byte-comparisons.json, consolidation-main-final-oracle.log.
Old observations are in consolidation-old-observations.json (0..8),
consolidation-matrix-observations.json (9..98) and
consolidation-end-observations.json (99..103), with corresponding raw
consolidation-{old,matrix-old,end-old}-ID.ram files. These stay local.
consolidation-parent-766c6246-sm-src/ preserves old main;
consolidation-before-source/ preserves the old experimental composition.

The prior charge-palette 60-suite/59-vector result and nine fatal pairs remain
historical evidence, not new integration coverage. Its artifacts, 815 verification
freeze entries and 284 runtime/338 harness embedded inputs remain unchanged.
consolidation-palette-reference-sm-src/ reconstructs its exact 341-file
composition; original archives and prior handoffs remain preserved.

## Paused launch draft

The reduction agent wound down on 2026-10-22 and paused its ongoing goal.
Its Rust-migration child is completed. No compiler, check, helper or tool session
was left running by that task. Resume only after a new implementation request.
The coordinator found one other running project thread; the inspected older
verification, reduction, architecture and cleanup threads were already inactive.

Resolved in the 2026-10-22 restart. The original three-file draft, based on
`e3fadf56`, remains preserved locally:

- `sm/src/game/launch_motion.rs`: shared beam speed data/selection and aim decoder.
- `sm/src/game/launch_motion_ram.rs`: shared decoding after subpixel clearing.
- `sm/src/game/projectile_flight.rs`: checked beam speed selection.

The adopted main change adds 13 lines; synchronized owned `launch.rs` removes
22 lines, giving -9 complete composition lines, -166 lexical tokens and -509
normalized text bytes. Existing local comparison code adds 167 lines, including
the independently frozen old decoder; including checks gives +158 lines.
No dependency or maintained native/launcher code is added.

In `runtime-impact-working/launch-draft/`, the three production drafts,
`owned-launch.rs`, `main.patch` and `source-hashes.json` preserve exact edited
bytes. The restart synchronized main, working `launch.rs`, local launch-motion
modules and `runtime-source`.
`launch-parent/` preserves pre-draft production,
owned launch, harness/preparation and relevant previous composition inputs.
The local `WIND_DOWN.md` records the historical dirty stop point; this section
supersedes its draft locations. Preserve existing caches,
backups and results; `impact-composition/` now includes this launch change.

Use `prepare_flight_control.py` and selected exports `owned_launch_experiment`,
`runtime_launch_experiment`, `flight_experiment`, `runtime_beam_experiment`,
`runtime_flight_compat_experiment` and `runtime_session_experiment`.
The launch checks pass 395 owned cases and 1,070 RAM/owned runtime cases,
including invalid diagnostics, kind bounds, captured selection, odd/broad
indices, fraction/direction aliases and live history rereads. The old decoder
matches pre-change Git source and is separately compiled; production diagnostics
stop at the native assertion boundary, while owned fixtures exercise returning
diagnostics. No movie runs or full-composition runtime comparison are claimed.
Final production and prospective Rust 2024 headless release builds pass.
Results: `launch-final-results.json`, `launch-final-check.log`,
`launch-main-source-check.json`, `launch-production-build.log`,
`launch-composition-price.json` and `launch-composition-build.log`.
Missile acceleration, extension creation and placement are now shared; remaining consumers
and runtime authority transfers are next.

## Local work and validation

Experiment root, outside Git:
`C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/`.

**Reuse `runtime-impact-working/`.** Its name is unchanged to preserve the
working cache. Do not copy an older carrier over it. Current files:

- `prepare_flight_control.py`: extends `prepare_impact.py`, exposes carrier
  entries and registers the shared controller and independent reference.
- `check_flight_control.py`: checks compiled inputs and runs selected exports.
- `runtime_flight_control_checks.rs`: saved production versus current RAM flight
  and owned extension contact using actual trail/room code, constructed content
  and controlled room hooks. Of 1,044 cases, 600 are new extension comparisons,
  divided evenly between the RAM and owned paths against the saved reference.
- `extension-final-results.json`, `extension-final-check.log`:
  final comparison results.
- `main-source-check-extension.json`: exact old macro/wrapper correspondence to
  `ba0d94c1`, current compiled inputs and unchanged launch/acceleration/spawn tails.
- `extension-production-source.json`, `extension-production-build.log`: normal
  release library/frontend/verifier build and loaded DLL source checks.
- `extension-composition-price.json`, `extension-composition-sources.json`,
  `extension-composition-build.log`: complete size/token accounting and prospective
  headless Rust 2024 build. `price_extension.py` reconstructs the changed
  composition files. The working `impact-composition/` now contains this extension
  composition; the older terminal composition is frozen in the stage below.

The independent `flight-parent/` backups include production flight at
`de9bf05e`, `harness.rs`, `flight.rs`, `flight_model.rs`, `encoded_flight.rs`,
`runtime_external_flight.rs`, `native_flight.rs`, `flight_checks.rs` and
`runtime_flight_compat_checks.rs`. Do not overwrite them. The saved production
file was checked against Git; the compiled reference changes only visibility.
The compiled new controller is byte-identical to main, and the production RAM
adapter differs only in visibility needed by the carrier.

`extension-parent/` saves source and results from the preceding `ba0d94c1`
stage, plus the changed prospective composition files. Combined with unchanged
`runtime-impact-stage/` inputs, it supplies the previous full price/reference.
Do not rerun the old `price_flight_control.py`: it expects the deleted axis
wrappers and would overwrite the wrong composition/results.

The current owned changes are in `flight.rs`, `flight_model.rs`,
`encoded_flight.rs` and `runtime_external_flight.rs`. The one-time
`adopt_flight_control.py` has already run; do not rerun it over those edits.
Context `advance` deliberately calls `advance_with` to recheck canonical
admission after a trail callback can change the selector. Broad-selector
fallbacks transfer the complete owner; native providers retain generic Direction.

Latest final result vectors:

| Export | Result |
| --- | --- |
| runtime_flight_control_experiment | [1044,116,96,4] |
| flight_experiment | [529,848,42] |
| runtime_flight_compat_experiment | [446,14,11,2] |
| runtime_flight_experiment | [92,1594,4,1,247] |
| runtime_collision_experiment | [196,535,172,1] |
| runtime_session_experiment | [170,1823,53,97] |
| runtime_beam_experiment | [338,817,32,52,3] |

All 143 runtime and 201 draft compiled inputs match. Production release and
loaded DLL checks pass (60.58s build). The prospective headless Rust 2024
library builds in 42.78s, as recorded in `extension-composition-price.json`;
this supplies no runtime or movie comparison of the full composition.
No movie ran. Synthetic content, controlled hardware/callbacks and these
selected actual-code comparisons do not establish full-game fidelity.

The inherited carrier remains `d03f819d` plus explicit adaptations, edition
2021, not the full current engine. Production is edition 2024. Only five runtime
files differ from the frozen impact carrier: lib/verify registrations, current
flight, the shared controller and independent reference. Earlier source
correspondence uses audited migration `e88ff8da` and pre-migration `49846692`.
Always check the final compiled inputs after changing preparation.

`runtime-impact-stage/` remains the frozen previous terminal stage, including
its results and 110,602-line prospective composition. Earlier room results
remain in `runtime-room-stage/`. Those historical passes do not validate future
source edits. The current continuation and unchanged frozen inputs are needed
to reproduce the local owned work; a fresh Git checkout alone lacks that work.

## Next dependency

The complete RoomMechanisms proposal and linear foreign-call ownership contract
are reviewed. The 1018-case final supplement is captured on source
`e6101bac42ec1e57472a46c51dd1f378fd1bd8f2f813221c4b1f27c9404628db`:
1015 normal outputs, two exact native diagnostics and one bounded nonreturn case.
Artifacts in `20261023/room-mechanisms`: `baseline-1018.dll`, `baseline-1018-inputs`,
`baseline-1018-capture.json` and binary outputs. All prior 1009 outputs match their
references, including the original 90. Each normal output starts with
131072 RAM bytes, then result kind, little-endian return word and diagnostic byte;
setup cases append displacement, message cases append per-call hardware traces.
All cases use isolated processes; message buffers need 32 MiB capacity. IDs
1012/1013 must exit 2 with the B3E3/Unreachable assertion; 1014 must be run with a
timeout and is not an ordinary capture. Its RAM write prefix is not observed
dynamically. IDs 1009..1011 exercise actual outer door-message continuation;
1015..1017 cover displacement aliases crossing owned-field boundaries.

Continue the full replacement in `owned-draft`, using its reviewed state/port and
common Fields/Execution pattern. Remaining checks cover qualified old-undefined
commands and final combined caller/ownership behavior. No replacement is
adopted yet. Follow PLAN's full caller/deletion scope; do not restart the archived
Session or isolated shared-consumer queue.

Keep these verified ordering/access constraints through subsequent changes:

- Trail decrement uses native-endian words; expiry resets through LE
  `checked_word!`. Reread the live selector after the trail callback.
- Capture direction before acceleration/callbacks; look up axes after missile
  acceleration. Invalid directions preserve the original failure prefix.
- Diagonal collision retains the original selector across horizontal callbacks.
  Super Missile extension collision precedes horizontal hit/return testing.
- An empty axes mask executes vertical collision. Beams test hit bit 0;
  missiles test any nonzero value.
- Offscreen positions are native-endian, camera values LE. Read Y only after
  X passes. Extension cleanup visits slots 8, 6, 4, 2, 0.
- Preserve odd/broad selectors, overlapping fields, captured versus live reads,
  callback order and the unified byte-lane resolver.
- Extension activity reads the unrounded parent selector's following byte;
  low-speed follow and positive-speed placement read LE positions, while the
  negative-speed displacement path alone uses a native word. On collision
  failure, retain the callback's writes and selector. Post-collision impact
  testing reads the restored parent's live kind before possibly releasing the child.

Use the existing controlled `old_flight.rs` / `flight_checks.rs`, native fixtures,
actual-runtime compatibility and room/session/beam consumers. Choose cases for
the changed branches. The projectile segment above establishes only its recorded
coverage; another responsibility needs its own exercised checks.
PLAN permits constructed comparisons with explicit limits. Unrelated green
frames add no coverage. Keep the independent reference separate from new code.

## Tools and continuation rules

The Rust migration agent completed. The user authorized coordinated gameplay
implementation and independent read-only review through one integration lane.
No compiler is left running after the completed builds above.
Use PowerShell with `login:false`, `rg`, and one heavy compiler
at a time across the machine (`-j 1`). Recheck HEAD/index before every Git write;
never reserve Git locks. Commit small verified changes to main and run
`sh tools/progress.sh`. Never commit ROMs, states, ROM-derived assets or harnesses.

Edition 2024 is adopted. Stable rustc `1.99.1 (58cf8f9c3 2026-10-13)` works.
Installed `nightly-2026-10-22` is upstream version
`1.101.0-nightly (1a2c4797e 2026-10-21)`, but Defender event 1121 / ASR rule
`01443614-CD74-433A-B99E-2ECDC07BFC25` blocks execution. Keep stable until an
administrator authorizes the compiler; do not bypass the block or change
security settings. Its version is not runtime-verified. Migration commands:
`C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/rust-2024-migration/native-handoff.txt`.

Local tools:

- Python: `C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/python-3.13.7/python.exe`
- Rust: `C:/Users/Jim.Lundin/.rustup/toolchains/stable-x86_64-pc-windows-gnu/bin`
- sh: `C:/Users/Jim.Lundin/AppData/Local/Programs/Git/bin/sh.exe`
- SDL: `C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/SDL2-2.32.10/SDL2-2.32.10/x86_64-w64-mingw32/lib`

Set PATH/RUSTC to stable and LIBRARY_PATH to SDL; see
[Windows shell setup](ITERATION.md#windows-shell-setup). Normal build:
`cargo build --release --locked -j 1 -p sm --lib --bin sm --example sm_verify`.
Set `SM_VERIFY_SOURCE_HASH` using `tools/verify_dll.py::source_hash(Path.cwd())`.
Use `load_host`, `check_source` and `check_library` on
`target/release/examples/sm_verify.dll`, with LIBRARY_PATH set in the loader too.
Stream compiler output to local logs.

Carrier: `python prepare_flight_control.py`, `python build.py --library`,
`python build.py`, then `python check_flight_control.py` with selected export
names. The existing local build.py now reuses content-checked library/harness
outputs and separate incremental caches; a changed fixture does not rebuild
the library. Logs are runtime-build.log and harness-build.log. Compiler options,
dependency bytes, native libraries and output bytes determine reuse; keep the
compiled-input assertions in the selected checks. Launch inputs are synchronized;
keep the separate old decoder in both carrier and harness on subsequent preparation.
For production, use `python tools/verify_dll.py --build-only --frontend-check`
with the installed SDK environment; it now shares normal Cargo flags/artifacts.
Preparation does not compile; reuse unchanged libraries. Formatter:
`rustfmt --edition 2024 --config skip_children=true,reorder_modules=false,reorder_imports=false`.
Repository-wide formatting already fails on the baseline. Do not format
`verify_source.rs` wholesale or introduce unrelated formatting changes.

Next design-only scope: next-visual-session/SPRITE-EFFECT-SESSION-CONTRACT.md.
Complete sprite-effect lifecycle plus actual Waffle/Metroid/Draygon/Maw
controllers; preserve two clear footprints and legacy content provenance.
Independent caller/union source reviews pass:1026newbytes,52079WRAM total,
native unchanged. Concrete interfaces and executable union checks remain.
No sprite implementation has started. Prepared content/Map Rando integration
and persistent whole-game ownership remain unfinished.
