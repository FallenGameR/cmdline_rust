use tail::count_lines;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let path = &args[1];
    let count = count_lines(path).expect("should be valid");

    dbg!(count);
}
