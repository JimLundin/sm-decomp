# Questions for Jim

The questions you need to answer, compiled on 2026-11-05 from:

- [FOUNDATION.md](FOUNDATION.md) section 12 (J1-J24), with sections 9 and 10;
- the open questions of the [frame](frame-requirements.md),
  [domain](domain-model.md) and [Map Rando](map-rando-requirements.md) reports;
- the questions in issues #252 (docs and rules), #253 (the game data audit,
  `docs/handoff/game-data-audit.md` on main) and #242 (mutations,
  `docs/handoff/mutation-sample.md` on main);
- the paused renderer branch in the rebuild handoff (`docs/handoff/rebuild-foundation.md` on
  `handoff/rebuild`).

Duplicates are merged. Each question lists every source it came from. The
questions are in three tiers:

- **Tier 1** blocks the foundation. No spine code is written until these are
  answered.
- **Tier 2** blocks a later slice. The slice is named in each question.
- **Tier 3** covers process and docs.

To answer, write under **Answer:**. "Yes" means you accept the recommendation.
Anything else is a change to it.

## At a glance

| Q | Decision | Recommendation | Was |
| --- | --- | --- | --- |
| **Tier 1: blocks the foundation** | | | |
| 1 | First slice | Two seeded Morph Ball Room rows, not boot | J24 |
| 2 | Where the new code goes | Rename today's crates to `legacy/`; start fresh crates under the accepted names | J7 |
| 3 | Size budget | 83-123k engine lines, a per-domain advisory budget | §9 |
| 4 | Revisions to the accepted destination | Approve all ten | §10 |
| 5 | Overrun lag | None. The oracle's structural lag is the Reference profile | J1, frame Q1, Q4 |
| 6 | Oracle or hardware where they differ | Follow the faithful oracle; list each hardware difference in LEGACY | J2, frame Q6 |
| 7 | State visibility | A public plain-data schema, with private behaviour | J8 |
| 8 | Checkpoint codec | Our own derive (`Canonical`) | J9 |
| 9 | Stale actor references | `SlotRef` where the original keeps slot numbers; generation checks elsewhere | J10, domain Q1 |
| 10 | Interpreter scope | Everything that is a script runs on `Program<F>` | J14, domain Q6 |
| 11 | Video memory | A layout-exact image in the Session | J3, frame Q3 |
| 12 | Renderer, and the paused `gate1/render` branch | A CPU line compositor with `wgpu` presentation; move the compositor into the fresh crate and don't land the branch | J4, handoff |
| 13 | Sound acknowledge model | The CPU protocol and driver intake in `sm_game`; measure the echo latency first | J16, frame Q5 |
| 14 | Where the ROM decoder lives | `sm_data`, at load time | J12, domain Q3 |
| 15 | Small constants used as instruction operands | They stay in source as named constants; only tables are imported | #253 Q1 |
| 16 | Tables an exact formula reproduces | Import them like every other table; the formula becomes a test | #253 Q2 |
| 17 | Lock streams | Redefine `ppu`, `apu` and the typed domains, then re-lock them once | J5 |
| 18 | The gate's measure | Green rows, plus a ratchet on green (row, stream) pairs | J6 |
| **Tier 2: blocks a later slice** | | | |
| 19 | Leftover slot state | Shared fields persist; residue only where the oracle shows it | J11, domain Q2 |
| 20 | How far bosses go into data | Phase machines plus combinators; motion stays in Rust | J13, domain Q5 |
| 21 | Corrupt SRAM slot | Reproduce the original's clear | J15, frame Q7 |
| 22 | Original crashes and softlocks | Reproduce them, then end the Session with a typed failure | J17, Map Rando Q3 |
| 23 | Load-time fidelity | The oracle's load Ticks, with no CPU-time model | J18, Map Rando Q4 |
| 24 | Oracle for Map Rando options | Later, a Map Rando-patched ROM on the emulator | J19, Map Rando Q1 |
| 25 | Map Rando scope at its first gate | Vanilla rules on an imported topology first | J20, Map Rando Q2, domain Q4 |
| 26 | Storing the ID mappings | Regenerate them at import from pinned commits | J21, Map Rando Q5 |
| 27 | Map Rando versions | One pinned version at a time | J22, Map Rando Q6 |
| 28 | Twins and the Toilet | Twins: two rooms in one placement unit. The Toilet comes later | J23, Map Rando Q7 |
| **Tier 3: process and docs** | | | |
| 29 | Nightly mutation sample | Add the ROM-free tier; give new-engine mutants to green rows | #242 |
| 30 | One current-rules file | Yes, about 100 lines, with dated records archived | #252 Q1 |
| 31 | Rules labelled "reviewer" | Drop the label and the completed items | #252 Q2 |
| 32 | Link check in Quality | Yes | #252 Q3 |
| 33 | Where handoffs go | `docs/handoff/` on the branch | #252 Q4 |
| 34 | README's "no game data" claim | The audit's wording, naming the hardware tables | #253 Q3, Q4 |
| 35 | Where the data boundary rule goes | The current-rules file of Q30 | #253 Q5 |

Not on this list: the measurements the build order takes before the code that
depends on them. These are door loading lag per room (frame Q2), the APU echo
latency (frame Q5, see Q13), and the domain report's *(confirm)* rows. Frame Q4,
whether a row shows mid-frame tearing, is answered: the oracle can't produce
tearing (Q5).

---

## Tier 1: blocks the foundation

### Scope and starting point

#### Q1. The first slice (J24)

- **Decide:** whether the first slice starts from a seeded corpus row or from
  power-on.
- **Why it matters:** this sets what is built first. 157 of the 161 rows start
  from a seed snapshot, so the oracle RAM → Session decoder is needed early in
  either case. Boot to the first controllable room is 8,780 Ticks of logo,
  title, intro and Ceres cinematics. That content is far from the foundation's
  seams.
- **Options:** (a) `area-brinstar` and then `item-morph-ball`, both seeded in
  room `9E9F`; (b) boot to one room.
- **Recommend (a).** Together the two rows exercise the HUD split, Samus, a PLM,
  enemies, a message box opened mid-pass with about 370 lag Ticks, the item
  fanfare through the sound protocol, and a Checkpoint restore. Boot comes at
  build step 8.
- **Source:** FOUNDATION §11.

**Answer:**

#### Q2. Where the new code goes (J7)

- **Decide:** how the fresh engine is kept apart from the frozen old one.
- **Why it matters:** today's `sm_game` (160k lines), `sm_data` and part of
  `sm_runtime` are the old engine, wired into `sm_host`. With a separate crate,
  the compiler guarantees that the new engine can't call the old code.
- **Options:** (a) rename today's crates in one mechanical commit, for example
  to `crates/legacy/sm_game_v1`, and start fresh crates under the accepted
  names; (b) new modules inside the old crates; (c) new crate names.
- **Recommend (a).** (b) shares one compile unit, so old code is easy to reuse
  by accident. (c) contradicts the accepted package names. An old module moves
  across only when it meets the new contracts.
- **Source:** FOUNDATION §3.

**Answer:**

#### Q3. Size budget (§9)

- **Decide:** whether to accept 83-123k lines of authored engine Rust,
  excluding tests, as the budget.
- **Why it matters:** the budget is below REVIEW's ~125k target for this
  phase. Each domain gets its own range: Samus 9-12k, bosses 10-15k, spine
  8-12k, decoder 5-9k, and so on. Verification is 12.5-19k on top, counted
  separately.
- **Options:** (a) accept it as advisory: a domain over its upper bound
  triggers a design review, not a ratchet failure; (b) make each range a
  ratchet; (c) set a different target.
- **Recommend (a).** The ranges are estimates until each domain's first rows
  turn green. A ratchet before then would push toward hacks rather than a
  better architecture.
- **Source:** FOUNDATION §9; domain report (57-84k for the rebuilt `sm_game`).

**Answer:**

#### Q4. The ten revisions to the accepted destination (§10)

- **Decide:** whether to approve the changes to `docs/target-architecture`
  that the requirements force. They would be applied in the PR that carries
  your answers, not before.
- **The ten:**
  1. The Scene becomes the display frame over a video memory image, rendered
     by a CPU compositor (Q11, Q12).
  2. Stale references and a public State schema (Q7, Q9).
  3. The Tick's stage table follows FOUNDATION §5's order, with live slot
     passes.
  4. `TickInput` and `TickOutput` gain the display frame, the sound frame, the
     SRAM write and the original failure.
  5. Sound becomes port frames and typed commands (Q13).
  6. Corrupt saves (Q21).
  7. The slowdown policy (Q5).
  8. Vanilla content is decoded at load time, and Tiled and RON authoring waits
     for a real need.
  9. Gate 1 becomes slice 0 (Q1, Q18).
  10. New glossary terms.
- **Why it matters:** agents follow the accepted destination. Until it is
  revised, it contradicts the design.
- **Recommend:** approve all ten. Items 1-7 and 9 follow from the answers
  above; 8 and 10 have no alternatives.
- **Source:** FOUNDATION §10.

**Answer:**

### Fidelity and lag

#### Q5. Overrun lag (J1; frame Q1 and Q4)

- **Decide:** whether the engine lags when the console's CPU would overrun a
  frame.
- **Why it matters:** the oracle never advances the beam while the CPU runs. So
  CPU-overrun lag, mid-frame HDMA tearing and the IRQ's multiplier clobber
  can't happen in the oracle, and no locked row or movie contains them. This
  also answers frame Q4.
- **Options:** (a) a cycle cost model: the engine would lag where the oracle
  doesn't, and the model is large and can't be verified; (b) a recorded
  overrun marker in `TickInput`, which matches only re-recorded hardware
  traces; (c) no overrun lag, keeping structural lag (message boxes, loading,
  door waits) as typed waits.
- **Recommend (c)** as the Reference profile, with one LEGACY entry covering
  overrun lag, tearing and the clobber. It matches every lock by construction,
  and gameplay stays exact because RNG and in-game time advance per
  iteration. The Tick structure leaves room for (b) later.
- **Source:** FOUNDATION §5.3; frame R10, R11.

**Answer:**

#### Q6. Faithful oracle or hardware, where they differ (J2; frame Q6)

- **Decide:** what the engine reproduces where the faithful oracle and the
  console disagree. The known cases are the oracle's kept `b` and `c` patches
  (soft reset and the gunship save confirmation, each a frame late), its line
  rule for mid-line IRQ writes, VRAM writes outside vblank, and IRQ jitter.
- **Why it matters:** the locks are recorded on the faithful oracle. Following
  hardware instead means re-locking rows, or treating them as failing until the
  oracle changes. This also needs checking against your earlier ruling that the
  unpatched original and hardware are the authority, and that oracle patches
  are removed.
- **Options:** (a) reproduce the faithful oracle, list each difference from
  hardware in LEGACY with its patch ID, and follow the lock when the oracle
  drops a patch; (b) follow hardware now, with the affected rows failing until
  the oracle is fixed.
- **Recommend (a).** Mesen and hardware spot checks (#250) settle disputes.
  Removing a patch from the oracle is the route to hardware behaviour: the lock
  changes and the engine follows. That keeps AGENTS.md's rule ("fixed, or
  listed in LEGACY") and keeps the gate meaningful.
- **Source:** FOUNDATION §2 and §12; frame R13.

**Answer:**

### State, actors and the interpreter

#### Q7. State visibility (J8)

- **Decide:** whether the Session's state schema is public.
- **Why it matters:** ARCHITECTURE says domain state is "deliberately
  private". But the oracle decoder must build the same State, and each stream
  is one projection over it, whether the State came from the engine or the
  oracle. If the schema is private, every domain needs a second, hand-kept
  projection.
- **Options:** (a) a public plain-data schema, with behaviour and mutation
  private to `sm_game`, and invariants validated when a Session opens from a
  State; (b) private state with per-domain projection traits.
- **Recommend (a).** With one projection over two sources, the two sides can
  differ only through behaviour or the decoder.
- **Source:** FOUNDATION §4.1, §8.1.

**Answer:**

#### Q8. Checkpoint codec (J9)

- **Decide:** how the State is encoded, decoded and diffed.
- **Why it matters:** this one codec replaces today's six. Its `diff` gives the
  first differing field path, which Session-space lockstep reports.
- **Options:** (a) our own derive macro (`Canonical`), with `syn` and `quote`
  as build-time dependencies; (b) hand-written codecs, about 3-5k more lines.
- **Recommend (a).** The derive is about 0.5-1k lines, outside the engine
  count. It is canonical by construction: declared field order, fixed-width
  little-endian, enums by index.
- **Source:** FOUNDATION §4.6.

**Answer:**

#### Q9. Stale actor references (J10; domain Q1)

- **Decide:** whether references between actors may be unchecked slot numbers.
- **Why it matters:** ARCHITECTURE requires generation-checked handles. The
  original keeps raw slot indices: a projectile's parent enemy, a PLM's linked
  enemy, a boss's part slots. When a slot is reused, the reader sees the new
  occupant, and that is observable.
- **Options:** (a) a typed `SlotRef` with the original's reuse semantics
  wherever the original keeps slot numbers, each one listed, and
  generation-checked handles elsewhere; (b) generation checks everywhere, which
  diverges from the original wherever it observes slot reuse.
- **Recommend (a).** It is exact, and it is visible in the types.
- **Source:** domain report §2; FOUNDATION §10 item 2.

**Answer:**

#### Q10. Interpreter scope (J14; domain Q6)

- **Decide:** which scripts run on the one shared `Program<F>` interpreter.
- **Why it matters:** one interpreter is a main lever on engine size. Samus's
  animation delay lists and the demo inputs use different encodings (byte
  control codes) from the instruction lists.
- **Options:** (a) PLMs, enemies and projectiles, plus Samus animation, demo
  input, Samus projectiles and cinematic scripts, all decoded into the same
  step form; (b) a separate interpreter for each encoding.
- **Recommend (a).** Encoding differences belong to the decoder; the engine
  never reads bytes.
- **Source:** domain report §1; FOUNDATION §4.3.

**Answer:**

### Video and sound

#### Q11. Video memory (J3; frame Q3)

- **Decide:** whether the Session owns an SNES-layout VRAM, CGRAM and OAM image,
  or a typed Scene of visual IDs that verification projects into SNES layout.
- **Why it matters:** the game reads video memory back (pause, x-ray), so the
  Session must own the bytes anyway. A projection would reimplement every
  upload a second time.
- **Options:** (a) a layout-exact image, written only at the two commit points
  (the NMI batch and forced blank), with the Scene being the display frame over
  it; (b) a typed Scene with a verification projection; (c) the image in the
  renderer only, which R16 and R18 rule out.
- **Recommend (a).** The `vram`, `cgram` and `oam` streams become direct
  digests, and seeds decode by copying. Custom Samus sheets and palettes are
  replaced where content is decoded, so they still work.
- **Source:** FOUNDATION §6.1; frame R16-R18.

**Answer:**

#### Q12. Renderer, and the paused `gate1/render` branch (J4; handoff)

- **Decide:** (1) the renderer's shape; (2) what happens to `gate1/render`.
- **Background:** at `839c69a` all 114 rows match. At `09c2895` the hardware
  departures are implemented rather than listed, but credits and menu rows run
  about 30% slower: Mode 1 with BG3 at normal priority takes the per-pixel
  path. `615e3b2` is diagnosis only.
- **Why it matters:** ARCHITECTURE names a "focused GPU renderer". The `video`
  stream must match LakeSnes's pixels, and that is easier to reach and prove
  on the CPU.
- **Options:** (1a) a CPU line compositor with the oracle's pixel rules, with
  `wgpu` only for scaling and presentation; (1b) GPU composition. (2a) land
  `09c2895` on the old path now; (2b) don't land the branch, and move the
  compositor (`draw_line` and the per-pixel composer) into the fresh
  `sm_runtime` in slice 0, fixing the slowdown there.
- **Recommend (1a) and (2b).** Under the rebuild decision the old path isn't
  improved further. The `sm_host` glue isn't reusable, and the compositor is.
  `615e3b2` never lands.
- **Source:** FOUNDATION §6.2; rebuild handoff, "Paused".

**Answer:**

#### Q13. Sound acknowledge model (J16; frame Q5)

- **Decide:** where the sound driver's acknowledgements are modelled.
- **Why it matters:** the `apu` stream compares the CPU's port writes, and the
  game reacts to the driver's echoes. The oracle hands the ports to the driver
  once per frame, so the handshake can match in Tick time.
- **Options:** (a) the CPU protocol and a deterministic driver intake model in
  `sm_game`, with the sequencer and mixer in `sm_runtime`; (b) the sequencer's
  control part in the Session from the start.
- **Recommend (a), measuring first.** Log the oracle's port reads per Tick over
  `music-fir-selector10` and an item row. If echoes depend on sequencer state,
  move only the control part into the Session.
- **Source:** FOUNDATION §6.3; frame R22.

**Answer:**

### Content and data

#### Q14. Where the ROM decoder lives (J12; domain Q3)

- **Decide:** whether content is decoded inside `sm_data` at load time, or by
  `sm_tools` into a prepared pack that `sm_data` validates.
- **Why it matters:** the choice decides whether the 5-9k lines of opcode,
  header and room tables count as engine lines. It also decides whether the
  address map is produced in the engine's crate.
- **Options:** (a) `sm_data`, at load time; (b) an `sm_tools` prepared pack.
- **Recommend (a).** It matches REVIEW decision 6 ("read from the ROM at load
  time"). There's no pack format to version, and Map Rando content goes through
  the same decoder.
- **Source:** FOUNDATION §7.1; domain report §3.

**Answer:**

#### Q15. Small constants used as instruction operands (#253 Q1)

- **Decide:** whether REVIEW decision 6 (game data is imported from the ROM,
  not committed) covers the 911 small constants: speeds, timers, sound IDs and
  thresholds that the original code uses as operands.
- **Why it matters:** the answer decides how every new module is written. If
  operands are imported, each one becomes a decoded content field read from its
  instruction's address, and the decoder grows a lot.
- **Options:** (a) tables only: operands stay in source as named constants;
  (b) operands are imported too.
- **Recommend (a),** the audit's reading. Operands are part of the code's
  behaviour, like the code itself. A per-species parameter that sits in a ROM
  table is a table, and is imported.
- **Source:** `docs/handoff/game-data-audit.md`, Questions 1.

**Answer:**

#### Q16. Tables an exact formula reproduces (#253 Q2)

- **Decide:** for tables such as `SINE_COSINE`, whether to generate them from
  the formula or import them. `trunc(-256·cos(2πi/256))` reproduces all 320
  values exactly in f64 and f32.
- **Why it matters:** importing keeps a modified ROM authoritative. Generating
  removes a ROM read.
- **Options:** (a) import, with a test that checks the formula against the ROM;
  (b) generate, with a ROM equality check.
- **Recommend (a).** It is one rule for all tables ("content comes from the
  ROM"). A hack that changes the table still works, and the formula remains as
  evidence.
- **Source:** game data audit, `SINE_COSINE` and Questions 2.

**Answer:**

### Verification and the gate

#### Q17. Lock streams (J5)

- **Decide:** whether to redefine three stream kinds and re-lock them once on
  the faithful oracle.
- **Why it matters:** `ppu` today is LakeSnes's internal state, with latches,
  H/V counters, open bus and renderer scratch, which no engine has. `apu` is
  not precisely defined. The typed domains are two hand-kept projections.
- **Options:** (a) define `ppu` as the architectural registers, with named
  exclusions; define `apu` as the Tick's port frame as delivered; define the
  typed domains, plus a new `rng`, as one projection over the decoded State;
  re-lock those once, when the decoder lands; (b) keep today's definitions.
- **Recommend (a).** Under (b), `ppu` can never be green. `vram`, `cgram`,
  `oam`, `sram` and `video` don't change.
- **Source:** FOUNDATION §8.1.

**Answer:**

#### Q18. The gate's measure (J6)

- **Decide:** what counts as progress for the gate.
- **Why it matters:** a whole row can take many changes to turn green. A
  regression inside a row that isn't green yet would go unseen.
- **Options:** (a) green rows, meaning every stream except `wram` matches the
  lock, plus a ratchet on green (row, stream) pairs; (b) green rows only.
- **Recommend (a).** It protects partial progress, and the gate itself stays
  REVIEW's "never fewer green rows".
- **Source:** FOUNDATION §8.3.

**Answer:**

---

## Tier 2: blocks a later slice

#### Q19. Leftover slot state (J11; domain Q2), build step 3, enemies

- **Decide:** what to do when a species reads a variable its init didn't clear
  and so sees the previous occupant's value. Whether any species does this is
  not yet confirmed.
- **Options:** (a) the shared fields persist across occupants, and a
  per-species `Residue` is added only where the oracle shows it, listed in
  LEGACY; (b) treat every case as a divergence to fix.
- **Recommend (a).** It is exact where it matters, and costs nothing until
  evidence appears.
- **Source:** domain report, Open questions 2.

**Answer:**

#### Q20. How far bosses go into data (J13; domain Q5), build step 7

- **Options:** (a) phase machines plus shared combinators, with motion in Rust
  (estimated 10-15k); (b) motion paths as authored curves too: smaller, but each
  path must be proven frame-exact against the oracle.
- **Recommend (a).** Motion is maths, and the domain report's rule keeps maths
  in Rust. (b) can still come later for a boss whose path is a pure table.
- **Source:** domain report §4; FOUNDATION §4.4.

**Answer:**

#### Q21. Corrupt SRAM slot (J15; frame Q7), build step 6

- **Decide:** the original's load clears a slot whose checksum fails. Should
  this be reconciled with CONTRACTS' "do not silently reset a corrupt slot"?
- **Recommend:** reproduce the clear inside the SRAM image, because it is game
  behaviour. The no-silent-reset rule then applies to our container file: a
  damaged container refuses to open.
- **Source:** FOUNDATION §6.4; frame R30.

**Answer:**

#### Q22. Original crashes and softlocks (J17; Map Rando Q3), build step 4 and Map Rando

- **Decide:** what the engine does with the original's crashes and softlocks:
  the spring ball menu, the Yapping Maw spark, X-mode, sprite overflow and the
  auto-reserve pause.
- **Why it matters now:** `TickOutput` carries the failure (Q4 item 4), so its
  shape is part of the foundation. The policy is needed later.
- **Options:** (a) reproduce up to the observable point, then end the Session
  with a typed `OriginalFailure`, with Map Rando presets mapping each failure
  to death, warn or continue; (b) always recover, with each case listed in
  LEGACY.
- **Recommend (a).** It is exact in vanilla, and options decide the rest.
- **Source:** Map Rando report §9; FOUNDATION §5.6.

**Answer:**

#### Q23. Load-time fidelity (J18; Map Rando Q4), build step 2, doors

- **Decide:** whether loads take the oracle's Tick counts and RNG calls per
  iteration, or follow a CPU-time model of decompression and music uploads.
- **Options:** (a) reproduce the oracle's load Ticks (structural waits, RNG per
  iteration), with no CPU-time model; (b) model the loaders' CPU time.
- **Recommend (a),** because the oracle has no CPU-time model either (Q5).
  Map Rando strats that rely on load timing are covered by matching the oracle.
- **Source:** FOUNDATION §5.5, §6.3; frame R23.

**Answer:**

### Map Rando (after build step 3)

#### Q24. Oracle for Map Rando options (J19; Map Rando Q1)

- **Options:** (a) later, a Map Rando-patched ROM on the emulator is the
  authority for "Map Rando base", with seeds generated in CI using Map Rando's
  MIT crate and a ROM the user supplies; (b) our own rules plus interaction
  checks, with no oracle for options.
- **Recommend (a).** Every option then has the same kind of evidence as
  vanilla.
- **Source:** Map Rando report, Open questions 1.

**Answer:**

#### Q25. Map Rando scope at its first gate (J20; Map Rando Q2, domain Q4)

- **Why it matters:** Map Rando patches routines (room setup, doors, PLMs), not
  only data. Every patched routine becomes either a vocabulary entry
  (`DoorAction`, `RoomMain`, `PlmOp`) or a named option. Complementary suits
  are always on in Map Rando, so its seeds assume them.
- **Options:** (a) first, vanilla rules on an imported topology; then the "Map
  Rando base" preset at the following gate, covering only that preset's patched
  routines; (b) "Map Rando base" at the first gate.
- **Recommend (a).** It isolates the topology work from the rule changes.
- **Source:** Map Rando report, Open questions 2; domain report, Open questions 4.

**Answer:**

#### Q26. Storing the ID mappings (J21; Map Rando Q5)

- **Decide:** whether the mapping from sm-json-data and Map Rando IDs to our IDs
  is committed, or regenerated from pinned upstream commits at import. It is
  derived from CC BY and MIT sources.
- **Recommend:** regenerate it. Commit the pins and our own ID definitions, not
  the mapping. That treats it like ROM data and avoids licence questions.
- **Source:** Map Rando report, Open questions 5.

**Answer:**

#### Q27. Map Rando versions (J22; Map Rando Q6)

- **Options:** (a) one pinned version at a time; (b) a range of versions, each
  with its own adapter.
- **Recommend (a).** The importer rejects unknown fields, so a version bump is
  an explicit change.
- **Source:** Map Rando report, Open questions 6.

**Answer:**

#### Q28. Twins and the Toilet (J23; Map Rando Q7)

- **Options:** (a) the twin rooms (Pants and East Pants, West Ocean and Homing
  Geemer) stay two rooms in one placement unit, and the Toilet overlap waits
  until after the first acceptance; (b) each pair becomes one room with two
  states, and/or the Toilet is in scope now.
- **Recommend (a).** It matches the original's room headers. The Toilet is a
  transform that can be added later without changing the foundation.
- **Source:** Map Rando report, Open questions 7.

**Answer:**

---

## Tier 3: process and docs

#### Q29. Nightly mutation sample (#242)

- **Decide:** what to add to the nightly. The design proposes about 16
  runner-minutes: the ROM-free `sm_mutants` tier (anchors, build, and run
  mutants on the decoder, `session_compare` and the lock), plus 2 of the 4
  replay-level lock mutants by rotating slot. The legacy Python suites would
  run weekly until the RAM encoder retires.
- **Why it matters:** this shows that the checks can still fail. The 4 lock
  mutants and the Python suites test the old engine, which is now frozen.
- **Options:** (a) the design as written; (b) the ROM-free tier only, with
  new-engine mutants added as rows turn green, as FOUNDATION §8.3 runs
  mutation sensitivity on every green row.
- **Recommend (b).** The decoder, compare and lock mutants carry over to the
  new foundation, while mutants on the old engine don't. This differs from the
  handoff's proposal.
- **Source:** `docs/handoff/mutation-sample.md` §3; #242.

**Answer:**

#### Q30. One current-rules file (#252 Q1)

- **Decide:** whether to merge PLAN's and REVIEW's standing rules into one file
  of about 100 lines, and move the dated records (lane S conditions, step
  records, "Then Gate 1") to an archive.
- **Recommend:** yes. Agents take these documents as instructions, so
  conflicts between them cost real work.
- **Source:** #252.

**Answer:**

#### Q31. Rules labelled "reviewer" (#252 Q2)

- **Decide:** what to do with REVIEW's "Review conditions (reviewer,
  2026-10-31)" and SCENARIO_HARNESS's Decision 1. Should they be relabelled as
  approved by you, or should the reviewer label and the completed items be
  dropped?
- **Recommend:** drop the label and the completed items. Anything still in
  force becomes a rule in Q30's file, which only text you approve can change.
- **Source:** #252.

**Answer:**

#### Q32. Link check in Quality (#252 Q3)

- **Recommend:** yes. `tools/check_doc_links.sh` is about 20 lines of sh, passes
  on the tree, and takes seconds.
- **Source:** #252.

**Answer:**

#### Q33. Where handoffs go (#252 Q4)

- **Decide:** the sm-handoff skill writes `HANDOFF.md` under `$TEMP`, where no
  one else can open it.
- **Options:** (a) `docs/handoff/` on the lane's branch; (b) the PR
  description.
- **Recommend (a).** Recent handoffs already do this. Small lanes may use the
  PR description.
- **Source:** #252.

**Answer:**

#### Q34. README's "no game data" claim (#253 Q3, Q4)

- **Decide:** the README's new wording. Lines 33 and 129 say "No ROM or game
  data is included", which isn't true today (314 copied tables).
- **Options:** (a) the audit's draft, which says copied tables are moving to
  the import and links the audit's list; (b) name every remaining copy in the
  README. Separately: name the console hardware tables (`GAUSS` and `RATES`
  from the S-DSP, and the SPC700 boot program in the oracle)?
- **Recommend:** (a), plus one clause naming the hardware tables, which the
  finished engine keeps. A list in the README would go stale.
- **Source:** game data audit, Questions 3 and 4.

**Answer:**

#### Q35. Where the data boundary rule goes (#253 Q5)

- **Decide:** where to write the rule that resolves the conflict #253 names.
  Hand-authored state machines live in `.rs`, and the values they use from the
  ROM are imported. This reconciles the Nov 4 data-driven-boss ruling with "never
  commit derived game data".
- **Options:** REVIEW decision 6, PLAN's standing rules, or Q30's file.
- **Recommend:** Q30's current-rules file, or PLAN until that file exists.
- **Source:** game data audit, Questions 5.

**Answer:**
