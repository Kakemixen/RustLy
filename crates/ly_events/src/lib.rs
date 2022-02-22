//! Event system in the LY engine
//!
//! The crate provides functionality to send event via "channels", inspired
//! by rust channels. A `Sync` alternative is provided, but it is about 4 times
//! slower (still OK fast).
//!
//! Event channels are instantiated for some event-type, and are read
//! by readers that are created by those instantiated channels.
//! In order to read events, they first need to be flushed.
//! Once read, events are not read again.
//!
//! A reader has a borrow of the channel it reads, the one that created the
//! reader. This is to ensure that the reader always has a channel to read from,
//! and to enable a more ergonomic API.
//!
//! A single channel may have multiple readers. Reading an event
//! does not consume it for other readers.
//!
//! ### Example event flow
//!
//! ```
//! # use ly_events::EventChannel;
//! #[derive(Debug, PartialEq, Eq, Clone)]
//! struct TestEvent { data: usize, }
//!
//! let test_channel = EventChannel::<TestEvent>::new();
//! let event = TestEvent { data: 42 };
//! let event_clone = event.clone();
//!
//! let reader = test_channel.get_reader();
//!
//! let events = reader.read().collect::<Vec<&TestEvent>>();
//! assert_eq!(events, Vec::<&TestEvent>::new(),
//! 	"initial events empty");
//!
//! test_channel.send(event);
//!
//! let events = reader.read().collect::<Vec<&TestEvent>>();
//! assert_eq!(events, Vec::<&TestEvent>::new(),
//! 	"still emply after send");
//!
//! test_channel.flush();
//!
//! let events = reader.read().collect::<Vec<&TestEvent>>();
//! assert_eq!(events, [&event_clone],
//! 	"reader can read flushed event");
//!
//! let events = reader.read().collect::<Vec<&TestEvent>>();
//! assert_eq!(events, Vec::<&TestEvent>::new(),
//! 	"cannot read twice");
//! ```
//!
//! ### Sync notes
//!
//! The [`SyncEventChannel`] is provided as [`Sync`], but it still placed on the
//! stack unless wrapped in `Arc` or something similar. This is important for
//! having the reader and emitting channel in different threads, keep in mind
//! the reader has a borrow to the channel.
//!
//! The [`SyncEventReader`] has some synchronization methods to wait for new
//! events, [`SyncEventReader::wait_new`], and to wait for someone else to flush
//! the channel [`SyncEventReader::wait_flushed`].
//!
//! For unscoped threads this is one way to work, note the `Arc` and
//! `wait_new()` ```
//! # struct TestEvent { data: usize, }
//! # use std::thread;
//! # use std::sync::Arc;
//! # use ly_events::SyncEventChannel;
//! let channel = Arc::new(SyncEventChannel::<TestEvent>::new());
//!
//! let c = Arc::clone(&channel);
//! let rec1 = thread::spawn(move || {
//! 	let reader = c.get_reader();
//! 	loop {
//! 		reader.wait_new();
//! 		reader.flush_channel();
//! 		for event in reader.read() {
//! 			// do stuff
//! 		}
//! 	}
//! });
//! ```

use core::marker::PhantomData;
use crossbeam::sync::Unparker;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use std::cell::UnsafeCell;
use std::slice::Iter;

mod event_signal;
pub use event_signal::SignalEvent;

#[derive(Debug)]
enum ReadableEventBuffer
{
	A,
	B,
}

/// Single-threaded event channel
pub struct EventChannel<T>
{
	events_a: UnsafeCell<Vec<T>>,
	events_b: UnsafeCell<Vec<T>>,
	start_idx_a: UnsafeCell<usize>,
	start_idx_b: UnsafeCell<usize>,
	readable_buffer: UnsafeCell<ReadableEventBuffer>,
}

/// Single-threaded event reader
///
/// Created by [`EventChannel::get_reader`].
/// Borrows the channel immutably upon creation.
pub struct EventReader<'a, T>
{
	read_events: UnsafeCell<usize>,
	channel: &'a EventChannel<T>,
}

/// Thread-safe event channel
pub struct SyncEventChannel<T>
{
	channel: EventChannel<T>,
	write_mutex: Mutex<()>,
	flush_mutex: RwLock<()>,
	new_event_waiters: UnsafeCell<Vec<Unparker>>,
	flushed_waiters: UnsafeCell<Vec<Unparker>>,
}

/// Thread-safe event reader
///
/// Created by [`SyncEventChannel::get_reader`].
/// Borrows the channel immutably upon creation.
pub struct SyncEventReader<'a, T>
{
	read_events: UnsafeCell<usize>,
	channel: &'a SyncEventChannel<T>,
	_not_send_sync: PhantomData<*const ()>, // to explicitly say it cannot be sent
}

unsafe impl<T> Sync for SyncEventChannel<T> {}

impl<T> EventChannel<T>
{
	/// Creates empty event channel
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

	/// Sends the event on the channel
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

	/// Flushes events on the channel
	///
	/// Makes the currently sent un-flushed events readable.
	///
	/// This drops all previously flushed events, making them unreadable.
	///
	/// Is is adviced to let one of the readers initiate the flush with
	/// [`EventReader::flush_channel`],
	/// as they are controlling consumation of events.
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

	/// Creates a reader for this channel
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
	/// Reads all unread events from this channel
	///
	/// Giver an `Iterator` over the currently flushed events.
	///
	/// Becaus of how this is setup, it reads all flushed events, or none at all
	/// if the flushed events have been read by this reader.
	pub fn read(&self) -> impl Iterator<Item = &T>
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

	/// Initiates a flush on the reader's connected channel
	///
	/// It is adviced to use this for flushing. Read [`EventChannel::flush`]
	/// for a descripion of the behaviour regarding flushing.
	pub fn flush_channel(&self) { self.channel.flush(); }
}

impl<T> SyncEventChannel<T>
{
	/// Creates a new thread-safe event channel
	pub fn new() -> SyncEventChannel<T>
	{
		SyncEventChannel {
			channel: EventChannel::new(),
			write_mutex: Mutex::new(()),
			flush_mutex: RwLock::new(()),
			new_event_waiters: UnsafeCell::new(Vec::new()),
			flushed_waiters: UnsafeCell::new(Vec::new()),
		}
	}

	/// Sends the event to the channel
	///
	/// This also wakes any threads waiting for new events via
	/// [`SyncEventReader::wait_new`].
	pub fn send(&self, e: T)
	{
		let _lock = self.write_mutex.lock();
		self.channel.send(e);
		unsafe {
			event_signal::signal_waiters(&mut *self.new_event_waiters.get());
		}
	}

	/// Flushes the channel
	///
	/// Makes the currently sent un-flushed events readable.
	///
	/// This drops all previously flushed events, making them unreadable.
	///
	/// This also wakes any threads waiting for a flush via
	/// [`SyncEventReader::wait_flushed`].
	///
	/// Is is adviced to let one of the readers initiate the flush with
	/// [`SyncEventReader::flush_channel`],
	/// as they are controlling consumation of events.
	pub fn flush(&self)
	{
		let _lock = self.flush_mutex.write();
		self.channel.flush();
		unsafe {
			event_signal::signal_waiters(&mut *self.flushed_waiters.get());
		}
	}

	/// Creates a reader for this channel
	pub fn get_reader(&self) -> SyncEventReader<T>
	{
		SyncEventReader {
			read_events: UnsafeCell::new(1), // avoid stupid stuff when read=0
			channel: self,
			_not_send_sync: PhantomData,
		}
	}

	// expects the write_mutex to already be locked by this thread
	// only called from reader.wait_new()
	fn has_new_events(&self) -> bool
	{
		unsafe {
			let buffer = self.channel.readable_buffer.get();
			match *buffer {
				ReadableEventBuffer::A => !(*self.channel.events_b.get()).is_empty(),
				ReadableEventBuffer::B => !(*self.channel.events_a.get()).is_empty(),
			}
		}
	}
}

impl<'a, T> SyncEventReader<'a, T>
{
	/// Reads all unread events from this channel
	///
	/// Giver an `Iterator` over the currently flushed events.
	///
	/// Becaus of how this is setup, it reads all flushed events, or none at all
	/// if the flushed events have been read by this reader.
	pub fn read(&self) -> impl Iterator<Item = &T>
	{
		let read_lock = self.channel.flush_mutex.read();

		if !self.has_unread() {
			return EventIterator {
				read_lock,
				iterator: [].iter(),
			};
		}

		let channel = &self.channel.channel;
		unsafe {
			let readable_buffer = channel.readable_buffer.get();
			let read_events = self.read_events.get();
			let iterator = match *readable_buffer {
				ReadableEventBuffer::A => {
					let start_idx_a = *channel.start_idx_a.get();
					*read_events = start_idx_a + 1;
					(*channel.events_a.get()).iter()
				}
				ReadableEventBuffer::B => {
					let start_idx_b = *channel.start_idx_b.get();
					*read_events = start_idx_b + 1;
					(*channel.events_b.get()).iter()
				}
			};
			EventIterator {
				read_lock,
				iterator,
			}
		}
	}

	// expects write_mutex to already be locked
	fn has_unread(&self) -> bool
	{
		let channel = &self.channel.channel;
		unsafe {
			let readable_buffer = channel.readable_buffer.get();
			let read_events = self.read_events.get();
			let start_idx = match *readable_buffer {
				ReadableEventBuffer::A => channel.start_idx_a.get(),
				ReadableEventBuffer::B => channel.start_idx_b.get(),
			};
			*read_events <= *start_idx
		}
	}

	/// Initiates a flush on the reader's connected channel
	///
	/// It is adviced to use this for flushing. Read [`EventChannel::flush`]
	/// for a descripion of the behaviour regarding flushing.
	pub fn flush_channel(&self) { self.channel.flush(); }

	/// Waits for un-flushed events to be present
	///
	/// If there already are un-flushed events, this returns directly,
	/// as there are new events that can be flushed.
	///
	/// If no events are present, the thread will halt and wake when the
	/// next [`SyncEventChannel::send`] occurs.
	pub fn wait_new(&self)
	{
		let _lock = self.channel.write_mutex.lock();
		if self.channel.has_new_events() {
			return;
		}

		unsafe {
			let p = event_signal::add_waiter(&mut *self.channel.new_event_waiters.get());
			drop(_lock);
			p.park();
		}
	}

	/// Waits for flushed un-read events to be present
	///
	/// This returns directly if the read has not read the currently flushed
	/// events
	///
	/// If the reader has read current events, it will halt and wake when the
	/// next [`SyncEventChannel::flush`] occurs.
	///
	/// Note: This may lead to a deadlock if this thread is responsible for
	/// flushing, but you already knew that.
	pub fn wait_flushed(&self)
	{
		let _lock = self.channel.flush_mutex.write();
		if self.has_unread() {
			return;
		}

		unsafe {
			let p = event_signal::add_waiter(&mut *self.channel.flushed_waiters.get());
			drop(_lock);
			p.park();
		}
	}
}

struct EventIterator<'a, T>
{
	#[allow(dead_code)]
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
	use std::time::Duration;

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
		let events = reader.read().collect::<Vec<&TestEvent>>();
		assert_eq!(events, Vec::<&TestEvent>::new(), "initial events empty");

		test_channel.send(event0);
		test_channel.flush();

		let events = reader.read().collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			[&TestEvent { data: 0 }],
			"reader can read flushed event0"
		);

		test_channel.send(event1);
		test_channel.flush();

		let events = reader.read().collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			[&TestEvent { data: 1 }],
			"We only retain the events most recently flushed, event0 is then dropped"
		);

		let reader2 = test_channel.get_reader();
		let events = reader2.read().collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			[&TestEvent { data: 1 }],
			"We only retain the events most recently flushed, reader2 reads after event0 has been \
			 dropped"
		);
		let events = reader2.read().collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			Vec::<&TestEvent>::new(),
			"Cannot read event multiple times"
		);

		test_channel.flush();
		let events = reader2.read().collect::<Vec<&TestEvent>>();
		assert_eq!(events, Vec::<&TestEvent>::new());
	}

	#[test]
	fn sync_001()
	{
		let channel = Arc::new(SyncEventChannel::<TestEvent>::new());
		let total = Arc::new(Mutex::new(0));
		let total_loc = Arc::clone(&total);
		let c = Arc::clone(&channel);
		let emitter1 = thread::spawn(move || {
			for i in 0..10 {
				let event = TestEvent { data: 2 * i };
				c.send(event);
				thread::sleep(Duration::from_millis(1));
			}
		});

		let rec1 = thread::spawn(move || {
			let rec = channel.get_reader();
			loop {
				thread::sleep(Duration::from_millis(5));
				let mut got_events = false;
				rec.flush_channel();
				for e in rec.read() {
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

	#[test]
	/// test waiting for new events
	fn sync_002()
	{
		let channel = Arc::new(SyncEventChannel::<()>::new());
		let total_loc = Arc::new(Mutex::new(0));

		let total = Arc::clone(&total_loc);
		let c = Arc::clone(&channel);
		let emitter1 = thread::spawn(move || {
			for i in 1..11 {
				c.send(());
				thread::sleep(Duration::from_millis(2));
				assert!(total.lock().eq(&i));
			}
		});

		let total = Arc::clone(&total_loc);
		let rec1 = thread::spawn(move || {
			let rec = channel.get_reader();
			for _ in 1..11 {
				rec.wait_new();
				rec.flush_channel();
				for _ in rec.read() {
					total.lock().add_assign(1);
				}
			}
		});

		emitter1.join().unwrap();
		rec1.join().unwrap();
		let total = total_loc.lock();
		assert!(total.eq(&10));
	}

	#[test]
	/// test waiting for flush
	fn sync_003()
	{
		let channel = Arc::new(SyncEventChannel::<()>::new());
		let total_loc = Arc::new(Mutex::new(0));

		let total = Arc::clone(&total_loc);
		let c = Arc::clone(&channel);
		let emitter1 = thread::spawn(move || {
			thread::sleep(Duration::from_millis(5)); // TODO shouldn't need
			for i in 1..11 {
				c.send(());
				thread::sleep(Duration::from_millis(5));

				// two readers should update when main thread flushes
				assert!(total.lock().eq(&(i * 2)));
			}
		});

		let c = Arc::clone(&channel);
		let total = Arc::clone(&total_loc);
		let rec1 = thread::spawn(move || {
			let rec = c.get_reader();
			for _ in 1..11 {
				rec.wait_flushed();
				for _ in rec.read() {
					total.lock().add_assign(1);
				}
			}
		});

		let c = Arc::clone(&channel);
		let total = Arc::clone(&total_loc);
		let rec2 = thread::spawn(move || {
			let rec = c.get_reader();
			for _ in 1..11 {
				rec.wait_flushed();
				for _ in rec.read() {
					total.lock().add_assign(1);
				}
			}
		});

		let rec = channel.get_reader();
		for _ in 1..11 {
			rec.wait_new();
			rec.flush_channel();
		}

		emitter1.join().unwrap();
		rec1.join().unwrap();
		rec2.join().unwrap();
		let total = total_loc.lock();
		assert!(total.eq(&20));
	}
}
