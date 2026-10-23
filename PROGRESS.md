# Engine migration

Green: adopted scope | Blue: draft | Orange: active | Grey: remaining | Red: failed
**Purple outline: changed since previous report.** Folder branches show containment; check arrows show prerequisites.

[Shared Session adopted: 74948127](https://github.com/JimLundin/sm-decomp/commit/74948127) / [Build + eight mutations: PASS](https://github.com/JimLundin/sm-decomp/actions/runs/40005712881)

[Four room mutations / CI PASS](https://github.com/JimLundin/sm-decomp/actions/runs/40005395335) · [Shared Session candidate CI](https://github.com/JimLundin/sm-decomp/actions/runs/40005712881)

### Enemy projectiles / next shared owner

```mermaid
flowchart TB
    CONTRACT["Complete contract<br/>independent review PASS"]:::checked
    UNION["49,819 WRAM + 16 native bytes<br/>no duplicate ownership"]:::checked
    CONTRACT --> API["Concrete Rust interfaces<br/>metadata compile PASS"]:::checked
    API --> REVIEW["Concrete interface review"]:::active
    UNION --> API
    REF["1,382 captures + 87 checks<br/>current main PASS"]:::checked
    API --> S["session.rs / session_encoding.rs"]:::pending
    S --> R["session_runtime.rs<br/>one owner / native spawn fields"]:::pending
    API --> SETUP["setup.rs / setup_helpers.rs"]:::pending
    API --> PRE["pre_actions.rs / shared_pre.rs"]:::pending
    API --> LIFE["lifecycle.rs / commands.rs"]:::pending
    API --> CONTACT["contact.rs / collision.rs"]:::pending
    API --> ATTACH["attachments_spore_maw.rs<br/>attachments_waffle_botwoon.rs"]:::pending
    API --> DROP["drops.rs / Inventory.restore"]:::pending
    DROP --> CALL["sm_90.rs / sm_a2.rs / sm_91.rs<br/>one restoration implementation"]:::pending
    R --> RETIRE["Retire compat.rs<br/>and ownership_transport.rs"]:::pending
    SETUP --> MERGE["Whole candidate integration"]:::pending
    PRE --> MERGE
    LIFE --> MERGE
    CONTACT --> MERGE
    ATTACH --> MERGE
    CALL --> MERGE
    RETIRE --> MERGE
    RESTORE["58 restoration cases<br/>review + capture PASS"]:::checked --> VERIFY["Comparisons / composed checks"]:::pending
    NATIVE["Native spawn / shared state checks<br/>design review PASS"]:::checked --> VERIFY
    MERGE --> VERIFY
    REF --> VERIFY
    VERIFY --> MUT["Targeted mutations"]:::pending
    MUT --> PLAY["Exercised replay / runtime cost"]:::pending
    PLAY --> ADOPT["Enemy shared core adopted"]:::pending
    class API,REVIEW,RESTORE,NATIVE changed
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
```

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
    class GET,GTEST,GMOV,GCI changed
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
    class C33,C34,C35,C36,MF changed
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
    GAME --> ROOMS["Rooms / transitions"]:::pending
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

[Plan](docs/PLAN.md) / [Checks and accounting](docs/HANDOFF.md) / [Target architecture](docs/target-architecture/README.md)
