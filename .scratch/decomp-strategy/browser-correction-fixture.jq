.updates = [
  {id:"browser-original", scope:"Original UI fixture", agent:"browser-check", stage:"translated", routines:["NextRandom"], source_revision:.source.revision, inventory_sha256:.source.inventory_sha256, evidence:[], limitations:["Browser rendering fixture only; grants no gameplay progress."], updated_at:1},
  {id:"browser-correction", scope:"Corrected UI fixture", agent:"browser-check", stage:"translated", evidence_context:"current-checkout", supersedes:["browser-original"], routines:["Multiply16x16"], source_revision:.source.revision, inventory_sha256:.source.inventory_sha256, evidence:[], limitations:["Browser rendering fixture only; grants no gameplay progress."], updated_at:2}
]
| .stale_update_ids = ["browser-correction"]
| .superseded_update_ids = ["browser-original"]
| .notices = ["Browser fixture: both original and correction are excluded; original must never re-enter counts."]
