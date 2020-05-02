//! New decoding method for Huffman encoded data

#[derive(Debug)]
pub struct Decoder<I> {
    data: I,
    buffer: u64,
    vaultstatus: u64,
    vault: u64,
    sentinel: u64,
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

impl<I: Iterator<Item = u8>> Decoder<I> {
    pub fn new(mut iter: I, sentinel: u64) -> Self {
        Decoder {
            buffer: initiate_buffer(&mut iter),
            data: iter,
            vaultstatus: 0,
            vault : 0,
            sentinel: initiate_sentinel(sentinel),
        }
    }
}

impl<I: Iterator<Item = u8>> Decoder<I> {
    fn consume_buffer(&mut self) -> Option<u8> {
        unimplemented!()
    }
}

impl<I: Iterator<Item = u8>> Iterator for Decoder<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(val) = self.data.next() {
            print!("{:64b}", self.buffer);
            self.vault += (val as u64) << (64 - self.vaultstatus);
            let lookup_value = self.buffer >> (64 - self.sentinel);
            let (cut, sym) = get_cut_and_symbol(lookup_value);

            self.buffer <<= cut;
            self.buffer += self.vault >> (64 - cut);
            self.vaultstatus += 8;
            self.vaultstatus -= cut as u64;
            return Some(sym)
        }
        self.consume_buffer()
    }
}

use rand::Rng;

fn get_cut_and_symbol(_val: u64) -> (usize, u8) {
    let mut rng = rand::thread_rng();
    let cut: usize = rng.gen_range(1, 8);
    print!(" {} {}", cut, 32);
    (cut, 32)
}
