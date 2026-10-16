# Retained original shot: next bounded comparison scope

Date: 2026-10-16. Research for integration owner
`01a144ca-27ba-7130-a9e2-a615c7c80e3a`; priority remains with the integrator.
Only this note is owned. No implementation, index, registry or ticket changes;
no builds, tests, replays, captures or authentication were run.

## Usable result

`/home/jim/sm-decomp-restart/restart/out/original/capture-04/` contains **34
complete HandleProjectile pairs and 34 pairs for each drawing root**, over
original frames 8661..8694. All 206 retained CPU records report native mode,
DP=0 and zero interrupts inside the observed call. The first Fire pair accounts
for the other two snapshots. These are inspectable retained observations;
their hashes were not freshly authenticated by this research.

The lowest-surface useful extension is to compare **all 34 captured Handle
invocations as independent original-entry Fixtures**, adding the source-defined
trail storage to the output projection. This exercises flight, trail allocation,
terrain impact, explosion instructions and removal in one existing scenario.
It requires no new original execution. Drawing has equally usable snapshots,
but needs OAM/trail output selection and its own CPU-context eligibility.

This recommendation is not an assignment or new gate. The integrator owns the
Rust runner extension, final source contract and whether to include drawing.

## Exact evidence and observed behavior

All relative artifact paths in this section are under `capture-04/` above.

- `result.toml:1`: status `captured`, 8,850 completed frames, 148,677 events,
  206 snapshots; original equivalence explicitly `not-tested`.
- `completion.txt`: zero active calls, shot created/impact/removed all true.
- `events.csv`: 17 columns; event E is physical line E+1. Pair keys are the
  recorded `call` ID, root kind and entry/exit/continuation events, not a native
  host step or assumed correspondence between video frames and updates.
- Every entry/return E below names full input/output
  `states/state-E.wram` and CPU context `states/state-E.cpu.toml`.
  Their per-file digests are retained in `result.toml`'s artifact map. Exit
  events are stack/return witnesses, **not** separate post-call snapshots.
- `request.toml`: scenario `original-df45-ordinary-shot-v2`; pristine ROM
  `12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`;
  observer `9d8b4d1d366cf9e336b191d88c209ef47edcee30391a827f196f40bb03948de4`;
  authored movie `b6d275b5b68da5c1a7702e6d581b8eec94dd0737d13c408f1e80ed4c7ff81cfb`.

Useful witnesses, read directly from retained entry/return files and ledger:

| Behavior | Original frame, call, entry / exit / return | Observation |
| --- | --- | --- |
| First flight | 8661, Handle 2, 73926 / 73931 / 73932 | Projectile X 141→145; instruction 0x86F3→0x86FB; no trail bytes change in this first update. |
| First visible beam draw | 8662, Player 7, 76192 / 76193 / 76194 | OAM cursor 28→32. First Player call 4 at8661 changes only seven stack bytes, so it alone is inadequate drawing evidence. |
| Trail allocation | 8664, Handle 11, 80569 / 80574 / 80575 | Eight bytes change in the named trail-array range; projectile X153→157. |
| Trail update/draw | 8664, Player 13, 80621 / 80623 / 80624 | OAM cursor28→32 and trail storage changes. |
| Terrain impact | 8676, Handle 47, 107129 / 107136 / 107137 | Type0x8000→0x8700; X208→221; instruction0x86FB→0xA00F; sound2/max6 request12 at107134; trail storage also changes. |
| Explosion drawing | 8676, Bomb 48, 107179 / 107180 / 107181 | Four OAM bytes and cursor0→4 change; full original entry/return retained. |
| Removal | 8694, Handle 101, 146905 / 146906 / 146907 | Count1→0, slot type/instruction/X cleared. |
| Post-removal boundary | 8694, event146958 | `shot-removed` follows dispatch-complete146957; both draw roots have already returned. |

**Impact occurs at frame8676; removal occurs at8694.** The ticket's compressed
“impacts and is removed at frame8694” wording should not be used as the impact
timestamp. Both events are supported separately by the actual ledger.

## Call context and complete inventory

These signatures occur consistently in all retained CPU files for each root.
Raw CPU PC is boundary PC+1 because the callback observes the program fetch.

| Root | Entry PC | Witness PC | Continuation PC | Entry S → return S | DB |
| --- | --- | --- | --- | --- | --- |
| Handle | 90:AECE | 90:AEF2 RTS | 90:DD03 | 0x1FF1→0x1FF3 | 90 |
| DrawBomb | 93:834D | 93:83BE RTL | A0:885D | 0x1FF4→0x1FF7 | A0 |
| DrawPlayer | 93:8254 | 93:82FC RTL | 90:EB42 | 0x1FEF→0x1FF2 | 90 |

All 102 pairs report zero interrupted episodes; entry/return CPU records retain
A/X/Y/P, stack linkage, continuation and saved-status fields. Do not copy Fire's
inherited-PHP eligibility to these ordinary JSR/JSL entries. In particular,
DrawBomb's DB=A0 is observed and differs from the other roots.

Cell syntax below is `call:entry/exit/return`. This is the full bounded pair
inventory, not a sample picked after comparing candidate outputs.

| Frame | Handle | DrawBomb | DrawPlayer |
| --- | --- | --- | --- |
| 8661 | 2:73926/73931/73932 | 3:73975/73976/73977 | 4:73979/73981/73982 |
| 8662 | 5:76140/76145/76146 | 6:76187/76188/76189 | 7:76192/76193/76194 |
| 8663 | 8:78355/78360/78361 | 9:78402/78403/78404 | 10:78407/78408/78409 |
| 8664 | 11:80569/80574/80575 | 12:80616/80617/80618 | 13:80621/80623/80624 |
| 8665 | 14:82782/82787/82788 | 15:82829/82830/82831 | 16:82834/82835/82836 |
| 8666 | 17:84996/85001/85002 | 18:85043/85044/85045 | 19:85048/85049/85050 |
| 8667 | 20:87211/87216/87217 | 21:87258/87259/87260 | 22:87263/87264/87265 |
| 8668 | 23:89425/89430/89431 | 24:89472/89473/89474 | 25:89477/89479/89480 |
| 8669 | 26:91638/91643/91644 | 27:91685/91686/91687 | 28:91690/91691/91692 |
| 8670 | 29:93851/93856/93857 | 30:93898/93899/93900 | 31:93903/93904/93905 |
| 8671 | 32:96064/96069/96070 | 33:96111/96112/96113 | 34:96116/96117/96118 |
| 8672 | 35:98277/98282/98283 | 36:98324/98326/98327 | 37:98329/98331/98332 |
| 8673 | 38:100490/100495/100496 | 39:100537/100538/100539 | 40:100542/100543/100544 |
| 8674 | 41:102703/102708/102709 | 42:102750/102751/102752 | 43:102755/102757/102758 |
| 8675 | 44:104916/104921/104922 | 45:104963/104964/104965 | 46:104968/104969/104970 |
| 8676 | 47:107129/107136/107137 | 48:107179/107180/107181 | 49:107184/107185/107186 |
| 8677 | 50:109345/109346/109347 | 51:109388/109389/109390 | 52:109393/109394/109395 |
| 8678 | 53:111556/111557/111558 | 54:111599/111600/111601 | 55:111604/111605/111606 |
| 8679 | 56:113766/113767/113768 | 57:113809/113810/113811 | 58:113814/113815/113816 |
| 8680 | 59:115975/115976/115977 | 60:116018/116019/116020 | 61:116023/116024/116025 |
| 8681 | 62:118185/118186/118187 | 63:118228/118229/118230 | 64:118233/118234/118235 |
| 8682 | 65:120396/120397/120398 | 66:120439/120440/120441 | 67:120444/120445/120446 |
| 8683 | 68:122606/122607/122608 | 69:122649/122650/122651 | 70:122654/122655/122656 |
| 8684 | 71:124815/124816/124817 | 72:124858/124859/124860 | 73:124863/124864/124865 |
| 8685 | 74:127024/127025/127026 | 75:127067/127068/127069 | 76:127072/127073/127074 |
| 8686 | 77:129233/129234/129235 | 78:129276/129277/129278 | 79:129281/129282/129283 |
| 8687 | 80:131442/131443/131444 | 81:131485/131486/131487 | 82:131490/131491/131492 |
| 8688 | 83:133651/133652/133653 | 84:133694/133695/133696 | 85:133699/133700/133701 |
| 8689 | 86:135860/135861/135862 | 87:135903/135904/135905 | 88:135908/135909/135910 |
| 8690 | 89:138069/138070/138071 | 90:138112/138113/138114 | 91:138117/138118/138119 |
| 8691 | 92:140278/140279/140280 | 93:140321/140322/140323 | 94:140326/140327/140328 |
| 8692 | 95:142487/142488/142489 | 96:142530/142531/142532 | 97:142535/142536/142537 |
| 8693 | 98:144696/144697/144698 | 99:144739/144740/144741 | 100:144744/144745/144746 |
| 8694 | 101:146905/146906/146907 | 102:146947/146948/146949 | 103:146952/146953/146954 |

## Projection and effect boundaries

The existing 441-byte Fire projection is declared in
`restart/original/src/fixture.rs:11` and `.scratch/restart-pipeline/original-calls.md:213`
in the restart worktree. It omits material output of these later roots.

Source-derived additions for the integrator's contract review:

- Handle allocation and Player trail processing require the ten 18-slot arrays
  at `[0xD658,0xD7C0)`, 360 bytes. Recovered C names are in
  `/home/jim/sm-decomp/deps/snesrev-sm/src/variables.h:1083`; scheduling and trail
  updates are `sm_90.c:2649`, `:2689`, `:3192`. Rust storage/behavior is in
  `restart/projectile/src/trails.rs`, called by `core.rs:295` and `draw.rs`.
  Combining this with the existing projection is 801 selected bytes; this is
  a source-based proposal, not an already validated new projection.
- Drawing additionally writes OAM low table, high table and cursor:
  `[0x370,0x592)`, 546 bytes. See `variables.h:105`, `sm_81.c:230`,
  `sm_93.c:161`/`:228`, and `restart/projectile/src/draw.rs`. Include full named
  tables, not just observed changed bytes. Combined selection would be 1,347
  bytes. Player also calls trail processing; checking OAM alone is insufficient.
- Preserve source-declared CPU-stack/DP scratch/removed collision-temporary
  exclusions and retain complete raw outputs. Source eligibility must cover
  ordinary beam/explosion, empty bomb slots, no shinespark echoes, ROM view,
  camera and frozen state. The snapshot retains these inputs; a Fire-only
  eligibility check does not by itself validate later roots.

Within flight Handle calls 2..47, the ledger records two multiplication-register
writes and two result reads per call (`$94:4202/$4203`, `$94:4216/$4217`). At impact,
call47 additionally has `sfx2-max6-entry` at107134, address `$80:90CB`, A=12, and
`semantic-status`107135. No `plm-entry` marker occurs in the captured shot.
Explosion-only Handle calls after impact have no such MMIO rows. C uses ordinary
arithmetic, so original MMIO equality is not implied by selected WRAM equality.

Draw spans sometimes contain `$00:2132` writes, e.g. event73980/call4 and
98325/call36. The observer tags **any** qualifying hardware callback with the
currently pending call (`observe.lua:117`), not with a proven causal writer.
The draw source above emits WRAM OAM, while hardware can operate during the
span. These rows must not become invented C/Rust draw effects. Their exact
hardware origin/timing is outside this inventory. Semantic sound requests and
queue storage can be compared separately once a matching native observer exists;
the existing Fire result explicitly leaves native request order/audio untested.

## Remaining implementation/evidence gap

The capture is sufficient for the selected-output Fixtures above. Missing work
is the bounded later-call selector/validator, actual C entry dispatch and fixed
projection/effect contract in the runner. `restart/original/src/fixture.rs`
currently selects the first Fire only. `validate.rs:23` already knows all four
return sites and full-capture validation tracks the other calls; its retained
historical reader/source are documented by `restart/ORIGINAL.md` at
`out/original/bin/sm-original-lifecycle` and `out/original/source/lifecycle/restart/`.
No new capture or old-main evidence migration is needed to select these pairs.

Run each Fixture from its **own captured original entry**. The space between
Handle and drawing includes other gameplay and interrupts (e.g. irq-entry73933
immediately after Handle return73932). Sequential event order does not mean that
one isolated routine's output can be fed directly to the next routine. The
observed inter-call state and hardware evolution have not been reconstructed.

Future successful comparisons can report full 131,072-byte **C/Rust** agreement
and original agreement on a fixed named projection for all chosen call pairs.
They cannot report full original WRAM equality, CPU/hardware equality, a Fixture
chain, original-to-native frame mapping, or reset-to-removal Live replay parity.
Existing native lifecycle evidence and the first-Fire original result keep
their separate scopes. Current ticket13 verification was not inspected or rerun.

## Smallest runner extension and source identity

Parent runner review: preserve the existing first-Fire command and add a finite
later-root selector with per-root eligibility and fixed projections. Existing
`restart/original/src/main.rs:83` parses a bounded comparison command and runs
it in a separate child with a 30-second deadline. Reuse that boundary and the
existing native-control approach rather than introducing a replay framework.

`fixture.rs:405` boots one authenticated `Game`, commits complete captured
original WRAM, begins native observation and calls the actual exported C body.
Its counters require exactly one isolated Fire call and no host-step, CPU or
NMI advancement. Generalize this to typed finite root dispatch and the matching
root-counter delta. Reset from each pair's original entry before every C call;
run Rust from a separate clone of that same entry. `fixture.rs:451` already
retains raw outputs and separates full C/Rust equality from projected original
equality. Preserve both contracts and explicit dependency/refusal checks.

`validate.rs:23` recognizes all four return sites. Whole-capture validation and
`first_fire_prefix` have separate scopes: preserve the latter for Fire, while
later comparisons validate their own entry/return context against the full
capture. `authenticate_capture` checks hashes, pinned inputs, observer generation,
the captured result and first-Fire prefix. Preserve the reviewed observer
template and the capture's absolute path so capture04 remains reusable.

The retained `out/original/bin/sm-original-lifecycle` and
`out/original/source/lifecycle/restart/` bind historical host/Rust source.
Running that old binary establishes no verdict about the current candidate.
A future extension should bind current `sm-original`, current projectile source
and the matching prepared native package after ticket13 integration, recording
exact source, ROM, command and host identities. Preserve historical library
authentication. This compatibility recommendation is source-derived and has
not been tested. These ordinary pre-escape pairs do not exercise ticket13's
high-bit extension.

Historical measurements were 0.88 seconds for Fire comparison and 1.63 seconds
for full capture authentication plus Fire, excluding build. No new batch timing
was measured. Start with all 34 Handle pairs; add drawing only through its own
reviewed output/context contract. Priority and assignment remain with the root.

## Cleanup

Read-only bounded ledger/CPU/snapshot/source inspection; byte comparisons were
between retained original entry and return files, not new candidate runs. Only
this Markdown file was created. No intermediates, verification processes or
worktrees require cleanup; bytes reclaimed: 0. All retained evidence stayed in
place and unmodified.
