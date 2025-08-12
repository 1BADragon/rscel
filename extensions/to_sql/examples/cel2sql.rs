use rscel::Program;
use rscel_to_sql::ToSql;
use std::env;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        panic!("No program passed")
    }

    let p = Program::from_source(&args[1]).expect("Failed to compile program");

    let sql_builder = p
        .ast()
        .unwrap()
        .to_sql()
        .expect("Failed to generate SQL builder");
    let sql = sql_builder.to_sql().expect("Failed to generate SQL");

    println!("{}", sql);
}
