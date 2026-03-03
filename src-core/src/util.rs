#[cfg(debug_assertions)]
use std::time::Instant;

#[cfg(debug_assertions)]
pub struct Timer {
    start: Instant,
    label: String,
}

#[cfg(debug_assertions)]
impl Timer {
    pub fn new(label: String) -> Self {
        Self {
            start: Instant::now(),
            label,
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for Timer {
    fn drop(&mut self) {
        println!("[{}] {:.2?}", self.label, self.start.elapsed())
    }
}

#[cfg(not(debug_assertions))]
pub struct Timer;

#[cfg(not(debug_assertions))]
impl Timer {
    pub fn new(_label: String) -> Self {
        Self
    }
}
