use std::cell::{Ref, RefCell};
use std::marker::PhantomData;

// TODO if two buffers, maybe I can use refcell to enable coupling between
// readers and channel
// I cannot return an iterator of a RefCell<Vec<T>>, so perhaps that's not the
// way

#[derive(Debug)]
enum EventBuffer
{
	A,
	B,
}

pub struct EventChannel<T>
{
	events_a: RefCell<Vec<T>>,
	events_b: RefCell<Vec<T>>,
	start_idx_a: RefCell<usize>,
	start_idx_b: RefCell<usize>,
	readable_buffer: RefCell<EventBuffer>,
}

pub struct EventReader<T>
{
	read_events: RefCell<usize>,
	dummy_: PhantomData<T>,
}

impl<T> EventChannel<T>
{
	pub fn new() -> EventChannel<T>
	{
		EventChannel {
			events_a: RefCell::new(Vec::new()),
			events_b: RefCell::new(Vec::new()),
			start_idx_a: RefCell::new(0),
			start_idx_b: RefCell::new(0),
			readable_buffer: RefCell::new(EventBuffer::A),
		}
	}

	pub fn send(&self, e: T)
	{
		match *self.readable_buffer.borrow() {
			EventBuffer::A => {
				(*self.events_b.borrow_mut()).push(e);
				(*self.start_idx_b.borrow_mut()) += 1;
			}
			EventBuffer::B => {
				(*self.events_a.borrow_mut()).push(e);
				(*self.start_idx_a.borrow_mut()) += 1;
			}
		}
	}

	pub fn flush(&self)
	{
		let mut readable_buffer = self.readable_buffer.borrow_mut();
		match *readable_buffer {
			EventBuffer::A => {
				(*self.events_a.borrow_mut()).clear();
				*readable_buffer = EventBuffer::B;

				(*self.start_idx_a.borrow_mut()) = *self.start_idx_b.borrow();
			}
			EventBuffer::B => {
				(*self.events_b.borrow_mut()).clear();
				*readable_buffer = EventBuffer::A;

				(*self.start_idx_b.borrow_mut()) = *self.start_idx_a.borrow();
			}
		}
	}

	pub fn get_reader(&self) -> EventReader<T>
	{
		EventReader {
			read_events: RefCell::new(0),
			dummy_: Default::default(),
		}
	}
}

impl<T> EventReader<T>
{
	pub fn iter<'a>(&self, channel: &'a EventChannel<T>) -> Iter<'a, T>
	{
		// TODO would like to find a way to couple reader and channel
		// A naive reference member had lifetime issues with current setup
		let mut read_events = self.read_events.borrow_mut();
		match *channel.readable_buffer.borrow() {
			EventBuffer::A => {
				if *read_events > *channel.start_idx_a.borrow() {
					Iter { inner: None }
				}
				else {
					*read_events = *channel.start_idx_a.borrow() + 1;
					Iter {
						inner: Some(Ref::map(channel.events_a.borrow(), |v| &v[..])),
					}
				}
			}
			EventBuffer::B => {
				if *read_events > *channel.start_idx_b.borrow() {
					Iter { inner: None }
				}
				else {
					*read_events = *channel.start_idx_b.borrow() + 1;
					Iter {
						inner: Some(Ref::map(channel.events_b.borrow(), |v| &v[..])),
					}
				}
			}
		}
	}
}

// thanks kwarrick
// https://stackoverflow.com/questions/33541492/returning-iterator-of-a-vec-in-a-refcell
pub struct Iter<'a, T>
{
	inner: Option<Ref<'a, [T]>>,
}

impl<'a, T> Iterator for Iter<'a, T>
{
	type Item = Ref<'a, T>;

	fn next(&mut self) -> Option<Self::Item>
	{
		match self.inner.take() {
			Some(borrow) => match *borrow {
				[] => None,
				[_, ..] => {
					let (head, tail) = Ref::map_split(borrow, |slice| (&slice[0], &slice[1..]));
					self.inner.replace(tail);
					Some(head)
				}
			},
			None => None,
		}
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[derive(Clone, Debug, PartialEq, Eq)]
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
		let events = reader
			.iter(&test_channel)
			.map(|x| x.clone())
			.collect::<Vec<TestEvent>>();
		assert_eq!(events, Vec::<TestEvent>::new(), "initial events empty");

		test_channel.send(event0);
		test_channel.flush();
		let events = reader
			.iter(&test_channel)
			.map(|x| x.clone())
			.collect::<Vec<TestEvent>>();
		assert_eq!(
			events,
			[TestEvent { data: 0 }],
			"reader can read flushed event0"
		);

		test_channel.send(event1);
		test_channel.flush();

		let events = reader
			.iter(&test_channel)
			.map(|x| x.clone())
			.collect::<Vec<TestEvent>>();
		assert_eq!(
			events,
			[TestEvent { data: 1 }],
			"We only retain the events most recently flushed, event0 is then dropped"
		);

		let reader2 = test_channel.get_reader();
		let events = reader2
			.iter(&test_channel)
			.map(|x| x.clone())
			.collect::<Vec<TestEvent>>();
		assert_eq!(
			events,
			[TestEvent { data: 1 }],
			"We only retain the events most recently flushed, reader2 reads after event0 has been \
			 dropped"
		);
		let events = reader2
			.iter(&test_channel)
			.map(|x| x.clone())
			.collect::<Vec<TestEvent>>();
		assert_eq!(
			events,
			Vec::<TestEvent>::new(),
			"Cannot read event multiple times"
		);

		test_channel.flush();
		let events = reader2
			.iter(&test_channel)
			.map(|x| x.clone())
			.collect::<Vec<TestEvent>>();
		assert_eq!(events, Vec::<TestEvent>::new());
	}
}
