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

impl<I: Iterator<Item = u8>> Decoder<I> {
    pub fn new(mut iter: I, sentinel: u64) -> Self {
        Decoder {
            buffer: initiate_buffer(&mut iter),
            data: iter,
            vaultstatus: 0,
            vault : 0,
            sentinel,
        }
    }
}

impl<I: Iterator<Item = u8>> Iterator for Decoder<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next()
    }
}
