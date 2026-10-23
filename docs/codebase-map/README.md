# Current-codebase atlas

[Open the hosted atlas](https://jimlundin.github.io/sm-decomp/).

Open `index.html` directly, or serve this directory as static files. Everything
needed to navigate is embedded; the page makes no network requests. Source and
architecture links open the analyzed commit on GitHub.

The first screen groups responsibilities for navigation. They are not separate
packages or crates. Open a region or folder, search a file, or choose a guided starting point.
Selecting a file shows incoming users on the left and outgoing source references
on the right. Select an edge for its first source occurrence. All neighbors remain
listed beneath the focused graph when the graph shows only its first ten per side.

The tree follows the actual repository. The graph can be dragged and zoomed;
`Fit` restores its view. `/` focuses search. Graph nodes support Enter/Space.
The URL fragment preserves selection and link/direction/tension filters.

## Refresh

From the repository root:

```powershell
python docs/codebase-map/generate_atlas.py --root . --revision HEAD
```

The generator reads immutable Git blobs from the selected revision, not modified
working files. Snapshot labels and source links therefore describe the same
revision. Uncommitted additions are intentionally absent. Commit source changes
first, regenerate, and publish the resulting snapshot in a subsequent commit.

`atlas.template.html` owns the interaction and visual design. `tensions.json`
contains the small curated architecture annotations with exact source evidence.
`index.html` is the self-contained deliverable; `atlas-data.json` is its inspectable
data companion for local inspection; it need not be published. Python's standard library is the only generation dependency.

## Evidence limits

- Every tracked file in the selected revision is an inventory node.
- Module edges resolve Rust `mod`/`#[path]` declarations. Reference edges resolve
  lexical source paths in the primary library/application contexts.
- References can be imports, type references or calls. This is not a compiler call
  graph. Macro expansion, traits, runtime callback identities, independent detached
  crate contexts, and non-Rust dependencies are incomplete.
- The overview shows the strongest cross-region source relationships. Folder
  drill-down is containment; focused file neighborhoods expose exact edges.
- Rust `unsafe`, `g_ram`, native ABI and `forbid(unsafe_code)` markers are syntax
  observations. They do not measure safety, migration completion or runtime risk.
- Curated tensions distinguish the current implementation from the target owned
  architecture. They need source review when the relevant responsibilities change.

No ROMs, captures, assets or full source-file contents are embedded in the data.

Curated tension evidence stores exact expected source lines. Generation reanchors
only a unique unchanged match; a changed or ambiguous match stops generation for
manual source review instead of publishing a misleading source link.

The Pages workflow publishes only the checked standalone HTML. Hosted Chrome
checks search, folder and dependency navigation, saved selections, keyboard and
mobile interactions, offline operation, and WCAG 2/2.1 AA rules in four views.
The initial browser/source review passed in GitHub run 40026991467; this is
navigation evidence, not gameplay verification.
