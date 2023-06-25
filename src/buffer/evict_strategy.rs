use super::core::{BufferId, Frame};

pub trait EvictStrategy {
    fn evict(pool_size: usize, victim_id: BufferId, buffers: Vec<Frame>) -> BufferId;
}
