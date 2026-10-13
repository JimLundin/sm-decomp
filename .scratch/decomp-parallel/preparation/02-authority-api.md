# Ticket 02 authority API

Status: implementation contract fixed; source in ownership worktree
Owner: `01a13453-b599-79c1-84d5-713e6d1e3f57`
Replacement coordinator: `01a13460-4aa9-7fe0-865e-a8f54ffca310`

Ticket 03 consumes `assignment::authority`, declared as a child module with
`#[path = "authority.rs"] pub(crate) mod authority;` inside `assignment.rs`.
Existing `Claim`, `Job`, `Update` and `WorkResult` are unchanged.
`assignment::Request` gains `pub authority: Option<SessionAuthority>` with
default `None`; `assignment::main_checkout` becomes `pub(crate)`.

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SessionAuthority {
    pub session_id: String,
    pub generation: u64,
}

pub(crate) fn validate(
    checkout: &Path,
    effort: &str,
    coordinator: &str,
    credential: Option<&SessionAuthority>,
    integration: bool,
) -> anyhow::Result<()>;

pub(crate) fn operate(request: authority::Request)
    -> anyhow::Result<authority::Receipt>;
```

Native coordination adds and forwards the optional authority. Call `validate`
before expensive native mutation/gates; the engine repeats it under its global
lock before durable mutations and retries. Read-only queue status remains
available without a credential. Managed mutation also binds the supplied
`session_id` to the current process `CODEX_THREAD_ID`; aliases such as `/root`
cannot authorize another real session.

`authority::Request` uses serde default plus denied unknown fields, with these
public fields:

```rust
operation: String, // status, prepare, register, transfer, activate
checkout: PathBuf,
coordinator: String,
authority: Option<SessionAuthority>,
preparation: Option<Preparation>,
registration: Option<RoleRegistration>,
transfer: Option<RoleTransfer>,
drill: Option<Artifact>,
```

```rust
Preparation {
    canonical_effort: String,
    capacity: usize,
    admitted_efforts: Vec<String>,
    drill_efforts: Vec<String>,
    bridge: Artifact,
    integrator: RoleRegistration,
}
RoleRegistration {
    token: String,
    kind: RoleKind, // integrator or dispatcher
    session_id: String,
    observation: Artifact,
}
RoleTransfer { token: String, session_id: String, observation: Artifact }
Role { token: String, kind: RoleKind, session_id: String, generation: u64, ... }
```

All these types derive Clone/Debug/Serialize/Deserialize; credentials and
registration/transfer requests deny unknown fields. Authority status returns
`Receipt { schema, status, policy, canonical_effort, capacity, roles, bridge,
drill }`; optional fields are absent before preparation. Policy lives at the
main checkout's `.scratch/decomp-parallel/authority.toml` and uses the existing
global assignment lock. `prepare` pins the retained legacy bridge, explicitly
registers its integrator at generation 1, and does not activate production.
Prepared mutation admits declared disposable `drill_efforts` and explicitly
admitted legacy/maintenance efforts; the canonical production effort remains
pending. Unlisted effort creation is rejected once policy exists.

Registration observation TOML schema `decomp-session-observation-v1` requires
`session_id`, observed running/active `status`, `observed_at`, and nonempty sealed
`evidence` artifacts. Initial integrator matches the calling real session;
registering another dispatcher requires current integrator authority.

Transfer observation schema `decomp-session-transfer-v1` requires original
`token`, `session_id`, `generation`, a stopped status (`completed`, `failed`,
`interrupted`), `observed_at`, sealed `evidence`, `replacement` containing the
new session observation, `children` carrying exact `(effort, claim, agent,
status)` and sealed observations, and observed `processes`. Every held original
claim is accounted for. A surviving child retains its claim and writable
ownership; transfer changes only the role generation. The replacement process
must match its real session. An exact transfer retry succeeds at that new
generation; older credentials reject every mutation, including result retries.

Managed `Update::Started.agent` is the actual launched child thread id. Store
display aliases separately in ticket 03 handoffs. Legacy stored aliases/claims
remain readable. Managed stopped-claim observations add real `session_id`,
`observed_at`, sealed `evidence` and observed processes to the existing exact
claim/agent/coordinator/status schema. Submitted ownership is retained through
integration and cannot be released merely because its implementer stopped.

Each child proof artifact uses `decomp-child-observation-v1` with exact `effort`,
`claim`, optional stored `agent`, real `session_id`, `status`, `observed_at`, and
nonempty sealed inspection `evidence`. Each process proof uses
`decomp-process-observation-v1` and matches exact `pid`, `argv`, `started_at`,
`status`, optional `agent`. Transfer rejects a live process that is not assigned
to a surviving live child. The evidence attests actual tool/process inspection;
the engine validates its identity/seal and does not infer stopped status from
age or an absent parent.

Initial historical integrator preparation: preserve the original persistent
token (`ceres-20261013`). Its generation-1 `SessionObservation.evidence`
includes a checked `TransferObservation` with historical `generation = 0`, old
real thread, stopped status, replacement live observation, and exact held
children from the old ledger, including submitted ones. This prepares session
fencing without releasing/rewriting those legacy claims. Native legacy `Started`
aliases remain readable; child inspection uses the original `owner_thread` from
the sealed bridge. Old raw owner-stop artifact alone is insufficient because it
does not identify the real session or account for surviving children.

Activation accepts a sealed `decomp-concurrency-drill-v1` receipt naming the
same checkout, bridge hash, exact current source fingerprint/full HEAD and all
required passed controls. Preparation and the drill never launch gameplay.
The current user request requires preparation only; production start remains a
separate user instruction.

## Bridge normalization needed

The stable bridge now exists. Its maintenance approval references the checked
recommendation. For ticket 03 the recommendation's legacy hold id
`decomp-port-141-focused-shared-registration` must be bound to actual bridge
holding `decomp-port-141`, with the exact `main.rs` maintenance path only. The
same exact maintenance path overlaps `ceres-host-adapter-03`; name that original
holding too. Add normalized `[[maintenance_adoptions]]` entries to the bridge
with `id`, `holding_ids`, `effort`, `session_id`, exact `writable_paths`, and
`observation` pointing to the approved sealed maintenance receipt. This keeps
the checked source lineage and does not release any legacy claim. Ticket 02's
actual holding ids already match the bridge.
