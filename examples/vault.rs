
use rscompress_huffman::vault;

fn main() {
    env_logger::init();
    let data = "What a lovely text".as_bytes().to_vec();
    println!("{:?}", data);
    let decoder = vault::Decoder::new(data.into_iter(), 5);
    println!("{:?}", decoder);
    for value in decoder {
        println!(" {}", value)
    }
}
