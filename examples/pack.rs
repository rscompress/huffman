
#[derive(Debug)]
struct Pack {
    buffer : u32,
    remainder: usize,
    codewords: [usize;256],
    lengths: [usize;256],
}

impl Pack {
    fn new(codewords: [usize;256], lengths: [usize;256]) -> Self {
        Pack { buffer: 0, remainder: 32 , codewords, lengths}
    }
}
impl Pack {
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
        let mut result : Vec<u8> = Vec::new();
        while 32 - self.remainder > 8 {
            let s = 32 - self.remainder - 8;
            result.push((self.buffer >> s) as u8);
            self.buffer <<= (self.remainder + 8);
            self.buffer >>= (self.remainder + 8);
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

use rscompress_huffman::encode::calculate_length;

fn main() {
    let words: Vec<u8> = vec![177, 112, 84, 143, 148, 195, 165, 206, 34, 10];
    let mut codewords = [0usize; 256];
    let mut length = [0usize; 256];
    for word in words.iter() {
        codewords[*word as usize] = *word as usize;
        length[*word as usize] = calculate_length(*word as usize);
    }

    let mut p = Pack::new(codewords,length);
    let mut result: Vec<u8> = words.iter().filter_map(|&a| p.pack(a)).flatten().collect();
    result.extend(p.last());

    for k in result {
        println!("{0} = {0:08b}", k as u8)
    }
}
