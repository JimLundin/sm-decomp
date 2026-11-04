fn f() {
    let a = "// not a comment /* nor this";
    let b = "escaped \" quote and \\";
    let c = b"bytes \x00";
    let d = r#"raw "quoted" // text"#;
    let e = br##"raw bytes "# still"##;
    let g = "two
lines";
    let h = c"c string";
    let i = r"plain raw \";
}
