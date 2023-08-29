#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StackVec<const CAPACITY: usize, T: Copy + Default> {
    len: usize,
    storage: [T; CAPACITY],
}

impl<const CAPACITY: usize, T: Copy + Default> StackVec<CAPACITY, T> {
    pub fn new() -> Self {
        Self {
            len: 0,
            storage: [T::default(); CAPACITY],
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, item: T) {
        assert!(self.len < CAPACITY);
        self.storage[self.len] = item;
        self.len += 1;
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.storage[0..self.len].iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.storage[0..self.len].iter_mut()
    }
}

impl<const CAPACITY: usize, T: Copy + Default> std::ops::Index<usize> for StackVec<CAPACITY, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len);
        &self.storage[index]
    }
}

impl<const CAPACITY: usize, T: Copy + Default> std::ops::IndexMut<usize> for StackVec<CAPACITY, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len);
        &mut self.storage[index]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let new = StackVec::<8, u8>::new();
        assert_eq!(new.len(), 0);
    }

    #[test]
    fn push_one() {
        let mut vec = StackVec::<8, u8>::new();
        vec.push(3);
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], 3);
        assert_eq!(vec.iter().copied().collect::<Vec<_>>(), vec![3]);
    }

    #[test]
    fn push_multiple() {
        let mut vec = StackVec::<8, u8>::new();
        vec.push(3);
        vec.push(54);
        vec.push(8);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 3);
        assert_eq!(vec[1], 54);
        assert_eq!(vec[2], 8);
        assert_eq!(vec.iter().copied().collect::<Vec<_>>(), vec![3, 54, 8]);
    }
}
