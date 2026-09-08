# Performance work

The parser uses `memchr` byte searches for line endings, comments, and the
no-carriage-return fast path. Expansion skips the scanner for literal values,
borrows already-resolved references, reuses recursion scratch storage, and avoids
the command-substitution output allocation when no command is present. The CLI
reuses the first loaded map, buffers printed output, and skips collecting the
process environment when expansion is disabled. On Unix, it replaces itself with
the child using `exec`, avoiding an idle parent process and preserving exit signals.

These changes retain randomized map hashing and existing ownership/API semantics.
They do not trade collision resistance for attractive benchmark numbers.

## Local before/after measurement

Baseline: `e1c43e9a2bcae26729258aa2d4c89300f012098e`. After: the performance changes
in this working tree. Host: Apple M3, macOS 27.0 (26A5425a), rustc 1.93.0,
`aarch64-apple-darwin`. Criterion 0.7.0, release/bench profile with thin LTO and one
codegen unit; 30 samples, 1-second warmup, 2-second measurement per case.

The existing 1,000-entry fixture actually includes two additional entries. Every
fourth generated entry references a previous value; it also includes comments,
escaped newlines, and a default expression. Expansion uses an empty inherited
environment with command substitution disabled. File reads are warm-cache.

| Operation | Before mean | After mean | Mean time reduction |
| --- | ---: | ---: | ---: |
| Parse only | 110.14 µs | 82.45 µs | 25.1% |
| Read + parse | 135.93 µs | 109.89 µs | 19.2% |
| Read + parse + expand | 268.04 µs | 172.11 µs | 35.8% |

These numbers are **provisional, not a performance guarantee**. All three
differences were significant in this run under Criterion's default test, but
statistical significance does not correct for changes in host load. A subsequent
10/10,000-entry sweep showed mixed results, including regressions; inspection
found an unrelated browser process consuming approximately six CPU cores. The
host was not isolated, and outliers occurred. Repeat measurements on an idle
machine before treating these ratios as representative or ruling out regressions.
These are library measurements, **not** CLI startup times or a claim of being
fastest on every input. The separate
[CLI suite](../benchmarks/README.md) compares complete process invocations.

To reproduce before/after locally from the repository root (POSIX shell):

```sh
benchmark_root="$PWD"
benchmark_before=$(mktemp -d)
git archive e1c43e9a2bcae26729258aa2d4c89300f012098e | tar -x -C "$benchmark_before"
export CARGO_TARGET_DIR="$benchmark_root/target/performance-comparison"
export CRITERION_HOME="$benchmark_root/target/performance-results"
cargo bench --manifest-path "$benchmark_before/Cargo.toml" --locked --bench parser -- \
  'parse/1000_entries/this' --sample-size 30 --warm-up-time 1 \
  --measurement-time 2 --save-baseline original
cargo bench --locked --bench parser -- \
  'parse/1000_entries/this' --sample-size 30 --warm-up-time 1 \
  --measurement-time 2 --baseline original
```

Run sequentially without other benchmarks or builds. Criterion writes raw samples,
estimates, and HTML reports under `target/performance-results`. Use the same compiler
and build flags for both revisions. The temporary baseline checkout is intentionally
left available for inspection.

## Further opportunities

Measure before changing these: a consuming expansion API could reuse the parsed
strings; borrowed parsing could reduce key/value allocations for callers that
retain the input buffer; quote scanning may benefit long certificate values.
These involve API or workload tradeoffs and are not assumed to be improvements.
CLI process startup and child spawning can dominate small files, so microbenchmark
gains should not be extrapolated to end-to-end speedups.
