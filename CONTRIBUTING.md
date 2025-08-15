# Contributing to rscel

Thank you for your interest in contributing!

## Filing Issues

* Check the existing issues to see if your problem has already been reported.
* Open a new issue with a clear description of the problem and steps to reproduce it.
* If you have a feature request, explain the motivation and possible alternatives.

## Running Tests

Before submitting changes, make sure the test suite passes.

```sh
cargo +nightly-2025-08-08 test
cargo +nightly-2025-08-08 test --no-default-features
```

If your change touches the Python bindings or WebAssembly support, also run:

```sh
make run-python-tests
make run-wasm-tests
```

## Submitting Patches

1. Fork the repository and create a topic branch for your changes.
2. Ensure your code is formatted with `cargo fmt --all`.
3. Commit your changes with clear messages.
4. Open a pull request describing your changes and why they are needed.
5. Be ready to address review feedback.

We appreciate your contributions!

