# Third-party test provenance

The production code in this repository is a clean-room Rust implementation.
Compatibility tests and fixtures were adapted from these upstream projects:

- npm `dotenv` 17.4.2, commit `f116f70310abab44fbfddbaeb833698b5bf84a9b`,
  BSD-2-Clause, Copyright (c) 2015 Scott Motte. The full upstream license is
  retained at `tests/fixtures/npm-dotenv/LICENSE`.
- npm `dotenv-extra` 0.10.0, commit
  `a2d2a72c77c37eef549119eaa0edb51fb5d1f82d`, MIT, Copyright (c) 2024 Gene
  Alyson Fortunado Torcende.
- Rust `dotenv` 0.15.0 from `dotenv-rs/dotenv`, MIT.
- Rust `dotenv-ng` 1.0.0 from `cachix/dotenv-ng`, MIT.

The benchmark dependencies remain under their respective upstream licenses and
are fetched by Cargo or npm; they are not redistributed as part of this source.

