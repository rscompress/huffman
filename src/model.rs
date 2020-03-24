pub trait Model {
    fn encode(&self, sym: u8) -> (usize, usize);
    fn sentinel(&self) -> usize;
}
