use super::DecoderError;
use crate::huffman::decode::symboltable;
use crate::model::Model;
use log::warn;
use std::io::Write;

const VAULT_MAX: u64 = 6;
const VAULT_MIN: u64 = 2;

#[derive(Debug)]
pub struct Decoder<W> {
    pub inner: W, // TODO reverse for publication
    buffer: u64,
    vault: u64,
    bufferstatus: i32,
    vaultstatus: i32,
    remaining_outputbytes: u64,
    symboltable: symboltable::SymbolTable,
    sentinel: u64,
}

impl<W: Write> std::fmt::Display for Decoder<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Decoder [ buffer: {0:064b} [{1}], vault: {2:064b} [{3}], remaining_outputbytes: {4}, sentinel: {5}]", self.buffer, self.bufferstatus, self.vault, self.vaultstatus, self.remaining_outputbytes, self.sentinel)
    }
}

impl<W: Write> Decoder<W> {
    pub fn new(writer: W, model: &dyn Model, outputbytes: u64) -> Self {
        Decoder {
            inner: writer,
            buffer: 0,
            vault: 0,
            bufferstatus: 0,
            vaultstatus: 0,
            remaining_outputbytes: outputbytes,
            symboltable: symboltable::SymbolTable::from_btree(&model.to_btreemap()),
            sentinel: model.sentinel() as u64,
        }
    }
    fn add_to_buffer(&mut self, byte: u8) -> Result<(), DecoderError> {
        assert!(self.bufferstatus <= 64);
        if self.bufferstatus <= 56 {
            self.buffer += (byte as u64) << (64 - self.bufferstatus - 8);
            self.bufferstatus += 8;
        } else {
            let bufferpart = 64 - self.bufferstatus;
            let vaultpart = 8 - bufferpart;
            self.buffer += byte as u64 >> (8 - bufferpart);
            self.bufferstatus += bufferpart;
            self.add_to_vault_partially(byte, vaultpart as usize)?;
        }
        Ok(())
    }
    fn add_to_vault_partially(&mut self, byte: u8, parts: usize) -> Result<(), DecoderError> {
        let value = byte & (2u8.pow(parts as u32) - 1);
        if self.vaultstatus <= ((VAULT_MAX as i32 * 8) - parts as i32) {
            self.vault += (value as u64) << (64 - self.vaultstatus - parts as i32);
            self.vaultstatus += parts as i32;
        } else {
            warn!(
                "Cannot add to vault partially: Needed [{}], Available [{}]",
                parts,
                VAULT_MAX as i32 * 8 - self.vaultstatus
            );
            self.consume_vault()?;
            self.add_to_vault_partially(byte, parts)?;
        }
        Ok(())
    }
    fn add_to_vault(&mut self, byte: u8) -> Result<(), DecoderError> {
        if self.vaultstatus <= VAULT_MAX as i32 * 8 {
            self.vault += (byte as u64) << (64 - self.vaultstatus - 8);
            self.vaultstatus += 8;
        } else {
            warn!(
                "Cannot add to vault: Needed [{}], Available [{}] {}",
                8,
                VAULT_MAX as i32 * 8 - self.vaultstatus,
                self.remaining_outputbytes
            );
            self.consume_vault()?;
            self.add_to_vault(byte)?;
        }
        Ok(())
    }
    fn consume_vault(&mut self) -> Result<(), DecoderError> {
        while self.vaultstatus >= (VAULT_MIN as i32) * 8 {
            self.put()?;
        }
        Ok(())
    }
    fn put(&mut self) -> Result<(), DecoderError> {
        if self.remaining_outputbytes == 0 {
            return Err(DecoderError::AllOutputAlreadyWritten);
        }
        assert!(self.bufferstatus >= self.sentinel as i32);
        let lookup_value = self.buffer >> (64 - self.sentinel);
        let (cut, sym) = self.symboltable.get_cut_and_symbol(lookup_value);
        self.inner.write(&[sym])?;
        self.remaining_outputbytes -= 1;
        self.buffer <<= cut;
        self.bufferstatus -= cut as i32;
        self.fill_buffer_from_vault()?;
        Ok(())
    }
    fn fill_buffer_from_vault(&mut self) -> Result<(), DecoderError> {
        let needed = 64 - self.bufferstatus;
        if needed == 0 {
            return Ok(());
        } else if self.vaultstatus >= needed {
            self.buffer += self.vault >> (64 - needed);
            self.bufferstatus += needed;
            self.vault <<= needed;
            self.vaultstatus -= needed;
        } else if self.vaultstatus > 0 {
            self.buffer += self.vault >> (64 - self.vaultstatus);
            self.bufferstatus += self.vaultstatus;
            self.vault <<= self.vaultstatus;
            self.vaultstatus -= self.vaultstatus;
        } else {
            warn!(
                "Buffer will not be filled from vault!: Needed [{}], Available [{}]",
                needed, self.vaultstatus
            )
        }
        Ok(())
    }
}

impl<W: Write> Write for Decoder<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        if self.remaining_outputbytes == 0 {
            return Ok(0);
        }
        let nbytes = self.remaining_outputbytes.min(buf.len() as u64);
        for &byte in buf[..nbytes as usize].iter() {
            if self.bufferstatus < 64 {
                match self.add_to_buffer(byte) {
                    Ok(()) => {}
                    Err(DecoderError::AllOutputAlreadyWritten) => return Ok(0),
                    Err(err) => return Err(err.into()),
                }
            } else if self.bufferstatus == 64 {
                match self.add_to_vault(byte) {
                    Ok(()) => {}
                    Err(DecoderError::AllOutputAlreadyWritten) => return Ok(0),
                    Err(err) => return Err(err.into()),
                }
            } else {
                return Err(DecoderError::BufferOverflow.into());
            }
            // self.remaining_outputbytes -= 1;
            // Can not count output bytes here since compressed byte might contain several uncompressed bytes
        }
        Ok(nbytes as usize)
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
        while self.remaining_outputbytes > 0 {
            match self.put() {
                Ok(()) => {}
                Err(DecoderError::AllOutputAlreadyWritten) => return Ok(()),
                Err(err) => return Err(err.into()),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::huffman::encode::Encoder;
    use crate::huffman::Huffman;
    use std::io::Cursor;

    fn encode_str(sentence: &str) -> (Vec<u8>, Vec<u8>, Huffman) {
        let data = sentence.as_bytes().to_vec();
        let h = Huffman::from_slice(data.as_slice());

        let mut enc = Encoder::new(Cursor::new(Vec::new()), &h);
        let _output_bytes = enc.write(&data).expect("");
        enc.flush().expect("");
        let encoded_data: Vec<u8> = enc.inner.get_ref().iter().map(|&x| x).collect();
        (data, encoded_data, h)
    }

    fn get_decoder_for_string(sentence: &str) -> (Vec<u8>, Decoder<Cursor<Vec<u8>>>) {
        let (data, encdata, h) = encode_str(sentence);
        let writer = Cursor::new(Vec::new());
        let d = Decoder::new(writer, &h, data.len() as u64);
        return (encdata, d);
    }

    #[test]
    fn test_buffer_and_vault_access() {
        let sentence = "What a lovely world";
        let (_, mut decoder) = get_decoder_for_string(sentence);
        decoder.write(&[43]).unwrap();
        assert_eq!(decoder.buffer, 43 << 56);
        decoder.buffer = 0;
        assert_eq!(decoder.buffer, 0);
        decoder.vault = 34;
        assert_eq!(decoder.vault, 34);
    }

    // buffer empty, vault empty
    #[test]
    fn test_fill_values_to_buffer() {
        let sentence = "What a lovely world";
        let (edata, mut decoder) = get_decoder_for_string(sentence);
        decoder.write(&edata[..8]).unwrap();
        for i in 0..8 {
            let bufvalue = (decoder.buffer >> 56 - i * 8) & 255;
            assert_eq!(bufvalue as u8, edata[i]);
        }
    }

    // buffer < 8 bits free, vault empty
    #[test]
    fn test_fill_values_to_buffer_and_vault() {
        let sentence = "What a lovely world";
        let (_, mut decoder) = get_decoder_for_string(sentence);
        for i in 0..8 {
            decoder.buffer = 0;
            decoder.bufferstatus = 64 - i;
            decoder.vault = 0;
            decoder.vaultstatus = 0;
            decoder.write(&[255]).unwrap();
            println!("{}", decoder);
            assert_eq!(decoder.buffer, 2u64.pow(i as u32) - 1);
            assert_eq!(decoder.vault >> 64 - 8 + i, 2u64.pow(8 - i as u32) - 1);
        }
    }

    // buffer full, vault empty
    #[test]
    fn test_fill_values_to_vault() {
        let sentence = "What a lovely world";
        let (edata, mut decoder) = get_decoder_for_string(sentence);
        decoder.bufferstatus = 64;
        decoder.write(&edata[..7]).unwrap();
        for i in 0..7 {
            let decvalue = (decoder.vault >> 56 - i * 8) & 255;
            assert_eq!(decvalue as u8, edata[i]);
        }
    }

    // buffer full, vault < 8 bits free => which forces an output
    #[test]
    fn test_fill_values_to_vault_overflow() {
        let sentence = "What a lovely world";
        let (edata, mut decoder) = get_decoder_for_string(sentence);
        decoder.vaultstatus = 61;
        decoder.write(&edata).unwrap();

        let expected = sentence.as_bytes();
        assert_eq!(
            *decoder.inner.get_ref(),
            expected[..expected.len() - decoder.remaining_outputbytes as usize].to_vec()
        );
    }

    // output all encoded bytes, without using conume_buffer()
    #[test]
    fn test_roundtrip_without_consuming_buffer() {
        let sentence = "What a lovely world in this whole world with a really nice graphics";
        let (edata, mut decoder) = get_decoder_for_string(sentence);
        decoder.write(&edata).unwrap();
        while decoder.remaining_outputbytes > 0 {
            decoder.write(&[0]).unwrap();
        }
        assert_eq!(*decoder.inner.get_ref(), sentence.as_bytes().to_vec());
    }

    // roundtrip with write_all
    #[test]
    fn test_roundtrip_with_write_all() {
        let sentence = "What a lovely world in this whole world with a really nice graphics";
        let (edata, mut decoder) = get_decoder_for_string(sentence);
        decoder.write_all(&edata).unwrap();
        decoder.flush().unwrap();
        assert_eq!(*decoder.inner.get_ref(), sentence.as_bytes().to_vec());
    }
}
