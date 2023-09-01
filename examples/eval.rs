use rscel::{BindContext, CelContext};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut context = CelContext::new();
    let mut exec = BindContext::new();

    context.add_program_str("prog", &args[1]).unwrap();

    if args.len() > 2 {
        exec.bind_params_from_json_obj(args[2].parse().unwrap())
            .unwrap();
    }

    let res = context.exec("prog", &exec).unwrap();
    println!("{}", res);
}
