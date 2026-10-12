# Decomp verification

## Language

**Fixture**:
A declared routine invocation with its input state and comparison requirements.

**Fixture pair**:
The Rust and actual-C executions of the same fixture whose outputs are compared.

**Fixture chain**:
A sequence of fixtures in which each continuation inherits matching state from
its declared predecessor.

**Offline family**:
A declared set of routines and fixtures verified against actual compiled C.

**Live replay**:
An execution over recorded inputs that observes gameplay state, ordered hardware
effects and exercised Rust ownership.
