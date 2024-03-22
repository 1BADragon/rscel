CARGO_ARGS?=

.PHONY: default
default: build

build:
	cargo build $(CARGO_ARGS)


.env:
	python3 -m venv .env
	. .env/bin/activate && pip install maturin && pip install pytest

wasm-binding:
	RUSTFLAGS="-C opt-level=s" wasm-pack build --target web --features wasm $(CARGO_ARGS)
	
wasm-binding-release:
	RUSTFLAGS="-C opt-level=s" wasm-pack build --target web --release --features wasm $(CARGO_ARGS)

python-binding: .env
	. .env/bin/activate && maturin build --features python $(CARGO_ARGS)
	
python-binding-release: .env
	. .env/bin/activate && maturin build --features python --release $(CARGO_ARGS)

wasm-example: wasm-binding
	cd examples/wasm && npm start

wasm-example-release: wasm-binding-release
	cd examples/wasm && npm start

.PHONY: all
all: wasm-binding python-binding build

run-tests:
	cargo test -q $(CARGO_ARGS)

run-no-feature-tests:
	cargo test -q --no-default-features $(CARGO_ARGS)

run-python-tests: .env python-binding
	. .env/bin/activate && \
	pip install --force-reinstall target/wheels/$(shell ls target/wheels) && \
	python -m pytest test/
	
.PHONY: run-all-tests
run-all-tests: run-tests run-no-feature-tests run-python-tests
