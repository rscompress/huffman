
use succinct::rank::BitRankSupport;
use succinct::rsdict::RsDict;
use succinct::BitVector;
use succinct::bit_vec::BitVecMut;
use std::collections:: BTreeMap;
use log::debug;
use lru::LruCache;

const CACHESIZE : usize = 25;

#[derive(Debug)]
pub struct SymbolTable {
    table: Vec<(u8, u8)>,
    rbv: RsDict,
    cache: LruCache<u64, (usize, u8)>
}

impl SymbolTable {
    pub fn from_btree(btree: &BTreeMap<usize, (u8, u8)>) -> Self {
        let (table, rbv) = prepare_lookup(btree);
        let cache = LruCache::new(CACHESIZE);
        SymbolTable { table, rbv, cache}
    }
    pub fn get_cut_and_symbol(&mut self, val: u64) -> (usize, u8) {
        if let Some(result) = self.cache.get(&val) {
            return *result
        } else {
            let pos = self.rbv.rank1(val + 1) as usize - 1;
            let (sym, length) = self.table[pos];
            let result = (length as usize, sym);
            debug!("Cut {} Symbol {:b}", length, sym);
            self.cache.put(val, result);
            return result
        }
    }
}

pub fn prepare_lookup(bt: &BTreeMap<usize, (u8, u8)>) -> (Vec<(u8, u8)>, RsDict) {
    debug!("Btree from encoder: {:?}", bt);
    let table: Vec<(u8, u8)> = bt.values().cloned().collect();
    let keys: Vec<usize> = bt.keys().cloned().collect();
    let m: usize = keys[keys.len() - 1];
    let mut bv: BitVector<u64> = BitVector::with_fill(m as u64 + 1, false);
    for k in keys {
        bv.set_bit(k as u64, true);
    }
    let mut jbv = RsDict::new();
    for bit in bv.iter() {
        jbv.push(bit);
    }

    (table, jbv)
}
