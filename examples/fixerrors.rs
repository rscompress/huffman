use std::io::{BufReader, Read, Cursor, Write};
use std::fs::File;
use log::{error, info, debug};
use rscompress_huffman::huffman::{Huffman, encode::Encoder};

fn main() {
    env_logger::init();
    let mut origin: Vec<u8> = Vec::new();
    let mut r = BufReader::with_capacity(4096, File::open("testdata/errors.itr.25.raw").unwrap());
    r.read_to_end(&mut origin).unwrap();
    info!("Size: {}", origin.len());

    // Prepare histogram
    let words: Vec<u8> = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
    let mut histogram = [0usize; 256];
    for i in 0..words.len() {
        histogram[i] = words[i] as usize;
    }
    let h = Huffman::from_histogram(&histogram);
    let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
    enc.write(&origin).expect("");
    enc.flush().expect("");
    let encoded_data : Vec<u8> = enc.inner.get_ref().iter().map(|&x| x).collect();

    let decoder = rscompress_huffman::huffman::decode::vault::Decoder::new(encoded_data.into_iter(), &h, origin.len() as u64);
    let decoded_data: Vec<u8> = decoder.collect();

    for (i, (exp, result)) in origin.iter().zip(decoded_data.iter()).enumerate() {
        if exp != result {
            error!("@{} -> {} != {}", i, exp, result)
        }
    }
}
