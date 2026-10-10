# TAS inputs

Keep local `.bk2`, `.lsmv`, and converted `.inputs` files here; they are ignored
by git. The importer supports clean-start lsnes `.lsmv`, BizHawk `.bk2`,
extracted BizHawk input logs, and hexadecimal `.inputs`.

From the repository root:

```sh
just movie movies/run.bk2 movies/run.inputs
just import-movies
just diff-fallback movies/super-metroid-any-3653.lsmv 3600
```

`just fetch-movies` fetches the three published runs recorded in `manifest.json`:
any%, low% (13%), and 100%. It verifies the actual `.lsmv` files and their ROM
hashes. The local ROM matches all three. Downloads remain gitignored.

All three have been imported with matching frame counts. Direct LSMV replay
checks the recorded ROM hash; converted inputs have no ROM metadata. Full
synchronization remains unverified. See [TAS_MOVIES.md](../docs/TAS_MOVIES.md)
for sources, replay observations, and initialization considerations.
