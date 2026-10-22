# Report and handoff checks

Checked on 2026-10-22. This pass covers the migration clarification and the portable documentation handoff. Architecture and presentation remain revision 3. It supplies no new engine build, replay, fidelity, or performance result.

| Check | Result |
| --- | --- |
| Overview | Three sections, five packages, six gates; current Rust status and Windows migration link are present |
| Changed detail content | Dependency explanation, ownership decision, existing reduction objective, and staged independence gate agree |
| Interaction smoke checks | Execution controls, build disclosures, detail dialogs, Escape, and restored trigger focus pass |
| Local links | Relative document links resolve in the handoff; repository and both original local report copies load directly from disk |
| Responsive content | Changed overview and details fit 320, 390, and 1440 px without horizontal page/dialog overflow |
| Accessibility | Zero violations in four axe-core WCAG 2.0/2.1 A/AA audits: overview, dependencies, expanded ownership/size decisions, and mobile dependencies |
| Offline operation | No JavaScript errors or HTTP requests during the browser checks |
| Repository scope | Documentation only; whitespace and local-link checks pass; maintained Rust remains 103,313 lines |

Browser checks used Playwright and headless Chromium on Linux. The desktop overview and mobile dependency dialog were also visually inspected. Viewport emulation does not establish physical-device or Windows-browser behavior. Existing report interaction handlers and styles are unchanged; this was a focused content pass, not a rerun of every historical UI check.

The PowerShell opening command is provided for Windows workers; no new Windows game or browser execution is claimed. Current engine validation and blocked runs retain their status in the project work plan. These checks add no engine tooling or evidence infrastructure.

Portable `report.html` SHA-256:

```text
c3676a879db5685e51bd5f6f9df942ee522d3a7f5529521f213044ba3ec32f8e
```
