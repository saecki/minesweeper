/// An Iterator that yields all unordered combinations of k elements from a pool of n numbers.
pub struct CombinationIter {
    indices: [u8; 8],
    n: u8,
    k: u8,
    stop: bool,
}

impl CombinationIter {
    pub fn new(n: u8, k: u8) -> Self {
        let mut indices = [0; 8];
        for i in 0..k {
            indices[i as usize] = i;
        }

        Self {
            indices,
            n,
            k,
            stop: false,
        }
    }
}

impl Iterator for CombinationIter {
    type Item = [bool; 8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop {
            return None;
        }
        let mut nums = [false; 8];
        for idx in self.indices[0..self.k as usize].iter() {
            nums[*idx as usize] = true;
        }

        // increment indices
        for i in (0..self.k).rev() {
            self.indices[i as usize] += 1;
            if self.indices[i as usize] >= self.n {
                if i > 0 {
                    self.indices[i as usize] = self.indices[i as usize - 1] + 2;
                }
                if self.indices[i as usize] >= self.n {
                    self.stop = true;
                }
            } else {
                break;
            }
        }

        Some(nums)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn check<const SIZE: usize>(n: u8, k: u8, expected: [[bool; 8]; SIZE]) {
        let values: Vec<_> = CombinationIter::new(n, k).into_iter().collect();
        assert_eq!(values, expected);
    }

    #[test]
    fn from_1_take_0() {
        check(
            1,
            0,
            [[false, false, false, false, false, false, false, false]],
        );
    }

    #[test]
    fn from_5_take_0() {
        check(
            5,
            0,
            [[false, false, false, false, false, false, false, false]],
        );
    }

    #[test]
    fn from_1_take_1() {
        check(
            1,
            1,
            [[true, false, false, false, false, false, false, false]],
        );
    }

    #[test]
    fn from_2_take_2() {
        check(
            2,
            2,
            [[true, true, false, false, false, false, false, false]],
        );
    }

    #[test]
    fn from_2_take_1() {
        check(
            2,
            1,
            [
                [true, false, false, false, false, false, false, false],
                [false, true, false, false, false, false, false, false],
            ],
        );
    }

    #[test]
    fn from_3_take_1() {
        check(
            3,
            1,
            [
                [true, false, false, false, false, false, false, false],
                [false, true, false, false, false, false, false, false],
                [false, false, true, false, false, false, false, false],
            ],
        );
    }

    #[test]
    fn from_3_take_2() {
        check(
            3,
            2,
            [
                [true, true, false, false, false, false, false, false],
                [true, false, true, false, false, false, false, false],
                [false, true, true, false, false, false, false, false],
            ],
        );
    }
}
