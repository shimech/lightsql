use crate::buffer::BufferPoolManager;
use crate::tuple;
use crate::{btree::BTree, disk::PageId};
use anyhow::Result;

#[derive(Debug)]
pub struct SimpleTable {
    pub meta_page_id: PageId,
    pub key_elems_count: usize,
}

impl SimpleTable {
    pub fn create(&mut self, bufmgr: &mut BufferPoolManager) -> Result<()> {
        let btree = BTree::create(bufmgr)?;
        self.meta_page_id = btree.meta_page_id;
        Ok(())
    }

    pub fn insert(&self, bufmgr: &mut BufferPoolManager, record: &[&[u8]]) -> Result<()> {
        let btree = BTree::new(self.meta_page_id);
        let mut key = vec![];
        tuple::encode(record[..self.key_elems_count].iter(), &mut key);
        let mut value = vec![];
        tuple::encode(record[self.key_elems_count..].iter(), &mut value);
        btree.insert(bufmgr, &key, &value)?;
        Ok(())
    }
}

pub struct Table {
    pub meta_page_id: PageId,
    pub key_elems_count: usize,
    pub unique_indices: Vec<UniqueIndex>,
}

pub struct UniqueIndex {
    pub meta_page_id: PageId,
    pub skey: Vec<usize>,
}
