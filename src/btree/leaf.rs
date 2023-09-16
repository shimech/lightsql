use crate::{disk::PageId, slotted::Slotted};
use zerocopy::Ref;

pub struct Header {
    prev_page_id: PageId,
    next_page_id: PageId,
}

pub struct Leaf<B> {
    header: Ref<B, Header>,
    body: Slotted<B>,
}
