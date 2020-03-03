//! A command line tool for compressing/decompressing files directly from the
//! command line.
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use rscompress_huffman::stats::generate_histogram;
use rscompress_huffman::huffman::generate_extended_codewords;
use rscompress_huffman::encode::calculate_length;
use rscompress_huffman::BUF;

/// Main function (duh!)
fn main() {
    let source = env::args().nth(1).expect("No source file found!");
    let sfile = File::open(source).expect("Failed to open source file");
    let destination = env::args().nth(2).expect("No destination file found");
    let dfile = File::create(destination).expect("Failed to create destination file");

    let mut reader = BufReader::with_capacity(BUF, sfile);
    let mut writer = BufWriter::with_capacity(BUF, dfile);
    let mut buffer: Vec<u8> = Vec::with_capacity(BUF);
    unsafe { buffer.set_len(BUF) }

    let histogram = generate_histogram(&mut reader);
    let codewords = generate_extended_codewords(&histogram);

    // Start information about compressed file
    let mut original_file_size = 0;
    let mut huffmann_file_size = 0;
    for (word,(count,code)) in histogram.iter().zip(codewords.iter()).enumerate() {
        println!("'{:08b}' -> '{:>8b}' ({}x)", word, code, count);
        original_file_size += count;
        huffmann_file_size += count * calculate_length(word);
    }
    huffmann_file_size = huffmann_file_size / 8 + 1;
    println!("Original file size: {}", original_file_size);
    println!(" Huffman file size: {}", huffmann_file_size);
    println!("Compression factor: {:.2}", original_file_size as f32 / huffmann_file_size as f32);
    println!(" Compression ratio: {:.2}", huffmann_file_size as f32 / original_file_size as f32);
    // End information about compressed file

    reader.seek(std::io::SeekFrom::Start(0)).expect("Can not move to start of file");
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
}
