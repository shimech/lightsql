use super::{
    core::{BufferId, BufferList, Frame},
    evict_strategy::{ClockSweepStrategy, EvictStrategy},
};
use std::ops::{Index, IndexMut};

pub struct BufferPool {
    pub buffers: BufferList,
    pub next_victim_id: BufferId,
    evict_strategy: Box<dyn EvictStrategy>,
}

impl BufferPool {
    pub fn new(pool_size: usize) -> Self {
        let mut buffers = vec![];
        buffers.resize_with(pool_size, Frame::default);
        Self {
            buffers: BufferList(buffers),
            next_victim_id: BufferId::default(),
            evict_strategy: Box::new(ClockSweepStrategy),
        }
    }

    fn size(&self) -> usize {
        self.buffers.len()
    }

    pub fn evict(&mut self) -> Option<BufferId> {
        self.evict_strategy
            .evict(self.size(), self.next_victim_id, &mut self.buffers)
    }
}

impl Index<BufferId> for BufferPool {
    type Output = Frame;

    fn index(&self, buffer_id: BufferId) -> &Self::Output {
        &self.buffers[buffer_id]
    }
}

impl IndexMut<BufferId> for BufferPool {
    fn index_mut(&mut self, buffer_id: BufferId) -> &mut Self::Output {
        &mut self.buffers[buffer_id]
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod BufferPoolTests {
    use super::*;

    mod new {
        use super::*;

        #[test]
        fn 指定したサイズのバッファプールが生成されること() {
            // Arrange
            let pool_size: usize = 1024;

            // Act
            let pool = BufferPool::new(pool_size);

            // Assert
            assert_eq!(pool.buffers.len(), pool_size)
        }
    }
}
