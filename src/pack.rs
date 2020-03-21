
use std::io::{Write};

pub struct Pack<W: Write> {
    pub inner: W,
    buffer : u32,
    remainder: usize,
    codewords: [usize;256],
    lengths: [usize;256],
}

impl<W: Write> Write for Pack<W>{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let result: Vec<u8> = buf.iter().filter_map(|&a| self.pack(a)).flatten().collect();
        self.inner.write(result.as_ref())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let result = self.last();
        self.inner.write_all(result.as_ref())?;
        self.inner.flush()?;
        Ok(())
    }
}

impl<W: Write> Pack<W> {
    pub fn new(inner: W, codewords: [usize;256], lengths: [usize;256]) -> Self {
        Pack { inner, buffer: 0, remainder: 32 , codewords, lengths}
    }

    fn pack(&mut self, sym: u8) -> Option<Vec<u8>> {
        let code = self.codewords[sym as usize];
        let len = self.lengths[sym as usize];
        if len < self.remainder {
            self.save(code, len);
            return None;
        }
        let result = self.writeout();
        self.save(code,len);
        Some(result)
    }

    fn save(&mut self, code: usize, len: usize){
        self.buffer <<= len;
        self.buffer += code as u32;
        self.remainder -= len;
    }

    fn writeout(&mut self) -> Vec<u8> {
        let mut result : Vec<u8> = Vec::with_capacity(4);
        while 32 - self.remainder > 8 {
            let s = 32 - self.remainder - 8;
            result.push((self.buffer >> s) as u8);
            self.buffer <<= self.remainder + 8;
            self.buffer >>= self.remainder + 8;
            self.remainder += 8;
        }
        result
    }

    fn last(&mut self) -> Vec<u8> {
        let mut result = self.writeout();
        result.push(self.buffer as u8);
        result
    }

}

#[cfg(test)]
mod tests {
    use crate::encode::calculate_length;
    use std::io::Cursor;
    use super::*;

#[test]
fn encode_numbers_pack() {
    let words: Vec<u8> = vec![177, 112, 84, 143, 148, 195, 165, 206, 34, 10];
    let mut codewords = [0usize; 256];
    let mut length = [0usize; 256];
    for word in words.iter() {
        codewords[*word as usize] = *word as usize;
        length[*word as usize] = calculate_length(*word as usize);
    }
    let mut enc = Pack::new(Cursor::new(Vec::new()), codewords, length);
    let output_bytes = enc.write(&words).expect("");
    enc.flush().expect("");


    assert_eq!(
        enc.inner.get_ref(),
        &[177, 225, 82, 62, 83, 14, 151, 58, 42]
    );
    assert_eq!(output_bytes, 9);
}
}
