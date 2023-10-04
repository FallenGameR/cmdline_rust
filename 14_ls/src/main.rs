fn main() {
    if let Err(e) = ls::get_args().and_then(ls::run) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
