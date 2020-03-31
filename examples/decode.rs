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
        let origin: Vec<u8> = generate_random_byte_vector(0, words.len() as u8, 25, &words);

        // Generate Huffman Model
        let h = Huffman::from_histogram(&histogram);

        // Generate Encoder and apply to data
        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
        enc.write(&origin).expect("");
        enc.flush().expect("");
        info!("Encoded length: {}", enc.writeout);

        // let now = Instant::now();
        let decoded_words = read(enc.inner.get_ref(), &h, enc.readbytes);
        // info!("Time: {}", now.elapsed().as_secs_f32());

        let reader = BufReader::new(Cursor::new(enc.inner.get_ref()));
        let mut decoder = Decoder::new(reader, &enc);
        let mut buf = [0u8;32];

        // for i in (0..100) {
        //     let nbytes = decoder.read(&mut buf).unwrap();
        //     assert_eq!(origin[i*buf.len()..buf.len()*(i+1)], decoded_words[i*buf.len()..buf.len()*(i+1)], "Not equal (old method)");
        //     // info!("{} Old method looks good", i);
        //     assert_eq!(origin[i*buf.len()..buf.len()*(i+1)], buf[..], "Not equal {}", nbytes);
        //     info!("{} Looks good: {:?} [{}]", i, buf, nbytes);
        // }
        let nbytes = decoder.read(&mut buf).unwrap();

        assert_eq!(decoded_words[..], origin[..]);
        info!("Old method successful");
        // Original encoding successful
        assert_eq!(buf[..nbytes], origin[..]);
        info!("New method successful");
    }
}
