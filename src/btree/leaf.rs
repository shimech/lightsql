use crate::{disk::PageId, slotted::Slotted};
use zerocopy::LayoutVerified;

pub struct Header {
    prev_page_id: PageId,
    next_page_id: PageId,
}

pub struct Leaf<B> {
    header: LayoutVerified<B, Header>,
    body: Slotted<B>,
}
