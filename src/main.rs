fn main() {
    if let Err(error) = frugal::run() {
        eprintln!("{error}");
        std::process::exit(error.exit_code());
    }
}
