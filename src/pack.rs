#![deprecated(since = "0.0.1", note = "Please use 'encode' instead")]

use std::io::Write;

pub struct Pack<W: Write> {
    pub inner: W,
    buffer: u32,
    remainder: usize,
    codewords: [usize; 256],
    lengths: [usize; 256],
}

impl<W: Write> Write for Pack<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for sym in buf.iter() {
            let code = self.codewords[*sym as usize];
            let len = self.lengths[*sym as usize];
            while len > self.remainder {
                let s = 32 - self.remainder - 8;
                self.inner.write(&[(self.buffer >> s) as u8]).expect("???");
                self.buffer <<= self.remainder + 8;
                self.buffer >>= self.remainder + 8;
                self.remainder += 8;
            }
            self.save(code, len);
        }
        Ok(1)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.last();
        self.inner.flush()?;
        Ok(())
    }
}

impl<W: Write> Pack<W> {
    pub fn new(inner: W, codewords: [usize; 256], lengths: [usize; 256]) -> Self {
        Pack {
            inner,
            buffer: 0,
            remainder: 32,
            codewords,
            lengths,
        }
    }

    fn save(&mut self, code: usize, len: usize) -> usize {
        self.buffer <<= len;
        self.buffer += code as u32;
        self.remainder -= len;
        1
    }

    fn writeout(&mut self) -> std::io::Result<usize> {
        while 32 - self.remainder > 8 {
            let s = 32 - self.remainder - 8;
            self.inner.write(&[(self.buffer >> s) as u8]).expect("???");
            self.buffer <<= self.remainder + 8;
            self.buffer >>= self.remainder + 8;
            self.remainder += 8;
        }
        Ok(1)
    }

    fn last(&mut self) {
        self.writeout().expect("???");
        self.inner.write(&[self.buffer as u8]).expect("???");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::calculate_length;
    use std::io::Cursor;

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
        enc.write(&words).expect("");
        enc.flush().expect("");

        assert_eq!(
            enc.inner.get_ref(),
            &[177, 225, 82, 62, 83, 14, 151, 58, 42]
        );
        // assert_eq!(output_bytes, 9);
    }
}
