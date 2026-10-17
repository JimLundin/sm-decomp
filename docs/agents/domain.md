# Domain docs

This repository uses a single context: root `GLOSSARY.md` and `docs/adr/`.

Before exploring code, read `GLOSSARY.md` and ADRs relevant to the area. If a
root `GLOSSARY-MAP.md` is introduced later, follow it to relevant context
glossaries and ADR directories instead.

When these documents do not exist, proceed silently. Create domain documents
lazily through the domain-modeling skill when terms or decisions are resolved.

Use the glossary's terms in issues, proposals and tests. If a needed concept is
missing, reconsider the terminology or note the gap for domain modeling.

Surface conflicts with an existing ADR explicitly, naming the ADR and explaining
why its decision should be reconsidered.
