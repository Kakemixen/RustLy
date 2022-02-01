use std::cell::UnsafeCell;

// TODO add channel flush mutex

#[derive(Debug)]
enum EventBuffer
{
	A,
	B,
}

pub struct EventChannel<T>
{
	events_a: UnsafeCell<Vec<T>>,
	events_b: UnsafeCell<Vec<T>>,
	start_idx_a: UnsafeCell<usize>,
	start_idx_b: UnsafeCell<usize>,
	readable_buffer: UnsafeCell<EventBuffer>,
}

pub struct EventReader<'a, T>
{
	read_events: UnsafeCell<usize>,
	channel: &'a EventChannel<T>,
}

impl<T> EventChannel<T>
{
	pub fn new() -> EventChannel<T>
	{
		EventChannel {
			events_a: UnsafeCell::new(Vec::new()), // maybe sensible initial?
			events_b: UnsafeCell::new(Vec::new()), // maybe sensible initial?
			start_idx_a: UnsafeCell::new(0),
			start_idx_b: UnsafeCell::new(0),
			readable_buffer: UnsafeCell::new(EventBuffer::A),
		}
	}

	pub fn send(&self, e: T)
	{
		unsafe {
			match *self.readable_buffer.get() {
				EventBuffer::A => {
					(*self.events_b.get()).push(e);
					(*self.start_idx_b.get()) += 1;
				}
				EventBuffer::B => {
					(*self.events_a.get()).push(e);
					(*self.start_idx_a.get()) += 1;
				}
			}
		}
	}

	pub fn flush(&self)
	{
		let readable_buffer = self.readable_buffer.get();
		unsafe {
			match *readable_buffer {
				EventBuffer::A => {
					(*self.events_a.get()).clear();
					*readable_buffer = EventBuffer::B;

					*self.start_idx_a.get() = *self.start_idx_b.get() // so that reading starts counting properly
				}
				EventBuffer::B => {
					(*self.events_b.get()).clear();
					*readable_buffer = EventBuffer::A;

					*self.start_idx_b.get() = *self.start_idx_a.get()
				}
			}
		}
	}

	pub fn get_reader(&self) -> EventReader<T>
	{
		EventReader {
			read_events: UnsafeCell::new(0),
			channel: self,
		}
	}
}

impl<'a, T> EventReader<'a, T>
{
	pub fn iter(&self) -> impl Iterator<Item = &'a T>
	{
		unsafe {
			// TODO should have a flush mutex
			let readable_buffer = self.channel.readable_buffer.get();
			let read_events = self.read_events.get();
			match *readable_buffer {
				EventBuffer::A => {
					let start_idx_a = *self.channel.start_idx_a.get();
					if *read_events > start_idx_a {
						[].iter()
					}
					else {
						*read_events = start_idx_a + 1;
						(*self.channel.events_a.get()).iter()
					}
				}
				EventBuffer::B => {
					let start_idx_b = *self.channel.start_idx_b.get();
					if *read_events > start_idx_b {
						[].iter()
					}
					else {
						*read_events = start_idx_b + 1;
						(*self.channel.events_b.get()).iter()
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[derive(Debug, PartialEq, Eq)]
	struct TestEvent
	{
		data: usize,
	}

	#[test]
	fn flow()
	{
		let test_channel = EventChannel::<TestEvent>::new();
		let event0 = TestEvent { data: 0 };
		let event1 = TestEvent { data: 1 };

		let reader = test_channel.get_reader();
		let events = reader.iter().collect::<Vec<&TestEvent>>();
		assert_eq!(events, Vec::<&TestEvent>::new(), "initial events empty");

		test_channel.send(event0);
		test_channel.flush();

		let events = reader.iter().collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			[&TestEvent { data: 0 }],
			"reader can read flushed event0"
		);

		test_channel.send(event1);
		test_channel.flush();

		let events = reader.iter().collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			[&TestEvent { data: 1 }],
			"We only retain the events most recently flushed, event0 is then dropped"
		);

		let reader2 = test_channel.get_reader();
		let events = reader2.iter().collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			[&TestEvent { data: 1 }],
			"We only retain the events most recently flushed, reader2 reads after event0 has been \
			 dropped"
		);
		let events = reader2.iter().collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			Vec::<&TestEvent>::new(),
			"Cannot read event multiple times"
		);

		test_channel.flush();
		let events = reader2.iter().collect::<Vec<&TestEvent>>();
		assert_eq!(events, Vec::<&TestEvent>::new());
	}
}
