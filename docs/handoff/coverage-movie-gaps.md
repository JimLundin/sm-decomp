# Gameplay coverage gaps in the existing movies

Inventory at source `1e91c617c3b7c1fed10df539ef21ce2d43728dc7`, before the
new `coverage-*` scenarios qualify. Counts come from `movies/manifest.toml`,
`scenarios/route-{any,100}.toml` and `scenarios/lock.toml`; they describe committed
scope, not a fresh execution. Video frames in the movies and native replay ticks
are different units. Seeded windows can overlap and reset machine state.

| Movie | Published video frames | Existing room chain | Faithful locks | Remaining scope |
| --- | ---: | --- | --- | --- |
| any% (`3653M`) | 129,712 | 197 rooms, 106,110 selected ticks | 193 locked; 4 unlocked | Unlocked rooms; final ship departure and movie-specific ending; continuous state across seeded starts |
| 100% (`4010M`) | 222,788 | 331 rooms, 140,106 selected ticks | 327 locked; 4 unlocked | Unlocked rooms; continuous state across seeded starts; rare enemy interactions bypassed by the route |
| low% 13% (`3273M`) | 167,797 | No low% room chain | 5 locked focused rows; 1 unlocked | Route-wide gameplay, completion and carried state outside six selected encounters |

The any% and 100% chains end at the landing site with a `state:8` fact and a
129-tick tail. That fact proves gameplay there, not ship departure or credits.
Existing `credits` and `credits-ending` use the 100% credits capture, not the
any% or low% movie. The six direct low% capture rows are `boss-botwoon`,
`boss-draygon-grab`, `volcano-room-lavaquake`, `boss-draygon-tail-whip`,
`boss-draygon-later-swoop` and `enemy-fire-arc`; the last remains unlocked.
Authored derivatives of movie seeds add interactions, not whole-movie proof.

The unlocked route rows are:

| Movie | Scenarios |
| --- | --- |
| any% | `chain-any-0011-DF45`, `chain-any-0074-AFA3`, `chain-any-0079-ACF0`, `chain-any-0196-92FD` |
| 100% | `chain-0011-DF45`, `chain-0086-AA41`, `chain-0211-C98E`, `chain-0330-92FD` |

Their existing `unlocked` notes and `docs/LEGACY.md` preserve the divergences;
more green lockstep rows do not retire them. The separate divergence owner keeps
those fixes. This coverage lane adds facts and scenarios without weakening comparisons.

Hosted coverage run `41276441854`, source
`9d0c0ab2facc89433254fe05db2dce6a04859ea4`, reports 63,145/94,688 engine lines
and 6,374/9,187 functions reached by the corpus. The full 100% route alone reaches
54,411/94,688 lines and 5,586/9,187 functions. It supplies no separate complete
any% or low% function coverage. T3/T4 subsequently qualified the Chozo hands,
Kihunter wings and baby Metroid remorse, so those stale cold entries are excluded.
Artifact SHA-256: `92a525e5a74e385caf01a1060419a855b89bcf992052ed9a45c4f43b17a97fd9`.

The first bounded batch targets `metroid::touch` (35 cold lines in that report)
and `botwoon::head_spit` (34). The two published rows require fresh latch/drain
and complete spit-volley facts. The bomb-release candidate did not demonstrate
release and remains private; it contributes no coverage claim. Source
`e138b9130ff4abdf7838a5c41f7f391747d261a12102edd6a17ef2ac3f81439b`
passes patched and faithful lockstep on both rows with zero failing frames.
The Metroid row runs 733 ticks: room DBCD at 0, fresh latch at 111 and drain at
316. The Botwoon row runs 901 ticks: room D95E at 0 and complete spit at 634.
These named events establish reached behavior, not a measured line-coverage
increase; a fresh hosted coverage run is needed to measure that increase.

Canonical DLL SHA-256:
`45c69328801207aaf4ae69d8acba1282e83df64464bd509e8b3881eb6ffb6334`.
The DLL runs all 148 registered unit checks, zero failed. Both rows also pass
faithful free-run (733 and 901 ticks, zero divergent or outside the reference)
and engine check-lock (13 streams each). Free-run compares machine storage,
video/audio, Samus, camera, room, progress and actors; input and effects are
excluded there and retained by paired lockstep. The named facts read verifier
snapshots; they do not introduce target-package probes or claim an owned
engine projection for RAM fields.

The final encrypted two-row bundle is
`coverage-v1-metroid-botwoon-82d7b3c1a2ef44f5.tar.gz.enc`, ciphertext SHA-256
`82d7b3c1a2ef44f523c8177d760e4cfd07117813fc0b98f0177916a2be8e6449`.
A fresh-folder restore compares all 18 private file identities and both
seed/input hashes with the originals. `scenarios/lock.toml` carries the exact
seed/input pins and faithful digests; it validates 691 pins, 32 bundles,
632 locked and 41 unlocked scenarios at this batch's base.

The deliberate Metroid mutant changes only `damage_player(1)` to `(2)`.
Mutated source `143c6cbce3d21011fb515157c0d75274d1ac00ece42347c6567df4dd815ca5cd`,
DLL `562023c635c51f4d0cb7eeccd6b35aa08bfe5da0b9afe6f83478317b1c7b74b1`:
faithful lockstep rejects it at tick 109, domain `samus`, field `samus.health`,
with 98 failing frames and 98 matching typed divergences. The touch handler
drains before latching, so that first failure precedes the anchored-drain fact
at 316. An independent engine-only run completes 733 ticks, retains room and
latch facts, and rejects the missing `metroid:drain` fact. The source is then
restored byte-for-byte; neither this mutant nor its DLL enters the batch.

The independent Botwoon mutant changes only the spit loop from 5/3 births to
4/2. Mutated source
`ce16df3e6fdbbcb6c12d1d0ba74bea752142c608638f95d1b1a748351dcb375c`,
DLL `9b83a314115cfd1b5eabfc87e3828d440abfa5f3c08bbf1b0399138a2bf99ac3`:
faithful lockstep rejects two frames, first at 634, with a missing `EPROJ_ID`
slot at WRAM `$7E1997` and OAM/video differences. Existing typed observations
remain equal because enemy projectile fields have no typed projection in this
batch. The independent engine-only run completes 901 ticks with the room fact
and rejects the missing `botwoon:spit` fact. Both gameplay files are restored
byte-for-byte. The preserved canonical DLL passes the existing source, compiled
library-source and native guards with no rebuild or restamp.

The change adds 215 net verifier Rust lines and 16 Python recipe lines. It adds
zero production gameplay lines, unsafe lines or target-package RAM coupling
sites. The local ratchet reports 2,138 sites and 98 coupled files, below its
2,145/101 baseline; gross production is 178,883 lines against that tool's
173,506 baseline. That baseline difference predates this verifier-only batch.

The final canonical lane gate passes on the restored source: formatting,
mutation anchors, target-package Clippy, target crates, detached host, both
compat configurations, verifier metadata and compiled source inventory. The
ROM import test compiles; Windows blocks its executable, so its execution is
pending hosted CI. PR CI and the full corpus remain pending on the published
composition. All local replay and mutant receipts above retain their own
source/DLL identities.

The next bounded assignments, in order:

1. The separate low% coverage owner extends its existing genuine capture inventory: enumerate all room
   boundaries and map which imported input windows exist, then qualify a small
   adjacent room batch with fresh arrival/completion facts. Begin with the rooms
   surrounding low% Botwoon and Draygon; preserve original capture/input hashes.
2. Add movie-specific completion for any% and low%: continue each final original
   seed/input through ship departure and credits; require fresh completion facts.
   A 100% ending seed cannot substitute for either movie.
3. Cover Metroid ice freeze then missile shatter, and the alternate Botwoon spit
   fan if the first row reaches only one pattern. First establish the event on
   the faithful oracle, then catch a one-line behavior mutant.
4. Resolve retail reach for grapple spike contact/anchored placement and Etecoon
   ceiling collision using the retained T3/T4 geometry findings before authoring.
   No impossible enemy placement earns gameplay coverage.

Keep a dedicated gameplay coverage owner beside the engine lanes and replenish
it with fresh bounded agents at task completion. Report newly reached behavior,
mutation sensitivity and movie-wide gaps separately from passing-row totals.
