use rscompress_huffman::encode::{read, Encoder};
use rscompress_huffman::huffman::Huffman;
use rscompress_huffman::stats::generate_random_byte_vector;
use std::io::{Cursor, Write};

fn main() {
    // Generate Huffman Encoder
    let words: Vec<u8> = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
    let mut histogram = [0usize; 256];
    for i in 0..words.len() {
        histogram[i] = words[i] as usize;
    }
    let h = Huffman::from_histogram(&histogram);
    let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);

    // Encode `words`
    // let origin : Vec<u8> = vec![0,9,9,9,9,9,7,0,7,4,9,9,0,0,0,4,0];
    let origin: Vec<u8> = generate_random_byte_vector(0, 9, 90094440);
    enc.write(&origin).expect("");
    enc.flush().expect("");
    if let Some(fill) = enc.fillbits {
        let decoded_words = read(enc.inner.get_ref(), &h, fill);
        // dbg!(enc.inner.get_ref());
        // print!("Encoded: ");
        // for k in enc.inner.get_ref().iter() {
        //     print!("{:b} ", k)
        // }
        // assert_eq!(decoded_words, origin);
        if decoded_words != origin {
            let mut errors = 0;
            for (i, w) in decoded_words.iter().zip(origin.iter()).enumerate() {
                if w.0 != w.1 {
                    println! {"Error@{}/{}: {} vs. {}", i, decoded_words.len(), w.0,w.1}
                    errors += 1;
                    if errors > 10 {
                        break;
                    }
                }
            }
        } else {
            println!("Success!");
        }
    }
}
