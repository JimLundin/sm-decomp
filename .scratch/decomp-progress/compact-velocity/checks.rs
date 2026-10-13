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
  assert(metricsTop < 110, 'compact progress visible near top');
  assert(document.querySelector('.metrics').nextElementSibling === $('report-details'), 'details beside overview');
  assert($('report-details').nextElementSibling.querySelector('#banks'), 'banks follow details');
  assert(!$('bank-details').open, 'bank grid collapsed initially');
  assert($('velocity-chart').getBoundingClientRect().height < 190, 'compact velocity chart');
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
      assert(Number($('velocity-chart').dataset.known) === counts.definitions, 'one dated event per definition and exact stage');
      assert(Number($('velocity-chart').dataset.undated) === 0, 'all current reports dated');
      const chartTotal = [...$('velocity-table').rows].reduce((sum, row) => sum + Number(row.dataset.total), 0);
      assert(chartTotal === number($('velocity-gains')), 'bucket additions agree with period total');
      assert(Number($('velocity-table').lastElementChild.dataset.cumulative) === counts.definitions, 'cumulative reaches exact stage total');
    }
  }
  change('context', ''); change('milestone', 'verified');
  change('weight', 'lines');
  assert(Number($('velocity-chart').dataset.known) === data.summary.stages.verified.body_lines, 'body-line history weighting');
  change('weight', 'routines');
  for (const period of ['24h', '7d', 'all']) {
    change('velocity-window', period);
    const expected = data.summary.stages.verified.definitions;
    assert(Number($('velocity-table').lastElementChild.dataset.cumulative) === expected, 'period preserves cumulative baseline');
    change('velocity-mode', 'cumulative');
    assert($('velocity-chart').querySelector('polyline'), 'cumulative line renders');
    change('velocity-mode', 'additions');
    assert([...$('velocity-chart').querySelectorAll('rect')].some(rect => !rect.classList.contains('bucket-hit')), 'velocity bars render');
  }
  change('velocity-window', '24h');
  const bucket = $('velocity-chart').querySelector('.bucket-hit');
  bucket.focus();
  assert($('velocity-readout').textContent.includes('UTC'), 'keyboard-accessible UTC chart values');
  bucket.dispatchEvent(new KeyboardEvent('keydown', {key:'ArrowRight',bubbles:true}));
  assert(document.activeElement !== bucket, 'arrow key moves to next bucket');
  $('velocity-chart').querySelector('.bucket-hit').dispatchEvent(new PointerEvent('pointerenter'));
  assert($('velocity-readout').textContent.includes('cumulative'), 'hover readout contains actual value');
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
    compact_velocity: true, exact_stage_history: true, deduplicated_history: true,
    body_line_weighting: true, cumulative_baselines: true, keyboard_and_hover: true,
    evidence_and_correction_history: true, live_refresh: location.protocol.startsWith('http'),
    visible_errors: true
  };
})();
"#;
