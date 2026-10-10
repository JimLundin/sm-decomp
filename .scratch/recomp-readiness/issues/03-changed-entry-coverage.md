# 03: Gate changed-entry coverage and report compiled progress

**What to build:** A candidate replay proves which address/MX keys executed and rejects additions or changed bodies that were never exercised.

**Blocked by:** 02: Run cold fallback and generated parity with complete evidence.

**Status:** ready-for-agent

- [ ] Count actual generated and interpreted instruction execution by address/MX and opcode/pattern, recording generated/interpreted totals and fallback fraction.
- [ ] Inventory generated executed/unexecuted keys, unsupported instructions and deferred work; keep call-target discovery separate from instruction counts.
- [ ] Bind changed-key requirements to the preserved predecessor/candidate identities and actual generated bodies/config, including newly enabled keys and changed existing bodies.
- [ ] An added never-executed key yields uncovered even when legacy compiled entries execute; an empty table or zero actual compiled execution cannot pass.
- [ ] Legacy unexecuted readiness-baseline variants remain explicitly unvalidated inventory and cannot count as new validated progress.
- [ ] Report comparable deltas only for identical ROM/movie/profile/seed/interval; aggregate counts or generation success do not waive key coverage.

