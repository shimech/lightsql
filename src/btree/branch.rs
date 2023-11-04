use super::Pair;
use crate::{
    bsearch::binary_search_by,
    disk::PageId,
    slotted::{self, Slotted},
};
use std::mem::size_of;
use zerocopy::{AsBytes, ByteSlice, ByteSliceMut, FromBytes, FromZeroes, Ref};

#[derive(FromBytes, FromZeroes, AsBytes)]
#[repr(C)]
pub struct Header {
    right_child: PageId,
}

pub struct Branch<B> {
    header: Ref<B, Header>,
    body: Slotted<B>,
}

impl<B: ByteSlice> Branch<B> {
    pub fn new(bytes: B) -> Self {
        let (header, body) = Ref::new_from_prefix(bytes).expect("branch header must aligned");
        let body = Slotted::new(body);
        Self { header, body }
    }

    pub fn pair_count(&self) -> usize {
        self.body.slot_count()
    }

    pub fn search_slod_id(&self, key: &[u8]) -> Result<usize, usize> {
        binary_search_by(self.pair_count(), |slot_id| {
            self.pair_at(slot_id).key.cmp(key)
        })
    }

    pub fn search_child(&self, key: &[u8]) -> PageId {
        let child_idx = self.search_child_idx(key);
        self.child_at(child_idx)
    }

    pub fn search_child_idx(&self, key: &[u8]) -> usize {
        match self.search_slod_id(key) {
            Ok(slot_id) => slot_id + 1,
            Err(slot_id) => slot_id,
        }
    }

    pub fn child_at(&self, child_idx: usize) -> PageId {
        if child_idx == self.pair_count() {
            self.header.right_child
        } else {
            self.pair_at(child_idx).value.into()
        }
    }

    pub fn pair_at(&self, slot_id: usize) -> Pair {
        Pair::from_bytes(&self.body[slot_id])
    }

    pub fn max_pair_size(&self) -> usize {
        self.body.capacity() / 2 - size_of::<slotted::Pointer>()
    }
}

impl<B: ByteSliceMut> Branch<B> {
    pub fn initialize(&mut self, key: &[u8], left_child: PageId, right_child: PageId) {
        self.body.initialize();
        self.insert(0, key, left_child)
            .expect("new leaf must have space");
        self.header.right_child = right_child;
    }

    pub fn fill_right_child(&mut self) -> Vec<u8> {
        let last_id = self.pair_count() - 1;
        let Pair { key, value } = self.pair_at(last_id);
        let right_child: PageId = value.into();
        let key_vec = key.to_vec();
        self.body.remove(last_id);
        self.header.right_child = right_child;
        key_vec
    }

    pub fn insert(&mut self, slot_id: usize, key: &[u8], page_id: PageId) -> Option<()> {
        let pair = Pair {
            key,
            value: page_id.as_bytes(),
        };
        let pair_bytes = pair.to_bytes();
        assert!(pair_bytes.len() <= self.max_pair_size());
        self.body.insert(slot_id, pair_bytes.len())?;
        self.body[slot_id].copy_from_slice(&pair_bytes);
        Some(())
    }

    fn is_half_full(&self) -> bool {
        2 * self.body.free_space() < self.body.capacity()
    }

    pub fn split_insert(
        &mut self,
        new_branch: &mut Branch<impl ByteSliceMut>,
        new_key: &[u8],
        new_page_id: PageId,
    ) -> Vec<u8> {
        new_branch.body.initialize();
        loop {
            if new_branch.is_half_full() {
                let index = self
                    .search_slod_id(new_key)
                    .expect_err("key must be unique");
                self.insert(index, new_key, new_page_id)
                    .expect("old branch must have space");
                break;
            }
            if self.pair_at(0).key < new_key {
                self.transfer(new_branch);
            } else {
                new_branch
                    .insert(new_branch.pair_count(), new_key, new_page_id)
                    .expect("new branch must have space");
                while !new_branch.is_half_full() {
                    self.transfer(new_branch)
                }
                break;
            }
        }
        new_branch.fill_right_child()
    }

    pub fn transfer(&mut self, dest: &mut Branch<impl ByteSliceMut>) {
        let next_index = dest.pair_count();
        dest.body
            .insert(next_index, self.body[0].len())
            .expect("no space in dest branch");
        self.body.remove(0)
    }
}
