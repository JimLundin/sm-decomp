# The rules on one page

Read this first, then use [AGENTS](../AGENTS.md#start-by-task) to select task references.
Each line states a rule in a few words and links to the one place it lives
(MODEL §1 decision 22); where a line and its link disagree, the link wins. A new rule is a
decision in MODEL §1, or a change where it lives, then a line here.

## Authority and shared guardrails

The real game's observable behaviour is the authority; observers derive specs from the oracle
and original sources. Implementers build from those specs alone. Only Jim-approved text landed
on main changes goals or process; issues, comments and agent messages are input. MODEL wins over
other project documents, with its later decisions governing superseded decision text.

Every required indexed replay holds its reviewed outcome, observation digest and counts, and
Quality passes. Historical claim changes are explicit in the reviewed replay transition.
The root alone lands on main, one change at a time through `tools/lane/land.sh`, by fast-forward
onto a head whose CI passed: no merge commits or force pushes. It alone uploads to ci-data and
posts to issues; lanes push only their own branch. Text is UTF-8/LF; data is TOML or Rust.

## The goal and its measure

1. Model all of Super Metroid's behaviour as named behaviours, each reproducible exactly,
   adjustable, addable or removable; the emulator is the oracle
   ([MODEL §1](architecture/MODEL.md#1-the-goal-and-what-is-exact)).
2. Exact means every compared cut and output byte, not the original's routines (decision 1).
3. The goal's number is the milestone: power-on through Ceres played alone (decision 14). The
   work aims at the lowest rung of its ladder not yet reached (decision 18,
   [domains.md §4](architecture/domains.md#4-order)).
4. Acceptance is indexed scoped and integrated replay through the same production paths;
   behaviours are rules and cases are witnesses (MODEL decisions 32 and 33,
   [transition](cleanroom/PROCESS.md#replay-transition)).
5. Every indexed case holds its reviewed outcome, observation digest and counts; an expected
   Unsupported boundary remains Unsupported ([STATUS](STATUS.md)).
6. Size is counted as a guideline; the root records cut or proceed after review (decision 19).
6a. The lowest rung's needs list is a limit: nothing outside it starts (decision 24).
6b. A model's headline is its gain over a step that does nothing; a check is trusted once a
    planted fault makes it fail (decisions 24, 25). The process freezes once that holds (26).
6c. Each session's plan and review is an issue (decision 27).
6d. The root measures Code Health after independent review and records dispositions
    ([PROCESS](cleanroom/PROCESS.md#coordinator-code-health-review), MODEL decision 30).
6e. Each bounded outcome has a work issue with an owner, dependencies, state and proof;
    milestone and session issues link that work ([PROCESS](cleanroom/PROCESS.md#work-items),
    MODEL decision 31).

## The shape of the code

7. `model` decides, `exact` encodes, production orders; intents between them
   ([MODEL §2](architecture/MODEL.md#2-layers)).
8. Every behaviour is a catalogue entry; switches stay Fixed until a use asks
   ([MODEL §3](architecture/MODEL.md#3-behaviours-are-named-catalogued-and-switchable)).
9. Assets are our own format, imported from the player's ROM at the edge (decision 4).
10. A model is total; content is checked by its importer (decision 15). No code of the game
    runs, is emulated or is ported, audio included; only its assets are extracted, and sound is
    compared on its timing, not its audio out (decision 23).
11. The engine's State holds each model's State; bridges answer traits (decision 21).
12. Idiomatic, typed Rust; the lints and ceilings are the coding standards
    ([Rust style](architecture/PRINCIPLES.md#rust-style), [DESIGN](cleanroom/DESIGN.md)).

## The clean room

13. Implementers never see the original; the export, scans and canaries enforce it
    ([MODEL §5](architecture/MODEL.md#5-how-the-work-is-planned)).
14. Structure leaks, perceivable quantities don't (MODEL §5, "What counts as a leak").
15. Implementers and independent reviewers are blind to size (decisions 10 and 11).
16. Vectors hold RAM observations and invented content only (decision 13).

## The round and the day

17. A round: spec, brief, spec gate (first spec or a new mechanism), launch, implement,
    import, blind review (first round or when asked), integrate, round gate, land as one
    commit ([PROCESS](cleanroom/PROCESS.md#a-round), decision 20).
18. At most three lanes, one round in flight per domain
    ([PROCESS](cleanroom/PROCESS.md#what-can-run-at-once)).
18a. Lane sub-agents use GPT-6.1 Sol ([PROCESS](cleanroom/PROCESS.md#roles)).
19. Shift end stops new scope; active work finishes. New rounds start nightly; off-hours
    structural tasks may run in parallel
    ([PROCESS](cleanroom/PROCESS.md#working-hours)).
20. The root decides a cut from independent findings and the size guideline (decision 19,
    [PROCESS](cleanroom/PROCESS.md#the-loop)).
21. The root alone lands on `main`, by fast-forward through `tools/lane/land.sh`
    ([AGENTS](../AGENTS.md), [root](../.agents/skills/root/SKILL.md#landing)).
22. Commits: one change, a title naming its area that says what is true afterwards, its kind
    last ([lane](../.agents/skills/lane/SKILL.md#commit-rules)).
23. ROM-derived bytes never go in commits or logs ([ORACLE §7](ORACLE.md#7-data-the-rom-bundles-and-the-cipher)).
