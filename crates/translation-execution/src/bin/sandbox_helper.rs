fn main() {
    let result = match std::env::args().nth(1).as_deref() {
        Some("--operation-runner") => translation_execution::operation::runner_main(),
        Some("--model-operation-runner") => translation_execution::model_rewrite::runner_main(),
        _ => translation_execution::confinement::helper_main(),
    };
    if let Err(error) = result {
        eprintln!("{error:#}");
        std::process::exit(125);
    }
}
