//! This module implements some statistical helper functions.
//! Mostly due to the histogram needed for defining the Huffman tree.
use crate::BUF;
use std::io::Read;

fn update_histogram(take: usize, buffer: &[u8], histogram: &mut [usize]) {
    for byte in buffer.iter().take(take) {
        histogram[*byte as usize] += 1;
    }
}

/// Build a histogram for each byte.
pub fn generate_histogram(reader: &mut impl Read) -> [usize; 256] {
    let mut buffer: Vec<u8> = Vec::with_capacity(BUF);
    unsafe { buffer.set_len(BUF) };
    let mut histogram: [usize; 256] = [0; 256];

    // First loop over the data to gather statistics about the source file.
    loop {
        let read_size = reader.read(&mut buffer);
        match read_size {
            Ok(0) => break, // fully read file
            Ok(n) => update_histogram(n, &buffer, &mut histogram),
            Err(err) => panic!("Problem with reading source file: {:?}", err),
        };
    }
    histogram
}

#[cfg(test)]
mod tests {
    use crate::stats::generate_histogram;
    use std::io::Cursor;

    #[test]
    fn testing_histogram() {
        let mut data = Cursor::new(vec![3, 32, 34, 34, 34, 2, 0]);
        let hist = generate_histogram(&mut data);

        assert_eq!(hist[233], 0);
        assert_eq!(hist[3], 1);
        assert_eq!(hist[32], 1);
        assert_eq!(hist[2], 1);
        assert_eq!(hist[0], 1);
        assert_eq!(hist[34], 3);
    }
}
