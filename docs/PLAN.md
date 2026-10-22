# Plan

Goal: a compact Super Metroid game engine with descriptive gameplay routines,
playing from boot to credits. SNES emulation remains the current video/audio
backend and verification oracle; it is not the long-term engine architecture.
Map Rando support comes after vanilla.

Git concurrency (user direction, 2026-10-19): agents must not manually
reserve or hold Git locks, including index.lock and branch ref locks.
Use Git's normal short-lived locks only for the actual Git write. Builds,
verification, reasoning, reviews and waits must run without a held lock.
Use isolated snapshots when needed; recheck HEAD and reconcile changes
before committing instead of preventing other agents from committing.
An agent holding a reservation must release its own reservation immediately,
preserving source changes and verification work. Do not delete another
process's lock; tell its owner to release it.

## Active direction (2026-10-20)

Safety requirement (user clarification, 2026-10-22): **remove all unsafe calls**
alongside code reduction and data extraction. This includes replacing raw-memory
access and manipulation, not merely removing `unsafe fn` annotations. Preserve
the complete game, exact defined behavior and byte-compatible SNES RAM. Necessary
safety work may temporarily increase source size; the volume-only parking rules
below do not veto it. Count all replacement code and explain increases.

The shared `word!` interface deliberately uses caller-side raw operations because
enemy pointers can remain live across calls. Do not replace it globally with a
whole-RAM borrow. Migrate retained pointers to offsets/handles, preserve live
rereads and overlapping byte/word fields, and use checked byte access with scoped
borrows. ROM ownership, packed queues, callbacks, initialization, native interfaces
and verification code remain part of the safety work. Hiding unsafe operations
inside a safe wrapper, macro or C adapter does not establish their elimination.
Enforce `forbid(unsafe_code)` in completed safe modules as they become independent
of the remaining unsafe calls; the overall safety requirement is still open.

Current composition (2026-10-22): **94,312 Rust lines in 95 files**, plus 262 C
bridge and 296 C-header lines = **94,870 maintained implementation lines**.
Counting all 803 dependency-installer lines and 28 added verifier-launcher lines
gives **95,701**. This complete recount includes frontend,
verifier, adapters, callers and registrations. Existing development scripts,
configuration and docs remain outside the metric. External dependencies add
**2,836 CPU, 2,104 SPC, 1,261 PPU, 685 DSP and 1,659 music C/header lines**, plus **7,762 stb header
lines**, plus **919 configuration/utility C/header lines** sharing the music
dependency's types header, **882 desktop C lines** sharing those headers, and
**659 shader C / 99 shader header lines** sharing the same types header,
disclosed separately like GL/SDL.
The preceding 96,406 total overcounted unchanged C by four lines: its corrected
baseline is 96,402 (198 C lines). This accounting correction is not new deletion.
No project implementation is hidden in experiment directories.
The final composition passes normal release library/frontend/verifier builds.
The complete GLSL dependency replacement removes **282 maintained lines**, with
no new macros. Normal and instrumented archives pass 38 actual old/new shader
state/GL/diagnostic scenarios and 46 loader cases. Four old/new source-allocation
failure checks cover recursive cleanup with returning and unwinding host errors;
the candidate heap guard separately passes a synthetic C-unwind boundary failure.
The final release-library consumer passes actual presenter initialization,
22 source checks and five shader scenarios, with all 96 embedded inputs checked.
No movie ran; GL/SDL are controlled boundaries and no physical GPU or gameplay
coverage is claimed. Allocation representation changes are not an unsafe-code
elimination result. Details: SHADER_DEPENDENCY.md.
The complete desktop-host dependency replacement removes **697 maintained lines**,
including all adapters, installers, source checks and the retained OpenGL owner.
All **125** actual old/new host scenarios pass, comparing ordered calls, state,
pixels and audio buffers through controlled SDL/runtime boundaries and the real
GameOwner gate. The final installed archive separately passes **33** source,
state, numeric and failure checks. No movie ran; physical devices, gameplay
and multithreaded stress coverage are not claimed. See DESKTOP_DEPENDENCY.md.
The complete configuration/input and file/string/ByteArray dependency replacement
removes **541 maintained lines**, including all adapters, installers and verifier
changes. Source-checked old/new comparisons pass 31 configuration scenarios,
5,463 Boolean cases, 263,438 input lookups and 133 utility cases, plus layout,
source-stamp and failure checks. The actual installed archive also passes
release-library configuration/input/byte-buffer checks. No movie ran; these
checks exercise the changed functions directly. See CONFIG_DEPENDENCY.md.
The native music-sequencer replacement removes **366 complete maintained lines**,
including its compatibility changes, C layout, installer and verifier costs.
All 346 actual old/new cases, 154 C/Rust layout comparisons and 416 ordered DSP
writes match, including two native failures. Continuous native playback matches
48 buffers, 25,632 stereo samples and 5,590 ordered DSP writes, with nonzero PCM.
The probed and clean native-music boot runs each pass **179/0**, inputs **0..178**:
effects and notes first execute at 173, PCM at 174, followed by four frames.
Inputs 0..172 are setup, not active-music coverage. The first 2,000-frame probe
run missed echo initialization and is not a coverage pass; it was not extended.
Echo initialization has actual-code component coverage. No gameplay, physical
SPC or full replay is claimed. Dependency and compatibility details are in
[SPC_DEPENDENCY.md](SPC_DEPENDENCY.md#native-music-sequencer).
Legacy PPU pixel selection and decoding now use the existing PPU dependency,
removing **256 complete maintained lines**. All **85,814** actual old/new pixel,
scanline, full-state and failure comparisons pass, including native VRAM spill
behavior. Probed and clean legacy-renderer boot pass **3/0**, inputs **0..2**;
pixel-query and visible-output probes first execute at frame zero, followed by
two frames. A local verifier host selects the frontend's legacy-renderer option;
normal comparisons, rendering and audio remain enabled. Wider modes and unusual
states have component coverage; no gameplay or new full replay is claimed.
The DSP dependency replacement removes **425 complete maintained lines** and
passes **66,590** actual old/new full-state, APU-RAM, failure and PCM comparisons.
Immediate KON/KOFF, stopped sample writes at 534, padding and partial state before
invalid envelope/FIR failures are preserved. Native and physical-audio boot each
pass probed and clean at **3/0**, inputs **0..2**: cycling, full-buffer, resampling
and actual-consumer probes first execute at frame zero. Active synthesis, BRR,
echo, envelopes and unusual states have component coverage; these boot segments
establish no gameplay/music coverage. Details are in CPU_DEPENDENCY.md.
The coordinated SPC/PPU replacement passes actual-code component checks and
probed/clean physical-audio boot **3/0**, input frames **0..2**, with normal
rendering, audio advancement and comparisons. Required SPC execution, DSP
advancement, VRAM writes, routing, mosaic and visible-rendering probes all first
execute at frame zero; two follow-up frames observe retained state and consumers.
PPU registers pass **77,824** full-state/read/write component comparisons,
including every register/value pair and typed state variations. Reads do not
execute in the selected movie: the initial 1,000-frame run had zero differences
but failed its required read probe, so it supplies no read coverage and was not
extended. At that stage, retained PPU rendering/lifecycle code had exact source
correspondence;
compiled C/Rust layout checks and actual/stale native-source stamp checks pass.
No new full replay or credits coverage is claimed. SPC component and lifecycle
details are in its focused commit and [SPC_DEPENDENCY.md](SPC_DEPENDENCY.md).
Previous grapple swing placement, rope-length changes, clearance and block contact pass
**2,950** actual old/new full-RAM, completion and failure-flag comparisons,
including 37,203 matching ordered projection calls with incoming grapple state.
Both swing directions, bounce/minimum-length stops, anchor snapping, extension blocks,
spike damage and sequential updates are covered using the real implementations.
No grapple movie segment is established; no movie ran. Previous scrolling sky
passes 1,702 components; spawning is modeled and hardware/movie execution is
unestablished. Previous liquid effects pass **6,598**
actual old/new full-RAM, completion and failure-flag comparisons, with 1,511
matching ordered boundary events. Checks cover motion/clamp boundaries, all
tide phases, wave modes, frozen updates, aliased slots and callback mutation.
Register, sound, spawning and assertion calls are recording boundaries;
liquid movie/hardware execution is not established. No movie ran for this change.
Previous layer blending passes 5,504 components. Its probed and clean **3/0**
Ceres fade-in segment, inputs 8324..8326, exercises defaults, mode selection and
actual PPU window/screen consumption. Earlier no-op calls have component coverage;
the setup prefix is cached separately. X-ray/Power Bomb movie execution and
color-math hardware consumption are not claimed. Previous X-ray backgrounds pass
1,201 components and retained-source/caller checks; movement/sound callbacks are
recording boundaries and no X-ray movie segment is established. Minimap passes
1,360 components and a probed/clean **35/0** Ceres segment, inputs 8324..8358,
with setup cached separately. Mother Brain visuals pass 644 components.
The earlier projectile animation change passes 112,184 component comparisons
and a checkpointed 6,562-frame drawing segment: four required probes and 120 follow-up frames,
then the same clean segment, with normal rendering/audio and zero differences.
The 2,218-frame cached setup prefix is separate from changed-code coverage.
Unvisited bomb/SBA/super-missile initialization has direct component coverage.
The under-100,000 maintained-source milestone is complete. Credits remain
uncovered; component/static checks cover edited paths beyond movie coverage.
Further bank retirement and the 90,000-line stretch checkpoint remain future work.

Artifact availability update (2026-10-21): the entire local TEMP
`sm-tailored-next` tree and the original-C/parser experiment directories
disappeared during work. Both long and short Windows paths were checked;
the user confirmed that system resources ran out. No relocated copy is known.
Git source and the unrelated shader
draft remain intact. Historical counts and checks below describe previously
observed results, whose local reproduction artifacts are now unavailable.
They cannot serve as a currently inspectable implementation or completion proof.
The proposed phase31 composition was not completed; no combined reduction is
accepted from its independent prices. Recover the historical inputs if available.
Meanwhile reconstruct and validate selected current-source rules from committed
Rust in isolated `Local/sm-decomp-experiments/20261021` directories, outside TEMP.
Do not reconstruct a partial historical baseline and present it as the old one.

User direction: reduce maintained Rust below **100,000 lines**, preferably
further, by tackling large structural changes before individual cleanup.
Use **90,000 lines as a stretch planning checkpoint**, not a demonstrated
forecast. This work order replaces earlier reduction and cleanup queues.

Completed immediate goal: the verified current implementation above is below
100,000 lines. The last reported complete historical translation composition was
**162,136 -> 102,539 (-59,597)**, also removing **636,768 tokens** and
**1,736,945 code-text bytes**. That historical profile stopped **2,540 lines**
above the target; it is separate from the completed current-source result.
Its 71 Rust files, helpers, callers and registrations count; its Cargo remains
ten lines. External GL/SDL dependencies and experiment support are disclosed
separately. Current Rust remains the behavior authority.

The preceding four-rule batch removes **1,069 lines**, **23,554 tokens** and
**57,729 code-text bytes** from the actual 104,325-line phase29 output:
byte-place recovery 437, native index locals 105, explicit compound RAM
updates 442, and inferred local annotations 85. These are sequential prices,
not sums of independent drafts. Standalone compound updates saved 451;
composition exposes nine lines of overlap. No formatter-only savings count.

Complete optimized library and actual frontend metadata builds pass. Byte
places retain all raw pointer/index operations and pass complete expanded
correspondence plus 327,678 C/old/new cases. Index types pass actual-input
compiler witnesses, source inverse and 131,615 finite cases per profile.
Compound updates preserve checked/wrapping widths, truncation and RHS order;
all 748 updates have complete expanded and resolved-type correspondence.
Annotation inference then passes fresh paired whole-library/frontend compiler
checks on that actual output. Parent independently recounted every stage.

The frozen output and exact reproduction are at TEMP
sm-tailored-next/integration/phase30. The subsequent RAM metadata rule removes
**452 lines**, **4,210 tokens** and **16,206 spelling bytes** from that actual
output. Its final root is TEMP
sm-tailored-next/ram-metadata-sequences/historical-final-v2. All 1,169 ordered
rows and 994 typed offsets have complete compiler-evaluated equality; exported
names/types/visibility/order also match exactly. Runtime access source is
unchanged, and complete optimized library/frontend metadata builds pass.
Sharing identical record metadata then removes **265 lines**, **2,136 tokens**
and **4,956 spelling bytes** from that actual output. Final root: TEMP
sm-tailored-next/record-metadata-groups/historical-final-v4. All 132 records,
their derived implementations and the complete remaining compiler expansion
match; complete builds pass. The independent frontend retains its declarations.
Experiment Python totals 28,605 lines in 371 scripts, separately counted from
emitted source. This is a compiled
translation profile with the documented component/static checks, not a linked
playable historical implementation or a whole-game replay result. It earns
no current-main deletion. Earlier stages and their original-C lineage,
alignment/packing repairs, component limits and rejected alternatives remain
in [REDUCTION_RESEARCH.md](REDUCTION_RESEARCH.md). Preserve those repairs when
composing subsequent rules; do not re-add previously counted savings.
The separate current-policy GL replacement prices **108,052 -> 106,886
(-1,166)** and passes source/build checks, but runtime/adoption is parked:
Defender ASR event 1121 explicitly blocks its test executable under rule
01443614-CD74-433A-B99E-2ECDC07BFC25. All attempted fallback results are excluded.
No current-policy GL reduction is included in the profile; no security settings change.
TEMP sm-tailored-next/current-gl-policy records the price, policy differences
and block. The separate private-header liveness audit finds only 13 removable
lines and is parked; those unused definitions remain in phase23. Five differing
RomPtr_RAM copies also remain. The used-copy stage is recorded at
TEMP sm-tailored-next/private-header-helpers. Continue selecting complete
current-derived emission cohorts; no remaining deletion budget is established.
Current source remains the behavior authority.
The earlier 789-handler executor trial now grows 809 lines on the composed
input; reject it. Its enemy-only subset saves 217 and remains parked. The
remaining C word-operation rules were subsequently composed at phase13:
181 safe countdown sites remove 165 lines and 270 word comparisons remove
100, after the separately counted 55-line alignment repair. All are already
included in phase28; there is no additional 265-line budget.
Numeric, comparison, entity-access and initialization trials
on current source save only 31, 69, 152 and 13 lines on their separate inputs;
park these as volume priorities. Moving functions into associated Game
functions increases code, so reject that spelling as a volume strategy.
Extend the accepted process instead of restarting prior audits or adding
standalone savings.

The requested parallel **implemented original-C pattern experiments** are
complete. The selected mixed C/native-Rust candidate removes **3,016 lines
(82,234 -> 79,218)** across 16 gameplay units and shared support after identical
RAM/numeric preparation. Whole-family replacements cover Samus, enemy and
projectile behavior, menus, bosses, collisions and doors. Paired compilation
and the documented component checks pass; integrated game replay is untested.
This is an original-source result, not a current-main deletion budget. All
previous resource holds remain released. Full costs, rejected alternatives
and coverage are in [REDUCTION_RESEARCH.md](REDUCTION_RESEARCH.md).

Additional user direction: compare original C, automated Rust and a deliberately
written reasonable Rust translation, then implement smarter substitutions for
the expanding categories. Parallel experiments cover named RAM/table lvalues,
control-flow lowering, and fixed-width carry/word arithmetic. Compare against
the already-normalized output as well as raw output, compile the actual
replacements, and check exact byte behavior and ordered effects. Keep this
translation work distinct from the continuing game-algorithm experiments.

Latest clarification: use current compact Rust as the reference for tailored
translation. Trace original C through initial generated Rust to current
`Game`, named RAM access, shared types, callbacks and structured control flow;
derive executable emission rules from those transformations. Test complete
cohorts and their composition, including shared-definition and caller costs.
Distinguish automatically reproducing reductions already present in main
from new algorithm sharing and cases that still require manual work.

Renewed user priority (2026-10-21): identify big changes that significantly
reduce code volume. Alias grouping is now adopted; stop expanding the remaining
small cohorts as the primary work queue. The enemy-field draft remains parked
at -107 with unresolved alignment concerns. The complete RAM-chain cohort is
now adopted after the larger replacements, with its composition proof below.

The distinct direct typed-RAM-prefix experiment on a2b507ae replaces 4,242 reads
and 1,399 assignments over 562 eligible fields. Its complete formatted result is
**103,313 -> 103,683 (+370)**, despite removing 15,139 tokens and 12,520 spelling
bytes. It includes metadata-derived layout, padding, 1,052 byte-place migrations
and 87 safe-function unsafe scopes. Even its caller-only control, with layout
and byte adapters artificially free, saves just 442 lines; existing Game
accessors remain. Reject this spelling for volume before compiler/runtime work.
This is separate from the earlier cached Samus view.
Durable candidate: Local/sm-decomp-experiments/20261021/typed-ram-prefix/candidate-v2.

Architectural comparisons must price complete responsibility replacement:
maintained dependencies for host presentation/shaders/images, and native game
video together with the oracle interface needed to retire the old backend.
At a2b507ae the host graphics/shader/image scope is 2,557 lines; PPU plus DMA is
2,431; CPU plus cartridge is 1,377. These are gross scopes, not deletion budgets.
The entire SNES directory is 6,075 lines, shared by native execution and oracle;
a new renderer alone does not let us delete the old one. An external oracle must
support instruction/PC hooks, state access, patches and snapshot comparisons.
External dependency code and all maintained adapters must be disclosed. Preserve
shader behavior. The user explicitly confirmed the existing DLL workflow is
permitted (2026-10-21). Ordinary compiled DLL component tests with embedded-source
checks may run through ctypes. Keep denied executable/build-script attempts
recorded as failures; do not change Defender settings or disguise those attempts
as passes. The new shader/presenter DLL comparisons below use this permitted path.
Native audio already includes an SM sequencer and DSP, so ordinary audio playback
libraries cannot replace its 2,507 lines of sound behavior. The broader ordered
tile-contact scope is only 2,617 gross lines and has live callback effects; no
multi-thousand-line gameplay saving is established. Bank retirement should follow
actual responsibility replacement. The complete shader/presenter replacement
below brings maintained source to 102,725 lines; no bank is retired by it.
The complete CPU dependency prototype prices 1,217 -> 450 lines (-767), including
16 legacy handlers and identified bus compatibility repairs. It is uncompiled:
Defender denied a dependency build script; zero of 20,480 prepared cases ran.
The direct PNG/inflater dependency prices -793 but is rejected: actual current
Rust accepts reserved Huffman-symbol fixtures that the upstream source rejects.
Three current-Rust DLL controls pass; the upstream C candidate was not executed.
Their prices use the separate 103,313-line input and are not accepted savings.
Complete costs, sources and limits are in REDUCTION_RESEARCH.md. Keep the CPU
earlier candidate parked; the later independently published CPU and compatible
codec pins are separately validated and adopted under item 3. Native shader/presentation ownership is adopted below.
External-oracle replacement alone cannot retire PPU/DMA/bus/cartridge code:
native register access and frame drawing still use it, and Bang_Main still calls
RunAsmCode through Call. A complete replacement must cover those native consumers
and oracle state/control together. Shared PPU rendering is a separate bounded
comparison, not another unused-mode deletion scan.
The ROM-template loader preflight is also closed: 22 fire-pillar shape callbacks
offer only 88 gross lines, while their parameters already follow source-proven
formulas. Decoding changed ROM operands would differ from current hardcoded Rust;
whitelisting, validation and fallback costs are not free. No saving is claimed.
Durable source assessment: Local/sm-decomp-experiments/20261021/rom-callback-descriptors.
The reconstructed alias, constant-group and native C-string cohorts are adopted
under item 3, excluding the entire shader file while its unrelated draft is
present. Their old interrupted prices are not additional savings. The original
60-line residual control-flow trial is superseded by the readable 89-line cohort
below, with complete component tests and both full integration replays passing. Its
awkward Boolean-expression spelling is rejected rather than adopted.

The current-source composition has landed in four focused commits:
**107,520 -> 106,189 (-1,331)**, including 159 terminal loops, 114 selected
field groups, 472 named-RAM write groups and the alignment repair below.
It also removes 6,104 noncomment tokens and 8,460 token-spelling bytes,
including all support and the repair. Complete frontend-enabled builds,
formatting and compiler-expanded library/frontend correspondence pass.
Final full boot any% **129712/0** and Speed Booster **148406/0** pass with
normal rendering, audio and comparisons. Neither route reaches credits.
Broad field and earlier loop spellings that reduced lines while increasing
text were rejected; retain this token/text check against line packing.
The unrelated shader draft is preserved and excluded from the committed
count. The subsequent complete beam-window replacement removes **205 more
lines**, reaching **105,984**, with all imports and source registration counted.
It owns X-ray and rainbow-beam rasterization under descriptive names, with
direct callers and no bank facades. Component checks and focused/full replays
pass as recorded below. Native CPU/SPC tracing removes **144 more lines**,
reaching **105,840**, with complete output/bus differential checks and builds.
Proven-nonnull callback tables remove **71 more lines**, reaching **105,769**,
with complete target/operation correspondence and fresh full boot replays.
The shared Samus pose-update sequences remove **84 more lines**, reaching
**105,685**, with complete expanded correspondence and a focused intro replay.
The packed-sprite word-access repair adds five lines, reaching **105,690**.
Shared gameplay import ownership removes another **381 lines**, reaching
**105,309**, with complete expanded-operation and resolved-binding checks.
Shared enemy callback ownership removes **350 more lines**, reaching **104,959**,
with complete builds, expanded bodies, full-u32 routing correspondence,
381,250 dispatch component cases and both full boot replay routes passing.
The native inflater removes **165 more lines**, reaching **104,794**, with
all allocation and caller costs counted. Complete release/frontend builds,
8,936 inflater cases and 3,080 PNG API comparisons pass; the frontend-only
change needs no game replay. Explicit compound RAM updates remove **370 more
lines**, reaching **104,424**, with complete compiler correspondence and both
full boot replay routes passing. Shared RAM metadata removes **452 more lines**,
reaching **103,972**, with complete constant/export equality and static builds.
Shared record metadata removes **271 more lines**, reaching **103,701**, with
complete expanded-definition/runtime correspondence and static builds.
Shared alias declarations remove **202 more lines**, reaching **103,499**, with
complete declaration/type identity and expanded-source checks plus release build.
Native C-string literals remove **92 more lines**, reaching **103,407**, with
exact byte/type/lifetime checks, source correspondence and release build.
Sharing constant-group headers removes **94 more lines**, reaching **103,313**,
with exact declaration/value/type checks, source correspondence and release build.
Owned shader passes, parameters and textures plus shared presentation setup remove
**588 more lines**, reaching **102,725**, including all support types and callers.
Normal release library/frontend build and 528 shader/336 presenter DLL comparisons
pass. The existing 83-line shader draft is incorporated: the additional working-tree
reduction is 505 lines. Actual GPU pixels are untested; no gameplay changes occur.
Shared PPU pixel evaluation removes **681 more lines**, reaching **102,044**.
Complete release, 175,040 old/new scanline comparisons and both full boot replays
pass. Dense native rendering takes 2.6-2.7 times the former PPU time; the private
unused background cache stops updating, with save layout and g_ram unchanged.
Shared native/SPC state transfers remove **230 more lines**, reaching **101,814**.
All 330 compiled map rows, 1,040 complete differential cases and both full boot
replays pass; the sequencer and DSP behavior outside those transfers is unchanged.
Live-typed snapshots and shared byte-memory diagnostics remove **239 more lines**,
reaching **101,575**, with component checks and both full boot replays passing.
The pinned LakeSnes CPU adapter removes **696 more Rust lines**, reaching
**100,879**, with complete component checks and both full boot replays passing.
Its 41-line installer counts separately; the upstream closure is 2,836 external
C/header lines. The pinned PNG/inflater dependency removes **777 more Rust
lines**, reaching **100,102**. Its four-line C bridge gives **773 net Rust/C
lines removed**. The complete RAM pointer-chain cohort removes **99 more
lines**, reaching **100,003**. The readable control-flow cohort removes **89**
more, reaching **99,914**, with both full integration replays passing.
No complete bank retirement is claimed.

Component checks also found an inherited unaligned RAM reference in the
music queue and related update captures. The historical repair changes 228
captures, costs 55 lines and passes debug/release C/Rust checks. Correctness
costs count explicitly; no reduction credit is claimed. Apply its safe raw
pointer semantics before countdown rules, which must not reintroduce the
same reference. The corresponding current repair covers 83 captures and
costs eight lines, included above. Independent expanded correspondence
preserves 124 reads and 83 writes. Actual music-queue checks pass 66,400 C
cases / 132,800 full-RAM comparisons, alongside debug/release effect checks.
The final adopted source matches the replayed snapshot byte-for-byte.

The current-derived algorithm prototype regenerates all 24 corpse tile
callbacks from original C effects: **793 C -> 2,319 initial Rust -> 208 emitted
ordinary Rust lines**, including helper and dispatch. The output matches
current Rust's equivalent ordinary representation exactly. All 126,720 C
component cases pass; the complete current crate with emitted callbacks
compiles optimized. A complete animated-tiles unit also emits automatically
into the current engine: **273 C -> 671 initial Rust -> 296 tailored Rust**,
with 5,004 C/generated-Rust cases passing. Current helpers and shared owners
are retained and counted; no current-main reduction is claimed. These are
executable emission rules, not a proposal to restart or fork a compiler.

The preceding compatible emission composition was **195,699 -> 162,136
(-33,563)** after the earlier compact preparation of the 278,936-line initial
Rust. It runs primitive types, prior constructors/constants, structured control,
current data/type templates, RAM metadata and callback/table sharing in order.
Both complete optimized roots compile. Every support file and stage overlap
is counted; the separate native-game pilots are not added. Treat this as a
reusable partial translation profile, not evidence of whole-game runtime
equivalence or a generated under-100,000 implementation. Current-main source
remains the production authority; use these rules to expose families before
pricing and validating an adoption there.

The user also sets the original C implementation as the Rust size ceiling.
At the exact upstream input 578f90b, the Makefile builds **100,374 physical
lines of C** in 52 files, including 83,657 in the 32 gameplay banks. All 54 C
files total 100,500; 31 headers add 29,595 separately. These use the progress
script's newline count, including comments and blanks. Keep the stricter
under-100,000 Rust target; do not inflate it by adding headers. Scope and
source provenance are recorded in [REDUCTION_RESEARCH.md](REDUCTION_RESEARCH.md).

The research-led structural/variant audit has now covered same-file and cross-file
function bodies, callback rows and compound blocks within large routines. It
selected the opcode-grid consolidation below, but established no new 500-line
net candidate. [REDUCTION_RESEARCH.md](REDUCTION_RESEARCH.md) supplies the method
background; the bounded follow-up findings are in item 2. Do not repeat these
detector passes without a materially new scope. Detector hits are not savings.

The requested parallel experiment on the original 1311d3b4 transpilation is
complete: combined tool-assisted replacements remove **49,687 lines
(278,936 -> 229,249)** after formatting, including all costs and overlap.
Optimized compilation, structural correspondence and focused routine checks
pass; no full-game/SNES replay is claimed. The compared current snapshot
already shares these implementations, so this result supplies no new main
deletion budget. Details and isolated patches are in the research note's
original-transpilation experiment.

The latest user correction explicitly authorizes isolated experiments on the
original C, superseding the no-C rule for this research. The question is which
large repeated algorithms and C representations should be changed before
transpilation, not whether a restart beats current main. Keep maintained Rust
as the implementation and measure actual paired C2Rust output, including
shared helpers and registration. Do not infer whole-game savings from a pilot.

Latest clarification: learn from the compact representation **before generated
code obscures its structure**. The user is not requesting a restart or a
maintained compiler fork. Use original C to discover whole repeated families;
prevent or normalize translation noise before evaluating their Rust output.
An experiment need not beat current main to answer that research question.
Current-main counts govern adoption and the remaining target, not whether
early-source discovery is worthwhile. Prefer isolated C-boundary changes and
small proved emission passes to a compiler-maintenance commitment.

The C-boundary experiment changes only RAM macros/getters, using two shared
pointer-forming helpers: all 32 gameplay units produce **221,547 -> 188,606
Rust lines (-32,941)**, including helper/declaration/root costs. This prevents
expansion before translation. All paired C/Rust units and both roots compile;
complete C correspondence and 12,032 constructed full-RAM component comparisons
pass, without a full-game replay claim. The complete original Rust control
separately composes numeric, declaration, native-call and address cleanup into **278,936
-> 195,699 (-83,237)**, with optimized compilation, frontend metadata and
complete inverse address correspondence. The results overlap and have different
scopes. No gameplay algorithms are removed by these representation changes.
Generic pointer helpers still lose C's meaningful RAM names; retain named C as
the discovery view and preserve those names in any adopted engine routines.

A paired control also applies the identical numeric policy to both C versions:
**191,082 -> 185,133**, so the existing C restructuring removes **5,949 lines**
after numeric noise is controlled. Numeric cleanup alone preserves all 6,371
function names/signatures and the measured declaration/control-flow counts;
restructuring then removes 861 function definitions and 255 extern declarations
net. Use this pair to distinguish structural changes from emission savings.

The expanded experiment now measures **44,215 fewer generated Rust lines
(229,348 -> 185,133, 19.3%)** across all 32 gameplay banks, CPU infrastructure
and SPC player, including replacement helpers. It composes shared C algorithms
and instruction executors with a proved numeric-emission postpass, using one
formatter. On the original complete 54-file crate, numeric emission alone
removes **35,000 lines**; composing it with shared declarations and native
gameplay calls gives **278,936 -> 219,709 (-59,227)**. The two combined results
use different inputs/scopes and overlap; do not add them or the older 49,687.
Final optimized Rust libraries compile; structural,
constant, arithmetic and focused C/generated-Rust comparisons pass within the
documented coverage. Neither result is a current-main deletion or full-game
replay. Source inspection identifies concrete C2Rust emitter changes; source
patches remain unbuilt. Details are in REDUCTION_RESEARCH.md.

The requested independent-transpiler trial also ran Corrode, CRUST and Sonder.
Corrode removes **312 lines (1,035 -> 723)** on two disjoint real algorithm
units with passing component comparisons, but changes an actual packed game
structure from 11 to 12 bytes. On a complete sm_87 bank, both outputs compile:
**679 -> 627**, a smaller **52-line (7.7%)** gain, without bank runtime checks.
CRUST/Sonder fail the tested real inputs.
Corrode is a useful emission comparator, not an unchanged replacement across
the project; do not extrapolate these bounded counts to the whole game.

Latest user request: actually build and test Cpp2Rust, and determine whether
Corrode's packing differences affect behavior. The 11/12-byte DebugDoorDef
example is unused and did not establish a gameplay failure. A new executed
test of the actual seven-byte VRAM queue consumer does: unmodified Corrode
writes destination 0x0058 instead of 0x5880. Preserving packing on its records
fixes all 1,056 constructed queue cases; normal local field access passes
65,536 controls even with different padding. Packing is a specific repairable
boundary issue, not grounds to dismiss the translator. Cpp2Rust now runs
real-input trials with a disclosed builtin-type rule pack and
Windows runtime profile. The default backend preserves the queue's byte
behavior in all 56 cases despite physically larger Rust records; raw-pointer
output fails 48 nonempty cases, fixed by preserving packing. On the two
disjoint original transfer units Cpp2Rust emits **1,312 default / 1,147 unsafe
lines**, versus **1,035 C2Rust / 723 Corrode**. Large sm_90 unsafe output also
grows, including when comparing the same 449 function definitions. These
results do not select Cpp2Rust for volume reduction. Complete-bank counts
have no runtime proof; details and shared-runtime costs are in the research
note. Keep unrelated reduction work parked.

Latest user clarification: code-volume reduction is the immediate priority.
Focus on repeated code patterns; broad system redesign is too costly unless
it directly removes substantial repetition. Investigate efficient methods in
parallel and price the complete repeated group before expanding implementation.
Re-architecture, shared lifecycle engines and declarative behavior sequences
are justified when they replace substantial implementation. Price complete
families, including their replacement costs, before runtime checks. Previous
module-move experiments do not test the payoff of changing the algorithms or
their representation. Do not use their small savings as a lower-bound claim.

User clarification: the long-term structure is descriptive gameplay modules
and routines, with **bank-based source organization removed**. Keep bank
addresses as ROM/dispatch data. Domain ownership and code-volume reduction
are separate outcomes to measure; a file move alone supplies no deletion.

Latest resource direction: the user has released the machine and agent
capacity for these parallel experiments. Focus it on substantial complete
replacements; keep unrelated small follow-ups parked. Existing source
snapshots, patches, reports and toolchains remain available for reuse.

Keep Game ownership, byte-compatible g_ram, ROM formats and observable
behavior, including quirks. sm/src is the source of truth; use ram_map.rs,
the oracle and https://patrickjohnston.org/bank/ when meaning is unclear.
The pre-transpile project remains archived and is not the work queue.

Design the complete replacement before splitting it into commits. A unit
of planning is a whole subsystem or behavior family, including setup,
updates, transitions, exceptional paths, callers and retirement of old code.
A commit still contains one reviewable idea; it may cover many files.

Internal function boundaries and redundant wrappers may change after checking
all callers and callback references. Preserve numeric dispatch behavior;
keep an old Rust entry point only where a real caller or identity contract
needs it. Use ordinary functions, enums and typed data where they simplify
the whole algorithm. Migrate one complete responsibility at a time, redirect
real callers, and remove its old bank implementation. Remaining banks are
temporary homes for unmigrated responsibilities. A shared engine must serve
concrete repeated behavior and reduce the complete maintained implementation;
an interpreter or data table is neither automatically disallowed nor a saving
by itself. Keep meaningful gameplay names and avoid permanent bank facades.

Small preparatory changes, including temporary line-count increases, are
allowed when attached to a concrete replacement with named consumers and
an explicit deletion step. Explain counter increases in the commit. Judge
the completed replacement's net size, including every helper, table, wrapper
and source registration. Do not leave both implementations maintained.

## Baseline and diagnosis

At **ec45c573**, 72 tracked Rust files in sm/src contain **112,265 lines**.
Reaching 99,999 requires **12,266 net lines removed (10.9%)**; reaching
90,000 requires 22,265. Count physical lines with the existing progress
script, including comments, blanks, helpers and declarations. Removing
checks, moving code outside sm/src or compressing formatting is not progress
toward the intended goal.

These regions partition the baseline. Their size is search territory,
not a forecast of removable code. Bank names refer to sm/src/game/sm_XX.rs.

| Region | Banks / files | Lines |
| --- | --- | ---: |
| Rooms, menus, HUD, effects and cinematics | 80, 81, 82, 85, 87, 88, 89, 8F, 8B, 8D | 22,408 |
| Bosses and endgame | A4, A5, A6, A7, A9, AA, AD | 20,337 |
| Ordinary enemies and enemy projectiles | A2, A3, A8, B2, B3, waver, 86 | 20,308 |
| Samus and player projectiles | 90, 91, 92, 93, 94, 9B | 16,524 |
| Enemy engine and sprites | A0, B4 | 4,268 |
| PLMs | 84 | 3,490 |
| Shared gameplay declarations | constants, callback_tables, enemy_layouts, enemy_ram | 985 |
| Hardware, frontend, runtime and verification | Remaining sm/src files | 23,945 |
| **Total** | **72 files** | **112,265** |

Gameplay is **88,320 lines (78.7%)**. The first four regions contain
79,577 lines. Assess the complete regions, then prioritize actual savings
and available coverage; largest file does not necessarily mean best cut.

The previous large-family reset, 51c0e735..ec45c573, produced 42 commits:
24 reductions, one verifier change and 17 documentation-only commits.
Net Rust deletion was 4,286 lines; median reduction was 109 lines, and the
latest ten reduction commits removed 1,043 combined. PLAN grew 1,518 lines
to 28,306 lines / 1.86 MB. These are Git-history counts, not labor timings.
The loop was rewarding completed small edits and extensive reporting while
the overall replacement design remained unpriced.

Larger landed changes demonstrate the useful scale: shared enemy layouts
removed 1,696 lines and ordered word transfers removed 1,021. Those savings
are already in the baseline. Repeated clone scans and signature macros now
have diminishing measured returns. The next experiment must change how a
complete behavior is represented, rather than repeat those searches.

## Domain experiments (2026-10-20)

Five independent compiled experiments start from 7d1af86e, whose maintained
Rust matches ec45c573. Counts include all new modules, imports, callers and
source registrations, after formatting. These are alternative drafts, not
landed savings; the three Samus alternatives overlap and cannot be added.

| Experiment | Complete measured scope | Whole sm/src | Net change |
| --- | --- | ---: | ---: |
| Samus motion/pose modules | 3,167 moved lines; input, movement, jump setup and pose progression | 112,237 | -28 |
| Typed Samus profiles | 14 air/ball routines and macros, including ground/landing/reset | 112,195 | -70 |
| Samus frame phases | 2,355 moved lines; input/frame/pose routing and real callers | 112,226 | -39 |
| Roach lifecycle | 33 functions plus two forwarding rows; all 17 phases | 112,236 | -29 |
| Menus, sprites and save data | Entire sm_81 removed; 120 ordinary routines moved | 112,306 | +41 |

Relocation controls grow by 65 lines for motion/pose, 42 for frame phases and
23 for Roach. Consolidation recovers that overhead but produces small net
reductions. No experiment establishes a 1,000-line replacement or a route
covering the 12,266-line gap. These results reject that forecast for the tested
designs; they do not prove that larger reductions elsewhere are impossible.

The useful architectural result is explicit ownership. Prefer ordinary named
domain routines; the profile pilot concentrates several exceptional branches
for only 70 lines. Samus needs separate prepare_frame and finish_frame seams:
enemy/projectile work and a possible message-box suspension occur between
them. Retain out-of-band grapple, demo, X-ray and cinematic entry points where
real callers need them. Roach has a true initialize/update interface. Menus
has three update entries, with private state handlers. The menu experiment
retires a complete bank; the Samus experiments still depend on unmigrated
physics/collision code and do not retire all Samus banks.

All drafts pass formatting and core compiler checks. The motion/pose,
frame-phase and bank-removal drafts each pass clean, source-checked optimized
any% **129712/0** and Speed Booster **148406/0** replays. Motion/pose's seven
required probes cover normal input/movement/pose, both spin branches and both
grounded-ball branches; frame phases have five exercised normal-control probes.
Demo, X-ray, grapple, Spring Ball and unusual transition coverage remains
limited to the stated static checks where those paths were not observed.

The bank-removal draft preserves all 120 ordinary function bodies under its
explicit name/visibility mapping and passes frontend-library metadata checks.
Its probes exercise menus, sprites and SRAM loading/saving; map selection has
zero hits, correctly failing that requested coverage requirement. The clean
full replays and source correspondence do not establish map-branch execution.
Fresh frontend Cargo dependency build scripts hit Windows OS error 5; the
gameplay checks use the existing optimized core/DLL verifier instead.

Roach passes **14,172 full-RAM comparisons** between actual optimized original/
replacement cores, including 21 persistent 600-update sequences. This is
constructed Rust differential coverage, not SNES-oracle or movie execution.
Typed profiles pass an exercised Speed Booster segment **46360/0**, covering
all five motion entries plus 120 follow-up frames; Spring Ball landing and
other exceptional branches remain unproven. None of the routes reaches credits.

Runnable draft roots under TEMP, in table order: sm-samus-minimal-clean-20261020,
sm-samus-profiles-20261020, sm-samus-common-20261020, sm-roach-domain-1cb6fb50,
and sm-domain-menu-7d1af86e/candidate. The menu experiment's parent directory
contains REPORT.md and trial.patch; the patch applies to main without changing
the unrelated shader work. Experiments remain outside maintained source.

Decision: extend domain ownership across coherent subsystems for readability,
while pricing actual algorithm deletion separately. Resolve each draft's
remaining coverage and dependency surface before landing it. Keep these
experiments parked as volume workstreams; use their domain design when the
relevant subsystem is migrated. Continue the independent large-reduction
comparison below without extrapolating savings from these pilots.

These domain moves remain parked during the immediate volume pass. Preserve
their useful phase/dependency findings when a replacement needs them, but
prioritize deleting repeated implementation over adopting relocation controls.

Four subsequent experiments at 7617b682 change the representation itself.
Their formatted net savings include replacement code, callers and source
registration; only moving platforms is adopted below.

| Replacement | Net deletion | Validation and disposition |
| --- | ---: | --- |
| Moving platforms and shutters: shared triggers, travel, waits and growth stages | 192 | 16,414 full-RAM comparisons; final integration below |
| Mother Brain: 124 typed encounter steps, shared delay gates and continuations | 318 | 2,208 comparisons cover 69 steps; park pending remaining coverage and independent module costs |
| Crawlers and Snail: shared surface movement and native lifecycle routing | 144 | 6,528 full-RAM comparisons; park before movie integration |
| Configuration/input: native collections and shared option parsing | 165 | Frontend-library compilation only; park before runtime validation |

Component comparisons use actual optimized original/replacement Rust, not a
SNES oracle. Mother Brain remains a child of sm_a9, so its saving does not price
complete bank removal; 55 steps and real encounter history remain untested.
The experiments establish useful local deletion, not a forecast for the gap.
Price complete repeated algorithms before spending on broad runtime checks;
continue discovery beyond these scopes instead of multiplying their savings.
Drafts and reports remain under TEMP: sm-mechanism-lifecycle-7617b682,
sm-motherbrain-sequence-7617b682, sm-native-crawlers-7617b682 and
sm-config-engine-7617b682. Unlanded drafts contribute nothing to the budget.

## Work order

1. [x] **Test Samus domain alternatives.** The concrete domain experiments
   above reject the proposed large volume payoff and establish useful named
   interfaces. Keep measured scope and unvisited behavior explicit. Subsequent
   candidates need complete formatted/compiled replacement counts, including
   retained exceptions and caller costs, before implementation expands. If a
   candidate produces only a small extraction, record its result and advance
   discovery; if stuck for an hour, record findings and stop.

   Pilot decision: reject this representation and advance to item 2. Consolidated
   domain functions remove **77 formatted lines**; ordinary typed profiles remove
   **71**, including all helpers, declarations and callers. Both compile across
   all frontend targets. The pilots cover all 28 input states and the morph/spring
   ball entry, grounded movement, landing, bounce and equipment-refresh lifecycle.
   Jump/fall/spin, grapple, knockback and bank-94 collision paths retain their
   existing algorithms and phase boundaries. No full replacement or 1,000-line
   budget was established; no oracle execution or landed savings are claimed.
   Drafts and exact counts: TEMP sm-bindings-root-9c4daf2f/samus-cycle/REPORT.md.

2. [x] **Compare the selected large subsystem replacements.** The research-led
   follow-up uses existing AST/Pygments/Copydetect tools on 2,393 function and
   callback bodies, normalizing names and numbers. Sparse comparison took 8.6s
   and included same-file and cross-file near-miss islands. A separate 53.8s
   pass covers 2,995 compound fragments inside 362 large routines, excluding
   ancestor overlaps and already-studied function pairs. Neither establishes a
   new 500-line net family; the strongest complete draft is the adopted 184-line
   opcode grid. These are bounded findings, not proof that all sharing is done.
   Reports: TEMP sm-near-miss-functions-f57e4abf,
   sm-inner-variants-c98e8439 and sm-gameplay-variant-audit-f57e4abf.
   Require a materially new named algorithm
   or scope before repeating discovery. The regions and prior results below
   guide coverage, not boundaries that candidate replacements must preserve:

   The complete synchronous tile-contact experiment is now implemented and
   parked. A shared traversal/material owner grows 45 lines; adding shared
   oriented grapple, Samus and enemy geometry yields only **68 lines removed**
   from its complete 102,725-line input. All 22 changed/new files, support,
   callers and registrations count. Formatting/source integrity pass; no build
   or runtime check is claimed. Retained callback effects, live rereads and
   actor-specific commit order consume most of the apparent sharing budget.
   Source: Local/sm-decomp-experiments/20261021/ordered-tile-contact.
   The separate tile-layout preflight found no new large common builder beyond
   already shared word transfers; its distinct clipping, stream and queue
   contracts are recorded in the sibling tile-layout-native report. No saving
   is claimed. The audio-core dependency preflight likewise establishes no
   adapter budget: alternate DSPs have differing KON timing and internal state
   contracts. The sequencer's actual state-transfer replacement is adopted below.
   Larger selected replacements now have component results; composing the
   previously priced pointer-chain and residual-control-flow cohorts may proceed
   after them, with fresh complete counts and validation rather than new scans.

   The current-derived translation experiments are complete as scoped above.
   Reuse their executable emission passes and C effect recognizers when
   examining another complete family; do not repeat raw expansion counts or
   the same clone scans. Domain judgment selects the native helper contract;
   recognition derives per-member parameters and rejects different effects.
   Price any production adoption against current Rust and validate it under
   item 3. The isolated 102,539-line output is a partial translation profile,
   not a replacement implementation or an under-100,000 result.

   The new finite typed instruction-data experiment also fails its complete
   price: **106,189 -> 106,740 (+551)** for 340 handlers, including all tables,
   execution, routing and retained exceptions. It shares complete operations
   rather than concatenating bodies, but their metadata costs outweigh the
   remaining compact callbacks. Reject without builds or behavior claims.
   Named destination metadata would cost still more; the numeric draft is not
   a naming-complete architecture. TEMP: sm-tailored-next/instruction-data.

   The distinct non-instruction state-program price also grows the complete
   source: **106,189 -> 106,831 (+642)** across 98 replaced callbacks and 306
   retained phases in 12 boss/presentation dispatchers. The shared runner
   covers exact waits, ordered word effects, native calls and transitions;
   all support and callback identity costs count. Complete builds, formatting
   and numeric routing checks pass; reject without runtime work.
   An infix arithmetic grammar prices 932 roots: ordinary operands save 74
   lines while growing tokens/text; named RAM operands save 120 lines and
   shrink tokens/text. Park the named version before broad validation.
   Reports: TEMP sm-tailored-next/finite-state-programs and infix-word-math.

   Further complete numeric/support prices remain small. Persistent SPC pitch
   fields as `Wrapping<u16>` grow **17 lines** after all callers/checkpoint
   conversions; a primitive-width control saves one. Complete builds and 115
   layout assertions pass, but Defender ASR event 1121 blocks the numeric test
   executable before entry; no runtime result or alternative launch is claimed.
   A shared CPU decimal-digit recurrence saves only **41 lines**, parked before
   builds. Compiler-resolved frontend/verifier/test consumers permit a separate
   **132-line** C-header/type/Sfx-wrapper deletion only under application scope;
   strict public Rust API retention saves zero. Complete builds pass, with no
   post-removal resolved correspondence or runtime claim. That candidate and its
   explicit public-path contraction remain parked. These are independent prices,
   not a current-main deletion budget. Reports: TEMP
   sm-tailored-next/{wrapping-pitch,support-consumer-liveness} and
   sm-cpu-decimal-recurrence-20261021. Do not repeat these scoped studies.

   Remaining non-game internal declaration ownership separately prices **110
   lines** on phase26: 146 foreign declarations become imports of 136 existing
   symbols across 16 files. Complete builds and preservation of 638 enumerated
   definitions pass; one incompatible frontend type boundary remains foreign.
   Provider/address/interposition equivalence is unproved. Park this draft,
   excluding it from the composition. Report: TEMP
   sm-tailored-next/internal-declaration-owners/REPORT.md.

   Two new complete domain prices on phase28 remain too small: ending
   particles save **17 lines**, including the named eight-word record and a
   separately priced three-line packed-OAM repair; rectangle-contact helpers
   save **nine lines** across six complete routines. Both are formatted prices
   only, without compilation/runtime claims or adoption. The ending Y-word
   store is at an odd OAM byte offset; its unaligned repair is explicit and
   unvalidated, not disguised as equivalent old Rust behavior. An existing-data
   gameplay liveness follow-up establishes zero historical reduction; its only
   eight additional current occupied lines do not justify another scanner.
   These scopes are stopped. Reports: TEMP sm-tailored-next/{ending-particles,
   projectile-collision-geometry,gameplay-liveness-followup}.

   Two further complete prices remain excluded. Current-derived PPU Mode 7,
   windows and scanline sharing saves only 12 lines; its window/Mode 7 subset
   saves 38 before the scanline helpers add 26. Lifting conditional RAM stores
   into one RHS grows 115 lines across 89 conditionals despite removing tokens.
   Both are formatted prices without build/runtime validation; park the PPU
   study and reject the conditional-store spelling. Reports: TEMP
   sm-ppu-current-composition-20261021 and
   sm-tailored-next/conditional-word-stores. Do not repeat these scopes.

   The complete raw-memory call boundary also saves only 41 historical lines
   across 120 calls, including two ordinary wrappers; park without builds.
   Applying the byte-place, native-index and annotation rules separately to
   committed main saves only six, 18 and 35 lines. These price-only current
   results are parked, excluded from the adoption budget and not additive.
   Reports: TEMP sm-tailored-next/{memory-call-boundary,ram-byte-current,
   current-native-index-locals,local-type-inference/CURRENT_PRICE.md}.

   Complete ordinary-place compound updates are also rejected: 1,957
   historical sites grow **71 lines**, and 1,703 current sites grow **108**,
   including support/imports. The alternative method-name spelling grows
   386/417 lines. This extends the old direct-assignment preflight to promoted
   arithmetic and captured places; neither complete spelling saves lines.
   Static legacy GL symbol grids remove 172 historical lines and 7,402
   spelling bytes but add 513 tokens; reject under the token/text selection
   rule. Both are source-only prices without builds or runtime claims.
   Reports: TEMP sm-tailored-next/{ordinary-place-updates,gl-static-groups}.
   The GL administrator-policy block remains unchanged; no tests were launched.

   A distinct context-pruning experiment removes injected `Game` arguments
   from context-free helper call graphs. Preserving public Rust signatures
   saves zero lines across six private helpers/nine calls. Explicitly changing
   two public signatures expands the cohort to eight functions/40 source calls,
   saving only 18 lines, 127 tokens and 316 spelling bytes. Callback/address-taken
   contracts remain excluded. Historical phase30 has no injected `Game`
   parameters, so its direct saving is zero. Park these complete formatted
   prices without builds or runtime checks. TEMP sm-tailored-next/pure-game-context.

   A distinct complete local-width trial includes callback bodies and known RAM
   macro expressions. Its last observed current draft selected 32 locals in 25
   functions (eight mutable), but grew five lines, 242 tokens and 389 spelling
   bytes. Required widening boundaries consumed the benefit. Right shifts,
   full-width comparisons, references, escapes and checked arithmetic observing
   discarded bits remain excluded. Final guard corrections and historical pricing
   were unfinished when the TEMP artifacts disappeared. Park with zero accepted
   savings; neither reproducibility nor behavior validation is established.

   Same-type alias declarations qualify for adoption under item 3. The rebuilt
   current rule excludes the shader draft and attributed aliases, using a smaller
   seven-line helper. Its final result is measured on the actual record-metadata
   output. The old 195-line current and 64-line historical independent prices
   are not additional savings; their TEMP reproduction artifacts are unavailable.

   Those representation studies are closed. Five subsequent domain studies
   use the same repaired 106,189-line control:

   - Complete palette pipelines across room transitions, Phantoon, cinematics
     and suit/visor effects save only **20 lines**, including all 20 replaced
     functions and shared support. Full library/frontend builds pass; park
     without runtime work.
   - Native recording/playback, snapshot loading/saving and history rebasing
     save only **68 lines**, including Vec initialization, cursor/file helpers,
     unchanged stream formats and checkpoint adapters. Shader consumers still
     require ByteArray. Park this formatted, uncompiled price.
   - Samus locomotion/contact preflight identifies 313 gross lines in 23 new
     items; existing shared motion and transition owners remain charged.
     Cross-actor collision preflight audits 1,428 lines, finding shared retry,
     square-slope and shot-axis owners already present. Neither establishes a
     new large replacement; no implementation or saving is claimed. These are
     bounded findings, not limits on every possible redesign.
   - The complete X-ray/Mother Brain beam-window replacement is adopted under
     item 3: **205 lines removed**, with corrected current-Rust behavior and
     direct callers validated. A 500-line discovery threshold is a prioritization
     aid, not a reason to reject a useful complete replacement.

   Reports: TEMP sm-tailored-next/palette-pipeline, state-recorder-native,
   samus-locomotion-cycle, actor-world-collision and beam-windows. These are
   separate controls; do not add their prices. Keep these scoped studies closed.
   Further discovery needs a new complete algorithm and replacement, not a
   repeat of these cohorts or another generic operation runner. Current-derived
   translation recovery remains demonstrated separately at 102,539 lines.

   Further complete experiments at 105,984 reject shared object-script traversal
   (**+117 lines**), contextual Kraid ownership (**+33**) and real receiver methods
   (**+196**). Shared live hardware/snapshot/archive state removes 95 lines but
   adds 1,542 tokens, so reject it too. A cached raw Samus RAM view adds **921
   lines** with endian preservation; even the native-endian control grows.
   Complete library/frontend builds pass for these priced candidates; no runtime
   equivalence is claimed. The PPU specialization preflight finds only 236 gross
   lines of remaining generic capabilities and proves none unused. Keep these
   studies closed. Native tracing and proven-nonnull callback tables instead
   qualify for adoption. Reports: TEMP sm-tailored-next/{object-lifecycle,
   contextual-kraid,samus-overlay,ppu-sm-specialization,nonnull-tables,
   tracing-native} and TEMP sm-hardware-state-d878362a.

   Three complete follow-ups use the committed 105,769-line control:
   native configuration/input ownership saves **100 lines**, but actual paired
   execution is blocked before entry by Windows administrator policy. Complete
   library/frontend and harness builds pass; runtime coverage is zero. Preserve
   the block and park adoption, including the old gamepad allocation boundary
   and differing allocation-failure handling. A standalone 124-step Mother Brain
   owner saves only **14 lines** and adds 593 code-text bytes; complete builds
   pass, but 52 bank dependencies remain. Park without new runtime checks.
   Compiler-guided immutable bindings remove 470 qualifiers but only **five
   lines**; complete builds and source-token correspondence pass. Park as a
   volume priority. These are independent prices, not an aggregate budget.
   Reports: TEMP sm-config-engine-31566113, sm-motherbrain-sequence-31566113
   and sm-tailored-next/immutable-bindings. None modifies maintained source.

   A distinct sibling-statement search covers 45,470 proper windows in 2,978
   ordinary functions and 2,421 callback bodies. It includes small routines
   and arbitrary consecutive statement sequences excluded by the earlier
   whole-function/compound-node scans. Its 39 non-subsumed groups establish
   no large new budget; the coherent Samus pose-update family qualifies below.
   This scope is now also closed. TEMP sm-tailored-next/sibling-sequences.
   Typed ROM-record indexing was checked against its existing complete study,
   not rerun: the exact current Standing routine already appears there, and
   the full family saves only 25 lines. TEMP sm-rom-records-443e8471.

   The new complete touch/shot/bomb extended-hitbox search price grows **26
   lines**, including its distinct byte/word counts and mandatory-first/wrapping
   traversal. Complete library/frontend builds and source checks pass; reject
   without runtime work. This extends the old pair-only 13/29-line drafts and
   claims no drawing-owner or bank-retirement saving. Decompression and SPC
   music follow-ups match already closed cohorts; they were not rerun. The
   separate current suit-beam helper price saves only **35 lines**, with no
   compilation/runtime claim; park it while retaining the validated historical
   121-line rule. Reports: TEMP sm-tailored-next/{enemy-spritemap-composition,
   spc-music-scope,suit-pickup-beam}. These are not an aggregate current budget.

   The expanded source-first C experiment is complete: sharing 35 algorithm
   groups, retiring forwarding functions, fusing 789 instruction bodies and
   changing 2,381 word-arithmetic occurrences removes **7,227 generated lines**
   from its raw C2Rust cohort. The actual composed numeric-emission candidate
   removes **44,215** under the same final formatter, as scoped above. Earlier
   slope/scrolling and partial-access studies remain in REDUCTION_RESEARCH.md;
   their overlapping standalone counts are not added. Optimized compilation
   and the documented component comparisons establish a useful translator
   profile, not a current-main deletion budget or full-game equivalence. Use
   these complete-family replacements and emitter findings for subsequent
   adoption decisions; keep unrelated small reduction work parked.

   A separate normalized/aligned audit of 1,331 bodies at least 20 lines long
   produced three complete prices: enemy collision sweeps **-38**, room loading
   **-24**, and Power Bomb/Crystal Flash windows **-90**. Counts include retained
   variants, helpers, callers, imports and source registrations. Park the first
   two; adopt the window drivers below. This overlaps the audits above and
   supplies no larger forecast. TEMP: sm-structural-audit-f57e4abf,
   sm-enemy-sweep-c98e8439, sm-room-loading-c98e8439 and
   sm-bindings-root-9c4daf2f/blast-window.

   Enemy/projectile preflight found no credible large replacement. Complete
   Metalee/Skree bodies occupy 252 lines; B2 plus laser/claw bodies occupy about
   996 before retaining distinct wall, ninja and walking behavior. Rio, linked
   flyers, crawlers, pipe bugs, hoppers and ordinary projectiles retain distinct
   lifecycle contracts after existing sharing. No draft or saving is claimed.

   Presentation preflight likewise found no candidate to price across title/
   cinematic exits, file-map and pause flows, Power Bomb/Crystal Flash, suit
   pickups and door presentation. Shared geometry, drawing, transfer and beam
   phases are already extracted; remaining ordered state effects differ.
   A broad scene profile would transcribe arbitrary callbacks and statements.
   Both bounded studies advance discovery without claiming exhaustive limits.

   Boss preflight found no credible roughly 1,000-line replacement across the
   examined Torizo, Crocomire, Ridley, Spore Spawn and Mother Brain phases and
   their projectile consumers. Existing algorithms are already shared; the
   remaining phase contracts differ. This is bounded evidence, not exhaustion.

   The intro narrative-page replacement qualifies for item 3: `intro_pages`
   owns page markers/input, four scene setups, both crossfade directions,
   Japanese continuations and the handoff to Ceres. The separate Japanese
   accumulator correction landed first; its helper is retired into the module.
   Engine primitives and scene sprite motion remain in their existing homes.
   Continue discovery from the remaining responsibilities, using these bounded
   preflights and measured drafts instead of repeating the same searches.

   Complete hopping-enemy repricing at b64a0470 includes initialization,
   routing, animation, movement and contact handling for Sidehopper, Beetom
   and Thin Hopping Blobs. Relocation costs 26 lines; the full replacement
   removes only **44 net lines**, including all imports and registrations.
   It formats and compiles across all targets, with no new runtime coverage.
   Park this volume candidate and advance discovery. The earlier covered
   Sidehopper/Beetom sharing is already in the baseline, so its historical
   projection cannot be counted again. Draft: TEMP
   sm-bindings-root-9c4daf2f/hopping-lifecycle/REPORT.md.

   - Enemy/projectile lifecycles across A2/A3/A8/B2/B3/86: initialization,
     state selection, movement, collision, animation and reset together.
     Classify real differences before choosing shared algorithms and data.
   - Presentation sequences across 80/81/82/88/8B/8D: scene setup, ordered
     drawing/transfers, waits and transition decisions together. Existing
     copy/queue helpers are inputs to the design, not new savings.
   - Boss phase families in A4/A5/A6/A7/A9/AA/AD, following their projectiles
     into 86: prioritize a complete phase with obtainable oracle coverage.
     Unvisited endgame behavior has an explicit coverage dependency.

   Rank complete candidates by net deletion, implementation/check cost and
   coverage availability. Examine broader opportunities before spending time
   on lower-volume engine, PLM, frontend or hardware refinements. Small edits
   may serve a selected replacement. Complete repeated-pattern cohorts can
   proceed through item 3; isolated naming/unsafe cleanup waits until item 4.
   Reopen a parked study only when the proposed representation or scope is
   materially different, and state that difference before starting. Sequential
   composition and validation of a selected complete cohort qualify; repeating
   its isolated price or scanner does not.

3. [x] **Implement the strongest measured replacements and recount.** Start
   a proven candidate as soon as it is ready; an exact budget for the entire
   remaining gap is not a prerequisite for useful implementation. At each
   completed replacement, record baseline -> result, net deletion including
   shared code, checks, and the remaining gap. Count overlapping spans once.
   Keep measured drafts separate from landed savings. The CPU and PNG dependency
   replacements now have passing component checks and complete release builds.
   Their main integration and the following RAM-chain/readable-control-flow
   composition are complete, reaching the verified count above. The measured
   discovery is bounded; it does not prove that all future large savings are gone.

   Readable control flow removes **89 lines (100,003 -> 99,914)**, 333 tokens
   and 2,669 spelling bytes across equipment selection, scrolling, grapple,
   Ridley angle clamps and PPU register writes. Ordinary branch chains and a
   named loop result replace numeric continuation tags; grouped PPU scroll arms
   retain Mode7 side effects in order. No helpers/dependencies/imports are added.
   This supersedes the smaller 60-line Boolean-expression draft, which was
   rejected for readability. Ridley's macro body is formatted independently;
   zero formatting-only savings count. All five original and adopted full
   routine bodies match the exact component-tested pair after composition with
   the CPU, codec, snapshot and RAM-chain changes. Complete normal main release
   library/frontend (38.71s) and formatting pass. Source-bound actual-body DLLs
   pass **1,145,096 comparisons in each of optimized and overflow-checked modes**,
   including complete PPU states, signed grapple ranges/callback mutation,
   scrolling boundaries and Ridley angle ranges. External callbacks are scripted;
   this does not claim every edited path executes in a movie. Final full boot
   any% **129712/0** and Speed Booster **148406/0** pass with normal rendering/
   audio, source fingerprint
   `a49e35e84ffd2c8e964c2b9df5414f9136ce9e1fb6ad6be255d1c541161b9e4b`.
   Replay times are 482.51s/717.66s. No new failing frames; neither reaches credits.
   Durable source/checks: Local/sm-decomp-experiments/20261021/residual-control-flow/
   composition-e2530722/readable-v2. Neither complete bank retirement nor credits
   is established by the volume result.

   The existing RAM-chain grammar removes **99 lines (100,102 -> 100,003)**,
   1,717 tokens and 3,580 spelling bytes across all 135 chains/24 files/five
   templates. Ordered byte offsets, receiver evaluation and final pointer types
   remain. No helper, import, dependency or source registration is added.
   The composed source has a complete inverse and all 135 original caller/template
   bodies match the previously proven cohort and its 141 contextual type witnesses.
   All 24 actual main input/output files match the composed proof after newline
   normalization; the intervening CPU, snapshot and codec changes are retained.
   Earlier complete compiler expansion proves all 7,098 functions and the ordered
   remainder after only thin pointer-cast normalization. Zero formatter credit.
   Actual changed-file formatting and main release library/frontend pass (44.52s).
   Static equivalence validates unchanged operations; no gameplay replay is
   required for this representation-only commit. Final integration follows the
   control-flow cohort. Cast-containing lines fall by nine; other counters remain.
   Durable composition: Local/sm-decomp-experiments/20261021/
   ram-pointer-chains-composed-6170cbea-cpu; original proof: ram-pointer-chains-e2530722.

   The complete PNG/inflater replacement removes **777 Rust lines
   (100,879 -> 100,102)**, or **773 including its four-line C bridge**.
   The published stb-sys 0.6.0 header supplies unmodified stb_image v2.26;
   its 7,762 external C lines are disclosed separately. The compatibility pin,
   PNG-only/dimension settings and zero-initialized missing palette entries are
   mandatory. Source/installed archive identity, formatting and the final normal
   main release library/frontend build pass (41.24s). Actual component checks pass
   **6,704 inputs / 13,412 memory/file API comparisons**, plus 202 focused checks
   after the final linkage-only adjustment. Allocation exhaustion and all malformed
   streams are not proven. This frontend-only change needs no gameplay replay.
   Decoder bodies match the tested candidate; the old inflater and registration
   are removed. The additional extern C block is the codec dependency boundary.
   Install and distribute as described in [PNG_DEPENDENCY.md](PNG_DEPENDENCY.md).
   Durable source/checks: Local/sm-decomp-experiments/20261021/codec-dependency/v226.

   The pinned LakeSnes CPU adapter is adopted after full replay validation:
   **101,575 -> 100,879 (-696 Rust lines)**, or **655 net lines** after its
   41-line installer. The independently published upstream closure is 2,836
   C/header lines, separately disclosed in [CPU_DEPENDENCY.md](CPU_DEPENDENCY.md).
   Fifteen compatibility/hook handlers retain current behavior; the Rust Cpu
   layout, 27-byte legacy save span and outer interrupt sequencing remain.
   A fresh foreign CPU is constructed each instruction, with no persistent
   foreign snapshot state. All seven embedded component sources match.
   **524,288 instruction cases and 4,096 IRQ/NMI/WAI/STP cases** have zero
   observable differences: registers, bus order/memory/history and final cycles.
   Intermediate callback cycle fields differ in 240,214/672 cases; the actual
   reachable bus and host callbacks do not read that field. This is deliberately
   not whole-Cpu byte identity during callbacks. Final main release build and
   formatting pass. Full boot any% **129712/0** and Speed Booster **148406/0**
   pass with normal rendering/audio and source fingerprint
   `3a08843e9e15b17aa59eeb5f1018dca693c40177a1f85e2fc1f2e8b13eb6efde`.
   Neither reaches credits. Direct C
   compilation and static linkage passed; earlier denied installer/build-script
   attempts remain failures. No security settings changed. The additional extern
   C block is the dependency boundary; unsafe declarations fall by 25.
   Durable source/checks: Local/sm-decomp-experiments/20261021/lakesnes-cpu-dependency.

   Live-typed snapshots remove **239 lines (101,814 -> 101,575)**, 2,453 tokens
   and 6,196 spelling bytes. Existing Cpu/Ppu/Snes/Dma types replace separate
   stored projections; explicit partial restore and diagnostic projections keep
   the original field selection and order. RAM/SRAM diagnostics share traversal
   while preserving their limits, word grouping and exact messages. Extra captured
   pointers/caches are inert. Uninitialized Dma.pad is never read: only its four
   initialized state fields are captured. Storage grows 2,608 bytes per snapshot,
   **7,824 bytes total**; g_ram and legacy/checkpoint serialized formats remain.
   All 144 ordered restore operations match; 24 other functions are unchanged.
   Complete main release library/frontend, formatting and progress pass. Ordinary
   source-bound DLL tests pass **512 capture/restore cases + 96 comparisons**,
   including all CPU flag bytes in both emulation modes, partial SRAM tails,
   allocation/history topology and **22,710 identical diagnostic messages**.
   The earlier denied executable ran no cases and remains recorded separately.
   Final full boot any% **129712/0** and Speed Booster **148406/0** pass with
   normal rendering/audio, source fingerprint
   `d7c1ddfdc819e388baad82ca4f277e807d976a845a3e7df179ac19713d0a1a6a`.
   Neither reaches credits. Old verifier checkpoints invalidate through the
   existing source fingerprint. Copy performance is unbenchmarked. Durable source,
   component results and final main logs:
   Local/sm-decomp-experiments/20261021/live-snapshots.

   The complete native/SPC state-transfer replacement removes **230 lines
   (102,044 -> 101,814)**, 2,559 tokens and 7,553 spelling bytes. Thirty-two
   equal-width runs generate the same 283 player-map rows at compile time;
   one shared walker replaces both directions, preserving all 659 ordered
   copies and the final eight channel-index writes. All helpers and 82 source
   files count. Complete main release library/frontend and formatting pass.
   Source-bound ordinary DLL comparisons use actual production types and pass
   **1,040 cases / 685,360 copies per side**, comparing all 66,400 player bytes
   against both the old functions and a sequential-copy model. Parent separately
   matched all 330 compiled channel/player rows to committed source and checked
   every recorded old/new trace. Final full boot any% **129712/0** and Speed
   Booster **148406/0** pass with normal rendering/audio, source fingerprint
   `a7d82ec0f552ff40614d7cba0bb195ef057a33c307c23b46da4899c879a4798f`.
   Neither reaches credits. The first Speed Booster loader attempt omitted the
   SDL library environment and ran zero frames; its failure log is retained.
   Correcting that environment reused the same compiled verifier successfully.
   Sources, exact maps/traces and final build/replay logs:
   Local/sm-decomp-experiments/20261021/native-sequencer.

   Shared PPU pixel evaluation removes **681 lines (102,725 -> 102,044)**,
   6,053 tokens and 16,513 spelling bytes. The complete 2,109-line renderer becomes
   1,428 lines, with both selected rendering paths sharing tile decoding, Mode7,
   windows and color composition under explicit compatibility policies. All 82
   source files count; register access, sprite evaluation and save layout remain.
   Normal library/frontend release and independent source review pass. Ordinary
   source-checked DLL comparisons pass **7,616 cases / 175,040 scanline checks**
   across both paths, all eight modes, windows, sprites, color math, noncanonical
   saved fields and 256-row histories. Every output byte and other Ppu state byte
   compares except allocation pointers and the unused bgBuffers private cache.
   That cache stops updating; no remaining runtime reader exists. Its field and
   checkpoint serialization remain; existing source fingerprints invalidate old
   verifier checkpoints. Legacy saves and g_ram keep their layouts.
   Final full boot any% **129712/0** and Speed Booster **148406/0** pass with
   normal rendering/audio and source fingerprint
   `2df5b594d509743f3be3aeaba787add99673680dbce2aac0543d64373225f2e0`.
   Neither route reaches credits; shared-backend replays are supplemented by the
   independent old/new pixel comparisons. Dense native Mode1/Mode7 cost **2.69x /
   2.61x** the old PPU time (about 3.79/1.12 ms per 224 rows in the DLL benchmark).
   These are component timings, not frontend frame-rate guarantees. An eight-line
   inlining trial recovered only about 1% and is rejected. Source and checks:
   Local/sm-decomp-experiments/20261021/ppu-render-native, including main build and
   both final replay logs. Complete bank retirement remains separate work.

   Complete shader/presentation ownership removes **588 lines
   (103,313 -> 102,725)**, 4,877 tokens and 15,722 spelling bytes across three
   files. Owned vectors replace pass/parameter/LUT allocation and linked lists;
   shared stage compilation, uniform groups and scale operations replace repeated
   code. All callers and obsolete shared-type removal count. No dependency or
   gameplay change. The 83-line working shader draft is preserved in a durable
   backup and incorporated through the same Option-based parser behavior; the
   additional reduction from that working tree is 505 lines.
   Final normal library/frontend release build, changed-file formatting and
   progress pass. Source-checked ordinary DLL comparisons pass **528 shader +
   336 presenter cases**, matching 1,522,996 ordered events: committed/draft
   shader baselines, twelve-frame history, parser and compiler failures, actual
   presenter callbacks, resize/reuse, filtering and viewport policies. Ten test
   sources and 78 embedded library sources match the compiled bytes; all 82
   production files count. Independent source review passes. Controlled GL/SDL
   callbacks establish call equivalence, not real GPU pixels; allocation
   exhaustion is untested. Headless gameplay movies do not exercise this code.
   Durable candidate/checks: Local/sm-decomp-experiments/20261021/native-shader.
   Shader-library incompatibility findings are in REDUCTION_RESEARCH.md.

   Sharing constant-group headers removes **94 lines (103,407 -> 103,313)**,
   564 tokens and 2,162 spelling bytes, with no support additions. Forty-seven
   redundant headers in 13 gameplay files disappear; all 1,123 names, types,
   literal spellings, visibility and existing body rows remain. No denser rows
   or formatter-only savings count. Standalone optimized library/frontend builds
   and complete expanded correspondence pass, including all 7,109 library and
   84 frontend functions. Compiler witnesses evaluate 1,119 public constants;
   the four private constants have unchanged source/expanded definitions.
   Actual sequential emission on the alias-plus-string result passes full source
   inverse, same-cohort binding and exact byte reproduction. A composition-only
   newline fix consumes complete CRLFs and preserves each file's existing newline
   spelling; the earlier 83-line trial is excluded. Parent normal release,
   13-file formatting and progress pass; other counters remain unchanged.
   Constant declaration equivalence needs no replay. Durable rule/checks:
   Local/sm-decomp-experiments/20261021/constant-groups; actual final source:
   after-alias-strings-constants-v2. The three current rules remove 388 lines,
   1,870 tokens and 7,551 spelling bytes from the actual 103,701-line input.

   Native C-string literals remove **92 lines (103,499 -> 103,407)**, 470
   tokens and 1,913 spelling bytes, with no helpers. All 111 selected casts in
   17 files become native literals with exactly the same bytes, terminating NUL,
   pointer type and static lifetime. Embedded/repeated NULs and differing pointer
   types are excluded; the entire shader file stays unchanged. Paired optimized
   library/frontend builds, compiled literal assertions and complete expanded
   correspondence pass on the standalone rule. Re-emitting on the actual alias
   commit retains the same 111 proven expressions and passes a complete source
   inverse, preserving every alias and unselected operation. The composed result
   is measured directly; formatting alone removes zero lines. Parent normal
   release, 17-file formatting and progress pass, with all other counters unchanged.
   Static constant-data equivalence needs no gameplay replay. Source checkpoint
   key changes from 5bb9937931800caf to 8e19b756787affed; regenerate checkpoints.
   Durable rule/checks: Local/sm-decomp-experiments/20261021/native-c-strings;
   actual composition: native-c-strings-alias43df in the same directory.

   Shared alias declarations remove **202 lines (103,701 -> 103,499)**, 836
   tokens and 3,476 spelling bytes, including seven support lines. Thirty-two
   same-module groups preserve all 184 selected public aliases; no attributed
   alias, independent frontend declaration or shader source is changed. Complete
   source reproduction and independent source review pass. Compiler identity
   witnesses cover every selected alias, including unused aliases, and a wrong
   type is rejected. All 233 library and ten frontend alias declarations match;
   the complete ordered expansion and all 7,109 library/84 frontend functions
   retain their signatures and operations. Embedded source/checkpoint payloads
   are validated against their actual providers before normalization. Reused
   baseline source differs only in CRLF/LF spelling, explicitly checked. Normal
   main Cargo release, formatting and progress pass; no other counter changes.
   Static declaration equivalence requires no gameplay replay. Checkpoint source
   fingerprints change and old checkpoints need regeneration. Durable current
   rule and checks: Local/sm-decomp-experiments/20261021/alias-groups.

   Shared record metadata removes **271 lines (103,972 -> 103,701)**, 1,963
   tokens and 4,586 spelling bytes, including six added support lines. Sixteen
   explicit attribute groups cover 116 same-module records in 14 consumers;
   the existing single-record syntax delegates to the same emission body.
   All fields, types, packing, names and visibility remain unchanged. Exact
   source reproduction, full optimized library/frontend builds and independent
   compiler checks pass: 116 records, 348 derived implementations and the entire
   ordered remainder, including 6,993 library and 84 frontend functions.
   Parent independently checked 396 frozen artifacts, both complete source
   pairs and their counts. Source-registry/checkpoint payloads match their actual files before
   normalization. The source-derived checkpoint key changes and old checkpoints
   need regeneration. Parent Cargo release and 15-file formatting pass; no other
   progress counter changes. This is equivalent declaration spelling, with no
   runtime operations changed, so static-only validation needs no game replay.
   Historical composition independently removes 265 lines. Frozen rule/proofs:
   TEMP sm-tailored-next/record-metadata-groups; main build:
   TEMP sm-record-metadata-main-build-20261021.log.

   Shared RAM metadata removes **452 lines (104,424 -> 103,972)**, 4,210
   tokens and 16,206 spelling bytes, including 22 lines of support. The 1,169
   named records become 101 explicit base/stride/type/shape groups plus 231
   singletons. Every name, alias and row order remains explicit. Compile-time
   comparison covers the entire ordered table and all 981 current exported
   offsets; compiler expansion proves the exact name/type/visibility/order
   inventory, with no extra exports. Runtime access helpers and every caller
   remain byte-identical. Parent checked all frozen artifacts, source manifests,
   committed inputs and the zero-drift formatter control. Ordinary Cargo release
   and formatting pass. Static-only validation is appropriate: no runtime
   initialization, access operation or layout changes, so no replay was run.
   The corresponding actual historical stage removes 452 separately. Report:
   TEMP sm-tailored-next/ram-metadata-sequences; main build:
   TEMP sm-ram-metadata-main-build-20261021.log.

   Explicit compound RAM updates remove **370 lines (104,794 -> 104,424)**,
   2,586 tokens and 9,250 spelling bytes, including 41 support lines. All 801
   updates in 35 files preserve wrapping/checked arithmetic, truncation, old
   reads, RHS evaluation and 106 post-store rereads. The existing raw unaligned
   word arm still owns the accesses. Identical formatting alone changes zero
   bytes. Exact source reproduction and complete expanded/resolved compiler
   checks pass; verifier source payloads and checkpoint fingerprints are checked
   against their actual captured files before normalization. No owner is omitted.
   Normal Cargo release and changed-file formatting pass. Full boot any%
   **129712/0** and Speed Booster **148406/0** pass with normal comparisons,
   rendering and audio. Complete static correspondence covers unchanged
   operations; the replays check shared-RAM integration, not all 801 sites or
   credits. TEMP sm-tailored-next/compound-word-updates holds the source proof;
   TEMP sm-compound-adoption-20261021 holds final main builds/replays.

   Native inflater ownership removes **165 lines (104,959 -> 104,794)**,
   2,066 tokens and 4,638 code-text bytes. Four private owners replace 18
   translated functions, retaining EOF, reserved-code, overlap and C allocator
   policies. Independent source review and frozen test-source/hash checks pass:
   8,936 original-C/old/new inflater cases plus 3,080 full old/new PNG API
   comparisons. Final source matches the tested module exactly. Formatting,
   optimized library/frontend builds and normal Cargo release pass. The initial
   release link needed the existing SDL SDK's actual nested directory; no source
   workaround was needed. This frontend-only change requires no movie replay.
   Malformed inputs with inherited undefined behavior, huge allocations and
   zero-capacity growth remain outside coverage. The unrelated shader draft
   is preserved and excluded from the count. TEMP sm-native-inflater-20261021
   contains the checks; sm-inflater-adoption-20261021 holds final build logs.

   Enemy callback ownership removes **350 lines (105,309 -> 104,959)**,
   3,499 tokens and 29,375 code-text bytes. This resumes the existing 913-arm
   cross-file draft after its temporary resource hold, rebuilding from current
   bodies and retaining all 1,002 named callbacks. Its old unvalidated 346-line
   price is superseded, not added. Existing callback macros place numeric
   registration beside definitions, with 28 owner routers and unchanged central
   precedence, defaults and preinstruction index capture. All support counts;
   no shared macro or import is added. This is ownership support, not bank
   retirement. Complete library/frontend builds, 7,085 unchanged expanded
   functions, 4,727 storage/type declarations and full-u32 flattened routes pass.
   Another 381,250 debug/optimized dispatch cases compare full RAM, ordered
   target/arguments and pointer returns using instrumented terminal callbacks.
   Probed boot Speed Booster **21015/0** hits all six central/owner family
   probes, followed by 120 frames. Clean full boot any% **129712/0** and Speed
   Booster **148406/0** pass with normal rendering, audio and comparisons.
   The 82-file adopted source matches the clean replay root after CRLF/LF
   normalization, excluding the preserved unrelated shader draft. Neither
   movie reaches credits; grouped probes do not claim every owner executes.
   The cast-line counter rises one because the same Torizo two-cast expression
   wraps onto two lines; expanded casts are unchanged. Unsafe declaration lines
   fall through grouping, without changing expanded unsafe signatures.
   TEMP sm-enemy-callback-current records the complete change and checks.

   Shared gameplay import ownership removes **381 lines (105,690 -> 105,309)**,
   4,481 tokens and 17,050 code-text bytes. A 39-line prelude consolidates 955
   import entries across all 32 banks, including both registrations and every
   remaining local import. It imports shared owners only, with no bank-module
   dependency. All existing bodies and macro paths remain unchanged. Complete
   library/frontend builds, source inverse and expanded-operation checks pass.
   Resolved compiler references/types match across 12,645 library and 313
   frontend owners; only independently checked source-cache bytes are excluded.
   All 441 tested old public type paths and 4,962 constant paths retain their
   definitions. Public globs expose extra common names, an explicit visibility
   tradeoff with no changed existing consumer. This import-only change needs no
   replay under the validation strategy. It includes the sprite repair and
   overlaps earlier parked import studies; their savings must not be added.
   TEMP sm-tailored-next/gameplay-prelude contains the final 82-file source.

   The sprite experiment also repairs an actual current debug abort on packed
   five-byte ROM records. Eight word reads and two OAM writes in the two shared
   writers use unaligned operations, preserving native endianness and RHS-before-
   destination evaluation. This costs **five lines (105,685 -> 105,690)**.
   Complete source correspondence, library/frontend builds and formatting pass;
   265,737 actual current-Game component cases pass on disjoint ROM sources.
   Full boot any% **129712/0** and Speed Booster **148406/0** pass with three
   required sprite probes and all rendering/audio comparisons. Synthetic
   source/Game overlap fails in both raw and repaired current controls; the
   repair makes no general aliasing claim. All actual source providers are ROM.
   TEMP sm-tailored-next/sprite-drawing records that separate limitation.

   Complete Samus pose-sequence sharing removes **84 lines
   (105,769 -> 105,685)**, including the 21-line owner and both registrations.
   Three named routines replace 36 repeated sequences; two bank history
   wrappers and their seven direct calls are retired. Six unused imports
   disappear. All 7,080 remaining compiler-expanded functions match after
   exact helper inlining; both removed wrapper bodies match the history helper.
   The eight history-only callers retain no RAM pointer across the new call.
   Optimized library/frontend builds and formatting pass. Probed and clean
   boot any% intro checks pass **2339/0**, including all six required helper/
   animation/history/tile-effect probes and 120 follow-up frames. This route
   visits no gameplay rooms; unvisited callers have unchanged-effect proof.
   Final import-only cleanup has source-token correspondence and fresh builds.
   TEMP sm-tailored-next/sibling-sequences contains the frozen source/checks.

   Complete nonnull callback-table inference removes **71 lines
   (105,840 -> 105,769)**, including three helper lines. All 79 immutable tables,
   922 slots, the demo-set slice aggregate and 82 consumers preserve checked
   indices, exact function targets/casts and argument order. Four genuinely
   nullable tables remain unchanged. Complete compiler-expanded correspondence,
   final optimized library/frontend builds and formatting pass. Full boot any%
   **129712/0** and Speed Booster **148406/0** pass for this broad dispatch
   integration; unvisited entries are covered by static correspondence.
   TEMP sm-tailored-next/nonnull-tables contains the emitter and checks.

   Complete CPU/SPC tracing removes **144 lines (105,984 -> 105,840)**.
   One native formatter replaces C varargs and the sparse short-immediate
   table; all opcode text, padding, NUL termination, flags and bus-read order
   remain exact. All 742,400 original-C/initial/current/native component cases
   pass in debug/release Rust, including stateful reads and processor-pointer
   changes. Complete optimized library/frontend builds and static checks pass.
   This diagnostic-only change is checked through its actual formatting bodies
   and three caller contracts; no new gameplay replay is claimed. Reusable
   emitter and checks: TEMP sm-tailored-next/tracing-native.

   Complete beam-window ownership removes **205 lines (106,189 -> 105,984)**,
   including the 668-line `beam_windows` module, direct imports/callers and
   source registration. Shared directional setup and edge advancement retain
   corrected X-ray positive-down stores, saturation and ordered rainbow headers.
   All 64,100 actual old/new Rust full-RAM component cases pass. The Charge Beam
   boot route passes **28699/0**, with seven required probes including 1,645
   shared-edge calls, 74 generic-ramp calls and cleanup plus 180 follow-up frames.
   The clean focused run and full any% **129712/0** and Speed Booster **148406/0**
   pass. Complete optimized library/frontend builds and independent review pass.
   Mother Brain and other unvisited branches have constructed component
   coverage; no endgame movie execution or credits is claimed.

   Complete local-binding cohort: **207 bindings across 31 files**, placing
   declarations at their first value or binding an exhaustive branch result,
   removes **192 lines (112,261 -> 112,069)** with no new helpers. Scalar/raw
   pointer types, calculations, branch exits and effect order are preserved.
   Source reconstruction and independent scope/macro review, headless compiler,
   normal release and compiled-source checks pass. Static-only validation;
   no new movie result is claimed. This is useful landed repetition cleanup,
   but does not establish the desired multi-thousand-line replacement.

   Complete intro-page ownership removes **233 lines (112,069 -> 111,836)**:
   605 bank lines retired, a 367-line descriptive module, and five import/source
   registration lines. All 47 numeric callback entries retain their addresses.
   English/Japanese routes exercise the page-to-Ceres transition; focused oracle
   cases cover slot exhaustion and idle/delayed input. Full any% and Speed
   Booster pass; final unused-import retirement has static/compiler validation.
   This is a completed family replacement, not a forecast of the remaining gap.
   Routine results are in its commit.

   Enemy callback references now use their owning modules directly, retiring
   1,020 duplicate private imports. The complete cohort removes **159 lines
   (111,836 -> 111,677)**, including macro support and table wrapping at the
   existing 100-column width. Source reconstruction and compiler IR establish
   unchanged routing and operations; release and all-target checks pass.
   This removes declaration repetition, without changing gameplay ownership.

   Ordered named WRAM stores share the existing word macro across 97 runs /
   312 stores, removing **263 lines (111,677 -> 111,414)** including its helper
   and documentation. Independent reconstruction and whole-library compiler
   expansion establish identical executable operations and store order. Normal
   release and compiled-source checks pass; this is static-only validation.

   Complete PNG texture loading removes **1,119 lines (111,414 -> 110,295)**:
   a shared filter/unpack/pass loop, chunk parser and buffered input replace
   repeated decoding paths; the exact legacy inflater remains maintained.
   Counts include both modules, imports, caller and source registrations; no
   dependencies are added. All 385 frozen cases match through decoding and file
   loading (770 comparisons), including unusual formats and malformed-stream
   quirks. Independent input-operation review, normal release and compiled-source
   checks pass. This frontend-only change needs no gameplay replay.

   Complete beam-firing ownership removes **65 lines (110,295 -> 110,230)**:
   charge input, reservation, uncharged/charged/hyper firing and flare reset
   share a 191-line module, with the old entry points retired. All callers,
   imports and source registrations are included. Focused SNES comparisons
   cover exceptional firing and subsequent projectile calls; full any% and
   Speed Booster pass on the clean release. Hyper coverage is constructed,
   not an endgame movie. This domain migration does not fund the remaining gap.

   Complete moving-platform/shutter ownership removes **192 lines
   (110,230 -> 110,038)**: 650 old lines become a 448-line module plus ten
   caller/registration lines. Named phases share triggers, travel, waits and
   timed growth while preserving carrying, pushing and shot differences.
   All 16,414 constructed full-RAM comparisons pass, including persistent
   updates and wrapping boundaries. Full integration and observed branch
   limits are below. This retires about 30% of the old family, not a full bank.

   The complete byte-partial-product cohort removes **71 lines
   (110,038 -> 109,967)**: six decomposed products become native multiplication,
   and two duplicate sine-scaling helpers become one with eight callers.
   Explicit magnitude/sign, selected product byte and low-amplitude truncation
   preserve the original arithmetic. Independent actual-function/RAM checks,
   normal release and full any%/Speed Booster integration pass. This is a
   completed repeated algorithm, with all helper/import/caller costs included.

   INI section/value decoding removes **65 lines (109,967 -> 109,902)**:
   section lookup, boolean suffix validation and keyboard/gamepad group lookup
   share their repeated algorithms. All 354,060 frontend cases pass against
   the original decoder, including normal-library file loading, invalid values,
   aliasing outputs and the accepted control-byte booleans. Release, formatting
   and all-target checks pass. This frontend-only change needs no movie replay.

   Complete countdown-callback sharing removes **198 lines
   (109,902 -> 109,704)** across 97 callbacks in eight banks, including the
   shared declaration helper. Exact setup, capture, expiry, effects and tails
   remain explicit; callback names and routes are unchanged. Independent
   compiler expansion matches all 97 bodies after bound-local renaming;
   outside selected spans and macro resolution remain unchanged. Formatting,
   normal release and compiled-source checks pass. This is static equivalence,
   with no new movie result. Lower unsafe/cast counters reflect shared source
   declarations, not removal of unsafe runtime operations.

   Complete beam/missile flight ownership removes **148 lines
   (109,704 -> 109,556)**, including the 331-line module and every caller/import.
   Shared direction routing, trail timing, acceleration and launch axes retire
   28 bank entry points while preserving all 35 numeric dispatch addresses.
   All 51,791 focused SNES comparisons pass, including 180 sustained-flight
   scenarios and inherited launch motion. All 15 requested component probes
   execute; focused movies and clean full integration pass as recorded below.
   The remaining projectile instruction, drawing and firing owners are separate.

   Complete callback entry-capture sharing removes **482 lines
   (109,556 -> 109,074)** across 754 callbacks / 67 groups, including seven
   shared support lines. Independent source reconstruction and compiler
   expansion preserve every selected callback and all 7,006 current named
   definitions exactly, with no binding renames. Formatting, normal release
   and compiled-source checks pass. This is static equivalence, with no new
   movie result. Lower unsafe/cast counters reflect shared declarations;
   no unsafe runtime operations disappear. Continue the structural/variant
   audit rather than repeating the completed prefix/signature census.

   Complete CPU/SPC opcode-grid consolidation removes **184 lines
   (109,074 -> 108,890)** by decoding repeated operation/address combinations.
   All 512 opcode selections independently preserve helper arguments, order
   and fallback behavior; all 134 existing helpers remain unchanged. The 8,192
   compiled selector-boundary traces pass, as do release/source checks and full
   any%/Speed Booster replays. The two added unsafe functions are shared
   selectors; all their costs are included. These checks preserve existing
   arithmetic/bus operations without claiming every opcode executes in movies.

   Complete blast-window ownership removes **90 lines (108,890 -> 108,800)**,
   including the 539-line module and all caller/registration costs. Two drivers
   replace five scaled/prescaled phase bodies; all 17 numeric routes survive.
   Independent review preserves 20 moved definitions, and all 4,700 focused
   SNES comparisons pass, including persistent phase progression. The two
   additional unsafe functions are shared phase helpers. Ceres exercises every
   Power Bomb phase and cleanup; Crystal Flash has constructed SNES coverage.

   Complete indexed-loop recovery removes **367 lines (108,800 -> 108,433)**
   across 132 loops in 113 functions / 29 files, with no new helpers. Typed
   ranges replace repeated counter setup, updates and termination tests.
   Independent source/body correspondence and compiled control comparisons
   match every visited index and normal termination, with unchanged early exits.
   Formatting, release/source checks and full any%/Speed Booster replays pass.
   Unvisited gameplay bodies rely on that static/control equivalence; the
   movies do not establish execution of all 132 loops.

   Complete callback/dispatch declaration sharing removes **913 lines
   (108,433 -> 107,520)** across 114 dispatchers and 1,925 named functions,
   including 52 support lines. One declaration supplies the function and its
   numeric route, retiring 1,752 separate registration arms. Independent source
   reconstruction and compiler expansion preserve all 7,012 named definitions
   and complete dispatch mappings, including defaults and argument order.
   Formatting, normal release/source checks and both full replays pass.
   Existing forwarding, prefix and countdown representations remain shared;
   body indentation/wrapping costs and every retained exception are included.

   Keep one active implementation per overlapping subsystem. Use a stable
   source snapshot and checkout-local build cache for its checks. Design
   globally and land coherent stages; do not force every individual call site
   through a separate build/replay/commit cycle. Follow the validation strategy
   below, requiring demonstrated coverage of changed code and its effects.
   Continue choosing the strongest
   complete replacement until below 100,000; then reassess the path to 90,000.

4. [ ] **Retire bank ownership by gameplay responsibility.** Select a complete
   responsibility, such as Samus movement, with an exercised boundary and named
   routines. Migrate its real callers and state transitions, then remove its
   retired bank facade.
   The [interactive architecture atlas](architecture.html) maps current source
   to proposed Super Metroid responsibilities, existing sharing and frame order.
   It is a design aid; proposed interfaces are not adopted replacements or savings.
   User-requested [RAM ownership research](RAM_OWNERSHIP.md) now traces ten
   producer/consumer contracts and separates live values, captured selectors,
   cross-update handoffs and nested continuations. The first implementation moves
   ordinary gameplay scheduling to `gameplay::advance`, migrates all real callers,
   and names its resume points while replacing the retained raw continuation
   pointer with checked access. The complete ownership cost is **+55 Rust lines**;
   it is not a volume reduction or elimination of the remaining unsafe callees.
   Actual scheduler bodies pass 18,400 full-RAM/return/ordered-call comparisons
   and 18 persistent steps. Four snapshot counterexamples and projectile selector
   experiments establish why value lifetimes must remain explicit. Probed and
   clean isolated runs pass **8359/0**, with 735 callback observations across
   Ceres inputs **8324..8358**; the earlier prefix is setup. No normal player-control,
   message-route, later-room or credits coverage is claimed. The cited research
   records harness limits, observed shared fields and the next experiments.
   Durable work: Local/sm-decomp-experiments/20261022/ram-ownership.
   Renewed user direction (2026-10-21): the search is still too narrow. Prioritize
   substantial system replacements and repeated algorithms across responsibilities.
   Stop spending implementation/validation effort on roughly hundred-line enemy
   extractions or adding macros to trim local branches. Small preparatory edits
   must belong to a concrete larger replacement with a measured complete price.
   Safety is now a required outcome alongside volume reduction. Start with
   complete access paths whose retained-pointer and callback contracts are known;
   expand the shared RAM/ROM interfaces as their real callers migrate. The earlier
   direct typed-RAM-prefix draft is not a safety solution: it introduces raw byte
   places and 87 unsafe scopes. Enemy-field flattening also retains raw pointers.
   Do not revive either merely because it exposes named fields.
   Cart and Game now share one owned ROM image. Game byte/word readers and both
   cartridge ROM bus-read paths use checked access; registration is safe and the
   image stays alive across cartridge replacement. The image module forbids unsafe
   code. SRAM, raw input copying, patch pointers and hardware ownership remain
   unsafe; this is not completion of ROM safety. The complete change adds **28
   maintained lines**, a safety cost with no new macros. Six actual Cart load/reload
   scenarios and actual initialization/patch/read comparisons pass, including full
   ROM/SRAM, all Game RAM and five exact bounds diagnostics. Seven ownership checks
   pass. The final merged release-library consumer repeats these checks; all 98
   library inputs and 79 installed-native source/stale checks pass. Normal release
   library/frontend/verifier builds pass. No movie ran: the synthetic component
   scenarios execute the changed reads, lifecycle, patching and bus consumers.
   Allocator-event identity, physical hardware and gameplay-route coverage are not
   claimed. Checks: Local/sm-decomp-experiments/20261022/owned-rom.
   The existing shared ROM-table interface now uses that owned image with checked
   signed offsets and byte reads, preserving image capture before index evaluation,
   live patch visibility, native byte order and linear bank crossings. Its 725
   source invocations include uses inside macro templates and the migrated direct
   equipment-selection call. No macro is added. Complete cost: **+1 maintained
   line** across five files, a safety cost rather than a reduction. Exact-source
   checks pass 3,000 table cases (1,598 unaligned), five evaluation/mutation cases
   and seven safe/failure checks. Actual release-library equipment, palette and
   mechanism-initialization callers pass 644 full-RAM/return comparisons, repeated
   on the final library with all 99 source inputs checked. Initialization of all
   four mechanisms now forbids unsafe code. Normal release builds pass. No movie
   ran; unexecuted parameterized callers have unchanged expressions and full
   compilation, not an individual execution claim. Other raw ROM consumers remain.
   Checks: Local/sm-decomp-experiments/20261022/checked-rom-tables.
   Asset decompression now has one safe byte decoder and a checked RAM-offset
   entry point, used by all room, graphics, menu and boss callers. The module
   forbids unsafe code; its input reads the owned ROM image. One output loop
   handles literals, fills, alternating/incrementing runs and overlapping absolute
   or relative references. The existing unsafe VRAM adapter preserves its register
   protocol. All callers, registrations and the adapter included, this removes
   **8 maintained lines** and adds no macros. Source-checked old/new comparisons
   pass 1,621 full-RAM/PPU/normalized-Snes/cursor/failure cases and 672,705 ordered
   register observations, using the real PPU backend. The actual release-library
   Kraid graphics and room tiles/palette callers pass another 256 full-state cases;
   all 98 old/99 final library inputs match. Checked out-of-range writes fail after
   the valid prefix; lower-half ROM warnings preserve failure state and exact
   Windows stdout bytes. Normal release library/frontend/verifier builds pass.
   No movie ran; synthetic streams establish no gameplay-route or physical-device
   coverage. Checks: Local/sm-decomp-experiments/20261022/safe-decompression.
   All 199 shared word-transfer invocations now use checked RAM byte places,
   owned ROM images and typed tables. The module and expanded transfer loop
   forbid unsafe code, preserving lane order, directional overlap, captured
   values, argument temporaries and native/little-endian interpretation. Three
   existing palette tables change signedness without changing their bytes. The
   odd cinematic ROM address now has defined byte access. Complete cost is **+25
   maintained lines**, including callers. Source-checked comparisons pass 1,864
   byte-level cases and 240 actual release-library menu, grapple and statue/palette
   caller cases, with all 99 library inputs checked. Evaluation/lifetime, bounds
   and exact 280-byte diagnostic checks pass; normal release builds pass. No movie
   ran. Full PLM slots exercise the real no-slot branch, not spawned PLM behavior.
   Legacy endpoint expressions and C diagnostic calls remain unsafe outside the
   checked transfers. C buffering is preserved; moving that output to Rust changed
   ordering and was rejected. Checks: Local/sm-decomp-experiments/20261022/safe-word-transfers.
   VRAM readback now owns all five X-ray/Kraid request variants and the NMI
   consumer in one module. Checked byte access replaces every packed queue
   pointer, and the obsolete record type is removed. The shared producer and
   three direct gameplay entries forbid unsafe code; hardware register calls
   remain explicit unsafe operations. Overlapping writes, the retained DMA byte,
   low-only sentinel/tail clearing and DMA changes to later records are preserved.
   Complete cost: **+1 maintained line**, including callers and registrations.
   Actual-Game comparisons pass 674 producer/consumer cases and 3,530 ordered
   register observations, then 264 actual release-library caller cases including
   24 producer-to-NMI readbacks. All 100 old/101 final source inputs and normal
   release builds pass. Existing unaligned-pointer requirements are eliminated;
   old optimized execution is observed behavior, not a proof of undefined accesses.
   No movie ran; unrelated overlay logic and physical hardware are not covered.
   Checks: Local/sm-decomp-experiments/20261022/safe-vram-read-queue.
   ROM sprite emission now uses one owned-image decoder and checked OAM writer
   for all eleven menu, Samus, projectile, enemy and cinematic drawing adapters.
   Five clipping policies preserve their distinct write order, wrapping, tile
   transforms and packed extension bits. The new module forbids unsafe code;
   native lower-bank warnings remain explicit in the bank-81 source selector.
   The raw-pointer helper has only two current callers and is now private with
   an owned source. Existing public drawing adapters remain available. Complete
   price: **+7 maintained lines**, including adapters and registrations, with no
   new macros. This is safety work; it establishes no large volume saving.
   All 1,872 actual old/final comparisons pass, including empty maps, odd and
   overlapping OAM addresses, clipping edges, multiple wraps and 22 actual NMI
   uploads. The latter assert all 544 PPU OAM bytes and compare typed machine
   state. Five branch probes, 245 exact diagnostic bytes and a checked ROM-end
   failure pass. All 108 unaffected ordinary bank-81 functions retain exact source.
   Final release library/frontend/verifier builds and all 102 library source
   inputs pass. No movie ran; the synthetic maps establish no gameplay-route,
   visible-pixel or physical-device coverage. Checks:
   Local/sm-decomp-experiments/20261022/owned-sprites.
   Recording now owns its log and base snapshot in checked Rust storage, with
   safe input/patch encoding, rebasing and stopping. The new module forbids unsafe
   code; native ByteArray calls, declarations and installer selections are removed.
   Logical shrink retains initialized tails, including replay reads beyond the
   shortened log length. RAM patches still wrap and mirror one byte at a time;
   native file/snapshot callbacks and that external mirror remain unsafe. Complete
   cost: **+9 maintained lines**, including 22 Rust lines, minus 12 C/header and
   one installer line. No new macros. Actual old/final comparisons pass 486 buffer,
   codec and replay cases, plus 104 save/load, clear/rebase, frame/cheat, SRAM and
   checkpoint cases. Two checked failure/recovery cases pass. The initialized
   synthetic hardware fixture compares complete file, RAM and continuation bytes;
   version-2/legacy headers, SRAM/full bases, APQ1/ABQ1 trailers and short reads are
   covered. Final library/frontend/verifier builds and all 100 embedded library
   inputs pass; dependent configuration/shader and desktop archives are rebuilt.
   No movie ran; gameplay, physical devices and allocator-event identity are not
   claimed. This safety change establishes no large volume saving. Checks:
   Local/sm-decomp-experiments/20261022/owned-recorder.
   Pose-history recording now copies the two four-byte records using checked byte
   places, preserving the previous record before replacing it. It eliminates four
   raw word reads and four raw word stores, and its entry point is safe. Direct
   indexing avoids adding a borrow of the entire RAM array. Its one-line source
   increase is a safety cost, not a reduction. Other pose transitions remain unsafe.
   Source-checked old/new fanfare callers pass 1,841 full-RAM/return comparisons,
   including 1,052 executions of the changed history path; release builds pass.
   No movie ran. Checks: Local/sm-decomp-experiments/20261022/safe-pose-history.
   Moving platforms and shutters now retain RAM offsets across callbacks and use
   checked byte places for all their RAM reads/writes, including extended fields
   and low-before-high movement updates. Live slot/index rereads are preserved.
   Five helpers and initialization are safe and enforce `forbid(unsafe_code)`;
   the owner denies implicit unsafe operations. Its remaining explicit unsafe
   scopes cover shot and failure handling. The legacy shared `word!` stays
   unchanged; only this migrated consumer uses the separate checked interface.
   The complete migration adds **203 maintained lines** (165 in the owner, 38 in
   shared access), a safety cost rather than a reduction. Full-RAM/completion and
   ordered-boundary comparisons pass 6,536 actual old/new cases, including 4,518
   callback observations and 428 matching failures. Another 47 checked-access
   cases cover odd addresses, signed indices, wrapping and evaluation order.
   Actual enemy-dispatch checks pass 480 full-RAM cases; final release builds pass.
   No movie ran. Collision, sound, damage and failure calls in the mechanism
   component are controlled boundaries; no gameplay-route/hardware coverage is
   claimed. Checks: Local/sm-decomp-experiments/20261022/safe-mechanisms.
   A complete room-object ownership draft on 0199ea48 prices **95,661 -> 95,581
   (-80)**, including all 19 changed files, helpers, retained scripts, callers and
   registrations. It shares 25 station/gate/breakable-block handlers but still uses
   unsafe calls. Park this volume-only proposal before compilation/runtime checks;
   it supplies no safety or adopted-reduction claim. Durable draft and price:
   Local/sm-decomp-experiments/20261022/room-object-model.
   The latest data-extraction goal also calls for authored room/asset definitions
   to live as data. Door-entry scroll changes now use 63 ordered cell/color
   sequences for all 74 dispatch identities, removing separate callback definitions,
   aliases and registrations. The complete price is **95,705 -> 95,661 (-44)**,
   including the data module, executor, caller and source registration. All 144
   ordered writes and retained bank operations match; 666 actual old/new dispatcher
   cases, 150 door-loading caller cases and 65,982 lookup/full-RAM cases pass.
   Final release library/frontend/verifier builds pass. No movie ran; constructed
   door records establish no camera, hardware or gameplay-route coverage. This
   modest extraction retires no bank and establishes no large deletion budget;
   substantial responsibility replacements remain the primary work queue.
   Durable source and checks: Local/sm-decomp-experiments/20261022/room-scroll-data.
   Complete desktop host ownership now uses the pinned Zelda3 dependency,
   retaining Rust GameOwner boundaries, verification and the OpenGL presenter.
   The complete price is **96,402 -> 95,705 (-697)**: Rust loses 1,010 lines;
   C/header adapters add 72, the installer 240 and the verifier launcher one.
   All 125 actual old/new host scenarios and 33 final installed-archive checks
   pass. Final release library/frontend/verifier and source checks pass. No movie
   ran; controlled SDL/runtime boundaries establish no physical-device or gameplay
   coverage. This supersedes the earlier partial -193 native frontend proposal.
   The 882-line published desktop input is disclosed separately; existing headers
   are shared. Details: DESKTOP_DEPENDENCY.md. Durable candidate and checks:
   Local/sm-decomp-experiments/20261021/host-dependency.
   Complete configuration/input and file/string/ByteArray ownership now uses
   the independently published Zelda dependency with explicit compatibility
   patches. The complete replacement prices **96,947 -> 96,406 (-541)**:
   Rust loses 808 lines; C/headers add 99, installation adds 159 and verifier
   launcher changes add nine. All callers and retained declarations count.
   The standalone configuration draft's -382 is included, not additional.
   Focused old/new state, diagnostics, input lookup, file, allocation and failure
   checks pass, as do the actual installed-library consumer and normal release
   builds. No movie ran. The direct verifier's missing native-library search
   path was repaired in two of those launcher lines. Dependency sources,
   installation, coverage and limits are in CONFIG_DEPENDENCY.md. Durable work:
   Local/sm-decomp-experiments/20261021/config-dependency.
   The complete native/oracle backend assessment now replaces shared SPC
   instruction execution and PPU register operations with independently published
   LakeSnes dependencies. All native consumers, bus callbacks, state layout,
   compatibility adapters, installers and verifier changes are counted. SPC
   removes **676 Rust / 632 complete maintained lines**; the composed PPU stage
   removes **340 Rust / 259 complete maintained lines**. Together they remove
   **1,016 Rust / 891 complete maintained lines** before the DSP stage below.
   SPC uses revision `1da16a2`; PPU registers use `28b8fbc`. Their separate pins,
   licenses and installation are documented in SPC_DEPENDENCY.md and
   CPU_DEPENDENCY.md. The earlier combined prototype supplies no extra savings.
   PPU preserves line-zero mosaic setup, full routing bytes and temporary
   beam/open-bus views, with 77,824 final installed-archive comparisons. The
   renderer, DMA, scheduling, snapshots and CPU hooks remain maintained and
   included in the price. The DSP synthesis replacement now preserves immediate
   KON/KOFF and full-buffer behavior around the same unmodified `28b8fbc`
   dependency. It removes **478 Rust lines**, adds **50 C bridge / three installer
   lines**, and removes **425 complete maintained lines**. All 66,590 actual-code
   state/RAM/failure/PCM comparisons and probed/clean native and physical-audio
   boot **3/0** segments pass. Invalid envelope/FIR indices retain Rust's partial
   state and failure instead of reaching unchecked foreign indexing. Existing
   allocation, snapshots, reads and resampling retain their Rust operations.
   The DSP dependency adds 685 external C/header lines, separately disclosed in
   CPU_DEPENDENCY.md. Current `snes/` owns 2,849 Rust lines; these are a gross
   retained scope, not a deletion budget. Durable DSP source/checks:
   Local/sm-decomp-experiments/20261021/dsp-dependency.
   A direct whole-PPU renderer substitution remains unsuitable: native VRAM
   addressing, layer priorities, windows, Mode7, sprite storage and snapshot state
   differ from upstream. Inspection establishes no complete compatibility-layer
   price or saving; no whole-renderer candidate was built or replayed.
   The bounded legacy pixel-query replacement is now adopted: layer selection,
   tile decoding, offset-per-tile traversal and Mode7 lookup use the existing
   unmodified PPU dependency. Native rendering, sprite evaluation, color math,
   scanline output and snapshots remain local. Adapting one packed sprite cell
   restores the obsolete buffers before returning. Invalid modes still fail in
   Rust. The replacement removes **269 Rust lines**, adds **13 C bridge lines**
   and removes **256 complete maintained lines**, with no added dependency code.
   All 85,814 actual-code comparisons and probed/clean legacy-renderer boot **3/0**
   checks pass. Native tile decoding retains base-before-column/plane wrapping;
   the other 21 retained functions have exact source correspondence. Details and
   coverage limits are in CPU_DEPENDENCY.md. Durable source/checks:
   Local/sm-decomp-experiments/20261021/ppu-pixel-dependency.
   The complete DMA dependency draft prices **97,313 -> 97,315 (+2)**: 16 Rust
   lines removed, 16 C bridge and two installer lines added. Initial zero-count
   HDMA termination stays local; preserving invalid-index failures, partial state
   and the ROM-source warning retains transfer fallbacks. Live bus callbacks need
   a nested, unwind-safe Game context. These costs consume the engine substitution's
   saving. Park this formatted, uncompiled and unexecuted draft; no runtime or
   compatibility claim is made. It would add 379 external DMA C/header lines.
   Source and complete price: Local/sm-decomp-experiments/20261021/dma-dependency-price.
   The complete special-attack lifecycle draft prices **97,313 -> 97,186 (-127)**.
   It replaces Wave/Ice/Spazer/Plasma launch, orbit/flight phases, trails, clipping,
   termination and charge-spread bombs with one owner and shared named routines;
   no new macros. Bank 90 loses 617 lines, the owner adds 484, and real callers
   and registrations add six. The 623-line original scope does not yield a
   substantial replacement saving. Park before compiler/runtime validation under
   the renewed larger-scope direction. The draft is formatted, uncompiled and
   unexecuted; no behavior, movie-coverage or adopted saving is claimed.
   Source and complete price: Local/sm-decomp-experiments/20261021/special-attacks.
   Retiring the parallel safe-word accessor interface prices **97,313 -> 97,126
   (-187)**. Two shared methods use existing RAM-map addresses and replace all
   235 field-specific methods without new macros. Their 764 real call sites
   across 16 changed files absorb 237 of the 424 declaration lines removed.
   Park this formatted, uncompiled and unexecuted draft as another small return;
   no maintained source change or compatibility/runtime claim. Durable draft:
   Local/sm-decomp-experiments/20261021/safe-ram-interface.
   Broadening DMA to the existing LakeSnes bus/scheduler does not establish a
   drop-in: current synchronous DMA, unconditional Vblank NMI, unchanged open bus,
   direct joypad reads and native audio routing differ. No combined price or
   runtime claim is established. The independently published Zelda hardware fork
   and its early fast-renderer revisions also differ in layout, sprite storage,
   rendering width and Mode7 priority. Downloaded hardware-only inputs:
   Local/sm-decomp-experiments/20261021/zelda-hardware-dependency.
   A compatibility probe of historical renderer revision `1f8985903a32` now
   resolves its interlace and Y=240 sprite differences. A 14-line C helper
   maps eligible source rows onto synthetic scanline 128 and hides ineligible
   sprites; the unmodified foreign evaluator still performs tile decoding,
   clipping, priority and limit handling. All **1,198 actual sprite-buffer and
   flag comparisons** pass across all sizes, both interlace fields, row edges,
   clipping, rotation, retained overflow flags, forced blank and VRAM spill.
   All **68 original row comparisons** now pass with foreign sprite evaluation,
   including the four original counterexamples. Another **284 of 290** drawing
   variants match across matrix transforms, inverted/empty windows, brightness,
   color math and VRAM boundaries. Six expose a remaining native Mode7 mosaic
   cache mismatch: 88..145 visible pixels differ when the native mosaic flag is
   set with size one and a zeroed cache. The early foreign revision instead
   selects mosaic from legacy BG flags and size greater than one. All sprite
   buffers/flags match in those six cases, isolating this rendering difference.
   The source-checked DLL uses the exact current release library and pinned,
   unchanged foreign inputs, with C assertions enabled. These are constructed
   line-one drawing cases and separate sprite scans through line 255; they do
   not establish full-state, failure, lifecycle, extended-width or movie coverage.
   The 14-line helper is only a partial adapter cost, not a saving. Sprite probe:
   Local/sm-decomp-experiments/20261021/renderer-compatibility. The original
   counterexamples remain in Local/sm-decomp-experiments/20261021/zelda-render-probe.
   A composed renderer draft now prices **97,313 -> 97,324 (+11)**. It pairs
   that sprite dependency with native drawing from `45a149d8b4bb`, retaining
   Rust's cached-mosaic path, legacy rendering, lifecycle and snapshot layout.
   Rust loses 205 lines; C adapters add 161, the installer 54 and the verifier
   launcher one. All callers, native/C layout checks and source-hash changes
   count. The additional published C/header inputs total 4,268 lines separately.
   The later drawing adapter passes all 68 original row cases with checked
   native/C layouts and source hashes. Its initial 766-pixel scratch width
   crashed because the published backdrop loop advances by four until exact
   equality; that attempt is excluded. The corrected 768-pixel physical buffer
   reserves space for native u8 margins. A focused windowed Mode7 case still crashes:
   size one and zeroed cached entries produce a negative initial span width at
   x=129, while current Rust renders successfully. The composed draft therefore
   retains the native mosaic implementation and its failure rules; those retained
   costs are included, and no complete native-renderer retirement is claimed.
   Park the final formatted, uncompiled and unexecuted draft before further
   validation: its adapters consume the deletion. The 68 passing probes do not
   validate the final fallback integration, extended width, full state or
   lifecycle. No movie ran and no saving is adopted. Source, failed probes and
   complete price: Local/sm-decomp-experiments/20261021/renderer-complete.
   The complete native music/SFX sequencer cannot directly reuse the uploaded
   SPC driver. A source-checked DLL probe executes 32 actual native/interpreted
   ticks from aligned entry state: all 269 ordered DSP writes and final DSP
   states match, but mapped driver state and RAM differ. Changing one uploaded
   instruction operand leaves native DSP writes/state and compared native fields
   identical across another 32 ticks; interpreter writes first differ at tick
   eight, and DSP state differs at tick 32. Native behavior is fixed in Rust;
   executing uploaded instructions introduces a dependency it currently lacks.
   Park this direct substitution before adapter pricing. No complete replacement
   saving is established. A future interpreter adapter must preserve that fixed
   behavior as well as native state, initialization and upload contracts; a
   fallback retaining the complete native sequencer earns no deletion.
   These are constructed single-tick comparisons, each seeded from native state,
   with explicit idle/sound inputs and no DSP advancement. They do not validate
   continuous execution, queue adaptation, PCM, physical upload/timing or full
   snapshots. No movie ran. Probe and results:
   Local/sm-decomp-experiments/20261021/native-sequencer-spc.
   The published Zelda music sequencer at `45a149d8b4bb` is now adopted through
   an explicitly maintained compatibility patch. Direct substitution failed on
   native commands and data layout; the complete replacement supplies native
   pitch envelopes, tremolo, fade completion, song/port behavior, table addresses
   and failure ordering. SFX, scheduling, uploads, snapshots and shared native
   primitives remain charged. The module changes **1,681 -> 1,003** lines;
   all C/header, installer, registration and verifier costs give **97,313 ->
   96,947 (-366)**. No new gameplay macros or hidden project implementation are added.
   Actual old/new full-state, ordered-write, failure and continuous PCM checks
   pass, followed by probed/clean **179/0** native-music boot runs. Active notes
   and effects first execute at frame 173, PCM at 174, with four follow-up frames;
   earlier inputs are setup. Echo and other unvisited branches have component
   coverage. No full replay or gameplay coverage is claimed. The patch selects
   only music routines; the entire 1,659-line pinned C/header input is disclosed
   separately. Installation and limitations: SPC_DEPENDENCY.md. Local draft,
   complete price and comparisons: Local/sm-decomp-experiments/20261021/patched-native-music.
   A current-Rust music-loop rewrite also fails the larger-savings test:
   shared pattern readers, the full order/track loop, channel/global fades and
   reset clearing price **97,313 -> 97,269 (-44)**, including all helpers and
   retained code. The complete sequencer module changes **1,681 -> 1,637** lines;
   its baseline is unchanged by the same formatter. SFX execution, public
   wrappers, state layout and lifecycle remain charged; no full SFX rewrite is
   claimed. Park this representation before extending it or running compiler/
   runtime checks. It adds no macros or dependencies and is not adopted.
   Draft and price: Local/sm-decomp-experiments/20261021/native-sequencer-rewrite.
   The complete native cartridge draft prices **97,313 -> 97,208 (-105)**:
   loading/header parsing falls from 273 to 177 lines and mapping/lifecycle
   from 156 to 147. Shared header scoring and SRAM address selection retain
   the public callers, allocation/reset order and LoROM crash diagnostics in
   the draft; F0 remains readable SRAM but excluded from LoROM writes. There
   are no new macros, dependencies or uncounted helpers. Park this formatted,
   uncompiled and unexecuted replacement under the larger-scope direction;
   no behavior, movie coverage or adopted saving is claimed. The separate
   LakeSnes cartridge preflight also supplies no replacement price: SRAM reset,
   invalid-read handling, successful-load output and oversized header shifts
   require compatibility work. No foreign cartridge code was built or run.
   Native draft: Local/sm-decomp-experiments/20261021/cartridge-native;
   pinned foreign inputs: Local/sm-decomp-experiments/20261021/cartridge-dependency.
   Equipment, multipart-geometry and remaining frontend-ownership preflights
   establish no substantial replacement to implement. Equipment initialization,
   activation and search already share their algorithms; fixed navigation tables
   and mutable activation descriptors cannot become one cached item model.
   Botwoon history and Ridley joints already share traversal; the other examined
   creatures retain different geometry and transition rules. Frontend ownership
   retains both rendering protocols and event policies; automatic cleanup would
   change failure behavior. Its existing **-193** is only a partial, unvalidated
   price, with allocation semantics unresolved. Park these specific proposals;
   no new draft, complete price, build or runtime result is claimed. Multipart
   findings: Local/sm-decomp-experiments/20261021/multipart-geometry;
   partial frontend draft: Local/sm-decomp-experiments/20261021/desktop-frontend.
   The broader weapon-firing lifecycle preflight covers 2,459 gross lines,
   including 451 overlapping the parked special-attack scope. Reservation,
   rollback, ammo consumption, counter mutation and callback rereads differ
   across weapons; flight, animation and trail algorithms already share code.
   This common reservation/emission proposal establishes no substantial net
   replacement. No draft, build or runtime check was warranted.
   The complete published-shader replacement is now adopted and composed with
   desktop ownership and room-scroll data: **95,661 -> 95,379 (-282)**. Rust owns
   source vectors and PNG temporaries, C scope cleanup handles recursive files
   and paths, and destruction separates GL deletion from owned-memory release.
   The existing GL loader supplies pointers without additional SDL lookups or
   forwarding wrappers. Rust loses 572 shader lines; two registration, 161 C/header
   and 127 installer lines count against that deletion. No new macros are added.
   Normal and instrumented archives pass 38 actual old/new state, ordered GL,
   pixel and diagnostic scenarios plus 46 loader/fallback/reload cases. Four
   source-allocation failure checks preserve returning/unwinding host behavior
   and release all temporary allocations. The candidate-only heap guard check
   injects failure at its declared C-unwind boundary; it supplies no guarantee
   of recovery through the original non-unwinding GL ABI. The actual final
   release-library consumer passes presenter initialization, 22 source-stamp
   checks and five shader scenarios, with 96 library inputs source-checked.
   Final library/frontend/verifier builds pass. No movie ran; physical GPU,
   concurrency and gameplay coverage are not claimed. The 659 C / 99 header
   external input remains separately disclosed. See SHADER_DEPENDENCY.md.
   Durable source and checks:
   Local/sm-decomp-experiments/20261021/shader-dependency/ownership-design.
   The standalone published recorder proposal also remains unpriced: its codec
   corresponds closely, but native version-2 headers, SRAM replay bases, APQ1/ABQ1
   trailers and clear-log audio preservation need substantial maintained code.
   The 467 recorder/helper and 41 ByteArray function lines inspected are gross
   scope. No candidate, build, runtime result or saving is claimed. Findings:
   Local/sm-decomp-experiments/20261021/recorder-dependency/PREFLIGHT.md.
   Continue selecting larger complete responsibilities and repeated algorithms;
   the 90,000-line stretch checkpoint still has no demonstrated remaining budget.
   Durable experiments: Local/sm-decomp-experiments/20261021/complete-backend.
   SPC passes 528,395 actual old/new full-state/cycle/failure comparisons and
   2,028,312 ordered bus events, including reset/save, sequential self-modification
   and callback unwinding. See [SPC_DEPENDENCY.md](SPC_DEPENDENCY.md) for its
   preserved state contracts, installation and limits. Dependency adoption reduces
   maintained project source; it does not reduce total source including dependencies.
   Grapple swing placement, rope-length changes, clearance
   and swept block contact now belong to `grapple_swing`, with bank 9B callers
   migrated. Shared projection, length traversal and bidirectional sweep retain
   cardinal-direction amplitude, overlapping BTS reads, damage, bounce/stop
   behavior and arithmetic/state-update order. Existing extension helpers stay
   shared with other collision users; their unused context parameters are checked.
   The complete replacement removes **154 lines (98,994 -> 98,840)**, 1,408
   tokens and 4,513 spelling bytes, including helpers, callers and registration.
   All 2,950 actual-code comparisons and retained-source/caller checks pass.
   This includes the earlier isolated angular sweep in a complete responsibility;
   that historical draft supplies no additional savings. Movie execution remains
   unestablished. Scrolling-sky setup, parallax, HDMA bands and tile uploads already
   belong to `scrolling_sky`, with 1,702 actual-code comparisons.
   Water, lava and acid setup, rising motion, tide and background HDMA tables
   already belong to `liquid_effects`, with 6,598 actual-code comparisons.
   Room, X-ray and Power Bomb layer/window composition already belongs to
   `layer_blending`; its probed/clean Ceres fade-in segment passes at **3/0**,
   inputs 8324..8326. Bank 88 retains other duties.
   X-ray background capture, hidden-block overlay, room-item revelation, upload
   and restoration already belong to `xray_background`. Its 1,201 component
   comparisons cover live descriptors, overlapping transfers and failure state;
   X-ray hardware/movie execution remains unestablished.
   HUD minimap rendering, exploration marking and boss-room suppression already
   belong to `minimap`; its measured Ceres segment passes probed and clean at 35/0.
   Mother Brain palette transitions, lighting, wall sealing, earthquakes and
   flicker already belong to `mother_brain_visuals`, with bank AD retired.
   Player projectile data, impact setup, instruction playback and drawing
   already belong to `projectile_animation`, with bank 93 retired.
   Samus animation/drawing already belong to `samus_animation`, message boxes
   to `message_boxes`, and room backgrounds to `room_backgrounds::BackgroundLoad`.
   Banks 85, 92, 93 and AD are fully retired. The unused-declaration cohort reserved
   for this stage is also adopted. Bang's live SNES execution path remains a
   future ownership task.
   Gameplay currently totals 80,859 lines; its 28 numeric bank modules still
   own 73,444. These are ownership scopes, not deletion budgets. File moves
   alone earn no volume reduction. The complete save-file menu ownership draft
   prices **99,516 -> 99,510 (-6)**, including caller migration, imports, helpers
   and registration. It shares confirmation flow and main-menu rows, removes
   forwarding wrappers and preserves separate SRAM/map primitives. That small
   return does not justify adoption as a volume priority; park the draft before
   compiler/runtime validation. No behavior or movie-coverage claim is made.
   Current-source draft: Local/sm-decomp-experiments/20261021/file-select.
   A separate complete sprite-effect pool rewrite saves only 11 lines, 117
   tokens and 301 spelling bytes; it is also parked before compiler/runtime
   work. Draft: Local/sm-decomp-experiments/20261021/sprite-effects.
   Complete Shaktool formation ownership also saves only **11 lines** after
   caller/registration costs. Park it uncompiled and without runtime claims.
   Its scope excludes the misleadingly named Chozo-statue callbacks.
   Draft: Local/sm-decomp-experiments/20261021/shaktool.
   Pause/file-select map icons, saved/debug stations, elevators, bosses,
   exploration masking and the Samus indicator price **98,840 -> 98,800 (-40)**
   after helpers, callers and registration. Park before compiler/runtime work;
   the hidden living-boss double bit-shift still needs behavioral validation.
   Draft: Local/sm-decomp-experiments/20261021/map-icons.
   Phantoon figure-eight acceleration/traversal and aimed swooping motion price
   **98,840 -> 98,793 (-47)** after complete ownership/caller costs. Park before
   compiler/runtime work; the draft's unconditional steering store still needs
   review against the original conditional store. Surrounding fight transitions
   already share countdown handling; inspection establishes no larger replacement.
   Draft: Local/sm-decomp-experiments/20261021/phantoon-motion.
   These are independent formatted prices, with no accepted savings or behavior
   claims. Neither draft ran a movie; no relevant segment is established.
   Complete Space Pirate ownership prices **98,840 -> 98,732 (-108)**, including
   B2 retirement and real callback migration. Park under the renewed larger-scope
   direction. Its actual old/new components passed 2,961 RAM/return/failure checks
   and 1,636 ordered events, using real sine/RNG/sprite code and controlled movement,
   sound and projectile boundaries. Complete source correspondence and release
   validation remain unfinished; no movie ran and no main savings are accepted.
   Draft: Local/sm-decomp-experiments/20261021/space-pirates.
   Reassess the
   90,000-line stretch checkpoint using complete replacements, including retained
   exceptions and oracle adapters; it has no demonstrated deletion budget yet.
   Local naming, unsafe removal and small duplication cleanup support that work.
   Boot-to-credits coverage, remaining native Bang AI ownership, ROM bug-patch
   review and Map Rando remain later project milestones. Correctness defects
   and coverage needed by an active replacement take priority when encountered.

## Previously measured work

Use these results to avoid repeating discovery. Draft figures are historical,
may overlap landed work and are not a current reduction budget.

- Whole-function templates: the latest reconciliation retained 29 groups /
  71 functions for only 185 net lines; generic representation remains parked.
- Named repeated-algorithm extraction: 713 lines already landed. Earlier
  548/808-line estimates and the rejected 1,227-line draft overlap it.
- Hopping cycles: the historical 290-line projection overlaps the covered
  Sidehopper/Beetom sharing landed in 7a9e0122. The remaining Blob animation,
  Beetom descent and release changes are unlanded; the complete current
  lifecycle draft above saves 44 lines, not 290.
- Samus jump/movement subset: 22 entries, 152 net lines; parked. It did not
  redesign the joint movement/pose/transition pipeline in work-order item 1.
- Script execution: 22 selected loop spans contain only 578 gross lines and
  have differing cursor, suspension and live-slot contracts. A universal
  fetch/execute engine has no demonstrated large payoff.
- Promoted wrapping arithmetic: 620 sites grew 576 lines. Fractional-position
  kernels saved 29. Current-enemy accessors, terminal/indexed-loop rewrites,
  cinematic initializer rows and remaining generic arithmetic are parked.
- Residual micro-families (queues, palette/homing kernels, HUD/menu rows,
  bitwise word updates, typed ROM records) price in tens to low hundreds of
  lines. They do not independently justify a new volume-reduction workstream.
- A chained VRAM-write cursor stores each argument immediately, preserving
  later expressions' access to earlier writes. Across 43 retained-pointer
  entries / 42 functions it grows **63 formatted lines**, including its helper.
  Reject this representation; wrapped/mixed writers and the consumer remain
  outside its measured scope. No compilation or runtime claim. Draft/report:
  TEMP sm-bindings-root-9c4daf2f/vram-cursors/REPORT.md.
- Shared layouts, RAM/word/byte access, callback declarations, dispatch,
  exact aliases, graphics/word transfers, corpse tile traversal and audited
  unused-code removal are already harvested. Count only further net deletion.
- The unused-declaration cohort reserved for item 4 is adopted there on current
  source: 353 unused import bindings and five unreferenced Bang helpers remove
  191 lines. The former 336-line draft used a different input and supplies no
  additional savings. Current reconstruction and checks:
  Local/sm-decomp-experiments/20261021/unused-declarations-current.
- The historical isolated grapple angular sweep is now subsumed by item 4's
  complete current-source swing/length/contact replacement. Its earlier draft
  and 6,574 comparisons/oracle cases are historical, not additional savings or
  current-source validation. No grapple movie execution is claimed.
- Renderer unification remains parked: assessed alternatives did not preserve
  the existing contracts. PNG inflater substitution is rejected by concrete
  EOF/reserved-code counterexamples; the native decoder replacement above
  retains that inflater and removes only the surrounding repeated algorithms.
- Single-use local inlining: 318 order-safe scalar candidates across 38 files
  grow by 86 formatted lines with ordinary inference, or 181 with explicit
  type constraints. Reject both; no compilation or behavior claim. Branch
  prefix/middle sharing across 72 files exposes only 72 gross repeated lines;
  residual audio families have no credible 150-line net budget. These bounded
  studies do not justify repeating the same scans. TEMP reports/drafts:
  sm-single-use-8d654c29, sm-branch-prefixes-8d654c29,
  sm-audio-repetition-8d654c29 and sm-dead-initializers-8d654c29.

Further syntax preflight at b64a0470 found no substantial unpriced cohort:
844 non-shader direct wrapping assignments occupy 963 lines; 329 adjacent
computed-result/store pairs occupy 844 gross lines before replacement costs.
Pointer, timer, null-guard and string-cast envelopes provide no credible
thousand-line replacement. These bounds do not exhaust algorithmic sharing.

A distinct compiler-lint check of current headless-library control flow finds
one type-limit comparison, two shadowed condition chains, one constant
self-comparison and 45 mostly local Boolean diagnostics, some overlapping.
It establishes no substantial deletion group. The final forced-lint metadata
pass succeeds; no candidate, saving or runtime claim. This is limited compiler
discovery, not exhaustive path analysis. Keep that cohort parked. Results:
Local/sm-decomp-experiments/20261021/semantic-branches.

Further complete-cohort pricing at 7617b682 excludes these repeats:

- Index-only Eproj/PLM/HDMA slot methods: 958 replacements, 253 locals and
  86 methods across nine callers add **668 formatted lines**. This differs
  from accessor-declaration drafts but also fails the volume test.
- Field-aware word arithmetic: 135 expressions across 16 banks remove only
  **39 net lines**, including helper/documentation costs. Park this notation.
- PPU kernel sharing is already landed. The surveyed 1,417 gross function
  lines include existing shared kernels and required renderer differences;
  they are not removable duplication.
- Remaining callback signatures expose 313 gross header lines and at most
  **255 net lines** after optimistic minimum grouping costs, before real
  formatting and ordering costs. This cohort has no large volume payoff.
- Multibox collision bodies occupy 219 lines and normal counterparts 117;
  traversal, count widths, edge rules, gating and effect order differ.
  Residual comparable PLM/scrolling tile loops total roughly 60 lines after
  existing tile sharing. Neither preflight establishes a large replacement.

These studies changed no maintained source and ran no runtime checks.
Drafts/reports: TEMP sm-slot-index-price-7617b682,
sm-word-arithmetic-7617b682, sm-ppu-kernels-7617b682 and
sm-signature-headers-7617b682. Continue discovery beyond these bounded
cohorts; they do not exhaust possible algorithmic reductions.

Countdown pricing selected the landed callback form over runtime stage enums
(-11 boss/presentation, +34 ordinary-enemy lines) and operation-level gates
(-173 before any additional caller scope). The bounded census found 374
countdown-containing functions / 9,364 gross lines, not removable duplication.
Snapshot capture/restore mapping removes only 84 lines including its macros;
park this uncompiled draft, with no runtime claim. Reports: TEMP
sm-timed-transitions-63b0ffe4, sm-enemy-programs-63b0ffe4 and
sm-snapshot-map-price-f698eb50.

The complete Torizo ROM-tuple projectile initialization draft covers four
variants and removes only 42 lines at c98e8439, including the shared facing/origin
selection. Park it without builds or runtime claims. Whole-initializer and
macro-aware audits found no credible 500-line group. Draft/report: TEMP
sm-eproj-init-study-c98e8439. A separate word-sign predicate census finds 1,512
predicates on 1,655 occupied lines: only 143 continuation lines before any
replacement costs. This does not justify another global arithmetic rewrite.

Further complete-cohort representation prices close three new syntax studies:

- Low-word arithmetic: 640 selected expressions grow **153 lines** with
  ordinary generic helpers and **161** with expression macros. Both include
  support/import costs. Reject without builds; conservative type/range
  selection is not an independently verified semantic proof.
- Exhaustive branch assignments: 254 roots / 573 stores grow **134 lines**;
  the pure-store subset grows **82**. Independent recount agrees. Temporary
  lifetime equivalence remains unproven; reject without builds.
- Native-width comparison/bitmask expressions and redundant cast chains:
  939 replacements save only **99 lines**. Park this uncompiled draft.

These prices use ordinary formatting and no new expression DSL. Reports:
TEMP sm-low-word-expressions-ddba5db2, sm-control-scaffold-price-20261020 and
sm-context-casts-d914d9aa. A dynamic-bound/retained-counter range
follow-up finds roughly 100 gross scaffold lines before replacement costs,
not another multi-hundred-line candidate. TEMP sm-indexed-ranges-dynamic-preflight.

The earlier cross-file callback draft's **346-line** formatting-only price is
superseded by the current 350-line adoption under item 3. Its temporary resource
hold was lifted by the user's later parallel-experiment direction. The exact
913-arm scope was resumed without another clone scan. Historical draft:
TEMP sm-unified-callbacks-crossfile-v4; validated current work:
TEMP sm-enemy-callback-current. Do not add both prices.
A macro-aware graph of 6,729 functions/callbacks and 1,274 globals found no
large disconnected component; new candidates occupy only 48 source lines.
No deletion is claimed. TEMP sm-dead-components-d914d9aa.

The existing typed-literal emission experiment saves 35,000 lines on the
original transpilation, but its adaptation at 65c7fc36 saves only **19 lines**
across 1,375 sites, including parsed macro invocation bodies. The complete
formatted snapshot is 107,520 -> 107,501; it has no compiler or behavior proof.
Park it without further validation. TEMP sm-literal-adoption-65c7fc36.

Further complete current-source prices at 106,189 remain small:

- Callback address constants combined with existing dispatch ownership save
  62 lines; grouping repeated constant declarations alone saves 94; the actual
  combination saves 110. Ownership after grouping increases tokens and text.
  Complete builds and source checks pass; park without runtime work.
- Residual typed/byte RAM pointer chains save 106 lines across 173 replacements,
  including macro bodies/templates, with no new support. Matcher guards and a
  source audit pass; park before broad compilation or replay.
- Ordinary internal macro imports remove 5,151 parser tokens and 16,032 spelling
  bytes but only 47 lines, including imports. Park this uncompiled price.

These are separate, overlapping controls, not an adoption budget. Reports:
TEMP sm-tailored-next/address-ownership, ram-pointer-chains and internal-imports.

When revisiting one of these exact families, consult its historical section
with `git show ec45c573:docs/PLAN.md` and search the relevant heading/symbol.
Temporary draft locations recorded there may no longer exist. Recheck current
source before using any old draft. Earlier "next" instructions are historical.

## Validation strategy (2026-10-21)

User direction: test movie segments that actually execute the changed code
and its relevant downstream effects. Unrelated frames add no coverage. This
replaces the former full-replay cadence, broader-regression defaults and
historical per-commit recipes; subsystem completion alone requires no full run.

1. **Select before running.** Identify changed routines, relevant branches,
   callers, shared data and downstream effects. Choose the smallest scenario
   that exercises them. Documentation/comment/format-only edits need applicable
   static checks; removing demonstrably unused private code needs a reference
   search and compiler checks. Neither needs a gameplay replay. Frontend-only
   changes need frontend checks, since the headless movie does not call the UI.
2. **Focused behavior changes.** Run formatting, a release build and the shortest
   exercised oracle segment on the final source. Require explicit routine and
   changed-branch probes with `--require-hit`; a missing hit fails verification.
   Use `--frames` for a known endpoint, or `--after-hits N` to stop after every
   required probe first executes plus N follow-up frames. Choose N to observe
   the actual effects, including subsequent callbacks, frames, transitions or
   persistent writes. Require a probe at the later effect too when available.
   Entry hits alone do not prove branch coverage or correct behavior. Keep all
   normal oracle comparisons, rendering and audio advancement enabled.
3. **Skip only a justified prefix.** A compatible local checkpoint must precede
   affected setup and calls. Inspect earlier callers and shared-state effects
   before reusing it after an edit: runtime compatibility alone does not prove
   the changed build reaches that state. If a checkpoint must first be built,
   identify that prefix as setup, cache it before affected setup/calls, and
   keep setup frames separate from changed-code coverage. An uncovered routine
   needs another route or a focused component/oracle test of its real implementation;
   repeatedly running an unrelated green movie adds no coverage. Constructed
   tests must state their limits and do not establish movie execution. Choose
   arithmetic/aliasing cases for plausible failure modes rather than defaulting
   to large unrelated sweeps. If relevant context remains uncovered, report it
   and hold further cleanup of that behavior.
4. **A full movie requires demonstrated coverage.** Before starting one, show
   why the changed code or its measured downstream effects require that whole
   route rather than bounded exercised segments. Name the required probes and
   expected frame ranges first. Code size, broad subsystem labels, elapsed time,
   commit count and batch completion never justify a full replay by themselves.
   If coverage is missing, select another exercised segment or a component/oracle
   case; do not lengthen an unrelated run. Stop a redundant run once its lack of
   relevance is established. Reuse source-checked builds and compatible checkpoints.

Record selected scenarios, required hits, executed frame range and result in
the commit message, with the reason for a full replay or static-only check.
Use source-checked builds and `sh tools/progress.sh` as usual. No new test
registry, evidence store or coverage dashboard is needed. Commands and
checkpoint constraints are in [movies/README.md](../movies/README.md).

## The oracle

Current backend verification and its PPU-read coverage limit are recorded in
the active composition above. Previous grapple swing passes **2,950** actual old/new
full-RAM, completion and failure-flag comparisons. All **37,203** ordered
projection calls match angle, length and incoming grapple-state bytes; ten
invalid-angle/BTS cases preserve the original panic and preceding RAM state.
Cases cover every sine-table angle, odd offsets and amplitude truncation,
all block classes, overlapping BTS reads, spike damage/invincibility, chained
extension blocks, radial clearance, length limits/wrap, both swing directions,
scaled/truncated motion, impulse/timer boundaries and successive updates.
Scenarios explicitly exercise free movement, bounce and minimum-length stops.
Block reactions, extension traversal and multiplication are real implementations,
not modeled boundaries. These are constructed component checks, not SNES/movie
execution. No grapple segment is established; no movie ran.
Complete retained-source/caller/import/registration, unused-context-parameter
and extracted component/support checks pass, as do normal release library/
frontend/verifier builds, formatting and embedded-source checks. Source:
`157aae78a39fc63b23fe207d0de2680ae9f398093e190c286e0bf8b335486fe4`.
Local checks: Local/sm-decomp-experiments/20261021/grapple-swing.

Previous scrolling sky passes **1,702** actual old/new
full-RAM, completion and failure-flag comparisons, with **30** matching ordered
spawn boundaries and incoming RAM. Cases cover actual ROM vertical/parallax
band edges, constructed counts and table exhaustion, wave phases/timers,
odd/high/aliased slots, speed carry and aliased camera/output, frozen paths,
upload-tail wrap, overlapping source/queue writes, top-edge damage and
successive updates. Spawn callbacks record arguments/RAM and model mutations;
real spawning and HDMA/VRAM hardware consumption are not established.
Complete retained-source/caller/import/dispatch/registration and extracted
component/support checks pass, as do normal release library/frontend/verifier
builds, formatting and embedded-source checks. No measured sky callback frame
range is established; no movie ran. Source fingerprint:
`5e7c6085b403ed3efbe784ce3c3660bfdbf5f1f273f3c1fa9894444106c0e941`.
Local checks: Local/sm-decomp-experiments/20261021/scrolling-sky.

Previous liquid effects pass **6,598** actual old/new
full-RAM, completion and failure-flag comparisons, with **1,511** matching ordered
boundary events. Cases cover velocity/subpixel carry, target equality and
overshoot, signed/clamped surface heights, every tide phase and amplitude,
motion states/timers, frozen updates, all background wave modes, odd/high slots
and aliases into liquid state and wave output. Register, sound, spawning,
earthquake and unknown-state assertion calls record incoming RAM; modeled
callback mutations check later reads. They do not establish hardware/audio or
real spawning execution. Source correspondence covers all retained code,
callers/imports, callback addresses, the shared sky/water wave table and exact
component bodies/support. Normal release library/frontend/verifier builds,
formatting and embedded-source checks pass. No measured liquid movie segment
is established by existing room-only route logs; no movie ran. Local checks:
Local/sm-decomp-experiments/20261021/liquid-effects.

Previous layer blending passes 5,504 components and the probed/clean Ceres
fade-in segment at **3/0**, inputs 8324..8326, with six composition and actual
PPU window/screen-consumption probes plus two follow-up frames. Discovery
establishes the first RAM-changing call at 8324; earlier calls are no-ops.
Setup is cached separately. Unvisited effect modes have component coverage;
color-math IRQ consumption is unestablished. Local checks:
Local/sm-decomp-experiments/20261021/layer-blending.

Previous X-ray backgrounds pass 1,201 actual-code components and retained-source/
caller checks. Movement and sound callbacks are recording boundaries, and no
X-ray movie execution is established. Local checks:
Local/sm-decomp-experiments/20261021/xray-background.

Previous minimap validation passes 1,360 components and the probed/clean Ceres
segment at **35/0**, inputs 8324..8358, with seven drawing/exploration/flashing/
upload probes and 32 follow-up frames. Setup is cached immediately before the
first changed call at 8324. Both initial 3,000-frame searches failed coverage
with zero minimap hits. Local checks: Local/sm-decomp-experiments/20261021/minimap.

Previous static-only change: unused declarations are removed after compiler
reference checks. All 82 Rust files retain the same executable source and numeric
callback rows outside five unreferenced Bang helpers; retained import bindings
keep their targets and visibility. Release library/frontend/verifier builds,
all-target and headless compiler checks, and embedded-source byte checks pass.
No new movie run or gameplay coverage is claimed.

Previous applicable verification: shared message-box lifecycle passes the gunship
acceptance scenario, with eleven required open/close, confirmation, delay,
save-cycle, restoration and resumed-gameplay probes, then 120 follow-up frames.
Probed and clean checks both pass **22962/0**, including the boot setup prefix;
the latest required effects execute at input 22841. Normal rendering/audio and
comparisons stay enabled. Component comparisons cover all continuation bytes,
all radii, input/timer boundaries and 224 complete lifecycles; unchanged rendering
and callback code has source correspondence. No credits coverage is claimed.
Clean source fingerprint:
`0dabe7ebd3a831aead1861bbd4a3e9cd48f7f31694c57fa599fcf842af3e9b81`.
Logs: Local/sm-decomp-experiments/20261021/message-boxes/
{probed-accept,clean-accept,components}.log. The redundant full Speed Booster
run was stopped following the clarified validation rule; it is not a pass.

Previous full integration: the complete CPU, PNG, RAM-chain and readable-control-flow
composition passes full boot any% **129712/0** and Speed Booster **148406/0**,
with normal rendering/audio and comparisons. Both use final source fingerprint
`a49e35e84ffd2c8e964c2b9df5414f9136ce9e1fb6ad6be255d1c541161b9e4b`.
All five final routine bodies match the independently tested component source.
Both installed native archives retain their tested hashes. The frontend codec
has separate component coverage; the movie does not exercise PNG loading.
Neither route reaches credits. Logs: Local/sm-decomp-experiments/20261021/
residual-control-flow/composition-e2530722/readable-v2/main-{any,speed}.log.

Previous full integration: the pinned LakeSnes CPU adapter passes full boot any%
**129712/0** and Speed Booster **148406/0**, with normal rendering/audio and
comparisons. Both use final source fingerprint
`3a08843e9e15b17aa59eeb5f1018dca693c40177a1f85e2fc1f2e8b13eb6efde`.
The adapter matches the independently compared component source; the installed
archive matches its pinned build hash. Neither reaches credits. Logs:
Local/sm-decomp-experiments/20261021/lakesnes-cpu-dependency/main-{any,speed}.log.

Previous full integration: live-typed snapshots pass clean full boot any%
**129712/0** and Speed Booster **148406/0**, with normal rendering/audio and
comparisons. Both use final source fingerprint
`d7c1ddfdc819e388baad82ca4f277e807d976a845a3e7df179ac19713d0a1a6a`.
The adopted cpu_infra exactly matches the source-bound component candidate,
composed with the prior SPC/PPU changes. Neither reaches credits. Logs:
Local/sm-decomp-experiments/20261021/live-snapshots.

Previous full integration: shared native/SPC state transfers pass clean full boot
any% **129712/0** and Speed Booster **148406/0**, with normal rendering/audio
and comparisons. Both use final source fingerprint
`a7d82ec0f552ff40614d7cba0bb195ef057a33c307c23b46da4899c879a4798f`.
The adopted source exactly matches the component-checked transfer candidate,
composed with the previous PPU replacement. Neither reaches credits. Logs:
Local/sm-decomp-experiments/20261021/native-sequencer.

Previous full integration: shared PPU pixel evaluation passes clean full boot
any% **129712/0** and Speed Booster **148406/0**, with normal rendering, audio
and comparisons. Both use final source fingerprint
`2df5b594d509743f3be3aeaba787add99673680dbce2aac0543d64373225f2e0`.
The adopted PPU matches the independently compared candidate byte-for-byte.
Replay times were 380.70s and 630.50s. Neither reaches credits. Old/new DLL
pixel/state checks separately cover the renderer because both replay branches
share the backend. Logs: Local/sm-decomp-experiments/20261021/ppu-render-native.

Previous full integration: explicit compound RAM updates pass clean full boot
any% **129712/0** and Speed Booster **148406/0**, with normal rendering, audio
and comparisons. All 801 source updates, including 106 rereads, retain their
expanded operations and resolved types/targets. Both runs use the same final
source hash, `2dc2a2bdf5feed4e522b0dc7f48c003941fdff0b3ee69b99be9d1d44b9bdcfbb`.
The candidate matches the maintained source, with the unrelated shader draft
preserved. No new required gameplay branch was introduced; complete static
correspondence covers unchanged operations. Neither route visits every updated
site or reaches credits. TEMP sm-compound-adoption-20261021 retains logs.

Previous full integration: shared enemy callback ownership passes clean full boot
any% **129712/0** and Speed Booster **148406/0**, with normal rendering, audio
and comparisons. Probed Speed Booster **21015/0** exercises all six required
`enemy_dispatch.{ai,pre,instr}.{entry,owner}` labels and 120 follow-up frames.
Every family delegates at runtime; the grouped probes do not establish every
owner's movie execution. Dispatch component tests cover all explicit routes,
adjacent/range/high-byte-invalid addresses and all index values; unchanged
gameplay bodies have complete expanded correspondence. The adopted 82-file
source matches the replayed source after CRLF/LF normalization, with the shader
draft preserved. Replay source hash:
`0b13d575b48aba371357ceb436af7b357440f9cbdc097ade972778b5281a02a9`.
Neither movie reaches credits. TEMP sm-enemy-callback-current retains logs.

Previous full integration: packed-sprite word access and Samus pose sharing pass
full boot any% **129712/0** and Speed Booster **148406/0**, with normal rendering,
audio and comparisons. Required `sprites.wrap`, `sprites.inside` and
`sprites.bounded` probes execute on both routes; other clipping variants have
component coverage. The repair passes 265,737 actual current-Game comparisons
on disjoint ROM sources, with the original debug alignment abort preserved.
The adopted repair matched its 81-file replay snapshot after CRLF/LF
normalization, excluding the unrelated shader draft. The subsequent import-only
prelude has complete expanded-operation and resolved-binding correspondence;
it adds no new replay claim. Repair replay source hash:
`fe8feacc6e4d8cf8258c7668badd8745c3ad00098511d0128cac4a16f1e6b1ba`.
Neither movie reaches credits. Synthetic RAM-source aliasing remains unsupported.

Latest focused change: shared Samus pose refresh/history passes probed and
clean boot any% intro **2339/0**. All six required helper/effect probes execute,
including 121 tile consumers after history writes, followed by 120 frames.
No gameplay room or normal movement execution is claimed. Complete expanded
correspondence covers all remaining 7,080 functions and all migrated callers;
the final dead-import cleanup has token correspondence and fresh builds.

Earlier full integration: nonnull callback tables and native tracing pass full
boot any% **129712/0** and Speed Booster **148406/0**, with normal rendering,
audio and comparisons. Complete static correspondence proves all 922 callback
targets and 82 consumers; 742,400 diagnostic component cases cover tracing.
Final optimized library/frontend builds pass. Source hash:
`83c7a77846eaea50a541bb4b02b1ebaa5ac19f2cae3f47fd57c3054f3fa0e9fc`.
Neither movie reaches credits or exercises every changed callback/diagnostic.

Earlier full integration: the shared beam-window owner passes full boot any%
**129712/0** and Speed Booster **148406/0**, with normal rendering, audio and
comparisons. Charge Beam **28699/0** exercises seven required probes through
the shared renderer and eye-beam cleanup plus 180 follow-up frames; the clean
run passes at the same endpoint. Actual old/new Rust components pass 64,100
full-RAM cases including all directions, corrected wraps, callers, saturation,
headers and null-dispatch effects. Unvisited Mother Brain behavior has component
coverage; no endgame movie or credits is claimed. The final source preserves
one existing macro comment; executable tokens match the full replay build.
Final-source library/frontend compilation and static checks pass.

Earlier full integration: terminal loops, selected field/RAM store groups and
the 83-capture unaligned RAM repair through dc5d2f08 pass full boot any%
**129712/0** and Speed Booster **148406/0**, with normal rendering, audio and
comparisons. The final source matches the replayed snapshot. Complete expanded
library/frontend correspondence covers unchanged operations; loop lifetime
checks and C/music-queue/ordered-effect components cover the selected semantics.
These do not establish movie execution of every changed path or credits.
Earlier callback/dispatch sharing over ecf93788 passes any%
**129712/0** and Speed Booster **148406/0**, with normal rendering, audio and
comparisons. Independent complete function/route correspondence covers the
unvisited callbacks; no claim of movie execution for every callback is made.
Earlier indexed loops over 58db72cb (same source as d914d9aa)
pass any% **129712/0** and Speed Booster **148406/0**, with normal rendering,
audio and comparisons. Independent checks cover all 132 control sequences
and unchanged bodies; these are not claims of movie execution for every loop.
Earlier blast windows over ddba5db2 pass any% **129712/0** and
Speed Booster **148406/0**, with normal rendering, audio and comparisons.
The focused Ceres run passes **42819/0**, exercising all 11 required Power Bomb
phase/transition/cleanup probes plus 180 follow-up frames. All 15 component
probes execute across 4,700 SNES comparisons; Crystal Flash coverage remains
constructed, not movie execution. No endgame execution is claimed.
Earlier opcode grids over c98e8439 pass any% **129712/0** and
Speed Booster **148406/0**, with normal rendering, audio and comparisons.
Independent complete selector correspondence and compiled boundary traces
cover all opcode values; these are not claims of every opcode's movie execution.
Earlier projectile flight over c1ebc4ca passes
any% **129712/0** and Speed Booster **148406/0**, with normal rendering, audio
and comparisons. Its 51,791 focused SNES comparisons exercise all 20 migrated
numeric entries, subsequent flight and launch inheritance. Probed any%
**11113/0** and Speed Booster **10934/0** cover the selected demo/Ceres branches
plus 120 follow-up frames. Exotic beam/super-missile paths have constructed
coverage; no endgame execution is claimed. Earlier byte-product checks have
independent actual-function coverage, without proven changed HDMA execution.
Earlier platform probes cover timed initialization, triggering,
closing and stage advancement. Required opening/carry and non-timed movement/
activation probes have zero hits: those coverage checks fail despite zero
differing frames. Those paths have constructed old/new Rust comparisons,
not movie/SNES coverage; hold further cleanup until that context is covered.
Beam firing's 20,420 focused SNES comparisons cover 5,780 input/
firing cases and 14,640 subsequent projectile calls. The earlier intro-page
checks include Japanese **7570/0** and 233 marker/inactive-input oracle cases.

The any% input visits six rooms; Speed Booster visits 55 rooms and 53 enemy
definitions. Neither reaches credits. Kraid/Varia are reached by the extended
route; later bosses/endgame and all branches still need appropriate coverage.
The local Charge Beam route includes Morph Ball eye-beam inputs 28200..28550,
as used in movies/README.md. A green movie does not cover unexecuted behavior.
Existing RAM masks, patched-ROM behavior, hardware/audio limits and persistence
coverage still matter; inspect current cpu_infra.rs comparisons and PatchBugs
when a change touches them. Preserve checks and disclose their actual limits.

The Windows launcher checks source bytes embedded in the compiled library;
register new library source files in sm/src/verify_source.rs. Keep source and
inputs fixed through checks. It rejects stale artifacts and source changes.
Checkpoints must precede affected history; required probes and --after-hits
select exercised segments without disabling comparisons. Commands and caveats
are in [movies/README.md](../movies/README.md).

## Definition of done for any change

1. Applicable checks from the validation strategy pass on the final source.
   Documentation-only changes need static checks; code changes need formatting
   and appropriate compiler checks, with release builds for behavior changes.
2. Changed behavior executes in the selected checks, with no new failing frames.
   Static equivalence must establish unchanged operations; an unexercised green
   replay or source inspection alone does not validate a behavior rewrite.
3. Run `sh tools/progress.sh`; explain counter increases. Report complete
   replacement savings and remaining budget only when actually measured.
4. Recheck HEAD and other work before committing to main. Commit one coherent
   idea with the verify result. Update this plan's active item when status or
   direction changes; routine check details belong in the commit message.

## Workspace discipline

Keep this file an active plan, not a per-commit journal. Replace current status
in place. Historical results and stopped experiments remain in Git; consult
only the relevant section when needed. Do not add evidence stores, dashboards,
coverage registries or new process tooling.

Performance work targets the verification pipeline; readability takes priority
in maintained gameplay and hardware. Reuse the normal build and select checks
by changed behavior. Measure edit/build/result time before optimizing another
verifier operation; existing release builds are around 20 seconds and the
recent full replay pair about 4.4 minutes, dependent on source and load.

Coordinate heavy work with existing owners. Keep game/host artifacts local to
the checkout; seed registry dependencies only, not whole target directories.
Use CARGO_INCREMENTAL=0 for disposable snapshots. Remove disposable artifacts
only after checking they are no longer needed and resolving paths safely.
Preserve unrelated source changes and active build caches. Never commit ROMs,
movies, save states or other ROM-derived files.

After the resource exhaustion, run one heavy compiler job at a time. Reuse
identical baseline builds and dependencies, stream compiler output directly to
files, and avoid full HIR/THIR dumps when source correspondence plus targeted
compiler witnesses establishes the required property. Finish selected current
cohorts before starting more experiments. Keep disposable build output bounded.

## Progress

Baseline at ec45c573: raw g_ram offsets 30; extern C blocks 3; counted C ABI
exports 4; no_mangle 8; unsafe functions 5307; i32/c_int cast lines 8848;
Rust lines 112265. The unrelated 83-line shader draft is excluded from the
committed baseline and budget.

After the Japanese correction, local bindings, intro pages, enemy callback
imports, ordered stores, PNG texture loading, beam firing, moving platforms,
byte-partial-product sharing, INI decoding, countdown callbacks, projectile
flight, callback entry captures, opcode grids, blast windows, indexed loops and
combined callback/dispatch declarations, terminal loops, selected field/RAM
store groups, the unaligned RAM repair, shared beam windows, native tracing,
nonnull callback tables, Samus pose sequences, the packed-sprite repair,
shared gameplay imports, enemy callback ownership, native inflater, compound
RAM updates, shared RAM/record metadata, alias declarations, native C-string
literals, shared constant headers, native shader/presentation ownership and
shared PPU pixel evaluation, native/SPC state transfers and live-typed snapshots,
the pinned CPU/PNG dependencies, RAM pointer chains, readable control flow and
shared room background loading, Samus animation ownership, the message-box
lifecycle, unused declarations, projectile animation, Mother Brain visual
effects, minimap ownership, X-ray backgrounds, layer blending, liquid effects,
scrolling-sky and grapple-swing ownership, the SPC/PPU/DSP/music replacements,
shared configuration/input and utility dependencies, the desktop host,
room-scroll data, the shader dependency, safe pose/mechanism RAM access,
gameplay continuation ownership, owned ROM reads, safe asset decompression,
checked ROM tables, safe word transfers, owned recording, checked VRAM readback
and owned sprite drawing, maintained Rust is **94,312**, with
applicable static, component and exercised movie checks passing.
Including 262 C and 296 header lines gives **94,870**; all 803 dependency-installer
lines and 28 added launcher lines bring the complete price to **95,701**.
The immediate volume target is complete.
External dependency counts and current coverage are listed at the top.
Counters: raw offsets 23; extern C blocks 7; counted C ABI exports 6; no_mangle 65;
unsafe functions 3040; cast lines 6601. Owned sprite drawing adds seven maintained
lines and removes two counted unsafe declarations and 13 cast-containing lines.
Its checked ROM/OAM writer forbids unsafe code; the native warning remains an
explicit unsafe boundary. Other counters are unchanged. This is a safety cost,
not a reduction. VRAM readback adds one line, removes four
counted unsafe declarations and seven cast-containing lines; other counters are
unchanged. Its hardware calls remain unsafe. Owned recording adds nine complete maintained
lines (22 Rust, minus 12 C/header and one installer line). It removes two counted
raw offsets, six unsafe declarations and five cast-containing lines; no other
counters increase. The safe recorder owns both buffers and the input/patch codec;
file and native snapshot interfaces remain unsafe. This is a safety cost, not a
reduction. Safe word transfers add 25 lines and remove
three counted unsafe declarations; other counters are unchanged. This is a safety
cost, not a reduction or elimination of the remaining unsafe caller expressions.
Checked ROM tables add one line and remove
two counted unsafe declarations; all other counters are unchanged. Safe asset
decompression removes nine
counted unsafe declarations and eight cast-containing lines, with no counter
increases. Its VRAM hardware interface still contains explicit unsafe operations.
Owned ROM access adds 28 lines and removes
one unsafe registration declaration plus the Game/cart raw ROM dereferences.
Gameplay continuation ownership adds 55 lines without changing other counters.
These are safety/ownership costs, not reductions. Safe pose history adds one line and
removes one unsafe function. Mechanism RAM migration adds 203 lines and removes
five unsafe-function declarations; one existing two-cast expression now spans
two source lines. These increases pay for checked RAM access and explicit unsafe
boundaries, not additional game behavior. Five migrated helpers forbid unsafe
code. The shader adds 19 no_mangle attributes
for its native callbacks and captured GL table, removes 16 counted unsafe
functions and 32 cast-containing lines. The script excludes C-unwind callbacks;
these residue counts do not measure all unsafe operations or establish their
elimination. Room-scroll extraction previously removed one counted function;
the shader's actual baseline was 3,089. Desktop ownership adds 32 no_mangle
attributes for the explicit native-host callbacks and removes 26 counted unsafe
functions and 120 cast-containing lines. Its internal C-unwind declarations are
not included in the script's narrower C-ABI counters. Configuration/utilities remove 28 counted
unsafe declarations and 92 cast-containing lines. Four added no_mangle attributes
belong to the three host/numeric/stderr callbacks and the verifier source check;
the script does not count the two C-unwind declaration blocks. Native music
previously added one counted export/no_mangle for its source check.
Legacy PPU pixel selection removes three
unsafe declarations and 46 cast-containing lines; no counter increases.
The DSP dependency adds one counted
foreign block and removes six unsafe declarations and 126 cast-containing lines.
The PPU dependency adds one counted
foreign block; its compiled-source verification export adds one counted C ABI
export and no_mangle. SPC also adds a C-unwind boundary and two export_name
callbacks, which this script does not count. Together the replacements remove
46 counted unsafe declarations and 122 cast-containing lines. Grapple-swing ownership removes two
unsafe declarations and 19 cast-containing lines; no counter increases.
Scrolling-sky ownership adds one unsafe
helper for paired channel writes, replacing existing raw stores, and removes
23 cast-containing lines. All other counters stay unchanged.
Liquid effects remove six unsafe
declarations and 37 cast-containing lines; no counter increases.
Layer blending removes 26 unsafe
declarations and two cast-containing lines; no counter increases.
X-ray background ownership removes five
counted raw-RAM offsets, five unsafe declarations and 65 cast-containing lines;
no counter increases. Minimap ownership removes one unsafe
declaration and 29 cast-containing lines; no counter increases.
Mother Brain visual ownership removes
seven unsafe declarations and five cast-containing lines; no counter increases.
Projectile animation removes ten
cast-containing lines and adds three counted unsafe declarations: two shared
helpers and the SBA entry previously declared inside a callback macro. These
replace existing unsafe operations; all other counters stay unchanged.
Unused declarations remove four counted
unsafe declarations and 13 cast-containing lines; no counter increases.
Message-box ownership changes only the
total line count. Samus animation ownership removes six
unsafe declarations and 39 cast-containing lines; no counter increases.
Room background ownership removes
14 unsafe callback/helper declarations and changes no other counter. RAM pointer
chains remove nine cast-containing lines without changing the other counters. The codec removes
five unsafe declarations and three cast-containing lines while adding one foreign
dependency boundary. The CPU adds one foreign dependency boundary and removes
25 unsafe declarations. Readable control flow changes no other counter.
Snapshot diagnostics add one unsafe
helper and remove 14 cast-containing lines while removing 239 total lines.
Shared state transfers add one unsafe
helper and remove six cast-containing lines while removing 230 total lines.
Shared PPU evaluation removes nine unsafe
declarations and 186 cast-containing lines. Shader/presentation ownership removes
five unsafe declarations and 84 cast-containing lines. RAM/record metadata remove 452/271 lines
without changing any other counter. Alias, C-string and constant grouping remove
202, 92 and 94 lines respectively without changing the other counters.
Compound updates remove 370 lines and
71 cast-containing lines without changing the other counters. The inflater removes 12 unsafe-function
declarations and 25 cast-containing lines. Callback grouping removes 314 counted
unsafe declaration lines while preserving expanded signatures. Its unchanged
Torizo two-cast expression wraps onto two lines, raising that counter by one.
The pose owner adds three unsafe
helpers and removes two wrappers, a net increase of one declaration while
removing 84 lines. The sprite alignment repair adds five lines, 38 tokens and
234 token-spelling bytes for defined packed-record reads/writes. Shared imports
remove 381 lines without changing the unsafe/cast counters. Tracing adds three unsafe helper
declarations for the caller-owned output buffer and removes 92 cast-containing
lines. Its complete implementation still removes 144 lines. The former
83-line shader draft is now incorporated in the complete shader replacement,
whose 588-line committed reduction includes those 83 lines once.
Item 2's function/compound-body variant audits are complete for their stated
scope. The under-100,000 milestone is complete; the 90,000-line stretch checkpoint
has no established replacement budget. Hopping and local-initialization drafts
remain parked; repeating completed scans or adding their gross matching spans
does not supply that budget.

Historical details before this reset remain in **ec45c573:docs/PLAN.md**.
