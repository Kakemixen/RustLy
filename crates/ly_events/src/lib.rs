//! Event system in the LY engine
//!
//! The crate provides functionality to send event via "channels", inspired
//! by rust channels, and event signals to synchonize threads.
//!
//! The most important module is [channel], which is probably why you are here.

mod event_channel;
mod event_signal;
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
/// Event channels are instantiated for some event-type, and are read
/// by readers that are created by those instantiated channels.
/// In order to read events, they first need to be flushed.
/// Once read, events are not read again.
///
/// A reader has a borrow of the channel it reads, the one that created the
/// reader. This is to ensure that the reader always has a channel to read from,
/// and to enable a more ergonomic API.
///
/// A single channel may have multiple readers. Reading an event
/// does not consume it for other readers.
///
/// There is a sync alternative [`channel::SyncEventChannel`],
/// which is probably the most useful one.
///
/// ### Example single-threaded event flow
///
/// ```
/// # use ly_events::channel::EventChannel;
/// #[derive(Debug, PartialEq, Eq, Clone)]
/// struct TestEvent { data: usize, }
///
/// let test_channel = EventChannel::<TestEvent>::new();
/// let event = TestEvent { data: 42 };
/// let event_clone = event.clone();
///
/// let reader = test_channel.get_reader();
///
/// let events = reader.read().collect::<Vec<&TestEvent>>();
/// assert_eq!(events, Vec::<&TestEvent>::new(),
/// 	"initial events empty");
///
/// test_channel.send(event);
///
/// let events = reader.read().collect::<Vec<&TestEvent>>();
/// assert_eq!(events, Vec::<&TestEvent>::new(),
/// 	"still emply after send");
///
/// test_channel.flush();
///
/// let events = reader.read().collect::<Vec<&TestEvent>>();
/// assert_eq!(events, [&event_clone],
/// 	"reader can read flushed event");
///
/// let events = reader.read().collect::<Vec<&TestEvent>>();
/// assert_eq!(events, Vec::<&TestEvent>::new(),
/// 	"cannot read twice");
/// ```
pub mod channel
{
	pub use super::event_channel::*;
	pub use super::sync_event_channel::*;
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
