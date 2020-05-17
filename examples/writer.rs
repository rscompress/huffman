use rscompress_huffman::huffman::Huffman;
use rscompress_huffman::huffman::encode::Encoder;
// use rscompress_huffman::stats::random_bytes;
use rscompress_huffman::huffman::decode::writer::Decoder;
use std::time::Instant;
use std::io::{Cursor, Write};
use log::{info};


fn main() {
    let origin: Vec<u8> = "What a lovely world".as_bytes().to_vec();

    // Generate Huffman Model
    let h = Huffman::from_reader(&mut Cursor::new(&origin));

    // Generate Encoder and apply to data
    let now = Instant::now();
    let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
    enc.write(&origin).expect("");
    enc.flush().expect("");
    info!("Encoding {}", now.elapsed().as_secs_f32());
    let encoded_data : Vec<u8> = enc.inner.get_ref().iter().map(|&x| x).collect();
    print!("                  ");
    for k in encoded_data.iter() {
        print!("{:08b}", k);
    }
    println!("");

    let mut decoder = Decoder::new(Cursor::new(Vec::new()), &h, origin.len() as u64);
    decoder.write(&encoded_data[..]).unwrap();
    println!("{}", decoder);


}
