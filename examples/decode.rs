use rscompress_huffman::encode::{Encoder};
use rscompress_huffman::huffman::Huffman;
use rscompress_huffman::stats::generate_random_byte_vector;
// use std::fs::File;
// use std::io::prelude::*;
// use std::io::{BufRead, BufReader, BufWriter};
use std::io::{Cursor, Write};
use log::debug;
use std::time::Instant;

#[allow(unreachable_code)]
fn main() {
    env_logger::init();
    // Generate Huffman Encoder
    let words: Vec<u8> = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
    let mut histogram = [0usize; 256];
    for i in 0..words.len() {
        histogram[i] = words[i] as usize;
    }

    // Encode `words`
    // let origin : Vec<u8> = vec![0,9,9,9,9,9,7,0,7,4,9,9,0,0,0,4,0];
    for i in 0..100 {
        let origin: Vec<u8> = generate_random_byte_vector(0, 9, 703094440);
        // let mut origin: Vec<u8> = Vec::new();
        // let mut r = BufReader::with_capacity(4096, File::open("erorrs.raw").unwrap());
        // r.read_to_end(&mut origin).unwrap();
        let h = Huffman::from_histogram(&histogram);
        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
        enc.write(&origin).expect("");
        enc.flush().expect("");
        let fill = enc.fillbits.unwrap();
        let now = Instant::now();
        let decoded_words = read(enc.inner.get_ref(), &h, fill, enc.readbytes, &origin);
        println!("Time: {}", now.elapsed().as_secs_f32());
        assert_eq!(decoded_words, origin);
        println!("{} successful", i)
    }
}

use std::collections::BTreeMap;

fn search_key_or_next_small_key(tree: &BTreeMap<usize, (u8, u8)>, key: usize) -> (u8, u8) {
    let mut iter = tree.range(..key+1);
    let prev = iter.next_back();
    let prev_prev = iter.next_back();

    if let Some((_, v)) = prev {
        return *v
    } else {
        let r = prev_prev.unwrap();
        return *r.1
    }
}


fn decode_next(searchvalue: u64, bt: &BTreeMap<usize, (u8, u8)>, result: &mut Vec<u8>) -> u8 {
    let (sym,length) = search_key_or_next_small_key(&bt, searchvalue as usize);
    result.push(sym);
    length
}


use rscompress_huffman::model::Model;

pub fn read(data: &[u8], model: &impl Model, fillbits: u8, goalsbyte: usize, origin: &[u8]) -> Vec<u8> {
    let mut buffer: u64 = 1 << 63;
    let mut bits_left_in_buffer = 63u8;
    let bt = model.to_btreemap();
    debug!("{:?}", &bt);
    let s = model.sentinel();
    let shift = 64 - s;
    let mut result: Vec<u8> = Vec::with_capacity(data.len());
    let mut first = true;
    let mut writeout = 0;
    for val in data.iter() {
        if bits_left_in_buffer >= 8 {
            // fill buffer
            let v = (*val as u64) << (bits_left_in_buffer - 8);
            buffer += v;
            debug!("     New Buffer: {:b}", buffer);
            bits_left_in_buffer -= 8;
            continue
        }
        // buffer filled
        if first {
            buffer <<= 1;
            first = false;
            bits_left_in_buffer += 1;
        }
        while (64 - bits_left_in_buffer) as usize >= s {
            let searchvalue = buffer >> shift;
            let length = decode_next(searchvalue, &bt, &mut result);
            // let s = result[writeout];
            // let exp = origin[writeout];
            // if s != exp {
            //     println!("Oh oh {}", writeout);
            // }
            debug!("{}: Buffer: {:64b} Select: {:b} Decoded to: {} Shift buffer: {}",
            writeout, buffer, searchvalue, result[writeout], length);
            writeout += 1;
            // let (sym,length) = search_key_or_next_small_key(&bt, searchvalue as usize);
            // result.push(sym);
            buffer <<= length;
            bits_left_in_buffer += length;
        }
        debug_assert!(bits_left_in_buffer >= 8, "Not enough bits left in buffer for val");
        // buffer += (*val as u64) << bits_left_in_buffer - 8;
        debug!("     New Buffer: {:64b}", buffer);
        let v = (*val as u64) << (bits_left_in_buffer - 8);
        buffer += v;
        bits_left_in_buffer -= 8;
    }
    debug!("GB {}", goalsbyte-writeout);
    // consume bits in buffer
    while goalsbyte > writeout {
        let searchvalue = buffer >> shift;
        let length = decode_next(searchvalue, &bt, &mut result);
        writeout += 1;
        // let (sym,length) = search_key_or_next_small_key(&bt, searchvalue as usize);
        // result.push(sym);
        buffer <<= length;
        bits_left_in_buffer += length;
    }
    result
}
