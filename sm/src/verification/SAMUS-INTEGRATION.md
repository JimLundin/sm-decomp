# Samus palette verification contracts

Current registration lives in `lib.rs` and `verify_dll.rs`; the observer is
registered in `game/visual_runtime.rs`. These contracts describe the retained
carriers, not additional installation steps. Current work follows [PLAN](../../../docs/PLAN.md).

## Defined predecessor and candidate: 224 local IDs

`samus_palette_defined_cases.rs` supplies the cases and
`samus_palette_defined_dll.rs` exports bounded results through `copy_case`. Capture IDs
0..223 against the predecessor before replacing algorithms. Each result has
131072 RAM bytes then a LE u16 defined return. Shared fixtures assert no panic or
diagnostic. Candidate swaps only the adapter path. IDs83/91/99 exercise shine170
and real channel3/max9. Existing palette512/animation438 IDs are unchanged.

## Original 65816 selector 2: 29 preserved local IDs

The Runtime probe invokes `samus_palette_observer.rs` with actual Session/Game;
projection copies RAM and never changes authority. Samus core hooks use event;
shared queue_sfx hook uses sound so nested movement-command sound31 is retained.

`samus_palette_candidate_dll.rs` is included without a cfg guard. The existing oracle
export, underlying cpuinfra oracle module, observer module and public library
bridges compile unconditionally. The direct tool sets sm_verify_dll on both
the library and DLL host. The production observation and
injection calls are cfg(sm_verify_dll); preserve the existing build pipeline.
Candidate export uses the same initialization as the original CPU oracle and
must run once in a fresh Python process per case. Outputs contain ROM-derived
RAM; save them locally outside the repo. Their SPCR1 framing is documented by
the parser rather than being confused with SPOR1 CPU-register framing.

Run `compare_samus_palette_oracle.py REFERENCE_DIR CANDIDATE_DIR` on the existing
oracle-results-1 directory and29 candidate captures named0.bin..28.bin. The
script checks frozen reference hashes, exact ordered semantic events and four
131072-byte phase snapshots. Only reviewed stack1FEB..2000 and charge scratch
24..26 for cases21..24 are excluded from equality; candidate must preserve entry
bytes there. Oracle values are retained in the report. No other ignored bytes.
The semantic sound mapping and all reference phase assumptions require the
independent oracle reviewer approval before acceptance.

IDs29..32 add explicit post-charge callback transitions, defined in
`samus_palette_oracle.rs`: charge palette index/timer pairs `(0, 2)`, `(9, 2)`,
`(0, 1)` and `(9, 1)`. The actual core calls the verification-only
`samus_observer::inject("samus.palette.dispatch", host.session_mut())` immediately
before its dispatch probe. Runtime's probe remains read-only. The injector acts
only for these four active cases, once, and emits the same22-byte tag9 payload as
the originalCPU experiment. It never changes ordinary production or IDs0..28.
The comparator accepts a29-case or33-case frozen manifest and labels the appended
cases `injected-callback-transition`; no additional excluded RAM ranges apply.
The extended oracle capture must preserve original0..28 bytes exactly.
