use super::BufferPool;
use crate::buffer::{BufferId, Frame};
use std::ops::{Index, IndexMut};

pub struct ClockSweepBufferPool {
    pub buffers: Vec<Frame>,
    pub next_victim_id: BufferId,
}

impl ClockSweepBufferPool {
    fn size(&self) -> usize {
        self.buffers.len()
    }

    fn update_next_victim_id(&mut self) -> () {
        self.next_victim_id = BufferId::new((self.next_victim_id.value() + 1) % self.size())
    }
}

impl BufferPool for ClockSweepBufferPool {
    fn evict(&mut self) -> Option<BufferId> {
        let pool_size = self.size();
        let mut consecutive_pinned = 0;

        let victim_id = loop {
            let next_victim_id = self.next_victim_id;
            let frame = &mut self[next_victim_id];
            if frame.usage_count == 0 {
                break self.next_victim_id;
            }
            if frame.has_reference() {
                consecutive_pinned += 1;
                if consecutive_pinned >= pool_size {
                    return None;
                }
            } else {
                frame.usage_count -= 1;
                consecutive_pinned = 0;
            }
            self.update_next_victim_id();
        };
        Some(victim_id)
    }
}

impl From<usize> for ClockSweepBufferPool {
    fn from(pool_size: usize) -> Self {
        let mut buffers = vec![];
        buffers.resize_with(pool_size, Frame::default);
        Self {
            buffers,
            next_victim_id: BufferId::default(),
        }
    }
}

impl Index<BufferId> for ClockSweepBufferPool {
    type Output = Frame;

    fn index(&self, buffer_id: BufferId) -> &Self::Output {
        &self.buffers[buffer_id.value()]
    }
}

impl IndexMut<BufferId> for ClockSweepBufferPool {
    fn index_mut(&mut self, buffer_id: BufferId) -> &mut Self::Output {
        &mut self.buffers[buffer_id.value()]
    }
}

#[cfg(test)]
mod clock_sweep_buffer_pool_test {
    use super::*;

    mod from {
        use super::*;

        #[test]
        fn 指定したサイズのバッファプールが生成されること() {
            // Arrange
            let pool_size: usize = 1024;

            // Act
            let pool = ClockSweepBufferPool::from(pool_size);

            // Assert
            assert_eq!(pool.buffers.len(), pool_size)
        }
    }
}
