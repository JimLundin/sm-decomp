use std::{collections::{BTreeMap,BTreeSet},fs};
fn main(){
 let source=fs::read_to_string("/home/jim/sm-decomp/deps/snesrev-sm/src/sm_a6.c").unwrap();
 let mut functions=BTreeMap::<String,String>::new();
 let mut start=0;
 for line in source.split_inclusive('\n'){
  if !line.starts_with(char::is_whitespace)&&line.contains(" { ")&&line.contains('(')||!line.starts_with(char::is_whitespace)&&line.trim_end().ends_with('{')&&line.contains('('){
   let prefix=line.split('(').next().unwrap();
   let name=prefix.split_whitespace().last().unwrap().trim_start_matches('*');
   if name.chars().all(|c|c.is_ascii_alphanumeric()||c=='_'){
    let opening=start+line.find('{').unwrap();let mut depth=0;let mut end=opening;
    for (i,b) in source.as_bytes()[opening..].iter().enumerate(){if *b==b'{'{depth+=1;}if *b==b'}'{depth-=1;if depth==0{end=opening+i+1;break;}}}
    functions.insert(name.into(),source[opening..end].into());
   }
  }
  start+=line.len();
 }
 let roots=["CeresRidley_Init","CeresRidley_Main","CeresRidley_Hurt","Ridley_Main","Ridley_Hurt","Ridley_Func_2","RidleysExplosion_Init","RidleysExplosion_Main","Ridley_Shot","Ridley_Powerbomb","sub_A6DF59","sub_A6DF60","Ridley_A2F2","HandleCeresRidleyGetawayCutscene","Ridley_Instr_1","Ridley_Instr_2","Ridley_Instr_3","Ridley_Instr_4","Ridley_Instr_5","Ridley_Instr_6","Ridley_Instr_7","Ridley_Instr_8","Ridley_Instr_9","Ridley_Instr_10","Ridley_Instr_11","Ridley_Instr_12","Ridley_Instr_13","Ridley_Instr_14","Ridley_Instr_15","Ridley_Instr_16"];
 let mut pending=roots.iter().map(|s|s.to_string()).collect::<Vec<_>>();let mut reached=BTreeSet::new();let mut external=BTreeSet::new();let mut lines=0;
 while let Some(name)=pending.pop(){if !reached.insert(name.clone()){continue;}let Some(body)=functions.get(&name)else{external.insert(name);continue;};lines+=body.lines().count();
  let b=body.as_bytes();for i in 1..b.len(){if b[i]!=b'(' {continue;}let mut end=i;while end>0&&b[end-1].is_ascii_whitespace(){end-=1;}let mut beginning=end;while beginning>0&&(b[beginning-1].is_ascii_alphanumeric()||b[beginning-1]==b'_'){beginning-=1;}if beginning==end{continue;}let call=&body[beginning..end];if ["if","while","for","switch","sizeof","int16","uint16","uint8","int8"].contains(&call){continue;}if functions.contains_key(call){pending.push(call.into());}else{external.insert(call.into());}}
  if name=="Ridley_Func_70"{for n in ["Ridley_CBC0","Ridley_Func_71","Ridley_Func_72","Ridley_CBC7","Ridley_CBCE","Ridley_Func_73","Ridley_Func_74","Ridley_CBD5"]{pending.push(n.into());}}
 }
 println!("{} reachable native definitions, {} source body lines",reached.len(),lines);println!("Native closure:\n{}",reached.into_iter().collect::<Vec<_>>().join("\n"));println!("External dependencies:\n{}",external.into_iter().collect::<Vec<_>>().join("\n"));
}
