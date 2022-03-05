//! Event system in the LY engine
//!
//! The crate provides functionality to send event via "channels", inspired
//! by rust channels, and event signals to synchonize threads.
//!
//! The most important module is [channel], which is probably why you are here.

mod event_channel;
mod event_signal;
mod event_types;
mod sync_event_channel;

/// Module for sending signal events to waiting threads
///
/// Currently this module only contains [`signal::SignalEvent`]
///
/// The signal is [`Sync`], but needs to be wrapped in something
/// to acually be shared between threads, like [`std::sync::Arc`].
///
/// Waiting always parks current thread until next, whether or not
/// the event has been signaled before.
///
/// ### Example
/// ```
/// # use ly_events::signal::SignalEvent;
/// # use std::sync::Arc;
/// # use std::sync::atomic::{AtomicBool, Ordering};
/// # use std::thread;
/// # use std::time::Duration;
/// let signal = Arc::new(SignalEvent::new());
/// let s = Arc::clone(&signal);
/// let running = Arc::new(AtomicBool::new(false));
/// let r = Arc::clone(&running);
///
/// let adder = thread::spawn(move || {
/// 	// Do stuff before wait
/// 	s.wait();
/// 	r.store(true, Ordering::Relaxed);
/// 	s.signal();
/// 	// Do stuff after wake
/// });
///
/// // do some stuff so that the other thread have entered wait()
/// thread::sleep(Duration::from_millis(2));
///
/// assert_eq!(running.load(Ordering::Relaxed), false);
/// signal.signal(); // Wake waiting thread
/// signal.wait();   // Wait for thread to continue
/// assert_eq!(running.load(Ordering::Relaxed), true);
/// ```
pub mod signal
{
	pub use super::event_signal::SignalEvent;
}

/// Module for sending events through channels
///
/// TODO
/// * Remove the basic `EventChannel`, the whole point is to send stuff
/// between threads
///
/// Event channels are instantiated for some event-type. They are read
/// by readers and written by writers that are created by those instantiated
/// channels. In order to read events, they first need to be flushed.
/// Once read, events are not read again.
///
/// There are no type constraints regarding the event type.
///
/// A writer/reader has a borrow of the channel it reads, the one that created
/// the reader. This is to ensure that the reader always has a channel to read
/// from, and to enable a more ergonomic API.
///
/// A single channel may have multiple readers. Reading an event
/// does not consume it for other readers.
///
/// ### Example single-threaded event flow
///
/// ```
/// # use ly_events::channel::EventChannel;
/// #[derive(Debug, PartialEq, Eq, Clone)]
/// struct TestEvent { data: usize, }
///
/// let test_channel = EventChannel::<TestEvent>::default();
/// let event = TestEvent { data: 42 };
/// let event_clone = event.clone();
///
/// let writer = test_channel.get_writer();
/// let reader = test_channel.get_reader();
///
/// let events = reader.read().collect::<Vec<&TestEvent>>();
/// assert_eq!(events, Vec::<&TestEvent>::default(),
/// 	"initial events empty");
///
/// writer.send(event);
///
/// let events = reader.read().collect::<Vec<&TestEvent>>();
/// assert_eq!(events, Vec::<&TestEvent>::default(),
/// 	"still emply after send");
///
/// test_channel.flush();
///
/// let events = reader.read().collect::<Vec<&TestEvent>>();
/// assert_eq!(events, [&event_clone],
/// 	"reader can read flushed event");
///
/// let events = reader.read().collect::<Vec<&TestEvent>>();
/// assert_eq!(events, Vec::<&TestEvent>::default(),
/// 	"cannot read twice");
/// ```
///
/// ## Sync options
///
/// The [`SyncEventChannel`](channel::SyncEventChannel) is the `Sync`
/// alternative, enabling e.g. multithreading. The interface is the same
/// as for the single-threaded event channel. In fact, the sync channel is just
/// a wrapper adding additional sync logic.
///
/// With the current implementation, the reader has a borrow of the channel.
/// This makes it necessary for the reading thread to also have a clone of
/// the `Arc` (or equivalent) holding the channel, because of lifetimes. In
/// order to enforce this, the sync reader is not `Send` nor `Sync`.
///
/// For unscoped threads this is one way to work:
/// ```
/// # struct TestEvent { data: usize, }
/// # use std::thread;
/// # use std::sync::Arc;
/// # use ly_events::channel::SyncEventChannel;
/// let channel = Arc::new(SyncEventChannel::<TestEvent>::default());
///
/// let c = Arc::clone(&channel);
/// thread::spawn(move || {
/// 	let reader = c.get_reader();
/// 	reader.flush_channel();
/// 	for event in reader.read() {
/// 		// do stuff
/// 	}
/// });
/// ```
///
/// ### Waiting for events
///
/// The [`SyncEventReader`](channel::SyncEventReader) has some synchronization
/// methods to wait for the channel to reach a certain state.
/// * [`wait_new`](channel::SyncEventReader::wait_new), to wait for default
///   events to be sent to the channel.
/// * [`wait_flushed`](channel::SyncEventReader::wait_flushed), to wait for
///   someone else to flush the channel.
///
/// In order to wait for multiple readers, the readers implement the
/// [`EventWaiter`](channel::EventWaiter) trait so that
/// [`wait_any_new`](channel::wait_any_new) can be used. The following example
/// will read event if the channel of either `reader1` or `reader2` has
/// unflushed events.
/// ```
/// # struct TestEvent { data: usize, }
/// # use std::thread;
/// # use std::sync::Arc;
/// # use ly_events::channel::{SyncEventChannel, EventWaiter, wait_any_new};
/// let channel1 = Arc::new(SyncEventChannel::<TestEvent>::default());
/// let channel2 = Arc::new(SyncEventChannel::<TestEvent>::default());
///
/// let c = Arc::clone(&channel1);
/// thread::spawn(move || {
///     c.get_writer().send(TestEvent { data: 42 });
/// });
///
/// let reader1 = channel1.get_reader();
/// let reader2 = channel2.get_reader();
/// let readers: [&dyn EventWaiter; 2] = [&reader1, &reader2];
///
/// wait_any_new(&readers);
///
/// // read events from reader1 and reader2
/// ```
///
/// Note that [`wait_any_new`](channel::wait_any_new) uses dynamic dispatch,
/// so it will be more performant to wait on a specific event reader.
pub mod channel
{
	pub use super::event_channel::*;
	pub use super::sync_event_channel::*;
}

/// Provides event types to be used with the LY engine
///
/// TODO: Consider having a channel per event, and not
/// grouped together like this. With some abstraction
/// to have a "group reader" this might be more effective,
/// as there are less/no irrelevant events for a reader.
pub mod types
{
	pub use super::event_types::*;
}

#[cfg(test)]
mod tests
{
	use super::channel::*;
	use parking_lot::Mutex;
	use std::ops::AddAssign;
	use std::sync::Arc;
	use std::thread;
	use std::time::Duration;

	#[derive(Debug, Default, PartialEq, Eq)]
	struct TestEvent
	{
		data: usize,
	}

	#[test]
	fn channel_flow()
	{
		let test_channel = EventChannel::<TestEvent>::default();
		let event0 = TestEvent { data: 0 };
		let event1 = TestEvent { data: 1 };

		let writer = test_channel.get_writer();
		let reader = test_channel.get_reader();
		let events = reader.read().collect::<Vec<&TestEvent>>();
		assert_eq!(events, Vec::<&TestEvent>::default(), "initial events empty");

		writer.send(event0);
		test_channel.flush();

		let events = reader.read().collect::<Vec<&TestEvent>>();
		assert_eq!(
			events,
			[&TestEvent { data: 0 }],
			"reader can read flushed event0"
		);

		writer.send(event1);
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
			Vec::<&TestEvent>::default(),
			"Cannot read event multiple times"
		);

		test_channel.flush();
		let events = reader2.read().collect::<Vec<&TestEvent>>();
		assert_eq!(events, Vec::<&TestEvent>::default());
	}

	#[test]
	fn sync_001()
	{
		let channel = Arc::new(SyncEventChannel::<TestEvent>::default());
		let total = Arc::new(Mutex::new(0));
		let total_loc = Arc::clone(&total);
		let c = Arc::clone(&channel);
		let emitter1 = thread::spawn(move || {
			let writer = c.get_writer();
			for i in 0..10 {
				let event = TestEvent { data: 2 * i };
				writer.send(event);
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
		let channel = Arc::new(SyncEventChannel::<()>::default());
		let total_loc = Arc::new(Mutex::new(0));

		let total = Arc::clone(&total_loc);
		let c = Arc::clone(&channel);
		let emitter1 = thread::spawn(move || {
			let writer = c.get_writer();
			for i in 1..11 {
				writer.send(());
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
		let channel = Arc::new(SyncEventChannel::<()>::default());
		let total_loc = Arc::new(Mutex::new(0));

		let total = Arc::clone(&total_loc);
		let c = Arc::clone(&channel);
		let emitter1 = thread::spawn(move || {
			let writer = c.get_writer();
			thread::sleep(Duration::from_millis(5)); // TODO shouldn't need
			for i in 1..11 {
				writer.send(());
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

	#[test]
	/// test dropping the writer
	fn sync_004()
	{
		let channel = Arc::new(SyncEventChannel::<()>::default());
		let total_loc = Arc::new(Mutex::new(0));

		let total = Arc::clone(&total_loc);
		let c = Arc::clone(&channel);
		let emitter1 = thread::spawn(move || {
			let writer = c.get_writer();
			for i in 1..11 {
				writer.send(());
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
}
