# Launch seal metadata closure

Status: complete. This repair changed only the two B session launch seals in
`../execution.toml` to the existing supplement metadata with absolute prompt
artifact paths, then refreshed the execution references in the outer drill.
The original canonical execution, outer drill and retained authentication were
preserved in `../history/before-launch-seal-repair-20261013/` before repair.
The post-publication interruption proof remained byte-identical; no runtime
control or source changed, and no additional control observation was needed.

## Completion evidence

The retained reader command
`.scratch/decomp-parallel/drill-retry/bin/decomp-drill-evidence authenticate`
passed with 317 artifacts. Its output is `retained-authenticate.toml`.
The independent reader command
`.scratch/decomp-parallel/drill-retry/review/bin/decomp-drill-independent-review /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry /tmp/sm-decomp-concurrency-drill-retry-20261013 /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/concurrency-drill.toml /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/retained-authentication.toml`
passed with 318 artifacts, twelve matching launch UUIDs, all eleven control
names, two integrations, unchanged eighteen receipt inventory, and complete
jobs. Its output is `independent-authentication.json`, SHA-256
`da20fa46873233599fc64215acea65261d3afd8755896bbcf60b8fb4cda50794`.

Final canonical SHA-256 values:

- Outer drill: `63912c67e38680fd163d09126ba285f8ce26da8a2445c801f489f8971b74751f`
- Execution: `805dde33a142cc10dc9d9c3c2d6791d1589639aebaebb9feaba7a71dbbccb67a`
- Authentication: `fc023a985e6dc5c9731c61f9d30ead4546daeabf53dd04e585fa368c8182caff`
- Unchanged interruption proof: `6735bb4c95e4ec1581ec5b13fb61b96ed2e5621b261f181decff49c68eb6b451`

After both readers finished, scoped cleanup ran the retained Rust utility with
`cleanup --root /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/metadata-closure-20261013 --apply`
and the same command with `--deduplicate --apply`. Both succeeded and reclaimed
zero bytes; outputs are `cleanup.toml` and `deduplicate.toml`.
`disk-after.txt` records 221,230,288,896 bytes available. This task created no
build output or worktree. Retained fixture history, frozen source, source
snapshots, readers, sealed artifacts and historical manifests remain required
dependencies. Cleanup changed none of the authenticated canonical evidence.
