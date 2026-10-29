# External review

Living independent review, commissioned by the user. This file holds current
direction; [frozen qualification history](https://github.com/JimLundin/sm-decomp/blob/170bc5351c64f0846bbcb7b4714064a7230db153/docs/REVIEW.md)
retains earlier results and contracts. Retired issue receipts remain in the
primary's owned archive, located by their original comment IDs in that history.

## Milestone plan (2026-10-29)

Approved by Jim on 2026-10-29; it supersedes the 2026-10-27 plan and its pacing
advice. PLAN remains authoritative for the Fast loop, Git, resources and commit
rules; the accepted architecture, ADRs and ENGINE_STATE_MIGRATION are unchanged.
Primary root 01a17ecd stays the sole Git writer; Linux stays stopped.

### Direction (Jim, 2026-10-29)

We are building the Super Metroid engine: a purpose-built, compact Rust engine
that plays the original game faithfully. It is as modular as Map Rando and
content replacement need, and no more: swappable room/door graphs, textures,
tilesets, palettes, sprites and sounds through `sm_data`, not a general engine.
Judge every change by whether it moves the code toward that engine and makes
the code base smaller and clearer.

**Progress means behaviour crossing into a target package** (`sm_data`,
`sm_game`, `sm_runtime`, `sm_app`, `sm_tools`) that the compiler keeps separate
from the emulator. Code that exists only to keep the original runtime and oracle
working is **compat**: it shrinks and gets no new features. Owners built inside
the RAM-compatible runtime stay inside the backend cycle and are not the goal.

Measured at `2c2928e` with `python3 tools/cycle_report.py`: the 376 `game/`
files (118,152 lines) are, apart from 44 files (10,850 lines), tied into one
dependency cycle with `game_state`, `ram_map`, `rtl`, `cpu_infra` and `snes/`. The couplings holding the cycle together are the `Game` handle (145 files),
`ram_map` addresses (102 files), `rom_table_word` ROM reads (369 sites) and PPU
register writes (95 sites). Bank retirement stopped after 2026-10-27 12:47, and
the checkpoint source fingerprint (85 files, 46,745 lines) invalidated every
checkpoint on 9 of the first 11 commits of 2026-10-29.

### Phases

Each phase closes on evidence, not a date. Lanes: A tooling/verification,
M seams and cycle cuts, B bank retirement.

0. **Verification loop (A, now; nothing else lands first).** Fingerprint the
   Checkpoint's serialised layout (`State` field names and types) instead of
   source text; keep the round-trip/extra-bytes/input-prefix rejections. Refresh
   the `f158` seed asset; split the checkpoint shard to at most ~60 minutes. Run
   `tools/cycle_report.py` in CI. *Exit:* two consecutive green scheduled full-
   corpus runs.
1. **Package graph and seams (M).** Add `sm_data`, `sm_game` and `sm_runtime`
   to the workspace; `sm` becomes compat plus oracle tools. Put the CONTRACTS
   types in place: `TickOutput`, a coarse `Scene`, a `Cue` record in `sm_data`.
   Route `game/`'s 4 audio backend calls (`write_sound_port`, `upload_bank`)
   through Cues; a compat adapter maps each Cue back to `ApuQueue` port bytes so
   oracle input is unchanged. Move the 44 cycle-free files into `sm_game`; CI
   fails if `sm_data` or `sm_game` depends on compat. *Exit:* both build headless
   without compat, the corpus is green, and the Cue trace reproduces the port
   trace on every corpus row.
2. **Cut the backend cycle (M and B, rolling).** Cheapest coupling first:
   (a) ROM reads become `sm_data` tables built by an `sm_tools` importer, checked
   byte-equal to the ROM at load and keyed by stable room/door IDs, which is what
   Map Rando's room graph needs; (b) PPU register writes pass through one
   display-state type feeding `sm_game::extraction` and the Scene; (c) RAM state
   moves to typed domain modules in CONTRACTS order (progress, rooms, player,
   actors) under the existing interval rules; (d) B resumes sm_8b, sm_a3,
   sm_a5..sm_a9 and sm_aa, retiring straight into `sm_game` domain modules.
   *Exit per domain:* it compiles in `sm_game` and its corpus rows pass.
3. **Gate 1 vertical slice.** Morph room 9E9F: `Session::open` from prepared
   content, the full Morph1200 inputs, `sm_runtime` renderer and sequencer, a
   minimal `sm_app`, headless replay in `sm_tools`, compared to the oracle
   through semantic projections. *Exit:* VALIDATION Gate 1 evidence.
4. **Gates 2–5**, then Map Rando import through the same `sm_data` IDs.

SPC700 CPU and APU bus emulation are frozen compat (unsafe permitted, outside
the unsafe goal). The N-SPC sequencer (`spc_player.rs`) and S-DSP are the seed
of `sm_runtime` audio; continue them only once separated from emulator types,
with verification growth no larger than the production change it covers.

### Current publication and acceptance

API source da718649 / codec c97578fc2a1bd899 removes the obsolete six-argument snapshot wrapper and five-tuple accessor after all14 verifier consumers and3 production restore tuples use the existing owned capture/selected restore paths. Captured machine/player identities precede the disjoint borrow; argument, failure, physical audio, queue, SRAM and reset/adoption phases remain preserved.
Exact1579-check review, strict library/DLL/frontend/format and all146 existing units pass. The unchanged146 safe snapshot phases/native adapters retain their source proof; original native9 and separate frontend-only desktop transfer through unchanged inputs, with no new native compilation.
Actual originals65643/95726 pass base13 groups/27 rows/199,063 and supplemental7 rows/25,845 comparisons, with no failures. Their disjoint union is34 rows/224,908/0 and required28/44, including full Morph1200, full intro8780 and BOTH independent Kago14350. Genuinec975 initialized329/resume5/save55650/FIR110076-to110077/reserve147273 and door13600 continuations pass; four durable rows bind fresh base captures.
Native/snapshot allocations remain disjoint, live, exclusive, stopped/serialized and non-reentrant. Rc/UnsafeCell/native interfaces, invalid/topology prefixes, late native fallback, foreign/history/rebinding aliases and physical audio/backend dependencies remain scoped. Comparison snapshots stay distinct from resumable checkpoints; payload/schema layout is unchanged.
Imported odd/>=8 aliases, arbitrary callback/rebinding, physical device/backend/full Session/boot-to-credits and size acceptance remain open. Earlier controls and146 monitor-only EXIT1 retain their exact original attribution; no prior execution or checkpoint is restamped.
The [qualified146 record](https://github.com/JimLundin/sm-decomp/blob/0ec67ed08e303adc0078c84ee54cb961df3563ba/docs/PLAN.md#current-work-and-limits) preserves prior source/compiler/native/runtime attribution; the primary updates the sole #6 issue-body report after every push.

Completed room/renderer/message/Scene/delivery reports are in the
[frozen history](https://github.com/JimLundin/sm-decomp/blob/170bc5351c64f0846bbcb7b4714064a7230db153/docs/REVIEW.md#current-publication-and-acceptance):

| Cohort | Qualified source / DLL | Retained evidence |
| --- | --- | --- |
| Room, 0fed8911 | 9787b619 / composition04–05 | 38 cases; 21,481 comparisons / 0; exact transfer/failure lineage |
| Shared renderer | c942fe04 / b01c2561 | 24 units; original 17 rows / 26,638 / 0; independent pixel witnesses |
| Message composition | 5787 / 8d58 | 26 units; 26 rows / 51,456 / 0; prepared display and content identity |
| Scene/native sharing | edbe402d / d02d370b | 46 units; original 26 rows / 199,063 / 0; atmospheric failure lineage |
| Delivery/settlement | dfdd6120 / 7fa1a335 | 48 units; original 26 rows / 199,063 / 0; register/routing/counter witnesses |

Frozen reports retain full hashes, native/cache/compiler/profile/source/codec/input/
loader pins, donor/caller costs, exact deltas and failed or unexecuted variants.
Read the relevant report and original owned receipts when touching its owner.
Catalog/schema/validation/adapters and verification growth all count; moves and
formatter joins earn no savings. Windows selected ROM/search paths retain ASCII
policy; pack/verifier paths retain Rust handling. Absolute ASCII ROM paths work
independently of cwd; --config skips relative-directory search; no-room-pack flow
adds no guard. Native dependencies stay disclosed until actual consumers retire.

### Shared renderer contracts

Preserve these interfaces and source contracts:

- BG wraps its base before column/plane additions. Reads reach801e; retain 31
  native tail words, port/padding/cgram aliases. No final-address mask or zeros.
- Sprite evaluation writes directly through a narrow mutable 256-word native
  workspace. Completed pixels survive explicit errors and Rust unwinding.
  Success publishes sticky overflow before lineHasSprites; failure keeps prior
  flags/lineHasSprites. No production panic wrapper or whole-Ppu borrow.
- Keep brightness/backdrop/forcedblank/line0 mutation order. C acquires gated
  signed sprite aliases from surrounding native storage. Per-pixel borrows end
  before the original four-byte native commit. Preserve failures and aliases.
- Pure color begin/needs-subscreen/finish stages retain native/reference policy,
  including pseudo-hires/modes5/6 subpixels without math. EXTBG/mosaic fallback,
  register/latch/snapshot/reference/device dependencies remain disclosed.
- Keep whole ppu.rs in checkpoint contributors and add all three rendering
  leaves, plus actual library/literal registrations, through the sole integrator.
  New codec requires genuine initialized/required starts/continuation and complete
  source/native/ROM/input guards. Never restamp 05 checkpoints.

The safe core and actual prepared consumer must compile without oldsm/SNES/native/
device/ROM dependencies. The direct metadata entry reaches only the three frozen
leaves and std/core; whole sm --no-default-features is not that proof. Retain the
original Linux detached proof; no new Linux job. Source authors remain source-only.
Release original handles before changed-entry reuse.

**Independent pixels are required.** Oracle and mine movie branches both use the
shared renderer. Their pixel/RAM agreement alone cannot establish fidelity.
The existing verifier now supplies independent Mode1/Mode7 pixels, priority/masks/
color/scanline/affine, alias/failure and actual EXTBG/mosaic boundary witnesses.
Broader variants and checked-overflow mutation sensitivity remain scoped/unproven.
PLAN affected rows/four smokes/full Morph1200 retain their original proofs.

### Message composition contracts

Reuse the unchanged 295-line dialogue runner and frozen V03 renderer. One pure
message-composition implementation serves the native writer and an actual prepared
display consumer. Remove replaced borders/text/button/selection/radius/scanline
bodies in the same cohort. Keep lazy draw/drawing/fresh modify reads, scratch/text/
DAS/word/bound failure order, do-until zero/odd/wrapped behavior, letter priority,
HDMA repeat/count and palette/saved-display restore order. Preserve live request
clearing, parent/child suspension, input filtering, reset/load and checkpoints.

Prepared starts require complete validated saved display, message/font/media IDs
and revisions; missing data rejects. Do not infer a message or initial display
from 9E9F. Native writable RAM and separately owned prepared display do not become
competing authorities. display_refresh::wait's0782 request/acknowledgment structure
is retained; no new clock, loop-to-completion or invented 60Hz policy. Audio/device
delivery and full Session/Scene extraction remain explicit dependencies.

### Full owned playable slice remains open

The four accepted interface decisions below refine the full Morph9E9F proposal.
The six-role +8,900..21,100 production estimate is not blanket authorization.
Preserve all eight full-slice proof groups and original full 1,200-frame Morph.

1. Tools construct G's complete sealed validated start; D owns immutable envelope/
   IDs. Session never runs CPU normalization, opens a RAM dump or guesses fields.
2. Initial observation is Tick 0; input index i commits Tick(i+1). Preserve actual
   normal/dialogue suspension, source sampling and late NMI/HDMA/IRQ settlement.
3. G checkpoints semantic audio intent/delays; A mixer/voices/echo and C epoch/
   acceptance/outbox stay outside. Opening never resends an accepted fanfare batch.
4. Actual portable/default/headless edges exclude oldsm/backend/native/reference/
   devices. Reuse actual pure leaves once; package moves alone do not establish it.

The [retained seed/trace facts](https://github.com/JimLundin/sm-decomp/blob/170bc5351c64f0846bbcb7b4714064a7230db153/docs/REVIEW.md#full-owned-playable-slice-20-remains-open)
use normalized WRAM06b4d253, not unavailable raw89960823. Room 9E9F / state 9EB1,
equipment 0 and next input 18109 are observed. The original full 1,200 trace stays
in 9E9F; award 590 / completion 977 retain source eba0 / DLL a1c0 / codec 3ac provenance.
These facts do not establish complete initial Scene/audio or a committed Scenario.
Exact cadence/reference policy and G's complete field schema still need evidence.
Use the completed original reader; do not start duplicate artifact surveys.

Scene extraction still needs immediate atmospheric/OAM/Samus aliases, NMI544-byte
display commitment and later effects. Terrain/observation/gamma surveys remain
scoped constraints; no standalone cache, dispatch facade or getter campaign is
released. ENGINE_STATE_MIGRATION and retained qualified history hold their actual
foreign-transfer/reset/adoption/failure contracts.

### Pace, readable handoffs and publication

Keep useful Windows owners supplied with bounded core work and a concrete ready
follow-up: exact owner/files/source/budget/rows/handles and next recipient. Prepare
successor commands and pinned inputs before upstream READY; collect/release the
prior family, sample shared resources and immediately launch pregranted successors.
Wake completed workers with followup_task; send_message does not activate them.
Preserve live jobs/frozen sources and send minimal actual results before prose.
Drain qualified code to main at the next safe boundary; documentation ACKs and
CI observation never gate publication or independent implementation.

The primary keeps the sole report in Jim's #6 issue body current once a day
(see the metrics below) and reads #6 at every work boundary. Mark pending checks honestly; keep exact proof in owned records.
Archive answered feedback and duplicate report comments before authorized retirement
after verified body publication. Task issues remain retired; direct handoffs continue during CI.

Measure each cycle's actual source/review/build/capture/replay/publication time,
queue/handoff gaps and net deltas. Preserve original handles and exact attribution;
cached refs/mailbox observations do not prove push time. Correct measured stalls;
do not repeat accepted campaigns for timing or claim unmeasured speedups.
Pages, progress images, treemaps and the codebase map are retired. Keep commit
messages concise under PLAN; retain exact metrics/proofs separately. Docs updates
travel with code. Current qualification requires compiler warnings as errors on
every real Rust path; historical warning-bearing results are not strict-lint proof.
Preserve other workers' source/caches/locks and all fidelity gates.

### Size, metrics and final acceptance

The engine should end compact: the 100,000-line goal applies to the target
packages (`sm_data`, `sm_game`, `sm_runtime`, `sm_app`) at completion. Compat
and verification are reported separately and do not count against it. Moving
code into a package is neutral, not growth; moves, formatting and relocation to
verification are still not savings, and every adapter counts once.

The #6 report carries these measures daily, not per commit:

| Measure | 2c2928e | Direction |
| --- | --- | --- |
| Lines in target packages | 0 | up |
| Lines outside the backend cycle (`tools/cycle_report.py`) | 10,850 | up |
| Legacy bank lines | 17,822 | 0 |
| Compat lines | about 12,900 Rust plus LakeSnes C | down |
| Unsafe fns/blocks/impls in `game/` (excluding compat) | 955 | 0 |
| Scheduled full corpus | failing | green |
| Validation gates passed | 0 of 6 | Gate 1 at phase 3 |

Full owned Session, independent renderer/audio, boot-to-credits fidelity and Map
Rando remain open; the accepted architecture and VALIDATION gates define them.
