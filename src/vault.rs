//! New decoding method for Huffman encoded data

use std::collections::LinkedList;
use log::debug;

#[derive(Debug)]
pub struct Decoder<I> {
    data: I,
    buffer: u64,
    vaultstatus: u64,
    vault: u64,
    sentinel: u64,
    _reserve: LinkedList<u8>,
}

fn initiate_buffer(iter: &mut impl Iterator<Item = u8>) -> u64 {
    let mut result = 0u64;
    result += (iter.next().unwrap() as u64) << 56;
    result += (iter.next().unwrap() as u64) << 48;
    result += (iter.next().unwrap() as u64) << 40;
    result += (iter.next().unwrap() as u64) << 32;
    result += (iter.next().unwrap() as u64) << 24;
    result += (iter.next().unwrap() as u64) << 16;
    result += (iter.next().unwrap() as u64) <<  8;
    result += iter.next().unwrap() as u64;
    result
}

fn initiate_sentinel(sentinel: u64) -> u64 {
    assert!(sentinel <= 8);
    sentinel
}

fn initiate_reserve() -> LinkedList<u8> {
    LinkedList::<u8>::new()
    // let mut list = LinkedList::<u8>::new();
    // list.push_front(3);
    // list
}

impl<I: Iterator<Item = u8>> Decoder<I> {
    pub fn new(mut iter: I, sentinel: u64) -> Self {
        Decoder {
            buffer: initiate_buffer(&mut iter),
            data: iter,
            vaultstatus: 0,
            vault : 0,
            sentinel: initiate_sentinel(sentinel),
            _reserve: initiate_reserve(),
        }
    }
}

impl<I: Iterator<Item = u8>> Decoder<I> {
    fn consume_buffer(&mut self) -> Option<u8> {
        unimplemented!("Can not consume the buffer? No more data?")
    }
    fn empty_vault(&mut self) {
        unimplemented!("Can not rob the vault yet")
    }
}

impl<I: Iterator<Item = u8>> Iterator for Decoder<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(val) = self.data.next() {
            // move value to vault
            debug!("Buffer {:064b} Read byte {:08b}", self.buffer, val);
            self.vault += (val as u64) << (64 - self.vaultstatus - 8);
            self.vaultstatus += 8;

            // decode word
            let lookup_value = self.buffer >> (64 - self.sentinel);
            let (cut, sym) = get_cut_and_symbol(lookup_value);
            assert!(cut as u64 <= self.vaultstatus);

            // fill buffer from vault
            self.buffer <<= cut;
            self.buffer += self.vault >> (64 - cut);

            // update vault
            self.vault <<= cut;
            self.vaultstatus -= cut as u64;
            if self.vault & 0x0000_0000_FFFF_FFFF > 0 {
                self.empty_vault();
            };

            // TODO Might be optimised using .or_else()
            match self._reserve.pop_front() {
                Some(from_reserve) => {
                    self._reserve.push_back(sym);
                    return Some(from_reserve);
                },
                None => {
                    return Some(sym);
                }
            }
        }
        self.consume_buffer()
    }
}

use rand::Rng;

fn get_cut_and_symbol(_val: u64) -> (usize, u8) {
    let mut rng = rand::thread_rng();
    let cut: usize = rng.gen_range(1, 8);
    let sym: u8 =rng.gen();
    print!(" {} {}", cut, sym);
    (cut, sym)
}
