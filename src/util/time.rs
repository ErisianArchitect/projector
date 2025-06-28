use std::time::{
    Duration,
    Instant,
    SystemTime,
};

use crate::ext::Replace;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stopwatch {
    last_time: Instant,
}

impl Stopwatch {
    #[inline]
    pub fn start() -> Self {
        Self { last_time: Instant::now() }
    }

    #[inline]
    pub fn reset(&mut self) -> Duration {
        self.last_time
            .replace(Instant::now())
            .elapsed()
    }
}

impl std::ops::Deref for Stopwatch {
    type Target = Instant;

    fn deref(&self) -> &Self::Target {
        &self.last_time
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timer {
    deadline: Instant,
}

impl Timer {
    #[inline]
    pub fn wait(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }

    #[inline]
    pub fn wait_nanos(nanos: u64) -> Self {
        Self::wait(Duration::from_nanos(nanos))
    }

    #[inline]
    pub fn wait_micros(micros: u64) -> Self {
        Self::wait(Duration::from_micros(micros))
    }

    #[inline]
    pub fn wait_millis(millis: u64) -> Self {
        Self::wait(Duration::from_millis(millis))
    }

    #[inline]
    pub fn wait_secs(secs: u64) -> Self {
        Self::wait(Duration::from_secs(secs))
    }

    #[inline]
    pub fn wait_secs_f32(secs: f32) -> Self {
        Self::wait(Duration::from_secs_f32(secs))
    }

    #[inline]
    pub fn wait_secs_f64(secs: f64) -> Self {
        Self::wait(Duration::from_secs_f64(secs))
    }

    #[inline]
    pub fn wait_mins(mins: u64) -> Self {
        Self::wait_secs(mins * 60)
    }

    #[inline]
    pub fn wait_mins_f32(mins: f32) -> Self {
        Self::wait_secs_f32(mins * 60.0)
    }

    #[inline]
    pub fn wait_mins_f64(mins: f64) -> Self {
        Self::wait_secs_f64(mins * 60.0)
    }

    #[inline]
    pub fn wait_hours(hours: u64) -> Self {
        Self::wait_secs(hours * 3600)
    }

    #[inline]
    pub fn wait_hours_f32(hours: f32) -> Self {
        Self::wait_secs_f32(hours * 3600.0)
    }

    #[inline]
    pub fn wait_hours_f64(hours: f64) -> Self {
        Self::wait_secs_f64(hours * 3600.0)
    }

    #[inline]
    pub fn finished(self) -> bool {
        Instant::now() >= self.deadline
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatTimer {
    current_deadline: Instant,
    duration: Duration,
}

impl RepeatTimer {
    #[inline]
    pub fn deadline(&self) -> Instant {
        self.current_deadline
    }

    #[inline]
    pub fn duration(&self) -> Duration {
        self.duration
    }

    #[inline]
    pub fn wait(duration: Duration) -> Self {
        Self {
            current_deadline: Instant::now() + duration,
            duration: duration,
        }
    }

    #[inline]
    pub fn wait_nanos(nanos: u64) -> Self {
        Self::wait(Duration::from_nanos(nanos))
    }

    #[inline]
    pub fn wait_micros(micros: u64) -> Self {
        Self::wait(Duration::from_micros(micros))
    }

    #[inline]
    pub fn wait_millis(millis: u64) -> Self {
        Self::wait(Duration::from_millis(millis))
    }

    #[inline]
    pub fn wait_secs(secs: u64) -> Self {
        Self::wait(Duration::from_secs(secs))
    }

    #[inline]
    pub fn wait_secs_f32(secs: f32) -> Self {
        Self::wait(Duration::from_secs_f32(secs))
    }

    #[inline]
    pub fn wait_secs_f64(secs: f64) -> Self {
        Self::wait(Duration::from_secs_f64(secs))
    }

    #[inline]
    pub fn wait_mins(mins: u64) -> Self {
        Self::wait_secs(mins * 60)
    }

    #[inline]
    pub fn wait_mins_f32(mins: f32) -> Self {
        Self::wait_secs_f32(mins * 60.0)
    }

    #[inline]
    pub fn wait_mins_f64(mins: f64) -> Self {
        Self::wait_secs_f64(mins * 60.0)
    }

    #[inline]
    pub fn wait_hours(hours: u64) -> Self {
        Self::wait_secs(hours * 3600)
    }

    #[inline]
    pub fn wait_hours_f32(hours: f32) -> Self {
        Self::wait_secs_f32(hours * 3600.0)
    }

    #[inline]
    pub fn wait_hours_f64(hours: f64) -> Self {
        Self::wait_secs_f64(hours * 3600.0)
    }

    #[inline]
    pub fn finished(&self) -> bool {
        Instant::now() >= self.current_deadline
    }

    #[inline]
    pub fn reset(&mut self) {
        self.current_deadline = Instant::now() + self.duration;
    }

    #[inline]
    pub fn reset_if_finished(&mut self) -> bool {
        let now = Instant::now();
        if now >= self.current_deadline {
            self.current_deadline = now + self.duration;
            true
        } else {
            false
        }
    }

    /// When the deadline is reached, this calls the given callback and then resets the timer.
    #[inline]
    pub fn on_tick<R, F: FnOnce(&Self, Instant) -> R>(&mut self, f: F) -> Option<R> {
        let now = Instant::now();
        if now >= self.current_deadline {
            let result = f(self, now);
            self.current_deadline = now + self.duration;
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn timer_test() {
        let timer = Timer::wait_secs(5);
        let mut sec_waiter = Timer::wait_secs(1);
        println!("Waiting five seconds.");
        while !timer.finished() {
            if sec_waiter.finished() {
                println!("Waiting...");
                sec_waiter = Timer::wait_secs(1);
            }
        }
        println!("Finished.");
    }
}