# Engine migration

```mermaid
flowchart LR
    NOW["NOW · Game checkpoints<br/>15 / 44 required checks pass"]:::active
    NEXT["NEXT · Delete duplicate logic<br/>Production below 110k lines"]:::pending
    GAIN["THEN · Owned, independent engine<br/>Safe gameplay · own video/audio"]:::pending
    TARGET["TARGET · Map Rando<br/>Custom rooms · sprites · optional QoL"]:::pending
    NOW -->|Catch regressions| NEXT
    NEXT -->|Smaller core to finish| GAIN
    GAIN -->|Configurable game| TARGET
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
```

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 117911 |
| Verification Rust lines | 36308 |
| Total Rust | 154219 |
| Lines containing `unsafe` | 5042 |
| Warnings (no-default library) | 482 |
| Required checkpoint checks passing | 15/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

[Current plan](docs/PLAN.md#work-order) · [Checkpoint recipes](movies/segments.toml) · [Codebase map](https://jimlundin.github.io/sm-decomp/)
