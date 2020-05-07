
use rscompress_huffman::{vault::Decoder, encode::Encoder, huffman::Huffman};
use rscompress_huffman::encode::calculate_length;
use std::io::{Write, Cursor};

fn main() {
    env_logger::init();
    // let data = "What a lovely text".as_bytes().to_vec();
    // println!("{:?}", data);

    // let decoder = vault::Decoder::new(data.into_iter(), 5);
    // println!("{:?}", decoder);
    // for value in decoder.into_iter().enumerate() {
    //     println!("Iteration {}", value.0);
    //     println!("Result {}", value.1);
    // }


    let data = "What a lovely.".as_bytes().to_vec();
    let mut codewords = [0usize; 256];
    let mut length = [0usize; 256];
    for word in data.iter() {
        codewords[*word as usize] = *word as usize;
        length[*word as usize] = calculate_length(*word as usize);
    }
    let h = Huffman::new(codewords, length);
    let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
    let _output_bytes = enc.write(&data).expect("");
    enc.flush().expect("");

    let encoded_data : Vec<u8>= enc.inner.get_ref().iter().map(|&x| x).collect();
    println!("{:?}", encoded_data);
    println!("{:?}", data);

    let decoder = Decoder::new(encoded_data.into_iter(), &h, data.len() as u64);
    let decoded_data: Vec<u8> = decoder.collect();
    println!("{:?}", decoded_data);
}
