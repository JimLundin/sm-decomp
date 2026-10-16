# Whole-batch verification performance assessment

Status: resolved

Investigate the user's slow whole decomp batch / integration verification and
recommend measured improvements. Read current source and retained performance
evidence; run bounded read-only CLI probes. Do not alter the active Offline
family extraction, gameplay, production installation, policies or claims.

Baseline: main `5c011eb`. The original planning probe took 32.03 seconds wall,
29.68 seconds user CPU and 1.67 seconds system CPU:

```sh
/usr/bin/time -f 'probe_wall_seconds=%e user_seconds=%U system_seconds=%S peak_rss_kib=%M' timeout 40s target/debug/sm-pipeline batch plan --profile offline-all --changed-since HEAD~1
```

It exited successfully. Changed tracked files were five architecture planning
documents. Many families remained selected through `uncertain-inputs`.

This is an advisory throughput assessment, not a functional regression fix.
Existing correctness controls and bounded elapsed measurements provide the
feedback loop; there is no user-specified latency threshold. No complete
catalog verification or new gameplay acceptance is required for this assessment.

Work: [01](issues/01-assessment.md).

## Completion evidence

Assessment complete; see [findings.md](findings.md). Current planning selected
160 of 177 families; 158 first refusals come from documentation-comment words.
Bounded startup/catalog/planning timings and the unsuccessful stripped-plan
comparison are retained with their limitations. No whole-batch speedup is claimed.

Both scoped utility cleanup passes succeeded with zero eligible bytes. Three
identified disposable probe binaries were removed (444,798,584 logical bytes;
444,616,704 allocated bytes represented by those files). Final observed free
space was 146,634,555,392 bytes. Source, reports and diagnostics remain; no
verification producer or new acceptance evidence was created. Source hashes
matched before retirement; `git diff --check` passed. The main Git/index and
active source owners were not changed.
