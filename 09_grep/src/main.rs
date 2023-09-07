fn main() {
    if let Err(error) = grep::get_args().and_then(grep::run) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
