use std::env;

use rscel::{CelCompiler, StringTokenizer};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: prog2json <program>");
        return;
    }

    let mut tokenizer = StringTokenizer::with_input(&args[1]);
    let prog = CelCompiler::with_tokenizer(&mut tokenizer)
        .compile()
        .unwrap();
    println!("{}", serde_json::to_string_pretty(&prog).unwrap());
}
