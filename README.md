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

Tagged releases build downloadable binaries for Linux, Intel/Apple Silicon
macOS, and Windows. Extract the appropriate archive and place `dotenv` (or
`dotenv.exe`) on `PATH`.

## Tests and benchmarks

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo bench
```

Criterion compares this crate with Rust `dotenv` 0.15.0 and `dotenv-ng`
1.0.0. The external suite adds npm `dotenv`'s regular and fast parsers and npm
`dotenv-expand`:

```bash
cd benchmarks
./compare.sh 100
```

The runners repeat work inside each process so startup does not dominate parser
measurements. See [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) for test
provenance and licenses.
