# Samus animation and input transitions
Status: claimed
Blocked by: 05
Triage: ready-for-agent

Translate substantial complete Samus animation delay/bytecode and speed-booster
state, liquid FX/splash/bubble routines, palette copying/loading and speed-boost
cancellation, pose-table input matching/cancel/momentum and input wrappers.
Read /tmp/sm-port-samus-closure.md for actual dependency closure. Preserve genuine
audio/physics dependencies, ROM tables and WRAM aliases. No fabricated collision
callbacks or default animation handlers. Differential actual-C evidence and
independent state-machine tests required. Own samus_control.rs and control.json.
