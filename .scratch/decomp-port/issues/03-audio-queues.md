# Port music and sound queues
Status: claimed
Blocked by: None
Triage: ready-for-agent

Port HasQueuedMusic, QueueMusic_Delayed8/Y and all three SFX internal routines
and max-count wrappers. Own crates/port/src/audio.rs, related tests and
tools/port/audio.json. Differential verification depends on ticket 01.
Do not port hardware-facing HandleMusicQueue yet. Preserve wrapping, scratch
state, full ring replacement priorities and game/debug/power-bomb gates.
