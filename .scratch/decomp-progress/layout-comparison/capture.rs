//! Capture only throwaway comparison files without touching the shared index.
use std::{fs,process::Command};
fn git(args:&[&str],index:&str)->String {
 let result=Command::new("git").args(args).env("GIT_INDEX_FILE",index).output().unwrap();
 assert!(result.status.success(),"git failed: {}",String::from_utf8_lossy(&result.stderr));
 String::from_utf8(result.stdout).unwrap().trim().to_string()
}
fn main(){
 let index="/tmp/sm-layout-comparison-capture.index";
 assert!(!std::path::Path::new(index).exists());
 let branch="throwaway/progress-layout-comparison-20261013";
 let executable=std::env::current_exe().unwrap();let reclaimed=fs::metadata(&executable).unwrap().len();fs::remove_file(&executable).unwrap();
 let path=".scratch/decomp-progress/layout-comparison.md";let note=fs::read_to_string(path).unwrap();fs::write(path,format!("{note}\nThe temporary capture executable was also removed ({} bytes); total identified cleanup is {} bytes. The temporary capture index is removed immediately after branch creation.\n",reclaimed,23455620+reclaimed)).unwrap();
 git(&["read-tree","HEAD"],index);
 git(&["add","--force","--",".scratch/decomp-progress/layout-comparison.md",".scratch/decomp-progress/layout-comparison"],index);
 let tree=git(&["write-tree"],index);let parent=git(&["rev-parse","HEAD"],index);
 let commit=git(&["commit-tree",&tree,"-p",&parent,"-m","prototype: compare three progress report layouts"],index);
 git(&["update-ref",&format!("refs/heads/{branch}"),&commit,""],index);
 fs::remove_file(index).unwrap();
 fs::write(".scratch/decomp-progress/layout-comparison/branch.txt",format!("Branch: {branch}\nCommit: {commit}\nBase: {parent}\nCapture uses a temporary index; shared checkout and index untouched.\n")).unwrap();
 println!("{branch} {commit}");
}
