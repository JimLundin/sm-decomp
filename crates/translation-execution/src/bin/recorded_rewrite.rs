fn main() {
    if let Err(error) = translation_execution::rewrite_adapter::recorded_main() {
        eprintln!("{error:#}");
        std::process::exit(125);
    }
}
