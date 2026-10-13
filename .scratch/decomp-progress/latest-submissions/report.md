# Latest submissions report completion

Status: resolved
Completed: 2026-10-13T15:41Z
Source commit: 52f44a1a7599f540397dfbd24a31ff8b3ef5f19e

The existing report at http://127.0.0.1:4185/ now has a latest-ten table directly
below its header. Newest publication appears first with UTC time, scope/stage,
what it added and expandable routine/evidence details. Normal seven-second
registry refresh and the JSON picker replace the table. The existing report,
denominator, stage counts and registry observations remain intact.

All 20 existing Rust progress tests and final served desktop/phone checks passed.
The latest host observation is present and adds current verification evidence for
five previously verified roots, with no duplicate coverage. This report change
earns no gameplay stage. Full completion evidence is in
[ticket07](../issues/07-latest-submissions.md).

The exact 99-file, 31,048,909-byte verification package is retained in `artifacts/`
and stored by the Rust evidence crate in `/home/jim/sm-decomp/out/evidence`:

```text
run 62659567eae8a095738b84324898e629012c0e6871875bb2bab7b257231b7ccd
observations 2f3f7e97d030c20ea220688c3cb63de906c2bd84e9a42e330f7e1db11542629e
```

Retain the shared store and exact `bin/sm-evidence` reader. Reauthenticate or
extract the full package with that reader; the destination for extraction must
be new:

```sh
.scratch/decomp-progress/latest-submissions/bin/sm-evidence inventory out/evidence 62659567eae8a095738b84324898e629012c0e6871875bb2bab7b257231b7ccd
.scratch/decomp-progress/latest-submissions/bin/sm-evidence verify out/evidence 62659567eae8a095738b84324898e629012c0e6871875bb2bab7b257231b7ccd .scratch/decomp-progress/latest-submissions/artifacts
.scratch/decomp-progress/latest-submissions/bin/sm-evidence extract out/evidence 62659567eae8a095738b84324898e629012c0e6871875bb2bab7b257231b7ccd /tmp/sm-latest-submissions-evidence-extracted
```

Cleanup passed twice through Rust pipeline, then removed 2,098,678,222 logical
bytes of identified reporting targets/profiles and retired the isolated worktree.
Exact source is retained and matches main; package authentication passed after
cleanup. The intended server PID803313 and its pipeline binary remain available.

Each achieved gameplay stage must be published by `sm-pipeline progress update`
before handoff. The evidence crate handles storage and authenticated inventories;
family/live admission still uses the pipeline's existing semantic checks.
