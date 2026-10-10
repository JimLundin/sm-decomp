# Measured iteration selection

`just select-iteration` allocates an immutable evidence run before checking the
accepted predecessor and current material inputs. `--predecessor RUN` selects a
preserved accepted lineage explicitly. Selection leaves reviewed configuration,
user changes and `out/accepted.json` untouched. It produces `selection.json`, a
ranked choice with its exact address/MX/opcode/ROM-byte keys, observed counts,
overlapping existing keys and unexecuted deferred inventory.

The accepted origin's complete sealed gates, source snapshot and pointer lineage
are verified first. The instruction profile is the accepted generated cold replay's
actual interpreted instruction counts over input indices 0..17999, bound to its
executable, ROM, movie, input masks and seed. Call-target misses only discover
routine roots. Their counts never enter gain estimates. A missing profile requires
a matching replay of the preserved accepted executable, or refreshed readiness
when the accepted evidence itself is incomplete. A timeout remains incomplete.

Current reviewed functions must equal the accepted configuration. Shared execution,
observer or hardware changes require refreshed readiness. Selection preserves both
current source and accepted baseline identities; it never relabels historical
execution as current candidate validation. A `Readiness baseline/setup required`
error is actionable by the iteration session: refresh the necessary readiness gates
and retry selection before treating a baseline as usable. It is not itself a request
for a human decision when those gates can run locally.

Candidates use production CFG recovery and the actual timed whitelist, tracking
REP/SEP width changes and deduplicating overlapping blocks. A routine can remain
partly interpreted. At most 128 blocks/1024 instruction keys keep a candidate
bounded. Gain is the sum of observed interpreted instructions at newly compilable
keys after subtracting every existing generated key. Larger exact measured gains
rank first; equal gains prefer fewer keys, then fewer blocks, then address/MX.
This deterministic tie rule treats indistinguishable counts as similar gains.

Configured routine/width slices are evaluated too: a reviewed root can contain
deferred supported keys absent from the compiled table. When those keys are
observed interpreted, `enable-deferred-keys` selects a bounded revisit before
the pattern route. Preparation preserves the reviewed variants and lifts only
the selected keys' exclusions; other exclusions and comments remain intact.

A `supported-routine` result chooses exactly one `{address,mx,variant}`. Candidate
preparation consumes `chosen.new_executed_keys`; `chosen.deferred_keys` must remain
excluded from the actual new compiled table until independently covered. Other
ranked discoveries are alternatives, not implicit configuration additions. Each
key includes original ROM bytes, allowing generation to reject stale proposals.
A candidate must still prove focused comparison, cold parity, exact changed-key
coverage and positive actual progress before acceptance.

`needs-timed-pattern` means no supported routine offers gain but native unsupported
instructions were observed; ranked opcode/addressing patterns identify that work.
`no-covered-candidate` means neither route has measured covered work. Neither
result authorizes execution beyond the accepted prefix; independently validate a
concrete domain extension first.

Tests exercise the public generator command and evidence interfaces with synthetic
LoROM instructions and literal measured counts. They catch ranking, CFG recovery,
overlap, count provenance and missing/incomplete baseline handling; commercial
ROM measurements remain ignored local artifacts.
