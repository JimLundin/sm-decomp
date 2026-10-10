# 05: Prove and freeze offline translation acceptance

**What to build:** A correct exercised decompression job can be accepted, while wrong behavior, hidden fallback, missing coverage or unreliable evidence fails.

**Blocked by:** 04: Replay captured decompression calls offline

Status: open
Triage: ready-for-agent

- [ ] Report complete write sets and ordered writes where the contract requires them; compare output, surrounding state and relevant transient writes.
- [ ] Execute intentional wrong-output and transient-write candidates and prove the real comparison command rejects them.
- [ ] Execute disabled or unexecuted replacements and missing/reduced fixture or branch declarations; each must fail coverage or remain explicitly nonaccepting.
- [ ] Prove build errors, crashes, timeouts and altered identities preserve failing evidence and cannot reuse a successful verdict.
- [ ] Freeze a minimal versioned offline contract with exact job/source/fixture identity, predecessor lineage and accepted exercised coverage, separate from recomp acceptance.
