# Enabled-pattern test seams

The user approved the real generated-step corpus/timing runner and
reference/subject execution APIs before tests were written. Ticket 05 tests
compiled batch execution against independent SingleStepTests state/bus vectors,
with synthetic boundary vectors and the existing independent SNES timing
oracles. The runner shares the production emitter body and fetch/PC operation;
it does not model instruction semantics with a second interpreter. Corpus
expectations establish CPU state, sparse memory, PC and ordered active bus
cycles; SNES fixtures establish hardware stalls and frame resumability.
