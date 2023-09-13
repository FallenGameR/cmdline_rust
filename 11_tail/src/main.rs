fn main() {
    if let Err(error) = tail::get_args().and_then(tail::run) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
