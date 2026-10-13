use serde_json::{Value, json};

/// Browser-only fixture; never admitted into the shared gameplay registry.
pub fn fixture(mut data: Value) -> Value {
    let end = 1_791_876_600_u64;
    let hour = 3_600_u64;
    let day = 24 * hour;
    data["routines"].as_array_mut().unwrap().truncate(7);
    let names = data["routines"].as_array().unwrap().iter()
        .map(|row| row["symbol"].as_str().unwrap().to_owned()).collect::<Vec<_>>();
    let event = |id: &str, index: usize, stage: &str, context: &str, at: Value, supersedes: Vec<&str>| json!({
        "id":id, "scope":"Synthetic report-history fixture",
        "agent":"browser-test", "stage":stage, "evidence_context":context,
        "routines":[names[index]], "updated_at":at, "supersedes":supersedes,
        "source_revision":data["source"]["revision"],
        "inventory_sha256":data["source"]["inventory_sha256"],
        "evidence":[], "limitations":[]
    });
    let events = vec![
        event("baseline",0,"translated","historical-frozen",json!(end-8*day),vec![]),
        event("repeat-context",0,"translated","current-checkout",json!(end-hour),vec![]),
        event("new",1,"translated","current-checkout",json!(end-2*hour+42),vec![]),
        event("duplicate",1,"translated","current-checkout",json!(end-hour),vec![]),
        event("stale",2,"translated","current-checkout",json!(end-5*hour),vec![]),
        event("old",3,"translated","unspecified",json!(end-5*hour),vec![]),
        event("replacement",3,"translated","current-checkout",json!(end-3*hour),vec!["old"]),
        event("undated",4,"translated","current-checkout",json!("unavailable"),vec![]),
        event("invalid-date",5,"translated","current-checkout",json!(1e300),vec![]),
        event("left-boundary",6,"translated","unspecified",json!(end-day),vec![]),
        event("verified",0,"verified","historical-frozen",json!(end-4*hour),vec![]),
        event("integrated",1,"integrated","current-checkout",json!(end-2*hour),vec![]),
    ];
    data["updates"]=json!(events);
    data["updated_at"]=json!(end);
    data["stale_update_ids"]=json!(["stale"]);
    data["superseded_update_ids"]=json!(["old"]);
    data["notices"]=json!([]);
    data
}

// Exercise real UI calculations via Chromium, not a mirrored chart algorithm.
pub const CHECKS: &str = r#"
(() => {
  const $=id=>document.getElementById(id);
  const assert=(ok,message)=>{if(!ok)throw Error(message);};
  const change=(id,value)=>{$(id).value=value;$(id).dispatchEvent(new Event('change'));};
  const chart=()=>$('velocity-chart').dataset;
  change('context','');change('milestone','translated');change('velocity-window','24h');
  assert(chart().known==='4','one event per definition across overlapping contexts');
  assert(chart().undated==='2','invalid/missing dates counted separately');
  assert(chart().additions==='3','left boundary included, older baseline excluded from additions');
  assert(chart().baseline==='1','older report kept as cumulative baseline');
  assert($('velocity-caption').textContent.includes('2 definitions have no report date'),'undated explanation');
  assert($('velocity-chart').querySelector('.bucket-hit').getAttribute('aria-label').includes('1 new definitions'),'exact start boundary bucket');
  change('velocity-mode','cumulative');
  assert($('velocity-chart').querySelector('polyline'),'cumulative chart');
  assert($('velocity-table').lastElementChild.dataset.cumulative==='4','cumulative excludes stale/replaced and duplicate records');
  change('velocity-window','7d');
  assert(chart().baseline==='1'&&chart().additions==='3','daily chart keeps older baseline');
  change('velocity-window','all');
  assert(chart().baseline==='0'&&chart().additions==='4','full history includes older evidence');
  change('context','current-checkout');
  assert(chart().known==='3'&&chart().undated==='2','context changes first qualifying report');
  change('context','historical-frozen');
  assert(chart().known==='1'&&chart().undated==='0','historical evidence explicit');
  change('context','');
  change('weight','lines');
  const data=JSON.parse($('progress-data').textContent);
  const expected=[0,1,3,6].reduce((sum,i)=>sum+data.routines[i].body_lines,0);
  assert(Number(chart().known)===expected,'line-weighted native definitions');
  change('weight','routines');change('milestone','verified');
  assert(chart().known==='1','exact stage, not cumulative rank');
  change('milestone','live-exercised');
  assert(chart().known==='0'&&chart().additions==='0','empty stage remains zero');
  assert(!document.getElementById('error').hidden===false,'no error on invalid report dates');
  return {overlapping_reports:true,duplicates:true,stale_exclusions:true,
    permanent_replacement_exclusions:true,old_baseline:true,start_boundary:true,
    missing_and_invalid_dates:true,line_weighting:true,exact_stages:true,empty_stage:true};
})();
"#;
