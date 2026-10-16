# Independent review: vertical player motion

Reviewer: `01a145c8-9bc8-78a2-b0f9-8519ad5f75f5`, 2026-10-16.
Disposition: **the retained evidence supports seven bounded actual-C definitions
and six live-exercised definitions.** UpNoColl has actual-C coverage but no live
reach in this replay. Terrain remains native and live root ownership is mixed.

Evidence root: `/home/jim/sm-decomp-restart/restart/out/player-motion/`.
Native source is `9868614012ae0ee0df6c2b2a40ed16778a0f0bac`; control and live
source is `361304751ba23aef358f31ebd2572e3f77e8fb97`. Candidate source archives,
executables and dependency manifests retain those frozen identities independently
of subsequent integration work. This review grants no original-ROM parity.

## Source scope and reuse

The seven definitions are Samus_MoveUp, Samus_MoveDown,
Samus_CheckSolidEnemyColl, Samus_MoveUp_NoColl, Samus_MoveDown_NoColl,
Samus_MoveUp_SetPoseCalcInput and Samus_MoveDown_SetPoseCalcInput. Only Up/Down
are scheduled roots; the other five are local helpers or direct fixture entries.
Down feedback has no invented ROM address. The module forbids unsafe code.

The implementation reuses main's checker/control ideas while preserving the
original C promoted left/up exact-contact comparison, byte 0A1F alias semantics
and fixed-point widths. The completed forward packet supplied source/dependency
research; this review does not reopen that investigation. Native terrain
Samus_MoveDown_NoSolidColl949763 remains an explicit synchronous dependency,
including cur_coll_amt32 lifetime and SpawnPLM effects. The finite admitted
geometry/reaction domain excludes standing radius21. Refusal before ownership
permits C fallback; failures after accepted foreign work cannot retry C.

## Actual-C and failure controls

The completed control reports 192 shared body pairs and 31 terrain pairs.
Independent comparison found all **223 full-WRAM, effect and observer pairs
byte-identical**. Both owners' recorded WRAM/effect hashes also authenticate.
The inspected comparator asserts raw compound return equality, whole state,
ordered effects/observations and exact ownership/branch counter deltas.

| Entry | Pairs |
| --- | ---: |
| Up root | 9 |
| Down root | 36 |
| Solid-enemy checker | 75 |
| UpNoColl | 6 |
| DownNoColl | 7 |
| Up feedback | 3 |
| Down feedback | 87 |

There are 45 scheduled-root pairs and 178 direct-helper pairs; direct calls do
not manufacture scheduler reach. Native terrain cases include actual quicksand
clamp, contact-zero/full-pool behavior, spawn parity, spikes, slopes, door state,
crumble and speed effects. Twelve admission controls comprise eleven rejected
unsafe/unowned inputs and one legal unchanged callback refusal with real C
fallback and no false Rust helper counts.

All 13 distinct fatal process receipts record exit2, verified diagnostics and
no timeout. Their retained stdout/stderr hashes independently authenticate.
They cover dirty/request refusal, postrequest error/panic, wrong buffer/size/
selector/generation/geometry, root/sibling/observer recursion and TerrainEnd
observer failure. Combined child elapsed time is936ms. No control was rerun by
this reviewer. Retained semantic logs show14 tests passing; the initial control
Clippy style failure remains recorded alongside its corrected passing check.

## One retained live replay

The recorded24832-input run completed in27.418678136seconds without a policy.
All13 accepted20 compatibility artifact pairs authenticate and match, including
inputs, observations, missile/item/Morph artifacts and final WRAM/PNG. This is
compatibility with the selected retained reference, not whole original-ROM parity.

| Counter | Up | Down |
| --- | ---: | ---: |
| Root entries | 1194 | 11022 |
| Rust completed | 539 | 1230 |
| C entries / Rust refusals | 655 | 9792 |
| Rust terrain-return branches | 539 | 1197 |
| Rust enemy-hit / NoColl branches | 0 | 33 |
| Rust feedback returns | 539 | 1230 |
| Terrain unavailable | 655 | 9797 |

Global terrain requests/completions12183 include both owners; Rust-owned terrain
returns total1736. Five Down enemy hits complete in Rust despite unavailable
terrain, so terrain-unavailable counts are not refusal counts.

The inspected retained Rust auditor validates48798 scalar callback rows:
12216 ordered root begin/end pairs,12183 matched terrain begin/end pairs,33 Down
roots without terrain, no reentrancy and no dangling terminal event. It checks
step, entry and amount matching. The CSV has no ownership field; separate typed
counters establish Rust reach. Native helper/feedback counters increment only
under rust_owned. The evidence therefore supports live reach of both roots,
checker, both feedback definitions and DownNoColl; **UpNoColl remains unexercised**.

The same replay usefully extends producer26's installed-runtime evidence:
Save/Load entries are3/7, Rust completions1/7, safe refusals/C2/0 and disk writes3/3.
This is genuine bounded gameplay reach without another persistence run. It does
not establish file reopen, durability, successful saved-game restoration or
separately instrumented local Pack/Unpack reach. Producer26's prior report remains
an immutable account of its earlier compact composition evidence.

## Authentication, cost and cleanup

| Artifact | SHA-256 |
| --- | --- |
| Native library, f06c662c recipe | `c02576a7672daca9cf737b18055a2fc08bc8f7591ee96b6b33622345eb8380cd` |
| Control executable | `7c9fafafbdf8467052d126e7762749c4ad64f18660a898e249cce3fe5f881f64` |
| Live executable | `661c0476121d215ca82e4324bd06ba54d302347691756175da3540858b54dfc0` |
| Actual-C result.json | `ccac9f5256f3fd71c17c12d0eda055774896f524907de646d49058c292e30f25` |
| Live result.toml | `2a0e43b53aeb27bd1313299790c5412fe58db308c09b801c9f4b01083960f07c` |
| Player-motion callback CSV | `cab45270c1fe06bf86e5cc20d1d21f92a91ba0ac580f0e83deac3a43d50b9a81` |

Native/control candidate seals passed in the preceding review; live candidate
archive/executable/library/input seals passed after retirement. Semantic-test
compilation records0.71seconds, shared build3.52seconds, control build2.58seconds
and warm live build0.04seconds. These plus the live/fatal timings are individual
receipts, not a complete delivery-time estimate or measured speedup.

Both project cleanup passes report zero disposable bytes. Retiring both worker
worktrees reduced allocated storage444985344→188416bytes, reclaiming
**444796928bytes**; recorded available space is97812611072bytes. Independent
inspection confirms both workers absent from Git worktree registration, no .git
or target at their retained paths, and all16 preserved source copies identical
to worker-source/. Native archive/library/source, private ROM, exact source
archives, executables, raw fixture/fatal evidence, accepted20 dependencies,
live files and auditor remain required. Active Waver work and the root warm
target remain owned by their existing producer.

This review performed only source/receipt inspection, retained comparisons,
hash authentication and the inspected read-only auditor. No build, native
control, test, gameplay or reference capture was launched. Reviewer cleanup is
zero bytes: only this Markdown and local tracking are authored. The integrator
owns stage publication and the final enclosing seal after copying this immutable
report; this review does not assert that an unpublished final manifest passed.
