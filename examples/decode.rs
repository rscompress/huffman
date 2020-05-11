#![allow(unused_imports)]
use log::info;
use rscompress_huffman::huffman::decode::{read, Decoder};
use rscompress_huffman::huffman::encode::Encoder;
use rscompress_huffman::huffman::Huffman;
use rscompress_huffman::stats::generate_random_byte_vector;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, BufWriter, Read};
use std::io::{Cursor, Write};
use std::time::Instant;

#[allow(unreachable_code)]
fn main() {
    env_logger::init();

    // Prepare histogram
    let words: Vec<u8> = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
    let mut histogram = [0usize; 256];
    for i in 0..words.len() {
        histogram[i] = words[i] as usize;
    }

    for j in 0..1 {
        // Generate random data
        // let origin: Vec<u8> =
            // generate_random_byte_vector(0, words.len() as u8, 500_000_000, &words);
        // If error found save to file
        // let dfile = File::create("errors.raw").expect("Failed to create destination file");
        // let mut w = BufWriter::with_capacity(4096, dfile);
        // w.write_all(&origin).unwrap();

        // If error found, read from file
        let mut origin: Vec<u8> = Vec::new();
        let mut r = BufReader::with_capacity(4096, File::open("errors.raw").unwrap());
        r.read_to_end(&mut origin).unwrap();
        info!("Size: {}", origin.len());

        // Generate Huffman Model
        let h = Huffman::from_histogram(&histogram);

        // Generate Encoder and apply to data
        let now = Instant::now();
        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
        enc.write(&origin).expect("");
        enc.flush().expect("");
        info!("E {}", now.elapsed().as_secs_f32());

        // Old decoding method
        let now = Instant::now();
        let decoded_words = read(enc.inner.get_ref(), &h, enc.readbytes);
        info!("O {}", now.elapsed().as_secs_f32());
        info!("Result {:?}",decoded_words);

        // Read decoding method
        let reader = BufReader::new(Cursor::new(enc.inner.get_ref()));
        let mut decoder = Decoder::new(reader, &enc);
        let mut buf = [0u8; 15];
        let mut full: Vec<u8> = Vec::with_capacity(origin.len());

        // Check results of both methods
        let now = Instant::now();
        let mut sum = 0;
        while let Ok(nbytes) = decoder.read(&mut buf) {
            if nbytes == 0 {
                break;
            }
            assert_eq!(origin[sum..sum+nbytes], decoded_words[sum..sum+nbytes], "Not equal (old method)");
            info!("[{}] Old method looks good", j);
            assert_eq!(origin[sum..(sum+nbytes)], buf[..nbytes], "Not equal [{};{}]", sum, sum+nbytes);
            info!("Range {}-{} fine", sum, sum+nbytes);
            sum+= nbytes;
            info!("[{}] New method looks good", j);
            full.append(&mut buf[..nbytes].to_vec());
        }
        info!("N {}", now.elapsed().as_secs_f32());
        assert_eq!(full, origin);
        assert_eq!(decoded_words, full);
        info!("{} Success", j)
    }
}
