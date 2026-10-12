# Fixture execution investigation

Status: resolved

## Decisions so far

- User selected fixture execution from the architecture review to improve
  offline-family and live-replay iteration speed.
- Investigate persistent workers alongside fresh-process scheduling and prompt
  child completion; no persistent execution has been selected or implemented.
- User chose experiments capped at approximately 60 seconds each before one
  complete affected-family verification of a selected change.
- Independent chains may overlap; preserve predecessor and report order.
- Investigate preloaded fork-per-fixture helpers with shared loaded data.
- Failed fixtures block descendants while independent chains continue.
- Fixture terminology is recorded in root `GLOSSARY.md`.
- Facts: [01](issues/01-execution-facts.md). Decisions:
  [02](issues/02-execution-design.md).

| Child | Type | Status |
| --- | --- | --- |
| [01](issues/01-execution-facts.md) | research | resolved |
| [02](issues/02-execution-design.md) | grilling | resolved |
| [03](issues/03-bounded-prototype.md) | prototype | resolved |
| [04](issues/04-production-execution.md) | task | resolved |

- [Production implementation](implementation.md): full projectile/system and
  short system live replay authenticated; scoped cleanup complete.

- [Prototype results](results.md): faster fresh execution and preloaded fork
  execution both show savings; full 1,840-fixture equality passed. Production
  integration is a subsequent task.
