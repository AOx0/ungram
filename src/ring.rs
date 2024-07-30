#[derive(Debug, Clone)]
pub struct Ring<T, const SIZE: usize> {
    data: [T; SIZE],
    head: usize,
}

impl<T, const SIZE: usize> Ring<T, SIZE> {
    pub fn push(&mut self, value: T) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % SIZE;
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < SIZE {
            Some(&self.data[(self.head + index) % SIZE])
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().cycle().skip(self.head).take(SIZE)
    }
}

impl<T: Default + Clone + Copy, const SIZE: usize> Ring<T, SIZE> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); SIZE],
            head: 0,
        }
    }

    pub fn data(&self) -> [T; SIZE] {
        let mut data = [T::default(); SIZE];
        for i in 0..SIZE {
            data[i] = self.data[(self.head + i) % SIZE];
        }

        data
    }
}

impl<T, const SIZE: usize> std::ops::Index<usize> for Ring<T, SIZE> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::ring::Ring;

    #[test]
    fn test_ring_2() {
        let mut ring = Ring::<i8, 2>::new();
        ring.push(1);
        ring.push(2);
        assert_eq!(ring[0], 1);
        assert_eq!(ring[1], 2);
        ring.push(3);
        assert_eq!(ring[0], 2);
        assert_eq!(ring[1], 3);
    }

    #[test]
    fn test_ring() {
        let mut ring = Ring::<i8, 3>::new();
        ring.push(1);
        ring.push(2);
        ring.push(3);
        assert_eq!(ring[0], 1);
        assert_eq!(ring[1], 2);
        assert_eq!(ring[2], 3);
        ring.push(4);
        assert_eq!(ring[0], 2);
        assert_eq!(ring[1], 3);
        assert_eq!(ring[2], 4);
    }
}
