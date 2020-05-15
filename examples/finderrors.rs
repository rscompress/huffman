#![allow(unused_imports)]
use log::{info, error};
use rscompress_huffman::huffman::decode::one::read;
use rscompress_huffman::huffman::encode::Encoder;
use rscompress_huffman::huffman::Huffman;
use rscompress_huffman::stats::{generate_random_byte_vector, random_bytes as generate_bytes};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, BufWriter, Read};
use std::io::{Cursor, Write};
use std::time::Instant;

const ROUNDS: usize = 50;

#[allow(unreachable_code)]
fn main() {
    env_logger::init();

    for j in 0..ROUNDS {

        // Generate random data
        let origin: Vec<u8> = generate_bytes(32*1024*1024);

        // Generate Huffman Model
        let h = Huffman::from_reader(&mut Cursor::new(&origin));

        // Generate Encoder and apply to data
        let now = Instant::now();
        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
        enc.write(&origin).expect("");
        enc.flush().expect("");
        info!("Encoding {}", now.elapsed().as_secs_f32());
        let encoded_data : Vec<u8> = enc.inner.get_ref().iter().map(|&x| x).collect();

        // There are three different implementations of the decoder.
        // They are being tested for possible errors while decoding.

        // Decoder using read()
        let now = Instant::now();
        let mut decoder = rscompress_huffman::huffman::decode::reader::Decoder::new(Cursor::new(&encoded_data), &h, origin.len() as u64);
        let mut decoded_data: Vec<u8> = Vec::new();
        decoder.read_to_end(&mut decoded_data).unwrap();
        info!("Decoder Reader {}", now.elapsed().as_secs_f32());
        if decoded_data != origin {
            error!("{} Decoder Reader wrong!", j);
            write_file("reader", j, &origin)
        }

        // Decoder using simple function
        let now = Instant::now();
        let decoded_words = read(enc.inner.get_ref(), &h, enc.readbytes);
        info!("Simple function {}", now.elapsed().as_secs_f32());
        if decoded_words != origin {
            error!("{} Simple function wrong!", j);
            write_file("simple", j, &origin)
        }

        // Decoder using iterator
        let now = Instant::now();
        let decoder = rscompress_huffman::huffman::decode::iterator::Decoder::new(encoded_data.into_iter(), &h, origin.len() as u64);
        let decoded_data: Vec<u8> = decoder.collect();
        info!("Iterator function {}", now.elapsed().as_secs_f32());
        if decoded_data != origin {
            error!("{} Iterator function wrong!", j);
            write_file("iterator", j, &origin)
            }
    }
}


fn write_file(method: &str, run: usize, data: &[u8]) {
    let file  = format!("testdata/test.{}.{}.raw", method, run);
    let dfile = File::create(file).expect("Failed to create destination file");
    let mut w = BufWriter::with_capacity(4096, dfile);
    w.write_all(&data).unwrap();
}
