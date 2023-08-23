fn main() {
    if let Err(error) = cut::get_args().and_then(cut::run) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
