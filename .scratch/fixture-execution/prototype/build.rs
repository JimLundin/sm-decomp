fn main() {
    println!("cargo:rustc-link-search=native=/tmp/sm-fixture-execution-prototype/native");
    println!("cargo:rustc-link-lib=static=fixture_oracle");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-arg=-no-pie");
}
