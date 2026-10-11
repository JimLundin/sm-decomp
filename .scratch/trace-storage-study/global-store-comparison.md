# Comparing a global content-addressed store with bounded packs

The competing proposal is a good **logical storage model**: content identities,
shared pages/payloads, per-frame lookup and a run manifest rooted in a hash.
It can share partial matches across runs better than independent packs. I would
adopt that direction, while keeping compact compressed packs as the **physical
storage format**. A content-addressed store and a packed binary encoding solve
different problems and can be combined.

The original recommendation intentionally started with self-contained packs
because they simplify retention and portability. This comparison supplies new
evidence that sharing across runs is useful. It also shows that choosing Merkle
trees does not, by itself, choose an efficient physical representation.

## Comparison

| Question | Bounded page/event packs | Proposed global store |
| --- | --- | --- |
| Redundancy within a run | Measured strong savings from binary events and page pools | Same basic savings, plus reuse of interior nodes |
| Redundancy between partly matching runs | Whole-pack sharing only, unless exporting common objects | Shares matching pages, tree nodes and payloads even when runs differ |
| Equality and locating differences | Hash reconstructed state; compare page tables/pages | Compare canonical roots; descend unequal tree branches to locate changed ranges |
| Physical storage | Compact integer page references and grouped compression | Needs an object packing/compression/index strategy; a file per tiny page is expensive |
| Retention and portability | Run is self-contained; easy to retain or copy | Must retain/export every reachable object and collect only unreferenced objects |
| Implementation work | Writer/reader plus existing consumer integration | Same work plus concurrent object publication, root publication, GC and pack management |

## Direct experiment

`tools/storage_merkle_study.py` reads all 870 + 132 WRAM snapshots and 130 + 10
complete frame snapshots from the same two retained C captures as the earlier
study. It creates no object-store files; candidate objects are in memory and
only compact measurements are retained. All snapshots reconstruct exactly.

The experimental Merkle tree uses 256-byte leaves, fanout 16, SHA-256 IDs,
explicit type/length prefixes and an exact-length snapshot root. Each unique
object is stored once. The physical estimate compresses one hash-sorted pack
containing object ID + u32 length + object bytes with system libzstd 1.4.4,
level 3. It omits a random-access index and per-run names/manifests.

Results in [merkle-estimate.json](merkle-estimate.json):

- Across both captures: 43,241 page objects, 12,422 branch nodes, 771 distinct
  snapshot roots; 56,434 total objects. This includes distinct partial-page
  lengths; the flat experiment pads the last page and records member length.
- All 10 frame snapshots in the room capture introduce **zero new objects**:
  each already occurs in the longer projectile capture. Cross-run sharing is
  demonstrably useful. Existing full-file hardlinks can also share exact whole
  matches; the global store additionally shares partial matches.
- The simple Merkle pack occupies **6,148,639 bytes**. For the *same complete
  snapshot set*, a global flat pool with u32 page IDs, exact lengths, all member
  page tables and grouped Zstandard compression occupies **1,230,236 bytes**.
- Earlier bounded pools occupy about **3.1 MB**, including the earlier first
  128 room WRAM snapshots rather than all 132. This is a useful approximate
  comparison, not identical membership. The two new global candidates use
  exactly identical membership.

These are experiments in physical packing, not proof that Merkle structures are
intrinsically larger. Hash-sorted packing loses page locality; 32-byte child
hashes, repeated branch references and object indexes add metadata. A Merkle
node with 16 children contains 512 bytes of hashes before its header, while
16 local u32 IDs use 64 bytes. Better ordering, deduplicated tables, local-ID
translation or pack deltas can improve the Merkle pack. The flat estimate uses
one whole-store dictionary/compression frame and therefore does **not** establish
bounded memory, seek speed, concurrent-write throughput or actual filesystem
allocation. Neither candidate is a production reader/writer.

The new conclusion is that a global store has a promising savings opportunity,
but a literal hash-reference encoding should be benchmarked rather than assumed
smaller. Keep stable logical object hashes and permit local compact references
inside physical packs; verify them against the logical identities on reading.

## Corrections to the competing claims

**“A new snapshot costs only its changed pages.”** More precisely, it costs
previously unseen page content, changed ancestor nodes up to the root, and the
snapshot/frame manifest entry. Changed pages already present elsewhere add no
new page payload. The cost also includes hashing/compression/index work. A
Merkle tree can update just the affected paths if the writer knows changed
ranges; hashing a full captured image still reads its bytes. Stable page size,
tree shape, final-page handling and serializer identity are part of equality.

**“Comparing two images means comparing root hashes.”** This establishes equal
canonical image content under the usual cryptographic-hash assumption, once
roots have been computed from or verified against each actual image. A trusted
root comparison is cheap; constructing/authenticating roots is not free. Root
inequality needs tree traversal for a first-difference report. Availability is
separate: a root hash does not prove the referenced pages are still present.
Keep the existing flat SHA-256 of complete decoded snapshots for migration.

**“Each upload burst is one record.”** This is valuable only as an exact encoding
of **observed** writes. The current port observer records PPU selector, address,
value and order; it does not supply explicit upload-burst boundaries
([port_live.py:594](/tmp/sm-trace-storage-study/tools/port_live.py:594)). Grouping
must preserve repeated writes, alternating registers, interleaved PPU/APU
effects, starting state and frame/reset labels. Store the literal ordered
address/value sequence as a shared payload first. Add specialized burst tags
only when exact expansion reproduces every original record. Never replace the
observed destination writes with “source payload X was uploaded” unless the
observer also verifies those writes; that would weaken validation. CPU traces
also have timing fields that this native port profile does not.

**“The root is deterministic by construction and covers everything.”** The
hash algorithm is deterministic; the object graph must also be canonical.
Specify type/version tags, integer widths, key order, tree shape, chunk/burst
boundaries, final-page length and what metadata belongs in each identity.
Hash canonical decoded content so codec upgrades do not change logical IDs.
Physical compressed bytes remain separately authenticated, with no assumption
that a codec upgrade preserves them. Merkle closure commits to referenced
content only when every referenced object is verified during reading. It does
not authenticate omitted records, missing completion, or provenance that the
root never referenced. Require frame/event/member counts and completion status.

Use a **run root** for provenance/configuration, observations and completion,
and a separate **observation root** for expected equality. Independent C and
mixed Rust runs should have different run roots because their producer identity
and ownership counters differ; their comparable observation roots should match
when validation passes. Current validation checks observation inventories,
progress and execution counters separately
([port_live.py:697](/tmp/sm-trace-storage-study/tools/port_live.py:697)). Include
these requirements explicitly rather than replacing them with one undifferentiated
root comparison. Store completion after objects/indexes are durable.

**“One store for all of out/ replaces hardlink dedup.”** It can replace hardlink
deduplication for immutable evidence artifacts. Mutable logs, active runs and
disposable compiler outputs still need their own lifecycles. Do not retain all
build products merely because they can be given content hashes. Root retention
must include failed diagnostic evidence and historical source dependencies.
Publish objects and roots atomically, protect active writers during collection,
verify existing objects before reuse and retain an exportable dependency closure.
A global store increases the impact of deleting or corrupting a shared object,
so these mechanisms are completion requirements, not optional cleanup polish.

Independent executions remain independent. Each producer must observe/hash its
own writes and state; it can deduplicate its authenticated output afterward.
It must not reference the expected run's objects instead of observing its own
execution.

## Revised recommendation

Use **content-addressed logical artifacts backed by compressed binary packs**:

1. Snapshot identities use canonical Merkle pages/nodes; events use exact ordered
   binary sequences with optional lossless burst/payload references. Keep legacy
   decoded hashes alongside new identities during migration.
2. Immutable pack files store pages, nodes and trace chunks compactly. Translate
   content IDs to local integer references where helpful, and keep a verified
   content-ID-to-pack-location index. Add bounded chunks and frame seeking.
3. Run manifests retain provenance and completion, and reference comparable
   observation roots separately. Retained manifests pin their dependency closure.
4. Build an opt-in store for sealed trace/snapshot evidence first. Evaluate
   streaming production, concurrent inserts, missing-object failures, GC under
   active writers and self-contained export before migrating historical evidence.

This adopts the stronger sharing model of the competing proposal and the measured
compact encoding from the original proposal. The next comparison should measure
actual packed allocated bytes, producer CPU/RAM, object-index costs and extraction
latency over many runs; two captures do not establish the best global pack policy.

## Verification and cleanup

Both runs used for the estimate were already reauthenticated against retained
artifact seals in [source-authentication.json](source-authentication.json).
The new script verified all 1,142 snapshot reconstructions in both candidate
representations. It stored no captures, pack files or object-store files.
Cleanup and work status are recorded in
[ticket 02](issues/02-global-store-comparison.md).
