//! This module packages functions needed for generating the codebase of
//! Huffman Encoding.
use log::debug;
use crate::model::Model;

pub struct Huffman {
    codewords: [usize;256],
    length: [usize;256]
}

impl Model for Huffman {
    fn encode(&self, sym: u8) -> (usize, usize) {
        (self.codewords[sym as usize], self.length[sym as usize])
    }
    fn sentinel(&self) -> usize {
        *self.length.iter().max().expect("Can not find maximum value.")
    }
}

use std::io::Read;
use crate::stats::generate_histogram;


impl Huffman {
    pub fn new(codewords: [usize; 256], length: [usize; 256]) -> Self {
        Huffman {codewords, length}
    }
    pub fn from_histogram(histogram: &[usize; 256]) -> Self {
        let (codewords, length) = generate_extended_codewords(histogram);
        Huffman::new(codewords, length)
    }
    pub fn from_reader(reader: &mut impl Read) -> Self {
        let histogram = generate_histogram(reader);
        Huffman::from_histogram(&histogram)
    }
}

/// Calculate the length of the codewords for each byte in place.
/// This will transform the histogram into a codeword length array for each
/// byte.
pub fn calculate_codeword_length_inplace(histogram: &mut [usize]) {
    let mut leaf = (histogram.len() - 1) as i32;
    let mut root = histogram.len() - 1;

    // Phase 1
    for next in (1..histogram.len()).rev() {
        if leaf < 0 || (root > next) && (histogram[root] < histogram[leaf as usize]) {
            histogram[next] = histogram[root];
            histogram[root] = next;
            root -= 1;
        } else {
            histogram[next] = histogram[leaf as usize];
            leaf -= 1;
        }
        if leaf < 0 || (root > next) && (histogram[root] < histogram[leaf as usize]) {
            histogram[next] += histogram[root];
            histogram[root] = next;
            root -= 1;
        } else {
            histogram[next] += histogram[leaf as usize];
            leaf -= 1;
        }
    }

    // Phase 2
    histogram[1] = 0;
    let mut next = 2;
    while next < histogram.len() {
        histogram[next] = histogram[histogram[next]] + 1;
        next += 1;
    }

    let mut avail = 1;
    let mut used = 0;
    let mut depth = 0;
    let mut root = 1;
    let mut next = 0;

    while avail > 0 {
        while root < histogram.len() && histogram[root] == depth {
            used += 1;
            root += 1;
        }
        while avail > used {
            histogram[next] = depth;
            next += 1;
            avail -= 1;
        }
        avail = 2 * used;
        depth += 1;
        used = 0
    }
}

pub fn sort_by_value(store: &[usize]) -> Vec<(usize, usize)> {
    assert!(store.len() <= 256);
    let mut sorted_tuple: Vec<(usize, usize)> = vec![];
    sorted_tuple.reserve_exact(256);
    sorted_tuple.extend(
        store
            .iter()
            .enumerate()
            .filter(|(_, b)| **b > 0 as usize)
            .map(|(a, b)| (a, *b)),
    );
    sorted_tuple.sort_unstable_by_key(|a| std::cmp::Reverse(a.1));
    // sorted_tuple.reverse();
    sorted_tuple
}

// pub fn sort_by_value(store: &[usize]) -> Vec<(usize, usize)> {
//     let mut indices : Vec<_> = (0..store.len()).into_iter().filter(|&c| store[c as usize] > 0).collect();
//     indices.sort_unstable_by_key(|&i| store[i as usize]);
//     indices.reverse();
//     indices.into_iter().map(|i| (i, store[i as usize])).collect()
// }

pub fn extract_values(store: &[(usize, usize)]) -> Vec<usize> {
    let values: Vec<_> = store.iter().map(|(_, b)| *b).collect();
    values
}

pub fn calculate_codewords_based_on_length(lengths: &[usize]) -> (Vec<usize>, Vec<usize>) {
    let max_wordlen = lengths[lengths.len() - 1];
    let mut li_small_codes: Vec<usize> = vec![0usize; lengths.len()];
    let mut li_big_codes: Vec<usize> = vec![0usize; lengths.len()];
    unsafe {
        li_small_codes.set_len(lengths.len());
        li_big_codes.set_len(lengths.len());
    }

    for i in 1..lengths.len() {
        li_big_codes[i] = (1 << (max_wordlen - lengths[i - 1])) + li_big_codes[i - 1];
        li_small_codes[i] = li_big_codes[i] >> (max_wordlen - lengths[i])
    }

    (li_small_codes, li_big_codes)
}

use crate::encode::calculate_length;

/// Generate extended codewords from a histogram.
///
/// # Steps
/// 1. Enumerate the histogram
/// 2. Sort the enumerated histogram by count
/// 3. Extract the counts of the sorted histogram
/// 4. Calculate codeword lengths inplace
/// 5. Generate canonical codewords based on length
pub fn generate_extended_codewords(histogram: &[usize]) -> ([usize; 256], [usize; 256]) {
    // let hist = enumerate(histogram);
    let sorted_tuple = sort_by_value(&histogram);
    let mut weights = extract_values(&sorted_tuple);
    calculate_codeword_length_inplace(&mut weights);
    let (codes, _) = calculate_codewords_based_on_length(&weights);

    let mut extended_codes = [0usize; 256];
    let mut length = [1usize; 256];
    for (code, (key, _)) in codes.into_iter().zip(sorted_tuple.into_iter()) {
        debug!(
            "Huffman code: {0:>8b} [{0:>3}] -> {1:b} [{1:>3}]",
            key, code
        );
        extended_codes[key as usize] = code;
        length[key as usize] = calculate_length(code);
    }
    (extended_codes, length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_extended_codewords_sorted_input() {
        let histogram = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
        let (ext_codes, _) = generate_extended_codewords(&histogram);
        assert_eq!(ext_codes[0], 0);
        assert_eq!(ext_codes[1], 2);
        assert_eq!(ext_codes[2], 12);
        assert_eq!(ext_codes[3], 26);

        // The following cases are necessary since the algorithm is not
        // deterministic if the count is the same between values
        assert!(ext_codes[4] <= 30 && ext_codes[4] >= 27);
        assert!(ext_codes[5] <= 30 && ext_codes[5] >= 27);
        assert!(ext_codes[6] <= 30 && ext_codes[6] >= 27);
        assert!(ext_codes[7] >= 30 && ext_codes[7] <= 63);
        assert!(ext_codes[8] >= 30 && ext_codes[8] <= 63);
        assert!(ext_codes[9] >= 30 && ext_codes[9] <= 63);
    }

    #[test]
    fn test_codeword_lengths_inplace() {
        let mut elements: HashMap<Vec<usize>, Vec<usize>> = HashMap::new();
        elements.insert(vec![1, 2, 4, 4, 4, 4], vec![10, 6, 2, 1, 1, 1]);
        elements.insert(
            vec![1, 2, 4, 5, 5, 5, 5, 5, 6, 6],
            vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1],
        );
        elements.insert(vec![2, 2, 2, 3, 4, 4], vec![99, 99, 99, 1, 1, 1]);
        elements.insert(vec![2, 2, 3, 3, 3, 3], vec![8, 7, 6, 5, 4, 3]);

        for (expected, hist) in elements.iter_mut() {
            calculate_codeword_length_inplace(hist);
            assert_eq!(hist, expected);
        }
    }
}
