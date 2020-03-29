use rscompress_huffman::stats::generate_random_byte_vector;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() {
    let fln = "test.raw";
    let words: Vec<u8> = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
    let origin: Vec<u8> = generate_random_byte_vector(0, words.len() as u8, 35_044, &words);
    let dfile = File::create(&fln).expect("Error generating testfile");
    let mut w = BufWriter::with_capacity(4096, dfile);
    w.write_all(&origin).expect("Error while writing file");
    println!("Success: {}", fln)
}
