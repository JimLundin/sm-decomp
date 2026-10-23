# Engine migration

2026-10-23 16:52 UTC

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
    ANIM["animated_tiles/<br/>independent typed core"]:::done
    ENEMY["enemy_projectiles/<br/>isolated replacement"]:::active
    REST["player / enemies / rooms<br/>remaining typed-core migrations"]:::pending
    PP --> PERSIST
    RM --> PERSIST
    PAL --> COMPOSE
    ANIM --> COMPOSE
    ENEMY --> PERSIST
    REST --> PERSIST
    COMPOSE["Session composition<br/>animation + palette"]:::done
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
    class ANIM,COMPOSE changed
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
        AS["state.rs<br/>6 slots + upload / ack"]:::done --> AE["execution.rs<br/>24 commands / no Game"]:::done
        AC["runtime.rs<br/>byte layout + ownership edge"]:::done --> AE
        CONTENT["content.rs<br/>captured records"]:::done --> AE
        AE --> AN["native.rs<br/>ordered graphics upload"]:::done
        AN --> ACL["21 calls / 8 files<br/>sm_87 + old NMI retired"]:::done
        ACL --> AB["release + frontend + format<br/>PASS"]:::done
        AB --> AV["438 frozen comparisons<br/>PASS"]:::done
        AV --> AM["movie: create / upload / ack<br/>20,947 frames / 0 failures"]:::done
        AM --> LAND["Adoption committed"]:::done
    end
    class AE,AC,CONTENT changed
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
        MORE["1,362 old references<br/>CAPTURING"]:::active --> FULL
        ROM["Native ROM owner<br/>remove g_rom field"]:::active --> RECORD["Captured RomImage records<br/>remove projectile raw reads"]:::active
        RECORD --> FULL
        CORE --> FULL
        FULL --> ECOMMIT["Commit adoption"]:::pending
    end
    subgraph composition["independent animation + palette - committed"]
        direction TB
        HOST["animation execution<br/>no Port / Game dependency"]:::done --> ONE["One Session<br/>one palette color array"]:::done
        ONE --> BUILD["standalone + production builds<br/>PASS"]:::done
        BUILD --> GUARD["actual runtime guards<br/>aliases + unwind: PASS"]:::done
        GUARD --> REF["438 old / new comparisons<br/>PASS"]:::done
        REF --> MOVIE["animation / upload replay<br/>20,947 frames / 0 failures"]:::done
        BUILD --> REPLAY["tick / reset / snapshot / replay<br/>independent execution: PASS"]:::done
        REPLAY --> CI["Independent host execution<br/>GitHub Windows: PASS"]:::done
        MOVIE --> CI
        CI --> CCOMMIT["Composed core adopted"]:::done
    end
    classDef done fill:#bbf7d0,stroke:#15803d,color:#14532d
    classDef checked fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
    classDef changed stroke:#a21caf,stroke-width:5px
    class HOST,ONE,BUILD,GUARD,REF,MOVIE,REPLAY,CI,CCOMMIT,MORE,ROM,RECORD changed
```

[Plan](docs/PLAN.md) / [Checks and accounting](docs/HANDOFF.md) / [Target architecture](docs/target-architecture/README.md)
