use std::{
    mem::size_of,
    ops::{Index, IndexMut, Range},
};
use zerocopy::{AsBytes, ByteSlice, ByteSliceMut, FromBytes, FromZeroes, Ref};

#[derive(FromBytes, AsBytes, FromZeroes)]
#[repr(C)]
pub struct Header {
    slot_count: u16,
    free_space_offset: u16,
    _pad: u32,
}

#[derive(FromBytes, AsBytes, FromZeroes, Copy, Clone)]
#[repr(C)]
pub struct Pointer {
    offset: u16,
    len: u16,
}

impl Pointer {
    fn range(&self) -> Range<usize> {
        let start = self.offset as usize;
        let end = start + self.len as usize;
        start..end
    }
}

pub type Pointers<B> = Ref<B, [Pointer]>;

pub struct Slotted<B> {
    header: Ref<B, Header>,
    body: B,
}

impl<B: ByteSlice> Slotted<B> {
    pub fn new(bytes: B) -> Self {
        let (header, body) = Ref::new_from_prefix(bytes).expect("slotted header must be aligned");
        Self { header, body }
    }

    pub fn capacity(&self) -> usize {
        self.body.len()
    }

    pub fn slot_count(&self) -> usize {
        self.header.slot_count as usize
    }

    pub fn free_space(&self) -> usize {
        self.header.free_space_offset as usize - self.pointers_size()
    }

    fn pointers_size(&self) -> usize {
        size_of::<Pointer>() * self.slot_count()
    }

    fn pointers(&self) -> Pointers<&[u8]> {
        Pointers::new_slice(&self.body[..self.pointers_size()]).unwrap()
    }

    fn data(&self, pointer: Pointer) -> &[u8] {
        &self.body[pointer.range()]
    }
}

impl<B: ByteSliceMut> Slotted<B> {
    pub fn initialize(&mut self) {
        self.header.slot_count = 0;
        self.header.free_space_offset = self.body.len() as u16;
    }

    fn pointers_mut(&mut self) -> Pointers<&mut [u8]> {
        let pointers_size = self.pointers_size();
        Pointers::new_slice(&mut self.body[..pointers_size]).unwrap()
    }

    fn data_mut(&mut self, pointer: Pointer) -> &mut [u8] {
        &mut self.body[pointer.range()]
    }

    pub fn insert(&mut self, index: usize, len: usize) -> Option<()> {
        if self.free_space() < size_of::<Pointer>() + len {
            return None;
        }
        let original_slot_count = self.slot_count();
        self.header.free_space_offset -= len as u16;
        self.header.slot_count += 1;
        let free_space_offset = self.header.free_space_offset;
        let mut pointers_mut = self.pointers_mut();
        pointers_mut.copy_within(index..original_slot_count, index + 1);
        let pointer = &mut pointers_mut[index];
        pointer.offset = free_space_offset;
        pointer.len = len as u16;
        Some(())
    }

    pub fn remove(&mut self, index: usize) {
        self.resize(index, 0);
        self.pointers_mut().copy_within(index + 1.., index);
        self.header.slot_count -= 1;
    }

    pub fn resize(&mut self, index: usize, new_len: usize) -> Option<()> {
        let pointers = self.pointers();
        let original_len = pointers[index].len;
        let increment_len = new_len as isize - original_len as isize;
        if increment_len == 0 {
            return Some(());
        }
        if increment_len > self.free_space() as isize {
            return None;
        }
        let free_space_offset = self.header.free_space_offset as usize;
        let original_offset = pointers[index].offset;
        let shift_range = free_space_offset..original_offset as usize;
        let new_free_space_offset = (free_space_offset as isize - increment_len) as usize;
        self.header.free_space_offset = new_free_space_offset as u16;
        self.body
            .as_bytes_mut()
            .copy_within(shift_range, new_free_space_offset);
        let mut pointers_mut = self.pointers_mut();
        for pointer in pointers_mut.iter_mut() {
            if pointer.offset <= original_offset {
                pointer.offset = (pointer.offset as isize - increment_len) as u16;
            }
        }
        let pointer = &mut pointers_mut[index];
        pointer.len = new_len as u16;
        if new_len == 0 {
            pointer.offset = new_free_space_offset as u16;
        }
        Some(())
    }
}

impl<B: ByteSlice> Index<usize> for Slotted<B> {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        self.data(self.pointers()[index])
    }
}

impl<B: ByteSliceMut> IndexMut<usize> for Slotted<B> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data_mut(self.pointers()[index])
    }
}
