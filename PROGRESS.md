# Engine migration

```mermaid
flowchart TB
    NOW["NOW · Power Bomb route → Crocomire<br/>Gameplay: 23 / 44 · Verifier tests: 20 / 20"]:::active
    DELETE["NOW · Consolidate enemies + projectiles<br/>Last two cleanups: −44 lines · target below 110k"]:::active
    OWN["NEXT · Finish safe game state<br/>Remove manual memory management"]:::pending
    PROVE["THEN · Own rendering + audio<br/>Play boot to credits without SNES runtime"]:::pending
    TARGET["TARGET · Modular engine · below 100k lines<br/>Map Rando: custom sprites, rooms/doors, opt-in QoL"]:::pending
    NOW -->|Passing checks protect each deletion| DELETE
    DELETE -->|One owner for each game responsibility| OWN
    NOW -->|Complete all 44 required checks| PROVE
    OWN --> PROVE --> TARGET
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
```

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 117869 |
| Verification Rust lines | 37482 |
| Total Rust | 155351 |
| Lines containing `unsafe` | 5042 |
| Warnings (no-default library) | 482 |
| Required checkpoint checks passing | 23/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

[Current plan](docs/PLAN.md#work-order) · [Checkpoint recipes](movies/segments.toml) · [Codebase map](https://jimlundin.github.io/sm-decomp/)
