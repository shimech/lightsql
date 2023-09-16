use bincode::Options;
use serde::{Deserialize, Serialize};

mod branch;
mod leaf;
mod meta;
mod node;

#[derive(Serialize, Deserialize)]
pub struct Pair<'a> {
    pub key: &'a [u8],
    pub value: &'a [u8],
}

impl<'a> Pair<'a> {
    fn to_bytes(&self) -> Vec<u8> {
        bincode::options().serialize(self).unwrap()
    }

    fn from_bytes(bytes: &'a [u8]) -> Self {
        bincode::options().deserialize(bytes).unwrap()
    }
}
