# Register and exercise the delivered equipment renderer

Type: task
Status: open
Blocked by: 11, 34
Triage: ready-for-agent

Eight routines are committed on main at `fa6e9cd747acabcb4568ccd2c0fbf532a9525c18`,
with ten Rust tests and607 full-WRAM/raw-return actual-C pairs passing.
[Coverage task02](../../decomp-coverage/issues/02-pause-equipment-render.md)
owns the exact source submission; do not reimplement it or claim its files again.

The durable integration job is `integrate-pause-equipment-render-offline`.
The original integrator registers public lib/catalog/caller ownership after
held Ceres integration, then checks the real pause schedule and ordered hardware
effects. Adapter34 is a live-integration dependency; offline source and verified
coverage are already delivered. Required tests/cleanup/source retention and
independent integrated/live publication precede resolution.

## Completion evidence

Delivered source and offline verification:
[result](../../decomp-coverage/renderer/coverage-result.toml),
[submitted receipt](../../decomp-coverage/renderer/submitted.stdout.toml).
Public and live integration remain pending.
