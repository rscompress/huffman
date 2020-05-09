
use rscompress_huffman::huffman::{decode::vault::Decoder, encode::Encoder, Huffman};
use std::io::{Write, Cursor};

fn main() {
    env_logger::init();

    let data = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".as_bytes().to_vec();
    let h = Huffman::from_slice(data.as_slice());

    let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
    let _output_bytes = enc.write(&data).expect("");
    enc.flush().expect("");

    let encoded_data : Vec<u8> = enc.inner.get_ref().iter().map(|&x| x).collect();
    println!("{:?}", encoded_data);
    println!("{:?}", data);

    let decoder = Decoder::new(encoded_data.into_iter(), &h, data.len() as u64);
    let decoded_data: Vec<u8> = decoder.collect();
    println!("{:?}", decoded_data);
}
