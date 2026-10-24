# Engine migration

```mermaid
flowchart TB
    DONE["JUST VERIFIED · Ice Beam<br/>21 → 22 of 44 checks · 0 failing frames"]:::done
    NOW["NOW · Route to Crocomire<br/>Capture and verify boss checkpoint"]:::active
    NEXT["NEXT · Finish 44/44 checks<br/>All areas, bosses, pickups and credits"]:::pending
    DELETE["THEN · Consolidate owned gameplay<br/>Delete duplicate logic · production below 110k"]:::pending
    TARGET["TARGET · Safe, independent engine · below 100k<br/>Map Rando: sprites, rooms/doors, opt-in QoL"]:::pending
    DONE --> NOW --> NEXT
    NEXT -->|Protect behavior during deletion| DELETE
    DELETE -->|Finish safe ownership + own video/audio| TARGET
    classDef done fill:#dcfce7,stroke:#15803d,color:#14532d
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
| Required checkpoint checks passing | 22/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

[Current plan](docs/PLAN.md#work-order) · [Checkpoint recipes](movies/segments.toml) · [Codebase map](https://jimlundin.github.io/sm-decomp/)
