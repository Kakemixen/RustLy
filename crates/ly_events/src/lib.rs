use std::cell::UnsafeCell;
use std::sync::Weak;
//use std::sync::{Arc, Mutex};
use parking_lot::Mutex;
use std::sync::Arc;

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

pub struct SyncEventChannel<T>
{
	channel: Arc<Mutex<EventChannel<T>>>,
}

pub struct SyncEventReader<T>
{
	read_events: UnsafeCell<usize>,
	channel: Weak<Mutex<EventChannel<T>>>,
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

	pub fn flush_channel(&self) { self.channel.flush(); }
}

impl<T> SyncEventChannel<T>
{
	pub fn new() -> SyncEventChannel<T>
	{
		SyncEventChannel {
			channel: Arc::new(Mutex::new(EventChannel {
				events_a: UnsafeCell::new(Vec::new()), // maybe sensible initial?
				events_b: UnsafeCell::new(Vec::new()), // maybe sensible initial?
				start_idx_a: UnsafeCell::new(0),
				start_idx_b: UnsafeCell::new(0),
				readable_buffer: UnsafeCell::new(EventBuffer::A),
			})),
		}
	}

	pub fn send(&self, e: T) { self.channel.lock().send(e); }

	pub fn flush(&self) { self.channel.lock().flush(); }

	pub fn get_reader(&self) -> SyncEventReader<T>
	{
		SyncEventReader {
			read_events: UnsafeCell::new(0),
			channel: Arc::downgrade(&self.channel),
		}
	}
}

impl<T> SyncEventReader<T>
{
	pub fn iter<'a>(&self) -> impl Iterator<Item = &'a T>
	where
		T: 'a,
	{
		let channel = self.channel.upgrade();
		match channel {
			None => [].iter(),
			Some(channel) => {
				let channel = channel.lock();
				unsafe {
					// TODO should have a flush mutex
					let readable_buffer = channel.readable_buffer.get();
					let read_events = self.read_events.get();
					match *readable_buffer {
						EventBuffer::A => {
							let start_idx_a = *channel.start_idx_a.get();
							if *read_events > start_idx_a {
								[].iter()
							}
							else {
								*read_events = start_idx_a + 1;
								(*channel.events_a.get()).iter()
							}
						}
						EventBuffer::B => {
							let start_idx_b = *channel.start_idx_b.get();
							if *read_events > start_idx_b {
								[].iter()
							}
							else {
								*read_events = start_idx_b + 1;
								(*channel.events_b.get()).iter()
							}
						}
					}
				}
			}
		}
	}

	pub fn flush_channel(&self) -> Result<(), String>
	{
		match self.channel.upgrade() {
			None => Err("channel has been dropped".to_string()),
			Some(channel) => {
				channel.lock().flush();
				Ok(())
			}
		}
	}
}

#[cfg(test)]
mod tests
{
	use super::*;
	use std::ops::AddAssign;
	use std::thread;

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

	#[test]
	fn sync_basic()
	{
		let channel = SyncEventChannel::<TestEvent>::new();
		let total = Arc::new(Mutex::new(0));
		let total_loc = Arc::clone(&total);

		let rec = channel.get_reader();
		let emitter1 = thread::spawn(move || {
			for i in 0..10 {
				let event = TestEvent { data: 2 * i };
				channel.send(event);
				thread::sleep_ms(1);
			}
		});

		let rec1 = thread::spawn(move || {
			loop {
				thread::sleep_ms(5);
				let mut got_events = false;
				if let Err(_) = rec.flush_channel() {
					break;
				}
				for e in rec.iter() {
					got_events = true;
					total.lock().add_assign(e.data);
				}

				if !got_events {
					break;
				}
			}
		});

		emitter1.join().unwrap();
		rec1.join().unwrap();
		let total = total_loc.lock();
		assert!(total.eq(&90));
	}
}
