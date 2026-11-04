fn f<'a>(x: &'a str) -> &'a str {
    let q = '\'';
    let s = '\\';
    let u = '\u{1F600}';
    let e = 'é';
    let b = b'x';
    let c = 'c';
    'outer: loop {
        break 'outer;
    }
    x
}
fn g() -> &'static str { "'" }
