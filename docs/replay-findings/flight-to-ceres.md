# The flight to Ceres

[Replay status](../STATUS.md#the-flight-to-ceres)

A row seeded at the flight to Ceres's first frame plays the flight alone
(`game::production::played::flight`, the screens-flight model), from the model's start with the
scene imported from the player's ROM and nothing of room play read from the seed. It steps once
per frame of work that finished, borrowing the lag and whether a track waits in audio's queue as
the frame began. `any-fly-to-ceres` holds all 802 cuts: the stage, the plane's angle, zoom and
position, its transformation and scroll, the flash, the brightness and the fade's count, the
label's step and hold, the sprites by the slot each kind takes, the places the sprite drawing is
told and whether it is done, and the letters the label types, compared from the setup's end at
cut 32 (before it, the brightness alone). Not compared: the colour cycles (R4), CGRAM's setup load
at cut 32, OAM, mode 7 VRAM, the BG1 tilemap's emptying, the letters' palette, priority and flips,
and the PPU mirrors the stages write. A lagged frame stepped as finished differs at cut 1 in
`flight.stage`. At each flight cut the run compares the appended music requests and their
delays in queue order.
On cut 32 the flight asks to load the Ceres tune with a delay of 8 frames, then to play it with
a delay of 14. The other 801 cuts issue no requests. Reversing the two requests first differs
at cut 32 in `flight.music`. Audio supplies the reply about whether a track is waiting.

At cut 803, movie frame 8,315, the flight turns the dark screen over to arrival during a
lag cut. Production carries both frame counts, brightness and the random sequence into
preparation. The run compares both counts at every cut and the random number at finished
flight cuts and every preparation cut. The first room and its actors come from room loading
and fresh-file Progress. The same run plays through the fade, the first room and the following
rooms up to movie frame 11,423. It holds 3,911 cuts from the flight seed and stops at
3,912 with `RidleyGetawayPush`. The cartridge save is outside the
comparison. Delaying the flight handoff until the work finishes first differs
at cut 803 in `flight.active`.
