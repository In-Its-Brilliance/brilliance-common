use flume;

use super::{EventInterface, EventReader};

/// Single-producer multi-consumer event channel.
///
/// Messages are distributed among readers — each message is received by only one reader.
/// If you need all readers to receive all messages, use `EventBroadcast` instead.
///
/// # Example
///
/// ```
/// use crate::utils::events::event_channel::EventChannel;
/// use crate::utils::events::{EventInterface, EventReader};
///
/// let mut channel = EventChannel::<String>::default();
/// let reader = channel.get_reader();
///
/// channel.emit_event("hello".to_string());
/// channel.emit_event("world".to_string());
///
/// let events: Vec<_> = reader.iter_events().collect();
/// assert_eq!(events, vec!["hello", "world"]);
/// ```
pub struct EventChannel<T> {
    tx: flume::Sender<T>,
    rx: flume::Receiver<T>,
}

impl<T> Default for EventChannel<T> {
    fn default() -> Self {
        let (tx, rx) = flume::unbounded();
        Self { tx, rx }
    }
}

pub struct ChannelReader<T> {
    rx: flume::Receiver<T>,
}

impl<T> EventInterface<T> for EventChannel<T> {
    type Reader<'a> = ChannelReader<T> where Self: 'a;

    fn emit_event(&self, event: T) {
        let _ = self.tx.send(event);
    }

    fn get_reader(&mut self) -> Self::Reader<'_> {
        ChannelReader { rx: self.rx.clone() }
    }
}

impl<T> EventReader<T> for ChannelReader<T> {
    type Iter<'a> = flume::Drain<'a, T> where Self: 'a, T: 'a;

    fn iter_events(&self) -> Self::Iter<'_> {
        self.rx.drain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_single_consumer() {
        let mut ch = EventChannel::<i32>::default();
        let r = ch.get_reader();
        
        ch.emit_event(1);
        ch.emit_event(2);

        let v: Vec<_> = r.iter_events().collect();
        assert_eq!(v, vec![1, 2]);

        ch.emit_event(3);
        let v2: Vec<_> = r.iter_events().collect();
        assert_eq!(v2, vec![3]);
    }

    #[test]
    fn multiple_readers() {
        let mut ch = EventChannel::<i32>::default();
        let r1 = ch.get_reader();
        let r2 = ch.get_reader();
        
        ch.emit_event(1);

        // Только один получит — это MPMC канал, first come first serve
        let v1: Vec<_> = r1.iter_events().collect();
        let v2: Vec<_> = r2.iter_events().collect();
        assert_eq!(v1.len() + v2.len(), 1);
    }
}
