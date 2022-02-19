use core::marker::PhantomData;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use std::cell::UnsafeCell;
use std::slice::Iter;

#[derive(Debug)]
enum ReadableEventBuffer
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
	readable_buffer: UnsafeCell<ReadableEventBuffer>,
}

pub struct EventReader<'a, T>
{
	read_events: UnsafeCell<usize>,
	channel: &'a EventChannel<T>,
}

pub struct SyncEventChannel<T>
{
	channel: EventChannel<T>,
	write_mutex: Mutex<()>,
	flush_mutex: RwLock<()>,
}

pub struct SyncEventReader<'a, T>
{
	read_events: UnsafeCell<usize>,
	channel: &'a SyncEventChannel<T>,
	_not_send_sync: PhantomData<*const ()>, // to explicitly say it cannot be sent
}

unsafe impl<T> Sync for SyncEventChannel<T> {}

impl<T> EventChannel<T>
{
	pub fn new() -> EventChannel<T>
	{
		EventChannel {
			events_a: UnsafeCell::new(Vec::new()), // maybe sensible initial?
			events_b: UnsafeCell::new(Vec::new()), // maybe sensible initial?
			start_idx_a: UnsafeCell::new(0),
			start_idx_b: UnsafeCell::new(0),
			readable_buffer: UnsafeCell::new(ReadableEventBuffer::A),
		}
	}

	pub fn send(&self, e: T)
	{
		unsafe {
			match *self.readable_buffer.get() {
				ReadableEventBuffer::A => {
					(*self.events_b.get()).push(e);
					(*self.start_idx_b.get()) += 1;
				}
				ReadableEventBuffer::B => {
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
				ReadableEventBuffer::A => {
					(*self.events_a.get()).clear();
					*readable_buffer = ReadableEventBuffer::B;

					*self.start_idx_a.get() = *self.start_idx_b.get() // so that reading starts counting properly
				}
				ReadableEventBuffer::B => {
					(*self.events_b.get()).clear();
					*readable_buffer = ReadableEventBuffer::A;

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
	pub fn iter(&self) -> impl Iterator<Item = &T>
	{
		unsafe {
			let readable_buffer = self.channel.readable_buffer.get();
			let read_events = self.read_events.get();
			match *readable_buffer {
				ReadableEventBuffer::A => {
					let start_idx_a = *self.channel.start_idx_a.get();
					if *read_events > start_idx_a {
						[].iter()
					}
					else {
						*read_events = start_idx_a + 1;
						(*self.channel.events_a.get()).iter()
					}
				}
				ReadableEventBuffer::B => {
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
			channel: EventChannel::new(),
			write_mutex: Mutex::new(()),
			flush_mutex: RwLock::new(()),
		}
	}

	pub fn send(&self, e: T)
	{
		let lock = self.write_mutex.lock();
		self.channel.send(e);
	}

	pub fn flush(&self)
	{
		let lock = self.flush_mutex.write();
		self.channel.flush();
	}

	pub fn get_reader(&self) -> SyncEventReader<T>
	{
		SyncEventReader {
			read_events: UnsafeCell::new(0),
			channel: self,
			_not_send_sync: PhantomData,
		}
	}
}

impl<'a, T> SyncEventReader<'a, T>
{
	pub fn iter(&self) -> impl Iterator<Item = &T>
	{
		let read_lock = self.channel.flush_mutex.read();
		let channel = &self.channel.channel;
		unsafe {
			let readable_buffer = channel.readable_buffer.get();
			let read_events = self.read_events.get();
			match *readable_buffer {
				ReadableEventBuffer::A => {
					let start_idx_a = *channel.start_idx_a.get();
					if *read_events > start_idx_a {
						EventIterator {
							read_lock,
							iterator: [].iter(),
						}
					}
					else {
						*read_events = start_idx_a + 1;
						let iterator = (*channel.events_a.get()).iter();
						EventIterator {
							read_lock,
							iterator,
						}
					}
				}
				ReadableEventBuffer::B => {
					let start_idx_b = *channel.start_idx_b.get();
					if *read_events > start_idx_b {
						EventIterator {
							read_lock,
							iterator: [].iter(),
						}
					}
					else {
						*read_events = start_idx_b + 1;
						let iterator = (*channel.events_b.get()).iter();
						EventIterator {
							read_lock,
							iterator,
						}
					}
				}
			}
		}
	}

	pub fn flush_channel(&self) { self.channel.flush(); }
}

struct EventIterator<'a, T>
{
	read_lock: RwLockReadGuard<'a, ()>,
	iterator: Iter<'a, T>,
}

impl<'a, T> Iterator for EventIterator<'a, T>
{
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> { self.iterator.next() }
}

#[cfg(test)]
mod tests
{
	use super::*;
	use std::ops::AddAssign;
	use std::sync::Arc;
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
		let channel = Arc::new(SyncEventChannel::<TestEvent>::new());
		let total = Arc::new(Mutex::new(0));
		let total_loc = Arc::clone(&total);
		let c = Arc::clone(&channel);
		let emitter1 = thread::spawn(move || {
			for i in 0..10 {
				let event = TestEvent { data: 2 * i };
				c.send(event);
				thread::sleep_ms(1);
			}
		});

		let rec1 = thread::spawn(move || {
			let rec = channel.get_reader();
			loop {
				thread::sleep_ms(5);
				let mut got_events = false;
				rec.flush_channel();
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
