use std::time::Instant;

pub struct Timer {
    start: Instant,
    label: String,
}

impl Timer {
    pub fn new(label: String) -> Self {
        Self {
            start: Instant::now(),
            label,
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        println!("[{}] {:.2?}", self.label, self.start.elapsed())
    }
}
