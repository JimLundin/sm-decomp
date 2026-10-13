use anyhow::{Context, Result, ensure};
use base64::Engine;
use serde_json::{Value, json};
use std::{fs, io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, path::{Path, PathBuf}, process::{Child, Command, Stdio}, thread, time::{Duration, Instant}};
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};

struct Process(Child);
impl Drop for Process { fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); } }
fn http(port: u16, method: &str, target: &str) -> Result<String> {
    let mut socket = TcpStream::connect(("127.0.0.1", port))?;
    socket.set_read_timeout(Some(Duration::from_secs(20)))?;
    socket.write_all(format!("{method} {target} HTTP/1.1\r\nHost: localhost:{port}\r\nConnection: close\r\n\r\n").as_bytes())?;
    let mut reader = BufReader::new(socket);
    let mut headers=String::new();
    loop { let mut line=String::new(); ensure!(reader.read_line(&mut line)? > 0,"HTTP headers ended early"); let done=line=="\r\n"; headers.push_str(&line); if done {break;} }
    ensure!(headers.starts_with("HTTP/1.1 200"), "HTTP failed: {headers}");
    if let Some(length)=headers.lines().find_map(|line|line.split_once(':').filter(|(key,_)|key.eq_ignore_ascii_case("content-length")).map(|(_,value)|value.trim().parse::<usize>())).transpose()? { let mut body=vec![0;length];reader.read_exact(&mut body)?;return Ok(String::from_utf8(body)?); }
    let mut body=String::new();reader.read_to_string(&mut body)?;Ok(body)
}
struct Browser { process: Process, socket: WebSocket<MaybeTlsStream<TcpStream>>, next: u64 }
impl Browser {
    fn launch(chrome: &Path, scratch: &Path) -> Result<Self> {
        let profile = scratch.join("chromium-profile");
        let process = Process(Command::new(chrome).args(["--headless", "--no-sandbox", "--disable-gpu", "--disable-dev-shm-usage", "--disable-background-networking", "--no-first-run", "--no-default-browser-check", "--remote-debugging-port=0"])
            .arg(format!("--user-data-dir={}", profile.display())).arg("about:blank").stdout(Stdio::null()).stderr(Stdio::from(fs::File::create(scratch.join("chromium.log"))?)).spawn()?);
        let started = Instant::now();
        let port = loop { if let Ok(text) = fs::read_to_string(profile.join("DevToolsActivePort")) { break text.lines().next().context("DevTools port missing")?.parse::<u16>()?; } ensure!(started.elapsed() < Duration::from_secs(30), "Chromium did not start"); thread::sleep(Duration::from_millis(100)); };
        let targets: Value = serde_json::from_str(&http(port, "GET", "/json/list").context("read Chromium CDP targets")?)?;
        let url = targets.as_array().context("browser targets missing")?.iter().find(|v| v["type"] == "page").and_then(|v| v["webSocketDebuggerUrl"].as_str()).context("page target missing")?;
        let (mut socket, _) = tungstenite::connect(url).context("connect Chromium page CDP")?;
        if let MaybeTlsStream::Plain(stream)=socket.get_mut() {stream.set_read_timeout(Some(Duration::from_secs(20)))?;stream.set_write_timeout(Some(Duration::from_secs(20)))?;}
        let mut browser = Self {process, socket, next:0};
        browser.call("Page.enable", json!({}))?; Ok(browser)
    }
    fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        self.next += 1; let id = self.next;
        self.socket.send(Message::Text(json!({"id":id,"method":method,"params":params}).to_string().into()))?;
        loop { let message = self.socket.read().with_context(||format!("read CDP response for {method}"))?; if let Message::Text(text) = message { let value: Value = serde_json::from_str(&text)?; if value["id"] == id { ensure!(value.get("error").is_none(), "CDP {method}: {value}"); return Ok(value["result"].clone()); } } }
    }
    fn eval(&mut self, expression: &str) -> Result<Value> {
        let result = self.call("Runtime.evaluate", json!({"expression":expression,"awaitPromise":true,"returnByValue":true}))?;
        ensure!(result.get("exceptionDetails").is_none(), "browser assertion failed: {result}");
        Ok(result["result"]["value"].clone())
    }
    fn navigate(&mut self, url: &str) -> Result<()> { self.call("Page.navigate", json!({"url":url}))?; self.wait("document.querySelector('#total') && document.querySelector('#total').textContent !== '—'") }
    fn wait(&mut self, expression: &str) -> Result<()> { let started=Instant::now(); loop { if self.eval(expression)? == true { return Ok(()); } ensure!(started.elapsed()<Duration::from_secs(15), "browser condition timed out: {expression}"); thread::sleep(Duration::from_millis(150)); } }
    fn select_file(&mut self, path: &Path) -> Result<()> { let doc=self.call("DOM.getDocument",json!({}))?; let node=self.call("DOM.querySelector",json!({"nodeId":doc["root"]["nodeId"],"selector":"#file"}))?; self.call("DOM.setFileInputFiles",json!({"nodeId":node["nodeId"],"files":[path.to_string_lossy()]}))?; Ok(()) }
    fn screenshot(&mut self, path: &Path) -> Result<()> { let data=self.call("Page.captureScreenshot",json!({"format":"png","captureBeyondViewport":false}))?; fs::write(path,base64::engine::general_purpose::STANDARD.decode(data["data"].as_str().context("screenshot data missing")?)?)?; Ok(()) }
}
fn command(binary: &Path, root: &Path, args: &[&str]) -> Result<String> {
    let result=Command::new(binary).arg("progress").args(args).arg("--root").arg(root).output()?;
    ensure!(result.status.success(), "progress failed: {}", String::from_utf8_lossy(&result.stderr)); Ok(String::from_utf8(result.stdout)?)
}
fn payload(root: &Path, source: &Value, id: &str, stage: &str, symbol: &str) -> Result<PathBuf> {
    let path=root.join(format!("{id}.toml"));
    fs::write(&path,format!("id = '{id}'\nscope = 'browser verification scope'\nagent = 'verification-worker'\nstage = '{stage}'\nroutines = ['{symbol}']\nsource_revision = '{}'\ninventory_sha256 = '{}'\nlimitations = ['synthetic isolated verification; no gameplay claim']\n[[evidence]]\npath = 'evidence.txt'\n",source["revision"].as_str().unwrap(),source["inventory_sha256"].as_str().unwrap()))?; Ok(path)
}
fn main() -> Result<()> {
    let repo=Path::new("/home/jim/sm-decomp"); let binary=repo.join("target/debug/sm-pipeline");
    const WRITERS: usize = 6;
    let root=PathBuf::from(std::env::var_os("SM_PROGRESS_BROWSER_ROOT").unwrap_or_else(|| "/tmp/sm-progress-validation-six".into())); ensure!(!root.exists(), "validation root already exists; inspect previous run"); fs::create_dir_all(root.join("tools/port"))?;
    fs::copy(repo.join("tools/dependencies.toml"),root.join("tools/dependencies.toml"))?;
    fs::copy("/tmp/sm-progress-current-inventory.toml",root.join("tools/port/source-inventory.toml"))?;
    fs::copy(repo.join("tools/port/system.toml"),root.join("tools/port/system.toml"))?;
    fs::write(root.join("evidence.txt"),"retained isolated evidence\n")?;
    command(&binary,&root,&["refresh"])?;
    let registry_path=root.join(".scratch/decomp-progress/registry.json");
    let initial: Value=serde_json::from_str(&fs::read_to_string(&registry_path)?)?;
    ensure!(initial["source"]["definition_count"] == 6053, "expected current 6053-definition inventory");
    let inventory: toml::Value=toml::from_str(&fs::read_to_string(root.join("tools/port/source-inventory.toml"))?)?;
    let scope: toml::Value=toml::from_str(&fs::read_to_string(repo.join(".scratch/decomp-port/ridley-whole-progress-translating.toml"))?)?;
    let known=inventory["routines"].as_array().unwrap().iter().map(|v|v["symbol"].as_str().unwrap()).collect::<std::collections::BTreeSet<_>>();
    let missing=scope["routines"].as_array().unwrap().iter().filter_map(|v|{let s=v.as_str().unwrap(); (!known.contains(s)).then_some(s)}).collect::<Vec<_>>(); ensure!(missing.is_empty(), "Ridley scope symbols absent: {missing:?}");
    let mut children=Vec::new();
    let publication_started=Instant::now();
    for i in 0..WRITERS { let path=payload(&root,&initial["source"],&format!("concurrent-{i}"),if i%2==0 {"translated"} else {"verified"},"APU_UploadBank")?; children.push(Command::new(&binary).args(["progress","update","--root"]).arg(&root).arg("--update").arg(path).stdout(Stdio::null()).stderr(Stdio::piped()).spawn()?); }
    let mut atomic_reads=0;
    let mut writer_completion_ms=vec![None;WRITERS];
    loop { let current: Value=serde_json::from_str(&fs::read_to_string(&registry_path)?)?; ensure!(current["updates"].as_array().unwrap().len()<=WRITERS,"excess updates"); atomic_reads+=1; let mut running=false; for (index,child) in children.iter_mut().enumerate() { if child.try_wait()?.is_none() {running=true;} else if writer_completion_ms[index].is_none() {writer_completion_ms[index]=Some(publication_started.elapsed().as_millis());} } if !running {break;} thread::sleep(Duration::from_millis(5)); }
    for child in children { let result=child.wait_with_output()?; ensure!(result.status.success(),"concurrent publication failed: {}",String::from_utf8_lossy(&result.stderr)); }
    let publication_ms=publication_started.elapsed().as_millis();
    let current: Value=serde_json::from_str(&fs::read_to_string(&registry_path)?)?; ensure!(current["updates"].as_array().unwrap().len()==WRITERS,"lost concurrent update");
    let before=current["updates"].as_array().unwrap().iter().find(|update|update["id"]=="concurrent-0").context("retry observation missing")?["updated_at"].clone(); let retry=root.join("concurrent-0.toml"); command(&binary,&root,&["update","--update",retry.to_str().unwrap()])?;
    let retried: Value=serde_json::from_str(&fs::read_to_string(&registry_path)?)?; ensure!(retried["updates"].as_array().unwrap().iter().find(|update|update["id"]=="concurrent-0").context("retried observation missing")?["updated_at"]==before && retried["updates"].as_array().unwrap().len()==WRITERS,"retry changed observations");
    println!("six concurrent CLI publications passed in {publication_ms} ms, {atomic_reads} atomic JSON reads");
    let listener=TcpListener::bind("127.0.0.1:0")?; let port=listener.local_addr()?.port(); drop(listener);
    let bind=format!("127.0.0.1:{port}"); let mut server=Process(Command::new(&binary).args(["progress","serve","--root"]).arg(&root).args(["--bind",&bind]).stdout(Stdio::piped()).stderr(Stdio::from(fs::File::create(root.join("server.log"))?)).spawn()?);
    let mut line=String::new(); BufReader::new(server.0.stdout.take().unwrap()).read_line(&mut line)?; ensure!(line.contains("Progress dashboard:"),"server startup failed");
    let mut browser=Browser::launch(Path::new("/home/jim/.cache/ms-playwright/chromium-1248/chrome-linux64/chrome"),&root).context("launch browser")?;
    browser.call("Emulation.setDeviceMetricsOverride",json!({"width":1360,"height":1000,"deviceScaleFactor":1,"mobile":false}))?;
    browser.navigate(&format!("http://127.0.0.1:{port}/index.html")).context("navigate served dashboard")?;
    browser.eval("(() => { const $=id=>document.getElementById(id); const assert=(v,m)=>{if(!v)throw Error(m)}; assert($('total').textContent.replace(/,/g,'')==='6053','inventory count'); assert($('reported').textContent==='1','deduplicated translated coverage'); assert($('error').hidden,'no initialization error'); $('search').value='APU_UploadBank'; $('search').dispatchEvent(new Event('input')); assert(document.querySelectorAll('.routine').length===1,'symbol search'); $('bank').value='80'; $('bank').dispatchEvent(new Event('change')); $('agent').value='verification-worker'; $('agent').dispatchEvent(new Event('change')); $('scope').value='browser verification scope'; $('scope').dispatchEvent(new Event('change')); assert(document.querySelectorAll('.routine').length===1,'combined bank agent scope filters'); document.querySelector('summary.routine-summary').click(); return true; })()")?;
    browser.wait("document.querySelectorAll('.observation').length === 6")?;
    browser.eval("(() => {const $=id=>document.getElementById(id);if(!document.querySelector('.routine-detail').textContent.includes('SHA-256'))throw Error('evidence digest details'); $('milestone').value='integrated';$('milestone').dispatchEvent(new Event('change'));if($('reported').textContent!=='0')throw Error('stages inferred');$('reset').click();$('show-remaining').click();if(!$('results').textContent.startsWith('6,053'))throw Error('remaining selection');$('theme').click();if(!document.documentElement.dataset.theme)throw Error('theme');return true;})()")?;
    browser.screenshot(&root.join("desktop.png"))?;
    browser.eval("document.getElementById('reset').click(); document.getElementById('search').value='Ridley_E90C'; document.getElementById('search').dispatchEvent(new Event('input')); true")?;
    let live=payload(&root,&initial["source"],"live-refresh","translated","Ridley_E90C")?; command(&binary,&root,&["update","--update",live.to_str().unwrap()])?;
    browser.wait("document.querySelector('.routine-stage [data-stage=translated]') !== null")?;
    browser.eval("(() => {if(document.getElementById('search').value!=='Ridley_E90C')throw Error('refresh filter lost');if(document.querySelectorAll('.routine').length!==1)throw Error('refresh routine');return true;})()")?;
    fs::write(root.join("evidence.txt"),"changed after publication\n")?;
    browser.wait("document.querySelector('.routine-stage [data-stage=unreported]') !== null")?;
    let snapshot:Value=serde_json::from_str(&http(port,"GET","/registry.json")?)?; ensure!(snapshot["summary"]["stale_updates"]==WRITERS+1,"HTTP evidence staleness missing");
    browser.eval("document.querySelector('summary.routine-summary').click(); true")?; browser.wait("document.querySelector('[data-stale=true]') !== null")?;
    let uploaded=root.join("opened.json"); let mut opened=snapshot.clone(); opened["updates"]=json!([]);opened["stale_update_ids"]=json!([]);fs::write(&uploaded,serde_json::to_vec(&opened)?)?; browser.select_file(&uploaded)?; browser.wait("document.getElementById('sync-mode').textContent.includes('Opened JSON snapshot')")?;
    let bad=root.join("malformed.json");fs::write(&bad,"{invalid json")?;browser.select_file(&bad)?;browser.wait("!document.getElementById('error').hidden")?;
    browser.navigate(&format!("file://{}/.scratch/decomp-progress/index.html",root.display()))?;
    browser.eval("(() => {if(!document.getElementById('sync-mode').textContent.includes('offline'))throw Error('offline embedded mode');if(document.getElementById('total').textContent!=='6,053')throw Error('offline inventory');return true;})()")?;
    browser.call("Emulation.setDeviceMetricsOverride",json!({"width":390,"height":844,"deviceScaleFactor":1,"mobile":true}))?;browser.screenshot(&root.join("phone.png"))?;
    browser.eval("(() => {if(document.documentElement.scrollWidth > window.innerWidth)throw Error('phone horizontal overflow');document.getElementById('search').value='Ridley_E90C';document.getElementById('search').dispatchEvent(new Event('input'));document.querySelector('summary.routine-summary').click();return true;})()")?;
    browser.wait("document.querySelector('.routine-detail') !== null")?;
    let results=json!({"success":true,"definitions":6053,"ridley_scope_symbols":scope["routines"].as_array().unwrap().len(),"concurrent_cli_writers":WRITERS,"concurrent_publication_ms":publication_ms,"writer_completion_ms":writer_completion_ms,"atomic_reads":atomic_reads,"observations_after_retry":WRITERS,"live_refresh":true,"changed_evidence_excluded":true,"file_loading":true,"malformed_file_error":true,"offline_embed":true,"desktop_and_phone":true,"checks":["symbol and combined filters","exact independent milestones","remaining","detail evidence","theme","filter-preserving refresh","stale detail","mobile layout"],"output_root":root.to_string_lossy()});
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&results)?)?;
    println!("{}",serde_json::to_string_pretty(&results)?); drop(browser.process); Ok(())
}
