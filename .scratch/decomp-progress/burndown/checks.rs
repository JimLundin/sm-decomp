// Real-registry browser checks: independent sets/window sums, disclosures,
// chart interaction and preservation of the original reporting contracts.
pub const BROWSER_CHECKS: &str = r#"
(async () => {
  const $ = id => document.getElementById(id);
  const assert = (value, message) => { if (!value) throw new Error(message); };
  const change = (id,value) => { $(id).value=value; $(id).dispatchEvent(new Event('change')); };
  const integer = node => Number(node.textContent.replace(/[^0-9]/g,''));
  const data=JSON.parse($('progress-data').textContent), history=data.coverage_history;
  const notes=[...document.querySelectorAll('#notices li')].map(node=>node.textContent);
  assert(history?.schema_version===1,'source history embedded');
  assert(!$('evidence-overview').open && !$('routine-explorer').open,'supporting evidence collapsed');
  assert(!$('report-details').open,'notices collapsed');
  assert(document.querySelector('.burn-metrics').getBoundingClientRect().top<150,'burndown visible near top');
  assert(document.querySelectorAll('h1').length===1,'one page title');
  assert(!$('burn-unavailable').hidden===false,'current reconstructed source');
  for(const note of data.notices) assert(notes.includes(note),'original notice preserved');
  const excluded=new Set([...data.stale_update_ids,...data.superseded_update_ids]);
  const verified=new Set(data.updates.filter(u=>u.stage==='verified'&&!excluded.has(u.id)).flatMap(u=>u.routines));
  const translated=new Set(history.current_symbols);
  const stageCounts={translated:translated.size,verified:verified.size,integrated:data.summary.stages.integrated.definitions,'live-exercised':data.summary.stages['live-exercised'].definitions};
  for(const measure of ['routines','lines']) {
    change('burn-weight',measure);
    const size=s=>measure==='lines'? data.routines.find(r=>r.symbol===s).body_lines:1;
    const total=data.routines.reduce((sum,r)=>sum+size(r.symbol),0), done=history.current_symbols.reduce((sum,s)=>sum+size(s),0);
    assert(integer($('burn-remaining'))===total-done,'remaining fixed denominator '+measure);
    assert(Number($('burndown-chart').dataset.total)===total,'chart native denominator');
    assert(Number($('burndown-chart').dataset.remaining)===total-done,'chart remaining');
    for(const hours of ['6','12','24']) {
      change('burn-window',hours);
      const finish=Number($('burndown-chart').dataset.at), duration=Number(hours)*3600;
      const recent=history.checkpoints.filter(p=>p.at>finish-duration&&p.at<=finish).reduce((sum,p)=>sum+p.added.reduce((n,s)=>n+size(s),0)-p.removed.reduce((n,s)=>n+size(s),0),0);
      assert(Number($('pace-chart').dataset.net)===recent,'equal-window net changes '+hours+'/'+measure);
      assert(Number($('pace-chart').dataset.pace)===recent/(Number(hours)/24),'calendar-time rate');
      assert(!$('burn-eta').textContent.includes('NaN'),'finite conditional forecast');
      assert($('pace-chart').querySelector('.velocity-line'),'rolling line rendered');
    }
  }
  const pipeline=history.pipeline;
  assert(pipeline?.status==='current'&&pipeline.samples.length===3,'retained hashed pipeline timings');
  for(const sample of pipeline.samples) {
    assert($('pipeline-samples').textContent.includes(sample.label),'retained timing label');
    assert($('pipeline-provenance').textContent.includes(sample.evidence.sha256),'retained timing hash');
  }
  change('burn-weight','routines');change('burn-window','24');
  for(const option of [...$('burn-bank').options].filter(o=>o.value&&!o.value.startsWith('group:'))) {
    change('burn-bank',option.value);
    const rows=data.routines.filter(r=>(r.source.match(/sm_([0-9a-f]{2})/i)?.[1].toUpperCase()||'Other')===option.value), done=rows.filter(r=>translated.has(r.symbol)).length;
    assert(Number($('burndown-chart').dataset.remaining)===rows.length-done,'bank denominator '+option.value);
  }
  const groupButton=$('work-table').querySelector('button');groupButton.click();
  assert($('burn-bank').value.startsWith('group:'),'source-group drilldown');
  const groupedRemaining=Number($('burndown-chart').dataset.remaining);
  assert(groupedRemaining>0&&groupedRemaining<data.routines.length,'grouped native denominator');
  change('burn-bank','');
  for(const id of ['burndown-chart','pace-chart']) {
    const hits=$(id).querySelectorAll('.sample-hit'); assert(hits.length>1,'interactive samples');
    hits[0].focus(); hits[0].dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight',bubbles:true}));
    assert(document.activeElement===hits[1],'keyboard sample navigation');
    hits[hits.length-1].dispatchEvent(new PointerEvent('pointerenter'));
  }
  const queues=data.routines.filter(r=>translated.has(r.symbol)&&!verified.has(r.symbol)).length;
  assert($('flow-queues').textContent.includes(queues.toLocaleString()),'deduplicated verification gap');
  assert([...$('flow-stages').querySelectorAll('button')].map(integer).join(',')===Object.values(stageCounts).join(','),'independent stage counts');
  $('flow-stages').querySelectorAll('button')[1].click();
  assert($('evidence-overview').open&&$('routine-explorer').open,'stage opens evidence');
  const stages=['translating','translated','verified','integrated','live-exercised'];
  for(const context of ['','current-checkout','historical-frozen','unspecified']) {
    change('context',context);
    for(const stage of stages) {
      change('milestone',stage);
      const expected=context?data.summary.context_stages[context][stage]:data.summary.stages[stage];
      assert(integer($('reported'))===expected.definitions,'preserved evidence total');
      assert(Number($('velocity-chart').dataset.known)===expected.definitions,'report-import history remains separate');
    }
  }
  change('context','');change('milestone','translated');
  const original=data.updates.find(u=>data.superseded_update_ids.includes(u.id)), symbol=original.routines[0];
  $('search').value=symbol;$('search').dispatchEvent(new Event('input'));
  const routine=[...document.querySelectorAll('.routine-summary')].find(n=>n.dataset.symbol===symbol);
  assert(routine,'routine search');routine.click();await new Promise(r=>setTimeout(r,100));
  assert(document.querySelector('.routine-detail').textContent.includes('Replaced by'),'replacement history');
  assert(document.querySelector('.routine-detail').textContent.includes('SHA-256'),'retained evidence hashes');
  assert($('error').closest('details')===null,'file errors visible');
  if(location.protocol.startsWith('http')) {
    const chartEnd=Number($('burndown-chart').dataset.at);
    await new Promise(r=>setTimeout(r,7800));
    assert($('checked').textContent.startsWith('Checked:'),'served refresh');
    assert($('search').value===symbol,'filter survives live refresh');
    assert($('burn-weight').value==='routines'&&$('burn-window').value==='24','burndown controls survive');
    assert(Number($('burndown-chart').dataset.at)>chartEnd,'calendar windows advance during a source pause');
  }
  $('reset').click();$('evidence-overview').open=false;$('routine-explorer').open=false;
  change('burn-weight','lines');change('burn-bank','');change('burn-window','24');window.scrollTo(0,0);
  return {source_checkpoints:history.checkpoints.length,current_mapped:translated.size,remaining_definitions:data.routines.length-translated.size,notices:notes.length,fixed_denominator:true,calendar_velocity:true,all_bank_scopes:true,weighted_equal_windows:true,conditional_forecast:true,keyboard_hover:true,evidence_gaps:true,original_evidence_preserved:true,served_refresh:location.protocol.startsWith('http')};
})()
"#;
