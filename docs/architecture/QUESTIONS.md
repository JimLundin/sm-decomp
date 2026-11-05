# Questions for Jim

Architecture lane, `research/architecture`, 2026-11-05.

These are the only open questions in the architecture that are about goals or values. Every
engineering choice is decided and recorded in [ARCHITECTURE.md](ARCHITECTURE.md), and you
can veto any of them there. Neither question below blocks the foundation. The architecture
supports every answer, and each answer only changes what gets built, and when.

To answer, write under **Answer:**. "Yes" accepts the recommendation.

## Q1. May widescreen change gameplay?

**What it means.** On the console, the screen is 256 pixels wide, and the game only runs
enemies that are on screen or very close to it. An enemy further than about 256 px to the
right of the camera's left edge is frozen: it doesn't move, animate, shoot or get drawn
until the camera comes close (bank log `$A0:8EB6`, "Determine which enemies to process").
Samus's shots are also deleted when they get too far off screen (`$90:B16A`). A widescreen
picture shows about 70 px more on each side. That margin is outside the original's "on
screen" window, so whatever is in it is frozen.

**A concrete example.** In a Brinstar corridor, a Zeela crawls along the wall 300 px to the
right of the camera's left edge. On the console it sits frozen off screen, then starts
crawling when you walk toward it. In widescreen it is already visible in the right margin.
Then one of these happens:

- **(a) Wide view, gameplay unchanged.** The Zeela is either hidden until it reaches the
  original window (it pops in about 70 px inside the picture's edge) or shown frozen in its
  last pose (it starts moving when it reaches the original window). Everything else is
  exactly the original game: the same enemy timing, RNG, damage and strats, and the oracle
  proves it on every row.
- **(b) Wide gameplay.** The Zeela wakes up as soon as it enters the wide view, and your
  shots travel further before they vanish. This is no longer the original game. Enemies
  wake earlier, so RNG and room timing change, some strats and speedrun routes change, and
  no oracle can check it, so it is tested by its own scenarios. It would be a gameplay
  option, like Map Rando's settings, and not "vanilla".

**What each answer changes in the engine.**

- **(a) only:** the wide renderer in `runtime` and a presentation setting for margin
  enemies (hide or show frozen). The rest is already designed and verified (ARCHITECTURE
  D16, ROBUSTNESS §1). No gameplay code.
- **(a), with (b) as an opt-in option later:** the same, plus one `Viewport` value in the
  gameplay rules that every "is it on screen" rule already reads. Building (b) means setting
  that value wider and writing its scenarios. Room edges and camera limits need a design
  pass when it is built.
- **(b) as the meaning of widescreen:** widescreen could never be verified against the
  original, and it would split widescreen players from vanilla players in races and
  routing.

**Recommendation:** (a) as what "widescreen" means, with margin enemies **hidden** by default
(pop-in is how the original behaves at its own edge) and "show frozen" selectable. Keep (b)
possible as a separate, clearly non-vanilla gameplay option, built only if players ask for
it. The architecture already supports both, so this decides priority, not structure.

**Answer:**

## Q2. Should playing the game reproduce the original's slowdown?

**What it means.** On the console, the game sometimes can't finish a frame's work in time,
and the picture repeats a frame. Players see this as slowdown: in busy rooms, with power
bombs, in some boss fights. Separately, door transitions take as long as the console takes
to decompress the next room. Our new oracle (a real emulator) reproduces both. In
verification, the engine follows the oracle's slowdown exactly, so the gate is unaffected
by this question. The question is what you want when someone *plays*: our engine is much
faster than the console, so without a deliberate model it would never slow down.

**A concrete example.** In Kraid's room, or with three power bombs going off, the console
visibly slows down for a moment, because some frames repeat. Speedrunners and the Map Rando logic
know this. Map Rando's tech list includes `canUsePowerBombLag`, and some TAS routes use the
extra frames. In a plain engine the same moment runs at a smooth 60. Door transitions are
similar: on the console, a door into a large room takes longer than a door into a small one.

- **(a) Reproduce both.** Load times are exact, through the model in ARCHITECTURE D9, which
  is checked frame by frame against the oracle. Busy-room slowdown follows a model measured
  against the oracle, approximate at first and refined over time. A comfort option turns
  slowdown off.
- **(b) Load times exact, no busy-room slowdown.** Doors take console time, gameplay never
  slows down. Tricks that rely on slowdown behave differently in play (but not in
  verification).
- **(c) No slowdown at all.** Doors are as fast as our engine (a "quick transitions" option,
  ARCHITECTURE D19). This is a quality-of-life setting, not vanilla.

**What each answer changes in the engine.** The architecture supports all three: the timing
source and cost model are already in the design (D8, D9). The answer decides what the
vanilla play profile defaults to, and how soon the busy-room part of the model is built.
Loading costs are a short table (on the order of 10-30 rows). Busy-room costs need a
measured cost per enemy species, projectile kind and collision kind, which is a larger
measurement effort, comparable to one domain cluster.

**Recommendation:** (a) as the long-term meaning of vanilla play, reached in two steps. Load
times exact from the door cluster (build step 2), which (b) gives us. Then busy-room slowdown
once the corpus is mostly green, with a comfort toggle to turn it off. The reason: "the
original game, faithfully" includes how it feels under load, and the trick and routing
community depends on it. Until busy-room slowdown lands, vanilla play is (b), and that
difference is listed in LEGACY.

**Answer:**
