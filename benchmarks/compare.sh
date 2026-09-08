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

echo 'Batched editor comparison: this crate vs npm dotenv-extra 0.10.0'
hyperfine --warmup 3 --runs 10 \
  --command-name 'this Rust editor' "../target/release/dotenv-bench-driver extra fixture.env $iterations" \
  --command-name 'npm dotenv-extra' "node node-runner.mjs extra fixture.env $iterations"
