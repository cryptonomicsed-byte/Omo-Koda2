#!/usr/bin/env bash
# Omo-Koda2 devcontainer post-create setup.
# Installs the three runtimes the universal base image does NOT include:
# Elixir/Erlang, Julia, and optionally Sui CLI.
set -euo pipefail

echo "==> Installing Erlang + Elixir..."
wget -q https://packages.erlang-solutions.com/erlang-solutions_2.0_all.deb
sudo dpkg -i erlang-solutions_2.0_all.deb
rm erlang-solutions_2.0_all.deb
sudo apt-get update -q
sudo apt-get install -y --no-install-recommends esl-erlang elixir

echo "==> Installing Julia 1.10..."
JULIA_VERSION=1.10.4
curl -fsSL \
  "https://julialang-s3.julialang.org/bin/linux/x64/1.10/julia-${JULIA_VERSION}-linux-x86_64.tar.gz" \
  | sudo tar -xz -C /usr/local --strip-components=1

echo "==> Installing Python tool runner deps..."
pip install -r omokoda-simulation/requirements.txt --quiet

echo "==> Initialising Julia memory module deps..."
julia --project=omokoda-memory -e 'using Pkg; Pkg.instantiate()' || true

echo "==> Installing Node deps for frontend..."
(cd omokoda-frontend && npm install --legacy-peer-deps) || true

echo "==> Installing Elixir deps for swarm..."
(cd omokoda-swarm && mix local.hex --force && mix local.rebar --force && mix deps.get) || true

# Sui CLI: compile time ~20 min — uncomment to enable in Codespace.
# echo "==> Installing Sui CLI..."
# cargo install --locked --git https://github.com/MystenLabs/sui sui || true

echo ""
echo "Omo-Koda2 dev environment ready."
echo "  Rust:   $(rustc --version 2>/dev/null || echo 'not found')"
echo "  Go:     $(go version 2>/dev/null || echo 'not found')"
echo "  Elixir: $(elixir --version 2>/dev/null | head -1 || echo 'not found')"
echo "  Julia:  $(julia --version 2>/dev/null || echo 'not found')"
echo "  Python: $(python3 --version 2>/dev/null || echo 'not found')"
echo "  Node:   $(node --version 2>/dev/null || echo 'not found')"
