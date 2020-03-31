use rscompress_huffman::decode::{read, Decoder};
use rscompress_huffman::encode::Encoder;
use rscompress_huffman::huffman::Huffman;
use rscompress_huffman::stats::generate_random_byte_vector;
// use std::fs::File;
// use std::io::prelude::*;
use std::io::{BufRead, BufReader, BufWriter, Read};
use log::info;
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

    for j in 0..50 {

        // Generate random data
        let origin: Vec<u8> = generate_random_byte_vector(0, words.len() as u8, 47, &words);

        // Generate Huffman Model
        let h = Huffman::from_histogram(&histogram);

        // Generate Encoder and apply to data
        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
        enc.write(&origin).expect("");
        enc.flush().expect("");
        // info!("Encoded length: {}", enc.writeout);

        // let now = Instant::now();
        let decoded_words = read(enc.inner.get_ref(), &h, enc.readbytes);
        // info!("Time: {}", now.elapsed().as_secs_f32());

        let reader = BufReader::new(Cursor::new(enc.inner.get_ref()));
        let mut decoder = Decoder::new(reader, &enc);
        let mut buf = [0u8;15];
        let mut sum = 0usize;

        for i in (0..4) {
            let nbytes = decoder.read(&mut buf).unwrap();
            assert_eq!(origin[sum..sum+nbytes], decoded_words[sum..sum+nbytes], "Not equal (old method)");
            info!("[{},{}] Old method looks good", j, i);
            assert_eq!(origin[sum..(sum+nbytes)], buf[..nbytes], "Not equal {}", nbytes);
            sum+= nbytes;
            info!("[{},{}] New method looks good", j, i);
        }
    }
}
