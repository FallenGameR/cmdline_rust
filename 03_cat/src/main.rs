fn main() {
    if let Err(e) = cat::run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
