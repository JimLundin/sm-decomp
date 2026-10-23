"""Exercise the actual offline atlas and save views for visual review."""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

root = Path(__file__).resolve().parent
output = Path("atlas-browser-results")
output.mkdir(exist_ok=True)
errors, network, accessibility = [], [], []

def audit_accessibility(page, name):
    if not page.evaluate("!!window.axe"):
        page.add_script_tag(path=os.environ["AXE_PATH"])
    result = page.evaluate("async () => await axe.run(document, {runOnly: {type: 'tag', values: ['wcag2a','wcag2aa','wcag21a','wcag21aa']}})")
    accessibility.append({"view": name, "violations": result["violations"]})
with sync_playwright() as playwright:
    browser = playwright.chromium.launch(channel="chrome")
    page = browser.new_page(viewport={"width": 1440, "height": 1000})
    page.on("pageerror", lambda error: errors.append(str(error)))
    page.on("request", lambda request: network.append(request.url) if request.url.startswith(("http:", "https:")) else None)
    page.goto((root / "index.html").as_uri())
    page.wait_for_function("window.atlas && window.atlas.data.files.length > 300")
    assert page.evaluate("new Set(atlas.data.files.map(f => f.id)).size === atlas.data.files.length")
    expect(page.locator("#graph .graph-node").first).to_be_visible()
    expect(page.locator("#direction-label")).not_to_be_visible()
    page.screenshot(path=str(output / "overview-desktop.png"))
    audit_accessibility(page, "overview-desktop")

    page.keyboard.press("/")
    expect(page.locator("#search")).to_be_focused()
    page.locator("#search").fill("session_runtime.rs")
    page.get_by_test_id("search-result").first.click()
    assert page.evaluate("atlas.getState().id") == "sm/src/game/session_runtime.rs"
    expect(page.get_by_test_id("source-link")).to_have_attribute("href", __import__("re").compile(r"https://github.com/JimLundin/sm-decomp/blob/[0-9a-f]{40}/sm/src/game/session_runtime.rs"))
    assert page.locator("#graph .edge").count() > 0
    page.screenshot(path=str(output / "session-neighborhood.png"))
    audit_accessibility(page, "session-neighborhood")
    page.locator("#direction").select_option("outgoing")
    assert page.evaluate("atlas.getState().direction") == "outgoing"
    page.reload()
    page.wait_for_function("window.atlas")
    assert page.evaluate("atlas.getState().id") == "sm/src/game/session_runtime.rs"
    assert page.evaluate("atlas.getState().direction") == "outgoing"
    old_box = page.locator("#graph").get_attribute("viewBox")
    page.locator("#zoom-in").click()
    assert page.locator("#graph").get_attribute("viewBox") != old_box
    page.locator("#zoom-fit").click()
    assert page.locator("#graph").get_attribute("viewBox") == old_box
    page.locator("#graph .edge").first.click(force=True)
    expect(page.locator("#details")).to_contain_text("Why these files are connected")
    page.screenshot(path=str(output / "connection-evidence.png"))

    page.locator("#search").fill("")
    page.locator('#tree [data-scope="sm/src/game"]').click()
    page.locator('#tree [data-scope="sm/src/game/room_mechanisms"]').click()
    page.locator('#tree [data-scope="sm/src/game/room_mechanisms/rules"]').click()
    expect(page.locator("#view-title")).to_contain_text("rules")
    assert page.locator("#extra [data-file]").count() > 0
    page.screenshot(path=str(output / "folder-drilldown.png"))
    page.locator("#back").click()
    assert page.evaluate("atlas.getState().id") == "sm/src/game/room_mechanisms"
    page.locator("#back").click()
    page.locator("#back").click()
    assert page.evaluate("atlas.getState().id") == "sm/src/game/session_runtime.rs"
    page.locator("#reset").click()
    assert page.evaluate("atlas.getState().type") == "overview"
    page.locator("#tensions").uncheck()
    assert not page.evaluate("atlas.getState().tensions")
    page.locator("#tensions").check()
    node = page.locator("#graph .graph-node").first
    node.focus()
    page.keyboard.press("Enter")
    assert page.evaluate("atlas.getState().type") != "overview"

    page.set_viewport_size({"width": 390, "height": 844})
    page.locator("#reset").click()
    assert page.evaluate("document.documentElement.scrollWidth <= innerWidth + 1")
    assert "pan-y" in page.locator("#graph").evaluate("element => getComputedStyle(element).touchAction")
    page.screenshot(path=str(output / "overview-mobile.png"))
    audit_accessibility(page, "overview-mobile")
    page.locator("#tree-toggle").click()
    page.locator("#search").fill("sprite_objects.rs")
    page.get_by_test_id("search-result").first.click()
    assert page.evaluate("atlas.getState().id") == "sm/src/game/sprite_objects.rs"
    page.locator("#details-toggle").click()
    expect(page.get_by_test_id("source-link")).to_be_visible()
    page.screenshot(path=str(output / "file-mobile.png"))
    audit_accessibility(page, "file-mobile")
    page.locator("#details .drawer-close").click()
    assert not page.evaluate("document.querySelector('#layout').classList.contains('show-detail')")
    page.screenshot(path=str(output / "file-graph-mobile.png"))
    page.locator("#details-toggle").click()
    page.keyboard.press("Escape")
    assert not page.evaluate("document.querySelector('#layout').classList.contains('show-detail')")
    assert not errors, errors
    assert not network, network
    (output / "accessibility.json").write_text(json.dumps(accessibility, indent=2))
    assert not any(view["violations"] for view in accessibility), [(view["view"], [issue["id"] for issue in view["violations"]]) for view in accessibility]
    (output / "results.json").write_text(json.dumps({"passed": True, "page_errors": errors, "network_requests": network, "files": page.evaluate("atlas.data.files.length"), "edges": page.evaluate("atlas.data.edges.length"), "source_commit": page.evaluate("atlas.data.commit")}, indent=2))
    browser.close()
