# Research the Ceres setup NMI/controller continuation

Status: open
Blocked by: 11
Triage: ready-for-agent
Type: research

Tracking origin: retained decomp-port ticket78
Consumer: milestone06
Source/evidence: [catalog holding](../catalog/ceres-continuation-holding.toml)

## Question

Why does the ordinary-reset causal candidate execute one extra controller poll
at ordinal7741, original frame8508/event63799 (C7960/native7959), after matching
7740updates through frame8317? Read the retained8600 native events and actual C
continuation/NMITIMEN state around setup; trace actual EnableNMI/DisableNMI,
Vector_NMI, explicit waits and ReadJoypadInputs against pinned original-ROM
instructions. `tools/decomp/prefix_host.c` currently presents a poll after every
wrapped C Vector_NMI. That is a concrete hypothesis, not an authorized change.

## Work

Continue the original owner/claim from11. Research reads may inspect held source;
source mutation needs a reviewed revised infrastructure scope. Exact proposed
files are the existing five controller paths plus `tools/decomp/prefix_host.c`;
the additional template remains integrator-owned until reviewed. No new native
`translation-scope-v1`: current promoter only authenticates bank86 projectile
exercise and rejects pipeline/observer/C-template/decomp-prefix gates.

Use authenticated original capture reuse and concrete8600 argv from the holding,
with the declared60s whole initial experiment. Never silently raise the timeout
or count a timed-out subject as passing. Preserve legacy full observer identity
and independently rejecting RNG, omitted-HDMA wait and omitted-screen wait
controls; add a meaningful NMI-disabled/enabled continuation control based on
observed native entry/read-pair/return behavior. EOF during required continuation
must reject without repeated input or invented dispatch/poll.

## Acceptance / Answer

Name the first failing original/C boundary with source line/address and retained
event evidence; prove or reject the hypothesis; state actual writable scope,
complete dependency review and concrete pre/merged gates. Create a dedicated
task child for the reviewed correction under preserved ticket78 authority, then
make06 depend on it. A diagnostic repair does not establish original-ROM full
WRAM/hardware/APU parity or ordinary-reset escape `32→33→34`. Missing material
hardware/ownership requirements remain explicit blockers.

## Completion evidence

Pending researched Answer and task/scope/artifact hashes. For any later runs,
apply both scoped cleanup passes, retain failures/source/store/readers/private
inputs and authenticate afterward; otherwise record no-run cleanup.
