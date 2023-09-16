use crate::disk::PageId;
use zerocopy::{ByteSlice, Ref};

pub struct Header {
    pub root_page_id: PageId,
}

pub struct Meta<B> {
    pub header: Ref<B, Header>,
    _unused: B,
}

impl<B: ByteSlice> Meta<B> {
    pub fn new(bytes: B) -> Self {
        let (header, _unused) = Ref::new_from_prefix(bytes).expect("meta page must be allowed");
        Self { header, _unused }
    }
}
