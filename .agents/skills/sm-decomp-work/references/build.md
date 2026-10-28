# Build role

1. Recover the assigned checkout, final pin and existing handles/artifacts from
   the handoff. Collect completed stages instead of rebuilding them. Preserve
   frozen source and loaded DLLs; changed inputs need a new source pin.
2. Read PLAN's Fast loop, host constraints and resource rules. Admit jobs using
   fresh RAM/disk and every other admitted job's remaining growth. Count a
   launcher and its Cargo/rustc descendants as one workflow. Use independent
   output directories for concurrent builds and retain the required reserve.
   When DLL growth blocks admission, run an admissible 1.6 GiB normal library
   checkpoint on the coherent frozen pin with the current remaining-growth ledger.
   Bind its result to actual inputs; final DLL and runtime checks remain pending.
3. Run required format, library/no-new-warning and DLL stages with the existing
   verifier. Qualification uses actual output, source/native guards and final
   artifact bytes. A warning increase, denied launch or missing artifact requires
   a correction or supported execution route. Compare warning identities;
   unchanged totals can hide a newly introduced warning.
4. Deliver final DLL/source identity, results and limitations to the replay
   worker and primary writer immediately. Release admission at terminal compilation;
   ready work can continue while results are summarized.

Use local execution for ordinary Windows work. Follow current user CI
authorization and PLAN's host policy. Research outside the normal loop needs an
explicit purpose and bounded result; compilation alone does not qualify runtime.
