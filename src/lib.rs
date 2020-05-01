//! This crate implements the Huffman Coding algorithm specified
//! in the [master thesis](http://compression.ru/download/articles/huff/huffman_1952_minimum-redundancy-codes.pdf)
//! of David A. Huffman.
//!
//! # Huffman Coding
//! This implementation uses stream coding and canonical Huffman Coding. The file
//! is being read in chunks and coded using multi-pass encoding.
//!
//! ## Encoding Workflow
//! The algorithm first traverses the file and builds a histogram for each byte.
//! Afterwards it builds the codewords using a compact representation of the codewords
//! described in the above paper. A second traversal of file then encodes each
//! byte and saves it on disk.

use log::info;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

//#[global_allocator]
//static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub const BUF: usize = 4096;
pub mod decode;
pub mod encode;
pub mod header;
pub mod huffman;
pub mod model;
pub mod stats;

pub fn stream_compress_with_header_information(source: &str, destination: &str) {
    info!("Starting compression");
    info!("Input:  {}", &source);
    info!("Output: {}", &destination);
    // Create reader object
    let sfile = File::open(source).expect("Failed to open source file");
    let filesize = std::fs::metadata(source).expect("Can not read filesize").len();
    let mut reader = BufReader::with_capacity(BUF, sfile);
    let mut buffer: Vec<u8> = Vec::with_capacity(BUF);
    unsafe { buffer.set_len(BUF) }

    // Create writer object
    let dfile = File::create(destination).expect("Failed to create destination file");
    let w = BufWriter::with_capacity(BUF, dfile);

    // Create encoder
    let h = huffman::Huffman::from_reader(&mut reader);
    let mut writer = encode::Encoder::new(w, &h);

    // Reset reader
    // TODO The reset can also be done in the encoder
    reader
        .seek(std::io::SeekFrom::Start(0))
        .expect("Can not move to start of file");

    // Write header
    // TODO The header write can also be done in the encoder
    let mut h = header::Header::from(&writer);
    h.update_readbytes(filesize);
    info!("Header: {:?}", h);
    let header = h.to_binary();
    let header_length = u64_to_bytes(header.len() as u64);
    writer
        .plain_write(&writer.magic())
        .expect("Could not write magic");
    writer
        .plain_write(&header_length)
        .expect("Could not write header length");
    writer
        .plain_write(&header)
        .expect("Could not write header");

    //Compress file
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

fn u64_to_bytes(num: u64) -> [u8; 8] {
    [
        (num >> 56) as u8,
        (num >> 48) as u8,
        (num >> 40) as u8,
        (num >> 32) as u8,
        (num >> 24) as u8,
        (num >> 16) as u8,
        (num >> 08) as u8,
        (num & 0xFF) as u8,
    ]
}

fn bytes_to_u64(bytes: &[u8]) -> u64 {
    assert_eq!(bytes.len(), 8);
    let mut result = 0u64;
    for (i, byte) in bytes.iter().enumerate() {
        result += (*byte as u64) << (56 - i * 8);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_to_bytes() {
        let input: Vec<u64> = vec![
            341,
            1,
            32425,
            23534134,
            3234159383273,
            534457654,
            8273839836383,
        ];
        let mut expected: Vec<[u8; 8]> = Vec::new();
        expected.push([0, 0, 0, 0, 0, 0, 1, 85]);
        expected.push([0, 0, 0, 0, 0, 0, 0, 1]);
        expected.push([0, 0, 0, 0, 0, 0, 126, 169]);
        expected.push([0, 0, 0, 0, 1, 103, 26, 54]);
        expected.push([0, 0, 2, 241, 2, 235, 210, 233]);
        expected.push([0, 0, 0, 0, 31, 219, 45, 54]);
        expected.push([0, 0, 7, 134, 103, 72, 204, 223]);

        for (num, expected) in input.into_iter().zip(expected.into_iter()) {
            assert_eq!(expected, u64_to_bytes(num))
        }
    }

    #[test]
    fn test_u64_to_bytes_roundtrip() {
        let input: Vec<u64> = vec![
            341,
            1,
            32425,
            23534134,
            3234159383273,
            534457654,
            8273839836383,
        ];
        for num in input {
            let bytes = u64_to_bytes(num);
            let reverse = bytes_to_u64(&bytes);
            assert_eq!(num, reverse)
        }
    }
}
