# dotenv-rs-fast

A fast Rust dotenv parser, variable expander, loader, and command runner. It
targets npm [`dotenv` 17.4.2](https://github.com/motdotla/dotenv/tree/v17.4.2)
and npm [`dotenv-expand` 1000.0.0](https://github.com/dotenvx/dotenv-expand/tree/v1000.0.0).

The parser uses a single-pass byte scanner without regular expressions. Parsed
entries retain insertion order because dotenv-expand applies values
progressively and ordering is observable.

## Library

```rust
use dotenv::{expand, parse};

let parsed = parse("HOST=localhost\nURL=https://${HOST}/api");
let values = expand(&parsed)?;
assert_eq!(values["URL"], "https://localhost/api");
# Ok::<(), dotenv::ExpandError>(())
```

Parsing supports comments, `export`, `=` and whitespace-followed `:`
separators, UTF-8 BOMs, CR/LF/CRLF, single/double/backtick quotes, multiline
quoted values, and escaped newlines in double quotes.

Expansion supports `$NAME`, `${NAME}`, default and alternate operators,
required operators, nesting, progressive expansion, escaped dollar signs, and
`$(command)` substitution. Command substitution executes with the user's shell
and must only be enabled for trusted files. It can be disabled through
`ExpandOptions` or the CLI. Dotenvx `encrypted:` values are decrypted before
expansion when `DOTENV_PRIVATE_KEY` is present and remain unchanged without it.

## CLI

Install from a checkout:

```bash
cargo install --path . --bin dotenv
```

Run commands or print the resulting entries:

```bash
dotenv -- cargo run
dotenv --file .env.production --override -- ./server
dotenv --file .env.production
```

```text
-f, --file PATH                Load PATH; repeat to layer files
    --override                 File values replace inherited variables
    --no-expand                Disable variable expansion
    --no-command-substitution  Disable $(command) evaluation
```

Tagged [GitHub releases](https://github.com/imranbarbhuiya/dotenv/releases)
provide binaries for Linux, Intel/Apple Silicon macOS, and Windows. Extract the
appropriate archive and place `dotenv` (or `dotenv.exe`) on `PATH`.

## Tests

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo bench
```

Criterion covers library parsing and expansion. See
[`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) for current methodology and
measurements.

## CLI benchmarks

The reproducible end-to-end suite compares this CLI with Rust `dotenv` 0.15.0,
Rust `dotenv-ng` 1.0.0, npm `dotenv` 17.4.2, and npm `dotenv-expand` 1000.0.0:

```bash
./benchmarks/compare.sh
```

Every timed command loads the same literal-only fixture and launches the same
child verifier. The harness checks every value and requires identical complete
output before timing. Baselines without a command-runner CLI are clearly marked
as wrappers; the suite makes no claim that all five have equivalent expansion
semantics. It exports raw results and machine metadata, and the `CLI benchmarks`
workflow ([runs and artifacts](https://github.com/imranbarbhuiya/dotenv/actions/workflows/benchmark.yml))
publishes them as an artifact. See
[`benchmarks/README.md`](benchmarks/README.md) for scope and reproduction details,
and [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) for provenance and
licenses.

### Latest CI benchmark results

[GitHub Actions run #34228346224](https://github.com/imranbarbhuiya/dotenv/actions/runs/34228346224)
on September 8, 2026, at commit
[`e45085e`](https://github.com/imranbarbhuiya/dotenv/commit/e45085e40c49a854c42abdb97b786c769983a421).
The clean Ubuntu runner used an AMD EPYC 7763 with four logical CPUs, Rust
1.93.0, Node.js 22.19.0, and Hyperfine 1.19.0. Each command loaded 1,000 literal
entries and launched the shared verifier. All five passed the complete-value and
byte-for-byte output checks before timing. Results use 20 measured runs after
three warmups.

| Implementation | Mean ± standard deviation | Min | Max | Relative |
| --- | ---: | ---: | ---: | ---: |
| **dotenv-rs-fast (native CLI)** | **4.6 ± 0.1 ms** | **4.4 ms** | **4.9 ms** | **1.00×** |
| Rust dotenv 0.15.0 (benchmark wrapper) | 4.7 ± 0.2 ms | 4.5 ms | 5.0 ms | 1.01 ± 0.04× |
| Rust dotenv-ng 1.0.0 (native CLI) | 5.1 ± 0.2 ms | 4.9 ms | 5.6 ms | 1.10 ± 0.05× |
| npm dotenv 17.4.2 (benchmark wrapper) | 49.8 ± 2.6 ms | 47.2 ms | 56.8 ms | 10.75 ± 0.62× |
| npm dotenv-expand 1000.0.0 + dotenv parser (benchmark wrapper) | 228.8 ± 6.5 ms | 219.7 ms | 246.6 ms | 49.40 ± 1.85× |

The workflow uploads the raw Hyperfine JSON, generated Markdown table, and full
machine metadata as its `cli-benchmark-e45085e...` artifact. Run
`./benchmarks/compare.sh` to reproduce the same benchmark locally.
