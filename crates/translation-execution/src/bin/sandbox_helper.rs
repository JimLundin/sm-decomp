fn main() {
    if let Err(error) = translation_execution::confinement::helper_main() {
        eprintln!("{error:#}");
        std::process::exit(125);
    }
}
