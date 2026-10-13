fn main() {
    let result = if std::env::args().nth(1).as_deref() == Some("--operation-runner") {
        translation_execution::operation::runner_main()
    } else {
        translation_execution::confinement::helper_main()
    };
    if let Err(error) = result {
        eprintln!("{error:#}");
        std::process::exit(125);
    }
}
