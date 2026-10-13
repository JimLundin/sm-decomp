# Specify full-state synchronous NMI hardware service and adapter

Status: open
Blocked by: 11
Triage: ready-for-agent
Type: task

Consumers: 23 and milestones06–08.
Origin: [NMI child proposals](../launch-20261013/research-13/children.toml).

## Question / Work

Prepare an exact reviewed infrastructure scope for the full-state NMI MMIO/DMA fixture/live adapter. Bind complete WRAM, PPU views/latches/VRAM/CGRAM/OAM, DMA channels/pending state, NMI/IRQ flags, timers and open bus. Preserve ordered byte MMIO, reverse-DMA WRAM changes, service failure/no-retry and preflight-before-effects. Reuse existing host consumers and receipt contracts.

The proposal lists hardware_lifecycle_adapter/model/host/ABI files and adapter tests, plus possible shared family/callback/live/input/main/catalog files. Select exact necessary files only after original host completion11 and retained pipeline ownership review. Integrator owns shared registration; this ticket conveys no source scope. Samus collision displacement is a separately declared existing collision-host requirement, never silently lost through header-only SpawnPlm.

## Answer / Acceptance

Publish exact baseline, approved files, source-bound state/order/DMA/latch/read/reverse-DMA and failure/refusal controls, versioned readers and admission rules. Initial cargo test -p pipeline --locked --offline --test hardware_lifecycle_adapter gate60s; merged required family/live controls3600s. Existing NMI scheduling, reset, APU and original-ROM timing remain external gates. Create a reviewed implementation child after this research and complete run cleanup. Full details are in the sealed research13 Answer.

## Completion evidence

Pending reviewed infrastructure scope; no adapter implementation or gameplay stage yet.

## Comments

2026-10-13: user classifies this adapter and its integration tests as a task.
Use the finished source/state contract for implementation, with checked exact
ownership; speculative research and broad refactoring are deferred. Its live
hardware requirements do not block independent reference-C offline families.
