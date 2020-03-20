
#[derive(Debug)]
struct Pack {
    buffer : u32,
    remainder: usize,
}
impl Pack {
    fn new() -> Self {
        Pack { buffer: 0, remainder: 32 }
    }
}
impl Pack {
    fn pack(&mut self, code: usize, len: usize) -> Option<Vec<u8>> {
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

    fn flush(&mut self) -> Vec<u8> {
        let mut result = self.writeout();
        result.push(self.buffer as u8);
        result
    }

}

use rscompress_huffman::encode::calculate_length;

fn main() {

    let words: Vec<usize> = vec![177, 112, 84, 143, 148, 195, 165, 206, 34, 10];
    let lens: Vec<_> = words.iter().map(|&x| calculate_length(x)).collect();

    let mut p = Pack::new();
    let mut result: Vec<u8> = words.iter().zip(lens.iter()).filter_map(|(a,b)| p.pack(*a,*b)).flatten().collect();
    result.extend(p.flush());

    for k in result {
        println!("{0} = {0:08b}", k as u8)
    }
}
