use clap::{arg, Command};

fn main() {
    let mut matches = Command::new("echo")
        .version("1.0")
        .author("FallenGameR")
        .about("Prints arguments to the standard output")
        .args([
            arg!(<text> ... "Input text"),
            arg!(-n --no_newline "Do not print newline")
        ])
        .get_matches();

    let text: Vec<String> = matches.remove_many("text").expect("No text provided").collect();
    let newline = !matches.get_flag("no_newline");

    let ending = if newline { "\n" } else { "" };
    print!("{}{}", text.join(" "), ending);
}
