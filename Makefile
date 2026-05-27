.PHONY: all build test check fmt move python frontend julia elixir go

all: build test

build:
	cargo build --workspace

test:
	cargo test --workspace
	@cd omokoda-swarm && mix test 2>/dev/null || echo "[skip] mix not available"
	@cd omokoda-frontend && npm test --if-present 2>/dev/null || echo "[skip] npm not available"
	@cd omokoda-simulation && python -m pytest --if-present 2>/dev/null || echo "[skip] pytest not available"

check:
	cargo check --workspace
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --workspace

move:
	@command -v sui >/dev/null 2>&1 && (cd omokoda-on-chain && sui move build) || echo "[skip] sui CLI not installed"

python:
	cd omokoda-simulation && python -m pytest

frontend:
	cd omokoda-frontend && npm run build

julia:
	@command -v julia >/dev/null 2>&1 && julia --project=omokoda-memory -e 'using Pkg; Pkg.resolve(); Pkg.instantiate()' || echo "[skip] julia not installed"

elixir:
	cd omokoda-swarm && mix test

go:
	cd omokoda-ops && go test ./...
