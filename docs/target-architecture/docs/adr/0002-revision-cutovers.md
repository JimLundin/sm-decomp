---
status: accepted
---
# Prepare content revisions before cutting over

Development reloads cross gameplay definitions, GPU resources, sound banks, and persistent IDs. Compile and validate a candidate, prepare its device resources, and publish it at a committed Tick seam only when the complete candidate is ready; an invalid candidate leaves the working revision intact. Media-only changes preserve the Session, while gameplay changes restart a named Scenario, avoiding an implicit migration of arbitrary live boss and room state.

This requires stable authored IDs, semantic change classification, temporary memory for two revisions, and retirement acknowledgments for old GPU submissions and sound voices. A future compatible-state migration must capture the current committed Checkpoint at cutover, not reuse an older one while the previous Session continues advancing.
