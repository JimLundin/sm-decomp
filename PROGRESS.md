# Engine migration

```mermaid
flowchart TB
    NOW["NOW · Power Bomb route → Crocomire<br/>Gameplay checks: 23 / 44 passing"]:::active
    NEXT["NEXT · Remaining areas, bosses + ending<br/>Complete all 44 required checks"]:::pending
    DELETE["THEN · Merge duplicate gameplay logic<br/>Production: 117,913 → below 110k lines"]:::pending
    OWN["THEN · Finish safe game state<br/>Remove manual memory management"]:::pending
    TARGET["TARGET · Modular engine · below 100k lines<br/>Map Rando: custom sprites, rooms/doors, opt-in QoL"]:::pending
    NOW --> NEXT
    NEXT -->|Catch regressions while deleting code| DELETE
    DELETE -->|One owner for each game responsibility| OWN
    OWN -->|Own video/audio; remove SNES runtime dependency| TARGET
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
```

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 117913 |
| Verification Rust lines | 37429 |
| Total Rust | 155342 |
| Lines containing `unsafe` | 5042 |
| Warnings (no-default library) | 482 |
| Required checkpoint checks passing | 23/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

[Current plan](docs/PLAN.md#work-order) · [Checkpoint recipes](movies/segments.toml) · [Codebase map](https://jimlundin.github.io/sm-decomp/)
