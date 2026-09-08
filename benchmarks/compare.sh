#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
iterations="${1:-100}"

command -v hyperfine >/dev/null || { echo 'hyperfine is required: https://github.com/sharkdp/hyperfine' >&2; exit 1; }
npm install --ignore-scripts
npm run generate -- 10000
cargo build --release --manifest-path ../Cargo.toml --bin dotenv-bench-driver

echo 'Rust library comparison: this crate vs dotenv 0.15.0 vs dotenv-ng 1.0.0'
cargo bench --manifest-path ../Cargo.toml --bench parser

echo 'Batched parser comparison: this crate vs both npm dotenv parser variants'
hyperfine --warmup 3 --runs 10 \
  --command-name 'this Rust parser' "../target/release/dotenv-bench-driver parse fixture.env $iterations" \
  --command-name 'npm dotenv regex' "node node-runner.mjs parse fixture.env $iterations" \
  --command-name 'npm dotenv fast' "node node-runner.mjs parse-fast fixture.env $iterations"

echo 'Batched parser + expansion comparison: this crate vs npm dotenv + dotenv-expand'
hyperfine --warmup 3 --runs 10 \
  --command-name 'this Rust parse + expand' "../target/release/dotenv-bench-driver expand fixture.env $iterations" \
  --command-name 'npm dotenv + dotenv-expand' "node node-runner.mjs expand fixture.env $iterations"
