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

**Affected Offline family**:
An Offline family whose verification inputs may be changed by a proposed or
current source change. Incomplete dependency knowledge leaves the family affected
until it can be ruled out.

**Translation scope**:
A coherent set of gameplay behavior selected for translation, together with its
required dependencies, writable source ownership and verification requirements.

**Translation attempt**:
One candidate translation of an approved Translation scope, evaluated against
fixed source dependencies and verification requirements. A resumed attempt keeps
its identity; changed candidate source or an authorized new evaluation creates
another attempt.

**Rewrite capsule**:
The approved source, recovered C, dependency context and bounded diagnostic
provided for rewriting within a Translation scope.

**Source replacement**:
Replacement contents for an approved source file in a Rewrite capsule, bound to
that capsule's identity and original file contents.
