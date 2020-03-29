use crate::model::Model;
use log::debug;
use std::collections::BTreeMap;

pub fn search_key_or_next_small_key(tree: &BTreeMap<usize, (u8, u8)>, key: usize) -> (u8, u8) {
    let mut iter = tree.range(..key + 1);

    if let Some((_, v)) = iter.next_back() {
        *v
    } else {
        panic!("Panic!!!!")
    }
}

fn decode_next(searchvalue: u64, bt: &BTreeMap<usize, (u8, u8)>, result: &mut Vec<u8>) -> u8 {
    let (sym, length) = search_key_or_next_small_key(&bt, searchvalue as usize);
    result.push(sym);
    length
}

pub fn read(data: &[u8], model: &impl Model, goalsbyte: usize) -> Vec<u8> {
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
            continue;
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
            debug!(
                "{}: Buffer: {:64b} Select: {:b} Decoded to: {} Shift buffer: {}",
                writeout, buffer, searchvalue, result[writeout], length
            );
            writeout += 1;
            // let (sym,length) = search_key_or_next_small_key(&bt, searchvalue as usize);
            // result.push(sym);
            buffer <<= length;
            bits_left_in_buffer += length;
        }
        debug_assert!(
            bits_left_in_buffer >= 8,
            "Not enough bits left in buffer for val"
        );
        // buffer += (*val as u64) << bits_left_in_buffer - 8;
        debug!("     New Buffer: {:64b}", buffer);
        let v = (*val as u64) << (bits_left_in_buffer - 8);
        buffer += v;
        bits_left_in_buffer -= 8;
    }
    debug!("GB {}", goalsbyte - writeout);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{calculate_length, Encoder};
    use crate::huffman::Huffman;
    use std::io::{Cursor, Write};

    #[test]
    fn decode_numbers() {
        // Generate Huffman Encoder
        let words: Vec<u8> = vec![177, 112, 84, 143, 148, 195, 165, 206, 34, 10];
        let mut codewords = [0usize; 256];
        let mut length = [0usize; 256];
        for word in words.iter() {
            codewords[*word as usize] = *word as usize;
            length[*word as usize] = calculate_length(*word as usize);
        }
        let h = Huffman::new(codewords, length);
        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);

        // Encode `words`
        enc.write(&words).expect("");
        enc.flush().expect("");
        let decoded_words = read(enc.inner.get_ref(), &h, enc.readbytes);
        assert_eq!(words.as_slice(), decoded_words.as_slice());
    }

    #[test]
    fn decode_numbers_histogram_encoded() {
        let words: Vec<u8> = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
        let mut histogram = [0usize; 256];
        for i in 0..words.len() {
            histogram[i] = words[i] as usize;
        }
        let h = Huffman::from_histogram(&histogram);
        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);

        // Encode `words`
        let origin: Vec<u8> = vec![
            0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 7, 0, 7, 4, 9, 9, 0, 0, 0, 4, 0,
        ];
        enc.write(&origin).expect("");
        enc.flush().expect("");
        let decoded_words = read(enc.inner.get_ref(), &h, enc.readbytes);
        assert_eq!(origin.as_slice(), decoded_words.as_slice());
    }
}
