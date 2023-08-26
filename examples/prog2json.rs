use std::env;

use rscel::CelCompiler;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: prog2json <program>");
        return;
    }

    let prog = CelCompiler::with_input(&args[1]).compile().unwrap();
    println!("{}", serde_json::to_string_pretty(&prog).unwrap());
}
