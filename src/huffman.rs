//! This module packages functions needed for generating the codebase of
//! Huffman Encoding.
use std::collections::HashMap;

/// Calculate the length of the codewords for each byte in place.
/// This will transform the histogram into a codeword length array for each
/// byte.
fn calculate_codeword_length_inplace(histogram: &mut [usize]) {
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
    while next <= histogram.len() - 1 {
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

/// Calculates the same codeword lengths as `calculate_codeword_length_inplace`.
/// But using a new output array. It leaves the histogram untouched.
fn calculate_codeword_length(histogram: &[usize]) -> Vec<usize> {
    let mut codeword_length: Vec<usize> = vec![0usize; histogram.len()];
    codeword_length.clone_from_slice(histogram);
    calculate_codeword_length_inplace(&mut codeword_length);
    codeword_length
}

fn enumerate(array: &[usize]) -> HashMap<u8, usize> {
    let mut hist: HashMap<u8, usize> = HashMap::with_capacity(256);
    for (i, val) in array.iter().enumerate() {
        hist.insert(i as u8, *val);
    }
    hist
}

fn sort_by_value(store: HashMap<u8, usize>) -> Vec<(u8, usize)> {
    let mut sorted_tuple: Vec<_> = store.iter().filter(|a| *a.1 > 0 as usize).collect();
    sorted_tuple.sort_by(|a, b| b.1.cmp(a.1));
    let sorted_tuple = sorted_tuple.iter().map(|(&a, &b)| (a, b)).collect();
    sorted_tuple
}

fn extract_values(store: &Vec<(u8, usize)>) -> Vec<usize> {
    let values: Vec<_> = store.iter().map(|(_, b)| *b).collect();
    values
}

fn calculate_codewords_based_on_length(lengths: &[usize]) -> (Vec<usize>, Vec<usize>) {
    let max_wordlen = lengths[lengths.len() - 1];
    let mut li_small_codes: Vec<usize> = vec![0usize; lengths.len()];
    let mut li_big_codes: Vec<usize> = vec![0usize; lengths.len()];
    unsafe {
        li_small_codes.set_len(lengths.len());
        li_big_codes.set_len(lengths.len());
    }
    let sentinel = 1 << max_wordlen;

    for i in 1..lengths.len() {
        li_big_codes[i] = (1 << (max_wordlen - lengths[i - 1])) + li_big_codes[i - 1];
        li_small_codes[i] = li_big_codes[i] >> (max_wordlen - lengths[i])
    }

    (li_small_codes, li_big_codes)
}

/// Generate codewords from a histogram.
///
/// # Steps
/// 1. Enumerate the histogram
/// 2. Sort the enumerated histogram by count
/// 3. Extract the counts of the sorted histogram
fn generate_codewords(histogram: &[usize]) -> Codewords {
    let hist = enumerate(histogram);
    let sorted_tuple = sort_by_value(hist); // FIXME: It might be possible to split into two Vectors
    let mut weights = extract_values(&sorted_tuple);

    calculate_codeword_length_inplace(&mut weights[..]);

    let sentinel = 1 << weights[weights.len() - 1];
    let (li_small_codes, li_big_codes) = calculate_codewords_based_on_length(&weights);

    Codewords::new(li_small_codes, li_big_codes, sentinel)
}

struct Codewords {
    codewords: Vec<usize>,
    alt_codewords: Vec<usize>,
    sentinel: usize,
}

impl Codewords {
    pub fn new(codewords: Vec<usize>, alt_codewords: Vec<usize>, sentinel: usize) -> Self {
        if codewords.len() != alt_codewords.len() {
            panic!("Codewords and alternative codewords must be of same size")
        }
        Codewords {
            codewords: codewords,
            alt_codewords: alt_codewords,
            sentinel: sentinel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_codeword_table() {
        let histogram = vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1];
        let codes = generate_codewords(&histogram);
        assert_eq!(codes.codewords, [0, 2, 12, 26, 27, 28, 29, 30, 62, 63]);
        assert_eq!(codes.alt_codewords, [0, 32, 48, 52, 54, 56, 58, 60, 62, 63]);
        assert_eq!(codes.sentinel, 64);
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
    #[test]
    fn test_codeword_lengths() {
        let mut elements: HashMap<Vec<usize>, Vec<usize>> = HashMap::new();
        elements.insert(vec![1, 2, 4, 4, 4, 4], vec![10, 6, 2, 1, 1, 1]);
        elements.insert(
            vec![1, 2, 4, 5, 5, 5, 5, 5, 6, 6],
            vec![20, 17, 6, 3, 2, 2, 2, 1, 1, 1],
        );
        elements.insert(vec![2, 2, 2, 3, 4, 4], vec![99, 99, 99, 1, 1, 1]);
        elements.insert(vec![2, 2, 3, 3, 3, 3], vec![8, 7, 6, 5, 4, 3]);

        for (expected, hist) in elements.iter() {
            let result = calculate_codeword_length(hist);
            assert_eq!(&result, expected);
        }
    }
}
