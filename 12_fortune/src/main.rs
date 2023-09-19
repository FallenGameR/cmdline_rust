fn main() {
    if let Err(error) = fortune::get_args().and_then(fortune::run) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
