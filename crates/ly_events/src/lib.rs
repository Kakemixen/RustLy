use std::marker::PhantomData;

// TODO enable several readers on single channel
// TODO maybe be inspired by bevy and have two event buffers
// TODO if two buffers, maybe I can use refcell to enable coupling between
// readers and channel

struct EventChannel<T>
{
	events: Vec<T>,
}

struct EventReader<T>
{
	dummy_: PhantomData<T>,
}

impl<T> EventChannel<T>
{
	fn new() -> EventChannel<T> { EventChannel { events: Vec::new() } }

	fn send(&mut self, e: T) { self.events.push(e); }

	fn flush(&mut self) { self.events.clear(); }

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
		channel.events.iter()
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
		let events = reader.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(events, [&TestEvent { data: 4 }, &TestEvent { data: 2 }]);
	}
}
