use super::BufferId;
use std::ops::{Index, IndexMut};

mod clock_sweep;
pub use clock_sweep::*;

pub trait BufferPool: Index<BufferId> + IndexMut<BufferId> {
    fn evict(&mut self) -> Option<BufferId>;
}
