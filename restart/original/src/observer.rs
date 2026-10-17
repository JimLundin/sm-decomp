use crate::Result;
use std::path::Path;

pub fn generate(output: &Path) -> Result<String> {
    let output = output.to_str().ok_or("Lua output path must be UTF-8")?;
    if output.contains("]]") || output.contains('\n') || output.contains('\r') {
        return Err("unsupported Lua output path".into());
    }
    Ok(include_str!("observer.lua").replace("__OUTPUT__", output))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_lua_path_terminators() {
        assert!(generate(Path::new("/tmp/]]bad")).is_err());
        assert!(generate(Path::new("/tmp/line\nbreak")).is_err());
        assert!(generate(Path::new("/tmp/path with spaces")).unwrap().contains("[[/tmp/path with spaces]]"));
    }
}
