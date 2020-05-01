//! A command line tool for compressing/decompressing files directly from the
//! command line.
use rscompress_huffman::encode::{calculate_length, Encoder};
use rscompress_huffman::huffman::{generate_extended_codewords, Huffman};
use rscompress_huffman::stats::generate_histogram;
use rscompress_huffman::BUF;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

use env_logger; // trace < debug < info < warn < error < off
use log::{info, log_enabled};

/// Main function (duh!)

fn main() {
    env_logger::init();
    let source = env::args().nth(1).expect("No source file found!");
    let destination = env::args().nth(2).expect("No destination file found");
    let method = env::args().nth(3).expect("No method found");
    if method == "h".to_string() {
        info!("Huffman with header information");
        rscompress_huffman::stream_compress_with_header_information(&source, &destination)
    } else {
        info!("Huffman without header information");
        old_main();
    }
}

fn old_main() {
    let source = env::args().nth(1).expect("No source file found!");
    let destination = env::args().nth(2).expect("No destination file found");
    info!("Starting compression");
    info!("Input:  {}", &source);
    info!("Output: {}", &destination);
    let sfile = File::open(source).expect("Failed to open source file");
    let dfile = File::create(destination).expect("Failed to create destination file");

    let mut reader = BufReader::with_capacity(BUF, sfile);
    let mut buffer: Vec<u8> = Vec::with_capacity(BUF);
    unsafe { buffer.set_len(BUF) }

    info!("Generating histogram...");
    let histogram = generate_histogram(&mut reader);
    info!("Generating codewords...");
    let (codewords, length) = generate_extended_codewords(&histogram);

    let w = BufWriter::with_capacity(BUF, dfile);
    let h = Huffman::new(codewords, length);
    let mut writer = Encoder::new(w, &h);
    if log_enabled!(log::Level::Debug) || log_enabled!(log::Level::Info) {
        let mut original_file_size = 0;
        let mut huffmann_file_size = 0;
        for (count, code) in histogram.iter().zip(codewords.iter()) {
            original_file_size += count;
            huffmann_file_size += count * calculate_length(*code);
        }
        huffmann_file_size = huffmann_file_size / 8 + 1;
        info!("Original file size: {}", original_file_size);
        info!("Huffman file size:  {}", huffmann_file_size);
        info!(
            "Compression factor: {:.2}",
            original_file_size as f32 / huffmann_file_size as f32
        );
        info!(
            "Compression ratio:  {:.2}",
            huffmann_file_size as f32 / original_file_size as f32
        );
    }

    reader
        .seek(std::io::SeekFrom::Start(0))
        .expect("Can not move to start of file");
    info!("Starting writing...");
    loop {
        let read_size = reader.read(&mut buffer);
        match read_size {
            Ok(0) => break, // fully read file
            Ok(n) => writer
                .write(&buffer[..n])
                .expect("Could not write buffer to destination"),
            Err(err) => panic!("Problem with reading source file: {:?}", err),
        };
    }
    writer.flush().expect("Could not flush file to disk!");
    info!("End compression")
}
