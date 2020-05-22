use log::debug;
use std::collections::BTreeMap;
use succinct::bit_vec::BitVecMut;
use succinct::rank::BitRankSupport;
use succinct::rsdict::RsDict;
use succinct::BitVector;

#[derive(Debug)]
pub struct SymbolTable {
    table: Vec<(u8, u8)>,
    rbv: RsDict,
}

impl SymbolTable {
    pub fn from_btree(btree: &BTreeMap<usize, (u8, u8)>) -> Self {
        let (table, rbv) = prepare_lookup(btree);
        SymbolTable { table, rbv }
    }
    pub fn get_cut_and_symbol(&mut self, val: u64) -> (usize, u8) {
        let pos = self.rbv.rank1(val + 1) as usize - 1;
        let (sym, length) = self.table[pos];
        debug!("Cut {} Symbol {:b}", length, sym);
        (length as usize, sym)
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
