# Engine migration

```mermaid
flowchart LR
    NOW["NOW · Build replay coverage<br/>Areas · bosses · save/load · ending"]:::active
    NEXT["NEXT · Consolidate player logic<br/>Retire RAM glue · shrink production"]:::pending
    GAIN["ENABLES<br/>Safe, owned game state"]:::pending
    TARGET["TARGET · Map Rando<br/>Rooms · sprites · opt-in rules"]:::pending
    HELD["HELD · Input draft<br/>Checks pass · not adopted"]:::draft
    NOW --> NEXT --> GAIN --> TARGET
    HELD -.-> NEXT
    classDef active fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef draft fill:#bfdbfe,stroke:#2563eb,color:#1e3a8a
    classDef pending fill:#f1f5f9,stroke:#94a3b8,color:#475569,stroke-dasharray:5 5
```

| Measure | Count |
| --- | ---: |
| Production files* | 117911 |
| Verification files | 35653 |
| Total Rust | 153564 |
| Lines containing `unsafe` | 5042 |
| Warnings (no-default library) | 482 |
| Checkpoint segments passing | not measured |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

[Current plan](docs/PLAN.md#work-order) · [Codebase map](https://jimlundin.github.io/sm-decomp/) · [Last verified milestone](https://github.com/JimLundin/sm-decomp/commit/300eaf15)
