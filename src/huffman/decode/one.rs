use super::symboltable;
use crate::model::Model;
use log::debug;
use std::collections::BTreeMap;
use succinct::rank::BitRankSupport;

pub fn search_key_or_next_small_key(tree: &BTreeMap<usize, (u8, u8)>, key: usize) -> (u8, u8) {
    let mut iter = tree.range(..key + 1);

    if let Some((_, v)) = iter.next_back() {
        *v
    } else {
        panic!("Panic!!!!")
    }
}

pub fn read(data: &[u8], model: &impl Model, goalsbyte: usize) -> Vec<u8> {
    let mut buffer: u64 = 0;
    let mut bits_left_in_buffer = 64u8;
    let bt = model.to_btreemap();
    debug!("{:?}", &bt);
    let (table, rbv) = symboltable::prepare_lookup(&model.to_btreemap());
    let s = model.sentinel();
    let shift = 64 - s;
    let mut result: Vec<u8> = Vec::with_capacity(data.len());
    let mut writeout = 0;
    for val in data.iter() {
        if bits_left_in_buffer >= 8 {
            // fill buffer
            let v = (*val as u64) << (bits_left_in_buffer - 8);
            buffer += v;
            debug!("     New Buffer: {:b}", buffer);
            bits_left_in_buffer -= 8;
            continue;
        }
        // buffer filled
        while (64 - bits_left_in_buffer) as usize >= s {
            let searchvalue = buffer >> shift;
            let pos = rbv.rank1(searchvalue + 1) as usize - 1;
            let (sym, length) = table[pos];
            result.push(sym);
            debug!(
                "{}: Buffer: {:64b} Select: {:b} Decoded to: {} Shift buffer: {}",
                writeout, buffer, searchvalue, result[writeout], length
            );
            writeout += 1;
            // let (sym,length) = search_key_or_next_small_key(&bt, searchvalue as usize);
            // result.push(sym);
            buffer <<= length;
            bits_left_in_buffer += length;
        }
        debug_assert!(
            bits_left_in_buffer >= 8,
            "Not enough bits left in buffer for val"
        );
        // buffer += (*val as u64) << bits_left_in_buffer - 8;
        debug!("     New Buffer: {:64b}", buffer);
        let v = (*val as u64) << (bits_left_in_buffer - 8);
        buffer += v;
        bits_left_in_buffer -= 8;
    }
    debug!("GB {}", goalsbyte - writeout);
    // consume bits in buffer
    while goalsbyte > writeout {
        let searchvalue = buffer >> shift;
        let pos = rbv.rank1(searchvalue + 1) as usize - 1;
        let (sym, length) = table[pos];
        result.push(sym);
        writeout += 1;
        // let (sym,length) = search_key_or_next_small_key(&bt, searchvalue as usize);
        // result.push(sym);
        buffer <<= length;
        bits_left_in_buffer += length;
    }
    result
}
