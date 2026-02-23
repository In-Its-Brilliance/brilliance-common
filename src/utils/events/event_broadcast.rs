use super::{EventInterface, EventReader};
use std::cell::RefCell;

pub struct EventBroadcast<T> {
    tx: tokio::sync::broadcast::Sender<T>,
}

impl<T: Clone> EventBroadcast<T> {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(capacity);
        Self { tx }
    }
}

pub struct BroadcastReader<T> {
    rx: RefCell<tokio::sync::broadcast::Receiver<T>>,
}

impl<T: Clone> EventInterface<T> for EventBroadcast<T> {
    type Reader<'a>
        = BroadcastReader<T>
    where
        Self: 'a;

    fn emit_event(&self, event: T) {
        let _ = self.tx.send(event);
    }

    fn get_reader(&mut self) -> Self::Reader<'_> {
        BroadcastReader {
            rx: RefCell::new(self.tx.subscribe()),
        }
    }
}

impl<T: Clone> EventReader<T> for BroadcastReader<T> {
    type Iter<'a>
        = std::vec::IntoIter<T>
    where
        Self: 'a,
        T: 'a;

    fn iter_events(&self) -> Self::Iter<'_> {
        // теперь &self
        let mut out = Vec::new();
        let mut rx = self.rx.borrow_mut();
        loop {
            match rx.try_recv() {
                Ok(v) => out.push(v),
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => continue,
            }
        }
        out.into_iter()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn broadcast_two_consumers() {
        let mut bus = EventBroadcast::<i32>::new(8);

        let r1 = bus.get_reader();
        let r2 = bus.get_reader();

        bus.emit_event(10);
        bus.emit_event(20);

        let v1: Vec<_> = r1.iter_events().collect();
        let v2: Vec<_> = r2.iter_events().collect();

        assert_eq!(v1, vec![10, 20]);
        assert_eq!(v2, vec![10, 20]);
    }
}
