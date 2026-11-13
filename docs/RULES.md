# The rules on one page

Read this first. Each line states a rule in a few words and links to the one place it lives
(MODEL §1 decision 22); where a line and its link disagree, the link wins. A new rule is a
decision in MODEL §1, or a change where it lives, then a line here.

## The goal and its measure

1. Model all of Super Metroid's behaviour as named behaviours, each reproducible exactly,
   adjustable, addable or removable; the emulator is the oracle
   ([MODEL §1](architecture/MODEL.md#1-the-goal-and-what-is-exact)).
2. Exact means every compared cut and output byte, not the original's routines (decision 1).
3. The goal's number is the milestone: power-on through Ceres played alone (decision 14). The
   work aims at the lowest rung of its ladder not yet reached (decision 18,
   [domains.md §4](architecture/domains.md#4-order)).
4. A domain is proven by its free run, fed its own State over the rows; the per-frame harness
   localises (decision 17, [MODEL §4](architecture/MODEL.md#4-the-reference-domain-samuss-motion)).
5. Green pairs, the lockstep ratchet, harness floors and free-run floors never fall
   ([STATUS](STATUS.md)).
6. Size is a diagnostic; only a domain larger than the old code it replaces fails (decision 19).
6a. The lowest rung's needs list is a limit: nothing outside it starts (decision 24).
6b. A model's headline is its gain over a step that does nothing; a check is trusted once a
    planted fault makes it fail (decisions 24, 25). The process freezes once that holds (26).
6c. Each session's plan and review is an issue (decision 27).

## The shape of the code

7. `model` decides, `exact` encodes, `tick` orders; intents between them
   ([MODEL §2](architecture/MODEL.md#2-layers)).
8. Every behaviour is a catalogue entry; switches stay Fixed until a use asks
   ([MODEL §3](architecture/MODEL.md#3-behaviours-are-named-catalogued-and-switchable)).
9. Assets are our own format, imported from the player's ROM at the edge (decision 4).
10. A model is total; content is checked by its importer (decision 15). No code of the game
    runs, is emulated or is ported, audio included; only its assets are extracted, and sound is
    compared on its timing, not its audio out (decision 23).
11. The engine's State holds each model's State; bridges answer traits (decision 21).
12. Idiomatic, typed Rust; the lints and ceilings are the coding standards
    ([AGENTS](../AGENTS.md#rust-style), [DESIGN](cleanroom/DESIGN.md)).

## The clean room

13. Implementers never see the original; the export, scans and canaries enforce it
    ([MODEL §5](architecture/MODEL.md#5-how-the-work-is-planned)).
14. Structure leaks, perceivable quantities don't (MODEL §5, "What counts as a leak").
15. The implementer is never told a score, budget or size target (decision 10).
16. Vectors hold RAM observations and invented content only (decision 13).

## The round and the day

17. A round: spec, brief, spec gate (first spec or a new mechanism), launch, implement,
    import, blind review (first round or when asked), integrate, round gate, land as one
    commit ([PROCESS](cleanroom/PROCESS.md#a-round), decision 20).
18. At most three lanes, one round in flight per domain
    ([PROCESS](cleanroom/PROCESS.md#what-can-run-at-once)).
19. Rounds run in the nightly session; outside it one short structural task at a time
    ([PROCESS](cleanroom/PROCESS.md#working-hours)).
20. A cut round runs only on two blind reads' findings (decision 19,
    [PROCESS](cleanroom/PROCESS.md#the-loop)).
21. The root alone lands on `main`, by fast-forward through `tools/lane/land.sh`
    ([AGENTS](../AGENTS.md), [lane](../.agents/skills/lane/SKILL.md#land-root-only)).
22. Commits: one change, a title naming its area that says what is true afterwards, its kind
    last ([lane](../.agents/skills/lane/SKILL.md#commit-rules)).
23. ROM-derived bytes never go in commits or logs ([ORACLE §7](ORACLE.md#7-data-the-rom-bundles-and-the-cipher)).
