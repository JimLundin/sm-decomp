# Palette effect preinstructions
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate palette-effect preinstructions and continue full palette frame closure.
Preserve bank-8D WRAM aliases, unsigned height tests, boss-state and intro
transitions. Actual-C full-WRAM oracle, live admission and scoped cleanup required.

## Work evidence

Seven native palette-effect preinstructions implemented: no-op, intro page,
enemy-health deletion, both unsigned height switches, miniboss deletion and
aliased flag deletion. All 392 actual-C full-WRAM cases pass:
out/port/palette-effect-preinstructions/20261011T091537.086837Z/verdict.json.
Strict Clippy and animated tile tests pass. Cleanup ordinary zero bytes;
post-dedup authentication recorded below. Heat preinstruction, palette instruction
interpreter/frame closure and live verification remain pending. Existing animated
frame replay session 25580 remains active; no restart or ownership claim.

Dedup reclaimed 130023424 bytes; all 392 cases authenticate after cleanup.
No implementer worktree created. Ticket remains claimed for remaining closure.

Heat preinstruction translated with fractional 32-bit damage wrap, suit immunity,
eight-frame sound cadence/health threshold and suit/index-dependent ROM lists.
All 512 actual-C full-WRAM cases pass:
out/port/palette-effect-heat/20261011T091645.307470Z/verdict.json.
Four animated tile tests and strict Clippy pass. Palette instruction/frame closure
and live verification remain pending; earlier preinstruction oracle is now a
historical module seal. Existing animated-frame replay session 25580 remains active.

Heat oracle ordinary cleanup zero bytes; dedup reclaimed 142082048 bytes;
post-cleanup authentication passes all 512 cases. No implementer worktree created.

Implemented palette_instruction for all 19 native CallPalInstr opcode paths,
including pair-valued return, color-cursor wrap, wait/list persistence, slot
operations, timer byte writes, audio and heat index. Independent pair/color-cursor
and high-byte preservation test added; strict Clippy passes. Actual-C pair-return
oracle bridge still required; do not claim native equivalence or live ownership.
Palette frame closure remains pending. Existing animated replay session 25580
continues independently with frozen binary.

Native PairU16 return oracle support added as explicit pair_u16_return contract,
requiring two uint16 arguments and packed uint32 comparison. All 1368 actual-C
full-WRAM/two-word return cases across all 19 palette instructions pass:
out/port/palette-instructions/20261011T091856.464411Z/verdict.json.
Five independent tests and strict Clippy pass. Live bridge still needs native pair
return support before admitting these direct wrappers; palette frame closure
remains pending. Existing animated replay session 25580 remains active.

Palette instruction cleanup ordinary zero bytes; dedup reclaimed 445644800
bytes. All 1368 cases authenticate post-cleanup. No implementer worktree created.

Palette frame processing, descending handler and enable/disable/clear controls
are now Rust. Preserved preinstruction-before-timer ordering, timer underflow,
two-phase pair-valued interpreter, color cursor skips and wait-pointer semantics.
Reproducible native-ROM fixtures: tools/port_palette_frame_fixtures.py.
All 268 actual-C full-WRAM cases pass:
out/port/palette-effect-frames/20261011T092153.466077Z/verdict.json.
All port tests and strict Clippy pass, including independent color-skip/wait test.
Cleanup ordinary zero bytes; dedup reclaimed 64749568 bytes; all 268 cases
authenticate afterward. Free disk 270807912448 bytes. Retained sealed binaries,
ROM and pinned source dependencies; no implementer worktree created.

First live admission used the full-C oracle verdict with --family-verdict,
which expects port_family.py provenance and correctly rejected before execution.
Rejected diagnostic run retained; both cleanup modes reclaimed zero bytes.
Generating the proper family admission seal next; live ownership remains pending.

Proper family admission also passes all 268 cases:
out/port/palette-effect-frame-live/20261011T092258.434801Z/verdict.json.
Cleanup ordinary zero; dedup reclaimed 64749568 bytes; all cases authenticate.
Four fresh full 129712-input C/mixed palette replay started via session 99942
under out/port/palette-effect-frame-replay. Final result, ownership counts and
finished-run cleanup remain pending. Existing animated replay session 25580
continues separately; do not restart either run.

Current-source family seal refresh passes 268 cases:
out/port/palette-effect-frame-live/20261011T093431.463226Z/verdict.json.
Cleanup ordinary zero; dedup 64749568 bytes; authentication passes. Eight retained
C handler entry samples from active replay contain native IDs E1A0/E1A4;
seven show four changed palette bytes and all show timer progression. This
demonstrates active script work in C samples; final exclusive Rust parity remains
pending. Expanded fixtures now include these two native headers; 396-case family
refresh running session 18081. Live replay session 99942 remains frozen to its
original source/dependency snapshot and is in mixed-0. No restart.

Expanded actual-ROM frame family refresh passes all 396 cases:
out/port/palette-effect-frame-live/20261011T093556.133187Z/verdict.json.
Includes native live headers E1A0/E1A4 across every slot and timer boundary.
Cleanup ordinary zero; dedup reclaimed 95158272 bytes; all cases authenticate
afterward. Free disk 269036613632 bytes. Full port suite, strict Clippy and
live-runner tests pass (16 tests, one skipped). Live replay session 99942 still
active; cleanup must wait for completion. Ticket stays claimed.

## Completion evidence

Palette frame live replay finished successfully:
out/port/palette-effect-frame-replay/20261011T092352.241110Z/verdict.json.
Four fresh full 129712-input processes match all 1038 state/effect artifacts.
Each mixed subject exclusively executes 114717 Rust PaletteFxHandler calls,
23 enables, 15 disables and 95 clears. Native C has 9905 direct per-object calls;
Rust handler invokes process_palette internally, so its direct wrapper count is
zero. Active native scripts and color changes are demonstrated in retained
samples. Measured CPU fallback zero; original-ROM parity not tested.

All 19 instructions and eight preinstructions have actual-C offline coverage;
current-source frame seal passes 396 cases including live native headers. Direct
pair-return wrappers remain offline only. Live binary predates later unrelated
room/projectile changes; immutable binary evidence and current oracle separate.
Full port tests, strict Clippy, live harness tests and diff whitespace pass.

Finished replay ordinary cleanup zero; dedup reclaimed 947740212 bytes, plus
4769723648 bytes automatically shared by runner. tools/port_authenticate_live.py
authenticates all four 1038-artifact inventories, live binary and ROM views;
report retained at replay/storage-authentication.json. Free disk 269968474112
bytes. Retained sealed source, pinned ROM and predecessor dependencies remain;
no implementer worktree created. Integration branch decomp-port-integration.
