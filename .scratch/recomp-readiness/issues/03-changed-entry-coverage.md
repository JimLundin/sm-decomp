# 03: Gate changed-entry coverage and report compiled progress

**What to build:** A candidate replay proves which address/MX keys executed and rejects additions or changed bodies that were never exercised.

**Blocked by:** 02: Run cold fallback and generated parity with complete evidence.

**Status:** done

- [x] Count actual generated and interpreted instruction execution by address/MX and opcode/pattern, recording generated/interpreted totals and fallback fraction.
- [x] Inventory generated executed/unexecuted keys, unsupported instructions and deferred work; keep call-target discovery separate from instruction counts.
- [x] Bind changed-key requirements to the preserved predecessor/candidate identities and actual generated bodies/config, including newly enabled keys and changed existing bodies.
- [x] An added never-executed key yields uncovered even when legacy compiled entries execute; an empty table or zero actual compiled execution cannot pass.
- [x] Legacy unexecuted readiness-baseline variants remain explicitly unvalidated inventory and cannot count as new validated progress.
- [x] Report comparable deltas only for identical ROM/movie/profile/seed/interval; aggregate counts or generation success do not waive key coverage.



## Completion evidence

Actual opcode reads now populate native/emulation address/MX and opcode/pattern
profiles; generated accounting occurs only after the real timed body executes.
Interrupt takeover and idle wait/stop dispatch are excluded from instructions.
Immutable generated body/table/config/source inventories establish changed-key
requirements, with source/table/predecessor corruption and incomplete scopes
rejected. A newly added unexecuted key is uncovered despite legacy execution;
unchanged baseline keys remain unvalidated inventory. Progress deltas require an
identical complete input domain. Call-target discovery remains separate.

Validation: eight focused public command/Subject tests and five coverage gate
behavior tests pass; repository Python checks, affected all-target Clippy,
formatting and diff check pass. Fresh generated 300-frame replay and coverage
gate passed in run `20261010T082028.708754Z-d04b8b493cea`: 453 actual generated
instructions, 4,631,901 interpreted instructions, three validated distinct keys,
three legacy unexecuted keys. This focused verification is not the final
18,000-frame readiness result; ticket 12 exercises the final integrated source.
