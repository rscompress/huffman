//! New decoding method for Huffman encoded data
//!
//! # Inner workings of the `Decoder`
//! The main elements of the `Decoder` are the `buffer`, `vault`, and
//! `sentinel`. The first `sentinel` bits of the `buffer` are read and
//! decoded (call to `get_cut_and_symbol()`).
//! This decoding process returns the number of bits evaluated (`cut`)
//! and the decoded symbol. Afterwards, the `cut` MSB from the buffer will be
//! removed. Next, the `cut` LSB from the buffer will be filled via the `cut`
//! MSB from the `vault`. The current symbol will be written at the end of
//! the vault. This may cause the vault to overfill. Should the vault be
//! close to overfilling, values from the buffer are decoded into the
//! `_reserve`. This causes the `vault` to be emptied, since the buffer gets
//! refilled with the `vault`.

use crate::huffman::decode::symboltable;
use crate::model::Model;
use log::debug;
use std::collections::LinkedList;
use std::io::Read;

const MAX_VAULT: u64 = 52;
const MIN_VAULT: u64 = 16;

/// The Decoder<I> struct decodes iterable data structures
#[derive(Debug)]
pub struct Decoder<R> {
    inner: R,
    buffer: u64,
    vault: u64,
    sentinel: u64,
    remaining_outputbytes: u64,
    symboltable: symboltable::SymbolTable,
    _reserve: LinkedList<u8>,
    _vaultstatus: u64,
    _bufferstatus: u64,
}

fn initiate_buffer<R: Read>(reader: &mut R) -> (u64, u64) {
    let mut result = 0u64;
    let mut buf: [u8; 8] = [0; 8];
    let nbytes = reader.read(&mut buf).expect("Cannot read");

    for i in 0..nbytes {
        result += (buf[i] as u64) << (56 - i * 8)
    }
    (result, nbytes as u64)
}

fn initiate_sentinel(sentinel: u64) -> u64 {
    // TODO Remove constraint
    assert!(sentinel <= 8);
    sentinel
}

fn initiate_reserve() -> LinkedList<u8> {
    LinkedList::<u8>::new()
}

impl<R: Read> Decoder<R> {
    pub fn new<M: Model>(mut reader: R, model: &M, output: u64) -> Self {
        let (buffer, bufferstatus) = initiate_buffer(&mut reader);
        Decoder {
            buffer,
            inner: reader,
            _vaultstatus: 0,
            _bufferstatus: bufferstatus,
            vault: 0,
            sentinel: initiate_sentinel(model.sentinel() as u64),
            _reserve: initiate_reserve(),
            remaining_outputbytes: output,
            symboltable: symboltable::SymbolTable::from_btree(&model.to_btreemap()),
        }
    }
    fn consume_buffer(&mut self) -> Option<u8> {
        debug!(
            "Consuming b{:064b} v{:064b} {} {}",
            self.buffer, self.vault, self._vaultstatus, self._bufferstatus
        );
        let lookup_value = self.buffer >> (64 - self.sentinel);
        let (cut, sym) = self.symboltable.get_cut_and_symbol(lookup_value);
        if cut as u64 > self._bufferstatus {
            return None;
        }
        if cut <= self._vaultstatus as usize {
            // normal process
            self.buffer <<= cut;
            self.buffer += self.vault >> (64 - cut);
            self.vault <<= cut;
            self._vaultstatus -= cut as u64;
            return Some(sym);
        } else if self._vaultstatus > 0 {
            // TODO Same as above might be just to a min(cut,vault)
            self.buffer <<= cut;
            self.buffer += self.vault >> (64 - self._vaultstatus);
            self._bufferstatus -= cut as u64 - self._vaultstatus;
            self.vault <<= self._vaultstatus;
            self._vaultstatus -= self._vaultstatus;
            return Some(sym);
        } else {
            self.buffer <<= cut;
            self._bufferstatus -= cut as u64;
            return Some(sym);
        }
    }
    fn empty_vault(&mut self) {
        while self._vaultstatus > MIN_VAULT {
            let lookup_value = self.buffer >> (64 - self.sentinel);
            let (cut, sym) = self.symboltable.get_cut_and_symbol(lookup_value);
            assert!(cut as u64 <= self._vaultstatus);
            self.buffer <<= cut;
            self.buffer += self.vault >> (64 - cut);
            self.vault <<= cut;
            self._vaultstatus -= cut as u64;
            self._reserve.push_back(sym);
        }
    }
    fn decode(&mut self, symbol: Option<u8>) -> Option<u8> {
        if self.remaining_outputbytes == 0 {
            debug!("Finished decoding");
            debug!("Buffer {:064b} Vault {:064b}", self.buffer, self.vault);
            return None;
        }
        if let Some(val) = symbol {
            // Inner data source still not empty
            debug!(
                "Buffer {:064b} Read byte {:08b} {:?}",
                self.buffer, val, self._reserve
            );

            // Check vault fill
            if self._vaultstatus > MAX_VAULT {
                self.empty_vault();
                debug!("Reserve {:?}", self._reserve)
            };

            // TODO Starting here a lot of overlap with empty_vault()
            // move value to vault
            debug!("{:?} {:?}", self.vault, self._vaultstatus);
            self.vault += (val as u64) << (64 - self._vaultstatus - 8);
            self._vaultstatus += 8;

            // decode word
            let lookup_value = self.buffer >> (64 - self.sentinel);
            let (cut, sym) = self.symboltable.get_cut_and_symbol(lookup_value);
            assert!(cut as u64 <= self._vaultstatus);

            // fill buffer from vault
            self.buffer <<= cut;
            self.buffer += self.vault >> (64 - cut);

            // update vault
            self.vault <<= cut;
            self._vaultstatus -= cut as u64;

            // TODO Might be optimised using .or_else()
            match self._reserve.pop_front() {
                Some(from_reserve) => {
                    self._reserve.push_back(sym);
                    self.remaining_outputbytes -= 1;
                    return Some(from_reserve);
                }
                None => {
                    self.remaining_outputbytes -= 1;
                    return Some(sym);
                }
            }
        } else if let Some(reserve) = self._reserve.pop_front() {
            // Inner data source empty. First output reserve
            self.remaining_outputbytes -= 1;
            return Some(reserve);
        } else {
            // Finish output by consuming buffer
            self.remaining_outputbytes -= 1;
            self.consume_buffer()
        }
    }
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let nbytes = buf.len().min(self.remaining_outputbytes as usize);
        if nbytes != 0 {
            self.inner.read(&mut buf[..nbytes]).unwrap();
            for i in 0..nbytes {
                buf[i] = self.decode(Some(buf[i])).unwrap()
            }
        }
        Ok(nbytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::huffman::encode::Encoder;
    use crate::huffman::Huffman;
    use std::io::{Cursor, Write};

    fn encode_str(sentence: &str) -> (Vec<u8>, Vec<u8>, Huffman) {
        let data = sentence.as_bytes().to_vec();
        let h = Huffman::from_slice(data.as_slice());

        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
        let _output_bytes = enc.write(&data).expect("");
        enc.flush().expect("");
        let encoded_data: Vec<u8> = enc.inner.get_ref().iter().map(|&x| x).collect();
        (data, encoded_data, h)
    }

    fn encode_file(mut file: std::fs::File) -> (Vec<u8>, Vec<u8>, Huffman) {
        let mut data: Vec<u8> = Vec::new(); //file.as_bytes().to_vec();
        file.read_to_end(&mut data).unwrap();
        let h = Huffman::from_slice(data.as_slice());

        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
        let _output_bytes = enc.write(&data).expect("");
        enc.flush().expect("");
        let encoded_data: Vec<u8> = enc.inner.get_ref().iter().map(|&x| x).collect();
        (data, encoded_data, h)
    }

    fn roundtrip_decode_blockwise(sentence: &str, blocksize: usize) {
        let (data, encoded_data, h) = encode_str(sentence);
        println!(
            "Encoded {:?} ({}) [{}]",
            encoded_data,
            sentence,
            sentence.len()
        );
        let mut decoder = Decoder::new(Cursor::new(encoded_data), &h, data.len() as u64);
        let mut decoded_data = vec![0u8; blocksize];
        let mut nbytes = 1;
        let mut iteration = 0;

        // TODO: read_to_end() does not seem to work
        while nbytes > 0 {
            nbytes = decoder.read(&mut decoded_data).unwrap();
            assert_eq!(decoded_data[..nbytes], data[iteration..iteration + nbytes]);
            iteration += nbytes;
            println!("{:?} {}", decoded_data, nbytes);
        }
    }

    fn roundtrip_decode_at_once(sentence: &str) {
        let (data, encoded_data, h) = encode_str(sentence);
        println!("Encoded {:?} ({})", encoded_data, sentence);
        let mut decoder = Decoder::new(Cursor::new(encoded_data), &h, data.len() as u64);
        let mut decoded_data: Vec<u8> = Vec::new();

        let nbytes = decoder.read_to_end(&mut decoded_data).unwrap();
        println!("Decoded {:?}", decoded_data);
        println!("{:?}", decoded_data);
        assert_eq!(data.len(), nbytes);
        assert_eq!(data, decoded_data);
    }
    fn roundtrip_decode_at_once_file(file: std::fs::File) {
        let (data, encoded_data, h) = encode_file(file);
        println!("Encoded {:?}", encoded_data);
        let mut decoder = Decoder::new(Cursor::new(encoded_data), &h, data.len() as u64);
        let mut decoded_data: Vec<u8> = Vec::new();

        let nbytes = decoder.read_to_end(&mut decoded_data).unwrap();
        println!("Decoded {:?}", decoded_data);
        println!("{:?}", decoded_data);
        assert_eq!(data.len(), nbytes);
        assert_eq!(data, decoded_data);
    }

    #[test]
    fn roundtrip_blockwise() {
        roundtrip_decode_blockwise("This is a lovely text in a big world", 10);
        roundtrip_decode_blockwise("aaafaaaaaaaaa", 5);
        roundtrip_decode_blockwise("aaaaaa", 2);
    }

    #[test]
    fn roundtrip_at_once() {
        roundtrip_decode_at_once("This is a lovely text in a big world");
        roundtrip_decode_at_once("aaafaaaaaaaaa");
        roundtrip_decode_at_once("aaaaaa");
    }
}
