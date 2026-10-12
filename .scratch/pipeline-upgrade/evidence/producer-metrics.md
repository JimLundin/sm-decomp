# Ticket 02 text-producer metrics support

Support branch `pipeline-upgrade-02-producer`, based on integration `09d3ed1`.
This completes only native text flush telemetry; ticket02's process accounting,
collector/publication stages, aggregate receipts and overhead gate belong to the
main metrics implementation.

`effect_writer.c.in` keeps the exact `vfprintf` legacy format and uses one GNU
stdio cookie with the same65536-byte buffer in enabled/disabled modes. Its write
callback limits batches to64KiB, retries interrupted/short writes, checks zero
writes/errors, and takes monotonic clocks once per flush batch only when enabled.
SIGPIPE becomes a checked EPIPE error; failed writes/open/abandon retain a best
effort failed sidecar. Explicit close and sidecar durability failures return
failure. The GNU feature macro precedes every generated live-host include.

The parent's explicit absolute `SM_PIPELINE_METRICS_PRODUCER_PATH` is outside the
evidence staging tree. `SM_PIPELINE_METRICS=enabled|disabled` selects clocks in
the same binary, not text behavior. The `pipeline-producer-metrics-v1` TOML
contains enabled/buffer bytes, transported bytes, flush/write counts, total/max
flush write elapsed nanoseconds, short writes/interruptions and outcome/error.
The parent maps disabled timing fields to unavailable. Sidecars publish with
file fsync, rename and directory fsync. These fields do not enter canonical
observation/provenance inventories.

## Verification

TDD at the user-confirmed process/metrics and evidence capture seams: initial
exact-byte test failed because the producer receipt did not exist; the open
failure receipt slice also failed before its implementation. Six public
controls passed afterward (final1.55s):

- Same producer binary with telemetry on/off preserves known signed PLM,
  maximum PPU/APU fields, frame resets,70KiB literal and10000typed text events.
  Current retained evidence CLI capture derives equal stream IDs, observation
  roots and member hashes against an independent legacy text reference.
- A system `write` linker control injects one EINTR and seventeen-byte short
  writes, refuses any call over65536bytes, and restores an exact140KiB literal
  plus2000APU events; receipts record retries and success.
- Real closed pipe, abandoned stream and failed destination open record failure.
- A missing sidecar directory makes explicit close refuse success after exact
  effect bytes were flushed.
- Public `packed-capture` refuses a nonzero real C producer, withholds
  `evidence.toml`, and leaves diagnostic staging/logs.

Final logs: integration `out/ticket-02-producer/logs/producer-controls.log` and
`producer-clippy.log`. Strict test Clippy, workspace formatting and whitespace
checks passed. C harness compilation uses `-std=c11 -O2 -Wall -Wextra -Werror`.
The unchanged evidence reader came from ticket01 retained-tools. No benchmark
or <=2% overhead claim is made by these correctness controls.

## Cleanup

Test guards removed their own successful/failed temporary runs after processes
finished; native scoped cleanup/dedup of integration
`out/ticket-02-producer/runs` both succeeded with0bytes/0snapshots (receipts in
logs). `cargo clean --target-dir .../out/ticket-02-producer/target` removed1842files
reported1015MiB; allocated target before clean922877952bytes, filesystem free
afterward221680988160bytes. Logs remain; no retained observation/store/source
dependency was created by these disposable tests. Borrowed ticket01 reader and
its dependencies remain unchanged. The integration owner removes the integrated
support worktree after merge, following run-cleanup.md.
