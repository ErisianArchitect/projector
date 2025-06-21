use std::sync::{atomic::{AtomicBool, Ordering}, Arc};


mod private {
    use std::sync::{atomic::AtomicBool, Arc};
    pub trait Sealed {}
    impl Sealed for AtomicBool {}
    impl<'a> Sealed for &'a AtomicBool {}
    impl<'a> Sealed for Arc<AtomicBool> {}
}

/// This is just a wrapper around AtomicBool with mark and reset functions.
#[repr(transparent)]
#[derive(Debug)]
pub struct AtomicMarker<T: private::Sealed = AtomicBool> {
    marker: T,
}

/// A [Marker] type that can be freely copied and passed around in your program.
pub type MarkerRef<'a> = AtomicMarker<&'a AtomicBool>;
pub type Marker = AtomicMarker<AtomicBool>;
pub type ArcMarker = AtomicMarker<Arc<AtomicBool>>;

impl<'a> MarkerRef<'a> {
    #[inline]
    pub const fn from_atomic(marker: &'a AtomicBool) -> Self {
        Self {
            marker,
        }
    }

    #[inline]
    pub fn mark(self) -> bool {
        !self.marker.swap(true, Ordering::Relaxed)
    }

    #[inline]
    pub fn mark_if(self, condition: bool) -> bool {
        condition && self.mark()
    }

    #[inline]
    pub fn reset(self) -> bool {
        self.marker.swap(false, Ordering::Relaxed)
    }

    #[inline]
    pub fn is_marked(self) -> bool {
        self.marker.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn mark_only(self) -> MarkOnly<'a> {
        MarkOnly::new(self)
    }
}

impl<'a> Clone for MarkerRef<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> Copy for MarkerRef<'a> {}

impl Marker {
    #[inline]
    pub const fn new() -> Self {
        Self {
            marker: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn mark(&self) -> bool {
        !self.marker.swap(true, Ordering::Relaxed)
    }

    #[inline]
    pub fn mark_if(&self, condition: bool) -> bool {
        condition && self.mark()
    }

    #[inline]
    pub fn reset(&self) -> bool {
        self.marker.swap(false, Ordering::Relaxed)
    }

    #[inline]
    pub fn is_marked(&self) -> bool {
        self.marker.load(Ordering::Relaxed)
    }

    #[inline]
    pub const fn marker_ref(&self) -> MarkerRef {
        AtomicMarker::from_atomic(&self.marker)
    }

    #[inline]
    pub const fn mark_only(&self) -> MarkOnly {
        MarkOnly::new(self.marker_ref())
    }
}

impl ArcMarker {
    #[inline]
    pub fn new() -> Self {
        Self {
            marker: Arc::new(AtomicBool::new(false)),
        }
    }

    #[inline]
    pub fn mark(&self) -> bool {
        !self.marker.swap(true, Ordering::Relaxed)
    }

    #[inline]
    pub fn mark_if(&self, condition: bool) -> bool {
        condition && self.mark()
    }

    #[inline]
    pub fn reset(&self) -> bool {
        self.marker.swap(false, Ordering::Relaxed)
    }

    #[inline]
    pub fn is_marked(&self) -> bool {
        self.marker.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn marker_ref(&self) -> MarkerRef {
        MarkerRef::from_atomic(self.marker.as_ref())
    }

    #[inline]
    pub fn mark_only(&self) -> MarkOnly {
        MarkOnly::new(self.marker_ref())
    }
}

impl Clone for ArcMarker {
    fn clone(&self) -> Self {
        Self {
            marker: self.marker.clone(),
        }
    }
}

#[inline]
pub const fn marker() -> AtomicMarker<AtomicBool> {
    Marker::new()
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct MarkOnly<'a> {
    marker: MarkerRef<'a>,
}

impl<'a> MarkOnly<'a> {
    #[inline]
    pub const fn new(marker: MarkerRef<'a>) -> Self {
        Self {
            marker,
        }
    }

    #[inline]
    pub fn mark(self) -> bool {
        self.marker.mark()
    }

    #[inline]
    pub fn mark_if(self, condition: bool) -> bool {
        self.marker.mark_if(condition)
    }

    #[inline]
    pub fn is_marked(self) -> bool {
        self.marker.is_marked()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn marker_test() {
        fn take_marker(marker: MarkerRef) {
            marker.mark();
        }
        fn take_mark_only(marker: MarkOnly, mark: bool) {
            if mark {
                marker.mark();
            }
        }
        let marker = marker();
        let mark = marker.marker_ref();
        take_marker(mark);
        assert!(mark.is_marked());
        mark.reset();
        assert!(!mark.is_marked());
        mark.mark();
        assert!(mark.is_marked());
        mark.reset();
        assert!(!mark.is_marked());
        take_marker(mark);
        assert!(mark.is_marked());
        mark.reset();
        let mark_only = mark.mark_only();
        take_mark_only(mark_only, false);
        assert!(!mark.is_marked());
        take_mark_only(mark_only, true);
        assert!(mark.is_marked());

    }
}