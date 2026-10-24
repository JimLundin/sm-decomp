# Current codebase map

[Open the interactive map](https://jimlundin.github.io/sm-decomp/).

Start with gameplay responsibilities. Tile area represents production Rust lines;
colour and the bottom strip show Owned / Partly owned / Legacy. Open a tile for
its files, then a file for incoming users and outgoing references. Select an edge
for source evidence. These responsibilities are navigation groups, not crates.

Search understands Kraid, grapple and doors; Enter opens the first match. `/`
focuses search. The file tree, breadcrumbs and Back button follow your selection.
Graph nodes support Enter/Space, drag and zoom. The URL saves selection and filters.
Tests, documentation and tools are hidden until their checkbox is enabled.
Contextual notes explain remaining dependencies and what resolves them; the
Current work link opens the live repository progress report.

## Generation

The Pages workflow generates and checks the map from HEAD on every main push.
Generated HTML and JSON are local/build outputs, not committed snapshots.
To generate a standalone offline page from the repository root:

```powershell
python docs/codebase-map/generate_atlas.py --root . --revision HEAD
```

Open the resulting `docs/codebase-map/index.html` directly. Navigation makes no
network requests; source links open the analyzed commit on GitHub. The generator
reads immutable Git blobs, excluding uncommitted work. Only Python's standard
library is required. `atlas.template.html` owns the UI; `tensions.json` supplies
curated notes with exact source evidence. Changed or ambiguous evidence omits
that note, recording its title in the diagnostic JSON for later source review.

## Reading the colours

Production accounting matches `tools/progress.sh`: Rust under `sm/src`, excluding
verification/verify directories and `verify*.rs` files. Inline verification hooks
remain production. The 110k and 100k figures are total production size targets.

- Owned: a whole-file `forbid(unsafe_code)` marker without observed unsafe, RAM,
  Game or foreign-call markers.
- Legacy: a legacy bank file, or observed unsafe without that whole-file marker.
- Partly owned: remaining production files. Mixed responsibility tiles also use
  this colour; their bottom strip shows the actual line proportions.

These source observations do not prove fidelity or completed migration. Read
About this map for extraction limits. Edges are lexical Rust module/reference
relationships, not a complete runtime call graph; callbacks, macros, traits and
non-Rust dependencies may be absent. All resolved neighbors remain listed below
the graph even when it displays only the first few. No ROM data is embedded.

The existing browser check covers navigation, offline behavior, responsive views,
keyboard interaction and axe WCAG 2/2.1 AA rules. UI checks do not verify gameplay.
