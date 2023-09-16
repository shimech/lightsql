use std::cmp::Ordering::{self, Greater, Less};

pub fn binary_search_by<F>(mut size: usize, mut f: F) -> Result<usize, usize>
where
    F: FnMut(usize) -> Ordering,
{
    let mut left = 0;
    let mut right = size;
    while left < right {
        let mid = left + size / 2;
        let cmp = f(mid);
        if cmp == Less {
            left = mid + 1;
        } else if cmp == Greater {
            right = mid;
        } else {
            return Ok(mid);
        }
        size = right - left;
    }
    Err(left)
}
