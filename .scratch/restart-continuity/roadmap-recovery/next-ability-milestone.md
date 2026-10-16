# Conditional successor: use Morph Ball and collect the first missiles

Status: recommendation consumed; ticket17 claimed at15810c7
Blocked by: None (16 resolved; cartridge corroboration complete)
Triage: ready-for-agent
Owner: watch/synthesis session `01a144f0-bd8c-7ec1-b86c-3a4065c33302`
Source owner: reserve `01a144fc-f095-7d71-aa00-10358e7f716e`, note released
Date: 2026-10-16 14:17 UTC

After Morph Ball is actually acquired and ticket16 closes, recommend one next
milestone: demonstrate ordinary-input Morph movement, traverse Construction
Zone, reveal and collect the first Chozo missile tank, and demonstrate resumed
control after message2. Preserve16's eventual successful complete input route,
then append one bounded tail through the existing Game/input mechanism. The
integrator selects/claims the successor and its finite step/time limits.
Author03 supplies the proven prefix after strict replay and ticket16 closeout.
No successor runtime cost is available yet.

At14:24 UTC the integrator's final16 handoff identified the narrow source-data
gap. Continuity assigned reserve one10-minute additive cartridge check at
`../reserve/first-missile-rom-corroboration.md`; no duplicate research or17
claim/run was assigned at that research handoff. The integrator subsequently
claimed17 at15810c7 with exact23036-pad prefix,8192 tail,31228 total and90seconds;
its ticket preserves reserve's independent data ownership. The returned
[cartridge note](../reserve/first-missile-rom-corroboration.md) completed within
the bound and closes the selected source-data gap. Geometry,
traversal and shot timing remain bounded authoring questions.

## Source basis and exact limits

The completed [reserve source note](../reserve/next-after-morph.md) owns the
research and exact citations/hashes. Its recovered C pin578f90b3 establishes
Morph gating, input mechanics, item persistence and ammo/HUD/message mechanics.
The initial route, placement and pose-table rows came from pinned community
source216f3a1. The completed additive cartridge note now directly corroborates
their room/door chain, A114→8802 population, EF2F block4,7/argument22,
EE52/E47C header and standing/crouched fresh-Down rows against the original ROM
SHA25612b77c4b…8cc2ca72. No selected data conflict was found. Its one corrected
wrong-bank diagnostic offset is documented as an address mistake, not a data
conflict. Exact script interpretation remains with material; raw E47C bytes
were supplied early. No duplicate audit or broader PLM research is needed.

Proposed route: Morph9E9F → Construction9F11 → First MissileA107, all area1.
Community data identifies door8EAA east from Morph, then8EDA through
Construction's lower west exit. The source cap anchors are(2016,608) and
(16,352), not Samus activation tolerances. The upper-right red-door route is
outside this checkpoint. First Missile's Chozo item is EF2F at block(4,7),
argument0022; reveal its shell with ordinary uncharged fire, then touch the
visible item. Shot position, tunnel geometry and exact input timing remain
native authoring questions.

## Material result

Use two fresh Down edges separated by release, observing standing-to-crouch
then Morph state. Down is native0020/SNES0400. Demonstrate consumed directional
input and displacement while the observed pose/movement type is Morph Ball;
inventory alone does not establish use. Source pose/movement observations are
WRAM0A1C/0A1F, with ground Morph movement type4.

Retain the actual starting ammo/capacity/item bit from the successful16 prefix.
Require item argument0022 clear-to-set at byteD874 bit04 and acquisition adding5
to words09C6 (current missiles) and09C8 (capacity). With zero initial ammo and
no missile firing, both should become5. Shell reveal alone is not collection.
Observe message2 completion and clear freeze/continuation, then positive-health
normal area1/A107/state8 with material response to newly consumed ordinary
input. Message09 behavior from16 does not prove message2's dismissal behavior.
Freeze a successful complete input sequence and replay without policy using
the existing records. Preserve a failed bounded attempt and fix only its
first reached dependency.

## Known gameplay ownership

The accumulated Rust projectile consumer handles admitted ordinary beam fire,
projectile terrain collision/trails/drawing and crosses synchronous C SpawnPLM
for relevant block reactions. Existing Rust scrolling, directional transitions,
room-transition roots and normal camera remain available. Their actual reached
counts still need to be reported; the Ceres-door consumer does not make these
Brinstar door actors Rust-owned.

Samus Morph transition/movement/collision, item/orb PLMs, persistence, ammo/HUD
mutation and message/continuation remain C-owned. The Rust input policy is tool
logic and grants no new gameplay translation ownership. Acquiring missile ammo
does not itself invalidate the ordinary projectile consumer, but firing a
missile creates a different unsupported projectile kind. Missile firing/flight
is outside this proposed pickup checkpoint and must not be implied by it.

The directly exposed next C-to-Rust opportunity is the coherent ordinary item
pickup lifecycle: shell reveal, touch, permanent item bit, inventory effect,
and message-return continuation. That is a future scope/dependency decision,
not a condition to run this progression milestone or a new assignment. This
note adds no original capture, broad verification suite, trace format or
runtime framework.

## Dependency, handoff and cleanup

Ticket16 author03 passed at gameplay/tool source
`c3b6a76e25c1d1dd80fb2a88f5e980414a21e714`:23036 total steps/4964 tail,
23.4255 seconds. All three pickup bits set at22624; message09 finished23022,
then14 consumed polls/12 moving polls supplied16 pixels of motion through23036,
health99 and zeroCPU fallback. Candidate input is
`/home/jim/sm-decomp-restart/restart/out/morph/author-03/inputs.txt`, SHA256
`0a8ab6a50be742c1124aea5b802ae088a3b12cd785b7da54aea6cfc6143ae76a`.
Replay-02 passed23.3259seconds with seven exact input/CSV/snapshot/finalWRAM/PNG
pairs. Ticket16 resolved at316f557; both workers retired and all1053 retained
pre-cleanup hashes plus1173 final hashes authenticated after cleanup. Both preceding failures remain retained.
Root now chooses the successor and its concrete bounds; the completed
synthesis pointer was delivered at14:19 UTC without a crossing start request.
Both reserve notes are now complete/released; reserve is available with no
additional assignment. This synthesis does not duplicate that source work or16 diagnosis.

14:25 UTC17 dispatch checkpoint: material owns only new missile.rs in its
missile-policy worktree; native owns runtime_cli.rs in missile-runtime. Root
owns integration/index and every gameplay run. Current cartridge check remains
was with reserve and completed by14:34 UTC; only necessary pinned-C/script
implementation details belong to material.
No17 native result exists yet. The integrator's concrete contract owns the
implementation; this recommendation adds no separate approval or evidence gate.

Only Markdown was written and the retained source note read. No gameplay,
build, test, capture, worktree or run artifact was created. Cleanup utility is
not applicable; reclaimed bytes0. Source note reports103596990464 bytes free.
All active ownership, failed evidence and warm caches remain preserved.
