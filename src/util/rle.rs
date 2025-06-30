use std::ops::Range;


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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RunLength {
    pub first_index: usize,
    pub range: Range<usize>,
}

impl RunLength {
    #[inline]
    pub const fn new(first_index: usize, range: Range<usize>) -> Self {
        Self { first_index, range }
    }

    #[inline]
    pub const fn new_start(first_index: usize) -> Self {
        Self::new(first_index, first_index..first_index + 1)
    }

    #[inline]
    fn replace(&mut self, new_value: RunLength) -> RunLength {
        std::mem::replace(self, new_value)
    }
}

pub fn run_length_encode_into<T: PartialEq<T>>(data: &[T], encoding: &mut Vec<RunLength>) -> usize {
    if data.is_empty() {
        return 0;
    }
    let mut current = RunLength::new_start(0);
    let mut index = 1usize;
    let mut count = 1;
    while index < data.len() {
        if data[index] == data[current.first_index] {
            current.range.end += 1;
        } else {
            encoding.push(current.replace(RunLength::new_start(index)));
            count += 1;
        }
        index += 1;
    }
    encoding.push(current);
    count + 1
}

pub fn run_length_encode<T: PartialEq<T>>(data: &[T]) -> Vec<RunLength> {
    let mut encoding = Vec::new();
    run_length_encode_into(data, &mut encoding);
    encoding
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn rle_counter_test() {
        let data = b"aaabbccccddeeeeeefffgggghhhiijjjjjjjjabc";
        let encoding = run_length_encode(data);
        for rl in encoding {
            let s = str::from_utf8(&data[rl.range]).unwrap_or("<error>");
            println!("Run: {s}");
        }
    }
}