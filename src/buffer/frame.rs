use super::Buffer;
use std::rc::Rc;

#[derive(Default)]
pub struct Frame {
    pub(crate) usage_count: u64,
    pub(crate) buffer: Rc<Buffer>,
}
