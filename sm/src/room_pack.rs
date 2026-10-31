//! Room-pack application selection, before Game construction.
#![forbid(unsafe_code)]
use sm_game::room_entry::world::WorldBinding;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

pub use sm_game::room_entry::pack::*;

/// Both native startup and the verifier consume this exact two-token selection.
/// Arguments exclude the executable name. Other arguments retain their order.
pub struct Selection {
    pub world: Option<Arc<WorldBinding>>,
    pub arguments: Vec<String>,
}
/// Select an optional room pack before native startup.
/// Windows desktop room-pack loading requires ASCII ROM path text; automatic
/// directory search for a relative ROM also requires an ASCII current directory.
/// Pack paths and verifier ROM paths retain Rust Unicode file handling.
pub fn select(arguments: &[String]) -> Result<Selection> {
    select_args(arguments, true)
}
/// Verifier argv starts with the ROM and does not enter native SwitchDirectory.
pub fn select_for_verifier(arguments: &[String]) -> Result<Selection> {
    select_args(arguments, false)
}
fn select_args(arguments: &[String], native: bool) -> Result<Selection> {
    let mut path = None;
    let mut retained = Vec::new();
    let mut position = 0;
    while position < arguments.len() {
        if arguments[position] == "--room-pack" {
            if path.is_some() {
                return Err(error("duplicate --room-pack selection"));
            }
            let value = arguments
                .get(position + 1)
                .filter(|value| !value.is_empty() && !value.starts_with('-'))
                .ok_or_else(|| error("--room-pack requires a path"))?;
            path = Some(value);
            position += 2;
        } else if arguments[position].starts_with("--room-pack=") {
            return Err(error("room pack selection requires --room-pack PATH"));
        } else {
            retained.push(arguments[position].clone());
            position += 1;
        }
    }
    let world = path.map(|path| load_file(Path::new(path))).transpose()?;
    if let Some(world) = &world {
        let path = selected_rom_path(&retained, native)?;
        let file = File::open(&path).map_err(|e| error(format!("{}: {e}", path.display())))?;
        let mut source = Vec::new();
        file.take(MAX_ROM_SOURCE_BYTES as u64 + 1)
            .read_to_end(&mut source)
            .map_err(|e| error(format!("{}: {e}", path.display())))?;
        let normalized = normalize_source(&source)?;
        let expected = world
            .prepared_room_source_size()
            .ok_or_else(|| error("selected room pack lacks a complete room catalog"))?;
        if expected != normalized.len() {
            return Err(error(format!(
                "room pack requires {expected} normalized ROM bytes; selected ROM has {}",
                normalized.len(),
            )));
        }
    }
    Ok(Selection {
        world,
        arguments: retained,
    })
}

/// Windows desktop pack ROM paths are ASCII to match native CRT path spelling.
/// An ASCII cwd is required only for relative ROMs using automatic directory search.
fn selected_rom_path(arguments: &[String], native: bool) -> Result<PathBuf> {
    let desktop = native && arguments.first().map(String::as_str) != Some("verify");
    let config = desktop && arguments.len() >= 2 && arguments[0] == "--config";
    let mut position = if config {
        2
    } else if native && !desktop {
        1
    } else {
        0
    };
    if desktop && arguments.get(position).map(String::as_str) == Some("--debug") {
        position += 1;
    }
    let filename = match arguments.get(position) {
        Some(filename) => filename.as_str(),
        None if desktop => "sm.smc",
        None => return Err(error("selected room pack requires a ROM path")),
    };
    #[cfg(windows)]
    if desktop && !filename.is_ascii() {
        return Err(error(
            "Windows desktop room-pack loading requires an ASCII ROM path",
        ));
    }
    let path = Path::new(filename);
    if !desktop || config || path.is_absolute() {
        return std::path::absolute(path).map_err(|e| error(format!("ROM path: {e}")));
    }
    let current = std::env::current_dir().map_err(|e| error(format!("ROM directory: {e}")))?;
    #[cfg(windows)]
    if !current.as_os_str().as_encoded_bytes().is_ascii() {
        return Err(error(
            "Windows desktop room-pack directory search requires an ASCII current directory",
        ));
    }
    #[cfg(windows)]
    let directory = {
        use std::os::windows::ffi::OsStrExt;
        current.as_os_str().encode_wide().collect::<Vec<_>>()
    };
    #[cfg(unix)]
    let directory = {
        use std::os::unix::ffi::OsStrExt;
        current.as_os_str().as_bytes().to_vec()
    };
    let mut position = directory.len();
    // Native getcwd uses4064 bytes; its three textual prefixes stop at position zero.
    if current.as_os_str().as_encoded_bytes().len() < 4064 {
        for _ in 0..3 {
            if position == 0 {
                break;
            }
            #[cfg(windows)]
            let prefix = {
                use std::os::windows::ffi::OsStringExt;
                std::ffi::OsString::from_wide(&directory[..position])
            };
            #[cfg(unix)]
            let prefix = {
                use std::os::unix::ffi::OsStringExt;
                std::ffi::OsString::from_vec(directory[..position].to_vec())
            };
            let mut config_path = prefix.clone();
            config_path.push("/sm.ini");
            if File::open(config_path).is_ok() {
                return Ok(Path::new(&prefix).join(path));
            }
            position -= 1;
            while position != 0 && directory[position] != 0x2f && directory[position] != 0x5c {
                position -= 1;
            }
        }
    }
    Ok(current.join(path))
}

/// Read once at the caller's current directory, before the native host changes it.
pub fn load_file(path: &Path) -> Result<Arc<WorldBinding>> {
    let path = std::path::absolute(path).map_err(|e| error(format!("room pack path: {e}")))?;
    let file = File::open(&path).map_err(|e| error(format!("{}: {e}", path.display())))?;
    let mut bytes = Vec::new();
    file.take(MAX_PACK_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| error(format!("{}: {e}", path.display())))?;
    load(&bytes).map_err(|e| error(format!("{}: {e}", path.display())))
}
