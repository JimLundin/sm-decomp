# Actual PLM callback collision harness
Status: claimed
Blocked by: 07
Triage: ready-for-agent

Build the actual-C callback seam described in /tmp/sm-port-collision-closure.md.
Synchronous PLM setup must preserve full WRAM, displacement mutation through
cur_coll_amt32 and callback return/effects; no record-only substitution. Avoid
overlapping mutable Rust/C borrows by explicit state transfer. Compile actual
pinned bank84 and dependencies, retain identities, and measure remaining C hits.
Own tools/port_collision.py and tests plus harness documentation.
