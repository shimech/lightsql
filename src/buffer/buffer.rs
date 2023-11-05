use crate::disk::{DiskManager, PageId};
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BufferId(usize);
impl BufferId {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod buffer_id_test {
    use super::*;

    mod new {
        use super::*;

        #[allow(non_snake_case)]
        #[test]
        fn BufferIdが正しく生成されること() {
            assert_eq!(BufferId::new(0), BufferId(0))
        }
    }

    mod value {
        use super::*;

        #[test]
        fn 内部で保持する値を返すこと() {
            assert_eq!(BufferId(0).value(), 0)
        }
    }
}

pub type Page = [u8; DiskManager::PAGE_SIZE];

#[derive(Debug, PartialEq)]
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
