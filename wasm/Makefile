wasm-binding:
	RUSTFLAGS="-C opt-level=s" wasm-pack build --target web $(CARGO_ARGS)
	
wasm-binding-release:
	RUSTFLAGS="-C opt-level=s" wasm-pack build --target web --release $(CARGO_ARGS)

wasm-example: wasm-binding
	cd example && npm start

wasm-example-release: wasm-binding-release
	cd example && npm start
