.PHONY: all build serve server test lint fmt clean

all: build
	@-pkill scrib-server 2>/dev/null; sleep 0.5
	@echo "=== Starting server (HTTP + WebSocket on :9876) ==="
	@cargo run -p scrib-server

build:
	wasm-pack build --target web --features wasm
	cp pkg/* www/pkg/

serve: build
	cargo run -p scrib-server

server:
	cargo run -p scrib-server

test:
	cargo test

lint:
	cargo clippy --all-targets

fmt:
	cargo fmt

clean:
	cargo clean
	rm -rf pkg/
