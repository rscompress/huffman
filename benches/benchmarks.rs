use criterion::{Throughput, BenchmarkId};
use criterion::{criterion_group, criterion_main, Criterion};
use rscompress_huffman::encode::Encoder;
use rscompress_huffman::huffman::{generate_extended_codewords, Huffman};
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
            let h = Huffman::from_reader(&mut bytes.as_slice());
            let mut writer = Encoder::new(Cursor::new(Vec::new()), &h);
            writer.write(bytes.as_slice())
        })
    });
    group.finish();
}

use std::fs::{metadata, File};
use std::io::BufReader;

fn benchmark_io(c: &mut Criterion) {
    let source = String::from("test.tmp");
    let destination = String::from("/tmp/bla.tmp");
    let md = metadata("test.tmp").expect("Nooo");
    let sfile = File::open(source).expect("Failed to open source file");
    let dfile = File::create(destination).expect("Failed to create destination file");
    let buf = 4096;

    let mut reader = BufReader::with_capacity(buf, sfile);

    let h = Huffman::from_reader(&mut reader);
    let mut writer = Encoder::new(dfile, &h);
    reader
        .seek(std::io::SeekFrom::Start(0))
        .expect("Can not move to start of file");

    let mut group = c.benchmark_group("throughput_encoding");
    group.throughput(Throughput::Bytes(md.len() as u64));
    group.bench_function("I/O", |b| {
        b.iter(|| {
            full_io(&mut reader, &mut writer);
        })
    });
    group.finish();
}

fn full_io(reader: &mut impl BufRead, writer: &mut impl Write) {
    let buf = 4096;
    let mut buffer: Vec<u8> = Vec::with_capacity(buf);
    unsafe { buffer.set_len(buf) }

    loop {
        let read_size = reader.read(&mut buffer);
        match read_size {
            Ok(0) => break, // fully read file
            Ok(n) => writer
                .write(&mut buffer[..n])
                .expect("Could not write buffer to destination"),
            Err(err) => panic!("Problem with reading source file: {:?}", err),
        };
    }
    writer.flush().expect("Could not flush file to disk!");
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
    let h = Huffman::from_reader(&mut bytes.as_slice());
    let mut writer = Encoder::new(Cursor::new(Vec::new()), &h);

    let mut group = c.benchmark_group("throughput_encoding");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("encoding", |b| b.iter(|| writer.write(bytes.as_slice())));
    group.finish();
}

use rscompress_huffman::huffman::{
    calculate_codeword_length_inplace, calculate_codewords_based_on_length, extract_values,
    sort_by_value,
};

// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_splits(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("whole_chain", |b| {
        b.iter(|| {
            let sorted_tuple = sort_by_value(&histogram); // Step 1
            let mut weights = extract_values(&sorted_tuple); // Step 2
            calculate_codeword_length_inplace(&mut weights); // Step 3
            let (codes, _) = calculate_codewords_based_on_length(&weights); // Step 4

            let mut extended_codes = [0usize; 256];
            for (code, (key, _)) in codes.iter().zip(sorted_tuple.iter()) {
                extended_codes[*key as usize] = *code;
            }
            extended_codes
        })
    });
    group.finish();
}

// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_excl1(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("step_1_sort_by_value", |b| {
        b.iter(|| {
            sort_by_value(&histogram); // Step 1
        })
    });
    group.finish();
}

// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_excl2(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());
    let sorted_tuple = sort_by_value(&histogram); // Step 1

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("step_2_extract_values", |b| {
        b.iter(|| {
            extract_values(&sorted_tuple); // Step 2
        })
    });
    group.finish();
}

// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_excl3(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());
    let sorted_tuple = sort_by_value(&histogram); // Step 1
    let mut weights = extract_values(&sorted_tuple); // Step 2

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("step_3_calculate_lengths", |b| {
        b.iter(|| {
            calculate_codeword_length_inplace(&mut weights); // Step 3
        })
    });
    group.finish();
}

// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_excl4(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());
    let sorted_tuple = sort_by_value(&histogram); // Step 1
    let mut weights = extract_values(&sorted_tuple); // Step 2
    calculate_codeword_length_inplace(&mut weights); // Step 3

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("step_4_calculate_codewords", |b| {
        b.iter(|| {
            calculate_codewords_based_on_length(&weights); // Step 4
        })
    });
    group.finish();
}

// Looking into codeword generation and it takes soo long
fn benchmark_codeoword_generation_excl5(c: &mut Criterion) {
    let bytes: Vec<u8> = vec![
        3, 12, 24, 222, 131, 151, 23, 141, 24, 234, 11, 1, 1, 1, 24, 242, 52, 231,
    ];
    let histogram = generate_histogram(&mut bytes.as_slice());
    let sorted_tuple = sort_by_value(&histogram); // Step 1
    let mut weights = extract_values(&sorted_tuple); // Step 2
    calculate_codeword_length_inplace(&mut weights); // Step 3
    let (codes, _) = calculate_codewords_based_on_length(&weights); // Step 4

    let mut group = c.benchmark_group("codeword_generation");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("step_5_generate_codelist", |b| {
        b.iter(|| {
            let mut extended_codes = [0usize; 256];
            for (code, (key, _)) in codes.iter().zip(sorted_tuple.iter()) {
                extended_codes[*key as usize] = *code;
            }
            extended_codes
        })
    });
    group.finish();
}

use rscompress_huffman::encode::calculate_length;
use rscompress_huffman::pack::Pack;

// Looking into codeword generation and it takes soo long
fn benchmark_packing_of_bits(c: &mut Criterion) {
    let words: Vec<u8> = vec![177, 112, 84, 143, 148, 195, 165, 206, 34, 10];
    let mut codewords = [0usize; 256];
    let mut length = [0usize; 256];
    for word in words.iter() {
        codewords[*word as usize] = *word as usize;
        length[*word as usize] = calculate_length(*word as usize);
    }
    let mut enc = Pack::new(Cursor::new(Vec::new()), codewords, length);

    let mut group = c.benchmark_group("packing");
    group.throughput(Throughput::Bytes(words.len() as u64));
    group.bench_function("pack", |b| {
        b.iter(|| {
            enc.write(&words).expect("");
            enc.flush().expect("");
        })
    });
    group.finish();
}

// Looking into codeword generation and it takes soo long
fn benchmark_packing_of_bits_encode(c: &mut Criterion) {
    let words: Vec<u8> = vec![177, 112, 84, 143, 148, 195, 165, 206, 34, 10];
    let mut codewords = [0usize; 256];
    let mut length = [0usize; 256];
    for word in words.iter() {
        codewords[*word as usize] = *word as usize;
        length[*word as usize] = calculate_length(*word as usize);
    }
    let h = Huffman::new(codewords, length);
    let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);

    let mut group = c.benchmark_group("packing");
    group.throughput(Throughput::Bytes(words.len() as u64));
    group.bench_function("encode", |b| {
        b.iter(|| {
            enc.write(&words).expect("");
            enc.flush().expect("");
        })
    });
    group.finish();
}

use rscompress_huffman::decode::{read, search_key_or_next_small_key};
use rscompress_huffman::model::Model;
use std::io::Write;

fn benchmark_packing_of_bits_decode(c: &mut Criterion) {
    // Generate Huffman Encoder
    let words: Vec<u8> = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
    let mut histogram = [0usize; 256];
    for i in 0..words.len() {
        histogram[i] = words[i] as usize;
    }
    let h = Huffman::from_histogram(&histogram);
    let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);

    // Encode `words`
    let origin: Vec<u8> = vec![0, 9, 9, 9, 9, 9, 7, 0, 7, 4, 9, 9, 0, 0, 0, 4, 0];
    enc.write(&origin).expect("");
    enc.flush().expect("");

    let inputs = (enc.inner.get_ref(), &h, enc.readbytes);

    let mut group = c.benchmark_group("packing");
    group.throughput(Throughput::Bytes(origin.len() as u64));
    group.bench_with_input(BenchmarkId::new("decoding", enc.readbytes), &inputs, |b, &inputs| {
        b.iter(|| {
            read(inputs.0, inputs.1, inputs.2);
        })
    });
    group.finish();
}

use std::collections::BTreeMap;
use std::io::{BufRead};

fn iter_search_key_or_next_small_key(bt: &BTreeMap<usize, (u8,u8)>, data: &[u8]) {
    for key in data {
        search_key_or_next_small_key(bt, *key as usize);
    }
}

fn benchmark_searching_for_key_value(c: &mut Criterion) {
    let words: Vec<u8> = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
    let mut histogram = [0usize; 256];
    for i in 0..words.len() {
        histogram[i] = words[i] as usize;
    }
    let h = Huffman::from_histogram(&histogram);
    let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
    let sfile = File::open("test.raw").expect("Failed to open source file");
    let mut b = BufReader::new(sfile);
    let mut origin = Vec::new();
    b.read_to_end(&mut origin).expect("");
    enc.write(&origin).expect("");
    enc.flush().expect("");
    let bt = h.to_btreemap();

    let mut group = c.benchmark_group("decode");
    group.throughput(Throughput::Bytes(origin.len() as u64));
    group.bench_function("search for key", |b| {
        b.iter(|| {
            iter_search_key_or_next_small_key(&bt, &origin)
        })
    });
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
criterion_group!(
    packing,
    benchmark_packing_of_bits,
    benchmark_packing_of_bits_encode,
    benchmark_packing_of_bits_decode,
    benchmark_searching_for_key_value,
);
criterion_group!(io, benchmark_io);
criterion_group!(search, benchmark_searching_for_key_value);

criterion_main!(search);
