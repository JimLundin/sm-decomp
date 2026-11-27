# The title, file select and options

[Replay status](../STATUS.md#the-title-file-select-and-options)

`any-power-on` plays on past the boot through the title, file select and the options menu
(the screens-title, screens-file-select and screens-options models, one step per file under
`game::production::played::front_end`), to the options' hand-on to the intro, and on through
the intro's opening and page 1 (screens-intro, through the intro's adapter) with no new seed.
Cut 568 is the first cut of the next row, `any-intro-0568`, which the run reaches as the row that
follows a power-on row: the frame end the options hand on at, where the front end begins the
intro in the State `Begins`. The run holds 11,423 cuts. It carries the fight through
the locked actor turns and return, then plays discovery, delivery and examination.
The last page ends at 7,512. Flight owns the active scene and forced blank at 7,513.
The flight takes the intro's frame counts, random sequence, pad latch, health and item bindings.
At 8,315 it hands its State to arrival during an unfinished work interval. The selected
file's save publishes its payload on that cut and both checks at 8,316. The continuation
restores the arrival and pending save checkpoints at 8,315. It checks the active slot
and the original selected file as well as every battery byte through room play. The movie starts a new file, so
arrival initializes fresh-file Progress. At 11,320 the carried room begins the flyby.
The next cut at 11,424 requires `RidleyGetawayPush`.

The sound Samus requests when she leaves her spin reaches the front end at 2,517. The standing
handoff at 2,615 runs before the final pad. The locked fight keeps its body sample
while her clip and existing shots advance.
At 569 the intro creates the hidden actor with the chosen file's energy, lends her missiles and switches to the intro controls.
The next 1,650 cuts carry those owners through the opening and page 1. The press latched at
2,218 installs the populated room, recorded controls and Mother Brain before interrupted
cut 2,219. At 2,220 the recorded pad drives Samus and the encounter takes its first turn.
The first missile fires at 2,310 and strikes at 2,329. A second missile strikes at 2,371.
A ring pushes Samus at 2,387.
The run checks missile slots, exhaust, stocks, incoming-shot protection and ordered weapon
draws. It borrows the interrupted call and its stack to distinguish work already drawn
from a pending room turn. The partial palette and text buffers remain outside this comparison.
The run compares her pose, centres, history, clip, radius, missile supply, controls and
cannon delay, the installed cell words and the encounter state. It leaves the inherited
room memory tail to the platform. From cut 568 it also compares the intro and the front end.
The front end compares clicks, queued music entries and the fight room's appearance sounds.
Music stops use empty
entries, and only the scoped intro runs compare them and the caret's picture.

The scoped case `intro/flashback-from-page-one` admits the last finished cut of page 1
and its latched pad. It carries the intro through the interrupted cut in the next row
and through page 2's wait. It compares 1,293 finished cuts and five interrupted cuts,
with 37,507 State fields and 9,186 outputs. Holding the preceding State matches five complete
cuts. The scoped frame borrows the encounter's return cue from its explosion hold and callback. It compares the intro's requests before their
consumers. The boss's active hurt flash owns sprite palette 7 on two palette
publication cuts. The durable intro checkpoint codec remains unavailable.

The R2 room adapter imports the populated collision grids for the Zebes fight and discovery.
`intro-rooms/fight-first-turn` compares movie cuts 2219 and 2220. The fight opens at the
interrupted cut and takes its first ordinary movement and appearance turn on completion.
`intro-rooms/discovery-first-turn` compares the same handoff at movie cuts 3517 and 3518.
The two scoped cases admit their preceding actor and backed room image once. Each compares
2,090 fields and restores from both cut checkpoints. Holding the preceding State matches zero
complete cuts in either room. Delaying installation first differs at movie 2219 in
`intro-room.cell.0` and at movie 3517 in `intro-room.width`. Integrated power-on play still needs the actor and the
room intent on the unfinished work interval, followed by the live demo and encounter.

The scoped case `flashback/zebes-encounter` starts with the latched press and inherited
background Y from `any-intro-0568` cut 1650. It compares the encounter through
`any-intro-2219` cuts 0 to 650, with 3,906 State fields and 6,510 outputs. Holding
the preceding State matches two complete cuts. All 651 checkpoint restorations hold.
Moving the contact line one pixel farther fails at cut 109 in `flashback.brain`. Room, demo, Samus, weapons
and intro inputs are admitted for this witness. The encounter publishes the selected
missile slot, ring knockback, palette flash, page-two cue, draw gate and visible encounter
OAM. Weapons owns the impact continuation, and the integrated intro and renderer still
have to consume these publications.

The scoped case `screens-playback/intro-demos` carries the recorded pad from the end of
page 1 across the fight and discovery. It compares 2,234 cuts, with 8,936 State fields
and 15,638 output observations. All 2,234 checkpoint continuations hold. Holding the preceding
State matches 656 complete cuts. Dropping a fresh press during a continued hold fails
at run cut 1786 in `playback.current.fresh`. The playback owner publishes held and fresh buttons and the
scene-start and finish handoffs. The scope admits room actor opportunities, Samus's
pre-motion whole X and the intro's fade count from the locked records.

From cut 271 it compares the screen, the game state and the count of finished frames at every
cut; at finished cuts, the random number, the sounds against the queues (accumulated over each
finished cut, because a sound can appear at a lag cut), the title's countdown, file select's
and the options' cursors, and the options' block size and settings (Japanese text,
weapon-reset, moonwalk). Borrowed, as the run's receipt lists: the frames of work finished in
each interval (from the cut kind, and 2 wherever the game's count steps by 2 at a `Requested`
cut), the random number as the game settles, and P2's save-memory answers.

Planted faults: a title fast step of 3 differs at cut 308 in `front-end.screen`; a file
select menu pace of 3 at cut 403 in `front-end.screen`; file select drawing a random number on a
frame of work that goes on at cut 395 in `front-end.random`; ChoiceMade's sound coded 2Bh at cut
433 in `front-end.sounds`; Up moving the options' cursor down at cut 488 in `options.cursor`; the
options keeping the settings they were handed at cut 519 in `options.moonwalk`. The boot's two
faults fail where they did on the longer run. Limits:
- from the title on, the count of finished frames is the sum of the borrowed steps, which are
  read from the game's own count, so comparing it is not an independent check; the screen and
  the random number check those cuts. A step of 2 is taken at any `Requested` cut; this run
  sees one at cut 308 alone;
- how the file begins is the harness's P2 answer for a new file (the intro), not observed; the
  intro's language is the options' setting;
- from the intro on, the music's stops are not compared: the queue holds a stop as an empty
  entry;
- the order of the sounds asked for within one interval is not compared, within a queue or
  across queues: expected and played are each sorted before they are compared;
- save memory (SRAM) is not compared;
- the pictures are not compared: OAM, CGRAM and VRAM;
- the queues' heads and timers are not compared, only the sounds in them;
- the sub-stage indexes are not compared.
