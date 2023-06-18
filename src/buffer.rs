use crate::disk::{DiskManager, PageId};
use std::{
    cell::RefCell,
    collections::HashMap,
    io,
    ops::{Index, IndexMut},
    rc::Rc,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("no free buffer is available in this buffer pool.")]
    NoFreeBuffer,
}

#[derive(Default, Clone, Copy)]
pub struct BufferId(usize);
impl BufferId {
    fn value(&self) -> usize {
        self.0
    }
}

pub type Page = [u8; DiskManager::PAGE_SIZE];

pub struct Buffer {
    pub page_id: PageId,
    pub page: RefCell<Page>,
    pub is_dirty: bool,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            page_id: Default::default(),
            page: RefCell::new([0u8; DiskManager::PAGE_SIZE]),
            is_dirty: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct Frame {
    usage_count: u64,
    buffer: Rc<Buffer>,
}

pub struct BufferPool {
    buffers: Vec<Frame>,
    next_victim_id: BufferId,
}

impl BufferPool {
    pub fn new(pool_size: usize) -> Self {
        let mut buffers = vec![];
        buffers.resize_with(pool_size, Frame::default);
        Self {
            buffers,
            next_victim_id: BufferId::default(),
        }
    }

    fn size(&self) -> usize {
        self.buffers.len()
    }

    fn evict(&mut self) -> Option<BufferId> {
        let pool_size = self.size();
        let mut consecutive_pinned = 0;

        let victim_id = loop {
            let next_victim_id = self.next_victim_id;
            let frame = &mut self[next_victim_id];
            if frame.usage_count == 0 {
                break self.next_victim_id;
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
            self.update_next_victim_id();
        };
        Some(victim_id)
    }

    fn update_next_victim_id(&mut self) -> () {
        self.next_victim_id = BufferId((self.next_victim_id.value() + 1) % self.size())
    }
}

impl Index<BufferId> for BufferPool {
    type Output = Frame;

    fn index(&self, buffer_id: BufferId) -> &Self::Output {
        &self.buffers[buffer_id.value()]
    }
}

impl IndexMut<BufferId> for BufferPool {
    fn index_mut(&mut self, buffer_id: BufferId) -> &mut Self::Output {
        &mut self.buffers[buffer_id.value()]
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

pub struct BufferPoolManager {
    disk: DiskManager,
    pool: BufferPool,
    page_table: HashMap<PageId, BufferId>,
}

impl BufferPoolManager {
    pub fn new(disk: DiskManager, pool: BufferPool) -> Self {
        Self {
            disk,
            pool,
            page_table: HashMap::new(),
        }
    }

    pub fn fetch_page(&mut self, page_id: PageId) -> Result<Rc<Buffer>, Error> {
        if let Some(&buffer_id) = self.page_table.get(&page_id) {
            let frame = &mut self.pool[buffer_id];
            frame.usage_count += 1;
            return Ok(frame.buffer.clone());
        }
        let buffer_id = self.pool.evict().ok_or(Error::NoFreeBuffer)?;
        let frame = &mut self.pool[buffer_id];
        let evict_page_id = frame.buffer.page_id;
        {
            let buffer = Rc::get_mut(&mut frame.buffer).unwrap();
            if buffer.is_dirty {
                self.disk
                    .write_page_data(evict_page_id, buffer.page.get_mut())?;
            }
            buffer.page_id = page_id;
            buffer.is_dirty = false;
            self.disk.read_page_data(page_id, buffer.page.get_mut())?;
            frame.usage_count = 1;
        }
        let page = Rc::clone(&frame.buffer);
        self.page_table.remove(&evict_page_id);
        self.page_table.insert(page_id, buffer_id);
        Ok(page)
    }
}
