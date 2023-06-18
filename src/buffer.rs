use crate::disk::{DiskManager, PageId};
use std::{
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BufferId(usize);
impl BufferId {
    fn value(&self) -> usize {
        self.0
    }
}

pub type Page = [u8; DiskManager::PAGE_SIZE];

pub struct Buffer {
    pub page_id: PageId,
    pub page: Page,
    pub is_dirty: bool,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            page_id: Default::default(),
            page: [0u8; DiskManager::PAGE_SIZE],
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
                self.disk.write_page_data(evict_page_id, &buffer.page)?;
            }
            buffer.page_id = page_id;
            buffer.is_dirty = false;
            self.disk.read_page_data(page_id, &mut buffer.page)?;
            frame.usage_count = 1;
        }
        let buffer = Rc::clone(&frame.buffer);
        self.page_table.remove(&evict_page_id);
        self.page_table.insert(page_id, buffer_id);
        Ok(buffer)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod BufferPoolManagerTests {
    use super::*;

    mod fetch_page {
        use super::*;
        use std::fs::remove_file;

        #[test]
        fn バッファプールに存在しないページを読み込もうとした場合ディスクから読み込みバッファプールに書き込んだ後ページの内容を返すこと(
        ) {
            // Arrange
            let file_path = "BufferPoolManagerTests::fetch_page::0.txt";
            let page_id = PageId(0);
            let data = ['a' as u8; DiskManager::PAGE_SIZE];
            let mut disk = DiskManager::open(file_path).unwrap();
            disk.write_page_data(page_id, &data).unwrap();
            let pool = BufferPool::new(3);

            // Act
            let mut buffer_pool_manager = BufferPoolManager::new(disk, pool);
            let buffer = buffer_pool_manager.fetch_page(page_id).unwrap();

            // Assert
            assert_eq!(buffer.page_id, page_id);
            assert_eq!(buffer.page, data);
            assert_eq!(
                *buffer_pool_manager.page_table.get(&page_id).unwrap(),
                BufferId(0)
            );

            // Cleanup
            remove_file(file_path).unwrap();
        }

        #[test]
        fn ページがバッファプールに存在する場合バッファプールの内容を読み込むこと() {
            // Arrange
            let file_path = "BufferPoolManagerTests::fetch_page::1.txt";
            let page_id = PageId(0);
            let data = ['a' as u8; DiskManager::PAGE_SIZE];
            let buffer_id = BufferId(0);
            let disk = DiskManager::open(file_path).unwrap();
            let mut buffer = Buffer::default();
            buffer.page_id = page_id;
            buffer.page = data;
            let mut frame = Frame::default();
            frame.usage_count = 1;
            frame.buffer = Rc::new(buffer);
            let mut pool = BufferPool::new(1);
            pool.buffers = vec![frame];
            let mut buffer_pool_manager = BufferPoolManager::new(disk, pool);
            buffer_pool_manager.page_table.insert(page_id, buffer_id);

            // Act
            let buffer = buffer_pool_manager.fetch_page(page_id).unwrap();

            // Assert
            assert_eq!(buffer.page_id, page_id);
            assert_eq!(buffer.page, data);
            assert_eq!(buffer.is_dirty, false);

            // Cleanup
            remove_file(file_path).unwrap();
        }
    }
}
