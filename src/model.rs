
use std::collections::BTreeMap;

pub trait Model {
    fn encode(&self, sym: u8) -> (usize, usize);
    fn sentinel(&self) -> usize;
    fn to_btreemap(&self) -> BTreeMap<usize, (usize, usize)>;
}
