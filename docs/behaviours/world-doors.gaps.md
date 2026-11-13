# Doors between rooms, part 1, the passage: open gaps

What [the spec](world-doors.md) leaves to other domains or to records not yet taken, found while
writing it (observer, 2026-11-13). Each names the rows, the frames and the quantity; the decode
notes keep the addresses. The integrator adds the gaps wiring finds; the observer answers them
in the spec, or in the spec of the domain a gap belongs to.

### G1. Doors up and down, and a doorway's own speed, have no record

No cached row passes a door up or down, or one whose doorway gives her speed: `SettingOff`,
`Sliding`, `Carrying` and `WhereSheLands` state them from the original only, with constructed
vectors. A short movie through a vertical door would check the 224 px slide, the up door's 32 px,
the 57-frame slide's still first frame, and, for a door up, the slide's step two frames after the
view sets off.

### G2. Colours other domains write during the fades

The passage's fades are exact over every frame of work but 29, each another domain's write:
- 17 in `boss-kraid-defeated` (colours 113 to 115): Kraid's corpse, rewritten at each enemy run
  while the room darkens (Kraid's family, C1).
- 12 in `chain-any-0010-DF8D` (colours 41 to 46, BG palette 2 colours 9 to 14): the Ceres door of
  the elevator room (B8's later piece, the elevator room), whose run cycles the elevator beacon's
  six colours by the frame count, at each enemy run while the fade's step is 0. In the passage
  that is the fade in's first frame of work, so the beacon's colours are set there and faded from
  there.

The harness masks both under their writer until those domains output them.

### G3. The colours the loading writes straight to the screen

While the screen is dark the room's loading writes some colours to the screen directly, not to
the colours it heads for (the enemies' palettes, sprite palettes 6 and 7; 182 colours over the
17 cached loadings). The harness reads them, an input, on the frame `Arriving` ends. R2 and C1
must output them; until then the engine's free run takes them from its own loading.

### G4. The waits' answers come from audio

`Quieting` waits for the sound requests to be sent and `Music { started: false }` for the music;
both answers are U1's. The music's is decoded from the music's queue as the frame of work finds
it: the queue takes one step at the frame of work's start (an entry whose delay runs out is
played and leaves the queue), then the music is ready when no entry is left (41 of 41 cached
waiting frames of work, `chain-any-0005-E06B` among them, 25 frames of work into Ridley's room).
U1's spec must state the queue's step.

### G5. The cost of each frame of work (T1)

T1 owns the cost of each frame of work, by the passage's cost event: `FadeStep`, `BuildRoom`,
`FetchTiles`, `FetchContents` and `Other`. The passage is the largest stretch of lag on every
Ceres rung (25 of the 40 frames of each fade, and 60 to 70 frames of the loading, are overruns).
The contents' cost is bounded above only: its frame of work then waits for the slide, which the
passage owns, and in every cached passage it ended with the slide.

### G6. Her sounds at the door and at the end (samus-appearance)

The passage asks for `AtTheDoor` on its first frame of work and `SpinAgain` on its last; which
sound plays is hers. On record (18 doors, 16 ends): at the door, spinning or wall jumping, the
spin's end sound; otherwise, shoot not held and her charge under 16, a silence; otherwise none.
At the end, spin jumping: the screw attack's sound in the screw attack poses, the space jump's in
the space jump poses, else the spin's; wall jumping, by how far her animation has run, the screw
attack's, the space jump's or the spin's; otherwise none. samus-appearance's observer is to take
these into its spec.
