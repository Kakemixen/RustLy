use std::marker::PhantomData;

// TODO enable several readers on single channel
// TODO maybe be inspired by bevy and have two event buffers
// TODO if two buffers, maybe I can use refcell to enable coupling between
// readers and channel

enum EventBuffer
{
	A,
	B,
}

struct EventChannel<T>
{
	events_a: Vec<T>,
	events_b: Vec<T>,
	total_events: usize,
	readable_buffer: EventBuffer,
}

struct EventReader<T>
{
	dummy_: PhantomData<T>,
}

impl<T> EventChannel<T>
{
	fn new() -> EventChannel<T>
	{
		EventChannel {
			events_a: Vec::with_capacity(10), // maybe sensible initial?
			events_b: Vec::with_capacity(10), // maybe sensible initial?
			total_events: 0,
			readable_buffer: EventBuffer::A,
		}
	}

	fn send(&mut self, e: T)
	{
		self.total_events += 1;

		match self.readable_buffer {
			EventBuffer::A => {
				self.events_b.push(e);
			}
			EventBuffer::B => {
				self.events_a.push(e);
			}
		}
	}

	fn flush(&mut self)
	{
		match self.readable_buffer {
			EventBuffer::A => {
				self.events_a.clear();
				self.readable_buffer = EventBuffer::B;
			}
			EventBuffer::B => {
				self.events_b.clear();
				self.readable_buffer = EventBuffer::A;
			}
		}
	}

	fn get_reader(&self) -> EventReader<T>
	{
		EventReader {
			dummy_: Default::default(),
		}
	}
}

impl<T> EventReader<T>
{
	fn iter<'a>(&self, channel: &'a EventChannel<T>) -> impl Iterator<Item = &'a T>
	{
		// TODO would like to find a way to couple reader and channel
		// A naive reference member had lifetime issues with current setup
		match channel.readable_buffer {
			EventBuffer::A => channel.events_a.iter(),
			EventBuffer::B => channel.events_b.iter(),
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
		let mut test_channel = EventChannel::<TestEvent>::new();
		let event0 = TestEvent { data: 4 };
		let event1 = TestEvent { data: 2 };

		let reader = test_channel.get_reader();
		let events = reader.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(events, Vec::<&TestEvent>::new());

		test_channel.send(event0);
		test_channel.send(event1);
		test_channel.flush();

		let events = reader.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(events, [&TestEvent { data: 4 }, &TestEvent { data: 2 }]);
		let reader2 = test_channel.get_reader();

		let events = reader2.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(events, [&TestEvent { data: 4 }, &TestEvent { data: 2 }]);

		test_channel.flush();
		let events = reader2.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(events, Vec::<&TestEvent>::new());
	}
}
