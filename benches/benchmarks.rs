use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use rscompress_huffman::encode::Encoder;
use rscompress_huffman::huffman::generate_extended_codewords;
use rscompress_huffman::stats::generate_histogram;
use std::io::prelude::*;
use std::io::Cursor;

// Example for `c.bench_function` usage for throughput analysis
fn benchmark_whole_encoding_chain(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];

    let mut group = c.benchmark_group("throughput_encoding");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("whole_chain", |b| {
        b.iter(|| {
            let histogram = generate_histogram(&mut bytes.as_slice());
            let codewords = generate_extended_codewords(&histogram);
            let mut writer = Encoder::new(Cursor::new(Vec::new()), codewords);
            writer.write(bytes.as_slice())
        })
    });
    group.finish();
}

// Example for `c.bench_function` usage for throughput analysis
fn benchmark_histogram_generation(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];

    let mut group = c.benchmark_group("throughput_encoding");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("histogram", |b| {
        b.iter(|| {
            generate_histogram(&mut bytes.as_slice());
        })
    });
    group.finish();
}

// Example for `c.bench_function` usage for throughput analysis
fn benchmark_codeword_generation(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());

    let mut group = c.benchmark_group("throughput_encoding");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("codewords", |b| {
        b.iter(|| generate_extended_codewords(&histogram))
    });
    group.finish();
}

// Example for `c.bench_function` usage for throughput analysis
fn benchmark_encoding(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());
    let codewords = generate_extended_codewords(&histogram);
    let mut writer = Encoder::new(Cursor::new(Vec::new()), codewords);

    let mut group = c.benchmark_group("throughput_encoding");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("encoding", |b| b.iter(|| writer.write(bytes.as_slice())));
    group.finish();
}

criterion_group!(
    benches,
    benchmark_histogram_generation,
    benchmark_codeword_generation,
    benchmark_encoding,
    benchmark_whole_encoding_chain
);
criterion_main!(benches);
