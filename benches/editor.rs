use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

fn editor_benchmarks(c: &mut Criterion) {
    for entries in [10, 1_000, 10_000] {
        let input: String = (0..entries)
            .map(|i| format!("KEY_{i}=value-{i}\n"))
            .collect();
        let mut group = c.benchmark_group(format!("editor/{entries}_entries"));
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_function("construct+indexed-upsert+dump+render", |b| {
            b.iter(|| {
                let mut editor = dotenv::DotenvEditor::from_content(black_box(&input)).unwrap();
                editor.upsert("KEY_5", "new value").unwrap();
                editor.upsert("NEW_KEY", "a # protected value").unwrap();
                black_box(editor.dump());
                black_box(editor.render())
            })
        });
        group.finish();
    }
}

criterion_group!(benches, editor_benchmarks);
criterion_main!(benches);
