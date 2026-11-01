# Next retail coverage batch

Source planning starts from integration `a64f9ad698db94c56706e6c03866ae92caf0c6c7`.
The inherited two scenarios, pins and original qualification are unchanged.
The next scenarios reached their interactions against both lockstep oracles
and in independent faithful oracle/engine runs. Both faithful locks match
the engine. The freeze-duration mutant first fails at tick 49 in
`actors.enemies[1].frozen`; independent engine replay rejects both fresh
freeze and shatter facts. The bomb-release mutant first fails at tick 706
in `actors.enemies[0].x`; independent engine replay rejects completed
release. Canonical source and DLL bytes are restored with their original
embedded guards passing. The final canonical gate passes.

The pair uses the real 100% Tourian entry seed `100-input-phase:317`.
One scenario changes Samus's equipment to ice/plasma/charge, unequips screw
attack, preserves the original 50-tick opening that freezes a Metroid, then
aligns underneath it and jumps while firing ordinary missiles. Its facts
require a fresh freeze and death of that same frozen slot, an ordinary missile
and a fresh explosion, with competing supers/bombs/power bombs excluded.
The other preserves the qualified latch seed and original 333-tick approach,
cancels the HUD selection, morphs and lays ordinary bombs. Its release fact
requires a fresh bomb-off event followed by the same live Metroid chasing
again. A loaded frozen or bombed-off actor does not establish either episode.
The 502-tick ice input freezes at tick 49 and shatters that frozen slot at 251.
The 757-tick bomb input freshly latches at 111 and completes release at 709.

The post-tick facts follow the real update order: projectile contact runs
before the selected enemy's AI. Metroid shot sets frozen time to 400 or the
bombed-off timer to four; that same tick's AI consumes the first tick.
The snapshots therefore observe 399 or three, respectively. The earlier
private bomb recipe's timer-four assertion could not observe this boundary;
its retained trace also showed no bomb-off activity. The reached input uses
ordinary bombs and observes the completed release. Earlier recipes and
receipts remain private evidence.

Botwoon's three-projectile fan cannot arise through the retail producers.
The [bank $B3 log](https://patrickjohnston.org/bank/B3) marks `$B3:9696` unused;
its `$96B1` store installs head function `$9E7D` without changing the boss
function. That is the producer that could select the three-shot branch at
`$9ED7`. The live producer at `$991C` installs `$9E7D` together with boss
function `$99E4`, so `$9ED2` instead selects the five-shot branch `$9EE0`.
The current engine has only that live producer. No scenario installs an
enemy function to manufacture the three-projectile fan.

Grapple spike contact/anchored placement and Etecoon ceiling contact remain
deferred, not proven unreachable. T3 found no suitable standing/crouching
grapple block or reachable spike geometry; T4's 4,000-tick release reached no
Etecoon ceiling contact. The retained 100% scan covers 198 unique room
geometries; its 31 attach-capable enemy observations count all retained
visits. It found no grounded enemy-anchor candidate at those recorded
positions. The scan does not cover all possible enemy movement. These gaps
retain no retail-unreachable annotation.

Complete any%, 100% and low% movie scope remains open as listed in
[the inherited inventory](coverage-movie-gaps.md). These two interactions
cannot substitute for boot, wait intervals, carried state or completion.
