use super::encode::Encoder;
use super::header::Header;
use crate::model::Model;
use log::debug;
use std::collections::BTreeMap;
use std::io::Error;
use std::io::{Read, Write};
use succinct::bit_vec::BitVecMut;
use succinct::rank::BitRankSupport;
use succinct::rsdict::RsDict;
use succinct::BitVector;

pub mod vault;

pub struct Decoder<R: Read> {
    inner: R,
    buffer: u64,
    bits_left_in_buffer: u8,
    // bt: BTreeMap<usize, (u8, u8)>,
    table: Vec<(u8, u8)>,
    rbv: RsDict,
    sentinel: usize,
    writeout: usize,
    goalsbyte: usize,
    shift: u8,
}

impl<R: Read> Decoder<R> {
    pub fn new<W: Write, M: Model>(reader: R, encoder: &Encoder<W, M>) -> Self {
        Decoder {
            inner: reader,
            buffer: 0,
            bits_left_in_buffer: 64,
            table: {
                let (t, _) = prepare_lookup(&encoder.model.to_btreemap());
                t
            },
            rbv: {
                let (_, v) = prepare_lookup(&encoder.model.to_btreemap());
                v
            },
            sentinel: encoder.model.sentinel(),
            writeout: 0,
            goalsbyte: encoder.readbytes,
            shift: 64 - encoder.model.sentinel() as u8,
        }
    }
    pub fn from_header(header: Header, reader: R) -> Self {
        Decoder {
            inner: reader,
            buffer: 0,
            bits_left_in_buffer: 64,
            table: {
                let (t, _) = prepare_lookup(&header.btree);
                t
            },
            rbv: {
                let (_, v) = prepare_lookup(&header.btree);
                v
            },
            sentinel: header.sentinel,
            writeout: 0,
            goalsbyte: header.readbytes,
            shift: 64 - header.sentinel as u8,
        }
    }
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let nbytes = (self.goalsbyte - self.writeout).min(buf.len());
        let mut consumed = 0;
        let mut iter = self.inner.by_ref().bytes(); //.skip(self.pos);
        while let Some(Ok(val)) = iter.next() {
            // debug!("Reading {}", val);
            if self.bits_left_in_buffer >= 16 {
                // There is still room for a byte in the buffer -> fill it up
                let v = (val as u64) << (self.bits_left_in_buffer - 8);
                self.buffer += v;
                self.bits_left_in_buffer -= 8;
                debug!(
                    "Add: {:064b} BLE {:2}",
                    self.buffer, self.bits_left_in_buffer
                );
                continue;
            }
            while (64 - self.bits_left_in_buffer - 8) as usize >= self.sentinel && consumed < nbytes
            {
                // Actual decoding of the values from the buffer. As long as the consumed is less than nbytes
                // or the buffer needs to be filled up again
                let searchvalue = self.buffer >> self.shift;
                let pos = self.rbv.rank1(searchvalue + 1) as usize - 1;
                let (sym, length) = self.table[pos];
                // debug!("Decoded {} {} {}", sym, length, consumed);
                buf[consumed] = sym;
                consumed += 1;
                self.writeout += 1;
                self.buffer <<= length;
                debug!(
                    "Rem: {:064b} SYM {:b} LEN {} SVA {} CNS {}",
                    self.buffer, sym, length, searchvalue, consumed
                );
                self.bits_left_in_buffer += length;
            }
            debug!(
                "Out: {:064b} BLE {} >",
                self.buffer, self.bits_left_in_buffer
            );
            // Do not forget to add the current value `val` into the buffer
            let v = (val as u64) << (self.bits_left_in_buffer - 8);
            self.buffer += v;
            self.bits_left_in_buffer -= 8;
            if consumed >= nbytes {
                // If consumed was the reason for the break above, return written bytes
                // otherwise continue
                debug!(
                    "Out: {:064b} BLE {} >",
                    self.buffer, self.bits_left_in_buffer
                );
                return Ok(consumed);
            }
        }
        // debug!("{} {} {} {}", consumed, self.writeout, self.goalsbyte, nbytes);
        // assert!(self.goalsbyte - self.writeout == nbytes);
        while consumed < nbytes {
            let searchvalue = self.buffer >> self.shift;
            let pos = self.rbv.rank1(searchvalue + 1) as usize - 1;
            let (sym, length) = self.table[pos];
            // debug!("{} {:?} {}", consumed, buf, sym);
            buf[consumed] = sym;
            consumed += 1;
            self.writeout += 1;
            self.buffer <<= length;
            self.bits_left_in_buffer += length;
        }
        Ok(consumed)
    }
}

pub fn prepare_lookup(bt: &BTreeMap<usize, (u8, u8)>) -> (Vec<(u8, u8)>, RsDict) {
    debug!("Btree from encoder: {:?}", bt);
    let table: Vec<(u8, u8)> = bt.values().cloned().collect();
    let keys: Vec<usize> = bt.keys().cloned().collect();
    let m: usize = keys[keys.len() - 1];
    let mut bv: BitVector<u64> = BitVector::with_fill(m as u64 + 1, false);
    for k in keys {
        bv.set_bit(k as u64, true);
    }
    let mut jbv = RsDict::new();
    for bit in bv.iter() {
        jbv.push(bit);
    }

    (table, jbv)
}

pub fn search_key_or_next_small_key(tree: &BTreeMap<usize, (u8, u8)>, key: usize) -> (u8, u8) {
    let mut iter = tree.range(..key + 1);

    if let Some((_, v)) = iter.next_back() {
        *v
    } else {
        panic!("Panic!!!!")
    }
}

pub fn read(data: &[u8], model: &impl Model, goalsbyte: usize) -> Vec<u8> {
    let mut buffer: u64 = 0;
    let mut bits_left_in_buffer = 64u8;
    let bt = model.to_btreemap();
    debug!("{:?}", &bt);
    let (table, rbv) = prepare_lookup(&model.to_btreemap());
    let s = model.sentinel();
    let shift = 64 - s;
    let mut result: Vec<u8> = Vec::with_capacity(data.len());
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
        while (64 - bits_left_in_buffer) as usize >= s {
            let searchvalue = buffer >> shift;
            let pos = rbv.rank1(searchvalue + 1) as usize - 1;
            let (sym, length) = table[pos];
            result.push(sym);
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
        let pos = rbv.rank1(searchvalue + 1) as usize - 1;
        let (sym, length) = table[pos];
        result.push(sym);
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
    use crate::huffman::encode::{calculate_length, Encoder};
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
