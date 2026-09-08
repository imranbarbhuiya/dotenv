# dotenv-rs-fast

A clean-room Rust implementation of the behavior exposed by npm
[`dotenv` 17.4.2](https://github.com/motdotla/dotenv/tree/v17.4.2), plus the
formatting-preserving editor features from npm
[`dotenv-extra` 0.10.0](https://github.com/kermage/dotenv-extra/tree/v0.10.0).

The hot parser is a single-pass byte scanner. It has no regular expressions and
the library has no runtime dependencies. The editor builds a key-to-line index
once, making updates O(1) while preserving comments, blank lines, line endings,
and the presence or absence of the final newline.

## Parse and load

```rust
use dotenv::{load_from_path, parse};

let values = parse("HOST=localhost\nPORT=3000");
assert_eq!(values["HOST"], "localhost");

let values = load_from_path(".env")?;
# Ok::<(), std::io::Error>(())
```

Supported npm-dotenv syntax includes `export`, `=` and whitespace-followed `:`
separators, LF/CRLF/CR files, UTF-8 BOMs, comments, single/double/backtick
quotes, physical multiline quoted values, and `\\n`/`\\r` expansion inside
double quotes. Duplicate keys use the last value.

`load()` searches the current directory and its ancestors for `.env` without
mutating the process environment. `populate()` merges into an explicit map with
an opt-in override policy. Avoiding hidden global environment mutation keeps the
API safe in multithreaded Rust programs.

## Edit without destroying formatting

```rust
use dotenv::DotenvEditor;

let mut file = DotenvEditor::open(".env")?;
file.upsert("APP_ENV", "production")?
    .upsert("MESSAGE", "hello # world")?;
file.save()?;
# Ok::<(), dotenv::EditorError>(())
```

The editor matches dotenv-extra's active-vs-commented lookup, last-duplicate
update, safe quoting, reactivation, CR/LF/CRLF preservation, and newline
injection rejection. Files with mixed line endings are rejected rather than
silently rewritten.

## Tests

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Compatibility tests are adapted from all four upstream projects used as
references. npm-specific cases include BOM handling, multiline quotes,
backticks, colon assignments, inline comments, formatting round trips,
reactivation, duplicate handling, and 100,000-backslash linear-time guards.
See [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) for exact versions,
commits, and licenses.

## Benchmarks

| Harness | Implementations | Measurement |
| --- | --- | --- |
| Criterion | this crate, Rust `dotenv` 0.15.0, Rust `dotenv-ng` 1.0.0 | identical persistent fixture; open + read + parse + allocate |
| Hyperfine batch runner | this crate, npm `dotenv` 17.4.2 regex, npm `dotenv` fast scanner | identical preloaded bytes parsed repeatedly per process |
| Hyperfine batch runner | this editor, npm `dotenv-extra` 0.10.0 | open + parse + update + dump + render repeatedly per process |

Run Rust-only benchmarks:

```bash
cargo bench
```

Run the complete Rust/npm comparison (requires Node.js 24+ and `hyperfine`):

```bash
cd benchmarks
./compare.sh 100
```

The repetition count is passed to each batch runner, amortizing Node and Rust
process startup rather than presenting startup time as parser time. The fixture
is generated locally and is not committed.
