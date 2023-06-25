use super::{
    buffer_pool::BufferPool,
    core::{Buffer, BufferId},
    error::Error,
};
use crate::disk::{DiskManager, PageId};
use std::{collections::HashMap, rc::Rc};

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
        use crate::buffer::core::Frame;
        use std::fs::remove_file;

        #[test]
        fn バッファプールに存在しないページを読み込もうとした場合ディスクから読み込みバッファプールに書き込んだ後ページの内容を返すこと(
        ) {
            // Arrange
            let file_path = "BufferPoolManagerTests::fetch_page::0.txt";
            let page_id = PageId(0);
            let data = ['a' as u8; DiskManager::PAGE_SIZE];
            let mut buffer_pool_manager = {
                let mut disk = DiskManager::open(file_path).unwrap();
                disk.write_page_data(page_id, &data).unwrap();
                let pool = BufferPool::new(3);
                BufferPoolManager::new(disk, pool)
            };

            // Act
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
            let mut buffer_pool_manager = {
                let disk = DiskManager::open(file_path).unwrap();
                let frame = {
                    let buffer = Buffer {
                        page_id,
                        page: data,
                        is_dirty: false,
                    };
                    Frame {
                        usage_count: 1,
                        buffer: Rc::new(buffer),
                    }
                };
                let mut pool = BufferPool::new(1);
                pool.buffers = vec![frame];
                BufferPoolManager::new(disk, pool)
            };
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
