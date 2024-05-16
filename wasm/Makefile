wasm-binding:
	RUSTFLAGS="-C opt-level=s" wasm-pack build --target web $(CARGO_ARGS)

wasm-binding-release:
	RUSTFLAGS="-C opt-level=s" wasm-pack build --target web --release $(CARGO_ARGS)

wasm-example: wasm-binding
	cd example && npm start

wasm-example-release: wasm-binding-release
	cd example && npm start

wasm-tests:
	wasm-pack build -t nodejs
	cd tests && npm install && npm test

.PHONY: wasm-binding wasm-binding-release wasm-example wasm-example-release wasm-tests
