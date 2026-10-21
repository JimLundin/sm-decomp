# Reducing transpiled Rust: tools and process

Research updated: 2026-10-21. Immediate objective: reduce maintained source,
preserving behavior. This note records primary-source research, scanner trials
and isolated measured experiments. Their scopes are explicit; they are not an
exhaustive discovery result or a whole-game deletion forecast.

Artifact availability: the user reported system resource exhaustion, and the
TEMP experiment trees disappeared. Historical results below are prior observed
measurements; their local inputs, emitters and checks are presently unavailable.
They are not a currently inspectable completion proof. Current-source work
continues from Git in Local/sm-decomp-experiments/20261021 with one heavy compiler
job at a time and shared baseline/dependency artifacts.

Executable translation rules learned from current Rust previously produced a
**102,539-line** historical-source composition. The current-source batch has
landed separately at **106,189 lines (-1,331)**, with 6,104 fewer tokens and
8,460 fewer token-spelling bytes, including the eight-line alignment repair.
All emitted support and overlap are counted. Complete builds, the documented
correspondence/component checks and both full boot replay routes pass for the
adopted source. Neither count is below 100,000, and their savings are not added.
The historical pipeline also fixes concrete packing and unaligned-reference
defects; correctness costs are charged separately from code reduction.
The subsequent shared beam-window owner removes another **205 lines**, followed
by **215** from native tracing and nonnull callback tables, then **84** from
shared Samus pose sequences, reaching **105,685**. The packed-sprite alignment
repair adds five lines, reaching **105,690**, with complete builds, component
checks and both full boot replay routes passing as scoped below.
The shared gameplay prelude removes **381 more lines**, reaching **105,309**;
complete expanded-operation and resolved-binding checks pass.
Shared enemy callback ownership then removes **350 lines**, reaching **104,959**,
with complete builds, static/dispatch comparisons and both full boot replays.
Native inflater ownership then removes **165 lines**, reaching **104,794**,
with complete builds, 8,936 inflater cases and 3,080 PNG API comparisons passing.
Explicit compound RAM updates remove **370 more lines**, reaching **104,424**,
with complete compiler correspondence and both full boot replay routes passing.
Shared RAM metadata removes another **452 lines**, reaching **103,972**, with
complete constant/export equality and static builds passing.
Shared record metadata then removes **271 lines**, reaching **103,701**, with
complete compiler correspondence and static builds passing.
Shared alias declarations remove **202 lines**, reaching **103,499**, including
a seven-line helper. All 184 selected alias types have compiler identity witnesses;
all declarations and remaining expanded operations match, and release builds pass.
The shader draft is excluded. The current rule and checks were reconstructed from
Git and retained outside TEMP at Local/sm-decomp-experiments/20261021/alias-groups.
Native C-string emission removes **92 further lines**, reaching **103,407**.
The actual after-alias composition preserves the same 111 byte/type/lifetime-
checked expressions and every unselected operation. Complete standalone builds
and expanded correspondence, composed source inverse, and final main release
build pass. The shader remains unchanged. Durable sources and proof are at
Local/sm-decomp-experiments/20261021/native-c-strings-alias43df.
Sharing identical constant-group headers removes **94 further lines**, reaching
**103,313**. All 1,123 declarations remain, with complete expanded correspondence
and public value/type witnesses. Actual sequential emission preserves the prior
alias and C-string changes and all original body rows; final release passes.
These three current rules remove **388 lines**, **1,870 tokens** and **7,551
spelling bytes** from the actual 103,701-line input. Durable final source/checks:
Local/sm-decomp-experiments/20261021/constant-groups/after-alias-strings-constants-v2.
Native shader/presentation ownership removes **588 more lines**, reaching
**102,725**, with 4,877 fewer tokens and 15,722 fewer spelling bytes. Complete
normal release library/frontend build and 528 shader/336 presenter DLL
comparisons pass. This incorporates the former 83-line shader draft; the
additional reduction beyond that working draft is 505 lines.
Shared PPU pixel evaluation removes **681 more lines**, reaching **102,044**,
with 6,053 fewer tokens and 16,513 fewer spelling bytes. Complete release,
175,040 actual old/new scanline comparisons and both full boot replays pass.
The native dense renderer takes 2.6-2.7x its former component time; details below.
Shared native/SPC state transfers remove **230 further lines**, reaching
**101,814**, with 2,559 fewer tokens and 7,553 fewer spelling bytes. Complete
release, source-bound differential transfer checks and both full boot replays pass.
Live-typed snapshots remove **239 further lines**, reaching **101,575**, with
2,453 fewer tokens and 6,196 fewer spelling bytes. Complete release, source-bound
capture/restore/diagnostic checks and both full boot replays pass.
The pinned CPU adapter removes **696 more Rust lines**, reaching **100,879**.
Its 41-line installer and 2,836 external C/header lines are disclosed separately.
Full boot replays and component checks pass. The PNG/inflater replacement
removes **777 more Rust lines**, reaching **100,102**; its four-line C bridge
leaves 773 net Rust/C lines removed. Complete RAM pointer chains remove
**99 further lines**, reaching **100,003**. The final readable-control-flow
candidate removes **89 more**, reaching **99,914 Rust / 99,918 Rust+C lines**;
both final full integration replays pass. A separate
rule regenerates all 24 corpse tile callbacks as calls to the current helper:
**793 C -> 2,319 initial Rust -> 208 emitted Rust lines**. Its output matches
current Rust's ordinary-function form exactly, and 126,720 C cases pass.
The section "Tailored emission learned from current Rust" records the source
lineage, runnable rules, complete-unit integration and actual limitations.

Earlier implemented C game-family replacements plus native Rust door sequences
remove **3,016 Rust lines (82,234 -> 79,218)** across 16 complete gameplay units
and shared support under identical RAM/numeric controls. That candidate has a
different scope and cannot be added to the latest complete-original result.
The earlier constructor/constant-cast prototype is included in the new composed
pipeline at its measured incremental cost. These are original-source experiments
without a restart or maintained compiler fork; current-main adoption remains a
separate measured change.

The subsequent Cpp2Rust trial builds and executes both backends on real C;
neither reduces source volume in the tested scopes. Its default mode does
preserve packed byte accesses despite physically larger Rust structs. Corrode's
specific packing omission is repairable; layout differences alone are not a
correctness verdict. The independent-transpiler sections record these tests.

## Readable control-flow composition (2026-10-21)

The final candidate removes **89 lines (100,003 -> 99,914)**, 333 tokens and
2,669 spelling bytes. Equipment selection returns a named loop result;
scrolling, grapple and Ridley clamps use ordinary branches; PPU scroll arms
share their real continuation. The previous 60-line draft's nested conditions
and side-effect Boolean expressions were rejected for readability. Both prices
cover the same five routines and are alternatives, not additive savings.

All five complete original/adopted routine bodies match the component-tested
pair on actual main, retaining the earlier RAM-chain changes elsewhere in their
files. The revised source-bound DLLs pass **1,145,096 comparisons per arithmetic
profile**, optimized and overflow-checked, covering full PPU objects, grapple
signed ranges, callback mutations, scrolling boundaries and angle clamps.
Dependent callbacks are scripted, so component coverage does not establish every
movie context. Normal main release library/frontend and formatting pass (38.71s).
Full boot any% **129712/0** and Speed Booster **148406/0** pass with normal
rendering/audio on final source
`a49e35e84ffd2c8e964c2b9df5414f9136ce9e1fb6ad6be255d1c541161b9e4b`.
Neither reaches credits. Durable source and
checks: `Local/sm-decomp-experiments/20261021/residual-control-flow/composition-e2530722/readable-v2`.

The actual whole `sm` recount has **81 Rust files / 99,914 lines**, plus **four
C bridge lines = 99,918 implementation lines**. Including the new 41-line CPU
installer gives **99,959**. Existing development tools, configuration and docs
are outside the implementation metric. External dependency source is separately
**2,836 CPU C/header + 7,762 PNG header lines**; this result reduces maintained
project source, not total dependency-inclusive source. From the 101,575-line
continuation baseline, Rust falls **1,661** lines, or **1,616** after new C/setup
costs. Both final full replays pass; bank retirement remains separate work.
The recorded original C implementation ceiling is 100,374 lines (headers
excluded); current Rust+C is 456 lines below it. Original-C inputs were not
recreated during this continuation.

## Composed RAM pointer chains (2026-10-21)

The unchanged byte-pointer emission rule removes **99 lines (100,102 -> 100,003)**,
1,717 tokens and 3,580 spelling bytes on the actual CPU/snapshot/codec composition.
All 135 chains in 24 files, including five macro templates, count; no helper or
new import is needed. The existing macro retains each separate byte offset and
its evaluation order, receiver and final pointer type.

The actual main input/output files match the source-proven composition. Every
selected caller/template body is unchanged from the cohort with 141 instantiated
contextual type witnesses, and the whole source inverse retains all other work.
That cohort also has complete normalized expanded correspondence for 7,098
functions and the ordered remainder. All original formatter controls are byte
identical. Actual main formatting and release library/frontend pass (44.52s). This
representation-only change needs static validation rather than a gameplay replay.
Durable composition: `Local/sm-decomp-experiments/20261021/ram-pointer-chains-composed-6170cbea-cpu`;
original compiler proof: sibling `ram-pointer-chains-e2530722`.

## Compatible PNG/inflater dependency (2026-10-21)

Replacing the complete translated decoder and inflater with published stb_image
v2.26 removes **777 Rust lines (100,879 -> 100,102)**, or **773 net lines** after
its four-line maintained C bridge. The unmodified header is 7,762 external lines;
this is dependency adoption, not a reduction in total linked program source.
The final Rust adapter has 64 lines; all callers and source registrations count.
The header comes from the exact published stb-sys 0.6.0 archive. No Cargo package
or build script is added. [PNG_DEPENDENCY.md](PNG_DEPENDENCY.md) gives hashes,
installation commands, required flags and MIT distribution instructions.

The pin matters: newer stb versions reject reserved DEFLATE codes accepted by
current Rust; older versions lack required 16-bit PNG support. The required
zero-initialization flag preserves missing palette entries. Both memory and file
APIs pass **13,412 comparisons on 6,704 distinct inputs**, including malformed
headers, reserved codes, palettes, depths, filters and interlacing. The final
linkage-only change retains the same decoder bodies and passes another **202
focused API checks** with refreshed embedded-source validation. All five test
sources and the installed production archive are checked against the tested
candidate. Allocation exhaustion and every malformed stream remain unproven.
Main source correspondence, formatting and the final normal release library/
frontend build pass (41.24s).
This is frontend-only, so headless gameplay replay adds no decoder coverage.
Durable sources/checks: `Local/sm-decomp-experiments/20261021/codec-dependency/v226`.

## Pinned LakeSnes CPU replacement (2026-10-21)

The complete CPU becomes **1,217 -> 521 Rust lines (-696)** through the
unmodified [dinkc64/LakeSnes revision 048a0d7](https://github.com/dinkc64/LakeSnes/tree/048a0d72568668f74e0cdf371dcd9e8efd614965).
All 15 retained compatibility/hook handlers and the per-instruction register/bus
adapter count. The 41-line installer leaves **655 net lines removed**; the
independently published dependency has **2,836 C/header lines**, separately
disclosed. This reduces maintained implementation, not total program source.
Installation, pinned hashes and MIT distribution requirements are in
[CPU_DEPENDENCY.md](CPU_DEPENDENCY.md); Unix installation is untested.

Current Rust remains the behavior authority. Its Cpu layout, legacy save span,
outer IRQ/NMI/WAI/STP sequencing, hardware bus and game hooks remain. The adapter
constructs foreign state per instruction and synchronizes general registers at
each real bus callback. Indexed timing needs explicit compatibility handling.
The final source-bound component DLL passes **524,288 instruction cases and
4,096 lifecycle cases**, with zero observable register, memory/bus/history,
returned-cycle or panic mismatches. The seven embedded sources match disk.
There are **240,214/672 intermediate callback-cycle differences**. Reachable
production bus/host callbacks do not observe that field; final cycles match.
This is not intermediate whole-Cpu byte equivalence. Future observers of that
field would require changing the adapter.

The actual main release library/frontend build passes. Full boot any%
**129712/0** and Speed Booster **148406/0** pass on the **100,879-line** source:
`3a08843e9e15b17aa59eeb5f1018dca693c40177a1f85e2fc1f2e8b13eb6efde`.
Both keep normal rendering/audio; neither reaches credits. Earlier
denied installer/build-script attempts remain failed attempts; the successful
direct C compilation and ordinary static linkage are separate authorized builds.
Durable sources, independent observer review and final main logs:
`Local/sm-decomp-experiments/20261021/lakesnes-cpu-dependency`.

## Live-typed snapshots (2026-10-21)

Removing the separate stored CPU/PPU/SNES/DMA projection types and sharing complete
RAM/SRAM diagnostic traversal saves **239 lines (101,814 -> 101,575)**. Existing
live record types store snapshots; explicit partial restore preserves all 144
ordered operations and original cache/CPU-flag normalization points. Existing
sprite/counter records remain for exact diagnostic Debug output. Extra captured
pointers/caches are inert and never followed, compared or restored. Dma.pad is
uninitialized, so only its four initialized state fields are captured. Snapshot
storage grows **7,824 bytes across three snapshots**. No live or serialized layout
changes; verifier checkpoints invalidate through existing source fingerprints.

All support/callers and 82 Rust files count. The 24 other cpu_infra functions are
unchanged. Complete normal main release library/frontend and formatting pass.
Ordinary DLL comparisons bind 15 source inputs and pass **512 capture/restore
cases, 96 complete comparisons and 22,710 exact messages**. They cover every CPU
flags byte in both emulation modes, SRAM tail preservation, pointer/history
topology, untouched fields and brightness-cache validity. An earlier executable
launch was denied and ran no cases; the authorized DLL checks are separate.
Full boot any% **129712/0** and Speed Booster **148406/0** pass on final source
`d7c1ddfdc819e388baad82ca4f277e807d976a845a3e7df179ac19713d0a1a6a`.
Neither reaches credits; copy performance was not benchmarked. Durable source,
component records and final main build/replay logs:
`Local/sm-decomp-experiments/20261021/live-snapshots`.

## Shared native/SPC state transfers (2026-10-21)

The complete transfer responsibility shrinks **230 lines (102,044 -> 101,814)**.
The 283 explicit player-map records become 32 equal-width consecutive destination
runs, expanded at compile time into the identical ordered array. One walker
handles both directions, retaining the 47 channel records, all 659 ordered copies
and eight final channel-index writes. All helpers, wrappers and 82 Rust files
count; music processing, sample generation, state types and Cargo are unchanged.

The ordinary DLL uses actual production types and embeds the full harness and
four source inputs, checked byte-for-byte before execution. All 330 compiled map
rows match. **1,040 cases / 685,360 copies per side** compare every player byte,
every ordered copy and an independent sequential-copy model over alternating
transfers. Parent independently matched the compiled tables to committed source
and checked all captured traces. Complete main release library/frontend, formatting
and final full boot any% **129712/0** / Speed Booster **148406/0** pass on source
`a7d82ec0f552ff40614d7cba0bb195ef057a33c307c23b46da4899c879a4798f`.
Neither route reaches credits. The initial Speed Booster launch omitted the SDL
library environment and loaded no DLL; correcting it ran the same compiled source.
Its failed log remains. This is a state-transfer reduction, not a full music-engine
replacement. Durable candidate, all compiled maps/traces and final main logs:
`Local/sm-decomp-experiments/20261021/native-sequencer`.

## Shared PPU pixel evaluation (2026-10-21)

The complete replacement removes **681 lines (102,725 -> 102,044)**, including
all helpers and callers. `ppu.rs` becomes 1,428 lines from 2,109. Native and
legacy rendering share planar decoding, Mode7 sampling, windows and per-channel
composition. Explicit policies retain their differing priorities, window rules,
subscreen criteria, brightness handling and saved fixed-color packing. Sprite
evaluation, MMIO, state copying and legacy save layout remain unchanged.

Normal release library/frontend builds and independent source review pass.
Source-checked ordinary DLL comparisons pass **7,616 cases / 175,040 scanlines**,
covering both renderer selections, all modes, windows, sprites, color math,
noncanonical saved fields and dense 256-row histories. Every output allocation
and Ppu byte compares except output pointers and the unused `bgBuffers` cache.
The cache has no remaining runtime reader and stops updating; its declaration
and checkpoint representation remain. Existing source fingerprints invalidate
old verifier checkpoints. This deliberately changes private cache contents;
it does not claim whole-Ppu byte identity. `g_ram` and legacy save layout stay
byte-compatible.

Full boot any% **129712/0** and Speed Booster **148406/0** pass on final source
fingerprint `2df5b594d509743f3be3aeaba787add99673680dbce2aac0543d64373225f2e0`.
Neither reaches credits. Both replay branches use the same backend, so the
separate old/new DLL comparisons supply independent pixel evidence.

Measured dense native Mode1 and Mode7 cost **2.69x and 2.61x** their former PPU
time, approximately **3.79ms and 1.12ms per 224 rows** in the optimized component
benchmark; legacy timing is approximately unchanged. These are not whole-game
frame timings or real GPU measurements. An eight-line inline-only candidate
passed its comparisons but recovered only about 1% in Mode1 and none in Mode7;
it is rejected. This replacement reduces video implementation duplication; it
does not retire a gameplay bank or the SNES backend. Durable source, full costs,
component results and final main build/replay logs:
`Local/sm-decomp-experiments/20261021/ppu-render-native`.

## Complete shader/presentation replacement (2026-10-21)

The complete replacement on `498c8a4d` removes **588 lines** across shader
loading/rendering (1,389 -> 952), presentation (331 -> 230) and shared types
(1,004 -> 954). All 82 production Rust files count. No dependencies are added;
PNG/inflater behavior and GL loading remain unchanged. Owned passes, parameters
and LUT records replace linked lists and allocation bookkeeping. Stage
compilation, uniform lookup/binding and scaling share named operations.

Normal optimized library/frontend builds and independent source review pass.
Ordinary source-checked DLL tests compare actual implementations: **528 shader
cases** against committed and working-draft baselines, plus **336 presenter
cases**, matching **1,522,996 ordered events**. Twelve-frame sequences exercise
history and resizing; cases cover presets, includes, raw byte names, NaNs,
numeric prefixes, duplicate parameters/textures, compile/link failures, actual
renderer callbacks, filtering/aspect flags and allocation reuse. Existing
20-pass parameter panics, history-depth overwrite and viewport quirks remain.
The user explicitly confirmed this ordinary DLL/ctypes workflow is permitted.
No Defender configuration changes or denied executable results are counted.
Controlled GL/SDL calls are tested; real GPU pixels and allocation exhaustion
are untested. No headless gameplay replay is claimed for frontend-only code.
Durable sources, full metrics and checks:
`Local/sm-decomp-experiments/20261021/native-shader`.

The dependency alternative was investigated first. No inspected pipeline is a
compatible drop-in for the existing GLSL/glslp contract:

- **librashader 0.12.1**, pinned at `07ac790c`, implements a complete Slang
  pipeline. Its [stage preprocessor](https://github.com/SnowflakePowered/librashader/blob/07ac790cdb9826f37cf8e1daf5011f03e3c4daa7/librashader-preprocess/src/stage.rs)
  uses `#pragma stage`; its [compiler input](https://github.com/SnowflakePowered/librashader/blob/07ac790cdb9826f37cf8e1daf5011f03e3c4daa7/librashader-reflect/src/front/glslang.rs)
  uses Vulkan GLSL/SPIR-V. OpenGL is an output backend. The
  `parse_legacy_glsl` feature concerns preset metadata and does not supply the
  existing `VERTEX`/`FRAGMENT` shader input contract.
- **RetroArch**, pinned at `3f90f6d`, has the correct dialect in
  [shader_glsl.c](https://github.com/libretro/RetroArch/blob/3f90f6dc56cee2f64e92f489c5c4f4d6c935be44/gfx/drivers_shader/shader_glsl.c),
  but rendering/history ownership also resides in frontend
  [gl2.c](https://github.com/libretro/RetroArch/blob/3f90f6dc56cee2f64e92f489c5c4f4d6c935be44/gfx/drivers/gl2.c).
  Extracting it adds maintained adapters and GPLv3-or-later implementation;
  it is not a standalone library substitution for this MIT project.
- **libplacebo's** [custom shader API](https://github.com/haasn/libplacebo/blob/master/src/include/libplacebo/shaders/custom.h)
  parses mpv shader hooks, another contract.

These are primary-source compatibility findings, not executed upstream shader
comparisons or dependency savings. The native replacement's saving is measured
separately and does not assume shader conversion is free.

## Complete CPU and codec dependency trials (2026-10-21)

These isolated alternatives start from `498c8a4d` / **103,313 lines**. Neither
is adopted, and their independent prices are not added to the shader result.

| Complete replacement | Formatted source price | Result |
| --- | ---: | --- |
| CPU through pinned `wdc65816`, all compatibility adapters included | 1,217 -> 450; whole project 102,546 (-767) | Uncompiled; dependency build script denied |
| PNG/inflater through `stb_image = "=0.3.0"`, 44-line adapter | 837 -> 44; whole project 102,520 (-793) | Rejected: concrete decoder incompatibility |

The CPU uses [super-yane's standalone core at `ad2f1f5`](https://github.com/josefwaller/super-yane/tree/ad2f1f5d252e3cbde7ed30be48909b68df85aaae/wdc65816).
Public registers, instruction stepping and bus callbacks permit an adapter that
preserves the existing layout, snapshots, opcode hooks and interrupt sequencing.
Sixteen explicit legacy handlers and two identified bus-order/read-count repairs
are charged in its 450 lines. External production code is 3,911 Rust lines plus
dependencies; one Cargo declaration is added. The prepared 20,480-case DLL suite
executed **zero cases**: Defender denied the proc-macro2 build script before
compilation. Source checks are not proof of delegated CPU equivalence. No
compatible compiled dependency cache was found; no blocked build script was
converted to a DLL. Preserve this prototype pending a normal dependency build.

The already-inspected [w65c816 0.1.17](https://docs.rs/crate/w65c816/0.1.17/source/)
has no dependencies or build script, with cycle callbacks and cloneable state.
Its setters normalize stack/index registers and it has no raw-state/force-fetch
API. That defeats direct import by the existing adapter; it does not prove a
different adapter impossible. No replacement price or execution is claimed.

The PNG alternative's external direct package contains 534 Rust and 7,987 C
lines, with one Cargo and three build-configuration lines added locally.
Its [bundled stb_image 2.28 decoder](https://github.com/servo/rust-stb-image/blob/c977c0799c5dee7c58402daabfa3bac4029da08f/src/stb_image.c#L4314)
rejects reserved length symbols 286/287. An ordinary DLL containing the actual
unchanged current Rust decoder executed three 1x1 grayscale PNGs: the normal
control and both reserved-symbol variants return pixel `0x41`. Thus the direct
replacement changes accepted input behavior. Upstream rejection is source
reasoning, not an executed C comparison. Its normal build separately stopped at
a denied libc build script; all 6,604 generated broader fixtures remain unrun.

Both denied builds recorded Defender event 1121 under rule
`01443614-CD74-433A-B99E-2ECDC07BFC25`; no settings were changed and no failures
are reported as passes. Durable candidates, complete prices, sources and checks:
`Local/sm-decomp-experiments/20261021/{cpu-dependency,codec-dependency}`.

A bounded whole-oracle review also finds no drop-in deletion budget. Ares
`9408cb43` exposes an [instruction tracer callback](https://github.com/ares-emulator/ares/blob/9408cb43d4948fc3ea6e152a307a34348df3fe04/ares/sfc/cpu/debugger.cpp#L19)
and writable debugger memories, but [CPU interrupt/DMA/timing internals](https://github.com/ares-emulator/ares/blob/9408cb43d4948fc3ea6e152a307a34348df3fe04/ares/sfc/cpu/cpu.hpp#L92)
are private and precise stop/resume/state translation remains unresolved.
Locally, native register access, frame drawing and audio retain the same
PPU/DMA/bus/cartridge/DSP, while Bang_Main still reaches RunAsmCode. An external
oracle alone therefore retires none of those complete responsibilities. This
is source feasibility, not implementation or runtime evidence; a combined
native-backend/oracle replacement remains unpriced. Pinned source and local
interface maps: `Local/sm-decomp-experiments/20261021/oracle-dependency`.

## A directly relevant successful migration

**rav1d is a useful precedent.** Its [deduplication issue #26](https://github.com/memorysafety/rav1d/issues/26)
classifies duplicates created by C2Rust: declarations, types, constants,
expanded macros and template files. [Issue #25](https://github.com/memorysafety/rav1d/issues/25)
identifies algorithms separately transpiled for 8-bit and 16-bit modes.
The team replaced those variants with shared Rust generics:

| Merged change | Added | Deleted | Net reduction across the whole PR |
| --- | ---: | ---: | ---: |
| [#588: cdef_apply](https://github.com/memorysafety/rav1d/pull/588) | 148 | 679 | 531 |
| [#591: cdef](https://github.com/memorysafety/rav1d/pull/591) | 807 | 1,420 | 613 |
| [#683: itx](https://github.com/memorysafety/rav1d/pull/683) | 730 | 2,502 | 1,772 |

These are GitHub PR API additions/deletions, including support and caller
changes. The seven-file #683 includes a five-line build change; it is not
strictly a Rust-only count. [Cleanup issue #817](https://github.com/memorysafety/rav1d/issues/817)
explicitly waits for consolidation to avoid cleaning both implementations.
The transferable lesson is to identify what caused implementations to multiply,
recover one algorithm with explicit variants, then clean the shared result.
Bit-depth templates supply unusually clear common structure; these numbers
cannot be extrapolated to Super Metroid's distinct behaviors.

## Useful tools and their limits

| Tool | Relevant capability | Limit for this project |
| --- | --- | --- |
| [jscpd](https://github.com/kucherenko/jscpd/blob/v5.4.2/docs/rust.md) | Rust token clones; identifier/literal normalization; near-miss gaps; function AST similarity including Rust | Detection suggests candidates, not safe extraction. Its experimental embedding mode is similarity ranking, not equivalence proof. |
| [PMD CPD](https://docs.pmd-code.org/latest/pmd_userdocs_cpd.html) | Mature token-based copy/paste detection; [Rust supported since 7.9](https://docs.pmd-code.org/latest/pmd_languages_rust.html) | Rust support is CPD-only. Normalization flags are language-specific; do not assume Java's options apply to Rust. |
| [Copydetect](https://github.com/blingenf/copydetect) | Local Python/Pygments scanner using winnowing; normalizes names and strings | File-pair plagiarism detector: default run misses clones within one file, preserves numeric values, and does not recover a shared algorithm. |
| [ast-grep](https://ast-grep.github.io/guide/pattern-syntax.html) | Rust-capable structural search and supplied-pattern rewrites with metavariables | Useful for applying a discovered transformation consistently; it does not invent the abstraction or verify behavior. |
| [NiCad 7](https://github.com/CordyJ/Open-NiCad) | Near-miss detection, renaming, normalization and clone classes | Its documented bundled languages exclude Rust. A Rust grammar/plugin would be extra integration work. |

The useful distinction is exact copies (Type 1), renamed/value variants
(Type 2), edited structural copies (Type 3), and differently written similar
behavior (Type 4). A token detector alone will miss many latter cases. Blind
identifier normalization can also hide different data flow: `x + x` and
`a + b` are not equivalent. Macro bodies, state addresses, wrapping arithmetic,
aliasing and ordered RAM effects need particular care here.

### What C2Rust itself provides

The [official C2Rust README](https://github.com/immunant/c2rust) describes
behavior-preserving translation followed by refactoring and human/agent work.
Its refactor tool has concrete batch transformations:

- [`abstract`](https://github.com/immunant/c2rust/blob/master/c2rust-refactor/src/transform/funcs.rs)
  extracts a supplied expression pattern into a function and rewrites matches.
- [`rewrite_stmts`](https://github.com/immunant/c2rust/blob/master/c2rust-refactor/src/transform/rewrite.rs)
  rewrites matched statement sequences across the crate.
- [`generalize_items`](https://github.com/immunant/c2rust/blob/master/c2rust-refactor/src/transform/generics.rs)
  replaces marked concrete types with generic parameters and updates uses.
- [`reconstruct_for_range`](https://github.com/immunant/c2rust/blob/master/c2rust-refactor/src/transform/control_flow.rs)
  conservatively reconstructs loops when its induction-variable conditions hold.

These transformations require the caller to select patterns and conditions.
The refactor build is pinned to nightly-2023-04-15 and the documented hosts
are Linux/macOS. It is a useful reference, not a drop-in Windows cleanup command
for our current Rust. Header/declaration deduplication and ownership migration
are separate problems; much of the former is already harvested in this repo.
That initial Rust-side investigation did not consult original C. The later
C-side experiments below answer a different, explicitly requested question.

## The research term: anti-unification / clone refactoring

[Li and Thompson's PADL 2010 paper](https://kar.kent.ac.uk/30703/) derives the
least-general common abstraction of similar fragments, then eliminates clones.
Instead of merely reporting two matching blocks, this produces a shared
skeleton with the differences exposed as parameters. Their
[Wrangler tool](https://refactoringtools.github.io/wrangler/) supports this
workflow for Erlang, not Rust. It demonstrates a process we can use; it is not
a ready Rust installation or proof that all behavior can share one engine.

## Bounded local tool trial

Scanned the frozen committed source at **f5665502**: 77 Rust files,
**109,704 lines**, excluding the unrelated shader draft. Installed tools and
reports remain in TEMP, outside the repository. No source was rewritten.

Downloaded jscpd 5.4.2's Windows wheel and checked its PyPI SHA-256, but Windows
rejected launching its executable with error 5. Thus no local jscpd scan or
runtime confirmation of its documented flags is claimed.

Copydetect 0.5.0 ran successfully through Python. Both passes used its existing
API, Rust tokenization where enabled, and noise/guarantee thresholds of 100:

```python
detector = CopyDetector(test_dirs=[snapshot_src], extensions=["rs"],
    noise_t=100, guarantee_t=100, force_language="rust",
    disable_filtering=raw_pass, autoopen=False, silent=True)
detector.run()
```

The raw pass took 11.49 seconds and found 618 matching file pairs; the filtered
pass took 13.44 seconds and found 277. Unions of source lines touched by matches
were 7,265 and 4,212 respectively. **Neither number is removable lines, a
duplication percentage, or an upper bound on possible savings.** Raw and
filtered thresholds count different character representations and are not
equally sensitive: normalization shortens the source before matching.

Inspection found repeated imports/declarations, Waver/Goofball RAM accessor
wrappers, and Ridley/Mother Brain sprite-output fragments. The sprite loops
have different clipping conditions. These are candidates or known repetition,
not demonstrated complete replacements. This trial does not cover same-file
clones, numeric-literal normalization, near-miss structures, or semantic
equivalence. It validates an available discovery tool, not completion of the
larger audit. Reproduction script and raw results: TEMP/sm-clone-tool-research.

## Recommended change to the work order

Exact scans and whole-function templates have already been tried here; simply
repeating them is not a new strategy. The next experiment should test whether
**different names, constants, types or control-flow shapes conceal an algorithm
repeated across existing boundaries**:

1. Analyze the whole maintained tree, grouping gameplay and backend findings
   separately. Use normalized and structural comparisons, including clones
   within a file. Preserve source locations and count overlapping spans once.
2. For large groups, derive the common skeleton and list the true differences:
   axes, widths, thresholds, animation/state destinations, or effect order.
   Recover loops or normalize temporaries only when it enables that larger merge.
3. Price every instance together: retired bodies minus shared implementation,
   parameter data, exceptions, callers and imports. Rank by aggregate net
   deletion and verification cost, not pair similarity or largest file.
4. Apply one structural transformation to the complete valid group, preserving
   distinct behavior explicitly. Then use the existing differential/oracle
   checks for every variant and integration history. Clean the shared result.

The output should be a few priced cross-cutting replacements, with rejected
groups explained, before selecting another small family. Do not build a new
scanner, dashboard or permanent evidence system. No reviewed source provides
a turnkey Rust-to-compact-game-engine converter or establishes our remaining
reduction budget. The strongest practical precedent is consolidation before
cleanup; the new question is where that multiplicity still exists here.

## Executed experiment on the original transpilation

At the user's request, recovered **1311d3b4**, the earliest committed transpiled
Rust with initial mechanical compile fixes: **278,936 lines / 54 files**.
No C was regenerated. Parallel isolated studies used Copydetect, the official
ast-grep Python bindings and tree-sitter to find and transform repeated code.
Agents designed the shared implementations; these tools do not choose or
prove an abstraction automatically. All drafts remain outside main.

| Actual combined stage | Rust lines | Additional net deletion |
| --- | ---: | ---: |
| Original, unchanged by the same formatter | 278,936 | 0 |
| Shared corpse tile algorithm, preserving 24 callbacks | 276,862 | 2,074 |
| Shared duplicate types/constants | 261,656 | 15,206 |
| Shared compatible foreign declarations | 260,468 | 1,188 |
| Shared RAM and constant-ROM address expressions | 229,249 | 31,219 |
| **Combined result** | **229,249** | **49,687 (17.8%)** |

All helper, caller, import and registration costs count. The address experiment
alone saves 31,703; combining it after algorithm sharing avoids double-counting
484 overlapping lines. No formatting-width change contributes savings.

Original and combined libraries pass optimized code generation and frontend
metadata checks. Header passes preserve 6,888 input function definitions;
12,692 compiler-evaluated layout/constant values agree with the original.
Inverse expansion of 24,426 combined address calls restores the preceding
source tokens. The unchanged extracted helpers pass 17,301,508 pointer-result
comparisons, and the corpse algorithm passes 90,992 optimized full-RAM routine
comparisons. These are static and constructed checks, not a full game replay
or SNES/render/audio validation. The experiment changes no maintained gameplay.

The comparison snapshot **f57e4abf**, **109,556 lines**, already shares these
algorithms and address primitives. The address queries find zero current
matches; selected declarations expose only 65 gross duplicate lines before
costs. Consequently **the original saving is not additional current savings**.
It establishes a useful early-transpilation workflow. The user's subsequent
question concerns discovering repeated algorithms in the original C and
improving translation inputs; it does not ask for a restart comparison.

Scanner coverage also differs greatly: raw cross-file matches touch 138,426
original lines versus 7,290 in that current snapshot; normalized AST matches
at an 80-token threshold touch 31,748 versus 2,327. These overlapping matched
lines include retained implementations and are not deletion budgets. Same-file
scans were included; 35 original function bodies with parser errors were
excluded from structural analysis. Data-flow equivalence remains outside it.

Runnable sources, complete patches and reports: TEMP/sm-original-transpile-experiments/.
The combined candidate is in combined/sm/src, its exact patch in
combined/trial.patch, and the full results/limits in combined/REPORT.md.
Independent studies are declarations_extern/, clone-folding/,
algorithm-variants/ and tool-audit/. The original and current inputs were
checked byte-for-byte against Git. No ROMs or save states were added.

## Experiments restructuring the original C before C2Rust

The user clarified the question: identify large repeated algorithms while the
original C still exposes their structure, and test changing that C before
translation. A restart comparison does not answer this question. Original C
access is explicitly authorized for these isolated experiments; maintained
Rust remains the implementation.

### Original C size and provenance

The original Rust commit 1311d3b4 names snesrev/sm **578f90b** and C2Rust 0.22.1
as its inputs. Recovered that exact upstream revision,
[`578f90b3cc49557bb70060ad033bb90b8cf8ac50`](https://github.com/snesrev/sm/tree/578f90b3cc49557bb70060ad033bb90b8cf8ac50),
and checked every extracted C/H file against the downloaded archive.
Archive SHA-256:
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.

| Original source scope | Files | Lines |
| --- | ---: | ---: |
| Gameplay banks, `src/sm_XX.c` | 32 | 83,657 |
| SNES backend, `src/snes/*.c` | 10 | 7,444 |
| Runtime, audio, frontend and verifier C | 9 | 6,084 |
| Bundled OpenGL loader C | 1 | 3,189 |
| **C files selected by the original Makefile** | **52** | **100,374** |
| Other platform C, outside that build | 2 | 126 |
| **All C implementation files** | **54** | **100,500** |
| First-party headers | 29 | 17,995 |
| Bundled headers | 2 | 11,600 |
| **All C and headers** | **85** | **130,095** |

These are newline counts, matching `tools/progress.sh`, including comments and
blank lines. As with `wc -l`, an unterminated final line is not counted; there
is one in the build's C files and eight across the full C/H inventory.
The original Makefile selects `src/*.c`, `src/snes/*.c` and
`third_party/gl_core/gl_core_3_1.c`. Its first-party C subset is 97,185 lines.
Headers contain declarations, layouts and macros, plus the bundled stb_image
implementation; they are not all redundant prototypes. Comparing all maintained
Rust against C implementation files alone is therefore a deliberately stricter
target than counting all source on both sides.

Use the requested **under-100,000 Rust ceiling**, preferably lower, rather than
raising it to include the headers. The C implementation supplies a concrete size
baseline, not proof that the current approach will automatically reach it.
Reproducible file-by-file counts: TEMP/sm-c-input-experiments/c-baseline.json;
the adjacent `count_c_baseline.py` checks archive contents and the build scope.

### Two shared algorithms, measured through the actual translator

Parallel experiments rewrote original C and ran **actual C2Rust 0.22.1** on
unchanged and rewritten complete bank translation units. They use the same
project-header amalgamation, remote Linux standard headers, translation flags
and local rustfmt at 100 columns. All generated helpers, declarations and root
module registrations count. These are fresh paired translations, not a
byte-identical reconstruction of the historical compile database.

| Complete experimental scope | Before Rust | After Rust | Net removal |
| --- | ---: | ---: | ---: |
| Scrolling DMA and tile streaming, complete sm_80 unit | 8,015 | 7,496 | **519** |
| Collision, complete sm_94/sm_a0/sm_86 units plus helper and roots | 35,330 | 34,686 | **644** |

The disjoint groups remove **1,163 lines combined**, including the added module
registration. Their generated files are compiled; this is measured source
reduction in the experiment, not deletion already applied to current main.

**Collision:** twelve routines across Samus, post-grapple movement, player
projectiles, enemy projectiles and enemies repeat the same slope geometry under
different temporary names, context layouts, table encodings and branch trees.
The replacement shares square-slope occupancy and two nonlinear vertical
responses. It retains three distinct sampling contracts: legacy span, clipped
span, and point. A generic rectangle test would erase real behavioral differences.
The original C/H rewrite removes 243 lines including its helper/header costs;
the actual generated Rust removes 644 after full-module costs.

**Scrolling:** four layer/direction variants share one ordered DMA transfer
algorithm. A second shared algorithm maps a block's four tile quadrants using
its horizontal/vertical flip bits, replacing the repeated orientation branches
in row and column streaming. A controlled scrolling-only comparison produces
443 Rust lines originally, **506 with only a recovered struct layout**, and
**182 with algorithm sharing plus that layout**. Introducing types alone grows
the output; removing repeated work produces the saving. These extracted-unit
counts explain the result and are not added to the full-bank saving above.

Collision passes 360,000 original/replacement native-C comparisons and 48,000
original-C/generated-Rust comparisons, checking all 128 KiB RAM, returns and
collision context. Scrolling passes 6,250 cases across six compiled C/generated-
Rust variants (original, layout-only and shared), comparing full RAM and 358,220
ordered register write events. Tile streaming passes 3,076 cases across four
C/generated-Rust variants, both directions, layers, four flip orientations,
wrap positions and Mode 7 bypass.

Full affected generated modules pass Rust metadata compilation after identical
obsolete-feature/pointer-API compatibility edits on both sides. Executable
collision and scrolling checks use optimized generated Rust with matched
dependencies.
The constructed tests do not establish whole-game/SNES replay equivalence.
DMA uses a register-event recorder rather than emulated PPU transfers; collision
uses valid object indices and bounded shape/radius domains, not arbitrary C
undefined behavior. Those limits remain adoption requirements.

TinyCC 0.9.27 supplies the local C compiler. Compiler Explorer's
[`c2rust-master`](https://godbolt.org/) service reports version 0.22.1 for each
translation; a pinned service-binary hash is unavailable. Exact public C inputs,
version responses, translator output and formatting hashes are retained locally.
Only public upstream C and new nonsecret helpers were submitted. Local ROM
data, save states, test buffers and private current source were not submitted.

### Translation-wide control: high-byte/high-word access

A third experiment changes the original `HIBYTE`/`HIWORD` definitions to call
two shared pointer helpers. It retains lvalue access, single address evaluation
and the actual operand width. All **394 affected call sites in 30 C files** were
retranslated; low-byte accesses were left alone. The complete measured cohort
is **226,966 -> 226,396 Rust lines: 570 removed**, including the 32-line emitted
helper and the extra root registration. Thirty original units alone contain
226,936 lines; their replacements contain 226,333 before these costs.

This is a useful preprocessing change but only a **0.25%** reduction in its
tested scope. It does not provide a large algorithmic reduction by itself.
It overlaps the gameplay experiments, so **570 is not added to their 1,163**.
The broader count of 709 partial-access sites includes unchanged low-byte
operations and must not be used to inflate this experiment's coverage.

Native checks pass 16,384 pointer/lvalue cases across widths and input bytes,
including compound writes and guarded full-buffer comparison. Another 7,680
comparisons between the original C macros and actual generated Rust helpers
pass for pointer results and reads/writes. All 61 original/replacement/helper
units pass C compilation and Rust metadata checks; the 30/31-module roots
also pass. Both CPU-infrastructure controls require enabling `extern_types`
for the generated opaque glibc FILE declaration. Compatibility settings are
identical between controls; complete-game linking and performance are untested.
Exact paired outputs and per-file counts are in `toolchain/partial-access/`;
the source change is in `translation-shapes/types-partial-access.h` and
`translation-shapes/partial-access.c`. These access checks are distinct from
gameplay-equivalence or whole-game replay claims.

Artifacts under TEMP/sm-c-input-experiments: `algorithm-study/REPORT.md`,
`algorithm-study/trial.patch`, `translation-shapes/`, and `toolchain/REPORT.md`.
The native and generated-Rust comparison scripts, paired complete C/Rust units,
raw service responses and compiler records remain there for review and reruns.

The actionable process is to group original C by a repeated operation across
banks, identify its real variants, replace the whole group with a named shared
algorithm, then translate both versions and price the complete result. This
produces domain seams such as slope geometry and tile streaming. It is a useful
step toward descriptive engine modules; it does not by itself retire entire
banks or establish the remaining whole-game reduction budget.

## Broader C transformations and a tailored emission profile

The expanded experiment treats complete instruction systems and all eligible
clone groups together. It then tests C2Rust's emission policy separately. The
scope is the original public C revision already identified above; experiments
remain isolated from maintained Rust.

### Whole-cohort C transformations

| Complete responsibility | Actual standalone Rust removal | Scope |
| --- | ---: | --- |
| Shared algorithm groups and retired forwarding functions | 2,200 | 35 groups / 165 routines; 143 wrappers retired, 171 callers changed; 18 units |
| Enemy instruction execution | 1,522 | 419 handlers fused into native opcode executors; seven genuine wrappers retained; 12 units |
| PLM and enemy-projectile instruction execution | 866 | 200 handlers retired behind 201 existing opcode entries; two units |
| Presentation instruction execution | 356 | 170 handlers in seven animation, HDMA, palette and cinematic dispatchers; four units |
| Word arithmetic representation | 2,373 | 2,381 typed countdown, signed-comparison, sign-mask and pair-construction occurrences; 32 units plus helper |

These standalone numbers overlap. Applying the transformations in sequence to
the original C, retranslating the result and charging all helpers/declarations/
module registrations gives **229,295 -> 222,068: 7,227 net Rust lines removed**.
The cohort contains all **32 gameplay banks**, CPU infrastructure and SPC
player; the result adds one math module. It excludes the remaining frontend
and SNES backend units and is not a whole-project total.

All 69 original/replacement C object builds and Rust metadata builds pass,
as do both complete module roots. The immutable composed C input was checked
throughout translation. C2Rust version, source/options, raw outputs and exact
per-file counts are retained. No standalone saving was added to the combined
number without rerunning the composed source.

Validation covers all 165 clone instantiations and all 789 fused instruction
bodies structurally. Executed component checks include 10,560 clone cases,
18,112 enemy-script cases, 33,792 PLM/projectile cases and 5,394 presentation
cases, comparing complete RAM and relevant returns/call traces. The arithmetic
study checks 1,114,112 primitive cases and 9,984 states across 39 real gameplay
routines in both native C and generated Rust. Runtime script coverage is
partial: 236 changed enemy handlers, 88 PLM/projectile opcodes and every 174
presentation opcode entry; other paths have the stated structural checks.
Some unchanged external callees are instrumented. These are not full-game,
SNES, rendered-frame or audio comparisons.

This broader experiment also rejected alternatives on measured evidence.
PLM/projectile declarative action rows remove only 342 lines after generated
table/type costs, less than the 866-line native dispatcher. Packed rows improve
that count but expose an enum ABI mismatch and add C complexity. CFG-option
trials offer no substantial reduction, and one option emits an invalid label.
Changing all 15 coroutine macros to explicit resume labels saves only 50 lines.
Those are controls, not selected reduction workstreams.

### The emission policy accounts for substantial expansion

Inspected C2Rust v0.22.1 at
[`e9ccd715597f7d63e8635629f4450e040a230118`](https://github.com/immunant/c2rust/tree/e9ccd715597f7d63e8635629f4450e040a230118).
Its [`mk_int_lit`](https://github.com/immunant/c2rust/blob/e9ccd715597f7d63e8635629f4450e040a230118/c2rust-transpile/src/translator/literals.rs#L12-L25)
constructs an unsuffixed literal and then casts it to its target type. Its
[`compound-assignment emitter`](https://github.com/immunant/c2rust/blob/e9ccd715597f7d63e8635629f4450e040a230118/c2rust-transpile/src/translator/operators.rs#L200-L283)
widens an operand to Clang's compute type, calculates, then narrows it again.
Some of this generated syntax can be replaced with shorter equivalent typed
operations; it is not independent gameplay logic.

The runnable prototype applies these policies to actual emitted Rust:

```rust
// Examples, only when operand and result types are proved:
0 as uint16                         // becomes 0u16
65535 as i16                        // becomes (-1i16)
(a as i32 - b as i32) as u16        // becomes a.wrapping_sub(b), for u16 operands
```

It proves types from aliases, declarations, fields and expression structure,
skips unknown/mixed cases, preserves operand evaluation order and does not
rewrite arbitrary multiplication, shifts or division. This is an executable
**emission postpass prototype**, not a claim that a modified C2Rust binary was
built. The source study supplies conservative, unbuilt emitter patches as
implementation entry points; they do not alone reproduce every tested rewrite.

| Original complete 54-file generated crate | Rust lines | Cast AST nodes |
| --- | ---: | ---: |
| Original, unchanged by the same formatter | 278,936 | 253,611 |
| Proved promotion/arithmetic normalization | 275,526 | 233,877 |
| Plus equivalent typed integer literals | **243,936** | **145,534** |

The **35,000-line** result comprises **3,410** from promotion/arithmetic
normalization and **31,590** from literal emission. The latter eliminates
88,343 cast expressions. It reduces transpiler verbosity; it does not mean
31,590 lines of independent game behavior were deleted. No format-width
change, helper outside the counted tree or removed validation contributes.

Applied to the fresh composed C output, the same policy produces:

| Measured gameplay/support cohort, one formatter | Rust lines |
| --- | ---: |
| Unchanged original C translated and formatted | **229,348** |
| Composed C changes translated and formatted | 222,073 |
| Plus promotion/arithmetic normalization | 219,781 |
| Plus typed literal emission | **185,133** |

The combined result is **44,215 lines removed (19.3%)**, including the new
helper and module root. It is counted directly, not computed by adding the
whole-original 35,000 to the C experiment. Original and fresh scopes differ.
For exact provenance, the prior toolchain formatter produced 229,295 and
222,068 before/after C changes; applying the final identical formatter adds
53 and five lines respectively. Raw-baseline deletion is 44,162. The table
above uses the same formatter on both sides and gives no credit for changing
formatting rules.

Both final numeric candidates pass complete optimized Rust library code
generation. Every distinct rewritten constant has a compiler-evaluated
equality assertion: **9,040 original** and **8,430 fresh** values/types.
Arithmetic checks pass **4,325,376 operand pairs** plus **262,144 ordered
side-effect sequences**. These establish the stated typed identities and
compiled output; complete-game linking/replay remains outside this experiment.

A separate loop/local emission pass removes 428 lines across the original
crate: 303 dead initialization merges and 96 bounded loops. Optimized library
and frontend compiler checks and 672 iteration-trace comparisons pass. It is
not included in the combined table. The larger priority is the measured
numeric policy and complete C responsibility replacements.

### Native internal calls and shared declarations

A separate complete-gameplay emission experiment resolves internal calls to
their owning Rust modules and emits ordinary Rust function/callback types.
It converts **6,300 functions across all 32 gameplay modules**, including four
shared callback aliases, and removes **5,617 export attributes**. Real hardware
and frontend C interfaces remain at their existing boundaries. This changes
how internal routines are emitted and connected; it does not remove banks or
claim a new gameplay architecture.

Starting from the previously measured shared-declaration baseline, the complete
formatted crate goes **262,542 -> 254,713: 7,829 fewer lines**. Owner imports
account for 1,018 and native calls/export removal for 6,811, including normal
formatter wrapping effects. That price is incremental to the old declaration
deduplication, not another claim for its 16,394 lines.

Directly composing these policies with the original numeric candidate gives:

| Whole original generated crate, composed in sequence | Rust lines |
| --- | ---: |
| Original | 278,936 |
| Numeric policy | 243,936 |
| Shared type declarations | 228,736 |
| Shared foreign declarations | 227,548 |
| Calls imported from owning Rust modules | 226,530 |
| Native gameplay function/callback types and exports | **219,709** |

This is **59,227 fewer lines**, measured on the complete generated source.
The final two stages contribute 7,839 in this order, ten more than standalone
because the numeric pass changes formatting. Both native candidates pass full
optimized Rust library code generation and frontend metadata checks. This
whole-original experiment and
the fresh-C 44,215-line experiment have different scopes and overlapping
transformations; neither can be added to the other or the earlier 49,687-line
original-transpilation experiment. No current-main saving is established.

An executable DLL witness links the complete standalone owner-import and
native-call libraries and invokes nine real gameplay routines through typed optional function
pointers. **196,608 primitive cases** and **1,024 mixed callback cases**, the
latter comparing all 128 KiB RAM, produce identical 135,008,256-byte result
streams. Host stubs abort if called and none execute. This checks selected
calling-convention and callback behavior; it does not exercise all converted
routines or full-game linking/history. No ROM, movie or SNES replay claim.
Artifacts: TEMP/sm-transpiler-tailoring/native-emission/REPORT.md, its staged
candidate trees, compile logs, composition scripts and callback witness.

### A concrete C2Rust layout defect

The source investigation also found a correctness issue: with a forward
declaration before a pragma-packed definition, C2Rust emits a seven-byte
`VramWriteEntry` as an eight-byte Rust structure. Executed field-offset checks
show C destination offset 5 versus Rust offset 6, although the translated
`offsetof` constant still says 5. Adding an explicit packed attribute to the C
definition, or omitting the forward declaration, produces the correct 7/5
layout. Both input alternatives were actually translated, compiled and checked.

The [`AST exporter`](https://github.com/immunant/c2rust/blob/e9ccd715597f7d63e8635629f4450e040a230118/c2rust-ast-exporter/src/AstExporter.cpp#L2203-L2297)
reads record layout from the definition but pragma alignment from the canonical
forward declaration. The fix should obtain both from the definition. A source
patch applies cleanly but has not been built; the explicit C attribute is the
verified workaround. This fixes behavior and earns no size-reduction credit.

Other documented seams include temporary references for complex lvalues,
synthetic zero initialization, internal C linkage and macro expansion. The
generic function-macro option is not usable as a compactness switch: tested
full-bank output contains undefined Rust macros and untranslated C argument
tokens. The [option's own source](https://github.com/immunant/c2rust/blob/e9ccd715597f7d63e8635629f4450e040a230118/c2rust-transpile/src/lib.rs#L48-L61)
describes its incomplete support.

Artifacts: TEMP/sm-large-c-experiments/combined/TRANSLATION_REPORT.md and its
`final/src` and `transpiled` trees; per-family reports under that experiment's
siblings. TEMP/sm-transpiler-tailoring/numeric-emission/REPORT.md contains the
final whole-original and fresh sources, exact patches and repeatable commands.
The sibling `source-study/REPORT.md` links each emission decision to pinned
upstream source and includes both unbuilt translator patches. Only public C
and nonsecret experimental code were sent to Compiler Explorer; ROM data,
save states, test buffers and private current source stayed local.

## Learning from the compact source before expansion

These experiments answer the latest clarification directly. The original C is
the discovery input; generated Rust is a controlled measurement of what each
representation costs. The complete early Rust control is also retained so that
future comparisons can start after mechanical noise has been removed. Neither
requires replacing maintained Rust or committing to a compiler fork.

### Prevent RAM-address expansion at the C boundary

The original C expresses RAM access through named macros. Preprocessing erases
those names and exposes a pointer construction at every use. For example:

```c
#define samus_x_pos (*(uint16*)(g_ram+0xAF6))
// Experimental header replacement:
#define samus_x_pos (*RamWordAt(0xAF6))
```

Two ordinary shared C functions form byte/word pointers into the same `g_ram`;
dereferences, indexing and stores stay at the callers. The experimental header
changes 1,100 pointer constructions and three existing getters. All gameplay
`.c` bodies stay unchanged. Actual C2Rust translation gives:

| Complete measured source, including helper and root costs | Original Rust | C-boundary candidate |
| --- | ---: | ---: |
| Samus unit sm_90 pilot | 17,872 | 13,453 |
| All 32 gameplay units | **221,547** | **188,606** |

That prevents **32,941 lines (14.9%)** of generated address machinery across
the measured gameplay scope. Every unit decreases. The result includes the
25-line generated helper once, per-unit declarations and root registration.
It does not apply numeric cleanup or change gameplay algorithms. Source hashes,
options, translator version and output hashes guard reuse of earlier original
translations; the C2Rust service reports 0.22.1 but its build is not commit-pinned.

All 65 original/replacement C object builds, 65 generated Rust units and both
complete module roots compile; no constant-initializer exceptions were needed.
Independent source inversion restores all 1,100 macro constructions and three
getters; all 32 gameplay C files are byte-identical. Actual original/prepared C
and original/prepared generated Rust pass **12,032 full-RAM/return comparisons**:
9,984 cases across 39 real routines, plus 2,048 aliasing, odd-address, indexing,
address-of and single-evaluation cases. The gameplay cases use constructed RAM,
enemy index zero and synthetic zero ROM, without stubbing those routines. This
is component coverage, not a full-game/SNES replay or a runtime-speed claim.

Generic `RamWordAt` still emits an offset, so this experiment prevents address
expansion but does **not** recover the name
`samus_x_pos` in Rust. The named original C remains the better discovery view.
Artifacts: TEMP/sm-compact-source-first/ram-emission, including the exact header
patch, immutable candidate, emitted units and complete measurements.

### A complete compact Rust control for subsequent experiments

Composing the already-tested numeric, shared-declaration, native-call and
RAM/constant-ROM extraction policies produces a complete source tree:

| Complete original Rust project | Lines |
| --- | ---: |
| First transpilation | 278,936 |
| Numeric, shared-declaration and native-call policies | 219,709 |
| Plus shared RAM/constant-ROM expressions | **195,699** |

This removes **83,237 lines (29.8%)** of generated representation overhead
without changing gameplay algorithms. The final 56-file count includes
hardware/frontend source, all helpers, imports and registration, using the
same normal formatter. It is an actual composed count, not a sum of independent
estimates. The C-boundary experiment above overlaps this address extraction
and has a different scope; do not add its 32,941 lines.

The complete optimized library compiles and the frontend passes metadata
checking against it. Inverse expansion of **24,548 helper calls** recovers the
preceding source with zero token-differing files, ignoring formatting and
optional trailing commas. The 22-line helper is identical to the implementation
already checked by 17,301,508 pointer comparisons; those checks are reused.
These establish mechanical correspondence and compilation, not full-game
execution. The ROM matcher only adapts equivalent numeric literal spelling;
unmatched expression forms are retained.

Retained source: TEMP/sm-compact-source-first/compact-original/sm/src. Its
REPORT.md records exact commands and limits; ../EXAMPLES.md contains unedited
three-way extractions. `Samus_JumpingMovement` is 32 lines in C, 90 in raw Rust
and 56 in this compact Rust. C still preserves the meaningful RAM/table names
that both generated views lose. Use it for finding whole repeated operations;
use the compiled compact Rust to inspect and price their generated forms.

### A fair structural comparison after removing numeric noise

The earlier 229,348 -> 185,133 result combined two effects. Applying the same
existing numeric pass to the untouched C output now supplies the missing control:

| All 32 gameplay units, CPU infrastructure, SPC player and roots | Rust lines |
| --- | ---: |
| Original C, generated Rust under the common formatter | 229,348 |
| Original C, numeric emission normalized | **191,082** |
| Restructured C, identical normalization | **185,133** |

The C restructuring therefore removes **5,949 lines** with numeric noise
controlled. That is the measured structural benefit; the earlier 7,227 raw
output reduction also included differences in how much noise was generated.
The new comparison charges the replacement arithmetic helper and registration.

Numeric normalization preserves all **6,371 function names/signatures**, 2,191
extern declarations, 541 structs, 466 aliases, 12 enums and the measured counts
of conditionals, matches and loops. Cast nodes fall from 221,347 to 125,965.
The subsequent C restructuring removes **861 function definitions and 255
extern declarations net**, including the handler/wrapper consolidation already
documented above. These counts separate eliminated representation from deleted
entry points; the represented game operations remain.

The new compact original passes optimized library compilation and formatting.
The unchanged numeric policy's arithmetic/constant checks and the previously
validated restructured candidate are reused. This adds no gameplay execution
coverage. Exact paired per-file counts and three extracted C/raw-Rust/compact-
Rust examples are in TEMP/sm-compact-source-first/numeric-control/{REPORT.md,
PER_FILE.md,SNIPPETS.md,RESULTS.json}. The original compact source is under
numeric-control/compact-before/sm/src.

## Implemented game patterns and deliberately written Rust

The next round uses the original named C to implement repeated game behavior,
while separate experiments compare C, automatic Rust and a deliberately written
Rust equivalent. The user released the machine and agent capacity for this
work. The candidates are code replacements with compiled controls and component
checks; source discovery is not limited to the previous clone detector's shapes.

### Complete game-family replacements

The first ten selected experiments compose into **79,473 -> 76,900 Rust lines
(-2,573)** across 15 complete affected gameplay units, the shared RAM helper
and both module roots. Both sides receive the identical RAM preparation and
numeric policy. The corresponding raw emitted pair is **94,529 -> 91,374
(-3,155)**. These are two measurements of the same candidate, not additive
savings. This control excludes the previously measured emission improvements.

The composed responsibilities are:

| Responsibility | Implemented replacement |
| --- | --- |
| Samus control | All 28 movement and 28 input modes, five airborne forms and shared pose-commit stages |
| Player firing | Beam, missile, bomb and special-beam construction, slot allocation and record initialization |
| File management menus | Complete copy/clear phases and shared three-slot rendering, including different cancellation and SRAM effects |
| Room/block behavior | 74 door-scroll callbacks become ordered recipes; eight breakable-block variants share setup |
| Space Pirates | Direction-aware wall, somersault, lunge, recovery and patrol algorithms |
| Beetom | Shared walking, jumping, attachment and recoil lifecycle with explicit slot-capture differences |
| Boss phases | Draygon's 16 paired attack phases and six directional Mother Brain beam sweeps |
| Ridley tail | Ordered seven-joint dynamics, position accumulation, hit tests and mode transitions |
| Enemy projectiles | Shared integration, fan/linear movement, origin handling and retirement paths |
| Room collision | Common span, cell and strip traversal serving Samus, player projectiles and enemies |

The generated cohort has **3,128 -> 2,910 function definitions**: 290 old
names retire and 72 new names are introduced. Numeric dispatch identities
remain represented. The pure C patch removes **2,107 physical `.c` lines**
and **136 header lines**. Applying the projectile study's blank-line rule to
both C sides attributes **2,096** to the replacement and 11 to pre-existing
extra blank lines. All Rust prices already use one formatter. Headers remain
separate from the user's original-C size ceiling.

All 72 original/replacement C object builds, 64 raw/normalized Rust unit checks,
four metadata roots and four optimized roots pass. Component checks exercise
the actual selected routines, including persistent projectile and menu sequences,
byte aliases and mutation of the live enemy/projectile index by dependencies.
Some boundaries use traced stubs or synthetic ROM. These checks do not establish
full-game equivalence of the integrated candidate. The collision checks found
and corrected a real draft missile-return bug. Their final 106,496 cases use
valid mapped coordinates; an earlier off-map synthetic matrix is superseded.

The matched controls also reject plausible designs: Rio flight sharing saves
nine raw Rust lines but **adds 43** after numeric normalization; the alternative
special-beam recipe adds **28** normalized lines. They are retained as negative
experiments and excluded from the selected patch. The earlier door-clone study
overlaps 36 of the 74 callbacks, so this round cannot be added to that result.
The full ordered-door alternative also covers 38 previously unshared entries.

Frozen first aggregate: TEMP/sm-c-pattern-experiments/integration/combined,
including REPORT.md, RESULTS.json, algorithms.patch, complete source snapshots,
source hashes and per-family validation references. Individual candidate code
and reports are under samus, player-fire, presentation, blocks, pirates,
enemies, boss-phases, boss-motion, projectiles and collision in the same parent.

Two completed extensions then add Yellow Pipe Bug's complete lifecycle and
reuse the same room-collision algorithm for enemy projectiles. The latter
costs 11 more normalized lines than a standalone duplicated loop, but gives
one tested traversal shared by all four consumer classes. The final pure-C
cohort contains 16 gameplay units plus the RAM helper and root:

| Identically prepared/normalized complete cohort | Rust lines |
| --- | ---: |
| Original C | **82,234** |
| Selected C algorithm replacements | 79,432 |
| Same replacements, with deliberate native Rust door sequences | **79,218** |

The pure-C replacement saves **2,802** Rust lines. The mixed C/native-Rust
candidate saves **3,016**, including **214** beyond the pure-C output. Its
raw formatted pair is **97,689 -> 94,044 (-3,645)**. The native table replaces
the generated door implementation in the measured tree; both versions are not
left counted or maintained. The pure-C patch removes 2,288 physical `.c` lines,
or **2,277 with the blank-line control**, plus 144 separate header lines.

All 72 C checks, 68 raw/normalized Rust unit checks, four metadata roots and
the complete optimized roots pass. The unchanged original optimized builds
are reused for the final mixed comparison. Yellow Pipe Bug adds 47,104
four-way component cases; the projectile collision extension checks both
standalone/shared forms against C and generated Rust. The exact native door
adaptation passes all 6,464 cases across eight implementations. Integrated
gameplay/SNES replay remains untested. Frozen final artifacts are under
integration/combined-extended (C patch) and integration/combined-mixed
(complete mixed Rust roots, counts and deliberate-door-rust.patch).

### Test the Rust representation of the replacement itself

The door experiment exposes a concrete C/Rust representation choice. A packed
64-bit scroll recipe produces 2,196 lines for the full door bank; explicit C
rows produce 2,332. A deliberately written Rust version instead returns one
of 63 borrowed `(index, color)` slices and executes one ordered loop. Rust's
borrowed constant slices avoid the separate C table-index/count plumbing.

That complete bank is **1,980 lines**, versus **2,837** original generated
lines: **857 removed**, and **216 fewer than the packed C2Rust candidate**.
Combined with the selected block setup in that standalone comparison, it is
**11,362 -> 10,326 (-1,036)**. This is a native Rust replacement, not a claim
that C2Rust emitted it. All 74 numeric cases, nine other dispatcher branches
and 65 retained function bodies remain; 6,464 full-RAM/return cases pass across
seven C/Rust implementations without stubs. The complete optimized root builds.
These counts overlap the aggregate's door change; they are not extra savings
until composed under its identical controls. Source: blocks/native-door.patch
and the accompanying REPORT.md.

### Measure reasonable translation on complete routines

The broadest completed substitution in this round targets ordinary record
construction and named integer constants. On the **already compact complete
original Rust**, the actual composed result is **195,699 -> 192,013 (-3,686)**:

| Applied substitution | Complete Rust lines | Incremental removal |
| --- | ---: | ---: |
| Existing compact original control | 195,699 | - |
| Typed constructors for repeated C-origin record literals | 193,702 | 1,997 |
| Plus proved named-constant cast composition | **192,013** | **1,689** |

The constructor pass replaces **759 literals across 11 record types**, including
SPC transfer maps, DMA/HDMA and PLM arguments, points and collision results.
Every constructor, declaration and caller is counted. Layout, field expressions,
their evaluation order and the consumers remain unchanged; inverse expansion
recovers all original record literals with zero token differences. The actual
SPC copy-to/from-RAM routines pass 256 cases with 659 ordered transfers each.
This is a translation experiment; current main already uses constructors for
some of these types.

The second pass handles **3,026 integer cast sites** using actual module-local
types/values, preserving low bits or proving that an intermediate conversion
cannot truncate. Ambiguous symbols are rejected; a global map of repeated
generated typedef names was insufficient and was corrected before final checks.
It changes no constant declarations or ABI. All 986,066 executed comparisons,
including every actual selected constant and all 92 widening sites, pass.
The complete composed optimized crate compiles. Neither count includes the
older numeric/declaration/RAM savings or the new C algorithm aggregate.

The new Draygon profile supplies a concrete threshold control: its complete
normalized bank/helper/root is **3,088**, named-constant lowering gives **3,036**,
and an ordinary native enum/immutable profile gives **3,034**. Forcing a generic
constructor onto its two rows then grows it to **3,052**. Emit constructors
where their full repeated cohort repays the helper cost, rather than applying
one representation to every table. The four variants agree on all descriptor
words, direction/phase lookups, discriminants and layouts, and compile optimized.
Artifacts: TEMP/sm-c-pattern-experiments/smart-data.

Two independent experiments retain exact C/automatic/normalized/handwritten
examples and charge their complete support code:

| Complete measured unit | Automatic Rust | Compact control | Deliberately written Rust |
| --- | ---: | ---: | ---: |
| Twelve named-memory gameplay routines, including address map/API/root | 348 | 250 using generic RAM helpers | **213** |
| RNG plus three projectile-velocity routines, including helpers | 131 | 108 using numeric normalization | **63** |

The memory experiment preserves named C lvalues through translation markers,
then emits byte-compatible loads/stores without references into RAM or a getter
per field. Its full-bank automatic result is **14,583**, versus **13,453** with
generic pointer helpers: restoring names alone increases this comparison by
1,130 lines. The handwritten 12-routine target saves 37 beyond the generic
control. This separates a desirable naming property from an actually compact
implementation. Six representations pass 110,592 comparisons against original
C, plus pointer-offset, overlapping-write and evaluation-order checks. The
complete bank compiles; full-bank runtime coverage is not claimed.

The arithmetic family has 39 lines of original C routines, or 53 including
its C support. Native Rust recovers byte extraction and 8.8 velocity advancement,
preserving the fraction store before the position read. A reusable pass applies
114 proved local-byte substitutions, rejecting escaping addresses, effectful
right-hand sides and unknown types. On the complete compact snapshot it saves
47 lines beyond the already-studied constant-index control. Integrating the
four native routines adds 28, for **75 genuinely additional lines**, not the
large apparent difference from raw output. Optimized full-crate compilation,
all 65,536 RNG seeds, full-RAM cases and alias/ordering checks pass.

These prototypes identify exact emitter decisions and implement substitutes;
their measured scale determines whether a category deserves broader work.
Retained reports and full examples: TEMP/sm-c-pattern-experiments/smart-memory
and smart-arithmetic, each with REPORT.md and EXAMPLES.md. They do not require
a maintained compiler fork or replacing the current implementation wholesale.

The control-flow comparison supplies a useful negative control. Five complete
Samus routines and support occupy **395 fresh C2Rust lines, 283 compact lines,
and 247 with three deliberate Rust replacements**. A bounded executable
value-join rule recovers six of those 36 lines; across the complete bank it
removes **12 lines (17,871 -> 17,859)**. Fresh C2Rust already turns the examined
forward gotos into labeled blocks, so a separate marker-to-block pass yields
no additional compact saving. Do not attribute that compiler improvement to
the experimental postpass. All 4,954 constructed cases pass 39,632 full-RAM
comparisons across C and eight Rust forms, including ordered dependency effects.
Two other bank joins have static/compiler coverage only. Full examples,
executable rules and results: TEMP/sm-c-pattern-experiments/smart-control.

## Tailored emission learned from current Rust

The latest experiment treats current **38c65eba** as a source of target
representations. Each rule traces original C (578f90b), initial generated
Rust (1311d3b4) and current Rust, then emits the compact representation from
the old inputs. Reproducing current sharing automatically is the intended
result; this does not require rediscovering an algorithm or beating current
main. The prototypes remain under TEMP/sm-tailored-from-current.

### Generate an existing shared algorithm from the original effects

The strongest complete example regenerates all **24 corpse tile move/copy
callbacks** as calls to current `transfer_corpse_tiles`. The rule derives
tile bases, row gates, limits, indexing modes and move/copy flags from the
original C's ordered guards and memory effects. It has no lookup table of
member names or descriptors. The existing native helper is reused and its
complete 53 lines are charged to the output.

| Complete family, including all named callbacks and numeric routing | Lines |
| --- | ---: |
| Original C | 793 |
| Initial generated Rust | 2,319 |
| Automatically emitted ordinary Rust, including shared helper | **208** |
| Current Rust expanded to the same ordinary representation | **208** |

The two 208-line forms are byte-identical. The recognizer checks 6,053 original
functions, accepts the complete 24-member family, and derives all 21 members
outside its three grammar examples correctly. Renaming/parameter changes
pass; reordered writes, cached progress, different geometry, extra effects
and unsupported declarations are rejected. This is retrospective transfer
within a known family, not blind discovery of new domain abstractions.

All **126,720 original-C cases** pass **506,880 full-RAM comparisons** against
four Rust forms, including overlapping source/destination and aliases with
the live progress word. The full current crate containing the generated
callbacks compiles optimized. No external gameplay callees are stubbed in
this family; no whole-game/SNES replay is claimed. The original Mother Brain
typed accesses are exercised with aligned indices. Callback-macro line-wrap
differences are excluded by comparing ordinary expanded functions. Shared
Game/storage/layout infrastructure remains common, outside family counts.

The usable translation boundary is concrete: specify a current helper's
effect contract, recognize exactly that contract in C, derive parameters and
emit named calls directly. Domain selection remains a human decision;
unrecognized bodies remain available for ordinary translation. Sources,
standalone `emit.py`, tests and complete comparison: algorithms/REPORT.md.

### Reproduce the current memory interface

A separate source-informed rule preserves C RAM names and emits current
`word!`, `ram_ptr!` and `words!` access using the actual current `Game`.
Twelve complete routines occupy **319 raw-with-Game lines versus 176 emitted
lines**; adding both module-registration lines equally gives **321 -> 178**.
Imports are included, with zero new helper definitions or map rows. Existing
Game, RAM and runtime support is held identical in both full-core snapshots.

All **18,432 original-C states** agree with both raw-Game and emitted-Game
Rust on complete RAM, returns and dependency counts. Current-reference checks
cover 16,896 cases because `ClearBossBitForCurArea` has been removed from main.
Current boss-bit operations also write scratch byte 0x5e7, an explicit oracle
repair in 4cf25663 absent from upstream C. The 3,072 affected comparisons model
that specific repair; it is not smuggled into a generic translation rule or
hidden by a RAM mask. The comparable 11 current routines occupy 114 body lines
versus 156 emitted, exposing further arithmetic/control simplification to
learn. Full lineage and the executable emitter: memory/LINEAGE.md and REPORT.md.

### Avoid expanding auxiliary C metadata

The historical `ram_map.rs` table was generated from C headers after
transpilation; it is not C2Rust gameplay output. A new generator validates
every row against the original C definition and emits the current `ram_vars!`
representation: **7,042 -> 1,231 lines (-5,811)**, including the whole macro
and its named scalar/array maps. An optimized DLL compares all **1,169 rows**
in address, name, type, shape and sequence. The repeated alias is retained;
four malformed/mismatched inputs are rejected. This reproduces a current
metadata improvement and is accounted separately from game-function emission.
Source, reproducible generator and checks: ram-metadata/REPORT.md.

### Emit current shared declarations and structured control

The current `typed_struct!`, `typed_constants!` and `set_fields!` forms supply
another executable translation template. On the complete compact original
control, the pass emits 228 C-matched structures (6,285 fields), 5,373 named
C constants in 299 groups, and 590 ordered literal stores in 234 groups:
**195,699 -> 189,854 (-5,845)**. All macro definitions, owner modules,
compatibility reexports and registration are counted. Complete inverse
expansion recovers the input tokens; all structure sizes/alignments, 6,285
field offsets and 5,373 constant values are checked in compiled Rust. The
complete optimized root builds.

Applied after the earlier constructor/constant-cast pass, the actual composed
output is **186,168**, or **9,531 below 195,699**. Both independent passes
happen to retain their full measured savings in this composition; this is
an executed combined tree and optimized build, not a sum of detector hits.
The initial raw control separately gives 278,936 -> 267,217 from the current
data templates. Its cast-heavy stores become eligible only after numeric
lowering, demonstrating why pass order matters. Original named-constant
duplicates are already shared in the compact control; merely seeing local
enum aliases does not establish another ownership saving. Source and inverse
passes: data-types.

The control-flow rule similarly learns actual current transformations:
two-state generated dispatch variables become Boolean decisions; eligible
counter loops become typed ranges. The complete historical root gives
**278,936 -> 277,585 (-1,351)** across 113 decision variables, 74 terminal-tested
loops and 93 reused while-loop recoveries. All 106 previously identified
current decision variables map to the same original label pairs/polarities.
The full optimized crate and 4,348 compiled decision-trace comparisons pass;
628 compiled terminal-loop comparisons check 4,639 sequence values and all
74 exit values. These validate control/effect order, not complete gameplay.

Fresh C2Rust already avoids the tested historical `current_block` scaffolding.
The fresh complete Samus bank instead selects six terminal-tested and eight
while loops: **17,871 -> 17,814 (-57)**. The pass rejects counters changed by
the body or observed beyond the accepted scope. In particular, animated tiles'
handler reloads its index after a callback and remains a live-index loop.
Do not price obsolete compiler scaffolding as a fresh-translator defect.
Exact C/initial/current/emitted examples and executable rules: control.

The current primitive-type spelling is a material emission rule in its own
right. Resolving fixed-width typedef chains and `core::ffi::c_int/c_uint`
type nodes gives **195,699 -> 183,397 (-12,302)** on the complete compact
original. The pass makes 100,712 proved type substitutions across all 56 files,
preserving domain aliases, compatibility declarations, imports and ABI.
Generic/local shadowing, unresolved providers and unsupported native types
are excluded. Complete formatted inverse-token checks and optimized compilation
pass, as do 32 compiled type-identity witnesses and scoped negative controls.
It adds no helper code. Run this before macro grouping hides ordinary type
nodes. Source and executable CLI: primitive-types/lower_primitive_types.py.

The current callback representations also admit a generic emitter. On a
separate historical native-Rust cohort it produces **140,334 -> 138,342
(-1,992)**, including 120 lines of shared support and registration.
It handles 118 dispatchers, 1,904 callback bodies and 95 tables with 1,039
entries, preserving 647 external dispatch arms. Both complete optimized roots
compile. Compiler-expanded comparison retains the same 7,310 qualified
functions/signatures; 7,192 ordinary bodies match tokens, and the other 118
dispatchers match selector, setup, default and complete address/target/ordered
argument maps. All 7,231 expanded static/constant items also agree. These are
structural correspondence checks, not execution of every gameplay callback.
The separate complete animated-tiles runtime pilot exercises the composed
emitter. Its support is shared with current core; the broad cohort's savings
must not be added to another baseline without running the pass there. The
complete numeric map contains 2,609 routes; 672 are retained across the 647
external arms. Report and reusable `lower_tree.py`: dispatch/REPORT.md.

### Compile the rules together in a complete current-engine unit

The animated-tiles pilot composes source-named RAM access, native Game/call
adaptation, current constant ownership, two safe range recoveries and callback
emission over the complete original unit. It automatically derives Game
parameters for all 34 functions, 70 direct calls and the 13 external owners;
no current function bodies are copied. Unknown ownership, external static
state, function-pointer escapes, existing Game bindings and ambiguous alias
occurrences are rejected. This is a bounded complete-unit adapter, not a
general whole-program ownership converter.

The original C file has **273 lines**; its exact initial Git Rust file has
**671**, fresh C2Rust emits **723**, and the final tailored module has **296**.
The earlier 679-line independent-transpiler fixture is a different input and
is not used for this lineage. Source-naming markers temporarily produce
778 lines before lowering; these transport declarations are removed from
the final output. The 296-line candidate compiles with actual current Game,
RAM helpers, constant owners and callback macros, which remain counted in
the complete current source snapshot.

Independent runtime checks compare original C with the frozen final module:
**5,004 cases**, all 24 callbacks, 114 persistent sequences and 20 deliberate
live-index mutations pass. Comparisons include all RAM, returns and 3,958
ordered dependency entry/exit RAM snapshots. Twelve external dependencies
use disclosed traced contracts; actual ROM, those callees' complete behavior
and full-game replay are outside this coverage. All 77 supporting current
Rust files are byte-identical in the runtime fixture. The boss-bit scratch
repair is outside this unit and is not silently included in the C contract.

The final current-core candidate also contains the independently generated
corpse callbacks and compiles optimized. Its physical count is 107,520 ->
107,545: animated tiles adds 72 over the manually compact current module,
while corpse macro formatting removes 47. The latter is explicitly excluded
from algorithm savings; both ordinary corpse forms are 208 lines. This is
successful automatic recovery toward current output, not a selected main
size-reduction patch. Reports, frozen code and reproduction commands are in
integration and animated-tiles-validation.

### Measure the actual whole-original composition

The sequential pipeline runs all compatible broad passes on the same complete
compact-original control. Both endpoints use the same formatter and compile
as optimized complete Rust libraries. The result is **195,699 -> 162,136
lines (-33,563, 17.2%)**, across 56 input and 59 output Rust files. All shared
owners, macros, imports, aliases and registrations remain in `sm/src` and are
counted; no original source file is dropped.

| Applied stage in this exact order | Complete lines | Incremental removal |
| --- | ---: | ---: |
| Existing compact original | 195,699 | - |
| Proved native primitive types | 183,397 | 12,302 |
| Earlier constructors and named-constant casts | 181,260 | 2,137 |
| Current-derived structured control | 179,961 | 1,299 |
| Current shared data/type representation | 174,104 | 5,857 |
| Current C-derived RAM metadata representation | 168,293 | 5,811 |
| Shared callbacks, dispatchers and tables | **162,136** | **6,157** |

These are measured sequential outputs, not summed standalone results. For
example, the earlier constructor/cast pass saves 2,137 here rather than its
standalone 3,686; primitive lowering already handles some of that cost. The
callback pass saves more on this earlier input than on the separate historical
native cohort. The 5,811 metadata lines are auxiliary C-derived metadata,
explicitly included in the total and separate from gameplay emission. The
corpse-kernel and current-Game animated-tiles pilots use different integration
boundaries and are not added to this whole-original total.

Independent compiler-expanded verification of the actual final dispatch stage
preserves all **7,205 qualified function signatures**, 7,087 ordinary function
bodies, all 118 dispatcher selectors/setup/defaults/ordered route maps, and
8,687 static/constant items. This actual cohort covers 2,031 callback bodies,
2,715 numeric routes and 126 tables with 1,490 slots. Earlier stages retain
their type-identity, inverse-expansion, layout and control-order checks.
The complete composition has optimized compilation and these applicable
structural/component proofs; no whole-game runtime replay is claimed.

With the earlier 278,936 -> 195,699 preparation, the actual complete original
output is now 162,136 rather than 278,936. This is a repeatable partial profile,
not a claim to reproduce all current engine architecture or generate a whole
game below 100,000 lines. The useful direction is to extend current-helper
recognition and preserve source semantics before macro expansion, with ordinary
translation retained for unsupported forms. No maintained compiler fork is
needed. Frozen rules, all stage outputs, hashes, compiler results and the
reproducer are under integration/whole-original and integration/compose_whole.py.

### Extend the complete profile using current engine representations

The next implemented composition starts from the exact **162,136-line**
output above. Its complete new composition produces **125,802 lines (-36,334)**;
the complete optimized library and actual separate frontend entrypoint both
compile. This is additional reduction of the early generated implementation,
not 36,334 lines removable from the maintained 107,520-line engine.

The original input remains 278,936 Rust lines. The original Makefile's C
implementation remains 100,374 lines, including 83,657 gameplay lines, with
headers counted separately. Neither denominator changes for this experiment.
No under-100,000 result or whole-game runtime equivalence is established yet.
The cumulative generated-source reduction is **153,134 lines (54.9%)**.

| Sequential emission policy | Complete Rust lines | Incremental removal |
| --- | ---: | ---: |
| Previous complete profile | 162,136 | - |
| Numeric cleanup after primitive resolution | 152,687 | 9,449 |
| Literal array emission | 150,977 | 1,710 |
| Immutable C string tables | 150,407 | 570 |
| Unreferenced original-C function addresses | 148,214 | 2,193 |
| Unreferenced native functions | 147,112 | 1,102 |
| Ordered word-transfer kernels | 146,383 | 729 |
| Current RAM/ROM access representation | 143,637 | 2,746 |
| Source-recognized corpse-tile kernel | 143,104 | 533 |
| External GL bindings with original loader behavior | 131,428 | 11,676 |
| Macro registration repair | 131,428 | 0 |
| Source-specialized PNG kernel | 128,669 | 2,759 |
| Unreferenced auxiliary declarations | 127,973 | 696 |
| Canonical backend types | 126,418 | 1,555 |
| Shared forwarding/no-op declarations | 126,083 | 335 |
| Source-recognized tile-flip writers | **125,802** | **281** |

Each row uses the preceding actual output and the same formatter. All local
helpers, adapters, declarations and module registrations remain counted. The
literal-array pass saves 1,710 here, versus 5,758 alone: most of its standalone
gain overlaps numeric cleanup. The corpse kernel saves 533 here, versus 764
on the prior complete profile and 2,111 in its earlier isolated raw family.
Those larger overlapping measurements are not added. String-table emission
also contains macro representation compression; its standalone expansion
accounts for 615 lines, with 45 ordinary support lines. It is not removal of
game algorithms.

The final snapshot includes 62 Rust files and a nine-line dependency manifest.
This round's invoked Python rules, validators and orchestration total 3,911
lines separately; they are research tools, not gameplay code omitted
from the Rust count. All emitted game algorithms and shared support are counted.

The useful compiler lesson is ordering and visibility. Primitive resolution
exposed 35,495 remaining integer-literal casts. Callback expansion exposed
2,031 bodies previously hidden from syntax passes and compiler diagnostics.
The numeric pass applies only its proved rules and machine-applicable compiler
suggestions, then regenerates shared callbacks. All 3,407 distinct literal
assertions pass; complete expanded correspondence retains 7,205 functions,
including 118 dispatcher maps. The separate binary lacked two macro providers;
their one-line repair is charged to the numeric stage.

The memory policy recovers current caller-side named reads, writes, updates,
split-word operations, ROM reads and typed pointer declarations while keeping
the older global storage model. Its actual composed output checks 867 map rows
against original C, every one of 150 getter declarations and 49 ROM wrappers,
19,904 offset-cast values, and over 13 million executed getter/wrapper address
comparisons. It preserves unaligned access and evaluation order. Current main
already implements this supported policy: its measured remaining match count
is zero.

The algorithm policies recognize ordered C effects and emit existing ordinary
Rust kernels. All 150 actual composed transfer regions pass 9,600 C/pre-pass/
replacement cases, comparing full RAM and retained counters. The composed
corpse family exactly matches the 24-member replacement tested over 126,720
original-C cases, including the independently checked progress-field offset.
These are constructed component checks, not game-route execution.

Backend type sharing replaces 94 repeated declarations with eight canonical
C layouts. Actual composed-library checks pass for 2,018 fields, 194 original-C
header values and 6,054 raw-byte probes; all other source tokens are retained
across the 62-file stage. The actual forwarding stage preserves all 6,863
function signatures, 364 forwarding/no-op effect identities, 6,381 other
bodies, 118 complete dispatch maps and 4,350 storage items. The four tile-flip
writers emit the existing tile kernel and pass
540,672 C/pre-pass/replacement component cases, including overlap and boundary
counters. Their actual combined marginals appear in the table above; ordinary
kernel support and the forwarding macro are included.

The GL policy replaces locally generated bindings with published `gl` 0.14.0.
Its counted adapter retains original extension-dependent lookup order, version
caching and missing-symbol behavior. All 1,310 loader traces, 423 calls across
47 used procedures, and 121 unchanged caller argument lists pass against
scripted SDL/driver contracts. The external dependency generates **21,338
binding lines**, outside maintained project source; this is dependency reuse,
not a claim that total third-party build source shrinks. No real GPU test or
linked SDL executable is claimed.

The PNG policy recognizes the pinned public stb implementation, derives its
parameters, emits the existing compact PNG algorithm template, and extracts
all 18 legacy inflater functions from the actual input. It preserves the
original loader's malformed-input and C stream behavior. The tested kernel
and inflater are byte-identical in the composed result: 427 fixtures pass 854
decoder/loader comparisons, 427 actual texture-consumer comparisons and 3,702
ordered C file-call comparisons. A general PNG crate was not assumed equivalent.

The symbol passes retain exports, attributes, ambiguous names, dispatch routes
and referenced tables. Public native declarations must also be absent from the
frozen current source contract. Actual-input inverse token checks and guard
cases verify the omissions; these passes do not promise an API for unknown
external consumers.

Complete integration caught a real composition defect: a newly inserted module
split `#[macro_export]` from its macro. The module is now registered after the
complete item, with compiled attribute guards; the zero-line repair is explicit
above. Earlier failed output is retained and excluded from accepted compilation
claims. This is why isolated successful passes are insufficient.

Frozen passes, complete outputs, component checks and reproduction commands
are under `TEMP/sm-tailored-next/`, with the accepted composition in
`integration/phase4/RESULTS.json`. This checkpoint is retained unchanged;
the subsequent composition below includes the earlier C algorithm families
and alpha-equivalent enemy layouts. Current-source
numeric, comparison, entity-access and local-initialization experiments remove
only 31, 69, 152 and 13 lines respectively on their separate inputs. They have
compiled component checks but are parked as volume priorities, rather than
presented as a substantial deletion budget. The maintained source remains
unchanged, including the unrelated shader draft.

A signed-word delta helper also saves only 51 current lines across 491 typed
sites, with full compilation and source-expression correspondence. An eight-bit
wrapping subtraction was correctly excluded. This is another parked option;
its native algebra executable was blocked by Windows, so no executed Rust
algebra claim is made.

A separate current-source architecture experiment moves Game-dependent
functions into inherent associated functions, preserving argument order and
function-pointer signatures. It **increases** the complete source by 1,011
lines for 2,644 eligible functions; limiting it to 408 cross-module functions
still adds 97. Both full roots compile, all 17,058 ordinary calls retain their
argument/target structure, and callback/order controls pass. Reject associated
ownership as a volume reduction. Receiver methods and reorganization into
domain modules remain untested by this trial.

### Follow current Rust back to reusable emission rules

The next actual composition reaches **113,161 lines in 64 Rust files**,
removing another **12,641** from the frozen 125,802-line control. It uses the
current implementation to identify compact operations, then recognizes their
original C/generated-Rust forms and derives replacements. It does not copy
complete current gameplay bodies into a supposedly automatic result.

| Sequential stage | Complete Rust lines | Incremental removal |
| --- | ---: | ---: |
| Previous frozen profile | 125,802 | - |
| Selected original-C algorithm families | 124,291 | 1,511 |
| Shared enemy record layouts | 122,887 | 1,404 |
| CPU opcode grid | 122,095 | 792 |
| Planar background kernels | 121,689 | 406 |
| Configuration initialization | 121,216 | 473 |
| Shared decompression | 121,134 | 82 |
| Remaining ordinary dispatchers | 119,905 | 1,229 |
| Borrowed byte-array arguments | 119,496 | 409 |
| SPC opcode grid | 119,140 | 356 |
| Projectile trails | 119,042 | 98 |
| Scrolling tilemaps and DMA | 118,908 | 134 |
| Sound command/process family | 118,774 | 134 |
| Ordered register writes | 118,237 | 537 |
| Byte lanes and split-word field operations | 116,791 | 1,446 |
| Original packed VRAM record repair | 116,791 | 0 |
| Map rendering, labels and equipment navigation | 116,360 | 431 |
| Power Bomb / Crystal Flash window family | 115,980 | 380 |
| Reset and fixed-array initialization | 115,347 | 633 |
| Scalar initialization at the first defining assignment | 114,563 | 784 |
| Ordered enemy instruction/timer writes | 114,403 | 160 |
| Native assertion emission | 113,901 | 502 |
| Consecutive same-value RAM stores | **113,161** | **740** |

The register stage's standalone 570 and byte-field stage's standalone 1,565
are not added: their actual marginal savings are 537 and 1,446 after earlier
families have changed the source. The C-family stage also rejects two families
that increase volume. Its 1,511 includes downstream type/forwarding/tile overlap.
Later scalar initialization saves 784 here versus 841 alone, and enemy write
pairs save 160 versus 235 alone. The latest seven-stage batch removes 3,630.

A concrete translation rule is C's `HIBYTE(field)`: generated Rust expands
this into a raw-pointer cast, two `size_of` calls, division, subtraction and
an offset. Current Rust describes the same caller-side place with `byte!`.
The actual final stage rewrites 377 byte accesses and 134 split-word field
calls, retaining receiver rereads, pointer-before-value order and the original
low-before-high helper stores. An independent comparison of both complete
crates after rustc macro expansion passes; only redundant syntax and fixed
primitive lane constants are normalized. Twenty-one unsupported unstable
parser nodes are byte-identical and checked as opaque text. All 66,598 compiled
byte/value/alias/order cases pass. No gameplay algorithm is deleted by this rule.

The entire final library and real frontend entrypoint compile. Checks run
against each actual composition pair: full original-C enemy layout values,
compiled field offsets and ordered RAM writes; CPU/SPC state and bus traces;
PPU state and access order; C/old/new array, register, decompression and
algorithm comparisons. The final dispatcher cohort includes 750 numeric
routes and passes 27,684 boundary cases. Its first composed harness omitted
an earlier family's enum dependency; the corrected harness passes and the
failed log is retained. These checks remain bounded component evidence.

Map testing rediscovered the original C2Rust forward-declaration/pragma packing
defect already documented above: `VramWriteEntry` in the original-derived
control has destination offset 6, while C and maintained Rust require 5.
Actual map queue writes fail against C with that unchanged type on both sides
of the map refactor. The reduction rules did not introduce it. The control is
retained; a separate stage now applies the explicit zero-volume packing repair.
Its 65,536 complete-buffer stores and 8,192 actual original-C/Rust NMI consumer
cases pass, including 429,056 ordered events. The old layout deliberately fails
all store cases and 7,168 nonempty consumers. Auditing 98 packed C records finds
only two forward-before-definition cases; the other is the already-correct
byte-only OamEnt. Both complete corrected builds pass. This fixes a concrete
compiler defect; it supplies no reduction credit or whole-game proof.

Frozen outputs, stage-specific commands, rule hashes and checks are in
`TEMP/sm-tailored-next/integration/phase5` through `phase9-tail`; the final
corrected source is `phase9-tail/37-ram-store-groups`. The nine-line dependency manifest and the
external GL generator's separately disclosed 21,338 lines remain. Invoked local
Python rules, validators and orchestration add 3,820 unique lines across phases
5-7; phases 8-9 add 3,089, totaling 10,820 with the previous 3,911. This excludes
external compiler/parser
packages and owner-only fixtures not invoked by integration. These are separately
counted experiment tools, not emitted engine source. The maintained
107,520-line implementation and unrelated shader draft are unchanged. Continue
with the complete remaining policies; 113,161 is a partial
result, not completion of the under-100,000 goal.

The final batch passes full library/frontend compilation and all checks on
its actual preceding inputs. Map/menu sharing passes 12,862 comparisons using
the corrected real packed type, not a harness-only repair. The explosion-window
family passes 13,036 C/before/after cases and retains all 17 numeric routes;
its 38 rejection controls and outside-function/storage correspondence pass.
Its validator was versioned to handle rustfmt moving the new module declaration
alphabetically; emitted source did not change, and the earlier failed log remains.
Reset checks compare complete structure bytes, including padding and guards,
and ordered CPU/SPC reads. Scalar initialization has full-function scope/type
correspondence, primitive/drop/loop controls and an actual music-queue comparison.
Assertions compare actual expanded library and frontend syntax while retaining
diagnostic pointer identities. RAM grouping compares complete macro expansion
and adversarial full-RAM effects, including RHS-before-index order, aliases and
temporary destruction. These are component and syntax proofs, not full replay.

The earlier native instruction executors are now composed against the actual
113,161-line input. The combined enemy/PLM/projectile/presentation replacement
**adds 809 lines**, despite its old standalone reduction. Reject it. The
enemy-only alternative removes 217, with 466 opcode mappings, 407 bodies,
seven retained direct-call wrappers and 19,712 original-C/actual-Rust component
cases checked. Park it behind larger opportunities. The old 2,744-line
standalone saving is not missing budget. C word-operation recipes likewise
overlap prior numeric policies; their remaining countdown/comparison cohort
removes only 266 lines on its separate phase10 control. Integration must also
repair its inherited odd-address reference behavior before accepting it.

### Require less code, not only fewer lines

The next frozen historical-source composition measures the complete marginal
cost of the remaining current-guided families:

| Actual sequential stage | Complete Rust lines | Change |
| --- | ---: | ---: |
| Previous composition | 113,161 | — |
| Introduction page family | 112,970 | -191 |
| Moving platforms and shutters | 112,680 | -290 |
| Shared enemy getter prefixes | 112,446 | -234 |
| Control emission policy | 111,542 | -904 |
| Unaligned RAM update repair | 111,597 | +55 |
| Remaining original-C clone families | 111,311 | -286 |
| Safe countdown expressions | 111,146 | -165 |
| Typed word comparisons | 111,046 | -100 |
| Selected computed field writes | 110,953 | -93 |
| Computed named-RAM writes | 110,509 | -444 |
| Improved terminal-loop grammar | 110,509 | 0 |
| Ordinary standard-library imports | 110,009 | -500 |
| Packed DMA descriptor read repair | 110,009 | 0 |
| Named graphics-transfer records | 109,232 | -777 |
| Native tracing | 109,141 | -91 |
| Proven-nonnull callback tables | 109,069 | -72 |
| Corrected current beam-window owner | 109,010 | -59 |
| Shared eight-entry square-slope family | **108,758** | **-252** |

All actual stage checks and complete optimized library/frontend builds pass.
The getter cohort contains 47 groups / 391 functions after composition.
The clone cohort covers 18 groups / 84 routines, removes five unused wrappers
and redirects eight calls; 5,376 original-C/before/after cases pass. The 66-file
output and reports are under `integration/phase10` through `phase14`.

The alignment fix covers 228 RAM captures, including 89 odd literal-base
addresses. It replaces invalid `&mut u16` formation with raw pointers,
unaligned loads and ordered stores. The actual music queue now passes debug
execution at address 1599; 66,400 original-C cases and debug/release effect
checks pass. Its **55 lines, 4,230 tokens and 14,540 text bytes are a correctness
cost**, not a reduction. The earlier packing fix remains in the composition.
This historical candidate has no full-game replay result. The phase14 110,009
includes every safe word/store/import pass on its actual preceding input.
The complete composition removes 52,127 lines, 560,317 tokens and 1,516,811
code-text bytes from the 162,136-line frozen control, so its overall gain is
not merely formatting. The final import stage's paired complete builds pass;
optimized compiler-operation correspondence covers 1,109,529 library and
69,650 frontend lines. Reused successful build artifacts are identified in
`phase14/RESULTS.json`; compilation is not deferred or inferred from an earlier
source. Its strict gap was 10,010 lines. Local invoked Python experiment
rules/validators/orchestration total 14,700 lines across 163 files; the separate
nine-line Cargo manifest and external GL dependency costs remain disclosed.

The current-guided graphics rule then produces **110,009 -> 109,232 (-777)**,
including its 45-line owner, three caller imports and module registration.
It recognizes 59 complete original-C constant DMA sequences and their expanded
Rust forms, emitting `GraphicsAsset::VramWords`, `VramLowBytes`, `VramHighBytes`
or `WramBytes` loads. All descriptor fields and ordered destination/control
registers must agree with original C. Twelve noncanonical descriptors remain
explicit because their ordering or dynamic destination differs. Decompression
is outside this cohort. No descriptor address escapes its original setup call.

At that checkpoint the profile removes **52,904 lines, 564,355 tokens and 1,530,024
code-text bytes** from the same 162,136-line control. The isolated historical
gap was **9,233 lines** to reach 99,999. These figures come from a fresh complete
recount, including the packed-read correction, rather than summed pilot prices.
Current Rust already owns the equivalent graphics operation, so this rule
demonstrates recovering its representation without claiming a new main deletion.

All 59 actual original-C/repaired-old-Rust/emitted-Rust sites, 1,568 generic
parameter cases and 131,072 packed-descriptor cases pass in each debug and
optimized build. Observations include every ordered byte register write and
the DMA trigger after executing the actual setup routine. Whole-source inverse
correspondence reconstructs all unchanged source after replacing the selected
calls. Complete optimized library and actual frontend metadata builds pass.
There is no historical whole-game replay claim.

The checks reproduce an inherited misaligned-pointer abort in the original
Rust `SetupDmaTransfer`. Two packed word reads now use `read_unaligned`; this
separate repair costs **zero lines, six tokens and 28 code-text bytes** and is
included in the final profile. Its failure control and original source remain
preserved. Reports, source manifests and executable rules are in
`TEMP/sm-tailored-next/graphics-assets` and `integration/phase15`. Beam,
tracing and nonnull-table studies were outside that checkpoint.
Invoked local Python there totals 15,129 lines across 170 files, adding 429 lines
in seven scripts since phase14, including lineage discovery and parked beam
pricing. These experiment tools are separate from the 67 emitted Rust files;
the nine-line manifest and previously disclosed dependency costs remain.

The next actual composition adds four current-guided rules, producing
**109,232 -> 108,758 (-474)** with 4,979 fewer tokens and 14,742 fewer code-text
bytes. The complete 69-file profile is now **162,136 -> 108,758 (-53,378)**,
with **569,334 tokens** and **1,544,766 code-text bytes** removed. All shared
owners, imports, caller adaptations and registrations count. Complete optimized
library and actual frontend metadata builds pass after every stage. Its strict
historical-output gap is **8,759 lines**; it has no whole-game replay result.

Tracing adapts the current native formatter to the historical bus API and
passes 742,400 original-C/raw-Rust/historical/emitted component cases in debug
and release, including output tails, bus-read order and processor state changes.
Nonnull inference handles 106 private tables, 1,274 slots and 111 consumers,
retaining historical mutable-static and raw-offset semantics. Complete use
guards reject writes/escapes, and compiler-expanded correspondence proves the
same targets, casts, indices and remaining operations. Five nullable historical
tables remain unchanged. These two stages remove 91 and 72 lines respectively.

Beam-window emission removes 59 lines including adoption of the current
positive-down corrections. Its 64,100 component cases compare the actual
historical candidate and callers with the validated corrected current owner,
including all RAM, return/panic results and ordered print effects. This is
explicitly **not equivalence to uncorrected historical positive-down behavior**.

The new square-slope rule follows eight original entries across Samus, grapple,
projectiles and enemies into current shared occupancy/span/contact operations.
It emits a 141-line owner, retains callback names and argument order, and merges
two private tables only after proving their occupancy bits and complete uses
agree. The complete price is **252 lines removed**. All 49,920 actual original-C/
historical/emitted cases pass in each debug and optimized build, comparing return
values and every RAM byte, including collision contexts overlapping player/enemy
storage. All 69 files pass inverse source correspondence. No complete bank is
retired, and these historical savings are not new production deletions.

Frozen report, stages and reproduction commands:
`TEMP/sm-tailored-next/integration/phase16/REPORT.md`. Final source:
`TEMP/sm-tailored-next/square-slopes/composed-v2/sm/src`. Invoked local Python
totals 16,193 lines across 185 files, including 1,064 lines in 15 new files;
this experiment code remains separate from counted engine support. Cargo still
has nine lines and the external GL dependency costs remain disclosed.

The subsequent five-family composition is **108,758 -> 108,261 (-497)**,
removing 5,202 tokens and 11,487 code-text bytes. These are measured sequential
outputs; helpers, imports, registrations and repair costs are included:

| Complete emission family | Net lines removed | Executed component cases |
| --- | ---: | ---: |
| Four Samus jump-launch entries | 63 | 62,624 |
| Six Ridley/Shitroid homing entries | 130 | 420,928 |
| Fourteen DSP sample-limit expressions | 79 | 131,079 scalar + 22,784 routine |
| Solid-enemy query and temporary result type | 101 | 23,236 |
| Twelve sprite-drawing entries, including alignment repair | 124 | 266,171 |

The jump family makes the C-to-current learning path concrete: **131 original
C -> 385 initial Rust -> 160 previously composed Rust -> 97 emitted lines**.
An 81-line owner shares liquid selection, jump speed, optional boost and launch
state; all four wrappers and registration count. Actual C/old/new tests compare
every RAM byte, ordered writes and the bottom-helper call's count/state. Gravity
Suit skipping, water precedence and fraction addition without carry remain.

The homing recognizer derives axis, target, divisor, reversal, acceleration and
screen-call placement from C. Its cases include every 16-bit distance for all
six entries and 12,288 persistent updates. The DSP rule changes only exact
local signed-limit expressions, preserving surrounding shifts/casts and FIR/BRR
narrowing. Its selected alias fixtures avoid overflow in an unchanged old
addition; it claims no arithmetic correction. The solid-enemy query preserves
unusual fractional carry/borrow rounding and alias-sensitive collision-index
stores; all six intentionally faulty variants are detected.

Sprite sharing follows the current bounded and wrapping writers, retaining
their different store orders, clipping, ROM lookup guards and tile transforms.
A separate historical repair costs 26 lines before sharing removes 150. Packed
five-byte records require unaligned word access; current's related five-line
repair is accounted for separately below. Historical global-storage overlap
cases pass, while current Game cases use disjoint ROM sources. Both raw and
repaired current controls fail a synthetic Game/source overlap case; no general
aliasing repair is claimed. Component checks do not include hardware rendering.

Every sequential output passes complete optimized library/frontend builds and
source correspondence. The full **70-file** profile is now **162,136 -> 108,261
(-53,875)**, with **574,536 tokens / 1,556,253 code-text bytes** removed. It remains
8,262 lines above 99,999 and has no linked playable executable or whole-game
replay result. These rules reproduce current sharing, so none is new production
deletion credit. Final source: `TEMP/sm-tailored-next/sprite-drawing/integrated-v1`;
frozen composition: `TEMP/sm-tailored-next/integration/phase17/REPORT.md`.
Local prototype Python totals 17,599 lines across 220 files, separately from
counted Rust engine support. The nine-line Cargo manifest and disclosed external
GL dependency costs remain.

The next actual composition adds wall-jump decisions, inside-block reactions
and suit-pickup rendering: **108,261 -> 107,931 (-330)**, with 2,373 tokens and
9,330 code-text bytes removed. The respective complete reductions are 55, 154
and 121 lines, including every helper and retained setup/cleanup routine.
All 70 source files and the unchanged nine-line Cargo manifest count.

Wall-jump tests cover 17,184 defined-domain C/old/new cases, comparing all RAM,
ordered writes and the collision callbacks' arguments/state. Left-input priority,
the ignored right-query return and later pose rereads remain. Two separate
`i32::MIN` witnesses distinguish current wrapping from old debug panic and C's
undefined negation. Adopting current wrapping costs zero lines, three tokens and
14 bytes, included in the price; it is not historical equivalence at that input.

The inside-block recognizer derives one sample/retry kernel from three C copies
and sparse dispatch from all 112 original targets. It preserves the unusual
masked-XOR top test, live position reads after callbacks, sample markers and
initialized context across extension retries. All 28,312 C/old/new cases and
292 additional Rust invalid-index cases pass. Two zero-sized-array checks retain
the original bounds-panic payloads; their 14-token/18-byte correction to the
initial assertion draft is counted. The real PLM implementation is a modeled
callback boundary; that limit remains explicit.

The five suit-pickup rendering phases share two ten-line row helpers. All
69,118 C/old/new cases pass, including source/output overlap, byte/carry edges,
real dispatch/color routines and successive rendering frames. Twenty inherited
debug countdown panics preserve their partial RAM; optimized outputs match C.
Acquisition and cleanup are unchanged but unexercised by these component tests.
Its separate current-source adaptation saves only 35 lines and remains parked,
uncompiled and unexecuted. The additional all-three-consumer hitbox-search
experiment grows 26 lines despite successful builds; it is rejected without
runtime checks. Previously closed decompression/SPC music cohorts were not rerun.

All three actual sequential outputs pass full library/frontend builds and
complete source correspondence. The profile is now **162,136 -> 107,931
(-54,205)**, removing **576,909 tokens and 1,565,583 code-text bytes**. The strict
gap is 7,932 lines, with no linked playable executable or whole-game replay.
Final source: `TEMP/sm-tailored-next/suit-pickup-beam/integrated-v1`; frozen
composition: `TEMP/sm-tailored-next/integration/phase18/REPORT.md`. Local Python
prototype code totals 18,679 lines across 246 files, separate from engine support.
The separate GL experiment targets current production startup policy with
explicit legacy-C differences: use current `gl::load_with` and integer version
queries, retiring the remaining compatibility adapter. Its complete price on
the pre-suit input is **108,052 -> 106,886 (-1,166 lines)**, also removing 3,957
tokens and 28,855 code-text bytes. All 42 adapter definitions are retired, two
buffer-upload callers are adapted, nine renderer globals remain, and Cargo is
unchanged. Source correspondence and complete library/frontend builds pass.
The earlier external-GL dependency substitution is already in both inputs and
is not counted again. Startup lookup/version/cache differences are intentional
current-policy changes, without executed equivalence evidence.

Runtime and adoption are parked. Windows Defender event 1121 at
2026-10-21T01:16:41.050Z identifies ASR rule
01443614-CD74-433A-B99E-2ECDC07BFC25 for the exact
`current-gl-policy/runtime-v2/current-debug.exe` test. A DLL fallback had begun
before that event was checked; it was stopped, and all partial comparisons are
excluded. Accepted runtime coverage is zero. No security setting was changed,
and the denied test must not be repackaged to evade the block. No GL
composition was emitted; phase18 was retained at that point. Source, price,
builds and block records remain under `TEMP/sm-tailored-next/current-gl-policy`.

The subsequent initialization experiment recovers the current explicit resume
stages for both complete coroutines. The measured lineage is **284 original C
-> 690 initial Rust -> 377 preceding-profile Rust -> 229 selected Rust lines**.
Repeated transfer/wait and clear/draw/wait sequences share their stage logic;
pending waits resume before the next transfer or drawing operation. Historical
logo setup and all support remain counted. The actual complete-root result is
**107,931 -> 107,783 (-148)**, also removing 737 tokens and 3,644 code-text bytes.
Current diagnostic hooks were absent from this historical baseline and supply
no deletion credit; their six calls/ten lines and shared 132-line module are
disclosed separately.

Complete library/frontend builds and 70-file source inverse pass. **745 actual
C/old/new cases** include every byte-valued resume state, yield/counter edges
and 44 complete scripted sequences, all required to finish. The suite executes
2,005 coroutine invocations and 8,020 debug/optimized comparisons; four faulty
variants are caught. Final RAM compares byte-for-byte. Ordered callbacks carry
full-RAM digests, which are not exact intermediate-byte proofs. Child routines
and hardware are scripted boundaries; there is no whole-game replay claim.
The retained historical logo setup reads a word at an odd ROM offset; fixtures
use an odd ROM base to align it. Ordinary-base debug startup remains unvalidated.
Current Game already selects unaligned ROM reads; recover that policy in a
separate correctness stage. This state-model change does not hide that repair.

Frozen phase19 is **162,136 -> 107,783 (-54,353)**, removing 577,646 tokens and
1,569,227 code-text bytes. Its strict gap is 7,784 lines. All 70 engine files
count; local experiment Python is 19,015 lines/250 files, separate from engine
support. Source, complete costs, reproduction and component limits:
`TEMP/sm-tailored-next/initialization-coroutines/REPORT.md` and
`TEMP/sm-tailored-next/integration/phase19/REPORT.md`. No current-main or GL
saving is included.

Phase20 separately recovers the current unaligned-ROM endpoint policy. It costs
**zero lines, zero tokens and nine code-text bytes**. All 38 actual ROM-transfer
calls/42 lanes are audited, including 34 odd-offset lanes. Complete builds and
70-file inverse pass; 572 component cases cover both ROM-base alignments, all
actual call sites, direction/overlap and 16 complete logo sequences. Three old
debug diagnostics returned matching values: no old crash was observed. The old
misaligned `ptr::read` violates Rust's alignment precondition even on those
successful executions. The repair resolves phase19's tested logo alignment
limitation without claiming a portable equivalence proof for undefined old
accesses. Only the ROM endpoint constructor changes; RAM/table endpoints remain.

Phase21 then shares Torizo's three fall/countdown/landing branches through an
ordinary named helper learned from current Rust. **107,783 -> 107,723 (-60)**
includes helper, both callback rows and all registrations; 481 tokens and 1,138
code-text bytes disappear. It preserves the zero timer, timeout instruction
pairs, speed cap, movement-return contract and live facing reread after movement.
Complete builds and 70-file inverse pass. The 142,128-case C/old/new receipt
is bound to four byte-identical regenerated fixtures, original headers and all
consumed dependencies; the movement callback is controlled. Five faulty variants
are detected. No whole-game replay or current-main deletion is claimed.
Reports: `TEMP/sm-tailored-next/rom-transfer-alignment/REPORT.md` and
`TEMP/sm-tailored-next/torizo-fall/INTEGRATION_REPORT.md`.

Phase22 applies the adopted callback-registration ownership rule to the actual
historical bodies. Twelve Pirate routes already use shared inline helpers and
remain central; the other **901 original-C address/target pairs** acquire local
registration beside their named functions. All 1,024 names and direct paths,
28 owner routers, central precedence/defaults and index capture are counted.
No current Game implementation is transplanted. The exact sequential price is
**107,723 -> 106,915 (-808)**, with 3,811 tokens and 27,418 code-text bytes
removed. Its earlier standalone 807-line price is superseded, not added.

Fresh complete builds, 6,629 unchanged expanded functions, 4,696 storage/type
items and full-u32 flattened routing pass. All 901 C identities match; 381,250
debug/optimized tests exercise actual dispatchers with controlled leaf callbacks
and compare full RAM, ordered arguments and pointer returns. These are selector
checks, not gameplay-body execution. The whole historical profile now measures
**162,136 -> 106,915 (-55,221)**, removing 581,938 tokens and 1,597,774 text bytes.
All 70 source files count. Local experiment Python totals 20,558 lines/273 files;
external GL costs remain disclosed separately. The gap is 6,916 lines, without
a linked playable historical executable or whole-game replay claim.
Final source: `TEMP/sm-tailored-next/enemy-callback-ownership/phase21-composition-v1/candidate`;
freeze: `TEMP/sm-tailored-next/integration/phase22/REPORT.md`.

The module-specific private-header audit covers 325 declarations and 11,957
compiler owners. It identifies four unreferenced definitions, only **13 lines**;
park that omission. Its earlier global-name occurrence rule was conservative
across duplicated private names, but that limitation does not establish large
dead code. Almost all surviving copies are used, including macro-expanded calls.
A separate used-copy consolidation prices **428 lines** across 179 copies/32
groups, including a 125-line shared module and all imports. Differing RomPtr_RAM
variants and the four unused definitions remain. Standalone complete builds,
source inverse, expanded declarations and immediate-call-only function use pass.
Resolved targets and type records match across 11,792 normalized compiler
owners. That standalone price is not included in phase22.

The subsequent actual phase23 composition preserves the complete saving:
**106,915 -> 106,487 (-428)**, also removing **4,611 tokens and 9,901 code-text
bytes**. Replacement costs include 125 shared-definition lines, 74 import lines
and one module registration. All 71 Rust files count. A fresh source inverse
and compiler expansion compare 10,925 original and 10,778 resulting declarations
under the explicit 179-to-32 ownership map. All **1,406 selected function-value
uses** are immediate calls, including explicit function-type casts that are
themselves direct call operands. Storing, comparing or passing these addresses
is rejected. Resolved targets and parameter/expression types match across
**11,821 normalized compiler owners**, preserving external crate identities.
Complete library/frontend builds and exact frontend expansion pass. This
source-preserving ownership stage adds no runtime or replay claim.

The rule emits one shared Rust definition from each identical used private
C-header definition group, with imports replacing local copies. It includes
the moving-platform helper whose earlier emitter copied the same header body.
Different RomPtr_RAM bodies and the four unused declarations stay untouched;
the parked 13-line omission is not added. At phase23, historical output is
**162,136 -> 106,487 (-55,649)**, removing 586,549 tokens and 1,607,675 code-text
bytes. This actual batch removes **1,444 lines** from phase18, including the
zero-line ROM repair. Its remaining strict gap is **6,488 lines**. Maintained
current Rust remains separately verified at 104,959; the two budgets overlap.
Local experiment Python now totals 21,213 lines/279 files, separate from all
71 counted Rust files. Cargo remains nine lines; external GL costs are unchanged.
Source/proof package: `TEMP/sm-tailored-next/private-header-helpers/sequential-final-v1`;
frozen composition: `TEMP/sm-tailored-next/integration/phase23/REPORT.md`.

The subsequent current-to-historical owner comparison finds a missed ROM-base
emission case. Original C's `RomFixedPtr` expands to bank-address arithmetic
at every use. The earlier memory rule handled `rom_at` inputs; **306 raw
`g_rom.offset(...)` forms across 24 gameplay modules** bypassed it. Current
Rust's shared ROM access provides the replacement: call the existing `rom_at`
at the same base-expression position, retaining surrounding indices, casts,
reads, writes and alignment policy. There is no new support or dependency.

The actual phase24 composition is **106,487 -> 105,903 (-584)**, with **5,006
fewer tokens and 15,335 fewer code-text bytes**. All 172 selected addresses
match original-C definitions and the initial translated files. Independent
71-file correspondence and fresh compiler expansion pass, with 1,534,092
identical normalized library tokens and exact frontend tokens. Complete
optimized library/frontend builds pass; this static ownership change adds no
runtime or replay claim. Parent independently recounted both complete roots.

At phase24 the composition is **162,136 -> 105,903 (-56,233)**, removing 591,555
tokens and 1,623,010 code-text bytes. The actual batch removes 2,028 lines from
phase18, leaving **5,904** to reach 99,999. Current maintained Rust stays
104,959. Local experiment Python is 21,557 lines/283 files, separately counted.
The exact C/initial/current example, executable rule, source/build proofs and
output are frozen at `TEMP/sm-tailored-next/integration/phase24/REPORT.md`.

Two subsequent complete algorithm representations remove another **591 lines**
on that actual output. The native inflater removes **509**, reaching 105,394;
DSP synthesis removes **82**, reaching **105,312**. Every helper, import,
allocator boundary and unchanged caller remains counted. Parent independently
recounted all three roots and confirmed the exact one-module change per stage.
At phase26 the composition is **162,136 -> 105,312 (-56,824)**, also removing
595,722 tokens and 1,635,946 code-text bytes. It remains **5,313 lines** above
99,999. Current maintained Rust remains 104,959, without an adoption claim.

The inflater comparison follows 18 original functions and their dependencies:
386 selected C item lines, 1,136 initial translated item lines, 913 historical
module lines, 569 current module lines and **404 complete native module lines**.
Item counts omit inter-item blank lines; module counts include all imports,
blanks and wrappers. `BitReader`, `Huffman`, `Output` and `Inflater` retain
the existing algorithm, including zero-padded EOF reads, reserved-code tables,
overlapping copies and C allocation/growth/free behavior. This is a different
experiment from the rejected standards-only decoder substitution. Its emitter
recognizes the frozen input and supplies a deliberately written native template;
it does not infer arbitrary C algorithms automatically.

Fresh C/old/new debug and optimized checks pass **8,936 inflater cases** and
**3,080 complete PNG byte/filename API comparisons** per profile. They compare
exact outputs and ordered allocator events, including 1,320 injected failure
cases. Undefined malformed inputs, huge allocations and zero-capacity growth
are outside this bounded coverage. Complete library/frontend builds and source
ownership/data checks pass. Its separate current-source price is 165 lines;
it is subsequently adopted as recorded below, without adding that price to
the historical saving.
Report: `TEMP/sm-native-inflater-20261021/REPORT.md`.

The DSP replacement covers complete per-sample mixing, echo/FIR, voice,
envelope and BRR processing, retaining interpolation and noise. Shared stereo
operations and a wrapping APU-word reader retain the existing serialized types,
register behavior and output alias rules. **29,696 actual C/old/new cases**
pass, alongside eight persistent streams containing **9,584 operation
comparisons**. Intermediate stream states use SHA-256 digests; component cases
compare complete bytes. The caller-proven disjoint DSP/APU allocations remain
required. Complete builds/source checks pass. The separate current-source
price is 58 lines and remains unadopted. Report:
`TEMP/sm-tailored-next/dsp-signal-pipeline/REPORT.md`.

Neither stage adds a whole-game replay or playable historical executable claim.
All 71 Rust files and nine Cargo lines count. Experiment Python is now 22,991
lines/312 files; templates and generated harness costs are separately disclosed.
The frozen sequential checkpoint is
`TEMP/sm-tailored-next/integration/phase26/REPORT.md`.

Current Rust exposes another missed translation cohort: the opcode-grid rule
shared selection but retained the old CPU implementation helpers. The new
complete kernel rule replaces 40 functions with 51 ordinary helpers, four enums
and three impl blocks, including all 28 special consumers. Tuple address
returns replace low-out-pointer/high-return conventions. Historical bus,
interrupt, reset and hook policies remain; ADC/SBC arithmetic and grid choices
are retained. This is **105,312 -> 105,143 (-169)**, removing 3,458 tokens and
3,703 spelling bytes. Complete builds/source checks pass, together with
**471,360 original-C/old/new cases** in debug and optimized Rust. These cover
212 affected opcodes, all flag values, hook redispatch and persistent sequences,
comparing complete CPU state at ordered bus effects. Three incorrect variants
are detected. The constructed bus does not establish live SNES/game behavior.
Report: `TEMP/sm-tailored-next/cpu-implementation-kernels/REPORT.md`.

Published SDL ownership then removes 97 repeated declarations across three
files: 32 records, nine aliases, 49 functions and seven opaque targets. All
27 import lines count; no helper or adapter is added. Seventeen incompatible
integer/enum boundaries remain raw. **105,143 -> 104,742 (-401)** removes
2,413 tokens and 7,242 spelling bytes; Cargo adds one line. Complete optimized
builds and static source/ABI checks pass: 32 layouts, 208 field offsets/types,
49 exact extern-C signatures and recursive imported value domains. No arbitrary
integer is narrowed to an enum. This establishes the selected x86_64 Windows
GNU ABI, without runtime/GPU/replay coverage or hypothetical external Rust
source compatibility for renamed SDK fields and opaque types.

External ownership reduces maintained declarations, not total dependency code.
The pinned sdl2-sys 0.38.0 selected source is 28,711 Rust lines, including its
28,685-line binding; selected Windows libc 0.2.191 is 3,222 lines. Package-wide
and build-dependency counts are disclosed separately and overlap selected
counts. Native SDL was already required. The parked 110-line internal-symbol
import draft lacks provider/address/interposition proof and is not composed.
Report: `TEMP/sm-tailored-next/sdl-source-boundary/REPORT.md`.

The frozen phase28 composition is **162,136 -> 104,742 (-57,394)**, removing
601,593 tokens and 1,646,891 spelling bytes. It remains **4,743 lines** above
99,999. All 71 Rust files and ten Cargo lines count; local experiment Python
is 24,212 lines in 322 files, disclosed separately. Parent independently
recounted the complete roots. Main was 104,959 at this checkpoint. These stages
establish neither a playable historical build nor a whole-game replay result.
Checkpoint: `TEMP/sm-tailored-next/integration/phase28/REPORT.md`.

Five more current-derived cohorts compose on that exact output. These are
executable replacements of complete selected implementations or decision
fragments, with surrounding historical effects retained. Copying whole current
files would import unrelated policy differences and omit moved support costs.

| Sequential rule | Before | After | Lines removed |
| --- | ---: | ---: | ---: |
| Camera boundary policy and scroll-cell lookup | 104,742 | 104,645 | 97 |
| SPC shared operation helpers and consumers | 104,645 | 104,560 | 85 |
| DMA/HDMA execution and register handling | 104,560 | 104,494 | 66 |
| Fourteen Samus liquid decisions | 104,494 | 104,389 | 105 |
| Message drawing, selection and HDMA layout | 104,389 | 104,325 | 64 |

The actual batch removes **417 lines, 5,275 tokens and 11,163 spelling bytes**.
All 71 Rust files and ten Cargo lines count. Parent independently recounted
the roots; exactly seven source files change. The final optimized library and
actual frontend metadata build pass. Each stage preserves every other file and
binds the changed source and dependencies to its component checks. Intermediate
composed roots receive source checks; one final batch build covers integration.

Camera passes 489,298 C/old/new cases with full RAM comparisons. SPC passes
352,256 C/old/new cases and 1,179,648 old/new cases per build profile, including
state-mutating bus callbacks and persistent sequences. DMA passes 10,178 cases
with ordered bus/state comparisons, including its own register effects. Samus
passes 30,618 old/new decision cases and 67,723 original-C classifier comparisons;
unchanged surrounding effects have source correspondence, not fresh full-routine
execution. Targeted negative controls detect boundary/order mistakes in the
camera, SPC and liquid suites.

Message drawing passes 199,731 component cases, including 926 exact Rust bounds
panics and full RAM snapshots at register/callback boundaries. Its explicit
unaligned-read policy recovery costs zero lines/tokens and ten spelling bytes,
included above. The 840 odd-source cases compare C/new; old/new equivalence is
limited to aligned mapped words. Actual PPU, asynchronous message lifecycle and
arbitrary aliasing are outside those checks. No whole-game replay is claimed.

The large apparent Samus file gap is not a deletion budget: 818 lines in sm_90
and 305 in sm_91 include moved beam/projectile owners, different table spelling
and added shared macros. The selected liquid rule earns only its measured 105
lines. Separate complete particle and rectangle-contact drafts save 17 and nine
lines and remain uncompiled/parked; the liveness follow-up establishes zero
historical reduction. Their prices are excluded from the composition.

Phase29 reaches **162,136 -> 104,325 (-57,811)**, cumulatively removing 606,868
tokens and 1,658,054 spelling bytes. The remaining strict gap is **4,326 lines**.
Prototype Python adds 1,837 lines in 26 scripts, totaling 26,049 lines/348 files;
that development tooling is separate from emitted game source. Main remains
104,794 after its separate inflater adoption. This batch supplies no new main
deletion or playable historical executable. Frozen report and all owner links:
`TEMP/sm-tailored-next/integration/phase29/REPORT.md`.

Four further representation rules now compose on that exact output:

| Sequential rule | Before | After | Lines removed |
| --- | ---: | ---: | ---: |
| Raw-pointer low/high RAM-byte places | 104,325 | 103,888 | 437 |
| Native bounded index locals | 103,888 | 103,783 | 105 |
| Explicit compound RAM-word updates | 103,783 | 103,341 | 442 |
| Inferred local annotations | 103,341 | 103,256 | 85 |

The actual batch removes **1,069 lines, 23,554 tokens and 57,729 spelling
bytes**, including every helper and all 71 source files. Final complete
optimized library/frontend builds pass. Parent independently recounted every
stage. Compound updates save nine fewer lines than their standalone price;
annotation inference saves one more. Reapplying each emitter and proving its
actual input/output accounts for those overlaps.

The byte rule recovers the current `byte!(@pointer ...)` arm for 188 accesses
missed by the older address-of-only matcher. C's `HIBYTE(random_number)` no
longer expands into size arithmetic, casts and pointer offsets at each use.
All pointer/index operations stay intact; only fixed byte lanes are recovered.
Complete expanded correspondence and 327,678 C/old/new cases pass, including
all 65,536 random-number seeds and both lanes throughout valid RAM addresses.

Index inference chooses the existing native consumer type for 436 immutable
locals/ranges, removes 1,739 casts and retains nine explicit i32 boundaries.
Actual-input compiler witnesses check 424 byte/word operands and 1,793 index
uses. Source inverse, complete builds and 131,615 finite cases per profile pass.
The compound rule replaces 748 closure updates with explicit operation spelling,
delegating to the existing raw-pointer load/store arm. Checked i32 arithmetic,
wrapping u32 arithmetic, final u16 truncation, RHS types and evaluation order
remain explicit. Complete expansion and resolved-type checks pass across
11,864 library and 230 frontend compiler owners. A semicolon spelling that
prevented ordinary formatting was rejected; the accepted price uses normal
formatting and counts its full support.

Finally, 5,093 redundant local annotations disappear where the initializer
already fixes the type. Fresh complete expanded and typed comparisons run on
the actual preceding output; generic conversions requiring context remain.
Neither this rule nor compound notation changes the gameplay algorithms.
The full-source proofs supplement the bounded component checks; they do not
claim whole-game execution or a linked playable historical implementation.

Phase30 reaches **162,136 -> 103,256 (-58,880)**, removing 630,422 tokens and
1,715,783 spelling bytes cumulatively. Its remaining strict gap is **3,257**.
Cargo remains ten lines; Python experiment support totals 27,665 lines in
364 scripts and is separate from emitted engine source. Reports and executable
rules: `TEMP/sm-tailored-next/{ram-byte-places,native-index-locals,
compound-word-updates,local-type-inference}`. The actual composition and
reproduction are frozen in `TEMP/sm-tailored-next/integration/phase30`.

Further price-only controls are excluded: current-derived PPU Mode 7/window/
scanline sharing saves only 12 lines (38 without scanline helpers); lifting
conditional word stores grows 115; the complete 120-call raw-memory boundary
saves 41. Applying byte, index and annotation rules separately to committed
main saves only six, 18 and 35 lines. These scopes are parked without builds
or runtime checks and are not an aggregate budget. Current compound updates
are adopted separately below; historical savings are not main deletions.

The current-source compound rule removes **370 lines: 104,794 -> 104,424**,
plus 2,586 tokens and 9,250 spelling bytes. It replaces 801 updates across 35
files, including 575 method forms, 226 closure forms and 106 post-store rereads.
All 41 support lines and 82 Rust files count; identically formatting the control
changes zero bytes. The existing unaligned little-endian word arm still owns
destination capture, old load, RHS evaluation, store and reread timing. Explicit
promoted modes retain checked/wrapping behavior and final truncation.

Exact source reproduction and complete library/frontend expanded and resolved
type/target comparisons pass. All 78 embedded library source payloads match
their files; the corresponding 105 byte-array length types are the only allowed
library type differences. Both frontend checkpoint strings reproduce their
declared 23 source files before payload normalization; no compiler owner is
omitted. Ordinary Cargo release and changed-file formatting pass. Final main
full boot any% passes **129,712/0** and Speed Booster passes **148,406/0**, with
normal rendering, audio and comparisons. Both bind to source hash
`2dc2a2bdf5feed4e522b0dc7f48c003941fdff0b3ee69b99be9d1d44b9bdcfbb`.
Static correspondence covers unchanged operations; these replays do not visit
all updated sites or credits. The shader draft remains preserved and excluded
from committed counts. Current proof: `compound-word-updates/current-FROZEN.json`;
main build/replay logs: `TEMP/sm-compound-adoption-20261021`.

Shared RAM metadata is a further implemented emission rule. The older
original-C metadata pass retained one address/type/shape record per name.
Now consecutive equal-type/equal-shape arithmetic runs retain their explicit
names under a shared base and stride. For example:

```rust
0x00078 stride 2, uint16 Scalar => [reg_M7A, reg_M7B, reg_M7C, reg_M7D, reg_M7X, reg_M7Y];
```

All 1,169 records remain in their original order: 101 groups cover 938 records,
and 231 singletons retain their own metadata. Quoted names, duplicate aliases
and zero strides remain explicit. Array grouping describes bases only; it does
not infer lengths, ownership or disjointness. Including 22 added support lines,
the complete current source changes **104,424 -> 103,972 (-452)** and the actual
phase30 output changes **103,256 -> 102,804 (-452)**. Each independently removes
4,210 tokens and 16,206 spelling bytes. No formatting-only saving is included.
The initially ambiguous plus notation was replaced with explicit `stride`;
its extra six wrapped lines and 510 spelling bytes are charged.

Paired compiler constant evaluation compares every ordered row's address,
name bytes, C-type bytes and shape, plus all 981 current/994 historical exported
offsets. Complete expanded export inventories preserve names, `usize` types,
visibility and order, with no added or missing exports. All nonmetadata source
bytes and every caller are unchanged. The table is built entirely in constant
evaluation; RAM access/layout and runtime initialization do not change. Both
complete optimized library/frontend metadata builds pass. Parent independently
checked frozen artifacts, full source manifests, committed inputs and prices.
Current adoption additionally passes ordinary Cargo release and formatting.
These static-only changes require no gameplay replay under PLAN.

The resulting historical profile is **162,136 -> 102,804 (-59,332)**, removing
634,632 tokens and 1,731,989 spelling bytes cumulatively. The strict gap is 2,805
lines. Its 71 Rust files and ten Cargo lines count; prototype Python adds 212
lines in two scripts, totaling 27,877 lines/366 files. Constant-check harnesses
are separately 1,003 current and 1,016 historical Rust lines. This remains a
compiled translation profile, not a linked playable historical implementation.
Final source and reproducible rule: `TEMP/sm-tailored-next/ram-metadata-sequences`
(`historical-final-v2`, `current-final-v2`, `binding.json`, `REPORT.md`). Main
build log: `TEMP/sm-ram-metadata-main-build-20261021.log`.

The next declaration experiment recovers another original-C convention:
shared packing context. C2Rust emits `Copy`/`Clone` and `repr` attributes on
each record; current `typed_struct!` already shares consecutive field types
but still repeats that metadata. The new arm shares identical explicit
attributes over several same-module records. Legacy single-record syntax
delegates to the same body. Field order/types, visibility, record/union kind,
packing and names stay intact; macro-containing field types, generics and
custom metadata are excluded. Selected records have no preceding documentation
whose attachment could change. Aliases, impls and runtime callers stay intact.

Applied to the actual RAM-stage outputs, complete current source changes
**103,972 -> 103,701 (-271)** and historical source changes
**102,804 -> 102,539 (-265)**. Current removes 1,963 tokens/4,586 spelling bytes;
historical removes 2,136/4,956. Both charge six added helper lines. Current
covers 116 records in 16 groups/14 consumers; historical covers 132 in 21
groups/20 consumers. The historical standalone frontend lacks the crate-local
macro and retains its declarations rather than adding a new export or adapter.

Both final sources reproduce byte-for-byte, and complete optimized library/
frontend builds pass. Historical checks compare every selected expanded record
and derived implementation plus the entire remaining ordered compiler output.
Independent current checks compare all 116 records and 348 derived impls, then
the complete remainder with zero parser errors or missing nodes: all 6,993
remaining library functions and 84 frontend functions preserve their qualified
owners, signatures and operations. Every embedded source payload is checked
against its file before normalization. The checkpoint source key changes from
`8ef48681fb6b438b` to `400c4b06a98e46b9`; old checkpoints require regeneration.
Parent's normal Cargo release and changed-file formatting pass. Equivalent
declarations require static validation under PLAN, so no replay is claimed.

This actual historical composition reaches **162,136 -> 102,539 (-59,597)**,
removing 636,768 tokens and 1,736,945 spelling bytes cumulatively, with a strict
2,540-line gap. All 71 Rust files and ten Cargo lines count. It remains a compiled
translation profile, without a linked playable historical executable. These
metadata rules retire no bank and do not establish the remaining current budget.
Prototype support adds 728 Python lines in five scripts, bringing this profile's
reported support to 28,605 lines/371 scripts. Parent independently checked all
396 frozen artifacts, both complete source pairs, their counts and the maintained
source correspondence, preserving the unrelated shader draft.
Frozen recipe, support counts and proofs: `TEMP/sm-tailored-next/record-metadata-groups`
(`current-final-v3`, `historical-final-v4`, `current-proof-independent-v3`).
Main build: `TEMP/sm-record-metadata-main-build-20261021.log`.

The complete ordinary-place compound-update follow-up is rejected. Unlike
the named RAM rule, it covers repeated field/place assignments and captured
old values, excluding named RAM and raw unaligned chains. Its natural compound
spelling changes **103,256 -> 103,327 (+71)** across 1,957 historical sites and
**104,424 -> 104,532 (+108)** across 1,703 current sites. Support accounts for
69/79 lines and callers/imports for another 2/29. Tokens and text decrease,
but caller spelling alone does not save physical lines. The alternative
method-name grammar grows 386/417 lines. Formatter-only drift is zero. No
builds, behavior proof or runtime tests were pursued after these complete
prices. Frozen report: `TEMP/sm-tailored-next/ordinary-place-updates/REPORT.md`.

A separate static legacy GL metadata study also remains rejected. Four
declarations contain 807 symbol names; 25 Cartesian grids and five prefix
groups generate 314 entries while leaving other names explicit. Ordered
source-level expansion matches. Including the 18-line macro, the whole
historical root prices **103,256 -> 103,084 (-172)** and loses 7,402 spelling
bytes, but adds **513 syntax tokens**. All 12 runtime bodies and every other
file stay byte-identical. This fails the selected token/text rule and is not
composed. No compiler or runtime check was run; recursion limits and compiled
constant payloads remain unchecked. This is distinct from the runtime-blocked
current-policy GL replacement. No GL/test/DLL/executable was launched or
repackaged. Report: `TEMP/sm-tailored-next/gl-static-groups/REPORT.md`.

The context-free helper experiment is distinct from the associated-function
and receiver trials: it removes an unused injected `Game` parameter and its
pure call operand, rather than relocating routines. On committed `5fd12431`,
the strict public-contract-preserving version changes six private helpers and
nine calls but saves zero lines. An explicit public-signature migration covers
eight functions and 40 source calls: **104,424 -> 104,406 (-18)**, with 127 fewer
tokens and 316 fewer spelling bytes. The changed public signatures belong to
`CalculateAngleFromXY` and unused `nullsub_341`; callback and address-taken
contracts remain excluded. Phase30 contains no injected `Game` parameters,
so its direct cohort and saving are zero. Both complete current prices are
parked before builds or runtime validation. No maintained signatures change.
Report: `TEMP/sm-tailored-next/pure-game-context/REPORT.md`.

The separate named-alias grouping experiment also remains a price-only draft.
Unlike the earlier shared enemy layouts, it retains every alias while sharing
identical RHS type/signature text within a module. On the RAM-stage controls,
current **103,972 -> 103,777 (-195)** covers 186 aliases in 33 groups/14 files;
historical **102,804 -> 102,740 (-64)** covers 145 in 29 groups/18 files. The
13-line crate-local macro is included, and the independent frontend is unchanged.
Current removes 721 tokens/3,152 spelling bytes; historical removes 556/2,731.
Formatter controls contribute zero line/token/text savings; current's larger
price includes existing blank separators. Historical control formatting only
normalizes newline/module order without count savings. These complete prices
have no build, equivalence or runtime validation and are excluded from accepted
totals. Report: `TEMP/sm-tailored-next/type-alias-groups/REPORT.md`.

The follow-up experiments compare physical lines, noncomment syntax tokens
and token-spelling bytes. All helpers and imports count. This is a guard
against making progress appear larger by packing unrelated statements into
macro rows; it supplements the required physical-line count.

The broad computed-field experiment demonstrates the problem. On current
Rust it removes **982 lines but adds 8,301 bytes of code text**; on the
historical phase10 control it removes 634 lines but adds 31 tokens and 6,328
bytes. Repeating `crate::set_fields!` outweighs the short receiver expressions.
Reject those broad variants as volume strategies, despite passing compiler
and exact-expansion checks. The selected field policy admits only groups that
remove both tokens and text, then charges the shared macro arm: **299 lines,
1,800 tokens and 1,788 bytes** removed on its separate current-source control.

The named-RAM rule has a stronger result: **576 lines, 5,040 tokens and 10,080
bytes** removed from current Rust across 472 groups / 1,312 stores, with no
new support. It reuses the existing ordered-write form, preserving every
value and index evaluation at its original store. Complete library/frontend
builds and compiler-expanded correspondence pass across 7,097 functions;
the checker independently verifies the source bytes embedded by the verifier.
This is new current-source applicability, unlike reproducing reductions
already present in main. It is notation sharing, not algorithm removal.

The first terminal-loop notation similarly removed 465 current lines while
adding 200 tokens and 1,376 bytes, including its helper. Removing qualification
still added 263 bytes. Those spellings are parked as volume changes. A revised
`do_until! { guard; body }` rule names its terminal-check semantics directly
and removes the redundant delimiters/keyword; its measured complete current
cohort removes 465 lines, 438 tokens and 537 bytes. Exact body scope, control
transfer and temporary lifetime checks are required before composition.

The reduction-only current composition is **107,520 → 106,181 (-1,339)**,
removing **7,265 tokens and 12,393 token-spelling bytes**, including support.
It applies the revised 159 terminal loops, 114 selected field groups / 584
stores, then 472 RAM groups / 1,312 stores. Restoring the exact committed shader
excludes its one-line saving and preserves the user's unrelated draft.
Both complete frontend-enabled builds, formatting and independent expanded
library/frontend correspondence pass: 7,097 library functions, 84 frontend
functions and independently checked embedded source/cache payloads. Source
hash for that reduction-only snapshot:
`265a6c5dc4f9f690bc030ea1244fa9d9dcf4ae7e5fd5ab4390b7b4ae83face14`.

The separately verified current alignment repair adds eight lines, giving the
adopted **106,189-line** result: **1,331 fewer lines, 6,104 fewer tokens and
8,460 fewer token-spelling bytes**. It changes 83 captured RAM words to raw
unaligned accesses, preserving all 124 reads and 83 writes. The inherited
reference form aborts in debug in the actual music queue at odd address 1599.
The repaired function passes 66,400 original-C cases / 132,800 complete-RAM
comparisons. Ordered-effect checks pass 458,756 cases in each debug/release
build, including aliasing, rereads and temporary destruction. This correctness
cost remains explicit; these component checks do not establish every path's
movie execution or eliminate every unrelated unsafe access.

All four changes are committed on main: terminal loops `f9c75bc8`, selected
field writes `7a661f38`, named RAM writes `c556eb9d`, and alignment repair
`dc5d2f08`. Final optimized frontend-enabled library/frontend checks pass.
The final source matches `TEMP/sm-tailored-next/adoption-final` byte-for-byte,
including the preserved, uncommitted shader draft. Full boot any% passes
**129,712 frames / zero failures** (109.09 replay seconds); Speed Booster
passes **148,406 / zero** (140.81 seconds). Both retain normal rendering,
audio and comparisons; neither reaches credits. The final verifier source
hash is `f99e3783a9321629fab062c4161ba2e21bd5486ca0020cba2955b23037f33cc5`.
The shader draft accounts for the working count of 106,106 and is excluded
from the committed 106,189 and its reduction budget.

These rules remove no bank module. The adopted source needs another
**6,190 lines** removed to reach 99,999; it does not establish that remaining
budget. The historical-source pipeline and maintained-source reductions are
separate comparisons, not additive savings. The subsequent finite instruction
operation experiment is now complete, as recorded below. Adoption artifacts are under
`TEMP/sm-tailored-next/current-record-writes`, `current-ram-writes`,
`current-composed`, `record-writes`, `unaligned-current` and `adoption-final`.

### Price new representations against the compact engine

The next implemented experiments all use the repaired **106,189-line** current
root. These are complete separate prices; overlaps prevent adding their savings.

| Actual replacement | Complete lines | Change | Decision |
| --- | ---: | ---: | --- |
| Typed instruction operations, data and executor | 106,740 | +551 | Reject |
| Boss/presentation finite state programs | 106,831 | +642 | Reject |
| Callback address constants owned by dispatch rows | 106,127 | -62 | Park |
| Same-type constant declaration grouping | 106,095 | -94 | Park |
| Actual combination of the two constant changes | 106,079 | -110 | Park |
| Residual typed/byte RAM pointer chains | 106,083 | -106 | Park |
| Ordinary internal macro imports | 106,142 | -47 | Park |
| Infix word arithmetic with ordinary operands | 106,115 | -74 | Reject: tokens/text grow |
| Infix word arithmetic with named RAM operands | 106,069 | -120 | Park |

The instruction prototype replaces 340 handlers with complete ordered effects,
operand/destination data and separate pointer/offset/pair return contracts.
Its 1,377-line table/runner includes 204 support lines and 1,173 table lines;
direct/shared exceptions, imports, routing hooks and registration are counted.
It adds 299 tokens under its disclosed lexical counter, although LF-normalized
source bytes fall by 13,528. An earlier raw-byte increase mixed LF and CRLF and
is superseded by that normalized figure. The negative physical-line result
rejects the design before compilation or runtime checks. Its numeric place
data would also need descriptive names and their costs before any adoption.
This tests an actual finite operation representation, distinct from the older
experiment that merely fused whole handlers into match arms.

The distinct non-instruction runner replaces 98 complete callbacks across 12
boss/presentation dispatchers and retains 306 other phases. It executes 39
exact underflow waits, 179 ordered word effects, 137 native actions and two
word gates, with all retained callback identities and caller costs. Its
155-line shared runner is charged. The complete result adds 642 lines, 2,963
tokens and 12,989 spelling bytes. Final complete library/frontend compilation,
formatting and numeric routing checks pass; it is rejected before runtime work.
The old Mother Brain prototype's 318-line saving is unavailable after current
callback/prefix/countdown sharing; it is not additional missing budget.

Constant ownership covers 48 same-module address/function pairs. The complete
ownership and combined library/frontend builds pass; inverse source checks
retain all 1,123 constants. After grouping, ownership removes only another
16 lines while adding 265 tokens and 519 bytes. It is not a substantial new
source reduction. No gameplay replay was needed or claimed for this price.

The pointer rule removes casts to byte pointers and back using the existing
`ram_ptr!` byte-offset grammar: 173 replacements across 26 files, including
71 macro-body sites and eight macro templates. It preserves sequential offset
expressions, their units and order; typed offsets before the byte cast remain
outside the rule. The complete candidate also removes 2,132 tokens and 4,271
spelling bytes, with zero support cost. Thirteen matcher controls and the full
fixed-point source audit pass; broad compilation/replay is deferred because of
the small price. Its report traces the actual C flare-animation timer store
through original generated Rust, maintained Rust and emitted output.

Internal imports remove repeated paths at 1,670 sites across 52 files, including
62 added import rows. This removes 5,151 parser tokens and 16,032 spelling bytes
but only 47 lines; many statements already occupy one line. This is an actual
formatted, uncompiled price, with no macro-resolution or behavior proof.
Token counts use each experiment's documented counter and are not aggregated
across different lexers.

The arithmetic trial changes representation rather than repeating the older
nested `add16`/`sub16` helper experiment. A small infix grammar retains the
explicit binary tree and conventional operators; the second form also uses
confirmed RAM names directly, with parenthesized external Rust operands.
The complete cohort contains 932 roots: 472 signed deltas and 460 low-word
projections, counting 13 overlaps once. Ordinary operands remove 74 lines but
add 2,591 tokens and 1,373 spelling bytes. Named operands remove 120 lines,
810 tokens and 3,978 spelling bytes, including all macro support. Both remain
formatted pricing drafts without compilation or behavior proof. The named
form is parked because its complete saving is small.

None of these trials changes main or retires a bank. They establish no new
substantial adoption or demonstrated route through the remaining 6,190 lines.
Reports and source candidates are under `TEMP/sm-tailored-next/instruction-data`,
`finite-state-programs`, `address-ownership`, `ram-pointer-chains`,
`internal-imports` and `infix-word-math`.

### Test complete domain algorithms after recovering compact translation

The next studies use the repaired **106,189-line current source**, with original
C as a guide to algorithm boundaries. They include complete setup, processing,
exceptions, consumers and replacement support; gross function spans are not
reported as removable code.

| Experiment | Complete result | Validation and decision |
| --- | ---: | --- |
| Palette pipelines: room-transition scheduling, Phantoon interpolation/targets, cinematic palette planes and suit/visor effects | 106,169 (**-20**) | Full optimized library/frontend builds and source checks pass; park before runtime |
| Native recording/playback, save/load and history rebasing | 106,121 (**-68**) | Formatted price only; park before builds or behavior checks |
| Samus locomotion/contact lifecycle | No implemented price | 23 new items occupy 313 gross lines; existing sharing consumes larger repeats |
| Cross-actor world collision | No implemented price | 1,428 lines audited; no large complete replacement established |
| Complete X-ray and Mother Brain beam-window rasterization | 105,984 (**-205**) | Compiled components and focused/full replays pass; adopted |

Palette sharing replaces 20 functions and removes 221 lines from their bodies,
but the common numerical operations, range/health helper and caller costs leave
only 20 net. It also removes 858 noncomment tokens and 1,305 token-spelling bytes.
OR-versus-add color composition, full/swapped accumulators, divide-by-zero
behavior, do-once ranges and live timer reads prevent a single simpler formula.

The recorder's nine principal functions shrink from 442 to 331 lines. Native
buffer ownership, explicit initialization before `Game::assume_init`, fallible
growth, borrowed cursors and snapshot/file adapters repay most of that saving.
The stream/checkpoint formats and current audio/history behavior remain in the
candidate. Shader construction independently uses ByteArray, so its type and
remaining helpers cannot be counted as deletions. Original C supplies 17 lineage
functions occupying 292 lines, but lacks some current audio/history behavior.
No compatibility claim is made for this unbuilt candidate.

The locomotion and cross-actor studies expose meaningful differences after
existing sharing: ordinary Samus falling uses velocity captured before
acceleration, while bomb falling uses the updated velocity; enemy horizontal
movement commits against live coordinates, while vertical movement commits a
precomputed position; beams move before reactions and continue their span,
while actor sweeps stop at the first collision. `Game::react_to_block` and
`square_slope_occupied` already share the retry and occupancy algorithms.
These bounded preflights do not prove that broader redesign cannot help.

The complete beam-window replacement is adopted: **106,189 -> 105,984 (-205)**,
also removing 1,986 lexical tokens and 4,901 token-spelling bytes. Its 668-line
descriptive owner includes all shared routines and retained exceptions; direct
imports/callers and source registration count. Old bank facades are removed.
X-ray and rainbow beams share directional setup and fixed-point edge advancement,
while retaining the oracle-corrected positive-down carry/store/cleanup behavior.
Public C alone is insufficient to validate those corrections. The 500-line
discovery threshold does not veto a useful complete replacement below that size.

Both complete optimized library/frontend builds pass. Separate complete old/new
Rust DLLs pass **64,100 cases**, comparing all 128 KiB RAM, returned panic status
and ordered full-RAM hash observations at the sole fixed printf call. Cases
cover all X-ray directions, on/offscreen destinations, synthetic gradients,
row-zero/wrapping corrections, actual Samus/eye-beam callers and color aliasing,
all six rainbow directions, saturation, overlapping headers, all 16 public
dispatch indices and matched null-dispatch panics. Independent source review
checks directional arguments, retained helper tokens, live rereads and routing;
1,310,720 independent scalar calculations support the edge arithmetic check.
The final 1,732 added cases cover the special-right implementation and public
right-beam caller with real ROM gradients. They change no candidate source.

Tracing the complete responsibility back through its actual sources gives
**521 original C lines -> 1,322 initial Rust -> 876 corrected current Rust ->
668 lines in the shared owner**. The C blocks contain 218 X-ray and 303 rainbow
lines; the initial Rust contains 610 and 712; current blocks contain 631 and
245. Imports, retained neighboring declarations and source registration make
the complete adoption **205 net lines**, rather than the 208-line body delta.
Original C and initial Rust lack the later positive-down corrections, so this
lineage explains the representation and overhead without claiming equivalent
behavior or an additional saving. The reusable design is shared edge traversal
with explicit direction parameters, retaining distinct carry/cleanup contracts.

The Charge Beam boot route passes **28,699 frames / zero failures**, requiring
seven probes: calculate (289 calls), shared edges (1,645), generic ramp (74),
positive-down onscreen (177), eye update (289), full-beam phase (271) and cleanup
(one). Changed rendering executes at inputs 28,228..28,517, shared edges at
28,323..28,456, and cleanup at 28,518; 180 subsequent frames are checked. The
same clean endpoint passes. Full boot any% **129,712/0** and Speed Booster
**148,406/0** pass with normal rendering/audio/comparisons. Their compiled source
hash is `71a1ec954a42ee3b57783f80867b687b95ac7a2d32a41628a2e55ba2d24f77a2`.
Adoption preserves an existing field-store macro comment that the frozen
experiment predated; that sole final delta has identical executable tokens and
fresh complete library/frontend compilation. The shader draft remains untouched.

These movies do not exercise the Mother Brain encounter or credits. Constructed
components cover its changed behavior; they exclude raw out-of-allocation
geometry and the unchanged special-right no-carry cleanup that can leave the
RAM allocation. Hold further cleanup of that path without additional context.
The study retires this responsibility from two banks, not the entire banks.

Reports and candidates: TEMP/sm-tailored-next/{palette-pipeline,
state-recorder-native,samus-locomotion-cycle,actor-world-collision,beam-windows}.
The prices are separate experiments, not an additive reduction budget.

The complete CPU/SPC tracing replacement is adopted separately:
**105,984 -> 105,840 (-144)**, also removing 785 noncomment tokens and 1,863
token-spelling bytes. Its source lineage is **206 original C lines ->
1,994 initial Rust -> 464 current Rust -> 320 native Rust**. The initial total
includes duplicated declarations; the tracing table/function portion is 1,597
lines. No helper or caller cost is moved outside the counted source.

The executable emitter retains both full opcode-text tables and operand-kind
maps exactly. It derives the twelve short immediate forms from the wide forms,
removing the sparse 256-slot table, and shares native hex formatting and flag
letters across the four actual formatters. It preserves padding, the terminating
NUL, unconditional bus reads, relative-address arithmetic and post-read processor
state. All three callers retain their signatures and output buffers.

All **742,400 cases** pass across original C, initial Rust, current Rust and the
candidate, with Rust checked in debug and release. Comparisons include the full
256-byte output buffer and untouched tail, ordered bus addresses/count, both
processor states, pointer replacement and a trailing guard. Effectful bus
fixtures exercise state changes during reads. Six compiled incorrect variants
are detected, including changed padding, operand order, read count and premature
state capture. The actual complete library and frontend compile after adoption;
formatting and progress checks pass. These are diagnostic-component checks, not
a new gameplay replay or execution of every real hardware bus path. Sources,
emitter and checks: `TEMP/sm-tailored-next/tracing-native`.

Nonnull callback-table inference is also adopted: **105,840 -> 105,769 (-71)**,
including three helper lines, with 786 fewer tokens and 3,430 fewer code-text
bytes. All 79 immutable tables contain known functions in every one of their
922 slots, so their types become ordinary function-pointer arrays. The demo-set
slice aggregate and all 82 consumers change together. Four genuinely nullable
tables retain `Option` and their existing failure behavior. Every selected table
is private, with every reference accounted for and no raw-pointer escape.

Complete compiler-expanded correspondence proves the same targets, casts, checked
indices, lengths, argument order and callback bodies after removing only the
unreachable null-failure branches. Final-source optimized library/frontend
compilation, formatting and progress pass. Full boot any% **129,712/0** and
Speed Booster **148,406/0** pass on the combined production source, hash
`83c7a77846eaea50a541bb4b02b1ebaa5ac19f2cae3f47fd57c3054f3fa0e9fc`.
These replays retain rendering, audio and all comparisons; neither reaches
credits or executes every callback. Tracing retains its component coverage.
The actual combined maintained-source reduction is **215 lines**. The original
C timer table is explicitly `Func_U8 *const` with seven nonnull function entries;
this supplies a concrete C-to-native-Rust inference rule. Emitter, full inventory,
proof and adoption logs: `TEMP/sm-tailored-next/nonnull-tables`.

The latest adopted family shares Samus pose refresh and history:
**105,769 -> 105,685 (-84)**, with **1,270 tokens** and **3,951 code-text bytes**
removed. Three descriptive routines in the 21-line `samus_pose` owner replace
36 sequences; two duplicate bank history functions and their seven callers
are retired. All imports and both source registrations count. Original C makes
the paired history-copy meaning explicit: each 16-bit copy preserves adjacent
direction and movement-type bytes. The owner retains the exact ordered stores
and the existing movement/animation routine calls, without pre-capturing values.

Discovery used a materially new scope: proper sibling-statement windows rather
than whole functions or compound blocks. The 2.87-second pass covers 45,470
windows in 2,978 ordinary functions and 2,421 callback bodies, including small
routines. Its 39 non-subsumed groups do not establish another large deletion
budget; generalized RAM addresses otherwise tend to join unrelated effects.
The exact typed-ROM-index idea was also checked against its old study, not
repeated: the current Standing routine is token-identical to that control,
and the complete old family saved only 25 lines without compiler/runtime proof.

Samus validation proves all 7,080 remaining compiler-expanded functions match
after exact helper inlining. Both removed wrapper bodies match the actual new
history helper, all caller identities are accounted for, and none of the eight
history-only callers retains a RAM pointer across the new helper call. Complete
library/frontend builds and formatting pass. Probed and clean boot any% intro
runs pass **2339/0**, exercising all six required helper/animation/history/tile
probes plus 120 follow-up frames. There are 121 tile consumers after history
writes. This route visits no gameplay rooms and claims no normal movement,
Draygon, suit, Crystal Flash or endgame execution; unchanged-effect proof covers
unvisited callers. Final dead-import cleanup preserves all remaining replayed
tokens and has fresh builds. Frozen experiment: `TEMP/sm-tailored-next/sibling-sequences`.

The subsequent sprite emission experiment reproduces an actual current debug
abort: five-byte sprite records contain unaligned word fields. Its separate
production repair changes eight reads and two OAM stores in `draw_sprites` and
`draw_bounded_sprites` to unaligned operations. Explicit tile locals preserve
RHS-before-destination evaluation. Native endianness and store order remain.
The complete cost is **105,685 -> 105,690 (+5 lines, +38 tokens, +234 bytes)**.

All 265,737 supported current-Game component cases pass against C, comparing
every RAM byte. The raw current debug abort is preserved. Both raw and repaired
current helpers diverge for a synthetic source overlapping Game RAM; this is a
separate unsupported aliasing case, not a claimed repair. The complete caller
audit finds ROM sources only. Library/frontend builds, formatting and exact
source correspondence pass. Full boot any% **129,712/0** and Speed Booster
**148,406/0** pass on source hash
`fe8feacc6e4d8cf8258c7668badd8745c3ad00098511d0128cac4a16f1e6b1ba`.
Both execute the three required wrapping/inside/bounded sprite probes, with
normal rendering/audio and all comparisons. Other clipping variants have
component coverage; neither route reaches credits. Frozen repair, controls and
replays: `TEMP/sm-tailored-next/sprite-drawing/current-repaired` and its parent
`REPORT.md`. This is a correctness cost, with no production deletion credit.

The subsequent shared-import experiment is adopted: **105,690 -> 105,309
(-381 lines)**, with **4,481 tokens and 17,050 code-text bytes** removed. It
consolidates 955 import entries across all 32 banks into 119 common entries in
a 39-line `game/prelude.rs`. All 82 source files, remaining local imports and
both source registrations count. The prelude imports only existing shared
owners; bank-specific gameplay calls, distinct same-name origins and uncommon
dependencies remain explicit. Function bodies and macro paths are unchanged.

This extends the original-C header lesson: declarations already shared in
`types.rs` need not repeat their import scaffolding in every translated unit.
The origin/name/visibility rule emits ordinary Rust imports. Public bank paths
retain their definitions, although each public glob exposes additional common
names. Existing consumers are verified; arbitrary external glob-import
collisions are not an unchanged-API claim. No bank is retired by this change.

Final paired library/frontend builds, formatting and an independent 81-file
source inverse pass. External probes check 441 old public type paths and 4,962
constant paths. Complete compiler expansion preserves all operations. A
separate typed/resolved comparison checks ordered canonical definition targets
and types for 12,645 library owners and 313 frontend owners, preserving external
crate identities. Its only excluded owner is the source-cache `FILES`, whose
actual payloads and new registration are independently checked. This closes
the name-resolution question that expanded spelling alone cannot answer.

The exact sprite repair remains included. This import-only change has no new
replay, under the plan's static-validation rule. It is a current-source saving,
not an additional historical-pipeline reduction; earlier parked import/unused-
declaration prices overlap and cannot be added. Frozen source, patch and proofs:
`TEMP/sm-tailored-next/gameplay-prelude/REPORT.md` and `final/sm/src`.

The resumed cross-file enemy callback experiment is also adopted:
**105,309 -> 104,959 (-350 lines)**, removing **3,499 tokens and 29,375 code-text
bytes**. It uses the exact earlier 913-arm scope and current bodies, replacing
the old resource-parked 346-line price. Numeric registrations move beside their
existing named callbacks through the existing macro. Twenty-eight owner routers
retain central exact-route precedence, defaults and preinstruction index capture.
All 1,002 callback names/signatures remain; no macro, import or support cost is
omitted. This prepares ownership but retires no bank or central adapter.

Complete library/frontend builds pass. Compiler expansion preserves 7,085
functions and 4,727 storage/type items; flattening agrees over the complete u32
address domain. Debug/optimized tests pass **381,250 cases** using the actual
dispatchers and instrumented leaf callbacks, comparing full RAM, ordered
arguments and pointer returns. This suite tests routing, while unchanged
gameplay bodies have complete source correspondence. Probed boot Speed Booster
passes **21015/0**, reaching all six central/owner family labels plus 120 frames.
Clean full boot any% **129712/0** and Speed Booster **148406/0** pass with normal
rendering, audio and comparisons. Neither route reaches credits; grouped probes
do not establish every owner executes in movies.

The adopted 82-file source matches the frozen replay source after CRLF/LF
normalization, with the unrelated shader draft preserved. Replay source hash:
`0b13d575b48aba371357ceb436af7b357440f9cbdc097ade972778b5281a02a9`.
Progress reports 104,876 working lines including that unrelated 83-line draft
reduction; the committed count is 104,959. The cast-line counter increases one
because Torizo's unchanged two-cast expression wraps onto separate lines.
Unsafe-function declaration lines fall through grouping, without changing
expanded unsafe signatures. The remaining target gap is 4,960 lines; this result
does not fund it. Source, patch and exact checks: `TEMP/sm-enemy-callback-current`.

The complete native inflater is subsequently adopted: **104,959 -> 104,794
(-165 lines)**, with 2,066 fewer tokens and 4,638 fewer code-text bytes. All
four owners, lookup tables and C allocation boundaries count. The final module
and unchanged PNG caller match the frozen current test inputs exactly;
independent review and all checked source/binary/fixture hashes pass. Validation
reuses 8,936 C/old/new inflater cases and 3,080 old/new full PNG API comparisons,
then checks formatting and fresh complete optimized/frontend and normal Cargo
release builds. The initial link failure was an incorrect local SDL SDK path;
using its existing nested directory resolves it. No gameplay replay is needed
for this frontend-only change. Undefined malformed behavior, huge allocations,
zero-capacity growth and cross-platform execution remain outside coverage.
The unrelated shader draft remains unchanged; working count 104,711 excludes
no maintained files but includes that separate 83-line reduction. The committed
gap is **4,795 lines**. This adoption adds no historical-pipeline saving.
Reports: `TEMP/sm-native-inflater-20261021/REPORT.md` and final build logs in
`TEMP/sm-inflater-adoption-20261021`.

Three further complete current-source experiments use the same committed
**105,769-line** control. Their independent prices are not additive:

| Complete experiment | Result | Validation and decision |
| --- | ---: | --- |
| Native configuration/input collections and scalar routing | 105,669 (-100) | Complete library/frontend and paired harness builds pass; runtime launch blocked; park |
| Standalone Mother Brain sequence owner, all 124 steps | 105,755 (-14) | Complete builds pass, code text grows 593 bytes; park |
| Compiler-guided immutable local/parameter bindings | 105,764 (-5) | Complete builds and source-token correspondence pass; park as volume work |

The configuration candidate replaces manual keyboard hash chains with a
`BTreeMap` and gamepad lists with ordered per-button vectors. Duplicate attempts
still count toward the keyboard limit, first keyboard bindings win, and more
specific/newer gamepad entries retain priority. All support remains counted;
the complete result removes 911 noncomment tokens and 2,938 code-text bytes.
Actual before/after config sources, unchanged project helpers and real SDL key
lookup compile into the paired harnesses, but **zero runtime tests executed**.
Defender Exploit Guard event 1121, rule
`01443614-CD74-433A-B99E-2ECDC07BFC25`, explicitly blocks their launch under
administrator policy. No protection changes or repackaging were attempted.
The old gamepad allocation requests 64 entries but grows only every 256;
executions above 64 cannot serve as a defined-behavior reference. The native
collections also need reconciliation of allocation-failure handling before
adoption. Report, exact candidate and unexecuted harness:
`TEMP/sm-config-engine-31566113`.

The Mother Brain experiment re-emits current effects into one standalone owner,
including 47 delay gates, 99 typed continuations, all dependencies and both
source registrations. Earlier callback sharing has already absorbed nearly all
of its old 318-line price. The complete current owner occupies 1,160 lines and
still imports 52 bank helpers/tables/types. It removes 1,093 tokens but grows
code-text bytes; no bank retirement, current runtime or additional endgame
coverage is claimed. Frozen report: `TEMP/sm-motherbrain-sequence-31566113`.

The immutability experiment uses real compiler diagnostics with
`--force-warn=unused_mut`, selecting only machine-applicable removals of the
binding qualifier. It removes 470 qualifiers across 37 files, including 178
ordinary bindings inside existing callback/loop macro bodies. Shared macro
definition contracts remain unchanged; 23 expansion diagnostics are excluded.
Complete source-token correspondence allows only those removals, and library
and actual frontend builds pass. This improves binding intent but removes only
five formatted lines. Report: `TEMP/sm-tailored-next/immutable-bindings`.

Broader representation experiments use independent 105,984-line controls:

| Complete experiment | Line change | Decision |
| --- | ---: | --- |
| Ten existing ROM-script interpreters plus allocation/scheduling | +117 | Reject; common traversal and adapters grow |
| Shared live hardware, snapshots, checkpoints and audio diagnostics | -95 | Reject; adds 1,542 tokens and 2,921 code-text bytes |
| Complete contextual Kraid controller | +33 | Reject for volume; owns 132 entries but adds source |
| Kraid controller with actual receiver methods | +196 | Reject; 26 identity/order wrappers repay gains |
| Cached raw Samus RAM view, preserving little endian | +921 | Reject; all 2,520 accesses/stores/updates and layout costs counted |
| Remove unused generic PPU capabilities | No candidate | Only 236 gross lines identified; none proved unused |

The first five complete candidates compile across library/frontend targets;
they have no runtime-equivalence claim. The Samus study also prices local views
and a native-endian control; all grow. State ownership includes 1,962 migrated
field accesses and all retained cache, legacy-save and pointer-rebinding work.
These are bounded rejections of implemented representations, not limits on
every possible redesign. Reports are in `TEMP/sm-tailored-next/{object-lifecycle,
contextual-kraid,samus-overlay,ppu-sm-specialization}` and
`TEMP/sm-hardware-state-d878362a`. Do not add their independent prices.

A source-ownership count also clarifies the remaining search space. The repaired
control has **83,741 lines under game/**, **13,135 root-level lines**, **2,743
platform lines**, **6,077 SNES lines** and **493 verify-subdirectory lines**:
106,189 total. Original gameplay C is 83,657 lines; the other built C is 16,717.
Shared game types and RAM helpers live among the Rust root files. Consequently
the 84-line game-directory difference is ownership accounting, not semantic
equivalence, and the 22,448 lines outside game/ are not a deletion budget.
Header lines remain separate from the original C ceiling.

## Independent transpilers: executed comparisons

The initial trial built three independent translators from pinned source and
ran them locally against public original C; the Cpp2Rust trial follows below.
This answers whether another translator can produce
smaller useful Rust without inferring quality from its advertised features.

| Translator | Actual trial | Result |
| --- | --- | --- |
| [Corrode](https://github.com/jameysharp/corrode/tree/34053342c2f1ca04f23ad94d67057f14e74d9fb9) | Built with GHC 8.0.2; original entry runs through `runghc` | Smaller compiling Rust for scrolling DMA, tile streaming and complete sm_87; component comparisons pass, packed game layout fails |
| [CRUST](https://github.com/NishanthSpShetty/crust/tree/3d621dd2b92eb78d8927ad54c63c9f42706ac921) | Built; tested raw/preprocessed sm_87, util and scrolling units | Raw bank becomes unsupported-include/comment output that fails Rust parsing; util and preprocessed units panic |
| [Sonder](https://github.com/azaleacolburn/sonder/tree/24300112860b6f236c57cda3711a0d08a2ca627f) | Built; tested the same three raw and three preprocessed units | Repeated lexer messages until timeout; no Rust output |

Windows denies execution of the freshly built executables. Corrode therefore
uses its unchanged Haskell entry through the interpreter; small DLL adapters
invoke CRUST/Sonder's unchanged tokenizer/parser/converter. Adapters provide
input/output and panic capture, without implementing translation. Logs separate
host/harness failures from the observed translator failures.

### Corrode produces materially smaller code on compatible inputs

The real public C units are the same extracted complete algorithm units used
in the C2Rust experiment. Counts include emitted declarations and use the same
rustfmt with its default 100-column width. Neither output needs extra runtime
support for these units; the common external test harness is excluded equally.

| Complete algorithm unit | C2Rust 0.22.1 | Corrode | Fewer lines |
| --- | ---: | ---: | ---: |
| Original horizontal/vertical scrolling DMA | 443 | 256 | 187 |
| Original row/column tile streaming | 592 | 467 | 125 |
| Shared row/column tile algorithm, alternative to the previous row | 344 | 277 | 67 |

The two disjoint original units total **1,035 -> 723: 312 fewer lines (30.1%)**.
The shared-algorithm alternative is not additive. Corrode's generated units
compile without source fixes after formatting. Scrolling passes **6,250 cases**
against original C and C2Rust, comparing all 128 KiB RAM and **358,220 ordered
register writes**. Tile streaming passes **3,076 cases across six binaries**:
original/replacement C, C2Rust and Corrode, checking all RAM, both layers/axes,
all four flips, wrapping positions and Mode 7 skipping. These are component
comparisons with recorded register writes, not actual PPU/DMA, ROM or full-game
history. No whole-bank reduction is inferred.

A further **complete sm_87 bank** trial supplies both translators the identical
8,147-line preprocessed public C input: **679 C2Rust lines -> 627 Corrode lines,
52 fewer (7.7%)**. Both outputs compile and contain the same 34 function names.
Corrode's smaller declaration prefix offsets longer generated function bodies;
the 30.1% result above does not generalize even to this tested bank. Full-bank
runtime behavior remains untested.

Corrode also omits a relevant layout constraint. Its
[declaration handling](https://github.com/jameysharp/corrode/blob/34053342c2f1ca04f23ad94d67057f14e74d9fb9/src/Language/Rust/Corrode/C.md)
leaves attributes unprocessed. Translating the actual pragma-packed
`DebugDoorDef` from the original header gives **12 bytes / field offset 10**,
where the original C gives **11 bytes / offset 9**. Both compile; executed
measurement functions expose the difference. A subsequent use-site audit
finds only its declaration in the original C: that example alone does **not**
demonstrate changed gameplay. Padding can differ harmlessly for an internal
object accessed only through normal fields. The follow-up below tests both
that case and an active byte-layout contract.
Initial full sm_87/util attempts stop at standard-header extern declarations,
using both TinyCC and GHC/MinGW preprocessing. For the complete sm_87 trial
above, project-header amalgamation and public Clang 18 Linux preprocessing
resolve that input obstacle; the translator itself is unchanged. The packed
layout failure still prevents an unchanged project-wide rollout.

### Packing: a demonstrated behavior difference and a small repair

The user's distinction between stricter layout and wrong behavior is useful.
A control writes and reads every possible 16-bit value through a local
`DebugDoorDef`: all **65,536 cases agree**, despite Corrode's extra padding.
There is no need to preserve an old physical layout when nothing observes it.

The actual
[`NMI_ProcessVramWriteQueue`](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/src/sm_80.c#L478)
does observe it. It overlays a record on `g_ram + 0xd0`, advances by **seven
bytes**, and writes its terminator at a byte offset. Corrode expands nested
`LongPtr` from 3 to 4 bytes and `VramWriteEntry` from 7 to 8, moving its
destination field from offset 5 to 6. Executing the actual translated consumer
on the same synthetic seven-byte transfer produces **destination 0x0058 where
the C implementation writes 0x5880**. This is a different register operation,
not a conclusion drawn solely from `sizeof`.

The repair is small for this unit: change the relevant records from
`#[repr(C)]` to `#[repr(C, packed)]`, including the nested record, without
altering generated function bodies. Original C, unmodified Corrode and repaired
Corrode all compile as optimized DLLs. Across **1,056 constructed queues**
(0..32 entries), compare all 128 KiB RAM and ordered register operations:
unmodified Corrode differs in **1,024**, while the repaired output matches
**all 1,056**. Empty queues account for the 32 original matches. The local-field
control also matches in all three builds.

This tests the real consumer with recorded register operations and instrumented
scrolling calls. It does not run actual DMA/PPU processing, ROM access or a
whole game. Other packed records and references to unaligned fields still need
checks; this is a specific output repair, not a generic verified packing pass.
Current RAM/ROM byte contracts must be preserved or explicitly decoded at an
engine boundary. Native internal game objects need not retain historical
packing. Corrode's omission is repairable and does not invalidate its useful
results elsewhere. Artifacts and reproduction:
TEMP/sm-transpiler-tailoring/layout-use/REPORT.md and build_test.py.

### Cpp2Rust: built and tested on the original C

Built pinned
[Cpp2Rust 193a66a8](https://github.com/Cpp2Rust/cpp2rust/tree/193a66a8b8f172816e472c0f47a54b1714b7f24c)
with portable Clang/LLVM 23.1.3, CMake 4.4.4 and Ninja 1.13.2. A MinGW link
adjustment uses shared LLVM without duplicate static LLVM symbols. A DLL
bridge invokes the original translator because this host blocks fresh EXEs.
Neither adaptation changes an AST conversion or emission algorithm.

The full rule-preprocessing pipeline pulls in a pinned rustc-dev toolchain and
many library integrations. The executed trial instead materializes a small
builtin-type rule pack from the upstream C/Rust type and initializer mappings.
This is an explicit fixture configuration: the complete official installation
and its standard-library expression substitutions were not tested. The real
transfer and queue operations use no C library APIs; complete-bank results
may differ under a full library rule pack.

All outputs use the installed rustfmt's default 100-column width. Counts
include emitted types/helpers, with test dependencies and shared runtime
identified separately:

| Complete algorithm input | C2Rust | Corrode | Cpp2Rust default/refcount | Cpp2Rust unsafe |
| --- | ---: | ---: | ---: | ---: |
| Original horizontal/vertical scrolling DMA | 443 | 256 | 451 | 496 |
| Original row/column tile streaming | 592 | 467 | 861 | 651 |
| Shared tile-streaming algorithm, alternative to preceding row | 344 | 277 | 399 | 360 |

The disjoint original units total **1,035 C2Rust, 723 Corrode, 1,312 Cpp2Rust
default and 1,147 Cpp2Rust unsafe lines**. Neither Cpp2Rust mode reduces emitted
source here. Sharing the C tile algorithm still helps substantially in both
modes; that is a separate gain from changing translators. Cpp2Rust omits
externally defined RAM/functions, which the component harness supplies; these
omissions are not a free maintained-source reduction.

All six original/shared transfer outputs compile as optimized DLLs with their
typed test dependencies and unchanged generated game function bodies.
**6,250 scrolling cases** pass against native C/C2Rust in both modes, comparing
all 128 KiB RAM and **358,220 ordered register writes**. **3,076 tile-streaming
cases** pass across eight original/shared C, C2Rust and Cpp2Rust binaries,
covering both layers/axes, four flip orientations, wrapping positions and
Mode 7 skipping. These constructed checks record register calls; actual
PPU/DMA processing and full-game history are outside their scope.

The complete-bank controls also test a materially larger input:

| Entire emitted bank | C2Rust | Cpp2Rust default | Cpp2Rust unsafe |
| --- | ---: | ---: | ---: |
| Animated tiles, sm_87 | 679 | 5,233 | 3,977 |
| Samus, sm_90 | 17,789 | No output within 60-second bound | 24,680 |

Each row uses identical preprocessed public C for its compilers. Cpp2Rust emits
many extra unused header declarations, which should be shared/pruned during
integration rather than charged repeatedly. Nevertheless the difference is
not solely header cost: the same **34 sm_87 functions** occupy **429 C2Rust /
535 default / 472 unsafe lines**, and the same **449 sm_90 functions** occupy
**15,774 C2Rust / 20,122 unsafe lines**. These function-span counts include
signatures and are diagnostic breakdowns, not complete replacement prices.
The banks have no complete compiled/integrated runtime validation; the sm_90
default timeout is only a bounded attempt, not proof it cannot finish.

Cpp2Rust's default representation confirms why native Rust size alone cannot
decide correctness. Its
[record emitter](https://github.com/Cpp2Rust/cpp2rust/blob/193a66a8b8f172816e472c0f47a54b1714b7f24c/cpp2rust/converter/models/converter_refcount.cpp#L730-L765)
records C byte sizes and offsets, and the runtime mediates pointer/field access.
In actual queue execution its Rust records have physical sizes **4/8/12**,
while translated C `sizeof` correctly returns **3/7/11**. All **56 queue cases**
match native C's complete 128 KiB RAM and register trace. Its unsafe backend
returns 4/8/12 and fails all **48 nonempty cases**. Preserving packed attributes
fixes all 56 without changing its generated function bodies. The 65,536 local
field controls agree in each mode. These are actual optimized DLL comparisons
of the real consumer with translated test dependencies, not a layout-only
inference or a full-game replay.

The real runtime and macros compile under a Windows fixture profile. It gates
Unix I/O/descriptor/compatibility/formatting modules and unused library shims;
the necessary map/iterator shims remain. Every compiled pointer, byte, callback
and macro implementation is byte-identical to upstream; only the module root
selects this profile. Generated code drops an unused unconditional Unix import.
Actual dependency build scripts run through DLL entry wrappers to work around
the host's executable restriction. This is not a full Windows runtime port.

Support is a **one-time shared dependency cost**, shown separately from game
output: runtime source **7,623 lines**, macro implementation **1,434**, and all
library shims **2,207**; total **11,264** if vendored in full. The compiled
fixture source includes **9,739 lines**, including inline tests. Third-party
dependencies, Rust std and external test scaffolding are excluded. Do not
multiply this cost by the number of banks or confuse it with generated game
logic. Code-volume results already favor C2Rust/Corrode before adding support.

Artifacts: TEMP/sm-cpp2rust-experiment/REPORT.md, outputs/, queue-validation/,
runtime-manual/REPORT.md, source-counts.json and portability.patch. Exact
inputs, compiler/formatter logs, rule configuration, source hashes and scripts
remain available. Only public C was sent for Linux preprocessing and the
C2Rust comparator; Cpp2Rust ran locally. No maintained source was regenerated.

### Other available approaches, not executed here

[Intel Labs IDEAS](https://github.com/IntelLabs/IDEAS#use-with-locally-hosted-models)
can translate raw C through an LLM and supports a local vLLM endpoint. It is
not paid-key-only, but needs a configured model service and its documented
Ubuntu/CMake/Clang workflow. [Yale PROCTOR's translation harness](https://github.com/Yale-PROCTOR/llm_translation)
also has raw-C and C2Rust-cleanup modes with build/test feedback; its documented
providers need API credentials. Neither pipeline was run. Both require a
separate migration/equivalence experiment rather than a compiler substitution.

Decision: Corrode is an independent comparison of emission rules and a viable
candidate for further trials with explicit layout handling. Its tested packing
omission is a fixable boundary issue. Prioritize measured output and behavior
over accepting or rejecting a translator by its declared goals. The Cpp2Rust
trial demonstrates a working alternative byte representation, but no selected
code-volume advantage. CRUST/Sonder do not justify further production
trials on these results. Artifacts, source/input/output hashes, commands and
primary-source findings: TEMP/sm-transpiler-tailoring/alternatives/REPORT.md.
