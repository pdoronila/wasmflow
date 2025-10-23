//! Performance benchmarks for JSON parser node
//!
//! Tests T036, T037
//!
//! Run with: cargo bench --bench json_parser_bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use wasmflow::builtin::json_parser::parse;

/// Generate JSON of specified size
fn generate_json(size_kb: usize) -> String {
    let num_entries = (size_kb * 1024) / 50; // Approximate 50 bytes per entry
    let mut entries = Vec::new();

    for i in 0..num_entries {
        entries.push(format!(r#""field_{0}": "value_{0}""#, i));
    }

    format!(r#"{{{}}}"#, entries.join(","))
}

/// Generate deeply nested JSON
fn generate_nested_json(depth: usize) -> String {
    let mut json = String::from(r#"{"value": "deep"}"#);

    for i in (0..depth).rev() {
        json = format!(r#"{{"level_{0}": {1}}}"#, i, json);
    }

    json
}

/// Generate large array JSON
fn generate_array_json(size: usize) -> String {
    let items: Vec<String> = (0..size)
        .map(|i| format!(r#"{{"id": {0}, "value": "item_{0}"}}"#, i))
        .collect();

    format!(r#"{{"items": [{}]}}"#, items.join(","))
}

// ============================================================================
// T036: Performance benchmarks for different JSON sizes
// ============================================================================

fn bench_1kb_json(c: &mut Criterion) {
    let json = generate_json(1);

    c.bench_function("parse_1kb_json", |b| {
        b.iter(|| {
            let result = parse(black_box(&json), black_box("field_5"));
            black_box(result)
        })
    });
}

fn bench_10kb_json(c: &mut Criterion) {
    let json = generate_json(10);

    c.bench_function("parse_10kb_json", |b| {
        b.iter(|| {
            let result = parse(black_box(&json), black_box("field_50"));
            black_box(result)
        })
    });
}

fn bench_100kb_json(c: &mut Criterion) {
    let json = generate_json(100);

    c.bench_function("parse_100kb_json", |b| {
        b.iter(|| {
            let result = parse(black_box(&json), black_box("field_500"));
            black_box(result)
        })
    });
}

fn bench_1mb_json(c: &mut Criterion) {
    let json = generate_json(1024);

    c.bench_function("parse_1mb_json", |b| {
        b.iter(|| {
            let result = parse(black_box(&json), black_box("field_5000"));
            black_box(result)
        })
    });
}

// ============================================================================
// Deep nesting performance (SC-002)
// ============================================================================

fn bench_deep_nesting(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_nesting");

    for depth in [10, 50, 100].iter() {
        let json = generate_nested_json(*depth);
        let path = (0..*depth)
            .map(|i| format!("level_{}", i))
            .collect::<Vec<_>>()
            .join(".");
        let final_path = format!("{}.value", path);

        group.bench_with_input(
            BenchmarkId::from_parameter(depth),
            depth,
            |b, _| {
                b.iter(|| {
                    let result = parse(black_box(&json), black_box(&final_path));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// T037: Large array access performance (SC-003)
// ============================================================================

fn bench_large_array_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_access");

    for size in [100, 1000, 10000].iter() {
        let json = generate_array_json(*size);
        let mid_index = size / 2;
        let path = format!("items[{}].value", mid_index);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    let result = parse(black_box(&json), black_box(&path));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Complex path performance
// ============================================================================

fn bench_complex_paths(c: &mut Criterion) {
    let json = r#"{
        "data": {
            "users": [
                {
                    "id": 1,
                    "profile": {
                        "metadata": {
                            "settings": {
                                "preferences": {
                                    "theme": "dark"
                                }
                            }
                        }
                    }
                }
            ]
        }
    }"#;

    c.bench_function("parse_complex_path", |b| {
        b.iter(|| {
            let result = parse(
                black_box(json),
                black_box("data.users[0].profile.metadata.settings.preferences.theme"),
            );
            black_box(result)
        })
    });
}

// ============================================================================
// Tokenization performance
// ============================================================================

fn bench_tokenization(c: &mut Criterion) {
    use wasmflow::builtin::json_parser::tokenize;

    let mut group = c.benchmark_group("tokenization");

    // Simple path
    group.bench_function("simple", |b| {
        b.iter(|| {
            let result = tokenize(black_box("field"));
            black_box(result)
        })
    });

    // Nested path
    group.bench_function("nested", |b| {
        b.iter(|| {
            let result = tokenize(black_box("a.b.c.d.e.f"));
            black_box(result)
        })
    });

    // Array path
    group.bench_function("array", |b| {
        b.iter(|| {
            let result = tokenize(black_box("items[0][1][2]"));
            black_box(result)
        })
    });

    // Complex mixed path
    group.bench_function("complex", |b| {
        b.iter(|| {
            let result = tokenize(black_box("data.items[0].values[5].metadata.tags[2]"));
            black_box(result)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark groups
// ============================================================================

criterion_group!(
    benches,
    bench_1kb_json,
    bench_10kb_json,
    bench_100kb_json,
    bench_1mb_json,
    bench_deep_nesting,
    bench_large_array_access,
    bench_complex_paths,
    bench_tokenization,
);

criterion_main!(benches);
