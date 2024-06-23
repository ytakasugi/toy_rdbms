use std::{
    cmp::Ordering::{self, Greater, Less},
    result,
};

pub fn binary_search_by<F>(mut size: usize, mut f: F) -> Result<usize, usize>
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
            return Ok(mid);
        }
        size = right - left;
    }
    Err(left)
}

#[cfg(test)]
mod tests {
    use super::binary_search_by;

    #[test]
    fn test() {
        let a = vec![1, 2, 3, 5, 8, 13, 21];
        assert_eq!(Ok(0), binary_search_by(a.len(), |idx| a[idx].cmp(&1)));
        assert_eq!(Err(0), binary_search_by(a.len(), |idx| a[idx].cmp(&0)));
        assert_eq!(Ok(1), binary_search_by(a.len(), |idx| a[idx].cmp(&2)));
        assert_eq!(Ok(4), binary_search_by(a.len(), |idx| a[idx].cmp(&8)));
        assert_eq!(Err(4), binary_search_by(a.len(), |idx| a[idx].cmp(&6)));
        assert_eq!(Ok(6), binary_search_by(a.len(), |idx| a[idx].cmp(&21)));
        assert_eq!(Err(7), binary_search_by(a.len(), |idx| a[idx].cmp(&22)));
    }
}
