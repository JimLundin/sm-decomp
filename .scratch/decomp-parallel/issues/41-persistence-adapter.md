# Implement full WRAM/SRAM verification and fresh-process persistence

Type: task
Status: open
Blocked by: 11
Triage: ready-for-agent

Use the completed [state/ABI contract](../run-20261013/research-19/proposed-children/02-persistence-adapter.md).
Generic family currently lacks SRAM input/output. Implement the Rust adapter,
model, host/ABI and tests over131072-byte WRAM/8192-byte SRAM, raw returns and
ordered writes/disk service outcomes. Save requests persistence once; invalid
load requests none. Whole-call refusal preserves state; accepted service error
is fatal without C retry. Reuse canonical state and reviewed exact ownership.

Direct offline full-state fixture support is implementation/verification work;
live root21 and actual fresh-process file-only import are later integration
gates. Publish supported argv, meaningful bounds/short-file/service controls,
tests60s/merged3600s, both cleanup passes and retained source/readers. No further
subsystem inventory research is required for the recorded four-routine codec.

## Completion evidence

Pending Rust adapter and passing controls; sealed research19 facts add no coverage.
