# Engine migration

```mermaid
flowchart LR
    DONE["DONE: Shared player update<br/>movement + minimap + settlement"]:::done
    NOW["NOW: Design owned input preparation<br/>controls + movement rules + pose requests"]:::active
    NEXT["NEXT: Migrate input callers<br/>remove 2 more RAM handoffs"]:::pending
    THEN["GAIN: Shared player state<br/>less duplicate coordination"]:::pending
    TARGET["TARGET: Safe modular engine<br/>Map Rando: rooms / sprites / rules"]:::pending
    DONE --> NOW --> NEXT --> THEN --> TARGET
    class DONE,NOW,NEXT,THEN changed
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

[Navigate the current codebase →](https://jimlundin.github.io/sm-decomp/)

Green: adopted scope / passed check | Blue: draft | Orange: active | Grey: remaining | Red: failed
**Purple outline: changed since previous report.** Folder branches show containment; check arrows show prerequisites.

<details>
<summary>Shared player update: adopted / files and local checks</summary>

```mermaid
flowchart LR
    UPDATE["player_update/ / ADOPTED"]:::done
    UPDATE --> API["mod.rs / one update operation"]:::done
    UPDATE --> MAP["minimap.rs / exploration + HUD + bosses"]:::done
    UPDATE --> STATE["state.rs / 288 owned bytes"]:::done
    UPDATE --> CONTENT["content.rs / map inputs"]:::done
    UPDATE --> EFFECTS["effects.rs / ordered effects"]:::done
    UPDATE --> HOST["player_update_runtime.rs / compatibility edge"]:::done
    STATE --> SESSION["session.rs + session_encoding.rs<br/>shared owner + byte compatibility"]:::done
    HOST --> CALLERS["sm_90.rs + 5 boss bank files<br/>actual callers migrated"]:::done
    CALLERS --> RETIRE["Old minimap + 2 wrappers / retired"]:::done
    API --> BUILD["Formatting + compile + DLL / PASS"]:::done
    UPDATE --> PURE["Core compiles without backend / PASS"]:::done
    BUILD --> BETA["6 actual update comparisons / PASS"]:::done
    BUILD --> ENABLED["6 enabled-minimap checks / PASS"]:::done
    BUILD --> MOVIE["8,577 frames / 0 failures<br/>3 required probes + follow-up / PASS"]:::done
    ENABLED --> ALIASES["2 live alias checks / PASS"]:::done
    ENABLED --> BOSS["6 boss-mark checks / PASS"]:::done
    ENABLED --> EDGES["4 half-map + partial-write checks / PASS"]:::done
    ENABLED --> CODEC["30 HUD byte writes + room aliases / PASS"]:::done
    ENABLED --> LEGACY["Special movement + legacy map writer / PASS"]:::done
    BETA --> TRANSFERS["Normal / inactive gamma<br/>2 transfer pairs become 1 / measured"]:::done
    EDGES --> MUT["3 local mutation kills / audited"]:::done
    TRANSFERS --> MUT
    MUT --> M1["Stale exploration read / KILLED"]:::done
    MUT --> M2["Skipped gamma effect / KILLED"]:::done
    MUT --> M3["Deferred HUD write / KILLED"]:::done
    RETIRE --> ADOPT["Adopted / +225 production lines"]:::done
    MOVIE --> ADOPT
    MUT --> ADOPT
    class UPDATE,API,MAP,STATE,CONTENT,EFFECTS,HOST,SESSION,CALLERS,RETIRE,ALIASES,BOSS,EDGES,CODEC,LEGACY,TRANSFERS,MUT,M1,M2,M3,ADOPT changed
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef draft fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

[Adopted update](sm/src/game/player_update/mod.rs) / [Checks and limits](docs/PLAN.md#work-order) / source `31ef4c71`.

</details>

<details>
<summary>Movement adopted: files and local checks</summary>

```mermaid
flowchart LR
    MOVE["player_movement/"]:::done
    MOVE --> API["mod.rs / advance_normal"]:::done
    MOVE --> SPEED["speed.rs / shared motion rules"]:::done
    MOVE --> MODES["selectors.rs / 28 movement modes"]:::done
    MOVE --> FX["presentation.rs / footsteps + echoes"]:::done
    MOVE --> DATA["content.rs"]:::done
    MOVE --> EFFECT["effects.rs"]:::done
    MOVE --> HOST["player_movement_runtime.rs"]:::done
    HOST --> CALL["Actual callers + 52 routines retired"]:::done
    CALL --> BUILD["Format + normal compile + DLL / PASS"]:::done
    BUILD --> MODES28["28 modes + 20 edge comparisons / PASS"]:::done
    BUILD --> OWN["Shared ownership + 192 spin checks / PASS"]:::done
    BUILD --> SHARED["4 shared-producer comparisons / PASS"]:::done
    MOVE --> DETACHED["Compile without backend / PASS"]:::done
    MODES28 --> EDGE["6 exceptional alias / failure checks / PASS"]:::done
    BUILD --> AUDIT["Caller + deletion audit / PASS"]:::done
    MODES28 --> REPLAY["8,774 frames / 0 failures<br/>5 required probes + follow-up / PASS"]:::done
    MODES28 --> MUT["9 local mutation kills / audited"]:::done
    MUT --> M1["Native content"]:::done
    MUT --> M2["Dry speed-table retention"]:::done
    MUT --> M3["Echo wrap"]:::done
    MUT --> M4["Signed fraction"]:::done
    MUT --> M5["Scaled deceleration"]:::done
    MUT --> M6["Spin speed bytes"]:::done
    MUT --> M7["Unwind cleanup"]:::done
    MUT --> M8["Input refresh"]:::done
    MUT --> M9["Entry preflight"]:::done
    EDGE --> ADOPT["Adopted / -162 production lines"]:::done
    AUDIT --> ADOPT
    REPLAY --> ADOPT
    MUT --> ADOPT
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef draft fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

[Adopted movement](sm/src/game/player_movement/mod.rs) | [Local checks and limits](docs/PLAN.md#work-order).

</details>

<details>
<summary>Architecture and adopted player work</summary>

```mermaid
flowchart LR
    CORE["Small owned game core"]:::active
    CORE --> VANILLA["Vanilla fidelity profile"]:::active
    CORE --> MEDIA["Custom sprites + palettes"]:::pending
    CORE --> WORLD["Validated room + door graph<br/>compatible resources"]:::done
    CORE --> RULES["Opt-in re-spin + QoL"]:::pending
    MEDIA --> MR["Map Rando integration"]:::pending
    WORLD --> MR
    RULES --> MR
    MR --> REPLAY["Seed + settings + replay compatibility"]:::pending
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

### Player transitions / adopted

```mermaid
flowchart LR
    SESSION["Session + shared Runtime"]:::done --> CORE["player_transitions/"]:::done
    CORE --> A["animation.rs"]:::done
    CORE --> B["atmosphere.rs"]:::done
    CORE --> C["landing.rs"]:::done
    CORE --> D["requests.rs"]:::done
    CORE --> E["actions.rs"]:::done
    CORE --> F["resolver.rs"]:::done
    CORE --> G["pose.rs"]:::done
    CORE --> H["normalization.rs"]:::done
    CORE --> I["jump.rs"]:::done
    CORE --> CALL["3 callers ? finish_movement"]:::done
    CALL --> RET["Legacy bodies + pose facade retired"]:::done
    RET --> BUILD["Format + DLL + detached compilation ?"]:::done
    BUILD --> CASES["79 component comparisons ?"]:::done
    BUILD --> MOVIE["8,577 frames / 0 failures ?"]:::done
    BUILD --> MUT["96 mutation kills / hosted ?"]:::done
    CALL --> NEXT["Shared transitions + palette interval"]:::done
    NEXT --> DAMAGE["Owned fractional health + damage"]:::done
    DAMAGE --> DEL["RAM adapter retired / ?64 production lines"]:::done
    DEL --> LOCAL["87 cases + 8,577 frames ?"]:::done
    LOCAL --> EARLY["Early damage consumption / local kill ?"]:::done
    LOCAL --> FRACTION["Lost health fraction / local kill ?"]:::done
    DEL --> CM["Collision ? mechanism / one Session"]:::done
    CM --> CALLER["player_collision_runtime.rs"]:::done
    CM --> WRAPPER["room_mechanisms/mod.rs<br/>spawn_contact retired"]:::done
    CM --> CLEANUP["session_runtime.rs<br/>shared unwind cleanup"]:::done
    CALLER --> CHECK["Full slots / aliases / stale world / panic ?<br/>collision36/37 exact comparisons ?"]:::done
    CHECK --> CBUILD["Format + normal compilation + DLL ?"]:::done
    CBUILD --> CMOVIE["22,110 frames / both probes + tail ?"]:::done
    CMOVIE --> M1["Unwind cleanup / local kill ?"]:::done
    CMOVIE --> M2["Input refresh / local kill ?"]:::done
    CMOVIE --> M3["Entry preflight / local kill ?"]:::done
    CMOVIE --> M4["Displacement / local kill ?"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef draft fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

[Adopted source](https://github.com/JimLundin/sm-decomp/tree/5e546bdc85c8d6f5216c4bf69f247b0299ec1e20/sm/src/game/player_transitions) ? [96 checks](https://github.com/JimLundin/sm-decomp/actions/runs/40078990293) ? [Local validation](docs/PLAN.md#work-order)

</details>

| Rust accounting | Main | Latest adoption |
| --- | ---: | ---: |
| Production, including adapters | 117,001 | +225 |
| Verification and check support | 36,638 | +1,612 |
| Combined maintained Rust | 153,639 | +1,837 |

**36,638 verification lines excluded, not deleted.** Separate support: +169 Python/JSON lines. [Scope and local evidence](docs/PLAN.md#work-order).

<details>
<summary>Verification details and earlier migrations</summary>

### Common actors / adopted

```mermaid
flowchart LR
    AC["Complete callers + retirement<br/>design review PASS"]:::done --> AD["49-file declaration carrier<br/>source review PASS"]:::done
    AU["4,668 new bytes / union 57,041<br/>shared player scope audit PASS"]:::done --> AD
    AI["Per-instance appearance + lifecycle<br/>independent review PASS"]:::done --> AD
    AR["Typed read lookup + guarded edits<br/>design review PASS"]:::done --> AD
    AP["Production +4,998 / verification +5,406<br/>accounting + retirement audit PASS"]:::done --> AD
    AD --> AS["state.rs + codec.rs"]:::done
    AD --> AV["access.rs / typed reads + edits"]:::done
    AD --> AE["content.rs + effects.rs"]:::done
    AD --> AM["identity / media / scene / snapshots"]:::done
    AS --> AT["Independent declaration review<br/>PASS"]:::done
    AV --> AT
    AE --> AT
    AM --> AT
    AT --> ATC["Metadata + external constructor<br/>compile PASS"]:::done
    ATC --> AB["Parallel actor implementation"]:::done
    AV --> APR["Exceptional buffers<br/>helper review"]:::done --> AB
    AB --> AX["Combined predecessor<br/>format + build + source checks PASS"]:::done
    AX --> AXPOP["35 population / animation references<br/>capture PASS"]:::done
    AX --> AXMOVE["40 movement / contact references<br/>capture PASS"]:::done
    AXPOP --> AXAUDIT["75 captures / independent audit PASS"]:::done
    AXMOVE --> AXAUDIT
    AX --> AXTABLE["4 table-boundary references<br/>capture + independent audit PASS"]:::done
    AB --> ACOMPILE["Combined actor library<br/>format + cargo check PASS"]:::done
    AB --> ARETIRE["7 old actor modules + facades<br/>retired in production"]:::done
    ACOMPILE --> ADLL["79-case DLL<br/>build + source checks PASS"]:::done
    ADLL --> APCOMP["35 population / animation<br/>byte-identical"]:::done
    ADLL --> AMCOMP["40 movement / contact<br/>byte-identical"]:::done
    ADLL --> ATCOMP["4 table boundaries<br/>byte-identical"]:::done
    APCOMP --> ACAUDIT["79 comparisons + Runtime outputs<br/>independent audit PASS"]:::done
    AMCOMP --> ACAUDIT
    ATCOMP --> ACAUDIT
    ACAUDIT --> AVERIFY
    ADLL --> ARUN["5 ownership / prepared-media cases<br/>actual execution PASS"]:::done
    ADLL --> ASHARED["6 shared Runtime guards<br/>PASS"]:::done
    ADLL --> AINIT["Real spawn + nested initializer<br/>publication / unwind baseline PASS"]:::done
    ADLL --> AANGLE["18 angle aliases / foreign + drop<br/>single-owner baseline PASS"]:::done
    ADLL --> ACONTACT["Touch alias + two-skin collision<br/>baseline PASS"]:::done
    AINIT --> AHOOK["Observer hooks / 79 comparisons<br/>byte-identical"]:::done
    AANGLE --> AHOOK
    ACONTACT --> AHOOK
    AB --> AROUTEBUILD["Composed route / normal library + DLL<br/>format + compile PASS"]:::done
    AROUTEBUILD --> AROUTE["Two-room routes / cases 12 + 13<br/>execution + byte audit PASS"]:::done
    AROUTE --> AOLDWORLD["11 existing supplied-world routes<br/>byte-identical"]:::done
    AVERIFY --> MUTWIRE["Hosted / 62 detections audited<br/>11 actor DLL faults unrun / efdd9cda"]:::active
    MUTWIRE --> AHF["First CI / animation harness<br/>compile FAIL"]:::failed
    AHF --> AHFIX["Shared crate-root declarations<br/>all 3 harnesses hosted PASS"]:::done
    AHFIX --> ACFG["Second CI / ordinary verifier<br/>configuration compile FAIL"]:::failed
    ACFG --> ACFGLOCAL["Both modes compile + 3 captures<br/>configuration fix PASS"]:::done
    ACFGLOCAL --> AHOSTBUILD["Hosted full build + Cargo exports<br/>PASS"]:::done
    AHOSTBUILD --> AHRUN["Full CI / 50cec3e0 FAIL<br/>53 killed / 1 survived / 19 unrun"]:::failed
    AHRUN --> ANATIVE["Pre-Drop ownership witness<br/>review + compile + 3 baselines PASS"]:::done
    ANATIVE --> ARETRY["CI efdd9cda / 62 killed<br/>actor baseline buffer FAIL"]:::failed
    ARETRY --> ABUFFER["4 MiB buffer / all 5 selections<br/>local baselines PASS"]:::done
    ABUFFER --> APDET["Parallel CI bdbb7968<br/>48 / 48 detached faults KILLED"]:::done
    ABUFFER --> APDLL["Parallel CI bdbb7968<br/>25 / 25 DLL faults KILLED"]:::done
    ADLL --> AORACLE["21,015 frames / 10 probes<br/>0 failures + 120-frame tail / PASS"]:::done
    ADLL --> KWRITE["Kraid live-definition write + consumer<br/>PASS"]:::done
    KWRITE --> KRESTORE["Actual CPU snapshot restore<br/>definition retained / PASS"]:::done
    KRESTORE --> KRESET["Actual reset + seeded consumer<br/>definition retained / PASS"]:::done
    AB --> ACONTRACT["18 detached contract tests<br/>hosted execution PASS"]:::done
    AXTABLE --> AVERIFY
    AXAUDIT --> AVERIFY["Actual routes + oracle + mutations"]:::done
    AV --> BORROW["Actual state/access compiler carrier<br/>positive control PASS"]:::done
    BORROW --> BREAD["Actor read across edit<br/>compiler rejects / PASS"]:::done
    BORROW --> BSREAD["Spawn read across edit<br/>compiler rejects / PASS"]:::done
    BORROW --> BESC["Mutable edit escape<br/>compiler rejects / PASS"]:::done
    BORROW --> BCOPY["Actor move out of view<br/>compiler rejects / PASS"]:::done
    BORROW --> BSCOPY["Spawn move out of view<br/>compiler rejects / PASS"]:::done
    AVERIFY --> AA["Caller retirement + adoption PASS"]:::done
    AVERIFY --> MUTRECIPES["17 unique actor faults<br/>17 / 17 KILLED"]:::done
    MUTRECIPES --> MUTBASE["Detached + selected DLL baselines<br/>PASS"]:::done
    MUTBASE --> MUTBUILD["Mutant compilation<br/>detached + DLL PASS"]:::done
    MUTBUILD --> MF1["Multipart stop"]:::done
    MUTBUILD --> MF2["Birth publication"]:::done
    MUTBUILD --> MF3["Live selector"]:::done
    MUTBUILD --> MF4["Respawn preservation"]:::done
    MUTBUILD --> MF5["One angle owner"]:::done
    MUTBUILD --> MF6["Unwind identity"]:::done
    MUTBUILD --> MF7["Alias publication"]:::done
    MUTBUILD --> MF8["Skin / collision independence"]:::done
    MUTBUILD --> MF9["Discarded-future handles<br/>KILLED"]:::done
    MUTBUILD --> MFROW["Old palette receipts<br/>KILLED"]:::done
    MUTBUILD --> MFRES["Skip appearance residency<br/>KILLED"]:::done
    MUTBUILD --> MFEDIT["Discard guarded edits<br/>KILLED"]:::done
    MUTBUILD --> MFDAFF["Retire converted occurrence<br/>KILLED"]:::done
    MUTBUILD --> MFMEDIA["Omit default appearance revision<br/>KILLED"]:::done
    MUTBUILD --> MFRENDER["Flash / frozen palette bits"]:::done
    MUTBUILD --> MFKWRITE["Kraid live shot-gate write"]:::done
    MUTBUILD --> MFKREAD["Contact reads live shot gate"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef draft fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
```

### Player collision / adopted

```mermaid
flowchart LR
    PC["Player collision + displacement"]:::done --> PS["58 state bytes / all writers<br/>source closure PASS"]:::done
    PC --> PR["33 bank routines + 2 modules<br/>retirement review PASS"]:::done
    PC --> PV["Directed routes + 10 fault witnesses<br/>validation design reviewed"]:::done
    PS --> PD["Concrete owned interface<br/>declaration review PASS"]:::done
    PR --> PD
    PD --> PCOMP["Normal + Runtime + backend-free<br/>compile PASS"]:::done
    PCOMP --> PBA["Actor borrow across effect<br/>compiler rejects / PASS"]:::done
    PCOMP --> PBP["Player borrow across effect<br/>compiler rejects / PASS"]:::done
    PCOMP --> PBT["Terrain borrow across effect<br/>compiler rejects / PASS"]:::done
    PCOMP --> PBS["Schedule borrow across effect<br/>compiler rejects / PASS"]:::done
    PCOMP --> PBE["Guarded edit escapes<br/>compiler rejects / PASS"]:::done
    PCOMP --> PAUDIT["Compiler evidence<br/>independent audit PASS"]:::done
    PAUDIT --> PAP["Seven operations / one Session<br/>design approved"]:::done
    PAP --> PI["Five authors + caller integrator<br/>adopted actor baseline ddd21882"]:::done
    PV --> PI
    PV --> PREF["74 predecessor cases<br/>assembly review PASS"]:::done
    PREF --> PMOVE["36 movement / speed cases<br/>source review PASS"]:::done
    PREF --> PPOSE["24 wall / pose cases<br/>source review PASS"]:::done
    PREF --> POWN["12 history / ownership cases<br/>source review PASS"]:::done
    PREF --> PROUTE["2 composed routes / full phase RAM<br/>source review PASS"]:::done
    PMOVE --> PBUILD["Format + normal library<br/>compile PASS"]:::done
    PPOSE --> PBUILD
    POWN --> PBUILD
    PROUTE --> PBUILD
    PBUILD --> PDLL["Verifier compile FAIL<br/>unsafe helper in safe actor module"]:::failed
    PDLL --> PFIX["Separate verification choreography<br/>preserve forbid unsafe / PASS"]:::done
    PFIX --> PFINAL["Format + DLL + native/source checks<br/>PASS"]:::done
    PFINAL --> PCAP["74 predecessor captures<br/>PASS"]:::done
    PFINAL --> POLD["13 original world cases<br/>byte-identical"]:::done
    PCAP --> PEXEC["74 captures / 18,089,181 bytes<br/>independent audit PASS"]:::done
    PEXEC --> PWALK["Walked-obstacle source + admission<br/>review PASS"]:::done
    PWALK --> PWREF["9 additive cases 88–96<br/>format + DLL + captures PASS"]:::done
    PWREF --> PWAUDIT["Current / pending + native / loaded<br/>9-case execution audit PASS"]:::done
    PI --> PCS["state.rs"]:::done
    PCS --> PCSE["session.rs + session_encoding.rs"]:::done
    PCSE --> PCIN["external_inputs.rs + shared consumers"]:::done
    PI --> PCG["geometry.rs"]:::done
    PI --> PCM["motion.rs"]:::done
    PI --> PCC["control.rs"]:::done
    PCC --> PCJ["jump.rs"]:::done
    PCC --> PCP["pose.rs"]:::done
    PI --> PCRT["Runtime adapters / 16 dependencies"]:::done
    PCIN --> PCSR["State / codec source audit PASS"]:::done
    PCM --> PCMR["Motion / arithmetic source review PASS"]:::done
    PCP --> PCPR["Pose / control source review PASS"]:::done
    PCRT --> PCAR["Adapter / effect source review PASS"]:::done
    PCG --> PCGR["Geometry arithmetic source review PASS"]:::done
    PI --> PCAPI["operations.rs + effects.rs + mod.rs"]:::done
    PCAPI --> PCCALL["33 bank routines / actual callers"]:::done
    PCCALL --> PCDEL["2 contact modules + 3 actor helpers<br/>retired in production"]:::done
    PWAUDIT --> PCOWN["Owned-state / ordering checks PASS"]:::done
    PCOWN --> PCFAULT["Targeted mutation recipes"]:::done
    PCIN --> PCMERGE["Combined candidate / no stubs"]:::done
    PCG --> PCMERGE
    PCM --> PCMERGE
    PCJ --> PCMERGE
    PCP --> PCMERGE
    PCRT --> PCMERGE
    PCDEL --> PCMERGE
    PCMERGE --> PCFMT["Format + normal / verifier metadata<br/>PASS"]:::done
    PCFMT --> PCTESTMETA["15 collision tests<br/>hosted runner PASS"]:::done
    PCFMT --> PCRELEASE["Final overlays + release DLL<br/>source / native checks PASS"]:::done
    PCFMT --> PCCOMPARE["74 + 9 frozen comparisons<br/>20,448,594 bytes identical"]:::done
    PCCOMPARE --> PCINVAR["3 actual Runtime invariants PASS"]:::done
    PCCOMPARE --> PCOLD["13 old world / composed cases<br/>4,063,411 bytes identical"]:::done
    PCRELEASE --> PCGUARD["6 shared Runtime guards<br/>21 expected panics PASS"]:::done
    PCRELEASE --> PCACTOR["9 adjacent actor cases<br/>1,442,111 bytes identical"]:::done
    PCRELEASE --> PCCONTACT["11 adjacent contact cases<br/>3,670,343 bytes identical"]:::done
    PCFMT --> PCDET["3 backend-free core checks<br/>hosted PASS"]:::done
    PCFMT --> PCBUILD["Hosted game build + Cargo exports<br/>PASS"]:::done
    PCFMT --> PCORACLE["21,015 frames / 14 probes<br/>0 failures + 120-frame tail PASS"]:::done
    PCFAULT --> PCMUT["Hosted CI / 91 faults caught<br/>independent audit PASS"]:::done
    PCFMT --> PCMUT
    PCMUT --> PCMD["54 detached faults caught<br/>independent audit PASS"]:::done
    PCMUT --> PCML["37 DLL faults caught<br/>independent audit PASS"]:::done
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef draft fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

### Player animation + transitions / verified draft

```mermaid
flowchart LR
    TS["Complete interaction scope"]:::done --> TO["64 new bytes / 14 rehomed<br/>ownership + caller review PASS"]:::done
    TS --> TC["Content + finite effects<br/>review PASS"]:::done
    TO --> TI["Normal / verifier / detached<br/>interface compilation PASS"]:::done
    TC --> TI
    TC --> TU["Native word reads / 11 CPU references<br/>7 original + 4 authored PASS"]:::done
    TI --> TA["Animation + liquid effects"]:::pending
    TI --> TH["Hit / bomb + block requests"]:::pending
    TA --> TR["Arbitration + pose + jump + history"]:::pending
    TH --> TR
    TU --> TUA["Capture + provenance audits PASS"]:::done
    TUA --> TUD["Native-read correction / isolated draft<br/>format + normal + release build PASS"]:::done
    TUD --> TUF["Attempt 1 / allocation preflight FAIL"]:::failed
    TUF --> TUX["Exact loader extent + original spans<br/>review PASS"]:::done
    TUX --> TUG["Attempt 2 / comparator mapping FAIL"]:::failed
    TUG --> TUH["Separate snapshot / store tracking<br/>review PASS"]:::done
    TUH --> TN["7 normalization comparisons<br/>PASS"]:::done
    TUH --> TM["4 authored animation comparisons<br/>PASS"]:::done
    TN --> TVAUD["11-case output + provenance<br/>independent audits PASS"]:::done
    TM --> TVAUD
    TVAUD --> TV["36 remaining fixture families"]:::active
    TV --> T48B["Ordinary animation / isolated draft<br/>format + normal + DLL build PASS"]:::done
    T48B --> T48["48 authored animation cases<br/>capture + completion checks PASS"]:::done
    T48B --> T11["Original 11 cases<br/>CPU + exact saved bytes PASS"]:::done
    T48 --> T48A["48-case independent capture audit"]:::active
    TV --> TPOSE["Pose / history fixture batch"]:::active
    TV --> TCONTENT["Speed + content fixture batch"]:::active
    TV --> TREQUEST["Hit + block request fixture batch"]:::active
    TV --> TFX["Jump + atmosphere fixture batch"]:::active
    TVAUD --> TMT1["Metadata reload mutation<br/>reviewed / unrun"]:::pending
    TVAUD --> TMT2["Animation mask mutation<br/>reviewed / unrun"]:::pending
    class TVAUD,T48B,T48,T11,T48A,TPOSE,TCONTENT,TREQUEST,TFX changed
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

### Room entry + configurable connections / adopted

```mermaid
flowchart LR
    RC["Complete caller + content contract<br/>source review PASS"]:::done --> RD["24 concrete declaration files<br/>source review PASS"]:::done
    RU["84 additional bytes<br/>disjoint union audit PASS"]:::done --> RD
    RR["36 room + 20 contact/start references<br/>execution + independent audit PASS"]:::done --> RI["room_entry/ + actual callers<br/>adopted on main"]:::done
    RD --> RB["Declaration compile<br/>PASS"]:::done --> RS["Remove duplicate representations<br/>52 → 19 host methods"]:::done
    RS --> RV["25-file revised interface<br/>review + compile PASS"]:::done --> RI
    RI --> RW["world.rs / immutable bindings"]:::done
    RI --> RCONTENT["content.rs / live captures"]:::done
    RI --> RSTATE["state.rs / 82 room bytes"]:::done
    RI --> RCODEC["session_encoding.rs / one projection"]:::done
    RI --> ROWNER["game_state.rs / Game + GameOwner<br/>supplied-world constructors"]:::done
    RI --> RSELECT["selection.rs"]:::done
    RI --> RINSTALL["installation.rs"]:::done
    ROWNER --> ROWNERCHECK["Constructor + lifecycle<br/>source review PASS"]:::done
    RSELECT --> RSELECTCHECK["Selection + partial writes<br/>source review PASS"]:::done
    RINSTALL --> RINSTALLCHECK["Installation + retained captures<br/>source review PASS"]:::done
    RI --> RPROTO["protocol.rs / suspend + resume"]:::done
    RI --> RBG["background.rs"]:::done
    RI --> RCOMPAT["compatibility.rs / actual callers"]:::done
    RI --> RPLM["room_mechanisms/creation.rs<br/>owned exit + direction"]:::done
    RI --> RT["Supplied topology / actual callers<br/>11 PASS / output audit PASS"]:::done
    ROWNERCHECK --> RT
    RSELECTCHECK --> RT
    RINSTALLCHECK --> RT
    RBG --> RBPLAN["37 background / scroll / DMA cases<br/>plan review PASS"]:::done
    RBPLAN --> RBFIX["Actual-device fixtures<br/>37 cases / source review PASS"]:::done
    RBFIX --> RBCAP["37 reference captures<br/>execution + byte audit PASS"]:::done
    RI --> RBUILD["Combined candidate<br/>format + build PASS"]:::done
    RBUILD --> RCOMPARE["Room / contact / start<br/>56 comparisons match"]:::done
    RCODEC --> RGUARD["Runtime + world ownership<br/>execution PASS"]:::done
    RBCAP --> RBVERIFY["Replacement background comparisons<br/>38 / 38 byte-identical"]:::done
    RCOMPARE --> RT
    RGUARD --> RT
    RT --> WCONTACT["Contact / cases 0-1<br/>PASS"]:::done
    RT --> WSTART["Initial loading / cases 2-3<br/>PASS"]:::done
    RT --> WDOOR["Full door route / cases 4-5<br/>PASS"]:::done
    RT --> WIRQ["Terminal IRQ / cases 6-7<br/>PASS"]:::done
    RT --> WRESET["Reset frame / case 8<br/>PASS"]:::done
    RT --> WSTATION["Mapped load stations / cases 9-10<br/>PASS"]:::done
    RT --> RHOST["Hosted full build + format<br/>PASS"]:::done
    RHOST --> RCOMPOSE["Composed core execution<br/>PASS"]:::done
    RCOMPOSE --> RDETACHED["42 existing detached mutations<br/>stage PASS"]:::done
    RDETACHED --> RMUT["9 existing + 5 new DLL faults<br/>14 killed / terminal audit PASS"]:::done
    RBCAP --> ROFFSET["VRAM offset / case 37<br/>comparison + audit PASS"]:::done
    RBUILD --> RORACLE["Vanilla / 21,065 frames<br/>13 probes / 0 failures / PASS"]:::done
    RMUT --> MMARK["Late contact marker<br/>KILLED / assertion audit PASS"]:::done
    RMUT --> MLOCK["Late station lockout<br/>KILLED / assertion audit PASS"]:::done
    RMUT --> MAREA["Overwrite area high byte<br/>KILLED / assertion audit PASS"]:::done
    RMUT --> MBIND["Write before binding check<br/>KILLED / assertion audit PASS"]:::done
    RMUT --> MOFFSET["Wrong VRAM offset units<br/>KILLED / assertion audit PASS"]:::done
    RI --> RAPI["api.rs / arrival + resume API"]:::done
    RI --> RMODULE["mod.rs / owned entry module"]:::done
    RCONTENT --> RCSTATE["content_state.rs / state records"]:::done
    RCONTENT --> RCBG["content_background.rs / command records"]:::done
    RCONTENT --> RCRUNTIME["content_runtime.rs / live acquisitions"]:::done
    RCOMPAT --> REFFECTS["effects.rs / closed effect interface"]:::done
    RCODEC --> RSESSION["session.rs / immutable binding<br/>2 shared continuation bytes"]:::done
    RCODEC --> RRUNTIME["session_runtime.rs / interval transfer"]:::done
    ROWNER --> RCPU["cpu_infra.rs / persistence guard"]:::done
    ROWNER --> RRTL["rtl.rs / reset + persistence boundaries"]:::done
    RI --> RSM80["sm_80.rs / start + initial callers<br/>obsolete station structure retired"]:::done
    RI --> RSM82["sm_82.rs / door + IRQ callers"]:::done
    RI --> RCONTACT["samus_room_contact.rs / arrival decision"]:::done
    RCONTACT --> RCONTACTRAM["samus_room_contact_ram.rs / actual entry"]:::done
    RPLM --> REXTERNAL["external_inputs.rs / duplicate door fields retired"]:::done
    RINSTALL --> ROLDSTATE["room_state.rs / compatibility entries<br/>old rule bodies retired"]:::done
    RBG --> ROLDBG["room_backgrounds.rs / old rule bodies retired"]:::done
    RCOMPARE --> RALL["All 94 comparisons<br/>match frozen bytes"]:::done
    RBVERIFY --> RALL
    RGUARD --> RSHARED["6 shared Runtime guards + 106 player groups<br/>execution PASS"]:::done
    RALL --> RAUDIT["Predecessor + candidate outputs<br/>independent byte audit PASS"]:::done
    RT --> RAUDIT
    RMUT --> RMUTAUDIT["56 total mutation kills<br/>exact terminal-log audit PASS"]:::done
    RDETACHED --> RMUTAUDIT
    RI --> RCOUNT["Full production accounting<br/>+3,242 lines / +26,275 tokens<br/>692 lines above estimate"]:::done
    RI --> RLIFE["Persistent Session<br/>frame/lifecycle ownership remains"]:::pending
    RW --> RIMPORT["Map Rando importer<br/>portable prepared save/replay remains"]:::pending
    RCOMPAT --> RBACKEND["Independent assets + graphics/audio backend<br/>ROM compatibility + unsafe leaves remain"]:::pending
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef draft fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

[Room validation CI / terminal audit PASS](https://github.com/JimLundin/sm-decomp/actions/runs/40044596920) | [Scope, accounting and remaining boundaries](docs/PLAN.md)

### Progression flags / adopted

```mermaid
flowchart TB
    FCONTRACT["Interface + declarations<br/>review / compile PASS"]:::done
    FUNION["210 new bytes<br/>overlap audit PASS"]:::done
    FCONTRACT --> FCORE["progress_flags.rs<br/>adopted / reviewed"]:::done
    FUNION --> FCODEC["Session + codec + Runtime<br/>adopted / reviewed"]:::done
    FCORE --> FPLM["room_mechanisms / 14 files<br/>adopted / reviewed"]:::done
    FCORE --> FVIS["palette + animation / 4 files<br/>adopted / reviewed"]:::done
    FCORE --> FEPR["projectiles / 5 files<br/>adopted / reviewed"]:::done
    FCORE --> FLEG["legacy callers / 18 files<br/>adopted / reviewed"]:::done
    FLEG --> FSAVE["sm_81 save alignment repair<br/>adopted / reviewed"]:::done
    FSAVE --> FCPU["Save alignment / original CPU<br/>2 × 7 stages match"]:::done
    FCODEC --> FGUARD["210-byte ownership guard<br/>candidate execution PASS"]:::done
    FBASE["71 reference captures<br/>execution + audit PASS"]:::done --> FCHECK["Candidate comparisons<br/>71 / 71 match + 2 saves match"]:::done
    FPLM --> FCHECK
    FVIS --> FCHECK
    FEPR --> FCHECK
    FLEG --> FCHECK
    FCPU --> FCHECK
    FGUARD --> FCHECK
    FCHECK --> FSHARED["39 shared-consumer comparisons<br/>+ visual guard PASS"]:::done
    FLEG --> FKEVEN["Kraid / 8 even-area references<br/>actual caller execution PASS"]:::done
    FLEG --> FKODD["Kraid / original CPU<br/>12 cases / 36 stages match"]:::done
    FSHARED --> FMUT["42 detached + 9 DLL mutations<br/>51 / 51 caught"]:::done
    FKEVEN --> FKCOMPARE["Kraid candidate comparisons<br/>8 defined + 12 CPU PASS"]:::done
    FKODD --> FKCOMPARE
    FKCOMPARE --> FADOPT
    FMUT --> FADOPT["Progression owner<br/>adopted on main"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef draft fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

[Flags adopted: f88970aa](https://github.com/JimLundin/sm-decomp/commit/f88970aa) · [Build + 51 mutations: PASS](https://github.com/JimLundin/sm-decomp/actions/runs/40036842766)

```mermaid
flowchart LR
    FB["Composed baseline / PASS"]:::done --> FEVENT["Discard event write"]:::done
    FB --> FHI["Keep stale event mask"]:::done
    FB --> FBOSS["Clear boss mask high byte"]:::done
    FB --> FMAP["Write wrong object map"]:::done
    FB --> FRNG["Cache scratch before RNG"]:::done
    FB --> FEYE["Reuse Eye Door mask"]:::done
    FB --> FADJ["Drop adjacent door byte"]:::done
    FB --> FPRE["Read door before marking"]:::done
    FB --> FCLAMP["Clamp broad flag index"]:::done
    FB --> FNATIVE["Lose native high byte"]:::done
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
```

<details>
<summary>Adopted sprite effects — shared owner and verification</summary>

```mermaid
flowchart TB
    CONTRACT["Complete responsibility<br/>source review PASS"]:::done
    UNION["1,026 new bytes / no overlap<br/>source review PASS"]:::done
    CONTRACT --> DECL["Concrete Rust interfaces<br/>metadata compile PASS"]:::done
    UNION --> DECL
    DECL --> REVIEW["Independent interface review<br/>PASS / live actor reads"]:::done
    DECL --> OWNER["session.rs / sprite state<br/>52,079-byte owned union"]:::done
    OWNER --> CODEC["session_encoding.rs / runtime"]:::done
    DECL --> POOL["sprite_objects.rs replacement<br/>spawn / advance / draw / clear"]:::done
    POOL --> WAFFLE["enemy_projectiles/<br/>spawn + Waffle controllers"]:::done
    POOL --> METROID["sm_a3.rs / Metroid controllers"]:::done
    POOL --> DRAYGON["sm_a5.rs / Draygon controllers"]:::done
    POOL --> MAW["sm_a8.rs / Maw controllers"]:::done
    BASE["121 predecessor cases<br/>captures + alias assertions PASS"]:::done --> VERIFY["Candidate comparisons<br/>121 / 121 match"]:::done
    VERIFY --> MAWFIX["Maw appearance precedence<br/>both writes corrected / PASS"]:::done
    CPU["Draygon motion / original CPU<br/>5 candidate phase comparisons PASS"]:::done --> VERIFY
    PROBE["Sprite lifecycle + draw probes<br/>source review PASS"]:::checked --> EARLY["Two early routes / missing hits<br/>0 failing frames / coverage FAIL"]:::failed
    EARLY --> MOVIE["52-phase original CPU + OAM<br/>candidate comparison PASS"]:::done
    MOVIE --> HOOKFIX["Isolated original opcode restore<br/>strict guards + capture audit PASS"]:::done
    POOL --> BUILD["Assembled verifier + adapters<br/>format / release build PASS"]:::done
    BUILD --> HOST["Hosted composition baseline<br/>execution PASS"]:::done
    HOST --> HOSTFIX["Preserve larger fixture image<br/>corrected / baseline PASS"]:::done
    GUARD["Runtime + 1,026 bytes guard<br/>execution PASS"]:::done --> VERIFY
    MUT["32 detached mutations<br/>caught / stage PASS"]:::done --> KILLS["9 DLL mutations<br/>caught / PASS"]:::done
    MAWFIX --> MAWMUT["Two Maw precedence mutations<br/>caught / PASS"]:::done --> MUT
    SHARED["35 shared-caller comparisons<br/>+ visual runtime guard PASS"]:::done --> ADOPT
    CODEC --> VERIFY
    WAFFLE --> VERIFY
    METROID --> VERIFY
    DRAYGON --> VERIFY
    MAW --> VERIFY
    VERIFY --> ADOPT["Caller migration + retirement<br/>adopted on main"]:::done
    MOVIE --> ADOPT
    KILLS --> ADOPT
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

[Sprite build + 41 mutations: PASS](https://github.com/JimLundin/sm-decomp/actions/runs/40031500027)

```mermaid
flowchart LR
    subgraph pool["Sprite pool faults / caught"]
        A["Wrong free slot"]:::done
        B["Clear slot zero"]:::done
        C["Keep compatibility tails"]:::done
        D["Retirement clears timers"]:::done
        E["Cache selected slot"]:::done
        F["Read content before callback"]:::done
    end
    subgraph drawing["Drawing faults / caught"]
        G["Reverse draw order"]:::done
        H["Cache future draw slots"]:::done
    end
    subgraph controllers["Controller faults / caught"]
        I["Drop fractional carry"]:::done
        J["Cache actor Y"]:::done
        K["Cache first timer read"]:::done
        L["Cache second timer read"]:::done
        M["Wrong first Maw mask"]:::done
        N["Wrong final Maw mask"]:::done
    end
    pool --> drawing
    pool --> controllers
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef changed stroke:#a21caf,stroke-width:5px
```

</details>

[Shared Session adopted: 74948127](https://github.com/JimLundin/sm-decomp/commit/74948127) / [Build + eight mutations: PASS](https://github.com/JimLundin/sm-decomp/actions/runs/40005712881)

[Four room mutations / CI PASS](https://github.com/JimLundin/sm-decomp/actions/runs/40005395335) · [Shared Session candidate CI](https://github.com/JimLundin/sm-decomp/actions/runs/40005712881)

### Enemy projectiles / shared owner adopted

[Full build + 16 mutations / PASS](https://github.com/JimLundin/sm-decomp/actions/runs/40013612299)

```mermaid
flowchart TB
    CONTRACT["Complete contract<br/>independent review PASS"]:::done
    UNION["49,819 WRAM + 16 native bytes<br/>no duplicate ownership"]:::done
    CONTRACT --> API["Concrete Rust interfaces<br/>metadata compile PASS"]:::done
    API --> REVIEW["Concrete interface review<br/>PASS / area cache removed"]:::done
    UNION --> API
    REF["1,382 captures + 87 checks<br/>current main PASS"]:::done
    API --> S["session.rs / session_encoding.rs"]:::done
    S --> R["session_runtime.rs<br/>one owner / native spawn fields"]:::done
    API --> SETUP["setup.rs / setup_helpers.rs"]:::done
    API --> PRE["pre_actions.rs / shared_pre.rs"]:::done
    API --> LIFE["lifecycle.rs / commands.rs"]:::done
    API --> CONTACT["contact.rs / collision.rs"]:::done
    API --> ATTACH["attachments_spore_maw.rs<br/>attachments_waffle_botwoon.rs"]:::done
    API --> DROP["drops.rs"]:::done
    DROP --> INV["item_pickups.rs / restore<br/>source review PASS"]:::done
    INV --> IRT["item_pickups_runtime.rs<br/>source review PASS"]:::done
    INV --> RESOURCE["drop_rules.rs<br/>one Resource / review PASS"]:::done
    IRT --> CALL["sm_90.rs / sm_a2.rs / sm_91.rs<br/>7 callers / 4 bodies retired"]:::done
    R --> RETIRE["5 files + 5 getters retired<br/>adopted"]:::done
    SETUP --> MERGE["Whole candidate integrated<br/>build PASS"]:::done
    PRE --> MERGE
    LIFE --> MERGE
    CONTACT --> MERGE
    ATTACH --> MERGE
    CALL --> MERGE
    RETIRE --> MERGE
    RESTORE["58 restoration cases<br/>review + capture PASS"]:::done --> VERIFY["1,440 comparisons + 87 checks<br/>PASS"]:::done
    NATIVE["Native spawn / shared state checks<br/>design review PASS"]:::done --> VERIFY
    NATIVE --> NRET["1440 / native return PASS"]:::done
    NATIVE --> NPANIC["1441 / foreign panic PASS"]:::done
    NATIVE --> CPANIC["1442 / core panic PASS"]:::done
    MERGE --> VERIFY
    REF --> VERIFY
    REF --> BASEPLAY["Baseline replay / 26 probes<br/>28,314 frames / 0 failures"]:::done
    MERGE --> ROOM["1,038 room comparisons<br/>PASS"]:::done
    MERGE --> HOST["Composed host compile<br/>PASS"]:::done
    HOST --> HFAIL["Hosted execution failed<br/>1058 fixture too small"]:::failed
    HFAIL --> HFIX["enemy_checks.rs<br/>case-only fix / execution PASS"]:::done
    VERIFY --> MUT["16 targeted mutations<br/>16 / 16 caught"]:::done
    HFIX --> MUT
    MERGE --> PLAYER["106 groups / 56 comparisons<br/>2 runtime guards PASS"]:::done
    MERGE --> FRONT["Full build + formatting<br/>GitHub PASS"]:::done
    PLAYER --> PLAY["28,314 frames / 26 probes<br/>0 failures / 90.08s"]:::done
    PLAY --> ADOPT["Enemy shared core adopted"]:::done
    MUT --> ADOPT
    FRONT --> ADOPT
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

```mermaid
flowchart LR
    B["Passing baseline"]:::done --> C["Composed core"]:::done
    C --> C1["Beam radius / caught"]:::done
    C --> C2["Collision retirement / caught"]:::done
    C --> C3["Plant position / caught"]:::done
    C --> C4["New beam / caught"]:::done
    C --> C5["Private inventory copy / caught"]:::done
    C --> C6["Discarded shot mark / caught"]:::done
    C --> C7["Stale room selection / caught"]:::done
    D["9 frozen DLL baselines / PASS"]:::done --> R["Room scheduler"]:::done
    R --> R1["Repeated pre-action / caught"]:::done
    R --> R2["Repeated delay / caught"]:::done
    R --> R3["Early continuation / caught"]:::done
    R --> R4["Wrong flag map / caught"]:::done
    D --> N["Native ownership"]:::done
    N --> N1["Missing return import / caught"]:::done
    N --> N2["Owner retained on panic / caught"]:::done
    N --> N3["Missing panic publication / caught"]:::done
    D --> I["Inventory restoration"]:::done
    I --> I1["Health equality boundary / caught"]:::done
    I --> I2["Missile reserve limit / caught"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

### Shared visual effects / shared owner adopted

[Full build + 27 mutations / PASS](https://github.com/JimLundin/sm-decomp/actions/runs/40021042750)

```mermaid
flowchart TB
    subgraph owner["Shared owner / game/"]
        S["session.rs"]:::done
        VS["visual_state.rs"]:::done
        EI["external_inputs.rs"]:::done
        VH["visual_host.rs"]:::done
        SE["session_encoding.rs"]:::done
        SR["session_runtime.rs"]:::done
        VR["visual_runtime.rs"]:::done
    end
    subgraph palette["palette_effects/"]
        PE["execution.rs"]:::done
        PM["mod.rs"]:::done
    end
    subgraph animation["animated_tiles/"]
        AE["execution.rs"]:::done
        AM["mod.rs"]:::done
        AN["native.rs"]:::done
    end
    subgraph player["Samus / beam consumers"]
        SP["game/samus_palette.rs"]:::done
        LS["game/liquid_surface.rs"]:::done
        B90["sm_90.rs"]:::done
        B91["sm_91.rs"]:::done
    end
    subgraph callers["Other shared-state consumers / game/"]
        EP["enemy_projectiles/pre_actions.rs"]:::done
        EG["enemy_projectiles/graphics_runtime.rs"]:::done
        PF["player_projectiles/feedback.rs"]:::done
        PP["player_projectiles/mod.rs"]:::done
    end
    subgraph compatibility["Compatibility boundary / src/"]
        RAM["ram_map.rs"]:::done
        RTL["rtl.rs / pointer writer relocated"]:::done
    end
    S --> VS
    S --> EI
    S --> SE --> SR --> VR
    VH --> VR
    PE --> VH
    AE --> VH
    SP --> VH
    B90 --> SP
    B91 --> SP
    EP --> PE
    AE --> EP
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef changed stroke:#a21caf,stroke-width:5px
```

```mermaid
flowchart TB
    REVIEW["Production + caller reviews<br/>PASS"]:::done --> BUILD["Complete candidate build<br/>PASS"]:::done
    UNION["51,053 disjoint WRAM bytes<br/>zero duplicate ownership / PASS"]:::done --> BUILD
    BUILD --> PAL["Palette comparisons<br/>512 PASS"]:::done
    BUILD --> ANI["Animation comparisons<br/>438 PASS"]:::done
    BUILD --> SAM["Defined Samus comparisons<br/>224 PASS"]:::done
    BUILD --> CPU["Original-CPU oracle comparisons<br/>33 PASS"]:::done
    BUILD --> GUARD["Runtime guards<br/>3 PASS"]:::done
    BUILD --> ENEMY["Enemy regression<br/>1,443 cases + 87 checks PASS"]:::done
    BUILD --> ROOM["Room comparisons<br/>1,038 PASS"]:::done
    BUILD --> PLAYER["Player regression<br/>106 groups + 56 comparisons PASS"]:::done
    BUILD --> REPLAY["Replay / 38 required probes<br/>28,314 frames / 0 failures"]:::done
    BUILD --> HOST["Hosted full build<br/>PASS"]:::done
    HOST --> DLL["Hosted DLL mutations<br/>9 / 9 caught"]:::done
    BUILD --> DETACHED["Shared detached harness<br/>execution PASS"]:::done
    DETACHED --> OLD["7 existing detached mutants<br/>7 / 7 caught"]:::done
    DETACHED --> NEW["11 new visual mutants<br/>11 / 11 caught"]:::done
    BUILD --> FMT["Root + hosted formatting<br/>PASS"]:::done
    RETIRE["7 old production files retired<br/>adopted / 666 gross lines"]:::done --> ADOPT
    PAL --> ADOPT
    ANI --> ADOPT
    SAM --> ADOPT
    CPU --> ADOPT
    GUARD --> ADOPT
    ENEMY --> ADOPT
    ROOM --> ADOPT
    PLAYER --> ADOPT
    REPLAY --> ADOPT
    DLL --> ADOPT
    OLD --> ADOPT
    NEW --> ADOPT
    FMT --> ADOPT["Shared visual owner adopted"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

```mermaid
flowchart TB
    subgraph palette_mutants["Palette mutations / compiling faults caught"]
        M1["omit_fractional_heat / caught"]:::done
        M2["cache_heat_equipment_before_sound / caught"]:::done
        M3["discard_palette_cross_owner_alias / caught"]:::done
        M6["discard_shared_palette_color / caught"]:::done
        M7["discard_eye_palette_producer / caught"]:::done
    end
    subgraph upload_mutants["Upload mutations / compiling faults caught"]
        M4["acknowledge_live_upload_selection / caught"]:::done
        M5["acknowledge_failed_upload / caught"]:::done
    end
    subgraph samus_mutants["Samus mutations / compiling faults caught"]
        M8["replace_misc_carry_with_signed_branch / caught"]:::done
        M9["omit_selector2_final_misc / caught"]:::done
        M10["replace_charge_carry_with_signed_branch / caught"]:::done
        M11["replace_grapple_carry_with_signed_branch / caught"]:::done
    end
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```
[Hosted validation](https://github.com/JimLundin/sm-decomp/actions/runs/40021042750)


<details>
<summary>Room mechanisms / shared typed core adopted</summary>

```mermaid
flowchart TB
    CONTRACT["Shared contract v2<br/>review + compile PASS"]:::done
    subgraph shared["Shared ownership / lane A"]
        S["session.rs"]:::done --> CODEC["Aggregate codec<br/>source review PASS"]:::done
        S --> PLAYER["player_projectiles/<br/>same shared inventory"]:::done
        CODEC --> RT["session_runtime.rs"]:::done
        RT --> HOST["host.rs / runtime.rs"]:::done
    end
    subgraph creation["Creation / rules / lane B"]
        CREATE["creation.rs"]:::done --> SETUP["setup.rs"]:::done
        SETUP --> PRE["pre.rs"]:::done
        SETUP --> RULES["rules/ / 22 modules"]:::done
    end
    subgraph execution["Messages / pickups / lane C"]
        SCHED["scheduler.rs"]:::done --> CMD["commands.rs"]:::done
        CMD --> PICKUP["Pickups / shared Inventory"]:::done
    end
    subgraph drawing["Drawing / content / lane D"]
        DRAW["presentation.rs"]:::done --> GRAPHICS["graphics.rs"]:::done
        CONTENT["content.rs"]:::done --> DRAW
        CONTENT --> CR["content_runtime.rs"]:::done
        GRAPHICS --> GR["graphics_runtime.rs"]:::done
    end
    CONTRACT --> S
    CONTRACT --> CREATE
    CONTRACT --> SCHED
    CONTRACT --> CONTENT
    HOST --> CALLERS["Actual caller migration"]:::done
    RULES --> CALLERS
    PICKUP --> CALLERS
    GR --> CALLERS
    CALLERS --> DELETE["10 superseded files retired<br/>adopted"]:::done
    BASE["1,037 reference outcomes"]:::done --> COMPARE["Candidate comparisons<br/>1,037 / 1,037 PASS"]:::done
    RESUME["Case 1020: real message resume<br/>pre + timer preservation PASS"]:::done --> COMPARE
    DELETE --> BUILD["Integrated candidate compile<br/>build 4 PASS"]:::done
    BUILD --> COMPARE
    GAP["Cases 1021-1032<br/>adopted / PASS"]:::done --> COMPARE
    SHAREDTEST["Pickup / refill / hold<br/>actual player consumers"]:::done --> COMPARE
    FLAGS["commands.rs<br/>four flag-map branches fixed"]:::done --> COMPARE
    COMPARE --> REG["106 player groups / 56 comparisons<br/>2 runtime guards / PASS"]:::done
    COMPARE --> HC["Composed host compile<br/>PASS"]:::done
    HC --> HE["Composed host execution<br/>CI PASS"]:::done
    HE --> MUT["8 targeted mutations<br/>KILLED / CI PASS"]:::done
    HE --> CM["4 composed mutants<br/>KILLED / CI PASS"]:::done
    CM --> MUT
    BUILD --> FRONT["Full frontend build<br/>CI PASS / local ASR block"]:::done
    REG --> MOVIE["28,314 frames / 19 probes<br/>0 failures / PASS"]:::done
    MUT --> MOVIE
    MOVIE --> PERF["Runtime cost<br/>94.81s vs 88.18s / measured"]:::done
    PERF --> ADOPT
    MOVIE --> ADOPT["Shared typed core adopted"]:::done
    ADOPT --> GET["mod.rs / five getters<br/>aggregate copies removed"]:::done
    GET --> GTEST["room_mechanism_getters.rs<br/>54 queries + 16 nested cases PASS"]:::done
    GTEST --> GMOV["28,314 frames / 0 failures<br/>no measured speedup"]:::done
    GTEST --> GCI["New frozen getter baseline<br/>CI PASS"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

</details>

### New mechanism verification

```mermaid
flowchart LR
    C21["1021 / callback continuation"]:::done
    C22["1022 / loaded vs native"]:::done
    C23["1023 / native rebind"]:::done
    C24["1024 / overlapping tile reads"]:::done
    C25["1025 / custom draw A"]:::done
    C26["1026 / custom draw B"]:::done
    C27["1027 / item draw A"]:::done
    C28["1028 / item draw B"]:::done
    C29["1029 / wide ROM cursor"]:::done
    C30["1030 / four transfers"]:::done
    C31["1031 / vertical transfers"]:::done
    C32["1032 / live RAM stream"]:::done
    C20["1020 / resumed pre + timer"]:::done --> MP["Mutant: repeat pre<br/>KILLED / CI PASS"]:::done
    C20 --> MD["Mutant: repeat delay<br/>KILLED / CI PASS"]:::done
    C21 --> MC["Mutant: publish continuation early<br/>KILLED / CI PASS"]:::done
    C33["1033 / room flag set<br/>reference + candidate PASS"]:::done --> MF["Mutant: wrong flag map<br/>KILLED / CI PASS"]:::done
    C34["1034 / room flag clear<br/>reference + candidate PASS"]:::done
    C35["1035 / item flag set<br/>reference + candidate PASS"]:::done
    C36["1036 / item flag clear<br/>reference + candidate PASS"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

### Player projectiles / adopted typed core

```mermaid
flowchart TB
    subgraph group0["Owned state"]
        direction LR
        p0_0["state.rs"]:::done
        p0_1["host.rs"]:::done
        p0_2["access.rs"]:::done
        p0_3["encoding.rs"]:::done
        p0_4["runtime.rs"]:::done
        p0_5["content.rs"]:::done
        p0_6["mod.rs"]:::done
    end
    subgraph group1["Creation"]
        direction LR
        p1_0["weapons.rs"]:::done
        p1_1["preparation.rs"]:::done
        p1_2["placement.rs"]:::done
        p1_3["missiles.rs"]:::done
        p1_4["bomb_creation.rs"]:::done
    end
    subgraph group2["Simulation"]
        direction LR
        p2_0["motion.rs"]:::done
        p2_1["collision.rs"]:::done
        p2_2["lifecycle.rs"]:::done
        p2_3["bombs.rs"]:::done
        p2_4["bomb_terrain.rs"]:::done
        p2_5["specials.rs"]:::done
    end
    subgraph group3["Presentation"]
        direction LR
        p3_0["animation.rs"]:::done
        p3_1["presentation.rs"]:::done
        p3_2["feedback.rs"]:::done
    end

    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

```mermaid
flowchart LR
    REF["106 groups<br/>56 comparisons"]:::done --> LOCAL["Local checks PASS"]:::done
    FMT["cargo fmt<br/>PASS"]:::done --> LOCAL
    BUILD["Release + frontend<br/>local PASS"]:::done --> LOCAL
    MOVIE["11,549 frames<br/>12 probes / 0 failures"]:::done --> LOCAL
    COMPILE["Three-core host<br/>compile PASS"]:::done --> HOST["Independent execution<br/>CI PASS"]:::done
    LOCAL --> ADOPT["Adopted on main"]:::done
    HOST --> ADOPT
    RETIRE["15 old layers retired<br/>adopted"]:::done --> ADOPT
    FIX["player_checks.rs<br/>boundary fixture corrected"]:::done --> HOST
    HOST --> MUT1["Mutation: ignore radius<br/>KILLED / CI PASS"]:::done
    HOST --> MUT2["Mutation: skip retirement<br/>KILLED / CI PASS"]:::done
    MUT1 --> ADOPT
    MUT2 --> ADOPT
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

### Destination

```mermaid
flowchart TB
    GAME["sm/src/game/"]:::active
    GAME --> PP["player_projectiles/<br/>interval owner adopted"]:::done
    GAME --> RM["room_mechanisms/<br/>interval owner adopted"]:::done
    GAME --> EP["enemy_projectiles/<br/>interval owner adopted"]:::done
    GAME --> PAL["palette_effects/<br/>typed core adopted"]:::done
    GAME --> AT["animated_tiles/<br/>typed core adopted"]:::done
    PP --> PPT["Player typed core<br/>adopted / checks PASS"]:::done
    RM --> RMT["Mechanism typed core<br/>shared player / inventory"]:::done
    EP --> EPT["Enemy projectile typed core"]:::pending
    GAME --> SAMUS["Samus"]:::pending
    GAME --> ENEMIES["Enemies / bosses"]:::pending
    GAME --> ROOMS["room_entry/<br/>interval owner adopted"]:::done
    PAL --> COMPOSE["Composed animation + palette"]:::done
    AT --> COMPOSE
    PPT --> OWN["Persistent Session"]:::pending
    RMT --> OWN
    EPT --> OWN
    COMPOSE --> OWN
    SAMUS --> OWN
    ENEMIES --> OWN
    ROOMS --> OWN
    OWN --> RESET["Reset / load"]:::pending
    OWN --> REPLAY["Snapshot / replay"]:::pending
    OWN --> WRITERS["External writers retired"]:::pending
    RESET --> OUTPUT["Scene / sound outputs"]:::pending
    REPLAY --> OUTPUT
    WRITERS --> OUTPUT
    OUTPUT --> GPU["GPU renderer"]:::pending
    OUTPUT --> AUDIO["Audio engine"]:::pending
    GPU --> SHIP["Independent game"]:::pending
    AUDIO --> SHIP
    SHIP --> CREDITS["Boot to credits verified"]:::pending
    SHIP --> SAFE["Zero unsafe / manual memory"]:::pending
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

<details>
<summary>Adopted modules / expand folder maps</summary>

Green below means adopted responsibility; runtime and legacy edges still exist.

#### palette_effects/

```mermaid
flowchart LR
    ROOT["palette_effects/"]:::done
    ROOT --> f0["content.rs"]:::done
    ROOT --> f1["diagnostics.rs"]:::done
    ROOT --> f2["execution.rs"]:::done
    ROOT --> f3["mod.rs"]:::done
    ROOT --> f4["runtime.rs"]:::done
    ROOT --> f5["state.rs"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

#### animated_tiles/

```mermaid
flowchart LR
    ROOT["animated_tiles/"]:::done
    ROOT --> f0["content.rs"]:::done
    ROOT --> f1["execution.rs"]:::done
    ROOT --> f2["mod.rs"]:::done
    ROOT --> f3["native.rs"]:::done
    ROOT --> f4["runtime.rs"]:::done
    ROOT --> f5["state.rs"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

#### enemy_projectiles/

```mermaid
flowchart LR
    ROOT["enemy_projectiles/"]:::done
    ROOT --> f0["attachments_spore_maw.rs"]:::done
    ROOT --> f1["attachments_spore_maw_facade.rs"]:::done
    ROOT --> f2["attachments_waffle_botwoon.rs"]:::done
    ROOT --> f3["attachments_waffle_botwoon_edges.rs"]:::done
    ROOT --> f4["collision.rs"]:::done
    ROOT --> f5["commands.rs"]:::done
    ROOT --> f6["compat.rs"]:::done
    ROOT --> f7["contact.rs"]:::done
    ROOT --> f8["content.rs"]:::done
    ROOT --> f9["drops.rs"]:::done
    ROOT --> f10["edges.rs"]:::done
    ROOT --> f11["legacy.rs"]:::done
    ROOT --> f12["lifecycle.rs"]:::done
    ROOT --> f13["mod.rs"]:::done
    ROOT --> f14["pre_actions.rs"]:::done
    ROOT --> f15["pre_helpers.rs"]:::done
    ROOT --> f16["presentation.rs"]:::done
    ROOT --> f17["script.rs"]:::done
    ROOT --> f18["setup.rs"]:::done
    ROOT --> f19["setup_helpers.rs"]:::done
    ROOT --> f20["shared.rs"]:::done
    ROOT --> f21["shared_pre.rs"]:::done
    ROOT --> f22["state.rs"]:::done
    ROOT --> f23["tables.rs"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

#### room_mechanisms/

```mermaid
flowchart LR
    ROOT["room_mechanisms/"]:::done
    ROOT --> f0["commands.rs"]:::done
    ROOT --> f1["creation.rs"]:::done
    ROOT --> f2["effects.rs"]:::done
    ROOT --> f3["fields.rs"]:::done
    ROOT --> f4["fields_runtime.rs"]:::done
    ROOT --> f5["graphics.rs"]:::done
    ROOT --> f6["mod.rs"]:::done
    ROOT --> f7["native_content.rs"]:::done
    ROOT --> f8["port.rs"]:::done
    ROOT --> f9["pre.rs"]:::done
    ROOT --> f10["presentation.rs"]:::done
    ROOT --> f11["rules/animal_rescue.rs"]:::done
    ROOT --> f12["rules/booster_escape.rs"]:::done
    ROOT --> f13["rules/cannon_setup.rs"]:::done
    ROOT --> f14["rules/contact_physics.rs"]:::done
    ROOT --> f15["rules/crumbling_wall.rs"]:::done
    ROOT --> f16["rules/doors.rs"]:::done
    ROOT --> f17["rules/eye_door.rs"]:::done
    ROOT --> f18["rules/gates.rs"]:::done
    ROOT --> f19["rules/hands.rs"]:::done
    ROOT --> f20["rules/item_setup.rs"]:::done
    ROOT --> f21["rules/metroid_quota.rs"]:::done
    ROOT --> f22["rules/mother_brain_glass.rs"]:::done
    ROOT --> f23["rules/noob_tube.rs"]:::done
    ROOT --> f24["rules/plants.rs"]:::done
    ROOT --> f25["rules/scroll_setup.rs"]:::done
    ROOT --> f26["rules/shaktool.rs"]:::done
    ROOT --> f27["rules/sound_delay.rs"]:::done
    ROOT --> f28["rules/stations.rs"]:::done
    ROOT --> f29["rules/terrain_setup.rs"]:::done
    ROOT --> f30["rules/torizo_statue.rs"]:::done
    ROOT --> f31["rules/tourian_wall.rs"]:::done
    ROOT --> f32["rules/treadmill.rs"]:::done
    ROOT --> f33["scheduler.rs"]:::done
    ROOT --> f34["setup.rs"]:::done
    ROOT --> f35["state.rs"]:::done
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

</details>

</details>

[Plan](docs/PLAN.md) / [Checks and accounting](docs/HANDOFF.md) / [Target architecture](docs/target-architecture/README.md)
