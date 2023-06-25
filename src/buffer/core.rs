use crate::disk::{DiskManager, PageId};
use std::{
    ops::{Index, IndexMut},
    rc::Rc,
};

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

pub struct BufferList(pub Vec<Frame>);

impl BufferList {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<BufferId> for BufferList {
    type Output = Frame;

    fn index(&self, buffer_id: BufferId) -> &Self::Output {
        &self.0[buffer_id.value()]
    }
}

impl IndexMut<BufferId> for BufferList {
    fn index_mut(&mut self, buffer_id: BufferId) -> &mut Self::Output {
        &mut self.0[buffer_id.value()]
    }
}
