//! This module packages functions needed for generating the codebase of
//! Huffman Encoding.

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
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
