# Port music and sound queues
Status: resolved
Blocked by: None
Triage: ready-for-agent

Port HasQueuedMusic, QueueMusic_Delayed8/Y and all three SFX internal routines
and max-count wrappers. Own crates/port/src/audio.rs, related tests and
tools/port/audio.json. Differential verification depends on ticket 01.
Do not port hardware-facing HandleMusicQueue yet. Preserve wrapping, scratch
state, full ring replacement priorities and game/debug/power-bomb gates.

## Completion evidence

Implementation 703a590 merged in 1c875b4: 21 routines, six independent Rust
behavioral tests and 1,147 actual-C fixtures. Exact committed manifest evidence:
/tmp/sm-audio-committed/20261010T194908.663619Z/verdict.json.
Deliberate wrong quota mutation rejects at WRAM $0653; evidence is
/tmp/sm-audio-reject/20261010T194832.763528Z/verdict.json.
