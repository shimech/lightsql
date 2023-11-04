use crate::disk::{DiskManager, PageId};
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BufferId(pub(crate) usize);
impl BufferId {
    pub(crate) fn value(&self) -> usize {
        self.0
    }
}

pub type Page = [u8; DiskManager::PAGE_SIZE];

#[derive(Debug)]
pub struct Buffer {
    pub page_id: PageId,
    pub page: RefCell<Page>,
    pub is_dirty: Cell<bool>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            page_id: Default::default(),
            page: RefCell::new([0u8; DiskManager::PAGE_SIZE]),
            is_dirty: Cell::new(false),
        }
    }
}
