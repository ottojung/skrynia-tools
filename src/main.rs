fn main() {
    if let Err(error) = skrynia_tools::cli::run() {
        eprintln!("skrynia: {error:#}");
        std::process::exit(1);
    }
}
