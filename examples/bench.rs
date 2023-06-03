use chrono::prelude::*;
use rscel::{CelContext, ExecContext};

type BenchmarkFn = fn();

const BENCHMARKS: &[(&str, BenchmarkFn)] = &[
    ("Run One No Binding", bench_run_one_nobindings),
    ("Run Many No Binding", bench_run_many_no_bindings),
    ("Run One With Binding", bench_run_one_with_binding),
    ("Run Many With Bindings", bench_run_one_with_many_bindings),
    ("Build Many", bench_build_many),
    (
        "Build Many With Bindings",
        bench_construct_many_with_bindings,
    ),
];

fn main() {
    for benchmark in BENCHMARKS.iter() {
        let start_time = Local::now();
        benchmark.1();
        let end_time = Local::now();

        println!("{}: {}", benchmark.0, end_time - start_time);
    }
}

fn bench_run_one_nobindings() {
    let mut cel = CelContext::new();
    let exec = ExecContext::new();

    cel.add_program_str("entry", "((4 * 3) - 4) + 3").unwrap();

    cel.exec("entry", &exec).unwrap();
}

fn bench_run_many_no_bindings() {
    let mut cel = CelContext::new();
    let exec = ExecContext::new();

    cel.add_program_str("entry", "((4 * 3) - 4) + 3").unwrap();

    for _ in 0..10_000 {
        cel.exec("entry", &exec).unwrap();
    }
}

fn bench_run_one_with_binding() {
    let mut cel = CelContext::new();
    let mut exec = ExecContext::new();

    cel.add_program_str("entry", "((4 * 3) - foo) + 3").unwrap();
    exec.bind_param("foo", 6.into());

    cel.exec("entry", &exec).unwrap();
}

fn bench_run_one_with_many_bindings() {
    let mut cel = CelContext::new();
    let mut exec = ExecContext::new();

    cel.add_program_str("entry", "((4 * 3) - foo) + 3").unwrap();

    for o in 0..10_000 {
        exec.bind_param("foo", o.into());

        cel.exec("entry", &exec).unwrap();
    }
}

fn bench_build_many() {
    let mut cel = CelContext::new();

    for o in 0..1_000 {
        cel.add_program_str(&format!("prog{}", o), &format!("((4 * 3) - {}) + 3", o))
            .unwrap();
    }
}

fn bench_construct_many_with_bindings() {
    for o in 0..10_000 {
        let mut cel = CelContext::new();
        let mut exec = ExecContext::new();

        cel.add_program_str("entry", "((4 * 3) - foo) + 3").unwrap();
        exec.bind_param("foo", o.into());

        cel.exec("entry", &exec).unwrap();
    }
}
