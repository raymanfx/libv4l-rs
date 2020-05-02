use crate::v4l_sys::*;
use std::{fmt, mem, time};

#[derive(Debug, Default, Clone, Copy)]
/// Timestamp consisting of a seconds and a microseconds component
pub struct Timestamp {
    pub sec: i64,
    pub usec: i64,
}

impl Timestamp {
    /// Returns a timestamp representation
    ///
    /// # Arguments
    ///
    /// * `sec` - Seconds
    /// * `usec` - Microseconds
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::Timestamp;
    /// let ts = Timestamp::new(5, 5);
    /// ```
    pub fn new(sec: i64, usec: i64) -> Self {
        Timestamp { sec, usec }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let floating: f64 = self.sec as f64 + self.usec as f64 / 1_000_000.0;
        write!(f, "{} [s]", floating)
    }
}

impl From<timeval> for Timestamp {
    fn from(tv: timeval) -> Self {
        Timestamp {
            sec: tv.tv_sec,
            usec: tv.tv_usec,
        }
    }
}

impl Into<timeval> for Timestamp {
    fn into(self: Timestamp) -> timeval {
        let mut tv: timeval;
        unsafe {
            tv = mem::zeroed();
        }

        tv.tv_sec = self.sec;
        tv.tv_usec = self.usec;
        tv
    }
}

impl From<time::Duration> for Timestamp {
    fn from(duration: time::Duration) -> Self {
        Timestamp::new(duration.as_secs() as i64, duration.as_micros() as i64)
    }
}

impl From<Timestamp> for time::Duration {
    fn from(ts: Timestamp) -> Self {
        time::Duration::new(ts.sec as u64, (ts.usec / 1000) as u32)
    }
}
