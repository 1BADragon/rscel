
default:
	cargo build

test:
	cargo test

wasm-binding:
	wasm-pack build --target web --features wasm
	
wasm-binding-release:
	wasm-pack build --target web --release --features wasm

python-binding:
	source .env/bin/activate && maturin build --features python
	
python-binding-release:
	source .env/bin/activate && maturin build --features python --release

wasm-example: wasm-binding
	cd examples/wasm && npm start