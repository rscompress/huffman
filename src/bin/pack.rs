
struct Pack {
    buffer : u32,
    remainder: usize,
}
impl Pack {
    fn new() -> Self {
        Pack { buffer: 0, remainder: 8 }
    }
}
impl Pack {
    fn pack(&mut self, code: usize, len: usize) -> Option<Vec<u8>> {
        assert!(len < 8);
        if self.remainder > len {
            self.buffer += (code << (self.remainder - len)) as u32;
            self.remainder -= len;
        } else if self.remainder == len {
            self.buffer += code as u32;
            println!("{0}({0:b})", self.buffer);
            let result = self.buffer;
            self.remainder = 8;
            self.buffer <<= 8;
            return Some(vec![result as u8]);
        } else {
            let transfer = code >> (len - self.remainder);
            self.buffer += transfer as u32;
            println!("{0}({0:b})", self.buffer);
            let result = self.buffer;
            self.buffer = code as u32 & ((1 << self.remainder) - 1) as u32;
            self.remainder = 8 - self.remainder;
            return Some(vec![result as u8]);
        }
        None
    }
}


fn main() {
    let mut p = Pack::new();
    p.pack(8,4);
    p.pack(4,3);
    p.pack(3,2);
    println!("Buffer: {0}({0:b}) / Remainder: {1}", p.buffer, p.remainder);

    let data: Vec<(usize,usize)> = vec![(8,4),(4,3),(3,2),(16,5),(3,2)];
    let mut p = Pack::new();
    let result: Vec<u8> = data.iter().filter_map(|(a,b)| p.pack(*a,*b)).flatten().collect();

    for k in result {
        println!("k = {:?}", k)
    }
}
