(async () => {
  const $ = id => document.getElementById(id);
  const assert = (condition, message) => { if (!condition) throw Error(message); };
  const data = JSON.parse($('progress-data').textContent);
  const stale = new Set(data.stale_update_ids || []);
  const superseded = new Set([...(data.superseded_update_ids || []), ...data.updates.flatMap(u => u.supersedes || [])]);
  const count = (stage, context) => new Set(data.updates.filter(u => u.stage === stage && !stale.has(u.id) && !superseded.has(u.id) && (!context || (u.evidence_context || 'unspecified') === context)).flatMap(u => u.routines)).size;
  assert(!$('report-details').open, 'Report details must start collapsed');
  assert($('notices').closest('#report-details'), 'Notices must remain accessible in Report details');
  assert(!$('error').closest('details'), 'Load errors must remain visible outside details');
  assert($('report-detail-count').textContent.includes(String(data.notices.length + (stale.size ? 1 : 0))), 'Notice count must remain visible');
  assert($('context').querySelector('[value=current-checkout]').textContent === 'Checkout at observation', 'Checkout label must describe observation time');
  for (const context of ['', 'current-checkout', 'historical-frozen', 'unspecified']) {
    $('context').value = context;
    for (const stage of ['translated', 'verified', 'integrated', 'live-exercised']) {
      $('milestone').value = stage;
      $('context').dispatchEvent(new Event('change'));
      assert(Number($('reported').textContent.replace(/,/g, '')) === count(stage, context), 'Context count differs: ' + stage + '/' + context);
    }
  }
  $('reset').click();
  $('search').value = 'HandleProjectile'; $('search').dispatchEvent(new Event('input'));
  const row = [...document.querySelectorAll('.routine')].find(v => v.querySelector('.symbol').textContent === 'HandleProjectile');
  assert(row, 'Routine search remains available'); row.querySelector('summary').click();
  await new Promise(resolve => setTimeout(resolve, 50));
  assert(row.textContent.includes('Evidence context'), 'Routine details must expose evidence context');
  assert(row.textContent.includes('SHA-256'), 'Routine evidence hashes remain visible');
  $('report-details').open = true;
  assert($('notices').textContent.length > 0, 'Notices retained after opening details');
  $('report-details').open = false;
  $('reset').click();
  if (superseded.size) {
    $('search').value = 'Ridley_Func_17'; $('search').dispatchEvent(new Event('input'));
    const corrected = [...document.querySelectorAll('.routine')].find(v => v.querySelector('.symbol').textContent === 'Ridley_Func_17');
    assert(corrected, 'Reviewed Ridley helper remains searchable'); corrected.querySelector('summary').click();
    await new Promise(resolve => setTimeout(resolve, 50));
    assert(corrected.textContent.includes('Superseded observation'), 'Historical superseded claim stays visible');
    assert(corrected.textContent.includes('Reviewed maintained Ridley/Ceres implementation correspondence'), 'Reviewed correction stays visible');
    $('reset').click();
  }
  return {context_counts:true, observation_context_labels:true, collapsed_report_details:true, visible_errors:true, notices_retained:true, routine_evidence_details:true, supersession_history:true, definitions:data.routines.length, observations:data.updates.length};
})()
