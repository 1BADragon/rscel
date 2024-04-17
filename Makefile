CARGO_ARGS?=
PYTEST_ARGS?=

.PHONY: default
default: build

build:
	cargo build $(CARGO_ARGS)


.env:
	python3 -m venv .env
	. .env/bin/activate && pip install maturin pytest protobuf

wasm-binding:
	$(MAKE_COMMAND) -C wasm wasm-binding
	
wasm-binding-release:
	$(MAKE_COMMAND) -C wasm wasm-binding-release

python-binding: .env
	. .env/bin/activate && cd python && maturin build $(CARGO_ARGS)
	
python-binding-release: .env
	. .env/bin/activate && cd python && maturin build --release $(CARGO_ARGS)

wasm-example:
	$(MAKE_COMMAND) -C wasm wasm-example

wasm-example-release:
	$(MAKE_COMMAND) -C wasm wasm-example-release

.PHONY: all
all: wasm-binding python-binding build

run-tests:
	cargo test -q $(CARGO_ARGS)

run-no-feature-tests:
	cargo test -q --no-default-features $(CARGO_ARGS)

run-python-tests: .env python-binding
	. .env/bin/activate && \
	pip install --force-reinstall target/wheels/$(shell ls target/wheels) && \
	cd test && \
	python -m pytest test_rscel.py test_cel_spec.py $(PYTEST_ARGS)
	
.PHONY: run-all-tests
run-all-tests: run-tests run-no-feature-tests run-python-tests
