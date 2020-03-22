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
pub mod encode;
pub mod pack;
pub mod huffman;
pub mod stats;
pub mod model;
