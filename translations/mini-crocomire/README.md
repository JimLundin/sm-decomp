# Mini Crocomire translated source

Complete14-definition actor scope: MiniCrocomire_Instr_897E/8990/899D,
Init, Main, Func1..4, PreInstr5/6 and Func7..9. Main's no-op8A75 is also retained.
Translated/unverified and unregistered; no production adapter or executed tests.
Independent full-body source review accepted the corrected bounded candidate;
this does not establish actual-C or original-ROM agreement.

Recovered snesrev/sm revision578f90b3cc49557bb70060ad033bb90b8cf8ac50;
baseline4aa9eb660e7b02dfc176c3d84ab027161630e11d. Exact source SHA256
6db2f0026b034af0e7c16f1f4613953ac539a64ddd926a7cec5c98330785f520;
Cargo bee9b406ba00299d185f81ccad9b69135f2c77867ecc4838943d4822ea5e92d8.
Original rejected A1-bank offsets and private dependency manifests are excluded.
Corrected speed A2:8187/file110187 and instruction operand A2 bank mapping
remain caller-authenticated ROM; parameter2<=64 bounds all four speed reads.

The std-only safe seam uses exact128KiB WRAM and aligned64-byte slots below800.
Current-index helpers reacquire CURRENT after full synchronous copyback.
Services own horizontal movement, Y probe, RNG and projectile spawning; no
collision/random/spawn engine is duplicated. Refusal leaves state unchanged;
accepted errors retain effects and prohibit C fallback. Native adapter admission,
alias/allocation/generation/reentry and callback domains remain required.

Literal model tests distinguish A2 addresses, word order, parameter1=2,
signed32-bit motion, changed-current service order, random0/1/2/255, instruction
operand/jp behavior and terminal collision values. They have not executed.
Next gates: compilation/Clippy, all-entry actual-C pairs/chains/full-WRAM/effects,
refusal/terminal controls, runtime registration, separate live/original-ROM gates.
Root MIT and applicable retained upstream notices govern their respective code.
