use super::{Buffer, BufferId, BufferPool, Error, Frame};
use crate::disk::{DiskManager, PageId};
use std::{collections::HashMap, rc::Rc};

pub struct BufferPoolManager {
    disk: DiskManager,
    pool: Box<dyn BufferPool<Output = Frame>>,
    page_table: HashMap<PageId, BufferId>,
}

impl BufferPoolManager {
    pub fn new<T: 'static + BufferPool<Output = Frame>>(disk: DiskManager, pool: T) -> Self {
        Self {
            disk,
            pool: Box::new(pool),
            page_table: HashMap::new(),
        }
    }

    pub fn fetch_page(&mut self, page_id: PageId) -> Result<Rc<Buffer>, Error> {
        dbg!(page_id);
        if let Some(&buffer_id) = self.page_table.get(&page_id) {
            let frame = &mut self.pool[buffer_id];
            let buffer = frame.use_buffer();
            return Ok(buffer);
        }
        let buffer_id = self.pool.evict().ok_or(Error::NoFreeBuffer)?;
        let frame = &mut self.pool[buffer_id];
        let evict_page_id = frame.buffer.page_id;
        {
            let buffer = Rc::get_mut(&mut frame.buffer).unwrap();
            if buffer.is_dirty.get() {
                self.disk
                    .write_page_data(evict_page_id, buffer.page.get_mut())?;
            }
            buffer.page_id = page_id;
            buffer.is_dirty.set(false);
            self.disk.read_page_data(page_id, buffer.page.get_mut())?;
            frame.reset_usage_count();
        }
        let buffer = frame.use_buffer();
        self.page_table.remove(&evict_page_id);
        self.page_table.insert(page_id, buffer_id);
        Ok(buffer)
    }

    pub fn create_page(&mut self) -> Result<Rc<Buffer>, Error> {
        let buffer_id = self.pool.evict().ok_or(Error::NoFreeBuffer)?;
        let frame = &mut self.pool[buffer_id];
        let evict_page_id = frame.buffer.page_id;
        let page_id = {
            let buffer = Rc::get_mut(&mut frame.buffer).unwrap();
            if buffer.is_dirty.get() {
                self.disk
                    .write_page_data(evict_page_id, buffer.page.get_mut())?;
            }
            let page_id = self.disk.allocate_page();
            *buffer = Buffer::default();
            buffer.page_id = page_id;
            buffer.is_dirty.set(true);
            page_id
        };
        let page = frame.use_buffer();
        self.page_table.remove(&evict_page_id);
        self.page_table.insert(page_id, buffer_id);
        Ok(page)
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        for (&page_id, &buffer_id) in self.page_table.iter() {
            let frame = &self.pool[buffer_id];
            let mut page = frame.buffer.page.borrow_mut();
            self.disk.write_page_data(page_id, page.as_mut())?;
            frame.buffer.is_dirty.set(false);
        }
        self.disk.sync()?;
        Ok(())
    }
}

#[cfg(test)]
mod buffer_pool_manager_test {
    use super::*;

    mod fetch_page {
        use super::*;
        use crate::buffer::{BufferId, ClockSweepBufferPool, Frame};
        use std::{
            cell::{Cell, RefCell},
            fs::remove_file,
        };

        #[test]
        fn バッファプールに存在しないページを読み込もうとした場合ディスクから読み込みバッファプールに書き込んだ後ページの内容を返すこと(
        ) {
            // Arrange
            let file_path = "buffer_pool_manager_test::fetch_page::0.txt";
            let page_id = PageId::new(0);
            let data = ['a' as u8; DiskManager::PAGE_SIZE];
            let mut buffer_pool_manager = {
                let mut disk = DiskManager::open(file_path).unwrap();
                disk.write_page_data(page_id, &data).unwrap();
                let pool = ClockSweepBufferPool::from(3);
                BufferPoolManager::new(disk, pool)
            };

            // Act
            let buffer = buffer_pool_manager.fetch_page(page_id).unwrap();

            // Assert
            assert_eq!(buffer.page_id, page_id);
            assert_eq!(buffer.page, RefCell::new(data));
            assert_eq!(
                *buffer_pool_manager.page_table.get(&page_id).unwrap(),
                BufferId::new(0)
            );

            // Cleanup
            remove_file(file_path).unwrap();
        }

        #[test]
        fn ページがバッファプールに存在する場合バッファプールの内容を読み込むこと() {
            // Arrange
            let file_path = "buffer_pool_manager_test::fetch_page::1.txt";
            let page_id = PageId::new(0);
            let data = ['a' as u8; DiskManager::PAGE_SIZE];
            let buffer_id = BufferId::new(0);
            let mut buffer_pool_manager = {
                let disk = DiskManager::open(file_path).unwrap();
                let frame = {
                    let buffer = Buffer {
                        page_id,
                        page: RefCell::new(data),
                        is_dirty: Cell::new(false),
                    };
                    Frame {
                        usage_count: 1,
                        buffer: Rc::new(buffer),
                    }
                };
                let mut pool = ClockSweepBufferPool::from(1);
                pool.buffers = vec![frame];
                BufferPoolManager::new(disk, pool)
            };
            buffer_pool_manager.page_table.insert(page_id, buffer_id);

            // Act
            let buffer = buffer_pool_manager.fetch_page(page_id).unwrap();

            // Assert
            assert_eq!(buffer.page_id, page_id);
            assert_eq!(buffer.page, RefCell::new(data));
            assert_eq!(buffer.is_dirty.get(), false);

            // Cleanup
            remove_file(file_path).unwrap();
        }
    }
}
