use rscompress_huffman::decode::read;
use rscompress_huffman::encode::Encoder;
use rscompress_huffman::huffman::Huffman;
use rscompress_huffman::stats::generate_random_byte_vector;
// use std::fs::File;
// use std::io::prelude::*;
// use std::io::{BufRead, BufReader, BufWriter};
use log::info;
use std::io::{Cursor, Write};
use std::time::Instant;

#[allow(unreachable_code)]
fn main() {
    env_logger::init();
    // Generate Huffman Encoder
    let words: Vec<u8> = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
    let mut histogram = [0usize; 256];
    for i in 0..words.len() {
        histogram[i] = words[i] as usize;
    }

    // Encode `words`
    // let origin : Vec<u8> = vec![0,9,9,9,9,9,7,0,7,4,9,9,0,0,0,4,0];
    for i in 0..50 {
        let origin: Vec<u8> = generate_random_byte_vector(0, words.len() as u8, 70309444, &words);
        // let mut origin: Vec<u8> = Vec::new();
        // let mut r = BufReader::with_capacity(4096, File::open("erorrs.raw").unwrap());
        // r.read_to_end(&mut origin).unwrap();
        let h = Huffman::from_histogram(&histogram);
        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
        enc.write(&origin).expect("");
        enc.flush().expect("");
        info!("Encoded length: {}", enc.writeout);
        let now = Instant::now();
        let decoded_words = read(enc.inner.get_ref(), &h, enc.readbytes);
        info!("Time: {}", now.elapsed().as_secs_f32());
        assert_eq!(decoded_words, origin);
        info!("{} successful", i)
    }
}
