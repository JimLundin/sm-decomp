# Source role

1. Confirm the item is unclaimed with the coordinator and use the assigned base
   and file scope. Read applicable contracts.
2. Trace the complete responsibility: setup, update, callbacks, failure,
   reset/load, callers and retirement. Prefer existing owned modules and real
   shared behavior. Identify the actual state owner and remaining external writers.
3. Preserve native/LE access, full and odd identities, captured versus fresh
   selectors, lazy reads, callback order and effects before failure. A safe entry
   must be safe through its callees for every input its interface permits;
   unchecked pointer creation still matters when the pointer is discarded.
   Derive byte-versus-element offset units from the actual operation. Rust
   assignment evaluates its RHS before the destination place; function arguments
   follow argument order. Preserve that order when introducing helpers.
4. Implement in the isolated checkout. Keep reachable unsafe backend capabilities
   explicit under PLAN's migration policy; completed owned modules forbid unsafe.
   Include adapters, callbacks and callers in the replacement cost. Report net
   production, legacy, unsafe and verification changes, explaining required growth
   and distinguishing substantive removal from formatting or relocation.
5. Format and freeze the final source. Deliver the existing patch/handoff and
   source pin, contract findings and named affected rows to the assigned reviewers
   and integrator. Label unobserved behavior accurately. Compilation and replay
   completion belong to their assigned executors unless you also own those roles.

Correct a rejected frozen candidate in a separate successor. Preserve the prior
artifact and explain the correction; transfer checks only where the source and
relevant dependencies remain unchanged.
