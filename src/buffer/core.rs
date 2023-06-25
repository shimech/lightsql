use crate::disk::{DiskManager, PageId};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BufferId(pub usize);
impl BufferId {
    pub fn value(&self) -> usize {
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
    pub usage_count: u64,
    pub buffer: Rc<Buffer>,
}
