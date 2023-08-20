use crate::disk::PageId;
use zerocopy::{ByteSlice, LayoutVerified};

pub struct Header {
    pub root_page_id: PageId,
}

pub struct Meta<B> {
    pub header: LayoutVerified<B, Header>,
    _unused: B,
}

impl<B: ByteSlice> Meta<B> {
    pub fn new(bytes: B) -> Self {
        let (header, _unused) =
            LayoutVerified::new_from_prefix(bytes).expect("meta page must be allowed");
        Self { header, _unused }
    }
}
