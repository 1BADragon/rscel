CARGO_ARGS?=

.PHONY: default
default: build

build:
	cargo build $(CARGO_ARGS)

test:
	cargo test $(CARGO_ARGS)

test-all: test
	cargo test --no-default-features $(CARGO_ARGS)

.env:
	python3 -m venv .env
	source .env/bin/activate && pip install maturin

wasm-binding:
	RUSTFLAGS="-C opt-level=s" wasm-pack build --target web --features wasm $(CARGO_ARGS)
	
wasm-binding-release:
	RUSTFLAGS="-C opt-level=s" wasm-pack build --target web --release --features wasm $(CARGO_ARGS)

python-binding: .env
	source .env/bin/activate && maturin build --features python $(CARGO_ARGS)
	
python-binding-release: .env
	source .env/bin/activate && maturin build --features python --release $(CARGO_ARGS)

wasm-example: wasm-binding
	cd examples/wasm && npm start

wasm-example-release: wasm-binding-release
	cd examples/wasm && npm start

.PHONY: all
all: wasm-binding python-binding build
