# CLI benchmarks

This suite measures one end-to-end operation: load a generated dotenv file containing portable literal assignments, put those values in a child process environment, and run the shared compiled verifier. The verifier reads and validates every generated value on every invocation. Before timing, the harness also requires the complete stdout of all five commands to match byte for byte. Expansion remains enabled where a candidate supports it, though the portable fixture does not rely on expansion.

The comparison includes:

- this project's native `dotenv` CLI;
- Rust `dotenv` 0.15.0 through the small `dotenv015-wrapper` in this directory (the crate has no equivalent command-runner CLI);
- Rust `dotenv-ng` 1.0.0's native CLI, built with its `cli` feature;
- npm `dotenv` 17.4.2 through `node-wrapper.mjs` (the package is a library);
- npm `dotenv-expand` 1000.0.0 through `node-wrapper.mjs`, composed with the pinned npm `dotenv` parser (the package expands parsed data and is not a standalone parser or CLI).

The fixture intentionally has no interpolation. That gives every candidate the same semantics and makes this a CLI load-and-launch comparison; it is not a claim that all packages implement the same expansion language. Wrapper startup and command launch are included because the measured unit is the command users invoke. On Unix, this CLI, the Rust wrapper, and `dotenv-ng` use `exec`; the Node wrappers spawn and wait according to Node's process API.

## Reproduce

Install Rust, Node.js, npm, and [Hyperfine](https://github.com/sharkdp/hyperfine), then run from the repository root:

```bash
BENCH_ENTRIES=1000 BENCH_RUNS=20 BENCH_WARMUP=3 ./benchmarks/compare.sh
```

Rust and npm package versions are locked in `rust-wrapper/Cargo.lock` and `npm-shrinkwrap.json`; the project CLI is built with the repository `Cargo.lock`. The script installs exactly `dotenv-ng` 1.0.0 with its published lockfile. Generated files stay ignored.

Outputs are written to `benchmarks/results/`:

- `results.json`: raw Hyperfine measurements;
- `results.md`: Hyperfine's comparison table;
- `metadata.json`: commit/dirty state, OS, CPU, memory, tool versions, fixture size, and run counts.

Timing varies across machines, power modes, and runner images. Compare candidates within a single result file rather than comparing absolute timings from different hosts. The [GitHub Actions benchmark workflow](https://github.com/imranbarbhuiya/dotenv/actions/workflows/benchmark.yml) uses a fixed runner/toolchain setup and publishes all three files as an artifact.
