# Reducing transpiled Rust: tools and process

Research date: 2026-10-20. Immediate objective: reduce maintained source,
preserving behavior. This note records primary-source research, scanner trials
and isolated measured experiments. Their scopes are explicit; they are not an
exhaustive discovery result or a whole-game deletion forecast.

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
