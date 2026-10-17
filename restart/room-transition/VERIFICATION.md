# Focused verification

Fifteen dependency-free tests cross the same public scheduled-entry interface
as integration. Authored WRAM/ROM inputs and a typed synchronous Host probe
check the following distinct behavior:

- Same-invocation state 9 → private state 10 → state 11, exact nested routine
  observations, both CRE-dependent target-palette branches and timer handling.
- Downward elevator's immediate first predecrement and negative completion;
  upward entry bypassing the delay and signed high-bit behavior.
- Raw FE NMI suspension, cleanup of a pending wait after IRQ sets 8000,
  preserved state 1 at BG3 setup, repeated raw 7/80 PLM suspension, no repeated
  initialization/HDMA work, timer drawing only after completed phase invocation,
  and preservation of outer state 0 and PLM-owned state 2 bytes.
- An unfinished IRQ causing another NMI wait in the same resumed invocation.
- Nonempty background 4/2/2/0 with exact BA:C22A → WRAM 4000 decompression,
  two immediate VRAM transfers, odd operand positions and observed returns.
  All remaining commands, private 8/E → 2 calls and the door-dependent skip.
- Raw fade/music query conditions, service order, phase visibility, all four
  alignment directions and wrapping, upward correction, scroll setup, immediate
  graphics/decompression arguments and the special-door CRE exception.
- Final Samus/IRQ adjustments, exact HDMA clearing, sound/timer guards and the
  distinction between an invocation returning zero and completed gameplay entry.
- Refusal/invalid admission before observations or services, and an owned
  service failure that cannot become refusal or C retry.
- Complete 3 MiB image and expanded 4 MiB native view admission, with truncated
  input rejected before state changes or observations. All ordinary fixtures
  use the native 4 MiB extent, including nonempty bank-8F background reads.

The test Host validates balanced passive observations on every successful
return, including pending returns. Its foreign operations are probes; they do
not establish behavior of actual compiled C or hardware.

Isolated worker commands, with the temporary standalone workspace declaration:

```sh
cargo test --manifest-path restart/room-transition/Cargo.toml --offline -j1 --target-dir restart/room-transition/target
cargo clippy --manifest-path restart/room-transition/Cargo.toml --offline -j1 --target-dir restart/room-transition/target --all-targets -- -D warnings
cargo fmt --manifest-path restart/room-transition/Cargo.toml -- --check
```

Raw stdout/stderr are retained under the ignored `checks/` directory. The
integrator removes the temporary `[workspace]` declaration, retains the source
and logs, and retires the private target/worktree after integration.

Actual-C/live comparison belongs to the existing combined `sm-pilot` runner.
The integrated candidate at `9ee6419` matches all 9,380 observations against
the retained C reference; see `../TRANSITIONS.md` and
`../out/room-transition/runs/dev-02/result.toml`. These live observations
supplement the authored probes; they do not widen their branch coverage.
The 9,380-step Ceres route can prove its two rightward pre-escape transitions,
actual IRQ progress and the reached background stream. Elevators, alternate
directions, active message-box PLM suspension, timer/music branches and other
background commands remain focused branch tests unless separately live exercised.
The 27-routine implementation does not imply Rust ownership of every foreign
service executed during room loading, nor any original-ROM equivalence.
