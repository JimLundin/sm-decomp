# Engine migration

2026-10-23 16:47 UTC

🟩 Committed | 🟦 Checked, uncommitted | 🟧 In progress | ⬜ Remaining | **Purple border: changed since previous report**

```mermaid
flowchart TB
    ROOT["sm/src/game / owned Session"]:::active
    ROOT --> PP
    ROOT --> RM
    ROOT --> PAL
    ROOT --> ANIM
    ROOT --> ENEMY
    ROOT --> REST
    PP["player_projectiles/<br/>interval owner + callers"]:::done
    RM["room_mechanisms/<br/>interval owner + callers"]:::done
    PAL["palette_effects/<br/>independent typed core"]:::done
    ANIM["animated_tiles/<br/>owner + upload contract"]:::done
    ENEMY["enemy_projectiles/<br/>isolated replacement"]:::active
    REST["player / enemies / rooms<br/>remaining typed-core migrations"]:::pending
    PP --> PERSIST
    RM --> PERSIST
    PAL --> COMPOSE
    ANIM --> COMPOSE
    ENEMY --> PERSIST
    REST --> PERSIST
    COMPOSE["Session composition<br/>animation + palette"]:::active
    COMPOSE --> PERSIST
    PERSIST["Persistent Session<br/>reset / load / replay / external writers"]:::pending
    PERSIST --> SCENE["Scene / sound outputs"]:::pending
    SCENE --> GPU["Our renderer"]:::pending
    SCENE --> AUDIO["Our audio engine"]:::pending
    GPU --> SHIP["Independent shipping game"]:::pending
    AUDIO --> SHIP
    SHIP --> END["Boot / credits<br/>zero unsafe / manual memory"]:::pending
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
    class COMPOSE changed
```

```mermaid
flowchart LR
    subgraph palette["palette_effects/ / committed"]
        direction TB
        PS["state.rs<br/>slots + colors"]:::done --> PE["execution.rs<br/>typed interpreter"]:::done
        PC["content.rs<br/>captured reads"]:::done --> PE
        PE --> PR["runtime.rs<br/>legacy ownership edge"]:::done
        PR --> PCL["13 caller files<br/>sm_8d retired"]:::done
        PE --> PD["verification/palette_detached.rs<br/>independent host: PASS"]:::done
        PR --> PG["runtime guard checks<br/>aliases + unwind: PASS"]:::done
        PCL --> PV["512 comparisons<br/>272 opening frames: PASS"]:::done
    end
    subgraph animation["animated_tiles/ / committed"]
        direction TB
        AS["state.rs<br/>6 slots + upload / ack"]:::done --> AE["execution.rs<br/>24 commands"]:::done
        AC["compatibility.rs<br/>byte layout + aliases"]:::done --> AE
        AE --> AN["native.rs<br/>ordered graphics upload"]:::done
        AN --> ACL["21 calls / 8 files<br/>sm_87 + old NMI retired"]:::done
        ACL --> AB["release + frontend + format<br/>PASS"]:::done
        AB --> AV["438 frozen comparisons<br/>PASS"]:::done
        AV --> AM["movie: create / upload / ack<br/>20,947 frames / 0 failures"]:::done
        AM --> LAND["Adoption committed"]:::done
    end
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px

```

```mermaid
flowchart LR
    subgraph enemy["enemy_projectiles/ / isolated work"]
        direction TB
        ES["state + compatibility<br/>801 RAM bytes / 16 native"]:::checked --> EL["allocation + scheduler"]:::checked
        EL --> SETUP["setup callbacks"]:::checked
        EL --> PRE["pre-action callbacks"]:::checked
        EL --> CMD["commands + movement"]:::checked
        SETUP --> EC["library source check<br/>PASS"]:::checked
        PRE --> EC
        CMD --> EC
        ATT["Spore / Maw / Waffle / Botwoon<br/>actual producer migration"]:::active --> CALL["caller retirement draft"]:::active
        DROP["drops + contact + reset"]:::active --> CALL
        CALL --> FULL["Full adoption checks"]:::pending
        EC --> CORE["594 old / new comparisons<br/>PASS"]:::checked
        OLD["354 old references<br/>captured"]:::done --> CORE
        MORE["setup / pre / attachment / contact<br/>expanded cases being prepared"]:::active --> FULL
        CORE --> FULL
        FULL --> ECOMMIT["Commit adoption"]:::pending
    end
    subgraph composition["next: independent animation + palette"]
        direction TB
        HOST["animation execution<br/>no Port / Game dependency"]:::checked --> ONE["One Session<br/>one palette color array"]:::checked
        ONE --> BUILD["standalone + production builds<br/>PASS"]:::checked
        BUILD --> GUARD["actual runtime guards<br/>aliases + unwind: PASS"]:::checked
        GUARD --> REF["438 old / new comparisons<br/>PASS"]:::checked
        REF --> MOVIE["animation / upload replay<br/>20,947 frames / 0 failures"]:::checked
        BUILD --> REPLAY["tick / reset / snapshot / replay<br/>independent execution pending"]:::active
        REPLAY --> CI["Independent host execution<br/>GitHub Windows: QUEUED"]:::pending
        MOVIE --> CI
        CI --> CCOMMIT["Commit composed core"]:::pending
    end
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
    class CORE,HOST,ONE,BUILD,GUARD,REF,MOVIE,REPLAY,CI changed
```

[Plan](docs/PLAN.md) / [Checks and accounting](docs/HANDOFF.md) / [Target architecture](docs/target-architecture/README.md)
