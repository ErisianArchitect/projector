
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AltSelection {
    Left = 0,
    Right = 1,
}

impl AltSelection {
    // const fn index(self) -> usize {
    //     match self {
    //         AltSelection::Left => 0,
    //         AltSelection::Right => 1,
    //     }
    // }

    #[inline]
    fn select<'a, T>(self, left: &'a T, right: &'a T) -> &'a T {
        match self {
            Self::Left => left,
            Self::Right => right,
        }
    }

    #[inline]
    fn alternate(&mut self) {
        *self = match *self {
            AltSelection::Left => AltSelection::Right,
            AltSelection::Right => AltSelection::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Alternator<T: Clone> {
    values: [T; 2],
    selection: AltSelection,
}

impl<T: Clone> Alternator<T> {
    pub const fn new(initial: T, alternate: T) -> Self {
        Self {
            values: [initial, alternate],
            selection: AltSelection::Left,
        }
    }

    pub fn next(&mut self) -> T {
        let value = self.selection.select(&self.values[0], &self.values[1]).clone();
        self.selection.alternate();
        value
    }
}

impl<T: Copy> Copy for Alternator<T> {}