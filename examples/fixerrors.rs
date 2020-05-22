use log::info;
use rscompress_huffman::huffman::decode;
use rscompress_huffman::huffman::{encode::Encoder, Huffman};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, Write};
use std::time::Instant;

fn main() {
    env_logger::init();
    let mut origin: Vec<u8> = Vec::new();
    let mut r = BufReader::with_capacity(4096, File::open("testdata/test.tmp").unwrap());
    r.read_to_end(&mut origin).unwrap();
    r.seek(std::io::SeekFrom::Start(0)).unwrap();
    info!("Size: {}", origin.len());

    // Generate Huffman Model
    let h = Huffman::from_reader(&mut Cursor::new(&origin));

    // Generate Encoder and apply to data
    let now = Instant::now();
    let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
    enc.write(&origin).expect("");
    enc.flush().expect("");
    info!("Encoding {}", now.elapsed().as_secs_f32());
    let encoded_data: Vec<u8> = enc.inner.get_ref().iter().map(|&x| x).collect();

    // There are three different implementations of the decoder.
    // They are being tested for possible errors while decoding.

    // Decoder using read()
    let now = Instant::now();
    let mut decoder = rscompress_huffman::huffman::decode::reader::Decoder::new(
        Cursor::new(&encoded_data),
        &h,
        origin.len() as u64,
    );
    let mut decoded_data: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut decoded_data).unwrap();
    info!("Decoder Reader {}", now.elapsed().as_secs_f32());

    // Decoder using simple function
    let now = Instant::now();
    let decoded_words = decode(enc.inner.get_ref(), &h, enc.readbytes);
    info!("Simple function {}", now.elapsed().as_secs_f32());

    // Decoder using iterator
    let now = Instant::now();
    let decoder = rscompress_huffman::huffman::decode::iterator::Decoder::new(
        encoded_data.into_iter(),
        &h,
        origin.len() as u64,
    );
    let decoded_data: Vec<u8> = decoder.collect();
    info!("Iterator function {}", now.elapsed().as_secs_f32());
}
