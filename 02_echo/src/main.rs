use clap::Command;

fn main() {
    let _matches = Command::new("echo")
        .version("1.0")
        .author("FallenGameR")
        .about("Prints arguments to the standard output")
        .get_matches();
}
