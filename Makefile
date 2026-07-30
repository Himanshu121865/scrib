.PHONY: all build serve server test lint fmt clean

all: build
	@pkill -f "scrib-server" 2>/dev/null; pkill -f "http.server" 2>/dev/null; sleep 0.5
	@echo "=== Starting WebSocket server (background) ==="
	@cargo run -p scrib-server &
	@sleep 1
	@echo "=== Starting HTTP server ==="
	@cd www && python3 -m http.server 8080; EC=$$?; \
		kill %1 2>/dev/null; exit $$EC

build:
	wasm-pack build --target web --features wasm
	cp pkg/* www/pkg/

serve:
	cd www && python3 -m http.server 8080

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
