(() => {
  const count=document.getElementById('reported').textContent;
  if(count!=='0') throw new Error('Superseded source revived after replacement became stale: '+count);
  const search=document.getElementById('search');
  search.value='NextRandom'; search.dispatchEvent(new Event('input'));
  const routine=document.querySelector('.routine');
  if(!routine) throw new Error('Original routine no longer visible');
  routine.open=true; routine.dispatchEvent(new Event('toggle'));
  const text=routine.textContent;
  if(!text.includes('Superseded observation') || !text.includes('browser-correction')) throw new Error('Correction provenance hidden');
  if(!text.includes('No matching report')) throw new Error('Superseded stage still selected');
  return {translated_count:count,original_visible:true,replacement_reason_visible:true,permanent_exclusion:true};
})()
