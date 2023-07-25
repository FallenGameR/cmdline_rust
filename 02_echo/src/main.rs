use clap::{arg, Command};

fn main() {
    let matches = Command::new("echo")
        .version("1.0")
        .author("FallenGameR")
        .about("Prints arguments to the standard output")
        .args(&[
            arg!(<TEXT> ... "Input text"),
            arg!(-n --newline "Do not print newline")
        ])
        .get_matches();

    println!("{:#?}", matches);
}
