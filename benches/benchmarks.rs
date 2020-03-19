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


use rscompress_huffman::huffman::{sort_by_value, extract_values, calculate_codewords_based_on_length, calculate_codeword_length_inplace};

// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_splits(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("whole_chain", |b|
        b.iter(|| {
            let sorted_tuple = sort_by_value(&histogram);  // Step 1
            let mut weights = extract_values(&sorted_tuple);  // Step 2
            calculate_codeword_length_inplace(&mut weights);  // Step 3
            let (codes, _) = calculate_codewords_based_on_length(&weights);  // Step 4

            let mut extended_codes = [0usize; 256];
            for (code, (key,_)) in codes.iter().zip(sorted_tuple.iter()) {
                extended_codes[*key as usize] = *code;
            }
            extended_codes
        }));
    group.finish();
}


// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_excl1(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());
    // let hist = enumerate(&histogram);  // Step 1

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("step_1_sort_by_value", |b|
        b.iter(|| {
            sort_by_value(&histogram);  // Step 1
        }));
    group.finish();
}

// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_excl2(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());
    let sorted_tuple = sort_by_value(&histogram);  // Step 1

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("step_2_extract_values", |b|
        b.iter(|| {
            extract_values(&sorted_tuple);  // Step 2
        }));
    group.finish();
}


// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_excl3(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());
    let sorted_tuple = sort_by_value(&histogram);  // Step 1
    let mut weights = extract_values(&sorted_tuple);  // Step 2

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("step_3_calculate_lengths", |b|
        b.iter(|| {
            calculate_codeword_length_inplace(&mut weights);  // Step 3
        }));
    group.finish();
}


// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_excl4(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());
    let sorted_tuple = sort_by_value(&histogram);  // Step 1
    let mut weights = extract_values(&sorted_tuple);  // Step 2
    calculate_codeword_length_inplace(&mut weights);  // Step 3

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("step_4_calculate_codewords", |b|
        b.iter(|| {
            calculate_codewords_based_on_length(&weights);  // Step 4
        }));
    group.finish();
}


// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_excl5(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());
    let sorted_tuple = sort_by_value(&histogram);  // Step 1
    let mut weights = extract_values(&sorted_tuple);  // Step 2
    calculate_codeword_length_inplace(&mut weights);  // Step 3
    let (codes, _) = calculate_codewords_based_on_length(&weights);  // Step 4

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("step_5_generate_codelist", |b|
        b.iter(|| {
            let mut extended_codes = [0usize; 256];
            for (code, (key,_)) in codes.iter().zip(sorted_tuple.iter()) {
                extended_codes[*key as usize] = *code;
            }
            extended_codes
        }));
    group.finish();
}




criterion_group!(
    benches,
    benchmark_histogram_generation,
    benchmark_codeword_generation,
    benchmark_encoding,
    benchmark_whole_encoding_chain
);
criterion_group!(
    benches_details,
    benchmark_codeoword_generation_splits,
    benchmark_codeoword_generation_excl1,
    benchmark_codeoword_generation_excl2,
    benchmark_codeoword_generation_excl3,
    benchmark_codeoword_generation_excl4,
    benchmark_codeoword_generation_excl5,
);
criterion_main!(benches, benches_details);
// criterion_main!(benches_details);
