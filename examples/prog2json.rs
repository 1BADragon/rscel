use std::env;

use rscel::Program;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: prog2json <program>");
        return;
    }

    let prog = Program::from_source(&args[1]).unwrap();
    println!("{}", serde_json::to_string_pretty(&prog).unwrap());
}
