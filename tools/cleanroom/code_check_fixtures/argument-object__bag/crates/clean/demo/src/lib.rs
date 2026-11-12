//! A struct built only to carry one function's arguments.
struct Go {
    at: u16,
    d: u16,
}

fn go(go: Go) -> u16 {
    let Go { at, d } = go;
    at.wrapping_add(d)
}

/// Walks.
pub fn walk(a: u16, b: u16) -> u16 {
    go(Go { at: a, d: b }).wrapping_add(go(Go { at: b, d: a }))
}
