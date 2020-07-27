use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

// TODO: keep it under 1500ish bytes https://amethyst.github.io/laminar/docs/fragmentation.html
// can i enforce that?

/// Input passed to be used in rollbacks must satisfy this trait
pub trait GameInput: Clone + Debug + PartialEq + DeserializeOwned + Serialize {
    // // TODO: use bytes::Bytes instead?
    // fn from_bytes(bytes: Bytes) -> Self;
    // fn to_bytes(&self) -> Bytes;
}

impl<T> GameInput for T where T: Clone + Debug + PartialEq + DeserializeOwned + Serialize {}
// #[cfg(test)]
// impl GameInput for &str {
//     fn from_bytes(bytes: Bytes) -> Self {
//         todo!()
//     }
//     fn to_bytes(&self) -> Bytes {
//         todo!()
//     }
// }
