// DOM assertions are evaluated in Chromium by the Rust CDP inspector.
pub const BROWSER_CHECKS: &str = r#"
(async () => {
  const $ = id => document.getElementById(id);
  const assert = (condition, message) => { if (!condition) throw new Error(message); };
  const number = node => Number(node.textContent.replace(/[^0-9]/g, ''));
  const data = JSON.parse($('progress-data').textContent);
  const stages = ['translating', 'translated', 'verified', 'integrated', 'live-exercised'];
  const contexts = ['', 'current-checkout', 'historical-frozen', 'unspecified'];
  const change = (id, value) => { $(id).value = value; $(id).dispatchEvent(new Event('change')); };
  const notes = [...document.querySelectorAll('#notices li')].map(node => node.textContent);
  const metricsTop = document.querySelector('.metrics').getBoundingClientRect().top;
  assert(!$('report-details').open, 'report details collapsed initially');
  assert(metricsTop < 280, 'progress visible near top');
  assert(document.querySelector('.metrics').nextElementSibling === $('report-details'), 'details beside overview');
  assert($('report-details').nextElementSibling.querySelector('#banks'), 'banks follow details');
  for (const note of data.notices) assert(notes.includes(note), 'exact note missing: ' + note);
  assert(document.querySelectorAll('.note-group').length === 4, 'four note groups');
  const declarationCount = data.notices.filter(note => note.startsWith('Declaration ')).length;
  assert(document.querySelector('[data-kind="declarations"] .note-list').children.length === declarationCount, 'declaration note count');
  assert($('report-detail-count').textContent.includes(data.notices.length + ' notes'), 'accurate summary');
  assert($('report-detail-count').textContent.includes('stale reports excluded'), 'excluded report summary');
  assert($('error').closest('details') === null, 'errors visible outside details');
  for (const context of contexts) {
    change('context', context);
    for (const stage of stages) {
      change('milestone', stage);
      const counts = context ? data.summary.context_stages[context][stage] : data.summary.stages[stage];
      assert(number($('reported')) === counts.definitions, 'stage/context total ' + context + '/' + stage);
      assert(number($('remaining')) === data.routines.length - counts.definitions, 'remaining total');
    }
  }
  change('context', ''); change('milestone', 'translated');
  const original = data.updates.find(update => data.superseded_update_ids.includes(update.id));
  const routine = original.routines[0];
  $('search').value = routine; $('search').dispatchEvent(new Event('input'));
  const summary = [...document.querySelectorAll('.routine-summary')].find(node => node.dataset.symbol === routine);
  assert(summary, 'routine search works'); summary.click();
  await new Promise(resolve => setTimeout(resolve, 100));
  assert(document.querySelector('.routine-detail').textContent.includes('SHA-256'), 'evidence details retained');
  assert(document.querySelector('.routine-detail').textContent.includes('Replaced by'), 'correction history retained');
  $('reset').click();
  $('report-details').querySelector(':scope > summary').click();
  document.querySelector('[data-kind="declarations"] > summary').click();
  const list = document.querySelector('[data-kind="declarations"] .note-list');
  assert(list.tabIndex === 0, 'notes keyboard accessible');
  assert(list.clientHeight <= 300 && list.scrollHeight > list.clientHeight, 'notes bounded and scrollable');
  if (location.protocol.startsWith('http')) {
    $('search').value = routine; $('search').dispatchEvent(new Event('input'));
    await new Promise(resolve => setTimeout(resolve, 7800));
    assert($('report-details').open, 'refresh preserves outer disclosure');
    assert(document.querySelector('[data-kind="declarations"]').open, 'refresh preserves group');
    assert($('search').value === routine, 'refresh preserves filter');
    assert($('checked').textContent.startsWith('Checked:'), 'live refresh checks');
    $('reset').click();
  }
  for (const group of document.querySelectorAll('.note-group')) group.open = false;
  window.scrollTo(0, 0);
  return {
    progress_first: true, metrics_top: metricsTop, collapsed_report_details: true,
    exact_notices_retained: data.notices.length, declaration_notes: declarationCount,
    grouped_scrollable_notes: true, independent_context_stage_totals: true,
    evidence_and_correction_history: true, live_refresh: location.protocol.startsWith('http'),
    visible_errors: true
  };
})();
"#;
