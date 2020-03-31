use rscompress_huffman::decode::read;
use rscompress_huffman::encode::Encoder;
use rscompress_huffman::huffman::Huffman;
use rscompress_huffman::stats::generate_random_byte_vector;
use log::info;
use std::io::{Cursor, Write};
use std::time::Instant;

#[allow(unreachable_code)]
fn main() {
    env_logger::init();
    let words: Vec<u8> = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
    let mut histogram = [0usize; 256];
    for i in 0..words.len() {
        histogram[i] = words[i] as usize;
    }

    for i in 0..150 {
        let origin: Vec<u8> = generate_random_byte_vector(0, words.len() as u8, 820_000, &words);
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
