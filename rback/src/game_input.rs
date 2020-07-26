use std::fmt::Debug;
/// Input passed to be used in rollbacks must satisfy this trait
pub trait GameInput: Clone + Debug + PartialEq {
    // TODO: use bytes::Bytes instead?
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> &[u8];
}
// impl<T> GameInput for T where T: Clone + Debug + PartialEq {}
#[cfg(test)]
impl GameInput for &str {
    fn from_bytes(bytes: &[u8]) -> Self {
        use std::str;
        str::from_utf8(bytes).expect("should be valid")
    }
    fn to_bytes(&self) -> &[u8] {
        todo!()
    }
}
