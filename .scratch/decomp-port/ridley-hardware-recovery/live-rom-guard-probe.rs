const EXECUTION:&[u8]=include_bytes!("/tmp/sm-ridley-whole-live-probe-9000-r5/ridley-execution.rom");
fn execution_matches(actual:&[u8])->bool {
 actual.len()==EXECUTION.len() && actual[..0x2965a]==EXECUTION[..0x2965a]
 && (actual[0x2965a]==EXECUTION[0x2965a] || EXECUTION[0x2965a]==0x8f && actual[0x2965a]==0x8b)
 && actual[0x2965b..]==EXECUTION[0x2965b..]
}
fn main(){
 let native=std::fs::read("/tmp/sm-ridley-whole-live-probe-9000-r5/mixed-0/ridley-error-runtime.rom").unwrap();
 assert!(execution_matches(EXECUTION));assert!(execution_matches(&native));
 for offset in [0,0x29659,0x2965b,0x130000,EXECUTION.len()-1]{let mut changed=native.clone();changed[offset]^=1;assert!(!execution_matches(&changed),"accepted unknown mutation at {offset:x}");}
 let mut operand=native.clone();operand[0x2965a]=0x8a;assert!(!execution_matches(&operand));
 assert!(!execution_matches(&native[..native.len()-1]));
 println!("exact current guard accepts retained native bugfix byte and refuses all unrelated/operand/length mutations");
}
