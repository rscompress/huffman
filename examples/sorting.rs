// use std::collections::BTreeMap;
// use std::time::Instant;
// use rscompress_huffman::stats::generate_random_byte_vector;
// use rscompress_huffman::encode::search_key_or_next_small_key;

// fn main() {
//     // grab the rng
//     let mut data = generate_random_byte_vector(0,100_000,120);
//     let mut maps : Vec<Mapping> = Vec::new();
//     let mut btm : BTreeMap<usize,(usize,usize)> = BTreeMap::new();
//     for d in &data {
//         maps.push(Mapping {codeword:*d,sym:*d,length:*d} );
//         btm.insert(*d, (*d,*d));
//     }
//     maps.sort();
//     maps.dedup();
//     data.sort();

//     let searches = generate_random_byte_vector(0,1_0000_000,1_000_000_000);

//     let now = Instant::now();
//     for k in &searches {
//         let s = Mapping {codeword: *k as usize, sym:3, length:2};
//         log_search_key_or_next_smaller(s, &maps.as_slice());
//     }
//     println!("M {}", now.elapsed().as_secs_f32());

//     let now = Instant::now();
//     for k in &searches {
//         let s = Mapping {codeword: *k as usize, sym:3, length:2};
//         log_search_key_or_next_smaller_new(s, &maps.as_slice());
//     }
//     println!("Mn {}", now.elapsed().as_secs_f32());

//     let now = Instant::now();
//     for k in &searches {
//         search_key_or_next_small_key(&btm, *k as usize);
//     }
//     println!("BT {}", now.elapsed().as_secs_f32());

//     let val = 23;
//     let s = Mapping {codeword: val, sym:3, length:2};
//     let g = log_search_key_or_next_smaller(s, &maps.as_slice());
//     // println!("Search {:?} in {:?}: {:?}", val, data, *g);

//     let g = log_search_key_or_next_smaller(val, &data);
//     // println!("Search {:?} in {:?}: {:?}", val, data, *g);
// }

// fn log_search_key_or_next_smaller<C: PartialEq + PartialOrd>(key: C, data: &[C]) -> &C {
//     let v = &data[data.len()/2];
//     if v == &key || data.len() == 1 {
//         return &v
//     } else if v > &key {
//         return log_search_key_or_next_smaller(key, &data[..data.len()/2])
//     } else {
//         return log_search_key_or_next_smaller(key, &data[data.len()/2..])
//     }
// }

// fn log_search_key_or_next_smaller_new<C: PartialEq + PartialOrd + Copy>(key: C, data: &[C]) -> C {
//     let v = data[data.len()/2];
//     if v == key || data.len() == 1 {
//         return v
//     } else if v > key {
//         return log_search_key_or_next_smaller_new(key, &data[..data.len()/2])
//     } else {
//         return log_search_key_or_next_smaller_new(key, &data[data.len()/2..])
//     }
// }

// use std::cmp::Ordering;

// #[derive(Eq, Debug, Clone, Copy)]
// struct Mapping {
//     codeword: usize,
//     sym: usize,
//     length: usize
// }

// impl PartialOrd for Mapping {
//     fn partial_cmp(&self, other: &Mapping) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl PartialEq for Mapping {
//     fn eq(&self, other: &Mapping) -> bool {
//         self.codeword == other.codeword
//     }
// }

// impl Ord for Mapping {
//     fn cmp(&self, other: &Mapping) -> Ordering {
//         self.codeword.cmp(&other.codeword)
//     }
// }

fn main() {}
