---
name: architecture-review
description: Review a named repository engineering interface or ownership question when assigned to investigate structural friction in sm-decomp.
---

Start with the decision the owner needs and the revision under review. Use the task's permitted material. For engine interfaces, follow the [architecture references](../../../AGENTS.md#task-references) to the applicable layer, decision and crate owner.

Follow a real caller through the interface to the owner, then follow the test or production path that exercises it. Identify the knowledge the caller must supply, such as operation ordering, a repeated conversion or an invariant enforced in several places. Name the files and items where that cost is observable.

For a candidate change, describe what the caller would still need to know, where the responsibility would go and which current test or observation could distinguish the alternatives. Mentally remove the suspected wrapper and trace what each caller would inherit. Preserve boundaries that keep independent observations, production decisions and presentation separate.

Report the most consequential finding first, with its evidence, tradeoff and suggested owner. If the existing split already serves the callers and the project decision, explain why retaining it is the better choice. Distinguish a documented contract violation from an optional simplification. Root can assign a bounded change and its proof or record a disposition in the existing work item. A review assignment finishes with findings.
