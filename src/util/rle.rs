
pub struct RunLengthCounter<T: PartialEq<T>> {
    current: Option<T>,
    count: usize,
}

impl<T: PartialEq<T>> RunLengthCounter<T> {
    pub fn new() -> Self {
        Self {
            current: None,
            count: 0,
        }
    }

    pub fn push(&mut self, value: T) -> usize {
        self.current = Some(if let Some(current) = self.current.take() {
            if current == value {
                self.count += 1;
            } else {
                self.count = 1;
            }
            value
        } else {
            self.count = 1;
            value
        });
        self.count
    }

    #[inline]
    pub const fn count(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn rle_counter_test() {
        let mut counter = RunLengthCounter::<u8>::new();
        let data = b"aaabbccccddeeeeeefffgggghhhiijjjjjjjjkk";

    }
}