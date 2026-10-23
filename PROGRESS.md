# Engine migration

Green: adopted scope | Blue: locally checked draft | Orange: active | Grey: remaining | Red: failed
**Purple outline: changed since previous report.** Folder branches show containment; check arrows show prerequisites.

[Player draft: 026ab1a2](https://github.com/JimLundin/sm-decomp/commit/026ab1a25efcc7bd329cb631f1219dbca172fa32) / [CI](https://github.com/JimLundin/sm-decomp/actions/runs/39996910420)

### Player projectiles / typed-core migration

```mermaid
flowchart TB
    subgraph group0["Owned state"]
        direction LR
        p0_0["state.rs"]:::checked
        p0_1["host.rs"]:::checked
        p0_2["access.rs"]:::checked
        p0_3["encoding.rs"]:::checked
        p0_4["runtime.rs"]:::checked
        p0_5["content.rs"]:::checked
        p0_6["mod.rs"]:::checked
    end
    subgraph group1["Creation"]
        direction LR
        p1_0["weapons.rs"]:::checked
        p1_1["preparation.rs"]:::checked
        p1_2["placement.rs"]:::checked
        p1_3["missiles.rs"]:::checked
        p1_4["bomb_creation.rs"]:::checked
    end
    subgraph group2["Simulation"]
        direction LR
        p2_0["motion.rs"]:::checked
        p2_1["collision.rs"]:::checked
        p2_2["lifecycle.rs"]:::checked
        p2_3["bombs.rs"]:::checked
        p2_4["bomb_terrain.rs"]:::checked
        p2_5["specials.rs"]:::checked
    end
    subgraph group3["Presentation"]
        direction LR
        p3_0["animation.rs"]:::checked
        p3_1["presentation.rs"]:::checked
        p3_2["feedback.rs"]:::checked
    end
    class p0_0,p0_1,p0_2,p0_3,p0_4,p0_5,p0_6,p1_0,p1_1,p1_2,p1_3,p1_4,p2_0,p2_1,p2_2,p2_3,p2_4,p2_5,p3_0,p3_1,p3_2 changed

    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef failed fill:#fecaca,stroke:#dc2626,color:#7f1d1d
    classDef changed stroke:#a21caf,stroke-width:5px
```

```mermaid
flowchart LR
    REF["106 groups<br/>56 comparisons"]:::checked --> LOCAL["Local checks PASS"]:::checked
    FMT["cargo fmt<br/>CI PASS"]:::checked --> LOCAL
    BUILD["Release + frontend<br/>local PASS"]:::checked --> LOCAL
    MOVIE["11,549 frames<br/>12 probes / 0 failures"]:::checked --> LOCAL
    COMPILE["Three-core host<br/>compile PASS"]:::checked --> HOST["Independent execution<br/>CI FAIL"]:::failed
    LOCAL --> ADOPT["Adopt on main"]:::pending
    HOST --> ADOPT
    RETIRE["15 old layers retired<br/>in draft"]:::checked --> ADOPT
    class REF,FMT,BUILD,MOVIE,COMPILE,RETIRE,HOST changed
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
    PP --> PPT["Player typed core<br/>local PASS / CI failed"]:::checked
    RM --> RMT["Mechanism typed core"]:::pending
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

### Room mechanisms / next typed-core conversion

```mermaid
flowchart LR
    CONTRACT["State / effects contract"]:::active --> SLOTS["40 ordinary slots"]:::pending
    CONTRACT --> INVENTORY["Shared player inventory"]:::pending
    CONTRACT --> RESUME["Message suspension"]:::pending
    SLOTS --> CREATE["creation.rs / setup.rs"]:::pending
    SLOTS --> TICK["scheduler.rs / pre.rs"]:::pending
    INVENTORY --> RULES["commands.rs / rules/"]:::pending
    RESUME --> TICK
    CONTRACT --> DRAW["presentation.rs / graphics.rs"]:::pending
    CREATE --> EDGE["Runtime ownership edge"]:::pending
    TICK --> EDGE
    RULES --> EDGE
    DRAW --> EDGE
    EDGE --> CALLERS["Gameplay / door / contact / X-ray"]:::pending
    CALLERS --> DELETE["Retire Fields / Port routing"]:::pending
    DELETE --> COMPARE["Reference comparisons"]:::pending
    COMPARE --> MOVIE["Exercised movie probes"]:::pending
    MOVIE --> ADOPT["Typed core adopted"]:::pending
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
