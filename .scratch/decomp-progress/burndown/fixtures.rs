use serde_json::{Value, json};
pub fn registry(original: &Value, case: &str) -> Value {
    let mut data = original.clone();
    let source = data["source"].clone();
    let routines = original["routines"].as_array().unwrap()[..5].to_vec();
    let first = routines[0]["symbol"].as_str().unwrap().to_owned();
    let second = routines[1]["symbol"].as_str().unwrap().to_owned();
    let baseline = 1_700_000_000u64;
    let finish = baseline + 3 * 86400;
    data["routines"] = json!(routines);
    data["updates"] = json!([]);
    data["updated_at"] = json!(finish);
    data["source"]["definition_count"] = json!(5);
    data["source"]["body_lines"] =
        json!(data["routines"].as_array().unwrap().iter().map(|r| r["body_lines"].as_u64().unwrap()).sum::<u64>());
    data["notices"] = json!([]);
    data["stale_update_ids"] = json!([]);
    data["superseded_update_ids"] = json!([]);
    let mut points = vec![
        json!({"commit":"a".repeat(40),"at":baseline+86400,"subject":"boundary excluded from recent window","added":[first],"removed":[]}),
        json!({"commit":"b".repeat(40),"at":baseline+2*86400,"subject":"remove exact start boundary","added":[],"removed":[first]}),
        json!({"commit":"c".repeat(40),"at":finish-3600,"subject":"new and repeated selector","added":[second,second],"removed":[]}),
    ];
    let mut current = vec![second.clone()];
    let mut status = "current";
    if case == "zero" {
        points = vec![
            json!({"commit":"a".repeat(40),"at":baseline+3600,"subject":"old baseline","added":[first],"removed":[]}),
        ];
        current = vec![first.clone()];
    }
    if case == "negative" {
        points = vec![
            json!({"commit":"a".repeat(40),"at":baseline+3600,"subject":"old source","added":[first],"removed":[]}),
            json!({"commit":"b".repeat(40),"at":finish-3600,"subject":"coverage removed","added":[],"removed":[first]}),
        ];
        current.clear();
    }
    if case == "short" {
        data["updated_at"] = json!(baseline + 3600);
        points.clear();
        current.clear();
    }
    if case == "stale" {
        status = "source-changed";
    }
    data["coverage_history"] = json!({"schema_version":1,"generated_at":data["updated_at"],"head":"e".repeat(40),"source_revision":source["revision"],"inventory_sha256":source["inventory_sha256"],"baseline_commit":"d".repeat(40),"baseline_at":baseline,"checkpoints":points,"current":{"definitions":current.len(),"body_lines":0},"current_symbols":current,"uncommitted":[],"files":[],"inputs":[],"status":status,"limitations":["Synthetic reporting fixture, no gameplay."]});
    if case == "none" {
        data.as_object_mut().unwrap().remove("coverage_history");
    }
    data
}
pub const CHECKS: &str = r#"(() => {
 const $=id=>document.getElementById(id), data=JSON.parse($('progress-data').textContent), name=document.documentElement.dataset.fixture;
 const assert=(value,message)=>{if(!value)throw new Error(name+': '+message)};
 $('burn-weight').value='routines';$('burn-weight').dispatchEvent(new Event('change'));
 if(name==='none') {assert(!$('burn-unavailable').hidden,'no history message');assert($('burn-pace').textContent==='—','no fabricated velocity');}
 else if(name==='stale') {assert(!$('burn-unavailable').hidden,'stale banner');assert($('burn-pace').textContent==='—'&&$('burn-eta').textContent==='—','stale rate and forecast suppressed');}
 else if(name==='short') {assert($('burn-pace').textContent==='—','incomplete window rate suppressed');assert(!$('pace-chart').querySelector('.velocity-line'),'no line before full window');}
 else if(name==='zero') {assert($('burn-pace').textContent==='0/day','zero output preserved');assert($('burn-eta').textContent==='—','no infinity forecast');assert(Number($('burndown-chart').dataset.remaining)===4,'old baseline coverage retained');}
 else if(name==='negative') {assert(Number($('pace-chart').dataset.pace)===-1,'removal produces negative net velocity');assert($('burn-eta').textContent==='—','no negative completion forecast');assert(Number($('burndown-chart').dataset.remaining)===5,'burndown rises on removal');}
 else {assert(Number($('pace-chart').dataset.net)===1,'duplicate selector deduplication and start boundary');assert(Number($('burndown-chart').dataset.remaining)===4,'unique native coverage');assert($('burn-eta').textContent==='4 days','conditional forecast');}
 assert(!$('burn-eta').textContent.includes('NaN'),'no invalid numbers');
 return {case:name,passed:true,pace:$('burn-pace').textContent,forecast:$('burn-eta').textContent};
})()"#;
