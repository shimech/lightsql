use super::core::{BufferId, BufferList};
use std::rc::Rc;

pub trait EvictStrategy {
    fn next_victim_id(&self, victim_id: BufferId, pool_size: usize) -> BufferId;
    fn evict(
        &self,
        pool_size: usize,
        victim_id: BufferId,
        buffers: &mut BufferList,
    ) -> Option<BufferId>;
}

pub struct ClockSweepStrategy;

impl EvictStrategy for ClockSweepStrategy {
    fn next_victim_id(&self, victim_id: BufferId, pool_size: usize) -> BufferId {
        BufferId((victim_id.value() + 1) % pool_size)
    }

    fn evict(
        &self,
        pool_size: usize,
        victim_id: BufferId,
        buffers: &mut BufferList,
    ) -> Option<BufferId> {
        let mut current_victim_id = victim_id;
        let mut consecutive_pinned = 0;

        let victim_id = loop {
            let frame = &mut buffers[current_victim_id];
            if frame.usage_count == 0 {
                break victim_id;
            }
            if Rc::get_mut(&mut frame.buffer).is_some() {
                frame.usage_count -= 1;
                consecutive_pinned = 0;
            } else {
                consecutive_pinned += 1;
                if consecutive_pinned >= pool_size {
                    return None;
                }
            }
            current_victim_id = self.next_victim_id(current_victim_id, pool_size)
        };

        Some(victim_id)
    }
}
