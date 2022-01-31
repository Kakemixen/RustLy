use std::marker::PhantomData;

// TODO if two buffers, maybe I can use refcell to enable coupling between
// readers and channel

#[derive(Debug)]
enum EventBuffer
{
	A,
	B,
}

struct EventChannel<T>
{
	events_a: Vec<T>,
	events_b: Vec<T>,
	start_idx_a: usize,
	start_idx_b: usize,
	readable_buffer: EventBuffer,
}

struct EventReader<T>
{
	read_events: usize,
	dummy_: PhantomData<T>,
}

impl<T> EventChannel<T>
{
	fn new() -> EventChannel<T>
	{
		EventChannel {
			events_a: Vec::with_capacity(10), // maybe sensible initial?
			events_b: Vec::with_capacity(10), // maybe sensible initial?
			start_idx_a: 0,
			start_idx_b: 0,
			readable_buffer: EventBuffer::A,
		}
	}

	fn send(&mut self, e: T)
	{
		match self.readable_buffer {
			EventBuffer::A => {
				self.events_b.push(e);
				self.start_idx_b += 1;
			}
			EventBuffer::B => {
				self.events_a.push(e);
				self.start_idx_a += 1;
			}
		}
		println!(
			"send: channel start idx a:{} b:{}",
			self.start_idx_a, self.start_idx_b
		);
		println!("send: channel readable_buffer {:?}", self.readable_buffer);
	}

	fn flush(&mut self)
	{
		match self.readable_buffer {
			EventBuffer::A => {
				self.events_a.clear();
				self.readable_buffer = EventBuffer::B;

				self.start_idx_a = self.start_idx_b // so that reading starts counting properly
			}
			EventBuffer::B => {
				self.events_b.clear();
				self.readable_buffer = EventBuffer::A;

				self.start_idx_b = self.start_idx_a
			}
		}
		println!(
			"flush: channel start idx a:{} b:{}",
			self.start_idx_a, self.start_idx_b
		);
		println!("flush: channel readable_buffer {:?}", self.readable_buffer);
	}

	fn get_reader(&self) -> EventReader<T>
	{
		EventReader {
			read_events: 0,
			dummy_: Default::default(),
		}
	}
}

impl<T> EventReader<T>
{
	fn iter<'a>(&mut self, channel: &'a EventChannel<T>) -> impl Iterator<Item = &'a T>
	{
		// TODO would like to find a way to couple reader and channel
		// A naive reference member had lifetime issues with current setup
		println!("reader read events {}", self.read_events);
		println!(
			"channel start idx a:{} b:{}",
			channel.start_idx_a, channel.start_idx_b
		);
		println!("channel readable_buffer {:?}", channel.readable_buffer);
		match channel.readable_buffer {
			EventBuffer::A => {
				if self.read_events > channel.start_idx_a {
					[].iter()
				}
				else {
					self.read_events = channel.start_idx_a + 1;
					channel.events_a.iter()
				}
			}
			EventBuffer::B => {
				if self.read_events > channel.start_idx_b {
					[].iter()
				}
				else {
					self.read_events = channel.start_idx_b + 1;
					channel.events_b.iter()
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
		let mut test_channel = EventChannel::<TestEvent>::new();
		let event0 = TestEvent { data: 0 };
		let event1 = TestEvent { data: 1 };

		let mut reader = test_channel.get_reader();
		let events = reader.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(events, Vec::<&TestEvent>::new(), "initial events empty");

		test_channel.send(event0);
		test_channel.flush();
		let events = reader.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			[&TestEvent { data: 0 }],
			"reader can read flushed event0"
		);

		test_channel.send(event1);
		test_channel.flush();

		let events = reader.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			[&TestEvent { data: 1 }],
			"We only retain the events most recently flushed, event0 is then dropped"
		);

		let mut reader2 = test_channel.get_reader();
		let events = reader2.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			[&TestEvent { data: 1 }],
			"We only retain the events most recently flushed, reader2 reads after event0 has been \
			 dropped"
		);
		let events = reader2.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			Vec::<&TestEvent>::new(),
			"Cannot read event multiple times"
		);

		test_channel.flush();
		let events = reader2.iter(&test_channel).collect::<Vec<&TestEvent>>();
		assert_eq!(events, Vec::<&TestEvent>::new());
	}
}
