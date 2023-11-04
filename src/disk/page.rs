use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, FromBytes, FromZeroes, AsBytes)]
#[repr(C)]
pub struct PageId(pub u64);
impl PageId {
    pub const INVALID_PAGE_ID: PageId = PageId(u64::MAX);

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn valid(self) -> Option<Self> {
        if self == Self::INVALID_PAGE_ID {
            None
        } else {
            Some(self)
        }
    }
}

impl Default for PageId {
    fn default() -> Self {
        Self::INVALID_PAGE_ID
    }
}

impl From<Option<PageId>> for PageId {
    fn from(page_id: Option<PageId>) -> Self {
        page_id.unwrap_or_default()
    }
}

impl From<&[u8]> for PageId {
    fn from(bytes: &[u8]) -> Self {
        let arr = bytes.try_into().unwrap();
        PageId(u64::from_ne_bytes(arr))
    }
}

#[cfg(test)]
mod page_id_test {
    use super::*;

    mod value {
        use super::*;

        #[test]
        fn 内部で保持する値を返すこと() {
            assert_eq!(PageId(0).value(), 0)
        }
    }

    mod next {
        use super::*;

        #[allow(non_snake_case)]
        #[test]
        fn _1だけ足されたPageIdを返すこと() {
            let page_id = PageId(0).next();
            assert_eq!(page_id, PageId(1));
        }
    }
}
