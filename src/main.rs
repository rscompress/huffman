//! A command line tool for compressing/decompressing files directly from the
//! command line.
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use rscompress_huffman::stats::generate_histogram;
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

    for (i,v) in histogram.iter().enumerate() {
        println!("{}: {}", i, v)
    }

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
