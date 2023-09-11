fn main() {
    if let Err(error) = comm::get_args().and_then(comm::run) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
