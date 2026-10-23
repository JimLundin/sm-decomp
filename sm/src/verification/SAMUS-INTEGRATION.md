# Verification carrier installation

All files are staged only. No compile or execution was performed by lane D.
Coordinator owns final lib.rs/verify_dll.rs/source registration.

## Defined predecessor and candidate:224 new local IDs

Install `samus_palette_defined_cases.rs` as sibling module `samus_defined_cases`
and either adapter as `samus_defined_adapter` in lib.rs:

```rust
#[path="verification/samus_palette_defined_cases.rs"]
mod samus_defined_cases;
#[path="verification/samus_palette_defined_legacy.rs"] // candidate swaps this path only
mod samus_defined_adapter;
pub fn verify_samus_palette_defined_case(id:u32)->Option<Vec<u8>> {
    samus_defined_adapter::capture_case(id)
}
```

Use `samus_palette_defined_dll.rs` as an include in verify_dll.rs, or integrate its
single export. Ordinary existing copy_case handles bounded output. Capture IDs
0..223 against the predecessor before replacing algorithms. Each result has
131072 RAM bytes then a LE u16 defined return. Shared fixtures assert no panic or
diagnostic. Candidate swaps only the adapter path. IDs83/91/99 exercise shine170
and real channel3/max9. Existing palette512/animation438 IDs are unchanged.

## Original65816 selector2:29 preserved local IDs

Include `samus_palette_observer.rs` in visual_runtime.rs as
`pub(crate) mod samus_observer` with its verification path.
Existing Runtime probe invokes its observe function with actual Session/Game;
projection copies RAM and never changes authority. Samus core hooks use event;
shared queue_sfx hook uses sound so nested movement-command sound31 is retained.

```rust
pub fn verify_samus_palette_candidate(id:u32,game:&mut Game)->Result<Vec<u8>,String> {
    game::visual_runtime::samus_observer::capture(id,game)
}
```

Include `samus_palette_candidate_dll.rs` without a cfg guard. The existing oracle
export, underlying cpuinfra oracle module, observer module and public library
bridges must also compile unconditionally. The direct tool sets sm_verify_dll on
the library only, not on the DLL host. Only the inert production observation and
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

Approved extension: IDs29..32 add the explicit post-charge callback transitions
in `ORACLE-INJECTED-CASES.md`. The actual core calls the verification-only
`samus_observer::inject("samus.palette.dispatch", host.session_mut())` immediately
before its dispatch probe. Runtime's probe remains read-only. The injector acts
only for these four active cases, once, and emits the same22-byte tag9 payload as
the originalCPU experiment. It never changes ordinary production or IDs0..28.
The comparator accepts a29-case or33-case frozen manifest and labels the appended
cases `injected-callback-transition`; no additional excluded RAM ranges apply.
The extended oracle capture must preserve original0..28 bytes exactly.
