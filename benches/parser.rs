use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::io::Write;

fn fixture(entries: usize) -> String {
    let mut text = String::with_capacity(entries * 48);
    for i in 0..entries {
        use std::fmt::Write as _;
        let _ = writeln!(text, "KEY_{i}=value-{i} # comment");
    }
    text.push_str("MULTILINE=\"first\\nsecond\"\nexport LAST_KEY=last\n");
    text
}

fn parser_benchmarks(c: &mut Criterion) {
    for entries in [10, 1_000, 10_000] {
        let input = fixture(entries);
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(input.as_bytes()).unwrap();
        let path = file.path().to_owned();
        let mut group = c.benchmark_group(format!("parse/{entries}_entries"));
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_function("this/read+parse", |b| {
            b.iter(|| black_box(dotenv::load_from_path(black_box(&path)).unwrap()))
        });
        group.bench_function("rust-dotenv-0.15/read+parse", |b| {
            b.iter(|| {
                #[allow(deprecated)]
                let values: Vec<_> = dotenv_upstream::from_path_iter(black_box(&path))
                    .unwrap()
                    .collect::<Result<_, _>>()
                    .unwrap();
                black_box(values)
            })
        });
        group.bench_function("dotenv-ng-1.0/read+parse", |b| {
            b.iter(|| {
                black_box(
                    dotenv_ng_upstream::EnvLoader::with_path(black_box(&path))
                        .sequence(dotenv_ng_upstream::EnvSequence::InputOnly)
                        .load()
                        .unwrap(),
                )
            })
        });
        group.bench_function("this/parse-only", |b| {
            b.iter(|| black_box(dotenv::parse(black_box(&input))))
        });
        group.finish();

        // Keep the temporary file alive until all measurements finish.
        black_box(&file);
    }

    let multiline =
        "CERT='-----BEGIN-----\nline1\nline2\n-----END-----'\nJSON={\"a\": 1}\n".repeat(100);
    c.bench_function("parse/multiline/this", |b| {
        b.iter_batched(
            || multiline.clone(),
            |input| black_box(dotenv::parse(&input)),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, parser_benchmarks);
criterion_main!(benches);
