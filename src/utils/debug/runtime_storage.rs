use std::collections::HashMap;
use std::time::Duration;

pub type Entry = (Duration, u32, Duration); // total, count, last
pub type SpansType = HashMap<&'static str, Entry>;

pub struct RuntimeStorage {
    spans: SpansType,
}

impl RuntimeStorage {
    pub fn new() -> Self {
        Self {
            spans: HashMap::new(),
        }
    }

    pub fn push(&mut self, name: &'static str, elapsed: Duration) {
        let entry = self.spans.entry(name).or_insert((Duration::ZERO, 0, Duration::ZERO));
        entry.0 += elapsed;
        entry.1 += 1;
        entry.2 = elapsed;
    }

    pub fn get_spans(&self) -> &SpansType {
        &self.spans
    }

    pub fn clear(&mut self) {
        self.spans.clear();
    }
}
