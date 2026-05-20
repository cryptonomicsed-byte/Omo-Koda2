#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

sudo apt-get update -q
wget -q https://packages.erlang-solutions.com/erlang-solutions_2.0_all.deb
sudo dpkg -i erlang-solutions_2.0_all.deb
rm erlang-solutions_2.0_all.deb
sudo apt-get update -q
sudo apt-get install -y --no-install-recommends esl-erlang elixir curl ca-certificates wget

# Julia 1.10
JULIA_VERSION=1.10.4
curl -fsSL "https://julialang-s3.julialang.org/bin/linux/x64/1.10/julia-${JULIA_VERSION}-linux-x86_64.tar.gz" \
  | sudo tar -xz -C /usr/local --strip-components=1

# Python tool runner deps
if command -v python3 >/dev/null 2>&1; then
  python3 -m pip install --upgrade pip
  if [ -f "omokoda-simulation/requirements.txt" ]; then
    python3 -m pip install -r omokoda-simulation/requirements.txt --quiet
  fi
fi

# Julia memory module deps, if present
if [ -d "omokoda-memory" ]; then
  julia --project=omokoda-memory -e 'using Pkg; Pkg.instantiate()' || true
fi

# Node frontend deps
if [ -f "omokoda-frontend/package.json" ]; then
  (cd omokoda-frontend && npm install --legacy-peer-deps) || true
fi

# Sui CLI installation is intentionally commented out because it may take a long time.
# To install manually, uncomment the line below.
# cargo install --locked sui

echo "Omo-Koda2 dev environment ready"
