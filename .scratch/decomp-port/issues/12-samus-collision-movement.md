# Complete Samus collision and movement closure
Status: claimed
Blocked by: 05, 08
Triage: ready-for-agent

Translate bank94 complete collision traversal/reactions/slopes, bankA0 solid-enemy
collision, then bank90 movement controllers and dispatcher. Preserve real PLM
callback semantics through ticket11, room/BTS/door data, fixed-point amount and
directional effects. Scope context: /tmp/sm-port-collision-closure.md. Own
collision.rs and tests/contracts. Unknown tiles/callbacks cannot return defaults.
Differential valid synthetic/captured cases plus exercised live behavior required.
