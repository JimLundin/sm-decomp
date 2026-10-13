# Ridley inlined native definition correspondence

Date: 2026-10-13. Read-only source review; no verification or gameplay runs.
Native pin: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`.

The 220-entry native reachability list is not independent implementation proof.
Eight definitions absent from the source auditor's named/numeric entry mapping
have their bodies composed into owned Rust caller implementations. Native C
in `deps/snesrev-sm/src/sm_a6.c` and the following maintained Rust were compared.
These support translated correspondence at the callers' admitted domains. They
do not support separate native wrapper counts or unbounded domain equivalence.

| Native definition | Owned Rust composition | Reviewed behavior |
| --- | --- | --- |
| Ridley_Func_17, A6B641 | ridley_subsystem.rs:640, run_state B5E5/B613 | Samus Y capped352, native seek/profile, tile2write, movement byte3 condition, RNG low-byte80 and var0F/var10 attack gating. Its return is consumed directly by composed branch. |
| Ridley_Func_18, A6B68B | ridley_subsystem.rs:648 | tile9=240,tile15=16,tile0=1 and stateB6A7 after Func17's true branch. |
| Ridley_Func_24, A6B84D | ridley_subsystem.rs:684 and :723 | tile0=1/tile10=1 in native Func21/22 call sites. Equivalent writes are composed in caller; no independent entry. |
| Ridley_Func_25, A6B859 | ridley_subsystem.rs:683 | can_carry guard short-circuits grab_rect(4,4), corresponding to native Func40_Carry then Func26. |
| Ridley_Func_27, A6B889 | ridley_subsystem.rs:686 | C velocity made nonpositive, clamped at−512, Func24writes, stateBB8F and escape_grab corresponding Func33b. |
| Ridley_Func_41, A6BD20 | ridley_subsystem.rs:722 | SamusY352 signed comparison composed into Func22 conditional with short-circuit timer decrement. |
| Ridley_Func_48, A6BE61 | ridley_subsystem.rs:837 | var42/44 dx/dy,4pixel underflow guards and rectangle extents composed for the sole native Func47 caller rectangle(w4,h4). Native accepts generic Rect16U; owned caller fixes4/4, so only that domain is mapped. |
| Ridley_Func_127, A6DFD9 | ridley_movement.rs:489 | tile82/83 rectangle14/14 contact, native suit damage/health, invincibility96, knockback5 and horizontal direction via damage/knockback; existing caller conditions compose around it. |

No direct fixture root exists for these eight in the controlled 33-entry family.
Current controlled caller evidence can test their composed behavior only if its
specific cases reach these branches; this review does not infer that reachability.
The mapping is source correspondence and translated credit only. The 220-entry
closure includes these source-composed definitions, but stage verification must
remain separately bounded. Other source-audit concerns remain independent.
