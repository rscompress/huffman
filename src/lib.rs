//! This crate implements the Huffman Coding algorithm specified
//! in the [master thesis](http://compression.ru/download/articles/huff/huffman_1952_minimum-redundancy-codes.pdf)
//! of David A. Huffman.
//!
//! # Huffman Coding
//! This implementation uses stream coding and canonical Huffman Coding. The file
//! is being read in chunks and coded using multi-pass encoding.
//!
//! ## Encoding Workflow
//! The algorithm first traverses the file and builds a histogram for each byte.
//! Afterwards it builds the codewords using a compact representation of the codewords
//! described in the above paper. A second traversal of file then encodes each
//! byte and saves it on disk.

//#[global_allocator]
//static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub const BUF: usize = 4096;
pub mod decode;
pub mod encode;
pub mod header;
pub mod huffman;
pub mod model;
pub mod stats;


fn u64_to_bytes(num: u64) -> [u8;8] {
    [
        (num >> 56) as u8,
        (num >> 48) as u8,
        (num >> 40) as u8,
        (num >> 32) as u8,
        (num >> 24) as u8,
        (num >> 16) as u8,
        (num >> 08) as u8,
        (num & 0xFF) as u8,
    ]
}

fn bytes_to_u64(bytes: &[u8]) -> u64 {
    assert_eq!(bytes.len(), 8);
    let mut result = 0u64;
    for (i, byte) in bytes.iter().enumerate() {
        result += (*byte as u64) << (56 - i * 8);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_to_bytes() {
        let input: Vec<u64> = vec![341,1,32425,23534134,3234159383273,534457654,8273839836383];
        let mut expected: Vec<[u8;8]> = Vec::new();
        expected.push(
            [0, 0, 0, 0, 0, 0, 1, 85]
        );
        expected.push(
            [0, 0, 0, 0, 0, 0, 0, 1]
        );
        expected.push(
            [0, 0, 0, 0, 0, 0, 126, 169]
        );
        expected.push(
            [0, 0, 0, 0, 1, 103, 26, 54]
        );
        expected.push(
            [0, 0, 2, 241, 2, 235, 210, 233]
        );
        expected.push(
            [0, 0, 0, 0, 31, 219, 45, 54]
        );
        expected.push(
            [0, 0, 7, 134, 103, 72, 204, 223]
        );

        for (num,expected) in input.into_iter().zip(expected.into_iter()) {
            assert_eq!(expected, u64_to_bytes(num))
        }
    }

    #[test]
    fn test_u64_to_bytes_roundtrip() {
        let input: Vec<u64> = vec![341,1,32425,23534134,3234159383273,534457654,8273839836383];
        for num in input {
            let bytes = u64_to_bytes(num);
            let reverse = bytes_to_u64(&bytes);
            assert_eq!(num, reverse)
        }
    }
}
