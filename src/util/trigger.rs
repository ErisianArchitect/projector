use std::sync::{
    Arc,
    atomic::{
        AtomicBool,
        Ordering,
    },
};

#[derive(Debug, Clone)]
pub struct Trigger {
    trigger: Arc<AtomicBool>,
}

impl Trigger {
    #[inline]
    pub fn new(active: bool) -> Self {
        Self {
            trigger: Arc::new(AtomicBool::new(active)),
        }
    }

    #[inline]
    pub fn swap(&self, value: bool) -> bool {
        self.trigger.swap(value, Ordering::Relaxed)
    }

    #[inline]
    pub fn set(&self, value: bool) {
        self.trigger.store(value, Ordering::Relaxed);
    }

    #[inline]
    pub fn activate(&self) -> bool {
        !self.swap(true)
    }

    #[inline]
    pub fn deactivate(&self) -> bool {
        self.swap(false)
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.trigger.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn is_inactive(&self) -> bool {
        !self.is_active()
    }

    pub fn trigger_ref(&self) -> AtomicTrigger<'_> {
        AtomicTrigger {
            trigger: &self.trigger
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct AtomicTrigger<'a> {
    trigger: &'a AtomicBool,
}

impl<'a> AtomicTrigger<'a> {
    #[inline]
    pub fn new(trigger: &'a AtomicBool) -> Self {
        Self { trigger }
    }

    #[inline]
    pub fn from_trigger(trigger: &'a Trigger) -> Self {
        Self {
            trigger: &trigger.trigger
        }
    }

    #[inline]
    pub fn swap(self, active: bool) -> bool {
        self.trigger.swap(active, Ordering::Relaxed)
    }

    #[inline]
    pub fn set(self, active: bool) {
        self.trigger.store(active, Ordering::Relaxed);
    }

    #[inline]
    pub fn activate(self) -> bool {
        !self.swap(true)
    }

    #[inline]
    pub fn deactivate(self) -> bool {
        self.swap(false)
    }

    #[inline]
    pub fn is_active(self) -> bool {
        self.trigger.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn is_inactive(self) -> bool {
        !self.is_active()
    }

    #[inline]
    pub fn inner(self) -> &'a AtomicBool {
        self.trigger
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn trigger_test() {
        let trigger = Trigger::new(false);

        assert!(!trigger.is_active() && trigger.is_inactive());
        trigger.activate();
        assert!(trigger.is_active() && !trigger.is_inactive());
        trigger.deactivate();
        assert!(!trigger.is_active() && trigger.is_inactive());
        trigger.set(true);
        assert!(trigger.is_active() && !trigger.is_inactive());
        trigger.set(false);
        assert!(!trigger.is_active() && trigger.is_inactive());

        {
            let trigger = trigger.trigger_ref();

            assert!(!trigger.is_active() && trigger.is_inactive());
            trigger.activate();
            assert!(trigger.is_active() && !trigger.is_inactive());
            trigger.deactivate();
            assert!(!trigger.is_active() && trigger.is_inactive());
            trigger.set(true);
            assert!(trigger.is_active() && !trigger.is_inactive());
            trigger.set(false);
            assert!(!trigger.is_active() && trigger.is_inactive());

            fn take_trigger(trigger: AtomicTrigger<'_>) {
                assert!(!trigger.is_active() && trigger.is_inactive());
                trigger.activate();
                assert!(trigger.is_active() && !trigger.is_inactive());
                trigger.deactivate();
                assert!(!trigger.is_active() && trigger.is_inactive());
                trigger.set(true);
                assert!(trigger.is_active() && !trigger.is_inactive());
                trigger.set(false);
                assert!(!trigger.is_active() && trigger.is_inactive());
                trigger.activate();
            }

            take_trigger(trigger);
            assert!(trigger.is_active() && !trigger.is_inactive());

        }
    }
}