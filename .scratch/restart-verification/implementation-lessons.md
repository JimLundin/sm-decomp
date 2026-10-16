# Lessons established by the restart implementations

These are observations from completed deliveries, not additional acceptance
requirements. Main's reusable source ideas are listed in [main-reuse.md](main-reuse.md).

- **Keep the reference stable across gameplay edits.** The scrolling consumer
  reused the existing comparison tool and separated fixed C/host identity from
  candidate Rust identity. Its forced warm build/comparison took 16.622 seconds;
  a documentation change took 0.208 seconds and launched no game process.
  Changing the host recipe required a new C reference, not a new framework.
- **Measure behavior and ownership together.** Projectile evidence follows
  actual spawn, motion, drawing, impact and removal; scrolling records camera
  movement and row/column uploads in reached rooms. Invocation counts alone
  are insufficient. Apply that lesson to door opening and actual passage.
- **An independently valid observation can survive a later failure.** Original
  capture 03 completed a Fire entry/return before an unrelated Draw hook failed.
  Reusing that first eligible pair produced the first original comparison in
  0.88 seconds, before fixing or replaying the later lifecycle. Its failed
  overall status remained intact. Capture 04 later closed the lifecycle gap in
  37.918 seconds. This avoided making every useful result wait for every detail.
- **Capture the evidence needed to diagnose a guard.** X reached the original
  controller path without firing because Samus was front-facing. Input-time
  state established the pose; normal Right input enabled firing. The turn also
  moved X from 128 to 130, exposing a guessed guard. Preserve observed state
  before rejecting assumptions, and change only the hypothesis the evidence
  contradicts.
- **Describe exactly what equality establishes.** C/Rust agreed on all WRAM;
  the first original comparison used a predeclared 441-byte semantic projection.
  Original CPU stack, whole lifecycle and audio/hardware equality were separate
  unproven claims. A post-execution output mutation proved comparator rejection,
  not a gameplay implementation mutation; the final description says so.
- **Preserve the working cache and retire temporary builds.** Each delivery
  retained its primary incremental target and evidence dependencies while
  removing completed worker worktrees. Ticket 07 reclaimed approximately
  989 MB gross and authenticated retained evidence after cleanup. Rebuilding
  an unchanged emulator or discarding the warm target would add no gameplay
  evidence.
- **Retain synchronous behavior when moving a boundary into Rust.** The
  directional scope preserves the first movement inside setup and two separate
  scrolling child calls at terminal counters63 and64. These are observed source
  behaviors; replacing them with a convenient host-step timer would lose the
  actual call ordering. All13 routines are implemented, while the two rightward
  crossings live-exercise only six. That distinction lets progress continue
  without overstating coverage or requiring every direction before integration.
- **Review while the integrator executes.** Ticket10's first candidate passed
  in15.689s including an actual0.601s incremental rebuild. The independent
  reviewer read completed artifacts while the integrator directly launched
  short runs. Its one-off C capture took30.956s, but the measured warm loop
  stayed below30s; that alone warranted no profiling project or repeated run.
- **Use an existing long route to assess accumulated behavior.** After five
  consumers were installed, one unchanged16,991-input CLI run reached Ceres
  escape and area0 in27.445s. It added leftward scrolling and all13 Ceres-door
  entry kinds without a build or new recipe. Its projectile refusals exposed
  the already documented Mode7 boundary; keeping that limit explicit avoided
  treating mixed gameplay success as total Rust ownership or broadening scope
  merely to make a counter zero. Full per-step parity remains separate.
- **Main's bounded algorithms can produce new gameplay scope.** The camera
  implementation reuses selected camera constraints and distance arithmetic
  while computing the pure WRAM root once before atomic publication. Its first
  combined9,380-step candidate passes in12.834s including a0.549s rebuild;
  11/13 routines are live, including real state9 tail calls. The existing normal
  CLI and intentional one-pixel fault complete the same comparison path.
  Documentation reuse0.151s and post-cleanup reuse0.141s launch no game. Slow
  and finish-hook variants retain their separate focused evidence.
- **A full route and an active fixture can answer distinct questions.** The
  escape extension reused main's defined transform arithmetic and removed706
  live update fallbacks while preserving the undefined drawing exclusions.
  The16,991-input full comparison passes;108 bounded actual-C update-return
  pairs separately cover active transformed trails, impact and deletion.
  Neither establishes active escaped-shot rendering from reset.
- **Check whether a deliberate fault survives to the observation point.** A
  scheduler-index mutation was overwritten by later drawing, so the full-frame
  comparison passed. That ineffective probe stayed recorded. A persistent
  update-output mutation failed at14373; the same C reference served the final
  candidate. Its measured rebuild/comparison28.665s stayed within30s, with
  final documentation reuse0.962s and post-cleanup reuse0.259s/zero games.

Evidence: completed monitor tickets [02](issues/02-monitor-pilot.md),
[03](issues/03-monitor-scrolling.md), and
[04](issues/04-monitor-original-capture.md), plus the passing ticket10 results
in [08](issues/08-monitor-door-scrolling.md) and the full-route result in
[09](issues/09-monitor-full-ceres.md), and the camera results in
[10](issues/10-monitor-normal-camera.md), plus projectile escape results in
[12](issues/12-monitor-projectile-escape.md). No new
runs were performed for this note, and it creates no additional gate.

- **Use a complete retained original inventory before widening replay claims.**
  Ticket14 compares all34 captured ordinary projectile update calls, each from
  its own original entry, in7.134s using one retained capture/native package.
  All801 selected original bytes agree and full C/Rust WRAM agrees separately.
  Flight, trails, impact8676 and removal8694 are included without claiming a
  continuous replay. The output fault rejects the first call; no new capture
  or gameplay regression was needed for the original-runner extension. Cleanup
  and1093 final/dependency hashes pass. See
  [13](issues/13-monitor-original-projectile-updates.md).

- **A playable checkpoint must include resumption and input effect.** Ticket15
  extends the exact16991-pad native prefix by1081 ordinary inputs. Parlor appears
  during loading at17977; normal state8 resumes18064 and eight consumed polls
  move29px left by18072. The first author run16.825s and strict policy-free replay
  17.099s have identical complete inputs/CSV/WRAM/image. Existing Rust owners are
  exercised without claiming new translation or full differential/original
  parity. Cleanup and498 retained/dependency hashes pass. See
  [14](issues/14-monitor-parlor-progression.md).

- **Correct reached input assumptions without weakening the checkpoint.**
  Ticket16 retained two bounded failures: a disabled grounded jump in Climb,
  then premature Down before standing elevator contact. Narrow policy fixes
  preserve the same five-room route, prefix, budget and pickup predicate. The
  successful23.426s author and23.326s strict replay set all three Morph flags,
  observe398 presentation waits, then14 consumed polls/16px resumed movement.
  All seven artifact pairs agree; cleanup and1173 final/dependency hashes pass.
  Flags/state8 during presentation and generic inherited controller-response
  fields were kept distinct from completed Morph acquisition. See
  [15](issues/15-monitor-morph-progression.md).

- **Retain partial behavior and total diagnostic cost.** Ticket17 first proves
  actual Morph movement before it can complete tunnel/descent navigation. Four
  failures preserve that distinction and motivate four narrow policy fixes.
  Final1796-input tail proves shell/shot/reveal, then+5ammo/cap/item, message2
  completion and resumed response. Author24.906s/replay25.134s match eight artifact
  pairs; total gameplay173.644s includes123.604s of failures. Cleanup,1775 retained
  hashes and120 dependencies pass. This is useful native progression and live
  exercise of existing Rust, separate from new translation/full parity. See
  [16](issues/16-monitor-first-missile.md).

- **Use a proven route to validate new ownership rather than author another.**
  Ticket18 translates13 item instruction bodies and proves54Rust completions
  across normal/service52 contexts. Its unchanged24832-input C/Rust pair preserves
  eight accepted artifacts plus114full-WRAM/typed/service observations in
  25.603/24.861s. Compact25body/14refusal/composition and11fatal controls pass;
  real live scheduler phases satisfy overlapping K01/K02 requirements. A compiler
  declaration error and helper-parser failure remain diagnosed without an extra
  game after parser correction. Cleanup,604retained+22dependency hashes and all
  four stages pass. See [17](issues/17-monitor-item-instructions.md).

- **Reuse reached behavior and count actual ownership.** Ticket19's first
  candidate adds five Morph functions and completes1028Rust calls across12
  scheduled roots; local Common executes via186frame roots. Its fixed24832
  pair takes53.123s total and independently matches13artifact pairs plus prior
  Morph/missile evidence. Compact31body/20refusal and15fatal controls cover the
  new behavior and affected ABI; there is no second historical live run.
  Cleanup retires both workers and reclaims443600896bytes. The successor
  [resource-tank note](next-ordinary-item-scope.md) names four new bodies and
  two HUD services, while explicitly separating actual-C progress from new
  live collection absent from the current tape. No extra framework or gate.

- **Measure recurring closeout commands rather than estimating from old runs.**
  Ticket19's translated/verified/integrated/live updates take8.354/17.191/
  18.083/15.305s, total58.933s; translating separately takes9.648s. Installed
  command timings do not isolate hashing/rendering, but show publication is
  comparable to its53.123s gameplay pair. One bounded batch-publication plan
  is now ahead work, preserving each stage and all evidence checks. Source
  ownership must be reconciled before implementation; it cannot gate20.

- **Separate new source-family acceptance from route discovery.** Exposed
  tank20 closes88dead0 with four new collectors/twoHUDservices,36actual-C body
  pairs,21refusals and10real normal/collected scheduler pairs. Fresh-process
  child times total2.941s and all raw phase files independently match. One
  25.373s compatibility replay reuses19's13artifact baseline; newcollector
  callbacks/completions remain0 and no newcollector live stage is invented.
  Cleanup443797504bytes/946retained+43dependency hashes pass. Batch21's code
  integrates separately7ad531a after20seal, avoiding gameplay gate expansion.

- **Carry a concrete optimization through ordinary use and cleanup.** Batch
  publication21 closesab1055f, source7ad531a, with25focusedtests passing and one
  actual three-stage retry5.38s that preserves326observations/timestamps. Main
  and production binaries remain unchanged. A coldcompile131.537s fails on
  unrelated missing pinned C includes; dependency restoration/samefilter57.557s
  passes, then standalonebuild86s. These are real setup costs, not hidden by the
  0.16s test execution. Valid2GBauxiliary cache stays mutable outside sealed
  evidence; worker retirement reclaims221962240bytes and47retainedhashes pass.
  Prior separate20publications23.261s are context, not a controlled speedup.
  Check dependency completeness before future cold builds; retain the warm
  tooling target and assess later fresh batch costs during ordinary delivery.
