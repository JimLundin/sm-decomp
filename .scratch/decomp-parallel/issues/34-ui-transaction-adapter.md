# Implement full-state menu integration verification

Type: task
Status: open
Blocked by: 11, 21, 22, 33
Triage: ready-for-agent

Use the finished [UI contract](../run-20261013/research-18/proposed-children/03-ui-transaction-adapter.md)
for real equipment/category/options-object/screen signatures, canonical WRAM,
PPU/DMA/OAM/VRAM/CGRAM/SRAM/audio and coroutine state. Preserve ordered service
effects, whole-call refusal and fatal service failure without native retry.
Implement actual-C and exclusive live adapters with independent controls;
publish exact supported argv and checked source ownership before claiming.

This is verification implementation. The eight-routine renderer already has
607 passing offline pairs through maintained generic and direct gates. This
adapter is a later public/live integration dependency and cannot retroactively
block that coverage. Initial tests60s/merged controls3600s, nonzero tests,
both scoped cleanup passes, source retention and post-cleanup authentication.

## Completion evidence

Pending implementation and live acceptance; finished source facts are in
research18's sealed package.
