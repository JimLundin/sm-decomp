//! Disposable layout preview and browser inspection; no gameplay publication.
use anyhow::{Context, Result, ensure};
use base64::Engine;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};
mod layout_checks {
    include!("checks.rs");
}
mod fixtures;

struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn http(address: &str, target: &str) -> Result<String> {
    let mut socket = TcpStream::connect(address)?;
    socket.set_read_timeout(Some(Duration::from_secs(20)))?;
    socket.write_all(format!("GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n").as_bytes())?;
    let mut reader = BufReader::new(socket);
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        ensure!(reader.read_line(&mut line)? > 0, "HTTP headers ended early");
        let done = line == "\r\n";
        headers.push_str(&line);
        if done {
            break;
        }
    }
    ensure!(headers.starts_with("HTTP/1.1 200"), "HTTP failed: {headers}");
    let length = headers
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                .map(|(_, value)| value.trim().parse::<usize>())
        })
        .transpose()?;
    if let Some(length) = length {
        let mut body = vec![0; length];
        reader.read_exact(&mut body)?;
        return Ok(String::from_utf8(body)?);
    }
    let mut body = String::new();
    reader.read_to_string(&mut body)?;
    Ok(body)
}

fn render(template: &str, raw: &str) -> Result<String> {
    let data: Value = serde_json::from_str(raw)?;
    let json = serde_json::to_string(&data)?
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");
    ensure!(template.contains("__PROGRESS_JSON__"), "template data marker missing");
    Ok(template.replace("__PROGRESS_JSON__", &json))
}

fn serve(template: &Path, bind: &str) -> Result<()> {
    let listener = TcpListener::bind(bind)?;
    println!("Layout preview: http://{}/index.html?variant=A", listener.local_addr()?);
    for stream in listener.incoming() {
        let mut stream = stream?;
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        while !request.windows(4).any(|v| v == b"\r\n\r\n") && request.len() <= 8192 {
            let count = match stream.read(&mut buffer) {
                Ok(count) => count,
                Err(_) => break,
            };
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
        }
        if !request.windows(4).any(|v| v == b"\r\n\r\n") {
            continue;
        }
        let request = String::from_utf8_lossy(&request);
        let target =
            request.lines().next().unwrap_or("").split(' ').nth(1).unwrap_or("").split('?').next().unwrap_or("");
        let response = (|| -> Result<(String, String)> {
            let raw = http("127.0.0.1:4185", "/registry.json")?;
            match target {
                "/" | "/index.html" => {
                    Ok(("text/html; charset=utf-8".into(), render(&fs::read_to_string(template)?, &raw)?))
                }
                "/registry.json" => Ok(("application/json; charset=utf-8".into(), raw)),
                _ => anyhow::bail!("unknown preview route"),
            }
        })();
        let (status, mime, body) = match response {
            Ok((mime, body)) => ("200 OK", mime, body),
            Err(error) => ("404 Not Found", "text/plain".into(), error.to_string()),
        };
        let header = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(header.as_bytes()).and_then(|_| stream.write_all(body.as_bytes()));
    }
    Ok(())
}

struct Browser {
    _process: Process,
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next: u64,
}
impl Browser {
    fn launch(scratch: &Path) -> Result<Self> {
        let profile = scratch.join("chromium-profile");
        let chrome = "/home/jim/.cache/ms-playwright/chromium-1248/chrome-linux64/chrome";
        let process = Process(
            Command::new(chrome)
                .args([
                    "--headless",
                    "--no-sandbox",
                    "--disable-gpu",
                    "--disable-dev-shm-usage",
                    "--disable-background-networking",
                    "--no-first-run",
                    "--no-default-browser-check",
                    "--remote-debugging-port=0",
                ])
                .arg(format!("--user-data-dir={}", profile.display()))
                .arg("about:blank")
                .stdout(Stdio::null())
                .stderr(Stdio::from(fs::File::create(scratch.join("chromium.log"))?))
                .spawn()?,
        );
        let started = Instant::now();
        let port = loop {
            if let Ok(text) = fs::read_to_string(profile.join("DevToolsActivePort")) {
                break text.lines().next().context("DevTools port missing")?.parse::<u16>()?;
            }
            ensure!(started.elapsed() < Duration::from_secs(20), "Chromium did not start");
            thread::sleep(Duration::from_millis(100));
        };
        let targets: Value = serde_json::from_str(&http(&format!("127.0.0.1:{port}"), "/json/list")?)?;
        let url = targets
            .as_array()
            .context("browser targets missing")?
            .iter()
            .find(|v| v["type"] == "page")
            .and_then(|v| v["webSocketDebuggerUrl"].as_str())
            .context("page target missing")?;
        let (mut socket, _) = tungstenite::connect(url)?;
        if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
            stream.set_read_timeout(Some(Duration::from_secs(45)))?;
            stream.set_write_timeout(Some(Duration::from_secs(45)))?;
        }
        let mut browser = Self { _process: process, socket, next: 0 };
        browser.call("Page.enable", json!({}))?;
        Ok(browser)
    }
    fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        self.next += 1;
        let id = self.next;
        self.socket.send(Message::Text(json!({"id": id, "method": method, "params": params}).to_string().into()))?;
        loop {
            if let Message::Text(text) = self.socket.read()? {
                let value: Value = serde_json::from_str(&text)?;
                if value["id"] == id {
                    ensure!(value.get("error").is_none(), "CDP {method}: {value}");
                    return Ok(value["result"].clone());
                }
            }
        }
    }
    fn eval(&mut self, expression: &str) -> Result<Value> {
        let result = self
            .call("Runtime.evaluate", json!({"expression": expression, "awaitPromise": true, "returnByValue": true}))?;
        ensure!(result.get("exceptionDetails").is_none(), "browser assertion failed: {result}");
        Ok(result["result"]["value"].clone())
    }
    fn wait(&mut self, expression: &str) -> Result<()> {
        let started = Instant::now();
        loop {
            if self.eval(expression)? == true {
                return Ok(());
            }
            ensure!(started.elapsed() < Duration::from_secs(20), "browser condition timed out: {expression}");
            thread::sleep(Duration::from_millis(100));
        }
    }
    fn navigate(&mut self, url: &str) -> Result<()> {
        self.call("Page.navigate", json!({"url": url}))?;
        self.wait("document.getElementById('total')?.textContent !== '—' && !!document.getElementById('total')")
    }
    fn screenshot(&mut self, path: &Path) -> Result<()> {
        let data = self.call("Page.captureScreenshot", json!({"format": "png", "captureBeyondViewport": false}))?;
        fs::write(
            path,
            base64::engine::general_purpose::STANDARD
                .decode(data["data"].as_str().context("screenshot data missing")?)?,
        )?;
        Ok(())
    }
    fn file(&mut self, path: &Path) -> Result<()> {
        let doc = self.call("DOM.getDocument", json!({}))?;
        let node = self.call("DOM.querySelector", json!({"nodeId": doc["root"]["nodeId"], "selector": "#file"}))?;
        self.call("DOM.setFileInputFiles", json!({"nodeId": node["nodeId"], "files": [path.to_string_lossy()]}))?;
        Ok(())
    }
}

fn inspect(url: &str, output: &Path, checks: Option<&str>) -> Result<()> {
    ensure!(!output.exists(), "inspection output already exists; preserve previous evidence");
    fs::create_dir_all(output)?;
    let mut browser = Browser::launch(output)?;
    browser.call(
        "Emulation.setDeviceMetricsOverride",
        json!({"width": 1360, "height": 1000, "deviceScaleFactor": 1, "mobile": false}),
    )?;
    browser.navigate(url)?;
    let layout = browser.eval("(() => ({title:document.title, total:document.getElementById('total').textContent, metrics_top:document.querySelector('.burn-metrics').getBoundingClientRect().top, coverage:document.getElementById('burn-complete').textContent, remaining:document.getElementById('burn-remaining').textContent,pace:document.getElementById('burn-pace').textContent,change:document.getElementById('burn-change').textContent,forecast:document.getElementById('burn-eta').textContent, notices:document.getElementById('notices')?.textContent.length, scroll_width:document.documentElement.scrollWidth, viewport:innerWidth}))()")?;
    browser.screenshot(&output.join("desktop.png"))?;
    if let Some(checks) = checks {
        let result = browser.eval(checks)?;
        fs::write(output.join("checks.json"), serde_json::to_vec_pretty(&result)?)?;
        browser.screenshot(&output.join("details.png"))?;
    }
    browser.call(
        "Emulation.setDeviceMetricsOverride",
        json!({"width": 390, "height": 844, "deviceScaleFactor": 1, "mobile": true}),
    )?;
    browser.eval("window.scrollTo(0,0); true")?;
    browser.screenshot(&output.join("phone.png"))?;
    let phone = browser.eval("({width:innerWidth, scroll_width:document.documentElement.scrollWidth, metrics_top:document.querySelector('.burn-metrics').getBoundingClientRect().top})")?;
    if checks.is_some() {
        ensure!(phone["scroll_width"] == phone["width"], "phone horizontal overflow: {phone}");
        browser.eval("document.getElementById('report-details').open=false; window.scrollTo(0,0); true")?;
        browser.screenshot(&output.join("phone-collapsed.png"))?;
        browser.eval("document.documentElement.dataset.theme='dark'; true")?;
        browser.screenshot(&output.join("phone-dark.png"))?;
        browser.call(
            "Emulation.setDeviceMetricsOverride",
            json!({"width":1360,"height":1000,"deviceScaleFactor":1,"mobile":false}),
        )?;
        browser.screenshot(&output.join("desktop-dark.png"))?;
    }
    fs::write(
        output.join("inspection.json"),
        serde_json::to_vec_pretty(&json!({"url":url,"desktop":layout,"phone":phone,"checks":checks.is_some()}))?,
    )?;
    println!("{}", serde_json::to_string_pretty(&json!({"desktop":layout,"phone":phone}))?);
    Ok(())
}

fn cleanup_profiles(root: &Path) -> Result<()> {
    fn bytes(path: &Path) -> Result<u64> {
        let metadata = fs::symlink_metadata(path)?;
        let mut total = metadata.len();
        if metadata.is_dir() {
            for entry in fs::read_dir(path)? {
                total += bytes(&entry?.path())?;
            }
        }
        Ok(total)
    }
    let root = root.canonicalize()?;
    ensure!(root.ends_with("burndown"), "cleanup requires the burndown root");
    let mut removed = Vec::new();
    let mut total = 0;
    for run in [
        "offline-initial",
        "offline-second",
        "offline-final",
        "offline-live-clock-check",
        "served-final-initial",
        "served-checked",
        "served-final",
        "file-picker",
        "file-final",
        "history-fixtures",
        "history-final",
        "prototype/variant-a",
        "prototype/variant-b",
        "prototype/capture-a",
        "prototype/capture-b",
        "prototype/capture-c",
        "prototype/fresh-a",
        "prototype/fresh-b",
        "prototype/fresh-c",
    ] {
        let profile = root.join(run).join("chromium-profile");
        if profile.exists() {
            ensure!(fs::symlink_metadata(&profile)?.is_dir(), "browser profile must be a real directory");
            let size = bytes(&profile)?;
            fs::remove_dir_all(&profile)?;
            removed.push(json!({"path":profile,"bytes":size}));
            total += size;
        }
        let fixture = root.join(run).join("opened.json");
        if fixture.exists() {
            ensure!(fs::symlink_metadata(&fixture)?.is_file(), "upload fixture must be a real file");
            let size = bytes(&fixture)?;
            fs::remove_file(&fixture)?;
            removed.push(json!({"path":fixture,"bytes":size}));
            total += size;
        }
    }
    fs::create_dir_all(root.join("cleanup"))?;
    let receipt = json!({"identified_unsealed_bytes_reclaimed":total,"removed":removed,"retained":"Rust source, locked dependencies, embedded snapshot, screenshots, assertions, logs and malformed-file diagnostic fixtures"});
    fs::write(root.join("cleanup/browser-profiles.json"), serde_json::to_vec_pretty(&receipt)?)?;
    println!("Removed {total} identified unsealed browser bytes");
    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("cleanup-profiles") if args.len() == 2 => cleanup_profiles(Path::new(&args[1])),
        Some("serve") if args.len() == 3 => serve(Path::new(&args[1]), &args[2]),
        Some("publish") if args.len() == 4 => {
            let html = render(&fs::read_to_string(&args[1])?, &fs::read_to_string(&args[2])?)?;
            let output = PathBuf::from(&args[3]);
            let temporary = output.with_extension("layout-tmp");
            fs::write(&temporary, html)?;
            fs::rename(temporary, &output)?;
            println!("Published {}", output.display());
            Ok(())
        }
        Some("inspect") if args.len() == 3 => {
            inspect(&args[1], Path::new(&args[2]), Some(layout_checks::BROWSER_CHECKS))
        }
        Some("fixture-check") if args.len() == 4 => {
            let output = Path::new(&args[3]);
            ensure!(!output.exists(), "history inspection output already exists");
            fs::create_dir_all(output)?;
            let output = output.canonicalize()?;
            let original: Value = serde_json::from_str(&fs::read_to_string(&args[2])?)?;
            let mut browser = Browser::launch(&output)?;
            let mut checks = Vec::new();
            for case in ["boundaries", "zero", "negative", "short", "stale", "none"] {
                let data = fixtures::registry(&original, case);
                fs::write(output.join(format!("{case}.json")), serde_json::to_vec_pretty(&data)?)?;
                let html = render(&fs::read_to_string(&args[1])?, &serde_json::to_string(&data)?)?
                    .replace("<html lang=\"en\">", &format!("<html lang=\"en\" data-fixture=\"{case}\">"));
                let page = output.join(format!("{case}.html"));
                fs::write(&page, html)?;
                browser.navigate(&format!("file://{}", page.display()))?;
                checks.push(browser.eval(fixtures::CHECKS)?);
                browser.screenshot(&output.join(format!("{case}.png")))?;
            }
            fs::write(output.join("checks.json"), serde_json::to_vec_pretty(&checks)?)?;
            println!("{}", serde_json::to_string_pretty(&checks)?);
            Ok(())
        }
        Some("served-consistency") if args.len() == 2 => {
            let address = &args[1];
            let raw = http(address, "/registry.json")?;
            let served: Value = serde_json::from_str(&raw)?;
            let stored: Value = serde_json::from_str(&fs::read_to_string(".scratch/decomp-progress/registry.json")?)?;
            ensure!(served == stored, "served JSON differs from stored registry");
            let html = http(address, "/index.html")?;
            ensure!(html.contains("Decomp burndown"), "served template lacks burndown");
            let raw = html
                .split_once("<script id=\"progress-data\" type=\"application/json\">")
                .context("embedded data missing")?
                .1
                .split_once("</script>")
                .context("embedded data end missing")?
                .0;
            let embedded: Value = serde_json::from_str(raw)?;
            ensure!(embedded == stored, "served embedded snapshot differs");
            println!(
                "Stored registry, served JSON and served embedded snapshot agree; {} observations",
                served["updates"].as_array().unwrap().len()
            );
            Ok(())
        }
        Some("file-check") if args.len() == 3 => {
            let output = Path::new(&args[2]);
            fs::create_dir_all(output)?;
            let output = output.canonicalize()?;
            let mut browser = Browser::launch(&output)?;
            browser.navigate(&args[1])?;
            let mut data: Value = serde_json::from_str(&http("127.0.0.1:4185", "/registry.json")?)?;
            data["notices"]
                .as_array_mut()
                .context("notices missing")?
                .push(json!("Layout verification notice <literal HTML>"));
            let uploaded = output.join("opened.json");
            fs::write(&uploaded, serde_json::to_vec(&data)?)?;
            browser.file(&uploaded)?;
            browser.wait("document.getElementById('sync-mode').textContent.includes('Opened JSON snapshot')")?;
            ensure!(browser.eval("document.getElementById('notices').textContent.includes('Layout verification notice <literal HTML>')")? == true, "file notes lost");
            data.as_object_mut().unwrap().remove("coverage_history");
            let no_history = output.join("no-history.json");
            fs::write(&no_history, serde_json::to_vec(&data)?)?;
            browser.file(&no_history)?;
            browser.wait("document.getElementById('burn-pace').textContent==='—'")?;
            ensure!(browser.eval("document.getElementById('burn-change').textContent==='' && document.getElementById('pace-readout').textContent==='' && document.getElementById('pipeline-samples').textContent==='' && document.getElementById('work-table').textContent==='' ")? == true,"previous snapshot leaked into missing-history state");
            let malformed = output.join("malformed.json");
            fs::write(&malformed, "{invalid")?;
            browser.file(&malformed)?;
            browser.wait("!document.getElementById('error').hidden")?;
            ensure!(
                browser.eval("document.getElementById('error').closest('details') === null")? == true,
                "load error hidden in disclosure"
            );
            println!("JSON picker and visible malformed-file error pass");
            fs::write(
                output.join("verification.json"),
                "{\"file_picker\":true,\"literal_notice_text\":true,\"visible_load_error\":true}\n",
            )?;
            Ok(())
        }
        Some("prototype-inspect") if args.len() == 3 => inspect(&args[1], Path::new(&args[2]), None),
        _ => anyhow::bail!(
            "usage: serve TEMPLATE BIND | publish TEMPLATE REGISTRY OUTPUT | inspect URL OUTPUT [CHECKS] | file-check URL OUTPUT"
        ),
    }
}
