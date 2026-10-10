# 07: Restore paired checkpoints validated against cold execution

**What to build:** Focused replay can start from a trusted paired checkpoint, with incompatible or corrupt state rejected and rebuilt cold.

**Blocked by:** 06: Compare focused instruction and bus execution.

**Status:** ready-for-agent

- [ ] Capture complete reference CPU/hardware and subject pending/dispatch state plus input position and relevant counters; do not interchange emulator-core state formats.
- [ ] Bind each checkpoint to ROM/movie/source/profile/serializer identities, settings and its own digest.
- [ ] Validate restoration against at least the next complete cold-executed frame, comparing ordered traces and ending state, and preserve the validation report.
- [ ] Reject incompatible/corrupt checkpoints and rebuild a candidate cache cold; never relabel an old state to imply compatibility.
- [ ] Cross-candidate reuse must precede affected execution and verify equivalent replay history; otherwise require a new cold candidate cache.
- [ ] Report cache-creation and warm focused-replay times separately, targeting 60 seconds without weakening comparison or using cache execution as a substitute for required cold gates.

