fn main() {
    if let Err(error) = cal::get_args().and_then(cal::run) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
