//! Throwaway structural UI comparison; no gameplay or registry mutations.
use std::fs;
fn main() {
    let root = ".scratch/decomp-progress/layout-comparison";
    fs::create_dir_all(root).unwrap();
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    let template=fs::read_to_string(args.first().map(String::as_str).unwrap_or(".scratch/decomp-progress/layout-comparison/base-template.html.in")).unwrap();
    let input=fs::read_to_string(args.get(1).map(String::as_str).unwrap_or(".scratch/decomp-progress/layout-comparison/prototype.html")).unwrap();
    let registry=if input.contains("id=\"progress-data\"") { input.split_once("<script id=\"progress-data\" type=\"application/json\">").unwrap().1.split_once("</script>").unwrap().0.to_string() } else {input};
    let style = r#"<style>
/* Three structural alternatives on the existing viewer; PROTOTYPE only. */
.prototype-banner {border:1px dashed var(--accent);padding:9px 14px;border-radius:8px;color:var(--muted);font-size:12px;margin-bottom:18px}
.prototype-switcher {position:fixed;z-index:99;bottom:12px;left:50%;transform:translateX(-50%);display:flex;align-items:center;gap:10px;background:var(--ink);color:var(--bg);padding:7px 12px;border-radius:30px;box-shadow:0 2px 20px #0008;font-size:12px;max-width:calc(100vw - 20px);white-space:nowrap}
.prototype-switcher a {color:inherit;text-decoration:none;padding:4px 10px}
.shell {padding-bottom:90px}
.prototype-grid {display:grid;grid-template-columns:minmax(0,1fr) 310px;gap:24px;align-items:start}
.prototype-grid article,.prototype-grid aside {min-width:0}
.prototype-grid aside {position:sticky;top:18px}
.prototype-grid #report-details {margin:24px 0 0}
.prototype-grid .provenance-content {max-height:70vh;overflow:auto;overflow-wrap:anywhere}
.prototype-tabs {display:flex;gap:8px;margin:20px 0 8px}
.prototype-tabs button[aria-selected=true] {border-color:var(--accent);background:var(--accent-bg)}
[hidden] {display:none!important}
@media(max-width:1050px) {.prototype-grid {grid-template-columns:minmax(0,1fr)}.prototype-grid aside {position:static}.prototype-grid .provenance-content {max-height:none}.prototype-grid #report-details {margin-top:0}}
</style>"#;
    let script = r#"<script>
(() => {
  'use strict';
  const variants = ['progress','sidebar','tabs'];
  const names = {progress:'Progress first',sidebar:'Diagnostics sidebar',tabs:'Overview / details tabs'};
  const url = new URL(location.href);
  const variant = variants.includes(url.searchParams.get('variant')) ? url.searchParams.get('variant') : 'progress';
  document.body.dataset.prototypeVariant=variant;
  const main=document.querySelector('main'), error=document.getElementById('error'), details=document.getElementById('report-details');
  const metrics=document.querySelector('.metrics'), panels=[...main.querySelectorAll(':scope > section.panel')];
  const banner=document.createElement('div');banner.className='prototype-banner';banner.textContent='THROWAWAY PROTOTYPE · '+names[variant]+' · same registry and viewer behavior';main.prepend(banner);
  if(variant==='sidebar') {
    const grid=document.createElement('div');grid.className='prototype-grid';const article=document.createElement('article'),aside=document.createElement('aside');aside.setAttribute('aria-label','Report diagnostics');
    metrics.before(grid);grid.append(article,aside);article.append(metrics,...panels);aside.append(details);
    const desktop=matchMedia('(min-width:1051px)');details.open=desktop.matches;desktop.addEventListener('change',event=>details.open=event.matches);
  }
  if(variant==='tabs') {
    const tabs=document.createElement('div');tabs.className='prototype-tabs';tabs.setAttribute('role','tablist');tabs.setAttribute('aria-label','Report views');
    const overview=document.createElement('div'),diagnostics=document.createElement('div');overview.id='prototype-overview';diagnostics.id='prototype-diagnostics';overview.setAttribute('role','tabpanel');diagnostics.setAttribute('role','tabpanel');
    error.after(tabs,overview,diagnostics);overview.append(metrics,...panels);diagnostics.append(details);details.open=true;
    for(const [id,label,panel] of [['overview','Overview',overview],['diagnostics','Report details',diagnostics]]) {
      const button=document.createElement('button');button.type='button';button.id='prototype-tab-'+id;button.textContent=label;button.setAttribute('role','tab');button.setAttribute('aria-controls',panel.id);panel.setAttribute('aria-labelledby',button.id);tabs.append(button);
      button.addEventListener('click',()=>select(id));
    }
    function select(id) {overview.hidden=id!=='overview';diagnostics.hidden=id!=='diagnostics';for(const button of tabs.children)button.setAttribute('aria-selected',String(button.id==='prototype-tab-'+id));}
    select('overview');
  }
  const switcher=document.createElement('nav');switcher.className='prototype-switcher';switcher.setAttribute('aria-label','Prototype variant');
  const index=variants.indexOf(variant);
  function target(delta) {const next=new URL(location.href);next.searchParams.set('variant',variants[(index+delta+3)%3]);return next.href;}
  const prev=document.createElement('a'),next=document.createElement('a'),label=document.createElement('span');prev.href=target(-1);prev.textContent='←';prev.setAttribute('aria-label','Previous layout');next.href=target(1);next.textContent='→';next.setAttribute('aria-label','Next layout');label.textContent=(index+1)+'/3 · '+names[variant];switcher.append(prev,label,next);document.body.append(switcher);
  addEventListener('keydown',event=>{if(event.target.closest('input,textarea,select,[contenteditable=true]'))return;if(event.key==='ArrowLeft'||event.key==='ArrowRight'){event.preventDefault();location.href=target(event.key==='ArrowLeft'?-1:1);}});
})();
</script>"#;
    let output = template.replace("</head>", &format!("{style}</head>")).replace("</body>", &format!("{script}</body>"));
    fs::write(format!("{root}/prototype.html.in"), &output).unwrap();
    let safe = registry.replace('&', "\\u0026").replace('<', "\\u003c").replace('>', "\\u003e");
    fs::write(format!("{root}/prototype.html"), output.replace("__PROGRESS_JSON__", &safe)).unwrap();
    let checks = r#"(async()=>{
const $=id=>document.getElementById(id),assert=(ok,message)=>{if(!ok)throw Error(message)},data=JSON.parse($('progress-data').textContent),stale=new Set(data.stale_update_ids||[]),variant=document.body.dataset.prototypeVariant;
assert(!$('error').closest('details')&&!$('error').closest('[role=tabpanel]'),'Errors stay outside disclosures/tabs');
assert($('notices').closest('#report-details'),'Every notice remains accessible');
assert($('notices').querySelectorAll('li').length===data.notices.length+(stale.size?1:0),'Notice count exact');
if(variant==='progress')assert(!$('report-details').open,'Progress details starts collapsed');
if(variant==='sidebar')assert(!!document.querySelector('.prototype-grid aside #report-details'),'Sidebar structure present');
if(variant==='tabs'){$('prototype-tab-diagnostics').click();assert(!$('prototype-diagnostics').hidden&&$('prototype-overview').hidden,'Details tab opens');$('prototype-tab-overview').click();assert(!$('prototype-overview').hidden,'Overview returns');}
for(const context of ['', 'current-checkout','historical-frozen','unspecified'])for(const stage of ['translated','verified','integrated','live-exercised']){$('context').value=context;$('milestone').value=stage;$('context').dispatchEvent(new Event('change'));const expected=new Set(data.updates.filter(u=>u.stage===stage&&!stale.has(u.id)&&(!context||(u.evidence_context||'unspecified')===context)).flatMap(u=>u.routines)).size;assert(Number($('reported').textContent.replaceAll(',',''))===expected,'Independent context/stage counts preserved');}
$('reset').click();$('search').value='HandleProjectile';$('search').dispatchEvent(new Event('input'));const row=[...document.querySelectorAll('.routine')].find(r=>r.querySelector('.symbol').textContent==='HandleProjectile');assert(row,'Search preserved');row.querySelector('summary').click();await new Promise(resolve=>setTimeout(resolve,50));assert(row.textContent.includes('SHA-256')&&row.textContent.includes('Evidence context'),'Evidence details preserved');$('reset').click();
return {variant,definitions:data.routines.length,observations:data.updates.length,notices:data.notices.length,stale:stale.size,independent_stage_context_counts:true,all_notices_retained:true,errors_outside_collapsed_content:true,filters_and_evidence:true,offline_snapshot:true};
})()"#;
    fs::write(format!("{root}/checks-expression.txt"),checks).unwrap();
    println!("Generated three variants in {root}/prototype.html");
}
