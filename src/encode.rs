//! This module implements all necessary elements for the encoding of the data.
//! Most important is the actual `Encoder` struct which implements the `Write`
//! trait. This specific implementation of `Write` will represent the data in
//! the most compact form.
//!
//! # Inner workings of the `Encoder`
//! All write operations can only be performed on byte level. If the applications
//! needs to write `3u8` on disk, it would use 8 bits (i.e. `0000_0011`) instead of
//! the actually bare minimum of two bits i.e. `11`.

//! The Encoder implemented in this module buffers the output bits in a `u32`.
//! The actual output is only written on disk as soon as it has enough bits set,
//! that it looses no unused bits.

use std::io::{Error, ErrorKind, Write};

/// The Encoder<W> struct adds compressed streaming output for any writer.
///
/// `codewords` needs to be included into the `Encoder`, since the actual codeword
/// for a byte value is for sure longer than 8 bits. The `Write Trait` only takes
/// u8 values as input. If the codewords for any value is >255, it would through
/// an error since the maximum value for a `u8` is `255`. The codeword is also the
/// reason why `codewords` is an array of `usize` rather than `u8`.
pub struct Encoder<W: Write> {
    pub inner: W,
    codewords: [usize; 256],
    length: [usize; 256],
    buffer: u32,
    remaining_bits: usize,
}

impl<W: Write> Encoder<W> {
    /// Generate a new Encoder instance
    pub fn new(writer: W, codewords: [usize; 256], length: [usize; 256]) -> Encoder<W> {
        Encoder {
            inner: writer,
            length,
            codewords,
            buffer: 0x0000_0000,
            remaining_bits: 32,
        }
    }
}

impl<W: Write> Encoder<W> {
    fn put(&mut self) -> std::io::Result<usize> {
        let output = (self.buffer & 0xFF00_0000 >> 24) as u8;
        let no = self.inner.write(&[output])?;
        self.buffer <<= 8;
        self.remaining_bits += 8;
        Ok(no)
    }
    fn double(&mut self) -> std::io::Result<usize> {
        let no = self.inner.write(&[
            ((self.buffer & 0xFF00_0000) >> 24) as u8,
            ((self.buffer & 0x00FF_0000) >> 16) as u8,
        ])?;
        self.buffer <<= 16;
        self.remaining_bits += 16;
        Ok(no)
    }
    fn update_buffer(&mut self, val: usize) {
        self.buffer += (self.codewords[val] << self.remaining_bits) as u32;
    }
}

impl<W: Write> Write for Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut writeout = 0usize;
        for val in buf.iter() {
            let codelen = self.length[*val as usize];
            if codelen > 32 {
                return Err(Error::new(ErrorKind::InvalidData, "Codelen > 32"));
            }
            while codelen >= self.remaining_bits {
                writeout += self.put()?;
            }
            self.remaining_bits -= codelen;
            self.update_buffer(*val as usize);
            if self.buffer & 0x0000_FFFF > 0 {
                writeout += self.double()?;
            }
        }

        if self.buffer & 0x00FF_0000 > 0 {
            Ok(writeout + 2)
        } else if self.buffer & 0xFF00_0000 > 0 {
            Ok(writeout + 1)
        } else {
            Ok(writeout)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let writeout = [
            ((self.buffer & 0xFF00_0000) >> 24) as u8,
            ((self.buffer & 0x00FF_0000) >> 16) as u8,
            ((self.buffer & 0x0000_FF00) >> 8) as u8,
            (self.buffer & 0x0000_00FF) as u8,
        ];
        self.inner
            .write_all(&writeout[..(4 - self.remaining_bits / 8) as usize])?;
        self.inner.flush()?;
        Ok(())
    }
}

/// Calculate bit length of `val`
pub fn calculate_length(val: usize) -> usize {
    if val == 0 {
        return 1usize;
    }
    let mut size = 0usize;
    while val >> size > 0 {
        size += 1;
    }
    size
}

#[cfg(test)]
mod tests {
    use crate::encode::{calculate_length, Encoder};
    use std::io::Cursor;
    use std::io::Write;

    #[test]
    fn encode_numbers() {
        let words: Vec<u8> = vec![177, 112, 84, 143, 148, 195, 165, 206, 34, 10];
        let mut codewords = [0usize; 256];
        let mut length = [0usize; 256];
        for word in words.iter() {
            codewords[*word as usize] = *word as usize;
            length[*word as usize] = calculate_length(*word as usize);
        }
        let mut enc = Encoder::new(Cursor::new(Vec::new()), codewords, length);
        let output_bytes = enc.write(&words).expect("");
        enc.flush().expect("");

        assert_eq!(
            enc.inner.get_ref(),
            &[177, 225, 82, 62, 83, 14, 151, 58, 42]
        );
        assert_eq!(output_bytes, 9);
    }

    #[test]
    fn encode_stream() {
        let mut codewords = [0usize; 256];
        let mut length = [0usize;256];
        codewords[0] = 0;
        codewords[1] = 3;
        codewords[2] = 342;
        length[0] = calculate_length(0);
        length[1] = calculate_length(3);
        length[2] = calculate_length(342);
        let mut enc = Encoder::new(Cursor::new(Vec::new()), codewords, length);
        let output_bytes = enc.write(&[0, 1, 2]).expect("");
        enc.flush().expect("");

        assert_eq!(enc.inner.get_ref(), &[117, 96]);
        assert_eq!(output_bytes, 2);
    }

    #[test]
    fn binary_length() {
        assert_eq!(calculate_length(4), 3);
        assert_eq!(calculate_length(16), 5);
        assert_eq!(calculate_length(2), 2);
        assert_eq!(calculate_length(0), 1);
        assert_eq!(calculate_length(1), 1);
    }
}
