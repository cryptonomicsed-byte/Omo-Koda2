.PHONY: all build test check move python frontend julia lisp elixir go clean

# Default: build + test Rust core (the always-required foundation)
all: build test

# --- Rust core ---

build:
	cargo build --workspace

test:
	cargo test --workspace

check:
	cargo check --workspace
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --workspace

# --- Move contracts ---

move:
	@if command -v sui >/dev/null 2>&1; then \
		for pkg in omokoda-sui omokoda-on-chain; do \
			if [ -f "$$pkg/Move.toml" ]; then \
				echo "Building $$pkg..."; \
				(cd $$pkg && sui move build); \
			fi; \
		done; \
	else \
		echo "Sui CLI not installed — skipping Move build."; \
	fi

# --- Python executor ---

python:
	@if [ -f omokoda-simulation/requirements.txt ]; then \
		pip install -r omokoda-simulation/requirements.txt --quiet; \
	fi
	python3 -m py_compile omokoda-simulation/executor.py omokoda-simulation/simulation.py
	@if command -v pytest >/dev/null 2>&1; then \
		cd omokoda-simulation && python -m pytest; \
	fi

# --- Frontend ---

frontend:
	cd omokoda-frontend && npm ci && npm run build

frontend-check:
	cd omokoda-frontend && npm run type-check && npm run lint

# --- Julia ---

julia:
	@if command -v julia >/dev/null 2>&1; then \
		if [ -d omokoda-julia ]; then \
			julia --project=omokoda-julia -e 'using Pkg; Pkg.instantiate()'; \
			julia --project=omokoda-julia test/runtests.jl; \
		else \
			echo "omokoda-julia not found — skipping."; \
		fi; \
	else \
		echo "Julia not installed — skipping."; \
	fi

julia-build:
	@if command -v julia >/dev/null 2>&1 && [ -d omokoda-julia ]; then \
		julia --project=omokoda-julia omokoda-julia/build.jl; \
	fi

# --- Lisp ethics engine ---

lisp:
	@if command -v sbcl >/dev/null 2>&1; then \
		if [ -d omokoda-lisp ]; then \
			sbcl --noinform --load omokoda-lisp/ethics.lisp \
			     --load omokoda-lisp/consent_rules.lisp \
			     --load omokoda-lisp/policy_ast.lisp \
			     --load omokoda-lisp/tests/ethics_tests.lisp \
			     --eval '(quit)'; \
		else \
			echo "omokoda-lisp not found — skipping."; \
		fi; \
	else \
		echo "SBCL not installed — skipping."; \
	fi

# --- Elixir swarm ---

elixir:
	@if command -v mix >/dev/null 2>&1; then \
		if [ -d omokoda-elixir ]; then \
			cd omokoda-elixir && mix deps.get && mix test; \
		else \
			echo "omokoda-elixir not found — skipping."; \
		fi; \
	else \
		echo "Elixir not installed — skipping."; \
	fi

# --- Go flow control ---

go:
	@if command -v go >/dev/null 2>&1; then \
		if [ -d omokoda-go ]; then \
			cd omokoda-go && go test ./...; \
		else \
			echo "omokoda-go not found — skipping."; \
		fi; \
	else \
		echo "Go not installed — skipping."; \
	fi

go-build:
	@if command -v go >/dev/null 2>&1 && [ -d omokoda-go ]; then \
		cd omokoda-go && go build ./...; \
	fi

# --- Full integration ---

integration: build test python frontend-check julia go elixir lisp move

# --- Cleanup ---

clean:
	cargo clean
	@[ -d omokoda-julia/lib ] && rm -rf omokoda-julia/lib || true
	@[ -d omokoda-go/bin ] && rm -rf omokoda-go/bin || true
