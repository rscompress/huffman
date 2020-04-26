//! Module for definition of the header file for Huffman Encoding
//! The header file are information needed to concstruct a proper Decoder.
//! The decoder can then be created using the `from_header` method.
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use log::debug;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Header {
    pub btree: BTreeMap<usize, (u8, u8)>,
    pub sentinel: usize,
    pub readbytes: usize,
}

use crate::encode::Encoder;
use crate::model::Model;
use std::convert::From;
use std::io::Write;

impl<'a, W: Write, M: Model> From<Encoder<'a, W, M>> for Header {
    fn from(enc: Encoder<'a, W, M>) -> Self {
        Header {
            btree: enc.model.to_btreemap(),
            sentinel: enc.model.sentinel(),
            readbytes: enc.readbytes,
        }
    }
}

impl Header {
    pub fn to_binary(&self) -> Vec<u8> {
        let result = serialize(&self).unwrap();
        debug!("Header serialisation size: {} bytes", result.len());
        result
    }
    pub fn from_binary(vec: &[u8]) -> Self {
        deserialize(vec).unwrap()
    }
    pub fn to_file(&self, filename: &str) {
        unimplemented!();
    }
    pub fn from_file(filename: &str) -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::encode::{calculate_length, Encoder};
    use crate::huffman::Huffman;
    use std::io::{Cursor};
    use super::*;

    #[test]
    fn serialisation_roundtrip() {
        // Generate Huffman Encoder
        let words: Vec<u8> = vec![177, 112, 84, 143, 148, 195, 165, 206, 34, 10];
        let mut codewords = [0usize; 256];
        let mut length = [0usize; 256];
        for word in words.iter() {
            codewords[*word as usize] = *word as usize;
            length[*word as usize] = calculate_length(*word as usize);
        }
        let h = Huffman::new(codewords, length);
        let enc = Encoder::new(Cursor::new(Vec::new()), &h);

        let head = Header::from(enc);
        let temp = head.to_binary();
        let new_head = Header::from_binary(&temp);

        assert_eq!(new_head, head)
    }
}
