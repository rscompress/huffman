
use rscompress_huffman::huffman::{decode::vault::Decoder, encode::Encoder, Huffman};
use std::io::{Write, Cursor, Read};

fn main() {
    env_logger::init();

    let data = "This is a lovely text in a big world".as_bytes().to_vec();
    let data = "aafaaaaaaaaaa".as_bytes().to_vec(); // TODO breaking case
    let h = Huffman::from_slice(data.as_slice());

    let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
    let _output_bytes = enc.write(&data).expect("");
    enc.flush().expect("");

    let encoded_data : Vec<u8> = enc.inner.get_ref().iter().map(|&x| x).collect();
    println!("{:?}", encoded_data);
    println!("{:?}", data);

    let mut decoder = Decoder::new(Cursor::new(encoded_data), &h, data.len() as u64);
    let mut decoded_data = [0u8;10];
    let mut nbytes = 1;

    // TODO: read_to_end() does not seem to work
    while nbytes > 0 {
        nbytes = decoder.read(&mut decoded_data).unwrap();
        println!("{:?} {}", decoded_data, nbytes);
    }
}
