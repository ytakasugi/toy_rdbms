use std::cmp::Ordering::{self, Greater, Less};

pub fn binary_search<F>(mut size: usize, mut f: F) -> Result<usize, usize>
where
    F: FnMut(usize) -> Ordering,
{
    let mut left = 0;
    let mut right = size;

    while left < right {
        let mid = left + size / 2;
        let cmp = f(mid);
        // cmpがLessの場合、探索対象は中央値よりも右側にあると判断し、探索範囲の左端をmid + 1に更新。
        if cmp == Less {
            left = mid + 1;
        // cmpがGreaterの場合、探索対象は中央値よりも左側にあると判断し、探索範囲の右端をmidに更新
        } else if cmp == Greater {
            right = mid;
        } else {
            // cmpがEqualの場合、探索対象が見つかったと判断し、Ok(mid)を返して探索を終了。
            return Ok(mid)
        }
        size = right - left;
    }
    Err(left)
}