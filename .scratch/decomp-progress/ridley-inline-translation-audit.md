# Reviewed inlined Ridley implementation mappings

Status: translated source correspondences reviewed; no independent verification credit.

The whole Ridley recovery's reachable 220-definition list contains eight native
helpers absent from the automated source map. Comparing their pinned C bodies
with the maintained caller implementations confirms these correspondences:

| Native symbol | Maintained implementation | Domain |
| --- | --- | --- |
| Ridley_Func_17 | ridley_subsystem.rs run_state, B5E5/B613 branch, lines 641–659 | Samus Y capped at 352, profile seek, tile2 write, spin/RNG attack gate; return is represented by caller branches |
| Ridley_Func_18 | same branch, lines 648–651 | tile9=240, tile15=16, tile0=1 and B6A7 transition |
| Ridley_Func_24 | run_state B70E/B7B9, lines 684–685 and 720–721 | tile0/tile10 initialization |
| Ridley_Func_25 | run_state B70E, line 683 | can_carry and 4-by-4 grab rectangle |
| Ridley_Func_27 | run_state B70E, lines 686–689 | negative velocity capped at -512, tile setup, BB8F transition and escape_grab |
| Ridley_Func_41 | run_state B7B9, line 722 | signed Samus Y gate at 352, inverted by the caller |
| Ridley_Func_48 | run_state BE03, lines 837–839 | native rect test specialized to the caller's 4-by-4 rectangle |
| Ridley_Func_127 | ridley_movement.rs update-tail path, lines 487–492 | 14-by-14 tip rectangle, damage, 96 invincibility/5 knockback timers and direction through damage/knockback helpers |

These are executable inlined implementations within their admitted callers,
not standalone native-call wrappers. The general rectangle ABI of Func48 is
not claimed; the fixed caller specialization is. The source map's missing alias
records remain useful automation gaps. Neither reachability nor this comparison
grants verified, integrated, or live-exercised status for these helpers.

Pinned C revision `578f90b3cc49557bb70060ad033bb90b8cf8ac50`;
inventory `7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`.
Actual C bodies are in deps/snesrev-sm/src/sm_a6.c at lines 2001, 2018, 2122,
2127, 2143, 2402, 2470 and 4326 respectively. Existing host services, caller
admission and script boundaries remain explicit.
