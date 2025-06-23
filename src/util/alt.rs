
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AltSelection {
    Left = 0,
    Right = 1,
    LockLeft = 2,
    LockRight = 3,
}

impl AltSelection {
    // const fn index(self) -> usize {
    //     match self {
    //         AltSelection::Left => 0,
    //         AltSelection::Right => 1,
    //     }
    // }

    #[inline]
    pub fn select<T>(self, left: T, right: T) -> T {
        match self {
            Self::Left => left,
            Self::Right => right,
            Self::LockLeft => left,
            Self::LockRight => right,
        }
    }

    #[inline]
    pub fn alternate(&mut self) {
        *self = match *self {
            AltSelection::Left => AltSelection::Right,
            AltSelection::Right => AltSelection::Left,
            _ => return,
        }
    }

    #[inline]
    pub fn set_left(&mut self) {
        *self = Self::Left;
    }

    #[inline]
    pub fn set_right(&mut self) {
        *self = Self::Right;
    }

    #[inline]
    pub fn lock(&mut self) {
        *self = match *self {
            AltSelection::Left => Self::LockLeft,
            AltSelection::Right => Self::LockRight,
            _ => return,
        }
    }

    #[inline]
    pub fn lock_left(&mut self) {
        *self = Self::LockLeft;
    }

    #[inline]
    pub fn lock_right(&mut self) {
        *self = Self::LockRight;
    }

    #[inline]
    pub fn unlock(&mut self) {
        *self = match *self {
            AltSelection::LockLeft => Self::Left,
            AltSelection::LockRight => Self::Right,
            _ => return,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Alternator<T: Clone> {
    values: (T, T),
    selection: AltSelection,
}

impl<T: Clone> Alternator<T> {
    #[inline]
    pub const fn new(initial: T, alternate: T) -> Self {
        Self {
            values: (initial, alternate),
            selection: AltSelection::Left,
        }
    }

    #[inline]
    pub fn current(&self) -> &T {
        self.selection.select(&self.values.0, &self.values.1)
    }

    #[inline]
    pub fn alt(&self) -> &T {
        self.selection.select(&self.values.1, &self.values.0)
    }

    pub fn next(&mut self) -> T {
        let value = self.selection.select(&self.values.0, &self.values.1).clone();
        self.selection.alternate();
        value
    }

    #[inline]
    pub fn left(&self) -> &T {
        &self.values.0
    }

    #[inline]
    pub fn right(&self) -> &T {
        &self.values.1
    }
}

impl<T: Copy> Copy for Alternator<T> {}

impl<T: Clone> Iterator for Alternator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    }
}

impl<T: Clone> std::ops::Deref for Alternator<T> {
    type Target = AltSelection;
    fn deref(&self) -> &Self::Target {
        &self.selection
    }
}

impl<T: Clone> std::ops::DerefMut for Alternator<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.selection
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn alternator_test() {
        let mut alt = Alternator::new("foo", "bar");
        alt.set_right();
        println!("Left: {}", alt.left());
        println!("Right: {}", alt.right());
        for i in 0..10 {
            if i == 4 {
                alt.lock_right();
            }
            if i == 7 {
                alt.unlock();
            }
            println!("{}", alt.next());
        }
    }
}