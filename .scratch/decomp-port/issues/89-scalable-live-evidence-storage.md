# Scalable live evidence storage

Status: resolved
Blocked by: None
Triage: ready-for-agent

Losslessly compress ordered hardware effects through an explicit flag and share excluded duplicate ROM fixtures after finished processes, retaining each actual distinct ROM view and every snapshot path. Preserve historical plain runs and anchor sharing.

## Verification

Compiled C writer output must decompress to independent plain formatted events; changed values, reordered events and truncation must reject. ROM sharing must authenticate every fixture against a retained exemplar of its actual distinct ROM view before any mutation.

## Implementation and verification

`tools/port_live.py --compress-effects` writes every original formatted hardware event to deterministic checked zlib gzip. Plain storage remains the default. Completed excluded ROM fixture paths are preserved as hardlinks only after SHA256 and actual bytes match a retained exemplar of each distinct ROM view; all fixtures are checked before mutations. Storage accounting records counts and reclaimed bytes and the next-process guard includes the observed ROM snapshot peak. Inventory hashing streams all large artifacts. Historical plain and compressed dedup anchors retain separate byte seals.

`python3 -m unittest tools.test_port_live -q`: 14 passed, one opt-in expensive gameplay control skipped. Compiled independent plain-vs-gzip writer verifies byte equality, deterministic compressed bytes, changed value/order differences and truncated stream rejection. Compressed and historical plain anchor sharing controls pass. ROM controls verify all-before-mutation symlink rejection, retained distinct view exemplars and hardlink bytes. Focused rerun: 4 passed. `python3 -m py_compile tools/port_live.py` and `git diff --check` pass. Test TemporaryDirectory scopes automatically removed all compiled outputs. Bounded actual generated-host evidence and cleanup are recorded below; full-movie gameplay admission remains ticket90.

## Completion evidence

The initial compressed run `out/port/live-storage-compressed/20261011T072445.750129Z` exposed a real ROM view change: every decompression snapshot differs from initial rom-view at byte169562. No ROM repair was made. Grouping now preserves each distinct actual view instead of assuming initial identity. The failed diagnostic remains retained and authenticated; its17 snapshots were later losslessly shared (67108864 bytes reclaimed).

Final bounded320 evidence: compressed `out/port/live-storage-compressed/20261011T072613.801279Z/verdict.json`, plain `out/port/live-storage-plain/20261011T072659.503979Z/verdict.json`. Both four-process runs report `state_effects_match:true` and expected zero-leaf-hit rejection; this validates storage, not gameplay admission. All four decompressed gzip logs exactly equal plain log bytes: 9243333 bytes, SHA256 `dd4b8ac478d1526b238f2811efa5a2501bed19bea60931532c988a8331e2d3da`; gzip690263bytes (13.4x reduction). Each final C process shared16 of17 actual ROM snapshots, reclaiming67108864bytes while retaining every snapshot path and patched exemplar SHA256 `886600dd5c645cf7f44b21718dc87dc51908585a2c523f35fda425aca9e28892`.

Cleanup: both bounded scopes `python3 tools/port_cleanup.py --root <scope> --apply` and `--deduplicate --apply` reclaimed0additionalbytes. Retained artifacts, transformed sources, host snapshots and linked binaries were reauthenticated after cleanup against recorded hashes. No current-source admission claimed for earlier snapshots. Full test rerun15passed, one opt-in gameplay control skipped. Platform exact void(uint16) signature allowlist added for ticket90; seven focused signature/storage controls pass.

Authorized historical ticket87 storage cleanup: `out/port/projectile-paired-live/20261011T071554.615327Z/c-0` shared418 of419 identical actual patched ROM snapshots, reclaiming1753219072bytes. Every existing file hash was verified before and after sharing; historical inventory and grouping results are retained in `storage-cleanup.json`. Project cleanup apply/deduplicate both0additionalbytes. Actual remaining disk3.9GiB at completion of sharing.

Final frozen-source tool suite: 15 tests passed, one expensive opt-in control skipped. Bounded actual host storage comparison, cleanup and retained artifact authentication completed; ticket90 owns separate full-movie gameplay admission.

## Follow-up: compressed same-run sharing

Ticket90 full-movie disk constraints required compressed runs to share equal completed observations within the same run. This validates the finished baseline and subject SHA256 seals and exact streamed bytes before hardlinking `hardware.log.gz` and `rom-view.bin`. Changed values/order remain distinct, and gameplay admission is evaluated separately after all four independent processes. Baseline inventory and reclaimed bytes are recorded. Plain default behavior is unchanged. Frozen suite16passed, one opt-in control skipped; focused controls verify changed-value/order preservation and stale-seal rejection.
