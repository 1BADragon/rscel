use rscel::Program;
use std::env;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        panic!("No program passed")
    }

    let p = Program::from_source(&args[1]).expect("Failed to compile program");
    for bc in p.bytecode() {
        println!("{:?}", bc);
    }
}
