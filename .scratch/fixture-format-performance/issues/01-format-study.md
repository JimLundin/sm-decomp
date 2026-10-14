# Measure Fixture format alternatives

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research

## Question

Can a new performance-first format remove repeated parsing, allocation and patch reconstruction without weakening fixture semantics?

## Scope

Own only .scratch/fixture-format-performance and a private temporary benchmark directory.
Do not edit pipeline, gameplay, production policies, existing evidence or other worktrees.

## Validation

- Parse the same actual Manifest contract as the pipeline.
- Preserve all metadata, complete case membership and ordering.
- Compare resulting full WRAM for roots and for continuations with varied inherited inputs.
- Measure actual read, SHA-256, decode/validation and materialization work.
- Do not present input preparation timings as end-to-end actual-C verification speed.

## Comments

The architecture review measured 276 MB of tracked port TOML and approximately five million byte-write arrays.
The user has explicitly prioritized performance and removed legacy-format compatibility as a requirement.

## Answer

Yes. Indexed merged binary spans substantially reduce authenticated loading and
modeled state preparation. See ../findings.md and ../results-merged.json. Binary
fixtures are the user-selected source of truth. Production migration is ticket 02.

## Completion evidence

Both locked offline Rust release experiments completed successfully. Full state and
metadata comparisons passed for 14,573 preparations, including 4,032 continuations.
No actual-C or gameplay speed claim is made by this study.

Preserved the exact executable, nine final binary variants and results under
out/fixture-format-performance/20261014T191818Z and this study directory. SHA-256
verification of retained.sha256 passed before cleanup. Snapshotted original family_model.rs
in benchmark/src/original_family_model.rs so future production changes do not alter
the experiment contract.

Cleanup commands used /home/jim/sm-decomp-production-execution/bin/sm-pipeline:
cleanup --root /tmp/sm-fixture-format-0cZilr --apply and the same with
--deduplicate --apply (both successful, zero utility candidates). With writers finished,
removed that exact private directory using rm -r. It contained 1,451,402,116 bytes of
disposable benchmark corpora and build files. Available disk afterward: 174,314,188,800
bytes. Cleanup logs are retained beside the results. No project evidence or unrelated
session outputs were removed.

The original source entries in `../retained.sha256` referred to live workspace
paths subsequently changed by ticket 02. Their exact original bytes have now
been reconstructed and checked against that checkpoint under
`out/fixture-format-performance/20261014T202443Z/source-original/`.
`original-study-relocations.json` maps the old paths to the retained copies;
`../retained-final.sha256` seals the current retained inventory.
