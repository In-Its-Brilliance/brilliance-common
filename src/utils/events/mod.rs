pub mod event_channel;
pub mod event_broadcast;

pub trait EventInterface<T> {
    type Reader<'a>
    where
        Self: 'a;
    fn emit_event(&self, event: T);
    fn get_reader(&mut self) -> Self::Reader<'_>;
}

pub trait EventReader<T> {
    type Iter<'a>: Iterator<Item = T>
    where
        Self: 'a,
        T: 'a;
    fn iter_events(&self) -> Self::Iter<'_>;
}
