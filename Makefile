CARGO_ARGS?=

default:
	cargo build $(CARGO_ARGS)

test:
	cargo test $(CARGO_ARGS)

test-all: test
	cargo test --no-default-features $(CARGO_ARGS)

wasm-binding:
	wasm-pack build --target web --features wasm $(CARGO_ARGS)
	
wasm-binding-release:
	wasm-pack build --target web --release --features wasm $(CARGO_ARGS)

python-binding:
	source .env/bin/activate && maturin build --features python $(CARGO_ARGS)
	
python-binding-release:
	source .env/bin/activate && maturin build --features python --release $(CARGO_ARGS)

wasm-example: wasm-binding
	cd examples/wasm && npm start