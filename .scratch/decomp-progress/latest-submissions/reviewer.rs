//! Disposable layout preview and browser inspection; no gameplay publication.
use anyhow::{Context, Result, ensure};
use base64::Engine;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};

fn save(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn check(browser: &mut Browser, expected: &Value) -> Result<Value> {
    let expression = format!(r#"(() => {{
      const expected = {expected};
      const rows = [...document.querySelectorAll('#submissions-table tr[data-update-id]')];
      const assert = (ok, message) => {{ if (!ok) throw new Error(message); }};
      assert(rows.length === expected.length, 'wrong number of submissions');
      rows.forEach((row, index) => {{
        const wanted = expected[index];
        assert(row.dataset.updateId === wanted.id, 'wrong newest-first order at ' + index);
        assert(row.querySelector('time')?.dateTime === new Date(wanted.updated_at * 1000).toISOString(), 'wrong UTC timestamp');
        assert(row.querySelector('.submission-scope').textContent === wanted.scope, 'scope lost');
        assert(row.querySelector('.badge').dataset.stage === wanted.stage, 'stage lost');
        if (wanted.description) assert(row.textContent.includes(wanted.description), 'description lost');
      }});
      const table = document.querySelector('[aria-labelledby="submissions-title"]');
      assert(table.getBoundingClientRect().top < document.querySelector('.burn-controls').getBoundingClientRect().top, 'table is not at the top');
      assert(document.getElementById('burndown-chart') && document.getElementById('pace-chart') && document.getElementById('routine-explorer'), 'existing report missing');
      assert(document.getElementById('error').hidden, 'report error');
      return rows.map(row => ({{id:row.dataset.updateId,time:row.querySelector('time')?.dateTime,scope:row.querySelector('.submission-scope').textContent,added:row.querySelector('.submission-added').textContent}}));
    }})()"#);
    browser.eval(&expression)
}

fn expected(data: &Value) -> Value {
    let mut updates = data["updates"].as_array().unwrap().iter().enumerate().collect::<Vec<_>>();
    updates.sort_by(|(ai, a), (bi, b)| b["updated_at"].as_u64().cmp(&a["updated_at"].as_u64()).then(bi.cmp(ai)));
    json!(updates.into_iter().take(10).map(|(_, update)| json!({
        "id":update["id"], "scope":update["scope"], "stage":update["stage"],
        "updated_at":update["updated_at"], "description":update["limitations"][0]
    })).collect::<Vec<_>>())
}

fn inspect(url: &str, output: &Path, registry: &Path) -> Result<()> {
    ensure!(!output.exists(), "preserve existing browser evidence");
    fs::create_dir_all(output)?;
    let data: Value = serde_json::from_str(&fs::read_to_string(registry)?)?;
    let mut browser = Browser::launch(output)?;
    browser.call("Emulation.setDeviceMetricsOverride", json!({"width":1360,"height":1000,"deviceScaleFactor":1,"mobile":false}))?;
    browser.navigate(url)?;
    browser.wait("document.querySelectorAll('#submissions-table tr[data-update-id]').length === 10")?;
    let real = check(&mut browser, &expected(&data))?;
    ensure!(real[0]["id"] == "shared-game-host-current-verified-launch-20261013", "latest completed verification missing");
    ensure!(real[0]["added"].as_str().unwrap().contains("no new definitions"), "duplicate verification adds coverage");
    save(&output.join("real.json"), &real)?;
    browser.screenshot(&output.join("desktop.png"))?;
    browser.call("Emulation.setDeviceMetricsOverride", json!({"width":390,"height":844,"deviceScaleFactor":1,"mobile":true}))?;
    let mobile = browser.eval("({width:innerWidth,scroll_width:document.documentElement.scrollWidth,rows:document.querySelectorAll('#submissions-table tr[data-update-id]').length})")?;
    ensure!(mobile["width"] == mobile["scroll_width"], "phone horizontal overflow: {mobile}");
    save(&output.join("phone.json"), &mobile)?;
    browser.screenshot(&output.join("phone.png"))?;

    let refreshed = browser.eval(r#"(async () => {
      const originalFetch = window.fetch, data = JSON.parse(document.getElementById('progress-data').textContent);
      const update = {...data.updates[data.updates.length - 1], id:'browser-live-refresh', scope:'Live refreshed submission', updated_at:Math.floor(Date.now()/1000)+1};
      data.updates.push(update); data.updated_at=update.updated_at;
      let requests = 0;
      window.fetch = async () => { requests++; return new Response(JSON.stringify(data), {status:200,headers:{'Content-Type':'application/json'}}); };
      const until = async (predicate) => {
        const deadline=Date.now()+20000;
        while(!predicate() && Date.now()<deadline) await new Promise(resolve => setTimeout(resolve,100));
        return predicate();
      };
      const replaced=await until(() => document.querySelector('#submissions-table tr')?.dataset.updateId === update.id);
      window.fetch=originalFetch;
      const row=document.querySelector('#submissions-table tr');
      if(!replaced || !row.textContent.includes(update.scope)) throw new Error('Live refresh did not replace latest submissions: '+JSON.stringify({requests,hidden:document.hidden,top:row?.dataset.updateId,error:document.getElementById('error').textContent,synthetic_time:update.updated_at,latest_time:Math.max(...data.updates.map(value=>value.updated_at))}));
      if(!await until(() => document.querySelector('#submissions-table tr')?.dataset.updateId !== update.id)) throw new Error('Live registry did not restore real submission');
      return {status:'passed',synthetic_published:false,checks:['live refresh prepends latest submission','real registry restores table']};
    })()"#)?;
    save(&output.join("live-refresh.json"), &refreshed)?;

    let mut fixture = data.clone();
    fixture["updates"] = json!([]);
    fixture["stale_update_ids"] = json!([]);
    fixture["superseded_update_ids"] = json!([]);
    let empty = output.join("empty.json"); save(&empty, &fixture)?;
    browser.file(&empty)?;
    browser.wait("document.getElementById('submissions-table').textContent.includes('No progress submissions yet.')")?;
    ensure!(browser.eval("document.querySelectorAll('#submissions-table tr[data-update-id]').length")? == 0, "empty JSON left old rows");

    let symbols = [&data["routines"][0]["symbol"], &data["routines"][1]["symbol"]];
    let mut updates = Vec::new();
    for index in 0..12 {
        let mut update = data["updates"].as_array().unwrap().last().unwrap().clone();
        update["id"] = json!(format!("synthetic-{index}"));
        update["scope"] = json!(format!("Synthetic scope {index}"));
        update["stage"] = json!("translated");
        update["updated_at"] = json!(1000 + index);
        update["routines"] = json!([symbols[usize::from(index == 11)]]);
        update["limitations"] = json!([format!("Added behavior {index}")]);
        update["supersedes"] = json!([]);
        updates.push(update);
    }
    updates.swap(2, 11);
    let mut tied = updates.iter().find(|v| v["id"] == "synthetic-11").unwrap().clone();
    tied["id"] = json!("synthetic-newest-tie");
    tied["scope"] = json!("<img src=x onerror=alert(1)> literal scope");
    tied["limitations"] = json!(["Fresh evidence for an already reported routine"]);
    updates.push(tied);
    fixture["updates"] = json!(updates);
    fixture["stale_update_ids"] = json!(["synthetic-10"]);
    let synthetic = output.join("synthetic.json"); save(&synthetic, &fixture)?;
    browser.file(&synthetic)?;
    browser.wait("document.querySelector('#submissions-table tr')?.dataset.updateId === 'synthetic-newest-tie'")?;
    let sorted = check(&mut browser, &expected(&fixture))?;
    ensure!(sorted[0]["added"].as_str().unwrap().contains("no new definitions"), "tied duplicate counted twice");
    ensure!(sorted[1]["added"].as_str().unwrap().starts_with("+1 definition"), "new routine addition missing");
    ensure!(browser.eval("!document.querySelector('#submissions-table img') && [...document.querySelectorAll('#submissions-table .stale-tag')].some(node => node.textContent === 'Stale')")? == true, "literal scope or stale label failed");
    save(&output.join("synthetic-checks.json"), &sorted)?;

    fixture["updates"] = json!([fixture["updates"][0].clone()]);
    let single = output.join("single.json"); save(&single, &fixture)?;
    browser.file(&single)?;
    browser.wait("document.querySelectorAll('#submissions-table tr[data-update-id]').length === 1")?;
    save(&output.join("single-checks.json"), &check(&mut browser, &expected(&fixture))?)?;
    save(&output.join("verification.json"), &json!({"status":"passed","url":url,"checks":["ten newest submissions","exact UTC time","scope and stage","concrete description","zero unique coverage evidence update","top placement","existing report retained","phone layout","live refresh","empty JSON replacement","unordered submissions","equal-time append order","literal HTML","stale labels","single submission"]}))?;
    println!("Latest submissions: browser checks passed");
    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("serve") if args.len() == 3 => serve(Path::new(&args[1]), &args[2]),
        Some("publish") if args.len() == 4 => {
            fs::write(&args[3], render(&fs::read_to_string(&args[1])?, &fs::read_to_string(&args[2])?)?)?;
            Ok(())
        }
        Some("inspect") if args.len() == 4 => inspect(&args[1], Path::new(&args[2]), Path::new(&args[3])),
        _ => anyhow::bail!("usage: serve TEMPLATE BIND | publish TEMPLATE REGISTRY OUTPUT | inspect URL OUTPUT REGISTRY")
    }
}

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
        if done { break; }
    }
    ensure!(headers.starts_with("HTTP/1.1 200"), "HTTP failed: {headers}");
    let length = headers.lines().find_map(|line| {
        line.split_once(':').filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
            .map(|(_, value)| value.trim().parse::<usize>())
    }).transpose()?;
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
    let json = serde_json::to_string(&data)?.replace('&', "\\u0026").replace('<', "\\u003c")
        .replace('>', "\\u003e").replace('\u{2028}', "\\u2028").replace('\u{2029}', "\\u2029");
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
            let count = stream.read(&mut buffer)?;
            if count == 0 { break; }
            request.extend_from_slice(&buffer[..count]);
        }
        let request = String::from_utf8_lossy(&request);
        let target = request.lines().next().unwrap_or("").split(' ').nth(1).unwrap_or("").split('?').next().unwrap_or("");
        let response = (|| -> Result<(String, String)> {
            let raw = http("127.0.0.1:4185", "/registry.json")?;
            match target {
                "/" | "/index.html" => Ok(("text/html; charset=utf-8".into(), render(&fs::read_to_string(template)?, &raw)?)),
                "/registry.json" => Ok(("application/json; charset=utf-8".into(), raw)),
                _ => anyhow::bail!("unknown preview route"),
            }
        })();
        let (status, mime, body) = match response {
            Ok((mime, body)) => ("200 OK", mime, body),
            Err(error) => ("404 Not Found", "text/plain".into(), error.to_string()),
        };
        let header = format!("HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n", body.len());
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
        let process = Process(Command::new(chrome).args([
            "--headless", "--no-sandbox", "--disable-gpu", "--disable-dev-shm-usage",
            "--disable-background-networking", "--no-first-run", "--no-default-browser-check", "--remote-debugging-port=0",
        ]).arg(format!("--user-data-dir={}", profile.display())).arg("about:blank")
            .stdout(Stdio::null()).stderr(Stdio::from(fs::File::create(scratch.join("chromium.log"))?)).spawn()?);
        let started = Instant::now();
        let port = loop {
            if let Ok(text) = fs::read_to_string(profile.join("DevToolsActivePort")) {
                break text.lines().next().context("DevTools port missing")?.parse::<u16>()?;
            }
            ensure!(started.elapsed() < Duration::from_secs(20), "Chromium did not start");
            thread::sleep(Duration::from_millis(100));
        };
        let targets: Value = serde_json::from_str(&http(&format!("127.0.0.1:{port}"), "/json/list")?)?;
        let url = targets.as_array().context("browser targets missing")?.iter()
            .find(|v| v["type"] == "page").and_then(|v| v["webSocketDebuggerUrl"].as_str()).context("page target missing")?;
        let (mut socket, _) = tungstenite::connect(url)?;
        if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
            stream.set_read_timeout(Some(Duration::from_secs(20)))?;
            stream.set_write_timeout(Some(Duration::from_secs(20)))?;
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
        let result = self.call("Runtime.evaluate", json!({"expression": expression, "awaitPromise": true, "returnByValue": true}))?;
        ensure!(result.get("exceptionDetails").is_none(), "browser assertion failed: {result}");
        Ok(result["result"]["value"].clone())
    }
    fn wait(&mut self, expression: &str) -> Result<()> {
        let started = Instant::now();
        loop {
            if self.eval(expression)? == true { return Ok(()); }
            ensure!(started.elapsed() < Duration::from_secs(20), "browser condition timed out: {expression}");
            thread::sleep(Duration::from_millis(100));
        }
    }
    fn navigate(&mut self, url: &str) -> Result<()> {
        self.call("Page.navigate", json!({"url": url}))?;
        self.call("Page.bringToFront", json!({}))?;
        self.wait("document.visibilityState === 'visible'")?;
        self.wait("document.getElementById('total')?.textContent !== '—' && !!document.getElementById('total')")
    }
    fn screenshot(&mut self, path: &Path) -> Result<()> {
        let data = self.call("Page.captureScreenshot", json!({"format": "png", "captureBeyondViewport": false}))?;
        fs::write(path, base64::engine::general_purpose::STANDARD.decode(data["data"].as_str().context("screenshot data missing")?)?)?;
        Ok(())
    }
    fn file(&mut self, path: &Path) -> Result<()> {
        let doc = self.call("DOM.getDocument", json!({}))?;
        let node = self.call("DOM.querySelector", json!({"nodeId": doc["root"]["nodeId"], "selector": "#file"}))?;
        self.call("DOM.setFileInputFiles", json!({"nodeId": node["nodeId"], "files": [path.to_string_lossy()]}))?;
        Ok(())
    }
}
