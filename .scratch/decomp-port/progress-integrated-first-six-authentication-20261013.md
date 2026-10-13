# Current merged first six Offline family authentication

Status: passed after coordinator cleanup and worktree retirement.
Owner: `/root/progress_finish`, delegated by `/root`.

All six commands used `target/release/sm-pipeline family-authenticate --checkout
/home/jim/sm-decomp --verdict VERDICT.toml` with at most two concurrent jobs.
Every process exited zero and returned `status = "offline-verified"`; stderr
files are empty. The exact release binary SHA-256 is
`7fd44cf8a43d1b9896891fc029a9ebdebb74ad448846c725ee3bc97cece4d762`.
Frozen main and batch contracts remain unchanged. Full reports and stderr are
`progress-integrated-FAMILY-authentication.toml` / `.stderr`; their checked hashes
are `progress-integrated-authentication-source-20261013.sha256`.

| Offline family | Actual-C fixture pairs | Declared routines |
| --- | ---: | ---: |
| observed-projectile-initializers | 5,409 | 7 |
| observed-projectile-frame-boundaries | 577 | 2 |
| observed-projectile-ownership | 1,840 | 4 |
| enemy-projectile-rendering | 1,448 | 2 |
| enemy-projectile-detect-samus | 2,310 | 1 |
| enemy-projectile-detect-projectiles | 2,040 | 1 |

These current-source domains authenticate canonical WRAM/raw return comparison
after cleanup. They make no live exercise or original-ROM parity claim. Verdict
paths are recorded exactly in the returned reports and the shared batch record
`out/recovery-session/integrated/batch.toml`.

The coordinator completed both scoped cleanup passes before these commands:
`recovery-integrated-final-cleanup.toml` reports zero disposable bytes and
`recovery-integrated-final-dedup.toml` reports zero additional bytes across 63,551
snapshots. The batch automatic deduplication had already reclaimed 4,522,901,504
bytes across 64,952 snapshots. Stale implementer worktree registrations were
retired in `recovery-worktree-retirement-20261013.toml`, preserving exact source
snapshots and sealed evidence. Authentication creates no new fixture run.
