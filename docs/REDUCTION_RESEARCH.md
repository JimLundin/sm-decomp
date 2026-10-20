# Reducing transpiled Rust: tools and process

Research date: 2026-10-20. Immediate objective: reduce maintained source,
preserving behavior. This note records primary-source research, scanner trials
and isolated measured experiments. Their scopes are explicit; they are not an
exhaustive discovery result or a whole-game deletion forecast.

Latest result: the broader original-C changes plus a tested numeric emission
pass reduce the complete measured gameplay/support cohort from **229,348 to
185,133 lines (-44,215, 19.3%)**, under one formatter. The original complete
54-file Rust crate independently loses **35,000 lines** from numeric emission
alone, or **59,227 (278,936 -> 219,709)** when composed with shared declarations
and native gameplay calls. These overlapping candidates are generated-code
experiments, not current-main deletions. The
final sections distinguish algorithm sharing, emitter policy and independently
tested alternative transpilers.

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

## Independent transpilers: executed comparisons

Three independent translators were built from pinned source and run locally
against public original C. This answers whether another translator can produce
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

Corrode also has a concrete correctness blocker. Its
[declaration handling](https://github.com/jameysharp/corrode/blob/34053342c2f1ca04f23ad94d67057f14e74d9fb9/src/Language/Rust/Corrode/C.md)
leaves attributes unprocessed. Translating the actual pragma-packed
`DebugDoorDef` from the original header gives **12 bytes / field offset 10**,
where the original C gives **11 bytes / offset 9**. Both compile; executed
measurement functions expose the difference. A global switch requires fixing
packing and checking actual layouts/accesses, not just compiling the output.
Initial full sm_87/util attempts stop at standard-header extern declarations,
using both TinyCC and GHC/MinGW preprocessing. For the complete sm_87 trial
above, project-header amalgamation and public Clang 18 Linux preprocessing
resolve that input obstacle; the translator itself is unchanged. The packed
layout failure still prevents an unchanged project-wide rollout.

### Other available approaches, not executed here

[Cpp2Rust](https://github.com/Cpp2Rust/cpp2rust) is a separate Clang-based
translator with a [PLDI 2026 paper](https://web.ist.utl.pt/nuno.lopes/pubs/cpp2rust-pldi26.pdf).
Its documented build needs Clang 23 development libraries and CMake/Ninja;
no release asset or Compiler Explorer instance was available. Its default
backend uses checked reference-counted pointers through `libcc2rs`; it also
offers `--model=unsafe`. A useful trial must include that support code and test
RAM/pointer compatibility. It is the next deterministic candidate, not a
demonstrated replacement from this experiment.

[Intel Labs IDEAS](https://github.com/IntelLabs/IDEAS#use-with-locally-hosted-models)
can translate raw C through an LLM and supports a local vLLM endpoint. It is
not paid-key-only, but needs a configured model service and its documented
Ubuntu/CMake/Clang workflow. [Yale PROCTOR's translation harness](https://github.com/Yale-PROCTOR/llm_translation)
also has raw-C and C2Rust-cleanup modes with build/test feedback; its documented
providers need API credentials. Neither pipeline was run. Both require a
separate migration/equivalence experiment rather than a compiler substitution.

Decision: use Corrode as an independent comparison of emission rules on
compatible units. Prioritize the already measured C2Rust numeric policy and
complete C algorithm sharing; the tested Corrode layout defect blocks an
unchanged project-wide rollout. CRUST/Sonder do not justify further production
trials on these results. Artifacts, source/input/output hashes, commands and
primary-source findings: TEMP/sm-transpiler-tailoring/alternatives/REPORT.md.
