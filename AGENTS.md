# Agent Guidelines

## Overview
This repository provides a Rust implementation of the [CEL](https://github.com/google/cel-spec) language with optional Python and WebAssembly bindings.

## Coding Standards
- Use Rust 2021 edition and the nightly toolchain defined in `rust-toolchain.toml`.
- Format all Rust code with `cargo fmt --all` (automatically uses the pinned nightly toolchain).
- Keep module and item documentation using Rust doc comments (`//!` for modules, `///` for items).
- Follow standard Rust naming conventions (`snake_case` for functions/variables, `CamelCase` for types).

## Testing
- Before committing, run the complete test suite with `make run-all-tests`.
- The repository's `rust-toolchain.toml` pins the nightly toolchain, so `cargo test` automatically uses the correct version.
- If your changes affect the Python bindings or tests, rebuild the wheel and run the Python tests:
  - `make run-python-tests`
- For WebAssembly changes, run `make run-wasm-tests`.

## Misc
- The project targets sandboxed, bindable evaluation of user-provided CEL expressions; keep changes consistent with this design.

