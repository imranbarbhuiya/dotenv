#!/usr/bin/env bash
set -euo pipefail

bench_dir="$(cd "$(dirname "$0")" && pwd)"
repo_dir="$(cd "$bench_dir/.." && pwd)"
entries="${BENCH_ENTRIES:-1000}"
runs="${BENCH_RUNS:-20}"
warmup="${BENCH_WARMUP:-3}"
results_dir="${BENCH_RESULTS_DIR:-$bench_dir/results}"
tools_dir="$bench_dir/.tools"
root_target="$bench_dir/.build/root-target"
wrapper_target="$bench_dir/.build/wrapper-target"

for value in "$entries" "$runs" "$warmup"; do
  [[ "$value" =~ ^[0-9]+$ ]] || { echo "entry/run counts must be non-negative integers" >&2; exit 2; }
done
(( entries > 0 && runs > 0 )) || { echo "BENCH_ENTRIES and BENCH_RUNS must be positive" >&2; exit 2; }

for tool in cargo node npm hyperfine; do
  command -v "$tool" >/dev/null || { echo "missing required tool: $tool" >&2; exit 1; }
done

mkdir -p "$results_dir" "$tools_dir"
cd "$bench_dir"
npm ci --ignore-scripts
node generate.mjs "$entries"
CARGO_TARGET_DIR="$root_target" cargo build --locked --release --manifest-path "$repo_dir/Cargo.toml" --bin dotenv
CARGO_TARGET_DIR="$wrapper_target" cargo build --locked --release --manifest-path "$bench_dir/rust-wrapper/Cargo.toml"

if [[ ! -x "$tools_dir/bin/dotenv-ng" ]]; then
  cargo install dotenv-ng --version '=1.0.0' --features cli --locked --root "$tools_dir"
fi
"$tools_dir/bin/dotenv-ng" --version | grep -F '1.0.0' >/dev/null

candidate_names=(
  'dotenv-rs-fast (native CLI)'
  'Rust dotenv 0.15.0 (benchmark wrapper)'
  'Rust dotenv-ng 1.0.0 (native CLI)'
  'npm dotenv 17.4.2 (benchmark wrapper)'
  'npm dotenv-expand 1000.0.0 + dotenv parser (benchmark wrapper)'
)
printf -v root_cli '%q' "$root_target/release/dotenv"
printf -v fixture '%q' "$bench_dir/fixture.env"
printf -v wrapper '%q' "$wrapper_target/release/dotenv015-wrapper"
printf -v child '%q' "$wrapper_target/release/verify-child"
printf -v dotenv_ng '%q' "$tools_dir/bin/dotenv-ng"
printf -v node_wrapper '%q' "$bench_dir/node-wrapper.mjs"
printf -v entry_arg '%q' "$entries"
candidate_commands=(
  "$root_cli --file $fixture --override -- $child $entry_arg"
  "$wrapper $fixture $child $entry_arg"
  "$dotenv_ng --file $fixture --override --substitute $child $entry_arg"
  "node $node_wrapper dotenv $fixture $child $entry_arg"
  "node $node_wrapper dotenv-expand $fixture $child $entry_arg"
)

check_dir="$(mktemp -d)"
trap 'rm -rf "$check_dir"' EXIT
printf 'verified %s variables\n' "$entries" > "$check_dir/expected"
for index in "${!candidate_commands[@]}"; do
  bash -c "${candidate_commands[$index]}" > "$check_dir/actual"
  if ! cmp -s "$check_dir/expected" "$check_dir/actual"; then
    echo "correctness check failed for ${candidate_names[$index]}" >&2
    diff -u "$check_dir/expected" "$check_dir/actual" || true
    exit 1
  fi
done
echo "Correctness gate passed: all five commands produced identical complete output after validating all $entries variables."

hyperfine_args=(--shell bash --warmup "$warmup" --runs "$runs" --style basic --export-json "$results_dir/results.json" --export-markdown "$results_dir/results.md")
for index in "${!candidate_commands[@]}"; do
  hyperfine_args+=(--command-name "${candidate_names[$index]}" "${candidate_commands[$index]}")
done
hyperfine "${hyperfine_args[@]}"
node metadata.mjs "$results_dir/metadata.json" "$entries" "$runs" "$warmup"
echo "Results written to $results_dir"
