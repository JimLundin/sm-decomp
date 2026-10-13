# Shared refresh performance observation

Status: measured full catalog refresh in progress; source implementation unchanged.

Command:

```sh
target/debug/sm-pipeline progress refresh --root /home/jim/sm-decomp --source /home/jim/sm-decomp/deps/snesrev-sm > .scratch/decomp-progress/recovery-refresh.stdout 2> .scratch/decomp-progress/recovery-refresh.stderr
```

Process 46322 started at 2026-10-13 04:49:45 local time. Source regeneration
completed at about 04:49:52, producing the current 6,053-definition inventory
SHA-256 `7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`.
At 4m12s elapsed the process was still running with 4m08s CPU and
882,756 KiB resident memory. Intermediate samples are retained in
`recovery-refresh-performance.txt`; these are observed RSS samples, not a
measurement of peak memory.

The full `tools/port/*.toml` catalog contains 275,928,436 bytes. The largest
manifest is `floater-spike-motion.toml` at 22,771,777 bytes. The current
`add_declarations` reads and parses complete manifest TOML documents while
extracting only family and routine-symbol declarations. This observed path is
a plausible cause of the full-refresh cost; it has not been instrumented or
changed during the gameplay verification freeze.

The registry lock remains held during refresh. Concurrent updates would be
subject to the intentional ten-second lock timeout, so this coordinator waits
for the active refresh to finish before publishing. Existing JSON/HTML remains
readable while the replacement is prepared. CLI publication/status does not
rescan the full declaration catalog.

This is a continuous-refresh performance limitation. The isolated browser
verification used the same 6,053-definition inventory and a much smaller
declaration catalog; its six real CLI writers passed within 5,645 milliseconds.
The earlier isolated 24-writer limit is independently documented in
`recovery-browser/report.md`.
